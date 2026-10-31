//! Stockage des fichiers (DP-70, DP-71) : stockage objet privé en production, disque en développement seulement.
//!
//! Un objet est écrit « créer seulement » et n'est **jamais** réécrit : un PDF émis fait foi (DP-71).

use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

/// Erreurs du stockage de fichiers.
#[derive(Debug, thiserror::Error)]
pub enum FileStoreError {
    /// Clé vide, absolue, avec `..`, `\` ou un caractère hors `a-z A-Z 0-9 . _ - /`.
    #[error("clé de fichier invalide")]
    InvalidKey,
    /// L'objet existe déjà : jamais réécrit (DP-71).
    #[error("l'objet existe déjà")]
    AlreadyExists,
    #[error("objet introuvable")]
    NotFound,
    #[error("erreur d'entrée-sortie : {0}")]
    Io(#[from] std::io::Error),
}

/// Interface du stockage de fichiers. Les clés sont préfixées par organisation : `org/{org_id}/receipts/{id}.pdf`.
pub trait FileStore: Send + Sync {
    /// Écrit un objet **nouveau** ; refuse d'écraser.
    fn put_new(&self, key: &str, bytes: &[u8]) -> impl std::future::Future<Output = Result<(), FileStoreError>> + Send;
    /// Lit un objet.
    fn get(&self, key: &str) -> impl std::future::Future<Output = Result<Vec<u8>, FileStoreError>> + Send;
    /// Vrai si l'objet existe.
    fn exists(&self, key: &str) -> impl std::future::Future<Output = Result<bool, FileStoreError>> + Send;
    /// Contrôle de disponibilité (`/ready`).
    fn ping(&self) -> impl std::future::Future<Output = Result<(), FileStoreError>> + Send;
}

/// Valide une clé et la ramène à des segments sûrs.
fn validate_key(key: &str) -> Result<Vec<&str>, FileStoreError> {
    if key.is_empty() || key.len() > 512 || key.starts_with('/') || key.ends_with('/') {
        return Err(FileStoreError::InvalidKey);
    }
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/');
    if !key.chars().all(allowed) {
        return Err(FileStoreError::InvalidKey);
    }
    let segments: Vec<&str> = key.split('/').collect();
    if segments
        .iter()
        .any(|s| s.is_empty() || *s == "." || *s == ".." || s.starts_with('.'))
    {
        return Err(FileStoreError::InvalidKey);
    }
    Ok(segments)
}

/// Implémentation sur disque, **réservée au développement local** (DP-70).
#[derive(Debug, Clone)]
pub struct DiskFileStore {
    root: PathBuf,
}

impl DiskFileStore {
    /// Crée un stockage sur disque rooté dans `root` (le dossier est créé à la première écriture).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path(&self, key: &str) -> Result<PathBuf, FileStoreError> {
        let mut p = self.root.clone();
        for segment in validate_key(key)? {
            p.push(segment);
        }
        Ok(p)
    }

    /// Le dossier racine.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl FileStore for DiskFileStore {
    async fn put_new(&self, key: &str, bytes: &[u8]) -> Result<(), FileStoreError> {
        let path = self.path(key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        // `create_new` : échoue si le fichier existe, sans condition de concurrence entre le test et l'écriture
        let mut file = match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return Err(FileStoreError::AlreadyExists),
            Err(e) => return Err(e.into()),
        };
        file.write_all(bytes).await?;
        file.sync_all().await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, FileStoreError> {
        match tokio::fs::read(self.path(key)?).await {
            Ok(b) => Ok(b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(FileStoreError::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, FileStoreError> {
        Ok(tokio::fs::try_exists(self.path(key)?).await?)
    }

    async fn ping(&self) -> Result<(), FileStoreError> {
        tokio::fs::create_dir_all(&self.root).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> DiskFileStore {
        let dir = std::env::temp_dir().join(format!("immo-filestore-{name}-{}", uuid::Uuid::new_v4()));
        DiskFileStore::new(dir)
    }

    #[tokio::test]
    async fn ecrit_puis_relit() {
        let s = temp_store("rw");
        s.put_new("org/abc/receipts/1.pdf", b"%PDF-1.7 test").await.unwrap();
        assert_eq!(s.get("org/abc/receipts/1.pdf").await.unwrap(), b"%PDF-1.7 test");
        assert!(s.exists("org/abc/receipts/1.pdf").await.unwrap());
        assert!(!s.exists("org/abc/receipts/2.pdf").await.unwrap());
    }

    #[tokio::test]
    async fn jamais_reecrit() {
        let s = temp_store("once");
        s.put_new("org/abc/receipts/1.pdf", b"original").await.unwrap();
        assert!(matches!(
            s.put_new("org/abc/receipts/1.pdf", b"autre").await,
            Err(FileStoreError::AlreadyExists)
        ));
        assert_eq!(s.get("org/abc/receipts/1.pdf").await.unwrap(), b"original");
    }

    #[tokio::test]
    async fn deux_ecritures_simultanees_une_seule_reussit() {
        let s = std::sync::Arc::new(temp_store("race"));
        let mut handles = vec![];
        for i in 0..8u8 {
            let s = s.clone();
            handles.push(tokio::spawn(
                async move { s.put_new("org/x/f.bin", &[i]).await.is_ok() },
            ));
        }
        let mut ok = 0;
        for h in handles {
            if h.await.unwrap() {
                ok += 1;
            }
        }
        assert_eq!(ok, 1);
    }

    #[tokio::test]
    async fn cles_dangereuses_refusees() {
        let s = temp_store("keys");
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "org/../../x",
            "org//x",
            "org/x/",
            "a\\b",
            "org/.cache",
            "org/x y",
            "org/é.pdf",
            ".",
        ] {
            assert!(
                matches!(s.put_new(bad, b"x").await, Err(FileStoreError::InvalidKey)),
                "clé acceptée à tort : {bad:?}"
            );
        }
        assert!(!s.root().exists(), "rien ne doit être créé pour une clé refusée");
    }

    #[tokio::test]
    async fn introuvable() {
        let s = temp_store("nf");
        assert!(matches!(s.get("org/a/none.pdf").await, Err(FileStoreError::NotFound)));
    }
}
