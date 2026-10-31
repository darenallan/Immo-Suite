//! Serveur HTTP d'Immo Suite : `GET /health`, `GET /ready` (réseau interne), journal JSON à champs en liste blanche.
//!
//! Étape 01 : aucune autre route (« hors étape : toute route autre que /health et /ready »).

pub mod http;
pub mod openapi;
pub mod probe;

/// Version du binaire : celle du crate et, si connue, le commit de construction (`IMMO_GIT_SHA`).
pub fn version() -> String {
    match option_env!("IMMO_GIT_SHA") {
        Some(sha) if !sha.is_empty() => format!("{}+{}", env!("CARGO_PKG_VERSION"), sha),
        _ => env!("CARGO_PKG_VERSION").to_string(),
    }
}
