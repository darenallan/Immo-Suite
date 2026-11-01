//! Routes de compte de l'étape 03 (`API_CONTRACTS.md` §7.2) : profil, mot de passe, confirmation, double authentification,
//! réinitialisation par lien, page publique d'invitation.
//!
//! Les routes publiques « par jeton » (`Access::Token`) ne portent aucune session : le jeton ou le défi tient lieu de droit.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{post, put};
use axum::{Json, Router};
use immo_app::identity::authorize::Action;
use immo_app::identity::members::AcceptOutcome;
use immo_app::identity::service::{IdentityService, session_cookie};
use immo_app::identity::session::ABSOLUTE_HOURS;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::{LoginBody, OrganizationChoiceBody, UserBody, login_body};
use crate::extract::{ApiJson, Ctx, Session, session_token};
use crate::http::AppState;
use crate::problem::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/reauth", post(reauth))
        .route("/api/auth/totp/verify", post(totp_verify))
        .route("/api/auth/password-reset/complete", post(password_reset_complete))
        .route("/api/me/password", put(change_password))
        .route("/api/me/totp", post(totp_begin))
        .route("/api/me/totp/confirm", post(totp_confirm))
        .route("/api/users/{id}/password-resets", post(issue_reset))
        .route("/api/invitations/accept", post(accept_invitation))
        .route("/api/invitations/{id}", axum::routing::get(invitation_preview))
}

fn identity(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn with_cookie(mut response: Response, cookie: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(cookie) {
        response.headers_mut().append(header::SET_COOKIE, v);
    }
    response
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

// ───────── corps ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ReauthPayload {
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TotpVerifyPayload {
    /// Défi reçu de `POST /auth/login`.
    pub challenge: String,
    /// Code à 6 chiffres, ou code de secours (12 caractères, tirets facultatifs).
    pub code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateMePayload {
    pub display_name: Option<String>,
    /// Changer le numéro exige la confirmation du mot de passe.
    pub login_phone: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TotpConfirmPayload {
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TotpEnrollmentBody {
    /// Clé en base32, affichée une seule fois.
    pub secret_base32: String,
    pub otpauth_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupCodesBody {
    /// Dix codes de secours, affichés une seule fois, par groupes de 4 (`ABCD-EFGH-JKMN`).
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ResetCompletePayload {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResetLinkBody {
    /// Lien à usage unique de 15 minutes, affiché une seule fois.
    pub url: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationPreviewBody {
    pub organization_name: String,
    pub role: String,
    pub expires_at: String,
    pub identifier_exists: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AcceptInvitationPayload {
    pub token: String,
    /// Nouveau compte seulement.
    pub display_name: Option<String>,
    /// Nouveau compte seulement.
    pub password: Option<String>,
}

// ───────── gestionnaires ─────────

/// Confirme le mot de passe : valable 5 minutes pour les actions à enjeu de cette session.
#[utoipa::path(post, path = "/api/auth/reauth", request_body = ReauthPayload,
    responses((status = 204, description = "Confirmé pour 5 minutes"), (status = 401, description = "auth.invalid_credentials"), (status = 429, description = "auth.rate_limited")))]
pub async fn reauth(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<ReauthPayload>,
) -> Result<StatusCode, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    svc.reauth(&session.token, &body.password, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Second facteur : le défi de la connexion et le code ouvrent la session.
#[utoipa::path(post, path = "/api/auth/totp/verify", request_body = TotpVerifyPayload,
    responses((status = 200, description = "Connecté", body = LoginBody), (status = 401, description = "auth.totp_invalid ou auth.session_expired"), (status = 429, description = "auth.rate_limited")))]
pub async fn totp_verify(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    ApiJson(body): ApiJson<TotpVerifyPayload>,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &ctx)?;
    let grant = svc
        .totp_verify(&body.challenge, &body.code, &ctx.client)
        .await
        .map_err(|e| ctx.service(e))?;
    let cookie = session_cookie(&grant.session_token, ABSOLUTE_HOURS * 3600);
    Ok(with_cookie(Json(login_body(&grant)).into_response(), &cookie))
}

/// Change le mot de passe : les autres sessions sont révoquées, la session courante reçoit un nouvel identifiant.
#[utoipa::path(put, path = "/api/me/password", request_body = ChangePasswordPayload,
    responses((status = 204, description = "Changé ; cookie renouvelé"), (status = 401, description = "auth.invalid_credentials"), (status = 422, description = "auth.password_weak")))]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<ChangePasswordPayload>,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    let rotated = svc
        .change_password(
            &session.token,
            &body.current_password,
            &body.new_password,
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    let cookie = session_cookie(&rotated.session_token, ABSOLUTE_HOURS * 3600);
    Ok(with_cookie(StatusCode::NO_CONTENT.into_response(), &cookie))
}

/// Nom affiché et, avec confirmation du mot de passe, numéro de téléphone.
#[utoipa::path(patch, path = "/api/me", request_body = UpdateMePayload,
    responses((status = 200, description = "Compte mis à jour", body = UserBody), (status = 403, description = "auth.reauth_required"), (status = 409, description = "auth.identifier_taken")))]
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<UpdateMePayload>,
) -> Result<Json<UserBody>, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    let user = svc
        .update_profile(
            &session.token,
            body.display_name.as_deref(),
            body.login_phone.as_deref(),
            &session.ctx.client,
        )
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(UserBody {
        id: user.id.to_string(),
        display_name: user.display_name,
    }))
}

/// Crée le secret de la double authentification (affiché une seule fois).
#[utoipa::path(post, path = "/api/me/totp",
    responses((status = 200, description = "Secret en attente de confirmation", body = TotpEnrollmentBody), (status = 422, description = "validation.failed (déjà activée)")))]
pub async fn totp_begin(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<TotpEnrollmentBody>, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    let enrollment = svc
        .totp_begin(&session.token)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(TotpEnrollmentBody {
        secret_base32: enrollment.secret_base32,
        otpauth_url: enrollment.otpauth_url,
    }))
}

/// Active la double authentification et remet les codes de secours.
#[utoipa::path(post, path = "/api/me/totp/confirm", request_body = TotpConfirmPayload,
    responses((status = 200, description = "Activée ; codes de secours affichés une seule fois", body = BackupCodesBody), (status = 401, description = "auth.totp_invalid")))]
pub async fn totp_confirm(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<TotpConfirmPayload>,
) -> Result<Json<BackupCodesBody>, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    let codes = svc
        .totp_confirm(&session.token, &body.code, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(BackupCodesBody { backup_codes: codes }))
}

/// Génère un lien de réinitialisation de 15 minutes pour un membre (jamais pour un gérant : support HABEK).
#[utoipa::path(post, path = "/api/users/{membership_id}/password-resets",
    params(("membership_id" = String, Path, description = "Identifiant d'adhésion")),
    responses((status = 200, description = "Lien affiché une seule fois", body = ResetLinkBody), (status = 403, description = "auth.reauth_required ou auth.forbidden"), (status = 429, description = "auth.rate_limited")))]
pub async fn issue_reset(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<ResetLinkBody>, ApiError> {
    session.require(Action::UserManage)?;
    let svc = identity(&state, &session.ctx)?;
    let membership_id = parse_id(&session.ctx, &id)?;
    let issued = svc
        .issue_password_reset(&session.auth, membership_id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(ResetLinkBody {
        url: format!("{}/reinitialisation/{}", state.public_base_url, issued.token),
        expires_at: issued.expires_at.to_rfc3339(),
    }))
}

/// Choisit un nouveau mot de passe avec un lien de réinitialisation. Aucune session n'est ouverte.
#[utoipa::path(post, path = "/api/auth/password-reset/complete", request_body = ResetCompletePayload,
    responses((status = 204, description = "Mot de passe changé"), (status = 410, description = "auth.reset_expired"), (status = 422, description = "auth.password_weak"), (status = 429, description = "rate_limit.exceeded")))]
pub async fn password_reset_complete(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    ApiJson(body): ApiJson<ResetCompletePayload>,
) -> Result<StatusCode, ApiError> {
    // 10 essais par heure et par adresse : le jeton est aléatoire de 256 bits, la limite protège le coût de calcul
    if let Some(ip) = ctx.client.ip
        && let Err(wait) = state.limiter.hit(&format!("reset:{ip}"), 10, Duration::from_secs(3600))
    {
        return Err(ctx.err(ApiError::request_rate_limited(wait)));
    }
    let svc = identity(&state, &ctx)?;
    svc.complete_password_reset(&body.token, &body.new_password, &ctx.client)
        .await
        .map_err(|e| ctx.service(e))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Page publique d'invitation : organisation, rôle, échéance. Expirée, révoquée ou utilisée : 410, même corps.
#[utoipa::path(get, path = "/api/invitations/{token}",
    params(("token" = String, Path, description = "Jeton de l'invitation")),
    responses((status = 200, description = "Invitation valable", body = InvitationPreviewBody), (status = 404, description = "resource.not_found"), (status = 410, description = "invitation.expired")))]
pub async fn invitation_preview(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    Path(token): Path<String>,
) -> Result<Json<InvitationPreviewBody>, ApiError> {
    let svc = identity(&state, &ctx)?;
    let preview = svc.invitation_preview(&token).await.map_err(|e| ctx.service(e))?;
    Ok(Json(InvitationPreviewBody {
        organization_name: preview.organization_name,
        role: preview.role.as_str().to_string(),
        expires_at: preview.expires_at.to_rfc3339(),
        identifier_exists: preview.identifier_exists,
    }))
}

fn origin_allowed(headers: &HeaderMap, state: &AppState) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|o| state.allowed_origins.iter().any(|a| a == o))
}

/// Accepte une invitation. Nouveau compte : nom et mot de passe, la session est ouverte. Compte existant : connexion préalable
/// (la session de ce compte porte la requête), l'adhésion est créée et la session garde son organisation active.
#[utoipa::path(post, path = "/api/invitations/accept", request_body = AcceptInvitationPayload,
    responses((status = 200, description = "Invitation acceptée", body = LoginBody), (status = 409, description = "auth.identifier_taken"), (status = 410, description = "invitation.expired"), (status = 422, description = "auth.password_weak ou validation.failed")))]
pub async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    headers: HeaderMap,
    ApiJson(body): ApiJson<AcceptInvitationPayload>,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &ctx)?;
    // Une session n'est utilisée que pour un compte existant ; elle exige alors l'origine, comme toute écriture authentifiée.
    let cookie = session_token(&headers);
    if cookie.is_some() && !origin_allowed(&headers, &state) {
        return Err(ctx.err(ApiError::csrf_invalid()));
    }
    let outcome = svc
        .accept_invitation(
            &body.token,
            body.display_name.as_deref(),
            body.password.as_deref(),
            cookie.as_deref(),
            &ctx.client,
        )
        .await
        .map_err(|e| ctx.service(e))?;
    match outcome {
        AcceptOutcome::Session(grant) => {
            let cookie = session_cookie(&grant.session_token, ABSOLUTE_HOURS * 3600);
            Ok(with_cookie(Json(login_body(&grant)).into_response(), &cookie))
        }
        AcceptOutcome::Joined { organizations } => Ok(Json(LoginBody {
            user: None,
            organizations: organizations
                .iter()
                .map(|c| OrganizationChoiceBody {
                    id: c.id.to_string(),
                    name: c.name.clone(),
                    plan: c.plan.as_str().to_string(),
                    kind: c.kind.clone(),
                    lifecycle: c.lifecycle.clone(),
                    role: c.role.as_str().to_string(),
                })
                .collect(),
            active_organization_id: None,
            csrf_token: None,
            mfa_required: false,
            challenge: None,
        })
        .into_response()),
    }
}
