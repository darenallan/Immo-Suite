//! Routes d'administration de l'étape 03 (`API_CONTRACTS.md` §7.3, §7.12, §7.13) : membres, invitations, droits réglables,
//! paramètres, accord d'accès support, journal d'audit.
//!
//! Chaque gestionnaire commence par la session (401 d'abord), puis par `authorize` (403), puis appelle le service, qui exige la
//! confirmation du mot de passe pour les actions à enjeu avant tout effet (F02 §7.3).

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::routing::{get, patch, post, put};
use axum::{Json, Router};
use chrono::NaiveDate;
use immo_app::identity::audit_read::AuditQuery;
use immo_app::identity::authorize::Action;
use immo_app::identity::members::{InvitationView, MemberView};
use immo_app::identity::service::IdentityService;
use immo_app::identity::settings::{OrgSettingsView, SettingsPatch};
use immo_app::identity::support::SupportGrantView;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::{OrganizationDetailBody, org_body};
use crate::extract::{ApiJson, Ctx, Session};
use crate::http::AppState;
use crate::problem::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/settings", get(get_settings).patch(patch_settings))
        .route("/api/users", get(list_users))
        .route("/api/users/{id}", patch(change_role))
        .route("/api/users/{id}/disable", post(disable_user))
        .route("/api/users/{id}/enable", post(enable_user))
        .route("/api/users/{id}/unit-assignments", put(set_unit_assignments))
        .route("/api/invitations", get(list_invitations).post(create_invitation))
        .route("/api/invitations/{id}/revoke", post(revoke_invitation))
        .route("/api/invitations/{id}/renew", post(renew_invitation))
        .route("/api/role-settings", get(get_role_settings).put(put_role_setting))
        .route(
            "/api/support-grants",
            get(list_support_grants).post(create_support_grant),
        )
        .route("/api/support-grants/{id}/revoke", post(revoke_support_grant))
        .route("/api/audit", get(list_audit))
        .route(
            "/api/audit/{id}",
            put(audit_immutable).patch(audit_immutable).delete(audit_immutable),
        )
}

fn identity(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

/// `If-Match: "3"` ou `If-Match: 3` → 3. Absent ou illisible : `None` (le service répond 428).
fn if_match(headers: &HeaderMap) -> Option<i32> {
    headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().trim_start_matches("W/").trim_matches('"'))
        .and_then(|v| v.parse().ok())
}

// ───────── membres ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct MemberBody {
    pub membership_id: String,
    pub user_id: String,
    pub display_name: String,
    pub identifier: String,
    pub role: String,
    /// `active` ou `disabled`.
    pub status: String,
    pub is_self: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MembersBody {
    pub users: Vec<MemberBody>,
}

fn member_body(m: &MemberView) -> MemberBody {
    MemberBody {
        membership_id: m.membership_id.to_string(),
        user_id: m.user_id.to_string(),
        display_name: m.display_name.clone(),
        identifier: m.identifier.clone(),
        role: m.role.as_str().to_string(),
        status: m.status.clone(),
        is_self: m.is_self,
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangeRolePayload {
    pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UnitAssignmentsPayload {
    /// Vide jusqu'à l'étape 05 (les lots n'existent pas encore).
    pub unit_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnitAssignmentsBody {
    pub unit_ids: Vec<String>,
}

/// Membres de l'organisation active.
#[utoipa::path(get, path = "/api/users", responses((status = 200, description = "Membres", body = MembersBody), (status = 403, description = "auth.forbidden")))]
pub async fn list_users(State(state): State<Arc<AppState>>, session: Session) -> Result<Json<MembersBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let members = svc
        .list_members(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(MembersBody {
        users: members.iter().map(member_body).collect(),
    }))
}

/// Change le rôle d'un membre : effet à la requête suivante, aucune session révoquée.
#[utoipa::path(patch, path = "/api/users/{membership_id}", request_body = ChangeRolePayload,
    params(("membership_id" = String, Path, description = "Identifiant d'adhésion")),
    responses((status = 200, description = "Membre", body = MemberBody), (status = 403, description = "auth.reauth_required ou auth.forbidden"), (status = 409, description = "user.last_manager")))]
pub async fn change_role(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<ChangeRolePayload>, ApiError>,
) -> Result<Json<MemberBody>, ApiError> {
    session.require(Action::UserManage)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let member = svc
        .change_role(
            &session.auth,
            parse_id(&session.ctx, &id)?,
            &body.role,
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(member_body(&member)))
}

/// Désactive un membre : ses sessions sur cette organisation sont révoquées.
#[utoipa::path(post, path = "/api/users/{membership_id}/disable",
    params(("membership_id" = String, Path, description = "Identifiant d'adhésion")),
    responses((status = 200, description = "Membre désactivé", body = MemberBody), (status = 403, description = "auth.reauth_required ou auth.forbidden"), (status = 409, description = "user.last_manager")))]
pub async fn disable_user(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<MemberBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let member = svc
        .disable_member(&session.auth, parse_id(&session.ctx, &id)?, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(member_body(&member)))
}

/// Réactive un membre (confirmation du mot de passe exigée).
#[utoipa::path(post, path = "/api/users/{membership_id}/enable",
    params(("membership_id" = String, Path, description = "Identifiant d'adhésion")),
    responses((status = 200, description = "Membre réactivé", body = MemberBody), (status = 403, description = "auth.reauth_required")))]
pub async fn enable_user(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<MemberBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let member = svc
        .enable_member(&session.auth, parse_id(&session.ctx, &id)?, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(member_body(&member)))
}

/// Affecte des lots à un membre. Jusqu'à l'étape 05, seule la liste vide est acceptée (retrait des affectations).
#[utoipa::path(put, path = "/api/users/{membership_id}/unit-assignments", request_body = UnitAssignmentsPayload,
    params(("membership_id" = String, Path, description = "Identifiant d'adhésion")),
    responses((status = 200, description = "Affectations", body = UnitAssignmentsBody), (status = 422, description = "validation.failed")))]
pub async fn set_unit_assignments(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<UnitAssignmentsPayload>, ApiError>,
) -> Result<Json<UnitAssignmentsBody>, ApiError> {
    session.require(Action::UserManage)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let ids = svc
        .set_unit_assignments(
            &session.auth,
            parse_id(&session.ctx, &id)?,
            &body.unit_ids,
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(UnitAssignmentsBody {
        unit_ids: ids.iter().map(Uuid::to_string).collect(),
    }))
}

// ───────── invitations ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationBody {
    pub id: String,
    pub identifier: String,
    pub role: String,
    /// `pending`, `expired`, `accepted` ou `revoked`.
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub invited_by: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationsBody {
    pub invitations: Vec<InvitationBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedInvitationBody {
    pub invitation: InvitationBody,
    /// Lien à usage unique de 7 jours, affiché une seule fois.
    pub url: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateInvitationPayload {
    /// Téléphone (un des deux, pas les deux).
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: String,
    #[serde(default)]
    pub unit_ids: Vec<String>,
}

fn invitation_body(i: &InvitationView) -> InvitationBody {
    InvitationBody {
        id: i.id.to_string(),
        identifier: i.identifier.clone(),
        role: i.role.as_str().to_string(),
        status: i.status.to_string(),
        created_at: i.created_at.to_rfc3339(),
        expires_at: i.expires_at.to_rfc3339(),
        invited_by: i.invited_by.clone(),
    }
}

/// Invitations de l'organisation (en attente, expirées, acceptées, révoquées).
#[utoipa::path(get, path = "/api/invitations", responses((status = 200, description = "Invitations", body = InvitationsBody)))]
pub async fn list_invitations(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<InvitationsBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let all = svc
        .list_invitations(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(InvitationsBody {
        invitations: all.iter().map(invitation_body).collect(),
    }))
}

/// Crée une invitation. Le lien n'est dans aucune autre réponse : pour le retrouver, « Renvoyer ».
#[utoipa::path(post, path = "/api/invitations", request_body = CreateInvitationPayload,
    responses((status = 200, description = "Invitation créée", body = CreatedInvitationBody), (status = 403, description = "auth.reauth_required (rôle gérant)"), (status = 409, description = "auth.identifier_taken ou invitation.already_pending"), (status = 422, description = "validation.failed")))]
pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<CreateInvitationPayload>, ApiError>,
) -> Result<Json<CreatedInvitationBody>, ApiError> {
    session.require(Action::UserManage)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let identifier = match (body.phone.as_deref(), body.email.as_deref()) {
        (Some(p), None) => p,
        (None, Some(e)) => e,
        _ => return Err(session.ctx.err(ApiError::validation_reason("identifier_required"))),
    };
    let created = svc
        .create_invitation(
            &session.auth,
            identifier,
            &body.role,
            &body.unit_ids,
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(created_body(&state, &created)))
}

fn created_body(state: &AppState, created: &immo_app::identity::members::CreatedInvitation) -> CreatedInvitationBody {
    CreatedInvitationBody {
        invitation: invitation_body(&created.invitation),
        url: format!("{}/invitation/{}", state.public_base_url, created.token),
        expires_at: created.invitation.expires_at.to_rfc3339(),
    }
}

/// Révoque une invitation : le lien cesse de fonctionner.
#[utoipa::path(post, path = "/api/invitations/{id}/revoke",
    params(("id" = String, Path, description = "Identifiant de l'invitation")),
    responses((status = 200, description = "Invitation révoquée", body = InvitationBody), (status = 404, description = "resource.not_found")))]
pub async fn revoke_invitation(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<InvitationBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let view = svc
        .revoke_invitation(&session.auth, parse_id(&session.ctx, &id)?, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(invitation_body(&view)))
}

/// Renvoie une invitation : l'ancienne est révoquée, un nouveau jeton est créé.
#[utoipa::path(post, path = "/api/invitations/{id}/renew",
    params(("id" = String, Path, description = "Identifiant de l'invitation")),
    responses((status = 200, description = "Nouvelle invitation", body = CreatedInvitationBody), (status = 404, description = "resource.not_found")))]
pub async fn renew_invitation(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<CreatedInvitationBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let created = svc
        .renew_invitation(&session.auth, parse_id(&session.ctx, &id)?, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(created_body(&state, &created)))
}

// ───────── droits réglables ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleSettingBody {
    pub role: String,
    /// `record_payment`, `import_data` ou `export_data`.
    pub action: String,
    pub allowed: bool,
    /// Valeur d'`If-Match` pour la prochaine écriture (0 : aucune ligne).
    pub version: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleSettingsBody {
    pub settings: Vec<RoleSettingBody>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RoleSettingPayload {
    pub role: String,
    pub action: String,
    pub allowed: bool,
}

fn role_setting_body(s: &immo_app::identity::settings::RoleSettingView) -> RoleSettingBody {
    RoleSettingBody {
        role: s.role.as_str().to_string(),
        action: s.action.to_string(),
        allowed: s.allowed,
        version: s.version,
    }
}

/// Les droits réglables applicables au plan de l'organisation.
#[utoipa::path(get, path = "/api/role-settings", responses((status = 200, description = "Réglages", body = RoleSettingsBody)))]
pub async fn get_role_settings(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<RoleSettingsBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let all = svc
        .role_settings(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(RoleSettingsBody {
        settings: all.iter().map(role_setting_body).collect(),
    }))
}

/// Règle un droit (`If-Match` : la version lue, 0 pour créer). Effet à la requête suivante, sans reconnexion.
#[utoipa::path(put, path = "/api/role-settings", request_body = RoleSettingPayload,
    responses((status = 200, description = "Réglage", body = RoleSettingBody), (status = 412, description = "concurrency.version_mismatch"), (status = 428, description = "concurrency.version_required"), (status = 422, description = "validation.failed")))]
pub async fn put_role_setting(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    body: Result<ApiJson<RoleSettingPayload>, ApiError>,
) -> Result<Json<RoleSettingBody>, ApiError> {
    session.require(Action::UserManage)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let setting = svc
        .put_role_setting(
            &session.auth,
            &body.role,
            &body.action,
            body.allowed,
            if_match(&headers),
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(role_setting_body(&setting)))
}

// ───────── organisation et paramètres ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PatchOrganizationPayload {
    pub name: Option<String>,
    /// Vide : efface la ville.
    pub city: Option<String>,
}

/// Nom et ville de l'organisation (`If-Match` obligatoire).
#[utoipa::path(patch, path = "/api/organization", request_body = PatchOrganizationPayload,
    responses((status = 200, description = "Organisation", body = OrganizationDetailBody), (status = 412, description = "concurrency.version_mismatch"), (status = 428, description = "concurrency.version_required")))]
pub async fn patch_organization(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    body: Result<ApiJson<PatchOrganizationPayload>, ApiError>,
) -> Result<Json<OrganizationDetailBody>, ApiError> {
    session.require(Action::SettingsWrite)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let detail = svc
        .patch_organization(
            &session.auth,
            body.name.as_deref(),
            body.city.as_deref(),
            if_match(&headers),
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(org_body(detail)))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SettingsBody {
    pub receipt_prefix: String,
    pub receipt_header: Option<String>,
    pub address: Option<String>,
    #[schema(value_type = Object)]
    pub legal_ids: Value,
    pub default_paper: String,
    pub share_link_ttl_days: i32,
    /// Entier FCFA.
    pub payment_confirm_threshold: i64,
    pub version: i32,
    pub timezone: &'static str,
    pub currency: &'static str,
    pub locale: &'static str,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PatchSettingsPayload {
    pub receipt_prefix: Option<String>,
    pub receipt_header: Option<String>,
    pub address: Option<String>,
    #[schema(value_type = Object)]
    pub legal_ids: Option<Value>,
    pub default_paper: Option<String>,
    #[schema(value_type = i64)]
    pub share_link_ttl_days: Option<Value>,
    /// Entier FCFA de 1 à 999 999 999 ; un décimal ou un négatif est refusé.
    #[schema(value_type = i64)]
    pub payment_confirm_threshold: Option<Value>,
}

fn settings_body(s: OrgSettingsView) -> SettingsBody {
    SettingsBody {
        receipt_prefix: s.receipt_prefix,
        receipt_header: s.receipt_header,
        address: s.address,
        legal_ids: s.legal_ids,
        default_paper: s.default_paper,
        share_link_ttl_days: s.share_link_ttl_days,
        payment_confirm_threshold: s.payment_confirm_threshold,
        version: s.version,
        timezone: "Africa/Ouagadougou",
        currency: "XOF",
        locale: "fr",
    }
}

/// Paramètres de l'organisation (fuseau, devise et langue en lecture seule).
#[utoipa::path(get, path = "/api/settings", responses((status = 200, description = "Paramètres", body = SettingsBody)))]
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<SettingsBody>, ApiError> {
    session.require(Action::SettingsRead)?;
    let svc = identity(&state, &session.ctx)?;
    let settings = svc
        .org_settings(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(settings_body(settings)))
}

/// Modifie les paramètres (`If-Match`). Un champ inconnu est refusé ; un montant décimal ou négatif aussi.
#[utoipa::path(patch, path = "/api/settings", request_body = PatchSettingsPayload,
    responses((status = 200, description = "Paramètres", body = SettingsBody), (status = 412, description = "concurrency.version_mismatch"), (status = 422, description = "validation.failed")))]
pub async fn patch_settings(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    body: Result<ApiJson<PatchSettingsPayload>, ApiError>,
) -> Result<Json<SettingsBody>, ApiError> {
    session.require(Action::SettingsWrite)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let patch = SettingsPatch {
        receipt_prefix: body.receipt_prefix,
        receipt_header: body.receipt_header,
        address: body.address,
        legal_ids: body.legal_ids,
        default_paper: body.default_paper,
        share_link_ttl_days: body.share_link_ttl_days,
        payment_confirm_threshold: body.payment_confirm_threshold,
    };
    let settings = svc
        .patch_settings(&session.auth, &patch, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(settings_body(settings)))
}

// ───────── accord d'accès support ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct SupportGrantBody {
    pub id: String,
    /// `read` ou `import_assist`.
    pub scope: String,
    pub reason: String,
    pub granted_by: String,
    pub granted_at: String,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    /// `active`, `expired` ou `revoked` (dérivé).
    pub status: String,
    pub duration_hours: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SupportAccessBody {
    pub occurred_at: String,
    pub action: String,
    pub object_type: String,
    pub object_id: String,
    pub actor_label: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SupportOverviewBody {
    pub grants: Vec<SupportGrantBody>,
    pub accesses: Vec<SupportAccessBody>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSupportGrantPayload {
    /// `read` ou `import_assist`.
    pub scope: String,
    pub reason: String,
    /// De 1 à 168 (7 jours) ; 24 par défaut.
    pub duration_hours: Option<i64>,
}

fn grant_body(g: &SupportGrantView) -> SupportGrantBody {
    SupportGrantBody {
        id: g.id.to_string(),
        scope: g.scope.clone(),
        reason: g.reason.clone(),
        granted_by: g.granted_by.clone(),
        granted_at: g.granted_at.to_rfc3339(),
        expires_at: g.expires_at.to_rfc3339(),
        revoked_at: g.revoked_at.map(|t| t.to_rfc3339()),
        status: g.status.to_string(),
        duration_hours: g.duration_hours,
    }
}

/// Accords donnés à l'équipe HABEK et consultations qu'elle a faites.
#[utoipa::path(get, path = "/api/support-grants", responses((status = 200, description = "Accords et consultations", body = SupportOverviewBody)))]
pub async fn list_support_grants(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<SupportOverviewBody>, ApiError> {
    session.require(Action::SupportGrant)?;
    let svc = identity(&state, &session.ctx)?;
    let overview = svc
        .support_overview(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(SupportOverviewBody {
        grants: overview.grants.iter().map(grant_body).collect(),
        accesses: overview
            .accesses
            .iter()
            .map(|a| SupportAccessBody {
                occurred_at: a.occurred_at.to_rfc3339(),
                action: a.action.clone(),
                object_type: a.object_type.clone(),
                object_id: a.object_id.clone(),
                actor_label: a.actor_label.clone(),
            })
            .collect(),
    }))
}

/// Autorise l'équipe HABEK pour une durée limitée (confirmation du mot de passe exigée).
#[utoipa::path(post, path = "/api/support-grants", request_body = CreateSupportGrantPayload,
    responses((status = 200, description = "Accord", body = SupportGrantBody), (status = 403, description = "auth.reauth_required"), (status = 422, description = "validation.failed")))]
pub async fn create_support_grant(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<CreateSupportGrantPayload>, ApiError>,
) -> Result<Json<SupportGrantBody>, ApiError> {
    session.require(Action::SupportGrant)?;
    let ApiJson(body) = body?;
    let svc = identity(&state, &session.ctx)?;
    let grant = svc
        .create_support_grant(
            &session.auth,
            &body.scope,
            &body.reason,
            body.duration_hours
                .unwrap_or(immo_app::identity::support::DEFAULT_HOURS),
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(grant_body(&grant)))
}

/// Retire l'accès immédiatement (confirmation du mot de passe exigée).
#[utoipa::path(post, path = "/api/support-grants/{id}/revoke",
    params(("id" = String, Path, description = "Identifiant de l'accord")),
    responses((status = 200, description = "Accord révoqué", body = SupportGrantBody), (status = 404, description = "resource.not_found")))]
pub async fn revoke_support_grant(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<SupportGrantBody>, ApiError> {
    session.require(Action::SupportGrant)?;
    let svc = identity(&state, &session.ctx)?;
    let grant = svc
        .revoke_support_grant(&session.auth, parse_id(&session.ctx, &id)?, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(grant_body(&grant)))
}

// ───────── journal d'audit ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditActorBody {
    pub kind: String,
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditObjectBody {
    #[serde(rename = "type")]
    pub object_type: String,
    pub id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditRowBody {
    pub id: i64,
    pub occurred_at: String,
    pub actor: AuditActorBody,
    pub action: String,
    pub object: AuditObjectBody,
    #[schema(value_type = Object)]
    pub before: Option<Value>,
    #[schema(value_type = Object)]
    pub after: Option<Value>,
    /// Absente (jamais vide) pour les rôles qui ne voient pas l'adresse IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditActorChoice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditMetaBody {
    /// Personnes qui ont une adhésion dans l'organisation (filtre « utilisateur »).
    pub actors: Vec<AuditActorChoice>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditPageBody {
    pub items: Vec<AuditRowBody>,
    pub next_cursor: Option<String>,
    pub meta: AuditMetaBody,
}

fn date_param(
    ctx: &Ctx,
    params: &HashMap<String, String>,
    name: &'static str,
    issues: &mut Vec<immo_app::identity::error::FieldIssue>,
) -> Option<NaiveDate> {
    let raw = params.get(name)?;
    match NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        Ok(d) => Some(d),
        Err(_) => {
            let _ = ctx;
            issues.push(immo_app::identity::error::FieldIssue {
                field: name,
                code: "validation.enum",
            });
            None
        }
    }
}

/// Journal d'audit, du plus récent au plus ancien. Filtres facultatifs, pagination par curseur (25 par défaut, 100 au plus).
#[utoipa::path(get, path = "/api/audit",
    params(
        ("actor_id" = Option<String>, Query, description = "Identifiant d'utilisateur"),
        ("action" = Option<String>, Query, description = "Code objet.verbe"),
        ("object_type" = Option<String>, Query, description = "Type d'objet"),
        ("object_id" = Option<String>, Query, description = "Identifiant d'objet"),
        ("from" = Option<String>, Query, description = "AAAA-MM-JJ, fuseau de l'organisation, borne incluse"),
        ("to" = Option<String>, Query, description = "AAAA-MM-JJ, fuseau de l'organisation, borne incluse"),
        ("actor_kind" = Option<String>, Query, description = "user, system ou support"),
        ("limit" = Option<i64>, Query, description = "25 par défaut, 100 au plus"),
        ("cursor" = Option<String>, Query, description = "Curseur opaque de la page suivante")
    ),
    responses((status = 200, description = "Page du journal", body = AuditPageBody), (status = 400, description = "request.cursor_invalid"), (status = 403, description = "auth.forbidden"), (status = 422, description = "validation.failed")))]
pub async fn list_audit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AuditPageBody>, ApiError> {
    session.require(Action::AuditRead)?;
    let svc = identity(&state, &session.ctx)?;
    let mut issues = vec![];
    let from = date_param(&session.ctx, &params, "from", &mut issues);
    let to = date_param(&session.ctx, &params, "to", &mut issues);
    let limit = match params.get("limit").map(|l| l.parse::<i64>()) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(_)) => {
            issues.push(immo_app::identity::error::FieldIssue {
                field: "limit",
                code: "validation.enum",
            });
            None
        }
    };
    if !issues.is_empty() {
        return Err(session.ctx.err(ApiError::validation_fields(&issues)));
    }
    let query = AuditQuery {
        actor_id: params.get("actor_id").cloned(),
        action: params.get("action").cloned(),
        object_type: params.get("object_type").cloned(),
        object_id: params.get("object_id").cloned(),
        from,
        to,
        actor_kind: params.get("actor_kind").cloned(),
        limit,
        cursor: params.get("cursor").cloned(),
    };
    let page = svc
        .audit_page(&session.auth, &query)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(AuditPageBody {
        items: page
            .rows
            .into_iter()
            .map(|r| AuditRowBody {
                id: r.id,
                occurred_at: r.occurred_at.to_rfc3339(),
                actor: AuditActorBody {
                    label: match r.actor_kind.as_str() {
                        "support" => Some("Équipe HABEK".to_string()),
                        "system" => Some("Système".to_string()),
                        _ => None,
                    },
                    kind: r.actor_kind,
                    user_id: r.actor_user_id.map(|u| u.to_string()),
                    name: r.actor_name,
                },
                action: r.action,
                object: AuditObjectBody {
                    object_type: r.object_type,
                    id: r.object_id,
                },
                before: r.before,
                after: r.after,
                ip: r.ip,
                request_id: r.request_id,
            })
            .collect(),
        next_cursor: page.next_cursor,
        meta: AuditMetaBody {
            actors: page
                .actors
                .into_iter()
                .map(|(id, name)| AuditActorChoice {
                    id: id.to_string(),
                    name,
                })
                .collect(),
        },
    }))
}

/// Le journal ne se modifie pas : `405 audit.immutable`, `Allow: GET`, et une ligne `security.permission_denied`.
pub async fn audit_immutable(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(_id): Path<String>,
) -> ApiError {
    if let Ok(svc) = identity(&state, &session.ctx) {
        svc.trace_permission_denied(&session.auth, "/audit/{id}", "audit.write", &session.ctx.client)
            .await;
    }
    session.ctx.err(ApiError::audit_immutable())
}
