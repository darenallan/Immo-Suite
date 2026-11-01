//! Routes d'observation de l'étape 04 (`API_CONTRACTS.md` §7.15 et §7.16, F39-observation, DP-90) : événements d'usage par lots,
//! erreurs du front, signalements « Signaler un problème ». Même domaine, aucun cookie en plus, aucun service tiers.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use immo_app::identity::authorize::Action;
use immo_app::identity::service::IdentityService;
use immo_app::observation::errors;
use immo_app::observation::events::{self, MAX_BATCH_BYTES, ProductEvent};
use immo_app::observation::feedback::{self, FeedbackError, FeedbackInput, Reporter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::extract::{ApiJson, Session};
use crate::http::AppState;
use crate::problem::ApiError;

/// Limites par session / par utilisateur (F39-observation §4, valeurs proposées [Hypothèse]).
const EVENTS_PER_MINUTE: usize = 30;
const CLIENT_ERRORS_PER_MINUTE: usize = 10;
const FEEDBACK_PER_HOUR: usize = 5;

pub fn routes(test_routes: bool) -> Router<Arc<AppState>> {
    // 8 Kio au plus pour un lot d'événements ou une erreur du front : au-delà, 413 `request.too_large`
    let small = Router::new()
        .route("/api/events", post(post_events))
        .route("/api/client-errors", post(post_client_error))
        .layer(DefaultBodyLimit::max(MAX_BATCH_BYTES));
    let mut router = Router::new().merge(small).route("/api/feedback", post(post_feedback));
    if test_routes {
        router = router.route("/api/_test/server-error", get(test_server_error));
    }
    router
}

fn identity(state: &AppState, session: &Session) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| session.ctx.unexpected())
}

fn limit(state: &AppState, key: &str, max: usize, window: Duration) -> Result<(), ApiError> {
    state
        .limiter
        .hit(key, max, window)
        .map_err(ApiError::request_rate_limited)
}

fn client_version(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-client-version")
        .and_then(|v| v.to_str().ok())
        .filter(|v| events::valid_client_version(v))
        .map(str::to_string)
}

// ───────── POST /api/events ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EventsPayload {
    /// Jusqu'à 20 événements `{ name, properties, client_version }` ; les suivants sont écartés et comptés.
    #[schema(value_type = Vec<Object>)]
    pub events: Vec<serde_json::Value>,
}

/// Événements d'usage du front, par lots (20 et 8 Kio au plus). Liste blanche fermée : un événement inconnu, d'origine serveur ou
/// aux propriétés hors liste est écarté **et compté** ; la réponse reste `204` en production, `422` en développement et en essai.
#[utoipa::path(
    post,
    path = "/api/events",
    request_body = EventsPayload,
    responses(
        (status = 204, description = "Lot reçu (certains événements ont pu être écartés et comptés)"),
        (status = 413, description = "request.too_large"),
        (status = 422, description = "validation.failed : hors production, un événement a été écarté"),
        (status = 429, description = "rate_limit.exceeded")
    )
)]
pub async fn post_events(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    ApiJson(body): ApiJson<EventsPayload>,
) -> Result<Response, ApiError> {
    session.require(Action::EventsCreate)?;
    limit(
        &state,
        &format!("events:{}", session.token),
        EVENTS_PER_MINUTE,
        Duration::from_secs(60),
    )?;
    let svc = identity(&state, &session)?;
    let Some(org) = &session.auth.org else {
        return Err(session.ctx.err(ApiError::organization_required()));
    };
    let (accepted, report) = events::parse_batch(&body.events);
    let ctx = svc.event_context(&session.auth, &session.token, client_version(&headers));
    if !body.events.is_empty() {
        let mut tx = svc
            .db()
            .begin(org.id, session.auth.user.id)
            .await
            .map_err(|_| session.ctx.unexpected())?;
        events::emit_batch(&mut tx, &ctx, &accepted, &report)
            .await
            .map_err(|_| session.ctx.unexpected())?;
        tx.commit().await.map_err(|_| session.ctx.unexpected())?;
    }
    if state.strict_events && report.discarded_total() > 0 {
        return Err(session.ctx.err(ApiError::validation_reason("event_discarded")));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ───────── POST /api/client-errors ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientErrorPayload {
    /// Message de l'erreur JavaScript (200 caractères au plus après nettoyage ; jamais le texte de l'écran).
    pub message: String,
    /// Écran (`S00` à `S26`).
    pub screen: String,
    pub client_version: String,
    /// Jusqu'à 5 identifiants de requête (`INC-…`).
    pub request_ids: Vec<String>,
}

/// Code du type d'erreur JavaScript, lu au début du message (liste fermée de `client_error`).
fn client_error_code(message: &str) -> &'static str {
    let lower = message.trim_start().to_ascii_lowercase();
    if lower.starts_with("unhandled_rejection") || lower.starts_with("unhandled rejection") {
        return "unhandled_rejection";
    }
    for (prefix, code) in [
        ("typeerror", "TypeError"),
        ("rangeerror", "RangeError"),
        ("referenceerror", "ReferenceError"),
        ("syntaxerror", "SyntaxError"),
    ] {
        if lower.starts_with(prefix) {
            return code;
        }
    }
    "other"
}

/// Erreur du front (`window.onerror`, rejet non traité). Nettoyée (numéros, adresses, jetons), agrégée par empreinte ; ni pile ni
/// adresse ; `204`.
#[utoipa::path(
    post,
    path = "/api/client-errors",
    request_body = ClientErrorPayload,
    responses((status = 204, description = "Erreur reçue"), (status = 413, description = "request.too_large"), (status = 429, description = "rate_limit.exceeded"))
)]
pub async fn post_client_error(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<ClientErrorPayload>,
) -> Result<Response, ApiError> {
    session.require(Action::EventsCreate)?;
    limit(
        &state,
        &format!("client-errors:{}", session.token),
        CLIENT_ERRORS_PER_MINUTE,
        Duration::from_secs(60),
    )?;
    let svc = identity(&state, &session)?;
    let Some(org) = &session.auth.org else {
        return Err(session.ctx.err(ApiError::organization_required()));
    };
    let screen = if events::is_screen(&body.screen) {
        body.screen.as_str()
    } else {
        "S00"
    };
    let version = if events::valid_client_version(&body.client_version) {
        body.client_version.as_str()
    } else {
        "0.0.0"
    };
    let ids: Vec<String> = body
        .request_ids
        .iter()
        .filter(|i| feedback::valid_request_id(i))
        .take(5)
        .cloned()
        .collect();
    let report = errors::client_error(&body.message, screen, version, &ids);
    let ctx = svc.event_context(&session.auth, &session.token, Some(version.to_string()));
    let mut tx = svc
        .db()
        .begin(org.id, session.auth.user.id)
        .await
        .map_err(|_| session.ctx.unexpected())?;
    errors::record(&mut tx, &report)
        .await
        .map_err(|_| session.ctx.unexpected())?;
    if let Some(event) = ProductEvent::client_error(screen, client_error_code(&body.message)) {
        events::emit(&mut tx, &ctx, &event)
            .await
            .map_err(|_| session.ctx.unexpected())?;
    }
    tx.commit().await.map_err(|_| session.ctx.unexpected())?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ───────── POST /api/feedback ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct FeedbackContextPayload {
    pub screen: String,
    /// Modèle d'adresse : sans paramètre de requête ni jeton (nettoyé aussi par le serveur).
    pub path: String,
    pub client_version: String,
    /// Taille d'écran `390x844`.
    pub viewport: String,
    pub last_request_ids: Vec<String>,
    pub last_error_code: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct FeedbackPayload {
    /// `bug`, `request` ou `question`.
    pub category: String,
    /// Texte libre de 5 à 4 000 caractères : stocké tel quel, jamais journalisé.
    pub description: String,
    pub can_call_back: bool,
    pub context: FeedbackContextPayload,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FeedbackCreatedBody {
    pub id: String,
    /// Référence lisible `SIG-000042`, donnée au support.
    pub reference: String,
}

/// Signalement d'un problème. Crée la ligne, l'événement `feedback_submitted` et la tâche d'alerte dans la même transaction.
/// `Idempotency-Key` (UUID) : un double clic ne crée qu'un signalement.
#[utoipa::path(
    post,
    path = "/api/feedback",
    request_body = FeedbackPayload,
    responses(
        (status = 201, description = "Signalement créé", body = FeedbackCreatedBody),
        (status = 422, description = "validation.failed"),
        (status = 429, description = "rate_limit.exceeded : 5 par heure et par utilisateur")
    )
)]
pub async fn post_feedback(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    ApiJson(body): ApiJson<FeedbackPayload>,
) -> Result<Response, ApiError> {
    session.require(Action::FeedbackCreate)?;
    let (Some(org), Some(role)) = (&session.auth.org, session.auth.role) else {
        return Err(session.ctx.err(ApiError::organization_required()));
    };
    let key = match headers.get("idempotency-key") {
        None => None,
        Some(v) => Some(
            v.to_str()
                .ok()
                .and_then(|s| Uuid::parse_str(s.trim()).ok())
                .ok_or_else(|| session.ctx.err(ApiError::malformed()))?,
        ),
    };
    let svc = identity(&state, &session)?;
    let who = Reporter {
        org_id: org.id,
        user_id: session.auth.user.id,
        role: role.as_str().to_string(),
        session_ref: immo_app::identity::observe::session_ref(&session.token),
    };
    let input = FeedbackInput {
        category: body.category,
        description: body.description,
        callback_ok: body.can_call_back,
        screen: body.context.screen,
        path: body.context.path,
        client_version: body.context.client_version,
        viewport: body.context.viewport,
        last_request_ids: body.context.last_request_ids,
        last_error_code: body.context.last_error_code,
        idempotency_key: key,
    };
    // la limite compte les créations ; un rejeu d'une même clé ne la consomme pas davantage qu'un appel
    limit(
        &state,
        &format!("feedback:{}", who.user_id),
        FEEDBACK_PER_HOUR,
        Duration::from_secs(3600),
    )?;
    let created = feedback::create(svc.db(), svc.clock().as_ref(), &who, &input)
        .await
        .map_err(|e| match e {
            FeedbackError::Invalid(issues) => session.ctx.err(ApiError::validation_fields(&issues)),
            FeedbackError::Internal => session.ctx.unexpected(),
        })?;
    Ok((
        StatusCode::CREATED,
        Json(FeedbackCreatedBody {
            id: created.id.to_string(),
            reference: created.reference,
        }),
    )
        .into_response())
}

// ───────── route d'essai ─────────

/// Provoque une erreur 500 réelle (critère 4 et démonstration). Montée seulement hors production avec `ENABLE_TEST_ROUTES=1`,
/// absente du contrat d'API.
async fn test_server_error(session: Session) -> Result<Response, ApiError> {
    Err(session.ctx.unexpected())
}
