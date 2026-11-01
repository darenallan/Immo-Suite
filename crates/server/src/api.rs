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
    /// Sans session, mais protégée par un jeton à usage unique ou un défi signé (invitation, réinitialisation, second facteur).
    Token,
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

/// Registre des routes d'identité. `/health` et `/ready` (sondes) sont déclarées dans `http.rs`. Toute route y déclare l'accès
/// qu'elle exige : aucune ne peut exister sans déclaration (F02 §7.5). Les gestionnaires sont dans `api_account` et `api_admin`.
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
        method: "POST",
        path: "/api/auth/totp/verify",
        access: Access::Token,
    },
    RouteSpec {
        method: "POST",
        path: "/api/auth/reauth",
        access: Access::Session,
    },
    RouteSpec {
        method: "POST",
        path: "/api/auth/password-reset/complete",
        access: Access::Token,
    },
    RouteSpec {
        method: "GET",
        path: "/api/me",
        access: Access::Session,
    },
    RouteSpec {
        method: "PATCH",
        path: "/api/me",
        access: Access::Session,
    },
    RouteSpec {
        method: "POST",
        path: "/api/me/organization",
        access: Access::Session,
    },
    RouteSpec {
        method: "PUT",
        path: "/api/me/password",
        access: Access::Session,
    },
    RouteSpec {
        method: "POST",
        path: "/api/me/totp",
        access: Access::Session,
    },
    RouteSpec {
        method: "POST",
        path: "/api/me/totp/confirm",
        access: Access::Session,
    },
    RouteSpec {
        method: "GET",
        path: "/api/organization",
        access: Access::Action(Action::SettingsRead),
    },
    RouteSpec {
        method: "PATCH",
        path: "/api/organization",
        access: Access::Action(Action::SettingsWrite),
    },
    RouteSpec {
        method: "GET",
        path: "/api/settings",
        access: Access::Action(Action::SettingsRead),
    },
    RouteSpec {
        method: "PATCH",
        path: "/api/settings",
        access: Access::Action(Action::SettingsWrite),
    },
    RouteSpec {
        method: "GET",
        path: "/api/users",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "PATCH",
        path: "/api/users/{membership_id}",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/users/{membership_id}/disable",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/users/{membership_id}/enable",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "PUT",
        path: "/api/users/{membership_id}/unit-assignments",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/users/{membership_id}/password-resets",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "GET",
        path: "/api/invitations",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/invitations",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/invitations/accept",
        access: Access::Token,
    },
    RouteSpec {
        method: "GET",
        path: "/api/invitations/{token}",
        access: Access::Token,
    },
    RouteSpec {
        method: "POST",
        path: "/api/invitations/{id}/revoke",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "POST",
        path: "/api/invitations/{id}/renew",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "GET",
        path: "/api/role-settings",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "PUT",
        path: "/api/role-settings",
        access: Access::Action(Action::UserManage),
    },
    RouteSpec {
        method: "GET",
        path: "/api/support-grants",
        access: Access::Action(Action::SupportGrant),
    },
    RouteSpec {
        method: "POST",
        path: "/api/support-grants",
        access: Access::Action(Action::SupportGrant),
    },
    RouteSpec {
        method: "POST",
        path: "/api/support-grants/{id}/revoke",
        access: Access::Action(Action::SupportGrant),
    },
    RouteSpec {
        method: "GET",
        path: "/api/audit",
        access: Access::Action(Action::AuditRead),
    },
    RouteSpec {
        method: "POST",
        path: "/api/events",
        access: Access::Action(Action::EventsCreate),
    },
    RouteSpec {
        method: "POST",
        path: "/api/client-errors",
        access: Access::Action(Action::EventsCreate),
    },
    RouteSpec {
        method: "POST",
        path: "/api/feedback",
        access: Access::Action(Action::FeedbackCreate),
    },
];
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/logout-all", post(logout_all))
        .route("/api/me", get(me).patch(crate::api_account::update_me))
        .route("/api/me/organization", post(select_organization))
        .route(
            "/api/organization",
            get(organization).patch(crate::api_admin::patch_organization),
        )
        .merge(crate::api_account::routes())
        .merge(crate::api_admin::routes())
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
    /// Absent quand un second facteur est exigé : aucune session n'est ouverte avant le code.
    pub user: Option<UserBody>,
    pub organizations: Vec<OrganizationChoiceBody>,
    /// Absente tant que l'utilisateur de plusieurs organisations n'en a pas choisi une.
    pub active_organization_id: Option<String>,
    /// Absent quand un second facteur est exigé.
    pub csrf_token: Option<String>,
    /// Vrai si le compte a une double authentification : le défi se présente à `POST /auth/totp/verify` avec le code.
    pub mfa_required: bool,
    /// Défi signé valable 5 minutes (F01 §9.5), présent seulement si `mfa_required`.
    pub challenge: Option<String>,
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
    /// Version pour `If-Match` (`PATCH /organization`).
    pub version: i32,
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

pub(crate) fn login_body(g: &SessionGrant) -> LoginBody {
    LoginBody {
        user: Some(UserBody {
            id: g.user.id.to_string(),
            display_name: g.user.display_name.clone(),
        }),
        organizations: g.organizations.iter().map(choice_body).collect(),
        active_organization_id: g.active_org.map(|o| o.to_string()),
        csrf_token: Some(g.csrf_token.clone()),
        mfa_required: false,
        challenge: None,
    }
}

fn mfa_body(challenge: String) -> LoginBody {
    LoginBody {
        user: None,
        organizations: vec![],
        active_organization_id: None,
        csrf_token: None,
        mfa_required: true,
        challenge: Some(challenge),
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
        Err(LoginError::MfaRequired { challenge }) => Ok(Json(mfa_body(challenge)).into_response()),
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
            // un droit réglable est compté si son réglage est à « oui » à cette requête (F02 §7.1 point 5)
            let permissions = Action::ALL
                .iter()
                .filter(|a| match authorize(role, org.plan, **a) {
                    Decision::NeedsSetting(key) => auth.granted_settings.iter().any(|s| s == key),
                    d => d.is_allowed_without_settings(),
                })
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
    // les vrais réglages (F01 §9.8) ; une session sans organisation active n'en a pas
    let settings = match &auth.org {
        Some(_) => {
            let s = svc.org_settings(auth).await.map_err(|e| ctx.service(e))?;
            json!({
                "payment_confirm_threshold": s.payment_confirm_threshold,
                "default_paper": s.default_paper,
                "share_link_ttl_days": s.share_link_ttl_days,
            })
        }
        None => json!({}),
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
        settings,
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
    let detail = svc
        .organization_detail(&session.auth)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(org_body(detail)))
}

pub(crate) fn org_body(d: immo_app::identity::settings::OrgDetail) -> OrganizationDetailBody {
    OrganizationDetailBody {
        name: d.name,
        city: d.city,
        plan: d.plan.as_str().to_string(),
        kind: d.kind,
        lifecycle: d.lifecycle,
        version: d.version,
    }
}
