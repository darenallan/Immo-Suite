//! Erreurs des opérations de compte, de membres, de paramètres et d'accès support (étape 03, DP-88).
//!
//! L'API les traduit en codes stables (`API_CONTRACTS.md` §3.4). Aucune variante ne porte de valeur saisie (A13).

use super::password::PasswordError;
use crate::platform::db::DbError;

/// Une erreur de champ : chemin pointé et code (`validation.*`), renvoyée avec toutes les autres (API_CONTRACTS.md §3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldIssue {
    pub field: &'static str,
    pub code: &'static str,
}

#[derive(Debug)]
pub enum ServiceError {
    /// Session absente, expirée ou défi illisible (`auth.session_expired`).
    Unauthenticated,
    Internal,
    /// `403 auth.forbidden` pour une règle propre à l'opération (auto-retrait, réinitialisation d'un gérant…).
    Forbidden(&'static str),
    /// `403 auth.reauth_required` : le mot de passe n'a pas été confirmé depuis moins de 5 minutes.
    ReauthRequired,
    NotFound,
    /// `422 validation.failed` avec `params.reason`.
    Invalid(&'static str),
    /// `422 validation.failed` avec la liste des champs fautifs.
    Fields(Vec<FieldIssue>),
    /// `422 auth.password_weak` ; `reason` parmi `too_short`, `too_long`, `common`, `contains_identifier`, `same_as_current`.
    PasswordWeak(&'static str),
    InvalidCredentials,
    /// `403 auth.no_active_membership`.
    NoActiveMembership,
    RateLimited {
        retry_after_seconds: i64,
    },
    /// `409 auth.identifier_taken`.
    IdentifierTaken,
    /// `409 invitation.already_pending`.
    AlreadyPending,
    /// `410 invitation.expired` : expirée, révoquée ou déjà utilisée (même corps pour les trois).
    InvitationExpired,
    /// `410 auth.reset_expired`.
    ResetExpired,
    /// `409 user.last_manager`.
    LastManager,
    /// `412 concurrency.version_mismatch`.
    VersionMismatch,
    /// `428 concurrency.version_required`.
    VersionRequired,
    /// `400 request.cursor_invalid`.
    BadCursor,
    /// `401 auth.totp_invalid`.
    TotpInvalid,
}

impl From<DbError> for ServiceError {
    fn from(e: DbError) -> Self {
        tracing::error!(error = %e, "identité : erreur de base de données");
        ServiceError::Internal
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(e: sqlx::Error) -> Self {
        DbError::Query(e).into()
    }
}

/// Raison publique d'un mot de passe refusé.
pub fn weak_reason(e: PasswordError) -> &'static str {
    match e {
        PasswordError::TooShort => "too_short",
        PasswordError::TooLong => "too_long",
        PasswordError::Common => "common",
        PasswordError::ContainsIdentifier => "contains_identifier",
    }
}

/// Code SQLSTATE d'une erreur de base, s'il y en a un.
pub fn sql_code(e: &sqlx::Error) -> Option<String> {
    e.as_database_error().and_then(|d| d.code().map(|c| c.to_string()))
}

/// Vrai si la base a refusé l'opération par la garde du dernier gérant (G-16).
pub fn is_last_manager(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|d| d.code().as_deref() == Some("23001") && d.message() == "user.last_manager")
}
