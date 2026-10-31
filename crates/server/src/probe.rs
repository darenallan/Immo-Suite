//! Sondes de disponibilité pour `/ready` : base de données et stockage de fichiers.

use std::future::Future;
use std::pin::Pin;

use immo_app::platform::db::Db;
use immo_app::platform::file_store::FileStore;

/// Une vérification de disponibilité. Le résultat ne dit que « oui » ou « non » : aucun détail ne sort sur le réseau.
pub trait Probe: Send + Sync {
    /// Nom pour le journal interne.
    fn name(&self) -> &'static str;
    /// Vrai si la ressource répond.
    fn check(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
}

/// Sonde de la base de données.
pub struct DbProbe(pub Db);

impl Probe for DbProbe {
    fn name(&self) -> &'static str {
        "database"
    }
    fn check(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { self.0.ping().await.is_ok() })
    }
}

/// Sonde du stockage de fichiers.
pub struct StoreProbe<S: FileStore>(pub S);

impl<S: FileStore> Probe for StoreProbe<S> {
    fn name(&self) -> &'static str {
        "file_store"
    }
    fn check(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { self.0.ping().await.is_ok() })
    }
}
