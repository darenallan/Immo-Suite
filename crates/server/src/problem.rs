//! Erreurs d'API au format `application/problem+json` (RFC 9457), avec un code stable (`API_CONTRACTS.md` §3).
//!
//! Le front s'appuie sur `code`, jamais sur `title` ni `detail`. Aucune erreur ne cite un nom de table, une pile d'appels ou une valeur
//! d'une autre organisation (SECURITY.md §7.1).

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

/// Identifiant de requête (« code d'incident »), posé par l'intergiciel d'accès et renvoyé dans `X-Request-Id` et dans l'erreur.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Code d'incident court de la forme `INC-7K3F9A` (alphabet sans caractères ambigus).
pub fn new_request_id() -> String {
    const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let mut out = String::from("INC-");
    for byte in bytes.iter().take(6) {
        let index = usize::from(*byte & 0x1f);
        if let Some(c) = ALPHABET.get(index) {
            out.push(char::from(*c));
        }
    }
    out
}

#[derive(Debug, Clone)]
struct Problem {
    status: StatusCode,
    code: &'static str,
    title: &'static str,
    detail: String,
    params: Option<Value>,
    retry_after: Option<i64>,
    request_id: Option<String>,
}

/// Erreur d'API au format du contrat. Le contenu est dans une boîte : `Result<_, ApiError>` reste petit.
#[derive(Debug, Clone)]
pub struct ApiError(Box<Problem>);

/// « 30 secondes », « 1 minute », « 15 minutes » : durée arrondie par excès, pour le message de limitation.
pub fn duration_fr(seconds: i64) -> String {
    if seconds < 60 {
        let s = seconds.max(1);
        return format!("{s} seconde{}", if s > 1 { "s" } else { "" });
    }
    let minutes = (seconds + 59) / 60;
    format!("{minutes} minute{}", if minutes > 1 { "s" } else { "" })
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, title: &'static str, detail: impl Into<String>) -> Self {
        Self(Box::new(Problem {
            status,
            code,
            title,
            detail: detail.into(),
            params: None,
            retry_after: None,
            request_id: None,
        }))
    }

    pub fn code(&self) -> &'static str {
        self.0.code
    }

    pub fn status(&self) -> StatusCode {
        self.0.status
    }

    pub fn with_request_id(mut self, id: &str) -> Self {
        self.0.request_id = Some(id.to_string());
        self
    }

    fn with_params(mut self, params: Value) -> Self {
        self.0.params = Some(params);
        self
    }

    pub fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth.unauthenticated",
            "Non connecté",
            "Votre session a expiré. Reconnectez-vous.",
        )
    }

    pub fn session_expired() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth.session_expired",
            "Session expirée",
            "Votre session a expiré. Reconnectez-vous.",
        )
    }

    pub fn invalid_credentials() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth.invalid_credentials",
            "Identifiants refusés",
            "Identifiant ou mot de passe incorrect.",
        )
    }

    pub fn rate_limited(seconds: i64) -> Self {
        let mut e = Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "auth.rate_limited",
            "Trop de tentatives",
            format!(
                "Trop de tentatives. Réessayez dans {} ou réinitialisez votre mot de passe.",
                duration_fr(seconds)
            ),
        )
        .with_params(json!({ "retry_after_seconds": seconds }));
        e.0.retry_after = Some(seconds);
        e
    }

    pub fn request_rate_limited(seconds: i64) -> Self {
        let mut e = Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit.exceeded",
            "Trop de demandes",
            format!("Trop de demandes. Réessayez dans {}.", duration_fr(seconds)),
        )
        .with_params(json!({ "retry_after_seconds": seconds }));
        e.0.retry_after = Some(seconds);
        e
    }

    pub fn csrf_invalid() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.csrf_invalid",
            "Requête refusée",
            "Votre session n'est plus valide pour cette action. Rechargez la page.",
        )
    }

    pub fn forbidden(action: &str, required_roles: &[&str]) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.forbidden",
            "Droit refusé",
            "Vous n'avez pas le droit de faire cela avec votre rôle.",
        )
        .with_params(json!({ "action": action, "required_roles": required_roles }))
    }

    pub fn plan_unavailable() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "plan.feature_unavailable",
            "Fonction indisponible",
            "Cette fonction n'est pas disponible pour votre organisation.",
        )
    }

    pub fn organization_required() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.organization_required",
            "Organisation à choisir",
            "Choisissez une organisation pour continuer.",
        )
    }

    pub fn org_suspended() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "org.suspended",
            "Organisation suspendue",
            "Cette organisation est suspendue. Contactez le support.",
        )
    }

    pub fn no_active_membership() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.no_active_membership",
            "Aucun accès",
            "Votre compte n'a accès à aucune organisation. Contactez votre gérant.",
        )
    }

    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "resource.not_found",
            "Introuvable",
            "Introuvable. Cette page n'existe pas ou vous n'y avez pas accès.",
        )
    }

    pub fn malformed() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "request.malformed",
            "Requête mal formée",
            "Un problème est survenu. Réessayez.",
        )
    }

    pub fn unknown_field() -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "request.unknown_field",
            "Champ inconnu",
            "Un problème est survenu. Réessayez.",
        )
    }

    pub fn too_large() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "request.too_large",
            "Requête trop volumineuse",
            "Un problème est survenu. Réessayez.",
        )
    }

    pub fn upgrade_required() -> Self {
        Self::new(
            StatusCode::UPGRADE_REQUIRED,
            "client.upgrade_required",
            "Mise à jour requise",
            "Une mise à jour est disponible.",
        )
    }

    pub fn unexpected(request_id: &str) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server.unexpected",
            "Erreur inattendue",
            format!("Un problème est survenu. Réessayez. Si le problème continue, communiquez le code {request_id} au support."),
        )
        .with_request_id(request_id)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let p = *self.0;
        let mut body = json!({
            "type": "about:blank",
            "title": p.title,
            "status": p.status.as_u16(),
            "detail": p.detail,
            "code": p.code,
        });
        if let Some(map) = body.as_object_mut() {
            if let Some(params) = p.params {
                map.insert("params".to_string(), params);
            }
            if let Some(r) = &p.request_id {
                map.insert("request_id".to_string(), Value::String(r.clone()));
            }
            if let Some(s) = p.retry_after {
                map.insert("retry_after_seconds".to_string(), json!(s));
            }
        }
        let mut response = (p.status, Json(body)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if let Some(seconds) = p.retry_after
            && let Ok(v) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, v);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_code_d_incident_a_la_forme_inc_et_six_caracteres() {
        for _ in 0..50 {
            let id = new_request_id();
            assert!(id.starts_with("INC-") && id.len() == 10, "{id}");
            assert!(
                id[4..].chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase()),
                "{id}"
            );
            assert!(
                !id[4..].contains(['I', 'L', 'O', 'U']),
                "alphabet sans ambiguïté : {id}"
            );
        }
    }

    #[test]
    fn durees_en_francais_arrondies_par_exces() {
        assert_eq!(duration_fr(1), "1 seconde");
        assert_eq!(duration_fr(30), "30 secondes");
        assert_eq!(duration_fr(60), "1 minute");
        assert_eq!(duration_fr(61), "2 minutes");
        assert_eq!(duration_fr(300), "5 minutes");
        assert_eq!(duration_fr(900), "15 minutes");
    }
}
