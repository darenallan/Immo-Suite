//! Commandes de déploiement : `immo migrate [dossier]` et `immo event <action> [détail JSON]` (étape 01, §5.1 et §5.6).
//!
//! `MIGRATION_DATABASE_URL` (rôle `immo_owner`) pour `migrate`, `PLATFORM_DATABASE_URL` (rôle `immo_platform`) pour `event`.

use std::path::PathBuf;
use std::process::ExitCode;

use immo_app::platform::ops;

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

pub async fn migrate(args: &[String]) -> ExitCode {
    let Some(url) = env("MIGRATION_DATABASE_URL") else {
        tracing::error!("MIGRATION_DATABASE_URL absente");
        return ExitCode::FAILURE;
    };
    let dir = PathBuf::from(args.first().map_or("migrations", String::as_str));
    match ops::migrate(&url, &dir).await {
        Ok(count) => {
            tracing::info!(migrations = count, "migrations appliquées");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!(error = %e, "migration en échec");
            ExitCode::FAILURE
        }
    }
}

pub async fn event(args: &[String]) -> ExitCode {
    let (Some(action), Some(url)) = (args.first(), env("PLATFORM_DATABASE_URL")) else {
        tracing::error!("usage : immo event <domaine.verbe> [détail JSON] ; PLATFORM_DATABASE_URL requise");
        return ExitCode::FAILURE;
    };
    let detail = match args.get(1) {
        Some(text) => match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => {
                tracing::error!("détail JSON invalide");
                return ExitCode::FAILURE;
            }
        },
        None => serde_json::json!({}),
    };
    match ops::record_event(&url, action, &detail).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "événement non écrit");
            ExitCode::FAILURE
        }
    }
}
