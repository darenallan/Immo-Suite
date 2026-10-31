//! Routes et intergiciels HTTP.
//!
//! Le journal d'accès n'écrit que des champs de la liste blanche (`request_id`, `method`, `route`, `status`, `duration_ms`,
//! `version`) : jamais de corps, d'en-tête, de chaîne de requête ni de chemin brut (CODING_RULES.md §7 n° 13, SECURITY.md §7.7).
//! `route` est le **modèle** de la route (`/health`), jamais l'adresse demandée.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::probe::Probe;

/// État partagé : les sondes de `/ready` et la version publiée dans le journal.
pub struct AppState {
    pub probes: Vec<Arc<dyn Probe>>,
    pub version: String,
}

/// Construit le routeur. À servir avec `into_make_service_with_connect_info::<SocketAddr>()`.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .layer(middleware::from_fn_with_state(state.clone(), access_log))
        .with_state(state)
}

/// Réponse de `/health`.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct StatusBody {
    /// Toujours `ok` ; aucune donnée.
    pub status: &'static str,
}

/// Sonde externe : répond `{"status":"ok"}` et rien d'autre (API_CONTRACTS.md §7.17).
#[utoipa::path(get, path = "/health", responses((status = 200, description = "Le service répond", body = StatusBody)))]
pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// Disponibilité de la base et du stockage, réservée au réseau interne : 404 depuis l'extérieur, 503 si une ressource manque.
#[utoipa::path(get, path = "/ready", responses((status = 200, description = "Base et stockage disponibles", body = StatusBody), (status = 503, description = "Une ressource manque"), (status = 404, description = "Hors réseau interne")))]
pub async fn ready(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
) -> Response {
    // Défense en profondeur : le proxy ne route pas /ready, et une requête relayée (X-Forwarded-For) est traitée comme externe.
    let relayed = request.headers().contains_key("x-forwarded-for") || request.headers().contains_key("forwarded");
    if relayed || !is_internal(peer.ip()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mut all_ok = true;
    for probe in &state.probes {
        if !probe.check().await {
            all_ok = false;
            tracing::warn!(probe = probe.name(), "sonde de disponibilité en échec");
        }
    }
    if all_ok {
        Json(json!({ "status": "ready" })).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unavailable" })),
        )
            .into_response()
    }
}

/// Adresse du réseau interne : bouclage, réseaux privés IPv4, adresses locales de lien, adresses locales uniques IPv6.
fn is_internal(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || (v6.segments().first().is_some_and(|s| s & 0xfe00 == 0xfc00)),
    }
}

/// Journal d'accès, identifiant de requête et en-têtes de réponse communs.
async fn access_log(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string();
    let method = request.method().to_string();
    // modèle de route, jamais le chemin brut : un chemin peut contenir un identifiant ou un jeton
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_string(), |m| m.as_str().to_string());
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&request_id) {
        headers.insert(HeaderName::from_static("x-request-id"), v);
    }
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-store"),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(request_id = %request_id, method = %method, route = %route, status = response.status().as_u16(), duration_ms, version = %state.version, "requête");
    response
}
