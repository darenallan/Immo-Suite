//! Routes et intergiciels HTTP.
//!
//! Le journal d'accès n'écrit que des champs de la liste blanche (`request_id`, `method`, `route`, `status`, `duration_ms`,
//! `version`) : jamais de corps, d'en-tête, de chaîne de requête ni de chemin brut (CODING_RULES.md §7 n° 13, SECURITY.md §7.7).
//! `route` est le **modèle** de la route (`/health`), jamais l'adresse demandée.
//!
//! Chaque réponse porte les en-têtes de sécurité de SECURITY.md §7.3 et `X-Request-Id` (code d'incident, `INC-…`).

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, DefaultBodyLimit, MatchedPath, Request, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use immo_app::identity::service::IdentityService;
use serde_json::json;

use crate::extract::{Ctx, client_ip, session_token};
use crate::limiter::RateLimiter;
use crate::probe::Probe;
use crate::problem::{ApiError, RequestId, new_request_id};

/// Plafonds généraux (API_CONTRACTS.md §4.7, valeurs proposées [Hypothèse]) : par session et par adresse, sur une minute.
const SESSION_LIMIT_PER_MINUTE: usize = 300;
const IP_LIMIT_PER_MINUTE: usize = 600;
/// Corps de requête : 16 Ko suffisent aux routes d'identité.
const MAX_BODY_BYTES: usize = 16 * 1024;

/// État partagé : sondes de `/ready`, identité, réglages de l'API.
pub struct AppState {
    pub probes: Vec<Arc<dyn Probe>>,
    pub version: String,
    pub identity: Option<IdentityService>,
    /// Origines acceptées dans `Origin` pour les requêtes qui écrivent.
    pub allowed_origins: Vec<String>,
    pub trust_proxy: bool,
    pub min_client_version: String,
    pub limiter: RateLimiter,
    /// Adresse publique de l'application, sans barre finale : base des liens d'invitation et de réinitialisation.
    pub public_base_url: String,
    /// Hors production : un événement d'usage écarté répond `422` au lieu de `204` (F39-observation règle 4).
    pub strict_events: bool,
    /// Route d'essai `GET /api/_test/server-error` (jamais en production, F39-observation §4).
    pub enable_test_routes: bool,
    /// Stockage des fichiers (PDF des quittances, étape 08) ; absent : les PDF ne sont pas servis.
    pub file_store: Option<immo_app::platform::file_store::DiskFileStore>,
}

impl AppState {
    /// État minimal (sondes seulement) : sans identité, les routes d'API répondent `server.unexpected`.
    pub fn new(probes: Vec<Arc<dyn Probe>>, version: String) -> Self {
        Self {
            probes,
            version,
            identity: None,
            allowed_origins: vec![],
            trust_proxy: false,
            min_client_version: "0.0.0".to_string(),
            limiter: RateLimiter::new(),
            public_base_url: "http://localhost:8080".to_string(),
            strict_events: false,
            enable_test_routes: false,
            file_store: None,
        }
    }

    /// Stockage des fichiers (`FILE_STORE_DIR`).
    pub fn with_file_store(mut self, store: Option<immo_app::platform::file_store::DiskFileStore>) -> Self {
        self.file_store = store;
        self
    }

    /// Événements écartés : `422` (développement et essai) au lieu de `204` (production).
    pub fn with_strict_events(mut self, strict: bool) -> Self {
        self.strict_events = strict;
        self
    }

    /// Monte la route d'essai qui provoque une erreur 500 réelle.
    pub fn with_test_routes(mut self, enabled: bool) -> Self {
        self.enable_test_routes = enabled;
        self
    }

    /// Adresse publique (`PUBLIC_BASE_URL`) : sert à composer les liens remis au gérant.
    pub fn with_public_base_url(mut self, url: &str) -> Self {
        self.public_base_url = url.trim_end_matches('/').to_string();
        self
    }

    pub fn with_identity(
        mut self,
        identity: IdentityService,
        allowed_origins: Vec<String>,
        trust_proxy: bool,
        min_client_version: String,
    ) -> Self {
        self.identity = Some(identity);
        self.allowed_origins = allowed_origins;
        self.trust_proxy = trust_proxy;
        self.min_client_version = min_client_version;
        self
    }
}

/// Construit le routeur. À servir avec `into_make_service_with_connect_info::<SocketAddr>()`.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .merge(crate::api::routes())
        .merge(crate::api_receipt::public_routes())
        .merge(crate::api_observation::routes(state.enable_test_routes))
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .layer(middleware::from_fn_with_state(state.clone(), access_log))
        .with_state(state)
}

/// Toute adresse inconnue : 404 `resource.not_found` au format du contrat (dont `POST /api/auth/register`, absente au pilote).
async fn not_found(ctx: Ctx) -> ApiError {
    ctx.err(ApiError::not_found())
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

/// Compare deux versions `a.b.c` numériquement ; une version illisible n'est jamais « plus ancienne » (pas de blocage à tort).
fn version_older(client: &str, minimum: &str) -> bool {
    let parse = |v: &str| -> Option<Vec<u64>> { v.trim().split('.').map(|p| p.parse::<u64>().ok()).collect() };
    match (parse(client), parse(minimum)) {
        (Some(c), Some(m)) => {
            let n = c.len().max(m.len());
            let get = |v: &Vec<u64>, i: usize| v.get(i).copied().unwrap_or(0);
            (0..n)
                .map(|i| (get(&c, i), get(&m, i)))
                .find(|(a, b)| a != b)
                .is_some_and(|(a, b)| a < b)
        }
        _ => false,
    }
}

/// Plafonds de débit par adresse et par session, et version minimale du client (`X-Client-Version`, API_CONTRACTS.md §1.3).
async fn guard(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    if path == "/health" || path == "/ready" {
        return next.run(request).await;
    }
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map_or_else(new_request_id, |r| r.0.clone());
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip());
    let ip: Option<IpAddr> = client_ip(request.headers(), peer, state.trust_proxy);
    let window = Duration::from_secs(60);
    let mut limited = None;
    if let Some(ip) = ip
        && let Err(wait) = state.limiter.hit(&format!("ip:{ip}"), IP_LIMIT_PER_MINUTE, window)
    {
        limited = Some(wait);
    }
    if limited.is_none()
        && let Some(token) = session_token(request.headers())
        && let Err(wait) = state
            .limiter
            .hit(&format!("s:{token}"), SESSION_LIMIT_PER_MINUTE, window)
    {
        limited = Some(wait);
    }
    if let Some(wait) = limited {
        return ApiError::request_rate_limited(wait)
            .with_request_id(&request_id)
            .into_response();
    }
    if let Some(client) = request.headers().get("x-client-version").and_then(|v| v.to_str().ok())
        && version_older(client, &state.min_client_version)
    {
        return ApiError::upgrade_required()
            .with_request_id(&request_id)
            .into_response();
    }
    next.run(request).await
}

/// En-têtes de sécurité de SECURITY.md §7.3, valables pour toute réponse (API_CONTRACTS.md §4.8).
const SECURITY_HEADERS: [(&str, &str); 9] = [
    ("strict-transport-security", "max-age=31536000; includeSubDomains"),
    (
        "content-security-policy",
        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'; object-src 'none'",
    ),
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    (
        "permissions-policy",
        "camera=(), microphone=(), geolocation=(), payment=()",
    ),
    ("cross-origin-opener-policy", "same-origin"),
    ("cross-origin-resource-policy", "same-origin"),
    ("x-frame-options", "DENY"),
    ("cache-control", "no-store"),
];

/// Journal d'accès, identifiant de requête et en-têtes de réponse communs.
async fn access_log(State(state): State<Arc<AppState>>, mut request: Request, next: Next) -> Response {
    let started = Instant::now();
    let request_id = new_request_id();
    request.extensions_mut().insert(RequestId(request_id.clone()));
    let method = request.method().to_string();
    let session = session_token(request.headers());
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
    if let Ok(v) = HeaderValue::from_str(&state.min_client_version) {
        headers.insert(HeaderName::from_static("x-min-client-version"), v);
    }
    for (name, value) in SECURITY_HEADERS {
        headers.insert(HeaderName::from_static(name), HeaderValue::from_static(value));
    }
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    if let Some(observed) = Observed::of(&response, duration_ms) {
        observe_response(&state, session, &route, &method, &request_id, observed).await;
    }
    tracing::info!(request_id = %request_id, method = %method, route = %route, status = response.status().as_u16(), duration_ms, version = %state.version, "requête");
    response
}

/// Réponse lente en millisecondes (F39-observation, seuil proposé).
const SLOW_RESPONSE_MS: u64 = 2000;
/// Au plus 2 s d'attente pour écrire l'observation d'une réponse : jamais plus (règle 11).
const OBSERVE_TIMEOUT: Duration = Duration::from_secs(2);

/// Ce que la réponse apprend à l'observation : copié avant tout `await` (le corps d'une réponse n'est pas `Sync`).
struct Observed {
    status: u16,
    server_error: Option<crate::problem::ServerErrorInfo>,
    denied: Option<crate::problem::DeniedInfo>,
    duration_ms: u64,
}

impl Observed {
    fn of(response: &Response, duration_ms: u64) -> Option<Self> {
        let server_error = response.extensions().get::<crate::problem::ServerErrorInfo>().cloned();
        let denied = response.extensions().get::<crate::problem::DeniedInfo>().cloned();
        if server_error.is_none() && denied.is_none() && duration_ms < SLOW_RESPONSE_MS {
            return None;
        }
        Some(Self {
            status: response.status().as_u16(),
            server_error,
            denied,
            duration_ms,
        })
    }
}

/// Observation après la réponse (F39-observation §2 règles 11 à 13) : erreur inattendue agrégée par empreinte, refus de droit,
/// réponse lente. Ne coûte rien à une réponse ordinaire. Un échec est journalisé et ne change jamais la réponse.
async fn observe_response(
    state: &AppState,
    token: Option<String>,
    route: &str,
    method: &str,
    request_id: &str,
    observed: Observed,
) {
    use immo_app::observation::{errors, events::ProductEvent};
    let report = observed
        .server_error
        .as_ref()
        .map(|info| errors::server_error(route, method, observed.status, request_id, &state.version, &info.frames));
    let session = match (&token, state.identity.as_ref()) {
        (Some(token), Some(identity)) => identity
            .authenticate(token)
            .await
            .ok()
            .map(|auth| (token, identity, auth)),
        _ => None,
    };
    let Some((token, identity, auth)) = session else {
        // sans organisation identifiable : journal seulement, avec l'empreinte et l'identifiant de requête (règle 12)
        if let Some(report) = report {
            tracing::error!(request_id = %request_id, route = %route, fingerprint = %report.fingerprint, "erreur inattendue sans organisation");
        }
        return;
    };
    let work = async {
        if let Some(report) = &report {
            identity.record_error(&auth, report).await;
        }
        if let (Some(info), Some(role)) = (&observed.denied, auth.role) {
            identity
                .observe(
                    &auth,
                    token,
                    ProductEvent::permission_denied(&info.action, role.as_str()),
                )
                .await;
        }
        if observed.duration_ms >= SLOW_RESPONSE_MS {
            identity
                .observe(
                    &auth,
                    token,
                    ProductEvent::slow_response(route, method, observed.status, observed.duration_ms),
                )
                .await;
        }
    };
    if tokio::time::timeout(OBSERVE_TIMEOUT, work).await.is_err() {
        tracing::error!(request_id = %request_id, "observation : délai dépassé");
    }
}

#[cfg(test)]
mod tests {
    use super::version_older;

    #[test]
    fn comparaison_des_versions_du_client() {
        assert!(version_older("0.9.0", "1.0.0"));
        assert!(version_older("1.2", "1.2.1"));
        assert!(!version_older("1.2.1", "1.2"));
        assert!(!version_older("2.0.0", "1.9.9"));
        assert!(!version_older("1.0.0", "1.0.0"));
        assert!(
            !version_older("abc", "1.0.0"),
            "une version illisible n'est pas bloquée"
        );
        assert!(!version_older("1.0.0", "0.0.0"));
    }
}
