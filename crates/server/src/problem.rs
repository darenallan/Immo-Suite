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
    /// Erreurs de validation par champ (`API_CONTRACTS.md` §3.3).
    errors: Option<Value>,
    /// En-tête `Allow` d'une réponse 405.
    allow: Option<&'static str>,
    /// Cadres de pile de notre code, capturés à la création d'une erreur inattendue (F39-observation, règle 12).
    frames: Option<Vec<String>>,
}

/// Indice posé sur la réponse d'une erreur inattendue (500) : l'intergiciel d'observation l'agrège par empreinte.
#[derive(Debug, Clone)]
pub struct ServerErrorInfo {
    pub frames: Vec<String>,
}

/// Indice posé sur la réponse d'un refus de droit (`auth.forbidden`) : l'intergiciel écrit `permission_denied`.
#[derive(Debug, Clone)]
pub struct DeniedInfo {
    pub action: String,
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
            errors: None,
            allow: None,
            frames: None,
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

    // ───────── étape 03 (DP-88) ─────────

    pub fn reauth_required() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.reauth_required",
            "Confirmation requise",
            "Confirmez votre mot de passe pour continuer.",
        )
    }

    pub fn totp_invalid() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth.totp_invalid",
            "Code refusé",
            "Code incorrect. Vérifiez l'heure de votre téléphone.",
        )
    }

    pub fn identifier_taken() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "auth.identifier_taken",
            "Identifiant déjà utilisé",
            "Cet identifiant a déjà un compte. Voulez-vous vous connecter ?",
        )
    }

    pub fn password_weak(reason: &'static str) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "auth.password_weak",
            "Mot de passe refusé",
            match reason {
                "too_short" => "Au moins 10 caractères.",
                "too_long" => "128 caractères au plus.",
                "common" => "Ce mot de passe est trop courant. Choisissez-en un autre.",
                "contains_identifier" => "Le mot de passe ne doit pas contenir votre identifiant.",
                _ => "Choisissez un mot de passe différent de l'actuel.",
            },
        )
        .with_params(json!({ "reason": reason }))
    }

    pub fn invitation_expired() -> Self {
        Self::new(
            StatusCode::GONE,
            "invitation.expired",
            "Invitation indisponible",
            "Cette invitation n'est plus valable. Demandez au gérant de la renvoyer.",
        )
    }

    pub fn invitation_already_pending() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "invitation.already_pending",
            "Invitation déjà en attente",
            "Une invitation est déjà en attente pour cet identifiant. Renvoyez-la.",
        )
    }

    pub fn reset_expired() -> Self {
        Self::new(
            StatusCode::GONE,
            "auth.reset_expired",
            "Lien indisponible",
            "Ce lien n'est plus valable. Demandez-en un nouveau à votre gérant.",
        )
    }

    pub fn last_manager() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "user.last_manager",
            "Dernier gérant",
            "Vous ne pouvez pas retirer le dernier gérant. Nommez d'abord un autre gérant.",
        )
    }

    pub fn version_mismatch() -> Self {
        Self::new(
            StatusCode::PRECONDITION_FAILED,
            "concurrency.version_mismatch",
            "Fiche modifiée",
            "Cette fiche a été modifiée entre-temps. Rechargez-la avant de continuer.",
        )
    }

    pub fn version_required() -> Self {
        Self::new(
            StatusCode::PRECONDITION_REQUIRED,
            "concurrency.version_required",
            "Version requise",
            "Un problème est survenu. Réessayez.",
        )
    }

    pub fn cursor_invalid() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "request.cursor_invalid",
            "Curseur invalide",
            "Un problème est survenu. Réessayez.",
        )
    }

    pub fn validation_reason(reason: &'static str) -> Self {
        Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation.failed",
            "Saisie à corriger",
            "Une ou plusieurs informations sont à corriger.",
        )
        .with_params(json!({ "reason": reason }))
    }

    pub fn validation_fields(issues: &[immo_app::identity::error::FieldIssue]) -> Self {
        let mut e = Self::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation.failed",
            "Saisie à corriger",
            "Une ou plusieurs informations sont à corriger.",
        );
        e.0.errors = Some(Value::Array(
            issues
                .iter()
                .map(|i| json!({ "field": i.field, "code": i.code, "params": {} }))
                .collect(),
        ));
        e
    }

    /// `403 auth.forbidden` pour une règle propre à l'opération (le détail dit quoi faire).
    pub fn forbidden_reason(reason: &'static str) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "auth.forbidden",
            "Droit refusé",
            match reason {
                "self_removal" => "Demandez à un autre gérant.",
                "manager_reset_support" => "Pour un gérant, contactez le support HABEK.",
                "use_own_password" => "Pour votre propre mot de passe, utilisez « Changer mon mot de passe ».",
                "other_account" => "Cette invitation est destinée à un autre compte.",
                _ => "Vous n'avez pas le droit de faire cela avec votre rôle.",
            },
        )
        .with_params(json!({ "reason": reason }))
    }

    /// `405 audit.immutable` : le journal ne se modifie pas.
    pub fn audit_immutable() -> Self {
        let mut e = Self::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "audit.immutable",
            "Journal en lecture seule",
            "Le journal d'audit ne peut pas être modifié.",
        );
        e.0.allow = Some("GET");
        e
    }

    /// Traduction d'une erreur du service d'identité en erreur d'API stable.
    pub fn from_service(e: immo_app::identity::error::ServiceError, request_id: &str) -> Self {
        use immo_app::identity::error::ServiceError as S;
        let api = match e {
            S::Unauthenticated => Self::session_expired(),
            S::Internal => return Self::unexpected(request_id),
            S::Forbidden(reason) => Self::forbidden_reason(reason),
            S::ReauthRequired => Self::reauth_required(),
            S::NotFound => Self::not_found(),
            S::Invalid(reason) => Self::validation_reason(reason),
            S::Fields(issues) => Self::validation_fields(&issues),
            S::PasswordWeak(reason) => Self::password_weak(reason),
            S::InvalidCredentials => Self::invalid_credentials(),
            S::NoActiveMembership => Self::no_active_membership(),
            S::RateLimited { retry_after_seconds } => Self::rate_limited(retry_after_seconds),
            S::IdentifierTaken => Self::identifier_taken(),
            S::AlreadyPending => Self::invitation_already_pending(),
            S::InvitationExpired => Self::invitation_expired(),
            S::ResetExpired => Self::reset_expired(),
            S::LastManager => Self::last_manager(),
            S::VersionMismatch => Self::version_mismatch(),
            S::VersionRequired => Self::version_required(),
            S::BadCursor => Self::cursor_invalid(),
            S::TotpInvalid => Self::totp_invalid(),
        };
        api.with_request_id(request_id)
    }

    pub fn unexpected(request_id: &str) -> Self {
        // la pile est capturée ici, là où l'erreur naît ; seuls les noms de fonctions de notre code sont gardés
        let frames = immo_app::observation::errors::frames_of(&std::backtrace::Backtrace::force_capture().to_string());
        let mut e = Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server.unexpected",
            "Erreur inattendue",
            format!("Un problème est survenu. Réessayez. Si le problème continue, communiquez le code {request_id} au support."),
        )
        .with_request_id(request_id);
        e.0.frames = Some(frames);
        e
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let p = *self.0;
        let denied_action = p
            .params
            .as_ref()
            .and_then(|v| v.get("action"))
            .and_then(Value::as_str)
            .map(str::to_string);
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
            if let Some(errors) = p.errors {
                map.insert("errors".to_string(), errors);
            }
        }
        let mut response = (p.status, Json(body)).into_response();
        if let Some(frames) = p.frames {
            response.extensions_mut().insert(ServerErrorInfo { frames });
        }
        if p.code == "auth.forbidden"
            && let Some(action) = denied_action
        {
            response.extensions_mut().insert(DeniedInfo { action });
        }
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if let Some(seconds) = p.retry_after
            && let Ok(v) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, v);
        }
        if let Some(allow) = p.allow {
            response
                .headers_mut()
                .insert(header::ALLOW, HeaderValue::from_static(allow));
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
