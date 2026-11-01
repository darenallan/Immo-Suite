//! Événements d'usage émis par le service d'identité (F39-observation §5) : échecs de connexion, session échue, réinitialisation.
//!
//! Tous en **meilleur effort** : un échec d'écriture est journalisé (sans donnée) et ne change jamais la réponse (règle 11). Un
//! événement n'est écrit que si l'organisation est identifiable : compte connu appartenant à **une seule** organisation active.

// Requêtes non vérifiées du module, comme celles de `audit` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods)]

use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use super::service::{Authenticated, IdentityService, at, from_micros, micros};
use super::session::hash_token;
use crate::observation::errors::{self, ErrorReport};
use crate::observation::events::{self, EventContext, ProductEvent};
use crate::platform::db::DbError;

/// Pseudonyme de la session : 16 caractères hexadécimaux de l'empreinte du jeton (`product_events.session_id`).
pub fn session_ref(token: &str) -> Option<String> {
    let hash = hash_token(token)?;
    Some(hash.iter().take(8).map(|b| format!("{b:02x}")).collect())
}

impl IdentityService {
    /// Contexte d'événement d'une requête authentifiée : utilisateur, rôle, pseudonyme de session (jamais le jeton).
    pub fn event_context(&self, auth: &Authenticated, token: &str, client_version: Option<String>) -> EventContext {
        EventContext {
            user_id: Some(auth.user.id),
            role: auth.role.map(|r| r.as_str().to_string()),
            session_ref: session_ref(token),
            client_version,
        }
    }

    /// Écrit un événement dans l'organisation de la session, en meilleur effort.
    pub async fn observe(&self, auth: &Authenticated, token: &str, event: Option<ProductEvent>) {
        let (Some(org), Some(event)) = (&auth.org, event) else {
            return;
        };
        let ctx = self.event_context(auth, token, None);
        self.observe_in(org.id, auth.user.id, &ctx, &event).await;
    }

    /// Agrège une erreur dans l'organisation de la session, en meilleur effort.
    pub async fn record_error(&self, auth: &Authenticated, report: &ErrorReport) {
        let Some(org) = &auth.org else { return };
        let result: Result<(), DbError> = async {
            let mut tx = self.db.begin(org.id, auth.user.id).await?;
            errors::record(&mut tx, report).await?;
            tx.commit().await
        }
        .await;
        if let Err(e) = result {
            tracing::error!(error = %e, "observation : erreur non agrégée");
        }
    }

    pub(super) async fn observe_in(&self, org_id: Uuid, user_id: Uuid, ctx: &EventContext, event: &ProductEvent) {
        let result: Result<(), DbError> = async {
            let mut tx = self.db.begin(org_id, user_id).await?;
            events::emit(&mut tx, ctx, event).await?;
            tx.commit().await
        }
        .await;
        if let Err(e) = result {
            tracing::error!(error = %e, event = event.name(), "observation : événement non écrit");
        }
    }

    /// Événement d'un compte connu hors session : écrit seulement si le compte a une seule organisation active.
    pub(super) async fn observe_user(&self, user_id: Uuid, event: Option<ProductEvent>) {
        let Some(event) = event else { return };
        let Ok(orgs) = self.my_organizations(user_id).await else {
            return;
        };
        if let [only] = orgs.as_slice() {
            let ctx = EventContext {
                user_id: Some(user_id),
                role: Some(only.role.as_str().to_string()),
                session_ref: None,
                client_version: None,
            };
            self.observe_in(only.id, user_id, &ctx, &event).await;
        }
    }

    /// Comme `observe_user`, mais en tâche détachée : la réponse de l'appelant n'attend pas (connexion refusée, sans écart de durée).
    pub(super) fn observe_user_detached(&self, user_id: Uuid, event: Option<ProductEvent>) {
        let svc = self.clone();
        tokio::spawn(async move { svc.observe_user(user_id, event).await });
    }

    /// `session_expired` : la session présentée existe mais est échue (inactivité ou durée absolue). Écrit une fois, puis révoque la
    /// session pour que les requêtes suivantes avec le même cookie n'écrivent plus rien.
    pub(super) async fn observe_session_expired(&self, token: &str, now: DateTime<Utc>) {
        let Some(id_hash) = hash_token(token) else { return };
        let run = async {
            let mut tx = self.db.begin_system().await?;
            let sql = format!(
                "SELECT user_id::text, org_id::text, {}, {} FROM sessions WHERE id_hash = $1 AND revoked_at IS NULL \
                    AND (idle_expires_at <= $2::timestamptz OR absolute_expires_at <= $2::timestamptz)",
                at("idle_expires_at"),
                at("absolute_expires_at")
            );
            let row = sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(id_hash.to_vec())
                .bind(micros(now))
                .fetch_optional(tx.conn())
                .await?;
            tx.commit().await?;
            let Some(row) = row else { return Ok::<_, DbError>(()) };
            self.revoke(&id_hash, now).await?;
            let (Some(user), Some(org)) = (
                Uuid::parse_str(&row.get::<String, _>(0)).ok(),
                row.get::<Option<String>, _>(1).and_then(|s| Uuid::parse_str(&s).ok()),
            ) else {
                return Ok(());
            };
            let absolute = from_micros(row.get::<i64, _>(3))? <= now;
            let reason = if absolute { "absolute" } else { "idle" };
            let mut tx = self.db.begin(org, user).await?;
            let role: Option<String> = sqlx::query_scalar("SELECT role FROM memberships WHERE user_id = $1::uuid")
                .bind(user.to_string())
                .fetch_optional(tx.conn())
                .await?;
            tx.rollback().await?;
            if let Some(event) = ProductEvent::session_expired(reason) {
                let ctx = EventContext {
                    user_id: Some(user),
                    role,
                    session_ref: session_ref(token),
                    client_version: None,
                };
                self.observe_in(org, user, &ctx, &event).await;
            }
            Ok(())
        };
        if let Err(e) = run.await {
            tracing::error!(error = %e, "observation : session échue non tracée");
        }
    }
}
