//! Plate-forme : le seul endroit qui lit l'horloge, ouvre la base et touche au système de fichiers.
//!
//! Les lints de `clippy.toml` interdisent `PgPool`, `PgConnection` et la lecture directe de l'heure ailleurs ;
//! les `#[allow]` de ce module portent un commentaire qui cite la raison (décision n° 75).

pub mod clock;
pub mod config;
pub mod db;
pub mod file_store;
pub mod mailer;
pub mod pdf;
