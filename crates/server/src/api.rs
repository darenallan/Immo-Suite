//! Routes d'identité de l'étape 02 (`API_CONTRACTS.md` §7.2 et §7.3) : connexion, déconnexion, profil, choix d'organisation.
//!
//! `ROUTES` est le registre des routes : chacune y déclare l'accès qu'elle exige. Un test compare ce registre au contrat OpenAPI
//! et appelle chaque route sans session : une route absente du registre ou qui répond sans authentification fait échouer le build.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use immo_app::identity::authorize::{Action, Decision, authorize};
use immo_app::identity::service::{
    AuthError, Authenticated, IdentityService, LoginError, LoginRequest, OrgChoice, SelectError, SessionGrant,
    clear_session_cookie, session_cookie,
};
use immo_app::identity::session::ABSOLUTE_HOURS;
use immo_app::platform::clock::OUAGADOUGOU;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::extract::{ApiJson, Ctx, Session, session_token};
use crate::http::AppState;
use crate::problem::ApiError;

/// Accès exigé par une route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Sans session (liste fermée de `API_CONTRACTS.md` §4.1).
    Public,
    /// Session valide ; une écriture exige en plus `Origin` et `X-CSRF-Token`.
    Session,
    /// Session valide et l'action de la matrice (F02).
    Action(Action),
}

#[derive(Debug, Clone, Copy)]
pub struct RouteSpec {
    pub method: &'static str,
    pub path: &'static str,
    pub access: Access,
}

/// Registre des routes d'identité. `/health` et `/ready` (sondes) sont déclarées dans `http.rs`.
pub const ROUTES: &[RouteSpec] = &[
    RouteSpec {
        method: "POST",
        path: "/api/auth/login",
        access: Access::Public,
    },
    RouteSpec {
        method: "POST",
        path: "/api/auth/logout",
        access: Access::Public,
    },
    RouteSpec {
        method: "POST",
        path: "/api/auth/logout-all",
        access: Access::Session,
    },
    RouteSpec {
        method: "GET",
        path: "/api/me",
        access: Access::Session,
    },
    RouteSpec {
        method: "POST",
        path: "/api/me/organization",
        access: Access::Session,
    },
    RouteSpec {
        method: "GET",
        path: "/api/organization",
        access: Access::Action(Action::SettingsRead),
    },
];

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/logout-all", post(logout_all))
        .route("/api/me", get(me))
        .route("/api/me/organization", post(select_organization))
        .route("/api/organization", get(organization))
}

// ───────── corps et réponses ─────────

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LoginPayload {
    /// Téléphone (`+226` et 8 chiffres, formes usuelles acceptées) ou e-mail.
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectOrganizationPayload {
    pub organization_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserBody {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationChoiceBody {
    pub id: String,
    pub name: String,
    pub plan: String,
    pub kind: String,
    pub lifecycle: String,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginBody {
    pub user: UserBody,
    pub organizations: Vec<OrganizationChoiceBody>,
    /// Absente tant que l'utilisateur de plusieurs organisations n'en a pas choisi une.
    pub active_organization_id: Option<String>,
    pub csrf_token: String,
    /// Toujours faux à l'étape 02 (double authentification : étape 03).
    pub mfa_required: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MembershipBody {
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationBody {
    pub id: String,
    pub name: String,
    pub plan: String,
    pub kind: String,
    pub lifecycle: String,
    pub timezone: &'static str,
    pub currency: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeBody {
    pub user: UserBody,
    pub membership: Option<MembershipBody>,
    pub organization: Option<OrganizationBody>,
    pub organizations: Vec<OrganizationChoiceBody>,
    #[schema(value_type = Object)]
    pub features: serde_json::Value,
    /// Codes d'action autorisés (`API_CONTRACTS.md` §4.6), calculés par `authorize`.
    pub permissions: Vec<String>,
    /// Date du jour au fuseau de l'organisation (AAAA-MM-JJ).
    pub today: String,
    pub server_time: String,
    pub csrf_token: String,
    pub min_client_version: String,
    #[schema(value_type = Object)]
    pub settings: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationDetailBody {
    pub name: String,
    pub city: Option<String>,
    pub plan: String,
    pub kind: String,
    pub lifecycle: String,
}

fn choice_body(c: &OrgChoice) -> OrganizationChoiceBody {
    OrganizationChoiceBody {
        id: c.id.to_string(),
        name: c.name.clone(),
        plan: c.plan.as_str().to_string(),
        kind: c.kind.clone(),
        lifecycle: c.lifecycle.clone(),
        role: c.role.as_str().to_string(),
    }
}

fn login_body(g: &SessionGrant) -> LoginBody {
    LoginBody {
        user: UserBody {
            id: g.user.id.to_string(),
            display_name: g.user.display_name.clone(),
        },
        organizations: g.organizations.iter().map(choice_body).collect(),
        active_organization_id: g.active_org.map(|o| o.to_string()),
        csrf_token: g.csrf_token.clone(),
        mfa_required: false,
    }
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

// ───────── POST /api/auth/login ─────────

/// Connexion. Réponse identique que l'identifiant existe ou non (`auth.invalid_credentials`) ; 5 échecs consécutifs : attente
/// croissante (`auth.rate_limited`, `Retry-After`). Pose le cookie `__Host-session`.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Connecté ; cookie de session posé", body = LoginBody),
        (status = 401, description = "auth.invalid_credentials"),
        (status = 403, description = "auth.no_active_membership"),
        (status = 429, description = "auth.rate_limited, avec Retry-After")
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    ApiJson(body): ApiJson<LoginPayload>,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &ctx)?;
    let result = svc
        .login(LoginRequest {
            identifier: body.identifier,
            password: body.password,
            client: ctx.client.clone(),
        })
        .await;
    match result {
        Ok(grant) => {
            let cookie = session_cookie(&grant.session_token, ABSOLUTE_HOURS * 3600);
            Ok(with_cookie(Json(login_body(&grant)).into_response(), &cookie))
        }
        Err(LoginError::InvalidCredentials) => Err(ctx.err(ApiError::invalid_credentials())),
        Err(LoginError::RateLimited { retry_after_seconds }) => {
            Err(ctx.err(ApiError::rate_limited(retry_after_seconds)))
        }
        Err(LoginError::NoActiveMembership) => Err(ctx.err(ApiError::no_active_membership())),
        Err(LoginError::Internal) => Err(ctx.unexpected()),
    }
}

// ───────── POST /api/auth/logout, /api/auth/logout-all ─────────

/// Déconnexion de cet appareil. Idempotente : une session absente ou expirée répond aussi 204 et efface le cookie.
#[utoipa::path(post, path = "/api/auth/logout", responses((status = 204, description = "Déconnecté"), (status = 403, description = "auth.csrf_invalid")))]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    ctx: Ctx,
    request: axum::extract::Request,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &ctx)?;
    let (mut parts, _body) = request.into_parts();
    if let Some(token) = session_token(&parts.headers) {
        // une session valide exige les mêmes protections qu'une écriture ; sans session valide, on se contente d'effacer le cookie
        match <Session as axum::extract::FromRequestParts<Arc<AppState>>>::from_request_parts(&mut parts, &state).await
        {
            Ok(session) => {
                svc.logout(&session.token).await.map_err(|_| ctx.unexpected())?;
            }
            Err(e) if e.code() == "auth.csrf_invalid" => return Err(e),
            Err(_) => {
                let _ = svc.logout(&token).await;
            }
        }
    }
    Ok(with_cookie(
        StatusCode::NO_CONTENT.into_response(),
        &clear_session_cookie(),
    ))
}

/// Déconnecte tous les appareils de l'utilisateur (`session.revoked_all`).
#[utoipa::path(post, path = "/api/auth/logout-all", responses((status = 204, description = "Toutes les sessions révoquées"), (status = 401, description = "auth.unauthenticated"), (status = 403, description = "auth.csrf_invalid")))]
pub async fn logout_all(State(state): State<Arc<AppState>>, session: Session) -> Result<Response, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    svc.logout_all(&session.token, &session.ctx.client)
        .await
        .map_err(|e| match e {
            AuthError::Unauthenticated => session.ctx.err(ApiError::session_expired()),
            AuthError::Internal => session.ctx.unexpected(),
        })?;
    Ok(with_cookie(
        StatusCode::NO_CONTENT.into_response(),
        &clear_session_cookie(),
    ))
}

// ───────── GET /api/me ─────────

async fn me_body(state: &AppState, token: &str, auth: &Authenticated, ctx: &Ctx) -> Result<MeBody, ApiError> {
    let svc = identity(state, ctx)?;
    let organizations = svc.organizations_of(auth.user.id).await.map_err(|_| ctx.unexpected())?;
    let now = svc.clock().now();
    let (membership, organization, permissions) = match (&auth.org, auth.role) {
        (Some(org), Some(role)) => {
            let permissions = Action::ALL
                .iter()
                .filter(|a| {
                    authorize(role, org.plan, **a)
                        != Decision::Deny(immo_app::identity::authorize::DenyReason::Forbidden)
                })
                .filter(|a| authorize(role, org.plan, **a).is_allowed_without_settings())
                .map(|a| a.code().to_string())
                .collect();
            (
                Some(MembershipBody {
                    role: role.as_str().to_string(),
                }),
                Some(OrganizationBody {
                    id: org.id.to_string(),
                    name: org.name.clone(),
                    plan: org.plan.as_str().to_string(),
                    kind: org.kind.clone(),
                    lifecycle: org.lifecycle.clone(),
                    timezone: "Africa/Ouagadougou",
                    currency: "XOF",
                }),
                permissions,
            )
        }
        _ => (None, None, vec![]),
    };
    Ok(MeBody {
        user: UserBody {
            id: auth.user.id.to_string(),
            display_name: auth.user.display_name.clone(),
        },
        membership,
        organization,
        organizations: organizations.iter().map(choice_body).collect(),
        features: json!({}),
        permissions,
        today: svc.clock().local_date(OUAGADOUGOU).to_string(),
        server_time: now.to_rfc3339(),
        csrf_token: svc.csrf_for(token),
        min_client_version: state.min_client_version.clone(),
        settings: json!({}),
    })
}

/// Profil de la session : utilisateur, rôle, organisation, droits, date du jour de l'organisation, jeton anti-falsification.
#[utoipa::path(get, path = "/api/me", responses((status = 200, description = "Profil", body = MeBody), (status = 401, description = "auth.unauthenticated")))]
pub async fn me(State(state): State<Arc<AppState>>, session: Session) -> Result<Json<MeBody>, ApiError> {
    Ok(Json(
        me_body(&state, &session.token, &session.auth, &session.ctx).await?,
    ))
}

// ───────── POST /api/me/organization ─────────

/// Change l'organisation active. L'adhésion est revérifiée : une organisation dont on n'est pas membre répond 404 (et, si elle
/// existe chez un autre, `security.cross_org_attempt` est écrit chez l'organisation active de l'appelant).
#[utoipa::path(
    post,
    path = "/api/me/organization",
    request_body = SelectOrganizationPayload,
    responses((status = 200, description = "Organisation active changée ; nouveau cookie", body = MeBody), (status = 404, description = "resource.not_found"))
)]
pub async fn select_organization(
    State(state): State<Arc<AppState>>,
    session: Session,
    ApiJson(body): ApiJson<SelectOrganizationPayload>,
) -> Result<Response, ApiError> {
    let svc = identity(&state, &session.ctx)?;
    let Ok(org_id) = Uuid::parse_str(body.organization_id.trim()) else {
        return Err(session.ctx.err(ApiError::malformed()));
    };
    let grant = svc
        .select_organization(&session.token, org_id, &session.ctx.client)
        .await
        .map_err(|e| match e {
            SelectError::NotFound => session.ctx.err(ApiError::not_found()),
            SelectError::Unauthenticated => session.ctx.err(ApiError::session_expired()),
            SelectError::Internal => session.ctx.unexpected(),
        })?;
    let auth = svc
        .authenticate(&grant.session_token)
        .await
        .map_err(|_| session.ctx.unexpected())?;
    let me = me_body(&state, &grant.session_token, &auth, &session.ctx).await?;
    let cookie = session_cookie(&grant.session_token, ABSOLUTE_HOURS * 3600);
    Ok(with_cookie(Json(me).into_response(), &cookie))
}

// ───────── GET /api/organization ─────────

/// Fiche de l'organisation active (droit `settings.read` ; lecture partielle pour les rôles sans droit d'écriture).
#[utoipa::path(
    get,
    path = "/api/organization",
    responses((status = 200, description = "Organisation", body = OrganizationDetailBody), (status = 403, description = "auth.forbidden")))]
pub async fn organization(
    State(state): State<Arc<AppState>>,
    session: Session,
) -> Result<Json<OrganizationDetailBody>, ApiError> {
    session.require(Action::SettingsRead)?;
    let svc = identity(&state, &session.ctx)?;
    let summary = svc
        .organization_summary(&session.auth)
        .await
        .map_err(|_| session.ctx.unexpected())?;
    Ok(Json(OrganizationDetailBody {
        name: summary.name,
        city: summary.city,
        plan: summary.plan.as_str().to_string(),
        kind: summary.kind,
        lifecycle: summary.lifecycle,
    }))
}
