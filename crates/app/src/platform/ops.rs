//! Opérations de déploiement : migrations en avant et écriture dans `platform_events` (étape 01, §5.1 et §5.6).
//!
//! - `migrate` s'exécute avec le rôle propriétaire (`MIGRATION_DATABASE_URL`), uniquement à l'étape de déploiement.
//! - `record_event` s'exécute avec `immo_platform` : le seul droit de ce rôle sur `platform_events` est `INSERT`.

// Ouvre ses propres connexions : réservé à `platform` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::Path;
use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, migrate::Migrator};

#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error("adresse de base invalide")]
    Url,
    #[error("connexion à la base impossible")]
    Connect(#[source] sqlx::Error),
    #[error("dossier de migrations illisible")]
    MigrationsDir(#[source] sqlx::migrate::MigrateError),
    #[error("migration en échec : {0}")]
    Migrate(#[source] sqlx::migrate::MigrateError),
    #[error("écriture de l'événement impossible")]
    Event(#[source] sqlx::Error),
    #[error("le détail de l'événement doit être un objet JSON")]
    Detail,
}

/// Applique les migrations du dossier, dans l'ordre, une seule connexion. Retourne le nombre de migrations connues de la base ensuite.
/// Les sommes de contrôle des migrations déjà appliquées sont vérifiées : une migration modifiée fait échouer l'opération.
pub async fn migrate(url: &str, dir: &Path) -> Result<i64, OpsError> {
    let options = PgConnectOptions::from_str(url).map_err(|_| OpsError::Url)?;
    let migrator = Migrator::new(dir).await.map_err(OpsError::MigrationsDir)?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(OpsError::Connect)?;
    migrator.run(&pool).await.map_err(OpsError::Migrate)?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(&pool)
        .await
        .map_err(OpsError::Connect)?;
    pool.close().await;
    Ok(count)
}

/// Ajoute une ligne à `platform_events` (déploiement, sauvegarde, restauration…). `action` suit le motif `domaine.verbe`.
pub async fn record_event(url: &str, action: &str, detail: &serde_json::Value) -> Result<(), OpsError> {
    if !detail.is_object() {
        return Err(OpsError::Detail);
    }
    let mut conn = PgConnectOptions::from_str(url)
        .map_err(|_| OpsError::Url)?
        .connect()
        .await
        .map_err(OpsError::Connect)?;
    sqlx::query("INSERT INTO platform_events (action, detail) VALUES ($1, $2::jsonb)")
        .bind(action)
        .bind(detail.to_string())
        .execute(&mut conn)
        .await
        .map_err(OpsError::Event)?;
    Ok(())
}
