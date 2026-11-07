//! Binaire `immo` : `immo serve`, `immo version`, `immo openapi`, `immo backup …` (F39). Les autres commandes d'exploitation (`org`, `demo`) arrivent ensuite.

mod admin_cli;
mod backup_cli;
mod ops_cli;
mod pdf_worker;
mod worker;

use std::io::Write;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

use immo_app::identity::password::PasswordService;
use immo_app::identity::service::IdentityService;
use immo_app::platform::clock::SystemClock;
use immo_app::platform::config::Config;
use immo_app::platform::db::Db;
use immo_app::platform::file_store::DiskFileStore;
use immo_server::http::{self, AppState};
use immo_server::probe::{DbProbe, Probe, StoreProbe};
use immo_server::settings::HttpSettings;
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
        Some("migrate") => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            ops_cli::migrate(&args).await
        }
        Some("event") => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            ops_cli::event(&args).await
        }
        Some("pdf-worker") => pdf_worker::run(),
        Some("password") => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            match args.first().map(String::as_str) {
                Some("calibrate") => admin_cli::password_calibrate(args.get(1..).unwrap_or_default()).await,
                _ => {
                    let _ = std::io::stderr().write_all(
                        b"usage : immo password calibrate [--memory-kib N] [--iterations N] [--parallelism N]
",
                    );
                    ExitCode::from(2)
                }
            }
        }
        Some(group @ ("user" | "org")) => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            match (group, args.first().map(String::as_str)) {
                ("user", Some("create")) => admin_cli::user_create(args.get(1..).unwrap_or_default()).await,
                ("org", Some("create")) => admin_cli::org_create(args.get(1..).unwrap_or_default()).await,
                _ => {
                    let _ = std::io::stderr().write_all(
                        b"usage : immo user create ... | immo org create ...
",
                    );
                    ExitCode::from(2)
                }
            }
        }
        Some("work") => {
            init_tracing();
            worker::run().await
        }
        Some("backup") => {
            init_tracing();
            let args: Vec<String> = std::env::args().skip(2).collect();
            backup_cli::run(&args).await
        }
        _ => {
            let _ = std::io::stderr()
                .write_all(b"usage : immo serve | version | openapi | backup | work | migrate | event | pdf-worker\n");
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
    let settings = match HttpSettings::from_env(&config) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "configuration du serveur invalide");
            return ExitCode::FAILURE;
        }
    };
    let passwords = match PasswordService::new(settings.argon2) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "paramètres de mot de passe invalides");
            return ExitCode::FAILURE;
        }
    };
    let identity = IdentityService::new(
        db.clone(),
        passwords,
        Arc::new(SystemClock),
        settings.session_secret.expose().as_bytes().to_vec(),
    )
    .with_idle_minutes(settings.session_idle_minutes)
    .with_field_key(
        settings
            .field_encryption_key
            .as_ref()
            .and_then(|k| immo_app::identity::crypto::FieldKey::from_hex(k.expose()).ok()),
    );
    let mut probes: Vec<Arc<dyn Probe>> = vec![Arc::new(DbProbe(db))];
    match &config.file_store_dir {
        Some(dir) => probes.push(Arc::new(StoreProbe(DiskFileStore::new(dir.clone())))),
        None => tracing::warn!(
            "aucun stockage de fichiers configuré : le stockage objet arrive après le choix du fournisseur (DP-70)"
        ),
    }
    let production = config.environment == immo_app::platform::config::Environment::Production;
    let state = AppState::new(probes, version())
        .with_identity(
            identity,
            settings.allowed_origins,
            settings.trust_proxy,
            settings.min_client_version,
        )
        .with_public_base_url(config.public_base_url.as_deref().unwrap_or("http://localhost:8080"))
        // F39-observation : événement écarté = 422 hors production ; route d'essai jamais en production
        .with_strict_events(!production)
        .with_test_routes(!production && std::env::var("ENABLE_TEST_ROUTES").is_ok_and(|v| v == "1"));
    let state = state.with_file_store(config.file_store_dir.clone().map(DiskFileStore::new));
    let app = http::router(Arc::new(state));
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
