//! Conservation de 13 mois (F39-observation règle 10) : tâche de fond quotidienne.

// Appel d'une fonction SQL hors contexte d'organisation : elle parcourt elle-même les organisations (migration de l'étape 04).
#![allow(clippy::disallowed_methods)]

use crate::platform::db::{Db, DbError};

/// Type de la tâche quotidienne.
pub const PURGE_JOB: &str = "observation.purge";

/// Supprime les événements et les erreurs de plus de 13 mois ; retourne le nombre de lignes supprimées.
pub async fn purge_expired(db: &Db) -> Result<i64, DbError> {
    let mut tx = db.begin_system().await?;
    let n: i64 = sqlx::query_scalar("SELECT purge_expired_observation()")
        .fetch_one(tx.conn())
        .await?;
    tx.commit().await?;
    Ok(n)
}
