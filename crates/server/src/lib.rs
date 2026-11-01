//! Serveur HTTP d'Immo Suite : sondes `/health` et `/ready` (réseau interne), routes d'identité (étape 02), journal JSON à champs
//! en liste blanche, contrat OpenAPI généré.

pub mod api;
pub mod api_account;
pub mod api_admin;
pub mod api_observation;
pub mod extract;
pub mod http;
pub mod limiter;
pub mod openapi;
pub mod probe;
pub mod problem;
pub mod settings;

/// Version du binaire : celle du crate et, si connue, le commit de construction (`IMMO_GIT_SHA`).
pub fn version() -> String {
    match option_env!("IMMO_GIT_SHA") {
        Some(sha) if !sha.is_empty() => format!("{}+{}", env!("CARGO_PKG_VERSION"), sha),
        _ => env!("CARGO_PKG_VERSION").to_string(),
    }
}
