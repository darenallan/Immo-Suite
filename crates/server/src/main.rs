//! Binaire `immo` : `immo serve`, `immo version`, `immo openapi`, `immo backup …` (F39). Les autres commandes d'exploitation (`org`, `demo`) arrivent ensuite.

mod backup_cli;

use std::io::Write;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use immo_app::platform::config::Config;
use immo_app::platform::db::Db;
use immo_app::platform::file_store::DiskFileStore;
use immo_server::http::{self, AppState};
use immo_server::probe::{DbProbe, Probe, StoreProbe};
use immo_server::{openapi, version};
use tracing_subscriber::EnvFilter;

fn write_stdout(text: &str) -> ExitCode {
    if std::io::stdout().write_all(text.as_bytes()).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("version") => write_stdout(&format!("{}\n", version())),
        // `immo openapi` écrit sur la sortie standard ; `immo openapi <fichier>` écrit le fichier en UTF-8 exact.
        Some("openapi") => match std::env::args().nth(2) {
            Some(path) => match std::fs::write(path, openapi::openapi_json()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(_) => ExitCode::FAILURE,
            },
            None => write_stdout(&openapi::openapi_json()),
        },
        Some("serve") => serve().await,
        Some("backup") => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            backup_cli::run(&args).await
        }
        _ => {
            let _ = std::io::stderr().write_all(b"usage : immo serve | version | openapi | backup\n");
            ExitCode::from(2)
        }
    }
}

/// Journal JSON à champs en liste blanche : aucun corps de requête, aucune donnée personnelle (SECURITY.md §7.7).
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_current_span(false)
        .with_span_list(false)
        .init();
}

async fn serve() -> ExitCode {
    init_tracing();
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            // le message de ConfigError ne cite jamais la valeur fautive
            tracing::error!(error = %e, "configuration invalide");
            return ExitCode::FAILURE;
        }
    };
    let db = match Db::connect(config.database_url.expose(), 10).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!(error = %e, "connexion à la base impossible");
            return ExitCode::FAILURE;
        }
    };
    let mut probes: Vec<Arc<dyn Probe>> = vec![Arc::new(DbProbe(db))];
    match &config.file_store_dir {
        Some(dir) => probes.push(Arc::new(StoreProbe(DiskFileStore::new(dir.clone())))),
        None => tracing::warn!(
            "aucun stockage de fichiers configuré : le stockage objet arrive après le choix du fournisseur (DP-70)"
        ),
    }
    let app = http::router(Arc::new(AppState {
        probes,
        version: version(),
    }));
    let listener = match tokio::net::TcpListener::bind(config.bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(error = %e, "adresse d'écoute impossible");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(addr = %config.bind_addr, version = %version(), "serveur démarré");
    let served = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await;
    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "arrêt sur erreur");
            ExitCode::FAILURE
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("arrêt demandé");
}
