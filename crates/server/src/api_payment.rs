//! Routes d'encaissement et de quittances (étape 07 ; F09, F10, DP-96, `API_CONTRACTS.md` §6.1, §7.6). Droit vérifié d'abord,
//! corps lu ensuite (DP-89). Les montants, l'affectation, le statut et le montant en lettres viennent du serveur.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use immo_app::catalog::Input;
use immo_app::identity::authorize::Action;
use immo_app::identity::service::IdentityService;
use immo_app::ledger::{PAYMENT_FIELDS, PaymentQuery};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::extract::{ApiJson, Ctx, Session};
use crate::http::AppState;
use crate::problem::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/leases/{id}/payment-context", get(payment_context))
        .route("/api/leases/{id}/payments", get(lease_payments))
        .route("/api/payments/preview", post(preview_payment))
        .route("/api/payments", get(list_payments).post(record_payment))
        .route(
            "/api/payments/{id}",
            get(get_payment)
                .put(payment_immutable)
                .patch(payment_immutable)
                .delete(payment_immutable),
        )
        .route("/api/receipts", get(list_receipts))
        .route("/api/receipts/{id}", get(get_receipt))
}

fn service(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

fn body_of(
    ctx: &Ctx,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
    allowed: &[&str],
) -> Result<Input, ApiError> {
    let ApiJson(map) = body?;
    if map.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(ctx.err(ApiError::unknown_field()));
    }
    Ok(map)
}

fn filters(ctx: &Ctx, raw: &HashMap<String, String>, allowed: &[&str]) -> Result<(), ApiError> {
    if raw.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(ctx.err(ApiError::filter_unknown()));
    }
    Ok(())
}

fn date_filter(ctx: &Ctx, raw: &HashMap<String, String>, key: &str) -> Result<Option<NaiveDate>, ApiError> {
    raw.get(key)
        .map(|v| {
            NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d")
                .map_err(|_| ctx.err(ApiError::validation_reason("filter_value")))
        })
        .transpose()
}

fn uuid_filter(ctx: &Ctx, raw: &HashMap<String, String>, key: &str) -> Result<Option<Uuid>, ApiError> {
    raw.get(key)
        .map(|v| Uuid::parse_str(v.trim()).map_err(|_| ctx.err(ApiError::validation_reason("filter_value"))))
        .transpose()
}

// ───────── corps (contrat ; le détail des objets est dans `API_CONTRACTS.md` §6.1) ─────────

/// Contexte de paiement de S04 : bail, locataires, solde, lignes ouvertes dans l'ordre d'affectation, montant proposé, seuil.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentContextBody {
    #[schema(value_type = Object)]
    pub lease: Value,
    #[schema(value_type = Vec<Object>)]
    pub tenants: Value,
    #[schema(value_type = Object)]
    pub balance: Value,
    #[schema(value_type = Vec<Object>)]
    pub open_lines: Value,
    pub suggested_amount: i64,
    pub default_method: String,
    pub today: String,
    pub confirm_threshold: i64,
}

/// Aperçu d'affectation calculé par le serveur, sans écriture.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentPreviewBody {
    #[schema(value_type = Vec<Object>)]
    pub allocations: Value,
    pub surplus_amount: i64,
    pub credit_amount: i64,
    pub remaining_after: i64,
    pub requires_surplus_confirmation: bool,
    pub requires_high_amount_confirmation: bool,
}

/// Paiement enregistré : paiement, quittance numérotée, échéances touchées, avertissements.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentRecordedBody {
    #[schema(value_type = Object)]
    pub payment: Value,
    #[schema(value_type = Object)]
    pub receipt: Value,
    #[schema(value_type = Vec<Object>)]
    pub installments: Value,
    pub warnings: Vec<String>,
}

/// Corps de `POST /payments` (règles : `API_CONTRACTS.md` §6.1).
#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct PaymentPayload {
    pub lease_id: String,
    /// Entier FCFA, 1 à 999 999 999 (R-037).
    pub amount: i64,
    /// `AAAA-MM-JJ`, jamais futur (R-039).
    pub paid_on: String,
    /// `cash`, `mobile_money`, `bank_transfer` ou `cheque`.
    pub method: Option<String>,
    pub external_ref: Option<String>,
    /// `{ kind: "tenant", tenant_id }` ou `{ kind: "third_party", third_party_name, third_party_phone? }`.
    #[schema(value_type = Object)]
    pub payer: Option<Value>,
    pub note: Option<String>,
    /// `null` : répartition par défaut ; sinon `[{ installment_line_id, amount }]`.
    #[schema(value_type = Vec<Object>)]
    pub allocations: Option<Value>,
    /// `"advance"` si le montant dépasse ce qui est dû aujourd'hui (R-046).
    pub surplus: Option<String>,
    pub confirm_high_amount: Option<bool>,
    pub acknowledge_external_ref_duplicate: Option<bool>,
}

/// Corps de `POST /payments/preview`.
#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct PaymentPreviewPayload {
    pub lease_id: String,
    pub amount: i64,
    #[schema(value_type = Vec<Object>)]
    pub allocations: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentsBody {
    #[schema(value_type = Vec<Object>)]
    pub items: Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReceiptsBody {
    #[schema(value_type = Vec<Object>)]
    pub items: Value,
}

// ───────── gestionnaires ─────────

/// Contexte de paiement d'un bail (S04) : un seul aller-retour (§6.1).
#[utoipa::path(get, path = "/api/leases/{id}/payment-context", params(("id" = String, Path)),
    responses((status = 200, description = "Contexte de paiement", body = PaymentContextBody)))]
pub async fn payment_context(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRecord)?;
    let id = parse_id(&session.ctx, &id)?;
    let v = service(&state, &session.ctx)?
        .payment_context(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(v))
}

/// Aperçu d'affectation (R-043, R-044, R-046), sans écriture.
#[utoipa::path(post, path = "/api/payments/preview", request_body = PaymentPreviewPayload,
    responses((status = 200, description = "Aperçu", body = PaymentPreviewBody)))]
pub async fn preview_payment(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRecord)?;
    let input = body_of(&session.ctx, body, &["lease_id", "amount", "allocations"])?;
    let v = service(&state, &session.ctx)?
        .preview_payment(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(v))
}

/// Encaisse un paiement et émet sa quittance (F09, F10). `Idempotency-Key` obligatoire (DP-49) ; rejeu : même réponse,
/// `Idempotent-Replay: true`.
#[utoipa::path(post, path = "/api/payments", request_body = PaymentPayload,
    responses((status = 201, description = "Paiement et quittance", body = PaymentRecordedBody),
              (status = 409, description = "payment.external_ref_duplicate, payment.lease_not_in_progress, payment.stale_allocation"),
              (status = 422, description = "payment.confirmation_required, payment.surplus_not_confirmed, payment.allocation_invalid, idempotency.key_reused_with_different_body")))]
pub async fn record_payment(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Response, ApiError> {
    session.require(Action::PaymentRecord)?;
    let key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v.trim()).ok())
        .ok_or_else(|| session.ctx.err(ApiError::idempotency_key_missing()))?;
    let input = body_of(&session.ctx, body, PAYMENT_FIELDS)?;
    let recorded = service(&state, &session.ctx)?
        .record_payment(&session.auth, key, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let mut response = (StatusCode::CREATED, Json(recorded.body)).into_response();
    if recorded.replay {
        response
            .headers_mut()
            .insert("idempotent-replay", HeaderValue::from_static("true"));
    }
    Ok(response)
}

/// Journal des paiements (S11), du plus récent au plus ancien, 100 au plus.
#[utoipa::path(get, path = "/api/payments",
    params(("from" = Option<String>, Query), ("to" = Option<String>, Query), ("method" = Option<String>, Query),
           ("lease_id" = Option<String>, Query), ("tenant_id" = Option<String>, Query), ("unit_id" = Option<String>, Query),
           ("q" = Option<String>, Query)),
    responses((status = 200, description = "Paiements", body = PaymentsBody)))]
pub async fn list_payments(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    filters(
        &session.ctx,
        &raw,
        &["from", "to", "method", "lease_id", "tenant_id", "unit_id", "q"],
    )?;
    let q = PaymentQuery {
        from: date_filter(&session.ctx, &raw, "from")?,
        to: date_filter(&session.ctx, &raw, "to")?,
        method: raw.get("method").cloned(),
        lease_id: uuid_filter(&session.ctx, &raw, "lease_id")?,
        tenant_id: uuid_filter(&session.ctx, &raw, "tenant_id")?,
        unit_id: uuid_filter(&session.ctx, &raw, "unit_id")?,
        q: raw.get("q").cloned().filter(|s| !s.trim().is_empty()),
    };
    let items = service(&state, &session.ctx)?
        .list_payments(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(serde_json::json!({ "items": items })))
}

/// Paiements d'un bail (onglet Paiements de S09 et S10).
#[utoipa::path(get, path = "/api/leases/{id}/payments", params(("id" = String, Path)),
    responses((status = 200, description = "Paiements du bail", body = PaymentsBody)))]
pub async fn lease_payments(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let q = PaymentQuery {
        lease_id: Some(id),
        ..PaymentQuery::default()
    };
    let items = service(&state, &session.ctx)?
        .list_payments(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(serde_json::json!({ "items": items })))
}

/// Un paiement : affectations et quittance.
#[utoipa::path(get, path = "/api/payments/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Paiement", body = Object)))]
pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let v = service(&state, &session.ctx)?
        .payment(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(v))
}

/// Un paiement validé est immuable (R-047, E-08) : toujours `405 payment.immutable`.
#[utoipa::path(put, path = "/api/payments/{id}", params(("id" = String, Path)),
    responses((status = 405, description = "payment.immutable")))]
pub async fn payment_immutable(session: Session) -> Result<Json<Value>, ApiError> {
    Err(session.ctx.err(ApiError::payment_immutable()))
}

/// Variante `PATCH` (documentée pour le registre).
#[utoipa::path(patch, path = "/api/payments/{id}", params(("id" = String, Path)),
    responses((status = 405, description = "payment.immutable")))]
pub async fn payment_immutable_patch(session: Session) -> Result<Json<Value>, ApiError> {
    payment_immutable(session).await
}

/// Variante `DELETE` (documentée pour le registre).
#[utoipa::path(delete, path = "/api/payments/{id}", params(("id" = String, Path)),
    responses((status = 405, description = "payment.immutable")))]
pub async fn payment_immutable_delete(session: Session) -> Result<Json<Value>, ApiError> {
    payment_immutable(session).await
}

/// Quittances, 100 au plus (filtre `q` sur le numéro).
#[utoipa::path(get, path = "/api/receipts",
    params(("q" = Option<String>, Query), ("from" = Option<String>, Query), ("to" = Option<String>, Query)),
    responses((status = 200, description = "Quittances", body = ReceiptsBody)))]
pub async fn list_receipts(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    filters(&session.ctx, &raw, &["q", "from", "to"])?;
    let from = date_filter(&session.ctx, &raw, "from")?;
    let to = date_filter(&session.ctx, &raw, "to")?;
    let q = raw.get("q").map(String::as_str).filter(|s| !s.trim().is_empty());
    let items = service(&state, &session.ctx)?
        .list_receipts(&session.auth, q, from, to)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(serde_json::json!({ "items": items })))
}

/// Une quittance (S25) : instantané figé, rendu de l'écran sans calcul.
#[utoipa::path(get, path = "/api/receipts/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Quittance", body = Object)))]
pub async fn get_receipt(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let v = service(&state, &session.ctx)?
        .receipt(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(v))
}
