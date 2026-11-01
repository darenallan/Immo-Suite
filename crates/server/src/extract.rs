//! Extracteurs HTTP : contexte de requête, session, écriture protégée (jeton anti-falsification et `Origin`), corps JSON strict.
//!
//! L'organisation active est **uniquement** celle de la session : elle n'est jamais lue dans l'adresse, le corps ou un en-tête
//! (API_CONTRACTS.md §4.3). Le rôle est relu en base à chaque requête (F01 règle 9).

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{ConnectInfo, FromRequest, FromRequestParts, MatchedPath, Request};
use axum::http::header::{COOKIE, ORIGIN, USER_AGENT};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method};
use immo_app::identity::authorize::{Action, Decision, DenyReason, authorize, required_roles};
use immo_app::identity::service::{AuthError, Authenticated, ClientInfo};
use immo_app::identity::session::COOKIE_NAME;
use serde::de::DeserializeOwned;

use crate::http::AppState;
use crate::problem::{ApiError, RequestId};

/// Adresse du client : la dernière entrée de `X-Forwarded-For` (ajoutée par Caddy) quand le proxy est de confiance, sinon
/// l'adresse de la connexion. Une entrée placée par le client plus tôt dans la liste est ignorée.
pub fn client_ip(headers: &HeaderMap, peer: Option<IpAddr>, trust_proxy: bool) -> Option<IpAddr> {
    if trust_proxy
        && let Some(last) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.rsplit(',').next())
            .and_then(|v| v.trim().parse::<IpAddr>().ok())
    {
        return Some(last);
    }
    peer
}

/// Contexte d'une requête : identifiant d'incident, adresse, appareil.
#[derive(Debug, Clone)]
pub struct Ctx {
    pub request_id: String,
    pub client: ClientInfo,
}

impl Ctx {
    /// Erreur API portant le code d'incident de cette requête.
    pub fn err(&self, e: ApiError) -> ApiError {
        e.with_request_id(&self.request_id)
    }

    pub fn unexpected(&self) -> ApiError {
        ApiError::unexpected(&self.request_id)
    }

    /// Erreur du service d'identité, traduite au format du contrat avec le code d'incident de cette requête.
    pub fn service(&self, e: immo_app::identity::error::ServiceError) -> ApiError {
        ApiError::from_service(e, &self.request_id)
    }
}

impl FromRequestParts<Arc<AppState>> for Ctx {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .map_or_else(crate::problem::new_request_id, |r| r.0.clone());
        let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip());
        let ip = client_ip(&parts.headers, peer, state.trust_proxy);
        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        Ok(Ctx {
            client: ClientInfo {
                ip,
                user_agent,
                request_id: Some(request_id.clone()),
            },
            request_id,
        })
    }
}

/// Valeur du cookie de session, si présente.
pub fn session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value.to_string())
}

/// Requête authentifiée par cookie. Les écritures (`POST`, `PUT`, `PATCH`, `DELETE`) exigent en plus `Origin` et `X-CSRF-Token`
/// (API_CONTRACTS.md §4.2) ; absent ou faux : `403 auth.csrf_invalid` et une ligne `security.permission_denied`.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub auth: Authenticated,
    pub ctx: Ctx,
}

impl Session {
    /// Applique `authorize` : refus par défaut, `403 auth.forbidden` ou `403 plan.feature_unavailable`. Un droit réglable est
    /// accordé si son réglage est à « oui » (`role_settings`, lu à chaque requête). Sans organisation active :
    /// `403 auth.organization_required`.
    pub fn require(&self, action: Action) -> Result<Decision, ApiError> {
        let (Some(org), Some(role)) = (&self.auth.org, self.auth.role) else {
            return Err(self.ctx.err(ApiError::organization_required()));
        };
        if org.lifecycle == "suspended" {
            return Err(self.ctx.err(ApiError::org_suspended()));
        }
        let decision = authorize(role, org.plan, action);
        if decision.is_allowed_without_settings() {
            return Ok(decision);
        }
        // droit réglable : accordé si le réglage est à « oui », lu à CETTE requête (F02 §7.1 points 4 et 6)
        if let Decision::NeedsSetting(key) = decision
            && self.auth.granted_settings.iter().any(|s| s == key)
        {
            return Ok(Decision::Allow);
        }
        Err(self.ctx.err(match decision {
            Decision::Deny(DenyReason::PlanUnavailable) => ApiError::plan_unavailable(),
            _ => {
                let roles: Vec<&str> = required_roles(action).into_iter().map(|r| r.as_str()).collect();
                ApiError::forbidden(action.code(), &roles)
            }
        }))
    }
}

fn is_write(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

impl FromRequestParts<Arc<AppState>> for Session {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let ctx = Ctx::from_request_parts(parts, state).await?;
        let Some(identity) = state.identity.as_ref() else {
            return Err(ctx.unexpected());
        };
        let Some(token) = session_token(&parts.headers) else {
            return Err(ctx.err(ApiError::unauthenticated()));
        };
        let auth = identity.authenticate(&token).await.map_err(|e| match e {
            AuthError::Unauthenticated => ctx.err(ApiError::session_expired()),
            AuthError::Internal => ctx.unexpected(),
        })?;
        let session = Session { token, auth, ctx };
        if is_write(&parts.method) {
            let origin_ok = parts
                .headers
                .get(ORIGIN)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|o| state.allowed_origins.iter().any(|a| a == o));
            let csrf_ok = parts
                .headers
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|c| identity.csrf_valid(&session.token, c));
            if !(origin_ok && csrf_ok) {
                let route = parts
                    .extensions
                    .get::<MatchedPath>()
                    .map_or("unmatched", MatchedPath::as_str)
                    .to_string();
                let reason = if origin_ok { "csrf_invalid" } else { "origin_invalid" };
                identity
                    .trace_permission_denied(&session.auth, &route, reason, &session.ctx.client)
                    .await;
                return Err(session.ctx.err(ApiError::csrf_invalid()));
            }
        }
        Ok(session)
    }
}

/// Corps JSON strict : champs inconnus refusés, taille bornée par la couche (`DefaultBodyLimit`), erreurs au format du contrat.
pub struct ApiJson<T>(pub T);

impl<T> FromRequest<Arc<AppState>> for ApiJson<T>
where
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let ctx = Ctx::from_request_parts(&mut parts, state).await?;
        let req = Request::from_parts(parts, body);
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ApiJson(value)),
            Err(JsonRejection::BytesRejection(_)) => Err(ctx.err(ApiError::too_large())),
            Err(rejection) => {
                // le message de serde cite les noms de champs, jamais les valeurs ; on ne le renvoie pas
                let unknown = rejection.body_text().contains("unknown field");
                Err(ctx.err(if unknown {
                    ApiError::unknown_field()
                } else {
                    ApiError::malformed()
                }))
            }
        }
    }
}
