//! Routes des baux et de l'échéancier (étape 06 ; F07, F08, DP-94). Droit vérifié d'abord, corps lu ensuite (DP-89) ; le statut
//! et l'ancienneté d'une échéance viennent du domaine, jamais du front.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use immo_app::catalog::Input;
use immo_app::identity::authorize::Action;
use immo_app::identity::service::IdentityService;
use immo_app::identity::settings::rfc3339;
use immo_app::lease::{InstallmentView, LeaseQuery, LeaseView, LineView};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api_catalog::UnitBody;
use crate::extract::{ApiJson, Ctx, Session};
use crate::http::AppState;
use crate::problem::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/leases", get(list_leases).post(create_lease))
        .route("/api/leases/{id}", get(get_lease).patch(patch_lease))
        .route("/api/leases/{id}/activate", post(activate_lease))
        .route("/api/leases/{id}/amendments", post(amend_lease))
        .route("/api/leases/{id}/terminate", post(terminate_lease))
        .route("/api/leases/{id}/installments", get(lease_installments))
        .route("/api/leases/{id}/installment-lines", post(add_line))
        .route("/api/installment-lines/{id}/void", post(void_line))
        .route("/api/units/{id}/leases", get(unit_leases))
        .route("/api/units/{id}/works", post(set_works))
}

fn service(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

fn if_match(headers: &HeaderMap) -> Option<i32> {
    headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().trim_start_matches("W/").trim_matches('"'))
        .and_then(|v| v.parse().ok())
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

// ───────── corps des réponses ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct LeaseTenantBody {
    pub id: String,
    pub display_name: String,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AmendmentBody {
    pub id: String,
    pub effective_from: String,
    pub entered_on: String,
    pub new_rent: Option<i64>,
    pub new_charges: Option<i64>,
    pub new_end_date: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LeaseBody {
    pub id: String,
    pub unit_id: String,
    pub unit_reference: String,
    pub property_name: String,
    /// `residential` ou `commercial`.
    pub kind: String,
    /// `draft`, `active`, `notice` ou `ended`.
    pub status: String,
    pub start_date: String,
    pub end_date: Option<String>,
    /// Loyer et charges initiaux (FCFA).
    pub rent: i64,
    pub charges: i64,
    /// Loyer et charges en vigueur aujourd'hui (R-104).
    pub current_rent: i64,
    pub current_charges: i64,
    pub due_day: i16,
    pub due_day_defaulted: bool,
    pub deposit_amount: i64,
    pub vat_included: bool,
    pub duration_months: Option<i32>,
    pub indexation_note: Option<String>,
    pub terminated_on: Option<String>,
    pub termination_reason: Option<String>,
    pub activated_at: Option<String>,
    pub tenants: Vec<LeaseTenantBody>,
    pub amendments: Vec<AmendmentBody>,
    pub version: i32,
    /// Avertissements non bloquants (`lease.deposit_above_two_months`, `lease.due_day_defaulted`).
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LeasesBody {
    pub items: Vec<LeaseBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LineBody {
    pub id: String,
    /// `rent`, `charges`, `arrears` ou `exceptional`.
    pub kind: String,
    pub label: Option<String>,
    pub amount: i64,
    pub due_date: String,
    pub voided: bool,
    pub void_reason: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InstallmentBody {
    pub id: String,
    pub period: String,
    pub due_date: String,
    pub due_amount: i64,
    pub allocated_amount: i64,
    pub remaining_amount: i64,
    /// `upcoming`, `due`, `partial`, `paid` ou `late` (calculé à la lecture, R-032).
    pub status: String,
    pub days_late: i64,
    pub lines: Vec<LineBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InstallmentsBody {
    pub items: Vec<InstallmentBody>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct LeasePayload {
    pub unit_id: Option<String>,
    pub tenant_ids: Option<Vec<String>>,
    pub primary_tenant_id: Option<String>,
    pub kind: Option<String>,
    /// `AAAA-MM-JJ`.
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub rent: Option<i64>,
    pub charges: Option<i64>,
    pub due_day: Option<i32>,
    pub deposit_amount: Option<i64>,
    pub vat_included: Option<bool>,
    #[schema(value_type = Object)]
    pub commercial_terms: Option<Value>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct AmendmentPayload {
    pub effective_from: String,
    pub rent: Option<i64>,
    pub charges: Option<i64>,
    pub new_end_date: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct TerminatePayload {
    pub terminated_on: String,
    pub reason: String,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct LinePayload {
    /// `arrears` ou `exceptional`.
    pub kind: String,
    pub label: String,
    pub amount: i64,
    pub due_date: String,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct VoidPayload {
    pub reason: String,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct WorksPayload {
    pub under_works: bool,
}

const LEASE_FIELDS: [&str; 12] = [
    "unit_id",
    "tenant_ids",
    "primary_tenant_id",
    "kind",
    "start_date",
    "end_date",
    "rent",
    "charges",
    "due_day",
    "deposit_amount",
    "vat_included",
    "commercial_terms",
];

fn lease_body(l: LeaseView) -> LeaseBody {
    LeaseBody {
        id: l.id.to_string(),
        unit_id: l.unit_id.to_string(),
        unit_reference: l.unit_reference,
        property_name: l.property_name,
        kind: l.kind,
        status: l.status,
        start_date: l.start_date.to_string(),
        end_date: l.end_date.map(|d| d.to_string()),
        rent: l.rent,
        charges: l.charges,
        current_rent: l.current_rent,
        current_charges: l.current_charges,
        due_day: l.due_day,
        due_day_defaulted: l.due_day_defaulted,
        deposit_amount: l.deposit_amount,
        vat_included: l.vat_included,
        duration_months: l.duration_months,
        indexation_note: l.indexation_note,
        terminated_on: l.terminated_on.map(|d| d.to_string()),
        termination_reason: l.termination_reason,
        activated_at: l.activated_at.map(rfc3339),
        tenants: l
            .tenants
            .into_iter()
            .map(|t| LeaseTenantBody {
                id: t.id.to_string(),
                display_name: t.display_name,
                is_primary: t.is_primary,
            })
            .collect(),
        amendments: l
            .amendments
            .into_iter()
            .map(|a| AmendmentBody {
                id: a.id.to_string(),
                effective_from: a.effective_from.to_string(),
                entered_on: a.entered_on.to_string(),
                new_rent: a.new_rent,
                new_charges: a.new_charges,
                new_end_date: a.new_end_date.map(|d| d.to_string()),
                note: a.note,
            })
            .collect(),
        version: l.version,
        warnings: l.warnings.into_iter().map(str::to_string).collect(),
    }
}

fn line_body(l: LineView) -> LineBody {
    LineBody {
        id: l.id.to_string(),
        kind: l.kind,
        label: l.label,
        amount: l.amount,
        due_date: l.due_date.to_string(),
        voided: l.voided,
        void_reason: l.void_reason,
    }
}

fn installment_body(i: InstallmentView) -> InstallmentBody {
    InstallmentBody {
        id: i.id.to_string(),
        period: i.period.to_string(),
        due_date: i.due_date.to_string(),
        due_amount: i.due_amount,
        allocated_amount: i.allocated_amount,
        remaining_amount: i.remaining_amount,
        status: i.status.as_str().to_string(),
        days_late: i.days_late,
        lines: i.lines.into_iter().map(line_body).collect(),
    }
}

// ───────── gestionnaires ─────────

/// Baux (filtres `status`, `unit_id`, `tenant_id`) ; l'agent terrain ne voit que ceux de ses lots.
#[utoipa::path(get, path = "/api/leases", responses((status = 200, description = "Baux", body = LeasesBody)))]
pub async fn list_leases(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<LeasesBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    if raw
        .keys()
        .any(|k| !["status", "unit_id", "tenant_id"].contains(&k.as_str()))
    {
        return Err(session.ctx.err(ApiError::filter_unknown()));
    }
    let uuid = |k: &str| -> Result<Option<Uuid>, ApiError> {
        raw.get(k)
            .map(|v| {
                Uuid::parse_str(v.trim()).map_err(|_| session.ctx.err(ApiError::validation_reason("filter_value")))
            })
            .transpose()
    };
    let q = LeaseQuery {
        status: raw.get("status").cloned(),
        unit_id: uuid("unit_id")?,
        tenant_id: uuid("tenant_id")?,
    };
    let items = service(&state, &session.ctx)?
        .list_leases(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(LeasesBody {
        items: items.into_iter().map(lease_body).collect(),
    }))
}

/// Crée un bail brouillon ; les avertissements ne bloquent pas (R-024).
#[utoipa::path(post, path = "/api/leases", request_body = LeasePayload,
    responses((status = 201, description = "Bail brouillon", body = LeaseBody), (status = 409, description = "lease.kind_not_allowed, unit.archived")))]
pub async fn create_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<(StatusCode, Json<LeaseBody>), ApiError> {
    session.require(Action::CatalogWrite)?;
    let input = body_of(&session.ctx, body, &LEASE_FIELDS)?;
    let l = service(&state, &session.ctx)?
        .create_lease(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, Json(lease_body(l))))
}

/// Fiche d'un bail.
#[utoipa::path(get, path = "/api/leases/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Bail", body = LeaseBody), (status = 404, description = "not_found")))]
pub async fn get_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<LeaseBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let l = service(&state, &session.ctx)?
        .lease(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(lease_body(l)))
}

/// Modifie un bail brouillon (`If-Match`) ; un bail activé se change par avenant.
#[utoipa::path(patch, path = "/api/leases/{id}", params(("id" = String, Path)), request_body = LeasePayload,
    responses((status = 200, description = "Bail", body = LeaseBody), (status = 409, description = "lease.not_draft"), (status = 412, description = "concurrency.version_mismatch")))]
pub async fn patch_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<LeaseBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(
        &session.ctx,
        body,
        &["start_date", "end_date", "rent", "charges", "due_day", "deposit_amount"],
    )?;
    let l = service(&state, &session.ctx)?
        .patch_lease(&session.auth, id, &input, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(lease_body(l)))
}

/// Active un bail : échéances générées, lot occupé (R-020, R-028, R-093) ; `Idempotency-Key` : un rejeu renvoie le bail actif.
#[utoipa::path(post, path = "/api/leases/{id}/activate", params(("id" = String, Path)),
    responses((status = 200, description = "Bail actif", body = LeaseBody), (status = 409, description = "lease.overlap, unit.under_works, lease.not_draft")))]
pub async fn activate_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<LeaseBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let replay = headers.contains_key("idempotency-key");
    let l = service(&state, &session.ctx)?
        .activate_lease(&session.auth, id, replay, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(lease_body(l)))
}

/// Avenant daté, jamais rétroactif (R-023, R-104) ; une nouvelle date de fin prolonge le bail (R-091).
#[utoipa::path(post, path = "/api/leases/{id}/amendments", params(("id" = String, Path)), request_body = AmendmentPayload,
    responses((status = 200, description = "Bail", body = LeaseBody), (status = 409, description = "lease.not_in_progress")))]
pub async fn amend_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<LeaseBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(
        &session.ctx,
        body,
        &["effective_from", "rent", "charges", "new_end_date", "note"],
    )?;
    let l = service(&state, &session.ctx)?
        .amend_lease(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(lease_body(l)))
}

/// Résilie un bail (R-030) : motif obligatoire, lot de nouveau vacant.
#[utoipa::path(post, path = "/api/leases/{id}/terminate", params(("id" = String, Path)), request_body = TerminatePayload,
    responses((status = 200, description = "Bail terminé", body = LeaseBody)))]
pub async fn terminate_lease(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<LeaseBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &["terminated_on", "reason"])?;
    let l = service(&state, &session.ctx)?
        .terminate_lease(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(lease_body(l)))
}

/// Échéancier d'un bail, lignes annulées comprises ; statut et jours de retard calculés à la lecture.
#[utoipa::path(get, path = "/api/leases/{id}/installments", params(("id" = String, Path)),
    responses((status = 200, description = "Échéances", body = InstallmentsBody)))]
pub async fn lease_installments(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<InstallmentsBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let items = service(&state, &session.ctx)?
        .lease_installments(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(InstallmentsBody {
        items: items.into_iter().map(installment_body).collect(),
    }))
}

/// Ajoute un arriéré daté ou une ligne exceptionnelle (R-035, R-036).
#[utoipa::path(post, path = "/api/leases/{id}/installment-lines", params(("id" = String, Path)), request_body = LinePayload,
    responses((status = 201, description = "Ligne", body = LineBody)))]
pub async fn add_line(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<(StatusCode, Json<LineBody>), ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &["kind", "label", "amount", "due_date"])?;
    let l = service(&state, &session.ctx)?
        .add_installment_line(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, Json(line_body(l))))
}

/// Annule une ligne d'échéance avec un motif (R-036).
#[utoipa::path(post, path = "/api/installment-lines/{id}/void", params(("id" = String, Path)), request_body = VoidPayload,
    responses((status = 200, description = "Ligne annulée", body = LineBody)))]
pub async fn void_line(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<LineBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &["reason"])?;
    let l = service(&state, &session.ctx)?
        .void_installment_line(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(line_body(l)))
}

/// Baux d'un lot (onglet Baux de S07).
#[utoipa::path(get, path = "/api/units/{id}/leases", params(("id" = String, Path)),
    responses((status = 200, description = "Baux du lot", body = LeasesBody)))]
pub async fn unit_leases(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<LeasesBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let svc = service(&state, &session.ctx)?;
    // le lot doit être visible (portée de l'agent terrain, isolation) : sinon 404
    svc.unit(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let items = svc
        .list_leases(
            &session.auth,
            &LeaseQuery {
                unit_id: Some(id),
                ..LeaseQuery::default()
            },
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(LeasesBody {
        items: items.into_iter().map(lease_body).collect(),
    }))
}

/// Pose ou retire « en travaux » sur un lot vacant (R-093).
#[utoipa::path(post, path = "/api/units/{id}/works", params(("id" = String, Path)), request_body = WorksPayload,
    responses((status = 200, description = "Lot", body = UnitBody), (status = 409, description = "unit.works_requires_vacant")))]
pub async fn set_works(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Json<UnitBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &["under_works"])?;
    let u = service(&state, &session.ctx)?
        .set_unit_works(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(crate::api_catalog::unit_body_pub(u)))
}
