//! Accord d'accès support donné par le client (F40, R-009, DP-88).
//!
//! Un accord est donné à l'équipe HABEK pour une durée limitée (1 à 168 heures), révocable à tout moment. La garde qui rend la
//! base muette sans accord valide est dans le schéma (`support_grant_active`, politiques `support_*`) : ce module ne crée,
//! ne liste et ne révoque que les accords eux-mêmes, avec leur ligne d'audit dans la même transaction.

// Requêtes sur les tables de l'organisation active, dans son contexte : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use super::error::{FieldIssue, ServiceError};
use super::service::{Authenticated, ClientInfo, IdentityService, at, audit_context, from_micros, internal, micros};
use crate::audit::{self, AuditAction, AuditEntry};
use crate::platform::db::DbError;

/// Durée maximale d'un accord, en heures (7 jours, décision n° 38).
pub const MAX_HOURS: i64 = 168;
pub const DEFAULT_HOURS: i64 = 24;

#[derive(Debug, Clone)]
pub struct SupportGrantView {
    pub id: Uuid,
    pub scope: String,
    pub reason: String,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// `active`, `expired` ou `revoked` (dérivé).
    pub status: &'static str,
    pub duration_hours: i64,
}

/// Une consultation par l'équipe HABEK (ligne d'audit `actor_kind = 'support'`).
#[derive(Debug, Clone)]
pub struct SupportAccessView {
    pub occurred_at: DateTime<Utc>,
    pub action: String,
    pub object_type: String,
    pub object_id: String,
    pub actor_label: String,
}

#[derive(Debug, Clone)]
pub struct SupportOverview {
    pub grants: Vec<SupportGrantView>,
    pub accesses: Vec<SupportAccessView>,
}

fn grant_view(r: &sqlx::postgres::PgRow) -> Result<SupportGrantView, DbError> {
    let granted_at = from_micros(r.get(5))?;
    let expires_at = from_micros(r.get(6))?;
    Ok(SupportGrantView {
        id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
        scope: r.get(1),
        reason: r.get(2),
        granted_by: r.get(3),
        granted_at,
        expires_at,
        revoked_at: r.get::<Option<i64>, _>(7).map(from_micros).transpose()?,
        status: match r.get::<String, _>(4).as_str() {
            "revoked" => "revoked",
            "expired" => "expired",
            _ => "active",
        },
        duration_hours: (expires_at - granted_at).num_hours(),
    })
}

const GRANT_SQL: &str = "CASE WHEN g.revoked_at IS NOT NULL THEN 'revoked' WHEN g.expires_at <= $1::timestamptz THEN 'expired' ELSE 'active' END";

impl IdentityService {
    fn grant_select(extra_where: &str) -> String {
        format!(
            "SELECT g.id::text, g.scope, g.reason, u.display_name, {GRANT_SQL}, {}, {}, {} FROM support_grants g \
               JOIN users u ON u.id = g.granted_by {extra_where} ORDER BY g.granted_at DESC, g.id",
            at("g.granted_at"),
            at("g.expires_at"),
            at("g.revoked_at")
        )
    }

    /// `GET /support-grants` : les accords (actifs d'abord côté écran) et les consultations de l'équipe HABEK.
    pub async fn support_overview(&self, auth: &Authenticated) -> Result<SupportOverview, ServiceError> {
        let org = auth
            .org
            .as_ref()
            .ok_or(ServiceError::Forbidden("organization_required"))?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let grants = sqlx::query(sqlx::AssertSqlSafe(Self::grant_select("")))
            .bind(micros(now))
            .fetch_all(tx.conn())
            .await?;
        let accesses = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT {}, action, object_type, object_id FROM audit_log WHERE actor_kind = 'support' \
              ORDER BY occurred_at DESC, id DESC LIMIT 100",
            at("occurred_at")
        )))
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        Ok(SupportOverview {
            grants: grants.iter().map(grant_view).collect::<Result<_, DbError>>()?,
            accesses: accesses
                .iter()
                .map(|r| {
                    Ok(SupportAccessView {
                        occurred_at: from_micros(r.get(0))?,
                        action: r.get(1),
                        object_type: r.get(2),
                        object_id: r.get(3),
                        // le nom de l'agent vient avec la console (étape 11)
                        actor_label: "Équipe HABEK".to_string(),
                    })
                })
                .collect::<Result<_, DbError>>()?,
        })
    }

    /// `POST /support-grants` : portée, motif (5 à 500 caractères), durée en heures entières (1 à 168). Confirmation exigée.
    pub async fn create_support_grant(
        &self,
        auth: &Authenticated,
        scope: &str,
        reason: &str,
        duration_hours: i64,
        client: &ClientInfo,
    ) -> Result<SupportGrantView, ServiceError> {
        let org = auth
            .org
            .as_ref()
            .ok_or(ServiceError::Forbidden("organization_required"))?;
        Self::ensure_reauth(auth)?;
        let mut issues = vec![];
        if !matches!(scope, "read" | "import_assist") {
            issues.push(FieldIssue {
                field: "scope",
                code: "validation.enum",
            });
        }
        let reason = reason.trim();
        let length = reason.chars().count();
        if length < 5 {
            issues.push(FieldIssue {
                field: "reason",
                code: "validation.required",
            });
        } else if length > 500 {
            issues.push(FieldIssue {
                field: "reason",
                code: "validation.too_long",
            });
        }
        if !(1..=MAX_HOURS).contains(&duration_hours) {
            issues.push(FieldIssue {
                field: "duration_hours",
                code: "validation.amount_out_of_range",
            });
        }
        if !issues.is_empty() {
            return Err(ServiceError::Fields(issues));
        }
        let now = self.clock.now();
        let expires = now + Duration::hours(duration_hours);
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let id: String = sqlx::query_scalar(
            "INSERT INTO support_grants (org_id, scope, reason, granted_by, granted_at, expires_at) \
             VALUES (current_org_id(), $1, $2, $3::uuid, $4::timestamptz, $5::timestamptz) RETURNING id::text",
        )
        .bind(scope)
        .bind(reason)
        .bind(auth.user.id.to_string())
        .bind(micros(now))
        .bind(micros(expires))
        .fetch_one(tx.conn())
        .await?;
        let id = Uuid::parse_str(&id).map_err(internal)?;
        let entry = AuditEntry::new(AuditAction::SupportAccessGranted, "support_grant", id)
            .after(json!({
                "scope": scope,
                "reason": reason,
                "duration_hours": duration_hours,
                "expires_at": expires.to_rfc3339()
            }))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = self.grant_by_id(&mut tx, id, now).await?;
        tx.commit().await?;
        Ok(view)
    }

    async fn grant_by_id(
        &self,
        tx: &mut crate::platform::db::OrgTx,
        id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<SupportGrantView, ServiceError> {
        let r = sqlx::query(sqlx::AssertSqlSafe(Self::grant_select("WHERE g.id = $2::uuid")))
            .bind(micros(now))
            .bind(id.to_string())
            .fetch_one(tx.conn())
            .await?;
        Ok(grant_view(&r)?)
    }

    /// `POST /support-grants/{id}/revoke` : `revoked_at = maintenant`, la base ne rend plus rien dès la requête suivante de la console.
    pub async fn revoke_support_grant(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<SupportGrantView, ServiceError> {
        let org = auth
            .org
            .as_ref()
            .ok_or(ServiceError::Forbidden("organization_required"))?;
        Self::ensure_reauth(auth)?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = sqlx::query(sqlx::AssertSqlSafe(
            Self::grant_select("WHERE g.id = $2::uuid") + " FOR UPDATE OF g",
        ))
        .bind(micros(now))
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?;
        let Some(found) = found else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "support_grant", id, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if grant_view(&found)?.status != "active" {
            return Err(ServiceError::Invalid("already_ended"));
        }
        sqlx::query("UPDATE support_grants SET revoked_at = $2::timestamptz WHERE id = $1::uuid")
            .bind(id.to_string())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        let entry = AuditEntry::new(AuditAction::SupportAccessRevoked, "support_grant", id)
            .after(json!({}))
            .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = self.grant_by_id(&mut tx, id, now).await?;
        tx.commit().await?;
        Ok(view)
    }
}
