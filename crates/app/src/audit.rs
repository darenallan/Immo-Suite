//! Écriture du journal d'audit (F03, SECURITY.md §6, R-007, R-008).
//!
//! `record` prend la transaction de l'action : si l'écriture échoue, l'erreur remonte et l'appelant annule tout. L'heure et
//! l'organisation sont imposées par la base (déclencheur `audit_before_insert`, politique `org_isolation`).

// Insertion dans `audit_log` : seule requête non vérifiée du module, comme celles de `platform` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods)]

use std::net::IpAddr;

use serde_json::Value;
use uuid::Uuid;

use crate::platform::db::{DbError, OrgTx};

macro_rules! audit_actions {
    ($($variant:ident => $code:literal),+ $(,)?) => {
        /// Codes `objet.verbe` de SECURITY.md §6.1. Énumération fermée : un code hors liste ne compile pas.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum AuditAction { $($variant),+ }

        impl AuditAction {
            pub const ALL: &'static [AuditAction] = &[$(AuditAction::$variant),+];

            pub fn code(self) -> &'static str {
                match self { $(AuditAction::$variant => $code),+ }
            }
        }
    };
}

audit_actions! {
    // Argent
    PaymentRecorded => "payment.recorded",
    PaymentCorrectionRequested => "payment.correction_requested",
    PaymentCorrectionRejected => "payment.correction_rejected",
    PaymentReversed => "payment.reversed",
    AllocationAdjusted => "allocation.adjusted",
    // Preuves
    ReceiptIssued => "receipt.issued",
    ReceiptCancelled => "receipt.cancelled",
    ReceiptCorrectiveIssued => "receipt.corrective_issued",
    ReceiptSent => "receipt.sent",
    ReceiptLinkCreated => "receipt.link_created",
    ReceiptLinkRevoked => "receipt.link_revoked",
    ReceiptPdfFailed => "receipt.pdf_failed",
    // Baux
    LeaseCreated => "lease.created",
    LeaseActivated => "lease.activated",
    LeaseAmended => "lease.amended",
    LeaseNoticeGiven => "lease.notice_given",
    LeaseNoticeCancelled => "lease.notice_cancelled",
    LeaseEnded => "lease.ended",
    InstallmentLineAdded => "installment.line_added",
    InstallmentLineVoided => "installment.line_voided",
    // Patrimoine
    UnitWorksSet => "unit.works_set",
    UnitArchived => "unit.archived",
    TenantCreated => "tenant.created",
    TenantUpdated => "tenant.updated",
    TenantMerged => "tenant.merged",
    TenantArchived => "tenant.archived",
    TenantIdDocumentViewed => "tenant.id_document_viewed",
    OwnerUpdated => "owner.updated",
    MandateCreated => "mandate.created",
    MandateRateChanged => "mandate.rate_changed",
    MandateClosed => "mandate.closed",
    // Comptes rendus
    StatementValidated => "statement.validated",
    StatementReopened => "statement.reopened",
    PayoutRecorded => "payout.recorded",
    ExpenseImputationChanged => "expense.imputation_changed",
    // Accès et droits
    AuthLogin => "auth.login",
    AuthPasswordChanged => "auth.password_changed",
    AuthMfaEnabled => "auth.mfa_enabled",
    AuthResetIssued => "auth.reset_issued",
    AuthResetUsed => "auth.reset_used",
    UserInvited => "user.invited",
    MembershipAccepted => "membership.accepted",
    MembershipRoleChanged => "membership.role_changed",
    UserDisabled => "user.disabled",
    SessionRevokedAll => "session.revoked_all",
    // Données
    ExportCreated => "export.created",
    ImportConfirmed => "import.confirmed",
    ImportCancelled => "import.cancelled",
    SettingsUpdated => "settings.updated",
    FeatureToggled => "feature.toggled",
    // Support
    SupportAccessGranted => "support.access_granted",
    SupportAccessRevoked => "support.access_revoked",
    SupportRecordViewed => "support.record_viewed",
    SupportImportAssist => "support.import_assist",
    // Sécurité
    SecurityCrossOrgAttempt => "security.cross_org_attempt",
    SecurityPermissionDenied => "security.permission_denied",
    SecurityRateLimited => "security.rate_limited",
}

/// Fragments de nom de clé interdits dans `before` et `after` (F03 règle 5). Insensible à la casse, récursif.
const FORBIDDEN_KEYS: [&str; 6] = ["password", "token", "secret", "hash", "totp", "csrf"];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuditError {
    #[error("clé interdite dans les valeurs avant/après : {0}")]
    ForbiddenKey(String),
    #[error("valeurs avant/après : un objet JSON est attendu")]
    NotAnObject,
}

fn check_keys(value: &Value) -> Result<(), AuditError> {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                let lower = key.to_lowercase();
                if FORBIDDEN_KEYS.iter().any(|f| lower.contains(f)) {
                    return Err(AuditError::ForbiddenKey(key.clone()));
                }
                check_keys(inner)?;
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(check_keys),
        _ => Ok(()),
    }
}

/// Ligne à écrire. L'organisation n'en fait pas partie : c'est celle du contexte de la transaction.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    action: AuditAction,
    object_type: String,
    object_id: String,
    before: Option<Value>,
    after: Option<Value>,
}

impl AuditEntry {
    pub fn new(action: AuditAction, object_type: impl Into<String>, object_id: impl ToString) -> Self {
        Self {
            action,
            object_type: object_type.into(),
            object_id: object_id.to_string(),
            before: None,
            after: None,
        }
    }

    /// Valeurs avant, sans secret : un objet JSON dont aucune clé ne contient `password`, `token`, `secret`, `hash`, `totp` ou `csrf`.
    pub fn before(mut self, value: Value) -> Result<Self, AuditError> {
        check_object(&value)?;
        self.before = Some(value);
        Ok(self)
    }

    pub fn after(mut self, value: Value) -> Result<Self, AuditError> {
        check_object(&value)?;
        self.after = Some(value);
        Ok(self)
    }
}

fn check_object(value: &Value) -> Result<(), AuditError> {
    if !value.is_object() {
        return Err(AuditError::NotAnObject);
    }
    check_keys(value)
}

/// Qui agit et d'où : renseigné par la couche HTTP, jamais par le corps de la requête.
#[derive(Debug, Clone)]
pub struct AuditContext {
    /// `None` : action du système (tâche de fond).
    pub actor_user_id: Option<Uuid>,
    pub ip: Option<IpAddr>,
    pub request_id: Option<String>,
}

/// Écrit une ligne dans la transaction de l'action. Toute erreur doit entraîner l'annulation de la transaction (R-008).
pub async fn record(tx: &mut OrgTx, ctx: &AuditContext, entry: AuditEntry) -> Result<(), DbError> {
    let kind = if ctx.actor_user_id.is_some() { "user" } else { "system" };
    sqlx::query(
        "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id, before, after, ip, request_id) \
         VALUES (current_org_id(), $1, $2::uuid, $3, $4, $5, $6::jsonb, $7::jsonb, $8::inet, $9)",
    )
    .bind(kind)
    .bind(ctx.actor_user_id.map(|u| u.to_string()))
    .bind(entry.action.code())
    .bind(entry.object_type)
    .bind(entry.object_id)
    .bind(entry.before.map(|v| v.to_string()))
    .bind(entry.after.map(|v| v.to_string()))
    .bind(ctx.ip.map(|i| i.to_string()))
    .bind(ctx.request_id.as_deref())
    .execute(tx.conn())
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tous_les_codes_ont_la_forme_objet_verbe_et_sont_uniques() {
        let mut seen = std::collections::HashSet::new();
        for a in AuditAction::ALL {
            let code = a.code();
            assert!(seen.insert(code), "code en double : {code}");
            let parts: Vec<&str> = code.split('.').collect();
            assert_eq!(parts.len(), 2, "{code}");
            assert!(
                code.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
                "{code}"
            );
        }
        assert_eq!(AuditAction::ALL.len(), 57);
    }

    #[test]
    fn les_cles_de_secret_sont_refusees_meme_imbriquees_et_en_majuscules() {
        for bad in [
            json!({"password": "x"}),
            json!({"Password_Hash": "x"}),
            json!({"a": {"b": [{"csrf_token": "x"}]}}),
            json!({"totp_secret": "x"}),
            json!({"SESSION_TOKEN": "x"}),
        ] {
            let result = AuditEntry::new(AuditAction::AuthLogin, "user", "1").after(bad.clone());
            assert!(matches!(result, Err(AuditError::ForbiddenKey(_))), "{bad}");
        }
        assert!(
            AuditEntry::new(AuditAction::AuthLogin, "user", "1")
                .after(json!({"org_role": "manager", "sessions": 3}))
                .is_ok()
        );
        assert_eq!(
            AuditEntry::new(AuditAction::AuthLogin, "user", "1")
                .before(json!("texte"))
                .unwrap_err(),
            AuditError::NotAnObject
        );
    }
}
