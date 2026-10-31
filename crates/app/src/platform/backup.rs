//! Sauvegarde logique chiffrée (F39, DP-78, DP-82, `DEPLOYMENT.md` §5).
//!
//! Chaîne : `pg_dump -Fc` avec `immo_backup` → chiffrement à **clé publique** (`age`) → manifeste SHA-256 → envoi en **écriture
//! seule** chez un second fournisseur. Le travail ne porte que la clé **publique** : la clé privée reste hors ligne, en deux
//! exemplaires (DP-82), et ne sert qu'à vérifier en profondeur et à restaurer.
//!
//! Le second fournisseur n'est pas choisi (DP-82) : la cible est un `BackupWriter` ; `DirSink` (dossier) en tient lieu en
//! développement et en test. L'écriture seule est portée par le trait : `BackupWriter` n'offre ni lecture, ni suppression, ni écrasement.

// Accès à la base et à l'heure réservés à `platform` : requêtes de contrôle de la sauvegarde (comptes par table, journal de plateforme).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::future::Future;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, Connection};

use super::clock::Clock;
use super::file_store::{DiskFileStore, FileStore, FileStoreError};

/// Erreurs de la chaîne de sauvegarde ; aucun message ne cite d'identifiant de connexion.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("adresse de connexion à la base invalide")]
    BadUrl,
    #[error("clé publique de chiffrement invalide")]
    BadRecipient,
    #[error("clé privée invalide")]
    BadIdentity,
    #[error("pg_dump a échoué : {0}")]
    Dump(String),
    #[error("pg_restore a échoué : {0}")]
    Restore(String),
    #[error("chiffrement impossible")]
    Encrypt,
    #[error("déchiffrement impossible (mauvaise clé ou copie corrompue)")]
    Decrypt,
    #[error("empreinte différente de celle du manifeste : copie corrompue ou modifiée")]
    HashMismatch,
    #[error("taille différente de celle du manifeste")]
    SizeMismatch,
    #[error("manifeste illisible")]
    BadManifest,
    #[error("erreur de base de données")]
    Db(#[from] sqlx::Error),
    #[error("erreur du second fournisseur : {0}")]
    Sink(#[from] FileStoreError),
    #[error("erreur d'entrée-sortie : {0}")]
    Io(#[from] std::io::Error),
}

/// Manifeste d'une copie (DEPLOYMENT.md §5.2) : nom, taille, SHA-256 du fichier chiffré, version du schéma, date, lignes par table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    /// Clé de l'objet chiffré chez le second fournisseur.
    pub object_key: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub schema_version: Option<i64>,
    pub rows_by_table: BTreeMap<String, i64>,
}

impl Manifest {
    fn created(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    }
}

/// Paramètres d'une sauvegarde.
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Connexion du rôle `immo_backup` (jamais celui de l'application).
    pub dump_url: String,
    /// Connexion de `immo_platform`, pour écrire dans `platform_events` (facultative).
    pub events_url: Option<String>,
    /// Clé publique `age` (« age1… »).
    pub recipient: String,
    pub pg_dump: PathBuf,
}

/// Cible d'envoi en **écriture seule** : aucune lecture, suppression ni réécriture (DP-82).
pub trait BackupWriter: Send + Sync {
    fn put_new(&self, key: &str, bytes: &[u8]) -> impl Future<Output = Result<(), FileStoreError>> + Send;
}

/// Lecture des copies, réservée au développeur et au porteur du projet.
pub trait BackupReader: Send + Sync {
    fn get(&self, key: &str) -> impl Future<Output = Result<Vec<u8>, FileStoreError>> + Send;
    /// Les clés commençant par `prefix`, triées.
    fn list(&self, prefix: &str) -> impl Future<Output = Result<Vec<String>, FileStoreError>> + Send;
}

/// Second fournisseur simulé par un dossier (développement et tests).
#[derive(Debug, Clone)]
pub struct DirSink {
    store: DiskFileStore,
}

impl DirSink {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            store: DiskFileStore::new(root),
        }
    }
}

impl BackupWriter for DirSink {
    async fn put_new(&self, key: &str, bytes: &[u8]) -> Result<(), FileStoreError> {
        self.store.put_new(key, bytes).await
    }
}

impl BackupReader for DirSink {
    async fn get(&self, key: &str) -> Result<Vec<u8>, FileStoreError> {
        self.store.get(key).await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, FileStoreError> {
        let root = self.store.root().to_path_buf();
        let mut out = vec![];
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let mut rd = match tokio::fs::read_dir(&dir).await {
                Ok(r) => r,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if entry.file_type().await?.is_dir() {
                    stack.push(path);
                } else if let Ok(rel) = path.strip_prefix(&root) {
                    let key = rel
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    if key.starts_with(prefix) {
                        out.push(key);
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }
}

// ───────── utilitaires ─────────

struct PgTarget {
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    database: String,
}

fn parse_pg_url(url: &str) -> Result<PgTarget, BackupError> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))
        .ok_or(BackupError::BadUrl)?;
    let (authority, path) = rest.split_once('/').ok_or(BackupError::BadUrl)?;
    let database = path
        .split('?')
        .next()
        .filter(|d| !d.is_empty())
        .ok_or(BackupError::BadUrl)?
        .to_string();
    let (auth, hostport) = authority.rsplit_once('@').ok_or(BackupError::BadUrl)?;
    let (user, password) = match auth.split_once(':') {
        Some((u, p)) => (u.to_string(), Some(p.to_string())),
        None => (auth.to_string(), None),
    };
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (h.to_string(), p.to_string()),
        _ => (hostport.to_string(), "5432".to_string()),
    };
    Ok(PgTarget {
        host,
        port,
        user,
        password,
        database,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("immo-backup-{}", uuid::Uuid::new_v4()))
}

/// Exécute un programme PostgreSQL ; en cas d'échec, première ligne de stderr (jamais le mot de passe : il passe par l'environnement).
async fn run_pg(program: &Path, args: &[String], target: &PgTarget) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args).env_remove("PGPASSWORD");
    if let Some(p) = &target.password {
        cmd.env("PGPASSWORD", p);
    }
    let out = cmd.output().await.map_err(|e| format!("lancement impossible ({e})"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr)
            .lines()
            .next()
            .unwrap_or("échec sans message")
            .to_string())
    }
}

fn encrypt_file(recipient: &str, input: &Path, output: &Path) -> Result<(), BackupError> {
    let key: age::x25519::Recipient = recipient.parse().map_err(|_| BackupError::BadRecipient)?;
    let encryptor = age::Encryptor::with_recipients(std::iter::once(&key as _)).map_err(|_| BackupError::Encrypt)?;
    let mut reader = std::fs::File::open(input)?;
    let mut writer = encryptor
        .wrap_output(std::fs::File::create(output)?)
        .map_err(|_| BackupError::Encrypt)?;
    std::io::copy(&mut reader, &mut writer)?;
    writer.finish().map_err(|_| BackupError::Encrypt)?.sync_all()?;
    Ok(())
}

fn decrypt_file(identity: &str, input: &Path, output: &Path) -> Result<(), BackupError> {
    let id: age::x25519::Identity = identity.trim().parse().map_err(|_| BackupError::BadIdentity)?;
    let decryptor = age::Decryptor::new(std::fs::File::open(input)?).map_err(|_| BackupError::Decrypt)?;
    let mut reader = decryptor
        .decrypt(std::iter::once(&id as &dyn age::Identity))
        .map_err(|_| BackupError::Decrypt)?;
    let mut out = std::fs::File::create(output)?;
    let mut buf = vec![0_u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).map_err(|_| BackupError::Decrypt)?;
        if n == 0 {
            break;
        }
        out.write_all(buf.get(..n).ok_or(BackupError::Decrypt)?)?;
    }
    out.sync_all()?;
    Ok(())
}

async fn connect(url: &str) -> Result<sqlx::PgConnection, BackupError> {
    let options: PgConnectOptions = url.parse().map_err(|_| BackupError::BadUrl)?;
    Ok(options.connect().await?)
}

/// Version du schéma et lignes par table, lues avec le rôle de sauvegarde (qui contourne la RLS).
async fn describe(url: &str) -> Result<(Option<i64>, BTreeMap<String, i64>), BackupError> {
    let mut conn = connect(url).await?;
    let version: Option<i64> = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations")
        .fetch_one(&mut conn)
        .await
        .ok()
        .flatten();
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
          WHERE n.nspname = 'public' AND c.relkind = 'r' ORDER BY 1",
    )
    .fetch_all(&mut conn)
    .await?;
    let mut rows = BTreeMap::new();
    for t in tables {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT count(*) FROM \"{}\"",
            t.replace('"', "\"\"")
        )))
        .fetch_one(&mut conn)
        .await?;
        rows.insert(t, n);
    }
    Ok((version, rows))
}

async fn record_event(events_url: &Option<String>, action: &str, detail: serde_json::Value) {
    let Some(url) = events_url else { return };
    if let Ok(mut conn) = connect(url).await {
        let _ = sqlx::query("INSERT INTO platform_events (action, detail) VALUES ($1, $2::jsonb)")
            .bind(action)
            .bind(detail.to_string())
            .execute(&mut conn)
            .await;
    }
}

// ───────── sauvegarde ─────────

/// Fait une copie, la chiffre, l'envoie et renvoie son manifeste. Journalise `backup.done` ou `backup.failed` (sans donnée personnelle).
pub async fn run<W: BackupWriter>(clock: &dyn Clock, cfg: &BackupConfig, sink: &W) -> Result<Manifest, BackupError> {
    let result = run_inner(clock, cfg, sink).await;
    match &result {
        Ok(m) => {
            record_event(
                &cfg.events_url,
                "backup.done",
                serde_json::json!({ "name": m.name, "sha256": m.sha256, "size_bytes": m.size_bytes }),
            )
            .await
        }
        Err(e) => {
            record_event(
                &cfg.events_url,
                "backup.failed",
                serde_json::json!({ "reason": e.to_string() }),
            )
            .await
        }
    }
    result
}

async fn run_inner<W: BackupWriter>(clock: &dyn Clock, cfg: &BackupConfig, sink: &W) -> Result<Manifest, BackupError> {
    let now = clock.now();
    let name = format!("immo-{}", now.format("%Y%m%dT%H%M%SZ"));
    let object_key = format!(
        "daily/{:04}/{:02}/{:02}/{name}.dump.age",
        now.year(),
        now.month(),
        now.day()
    );
    let target = parse_pg_url(&cfg.dump_url)?;
    let dir = temp_dir();
    tokio::fs::create_dir_all(&dir).await?;
    let outcome = async {
        let dump = dir.join("copie.dump");
        let encrypted = dir.join("copie.dump.age");
        let args: Vec<String> = [
            "--format=custom",
            "--no-password",
            "-h",
            &target.host,
            "-p",
            &target.port,
            "-U",
            &target.user,
            "-d",
            &target.database,
            "-f",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .chain(std::iter::once(dump.to_string_lossy().into_owned()))
        .collect();
        run_pg(&cfg.pg_dump, &args, &target).await.map_err(BackupError::Dump)?;
        let (recipient, d, e) = (cfg.recipient.clone(), dump.clone(), encrypted.clone());
        tokio::task::spawn_blocking(move || encrypt_file(&recipient, &d, &e))
            .await
            .map_err(|_| BackupError::Encrypt)??;
        let bytes = tokio::fs::read(&encrypted).await?;
        let (schema_version, rows_by_table) = describe(&cfg.dump_url).await?;
        let manifest = Manifest {
            name: name.clone(),
            object_key: object_key.clone(),
            created_at: now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            size_bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            sha256: sha256_hex(&bytes),
            schema_version,
            rows_by_table,
        };
        // l'objet d'abord, le manifeste ensuite : un manifeste présent garantit un objet présent
        sink.put_new(&object_key, &bytes).await?;
        let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|_| BackupError::BadManifest)?;
        sink.put_new(&manifest_key(&object_key), &manifest_json).await?;
        Ok(manifest)
    }
    .await;
    let _ = tokio::fs::remove_dir_all(&dir).await; // la copie en clair ne reste jamais sur le disque du travail
    outcome
}

/// Clé du manifeste d'un objet chiffré.
pub fn manifest_key(object_key: &str) -> String {
    format!("{}.manifest.json", object_key.trim_end_matches(".dump.age"))
}

/// Dernier manifeste présent chez le second fournisseur, et le précédent.
pub async fn latest_manifests<R: BackupReader>(reader: &R) -> Result<Vec<Manifest>, BackupError> {
    let mut manifests = vec![];
    for key in reader
        .list("daily/")
        .await?
        .into_iter()
        .filter(|k| k.ends_with(".manifest.json"))
    {
        let m: Manifest = serde_json::from_slice(&reader.get(&key).await?).map_err(|_| BackupError::BadManifest)?;
        manifests.push(m);
    }
    manifests.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(manifests)
}

// ───────── vérification ─────────

/// Vérifie une copie : taille et SHA-256 contre le manifeste ; avec la clé privée, déchiffrement et `pg_restore --list`.
pub async fn verify<R: BackupReader>(
    reader: &R,
    manifest: &Manifest,
    identity: Option<&str>,
    pg_restore: &Path,
) -> Result<(), BackupError> {
    let bytes = reader.get(&manifest.object_key).await?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != manifest.size_bytes {
        return Err(BackupError::SizeMismatch);
    }
    if sha256_hex(&bytes) != manifest.sha256 {
        return Err(BackupError::HashMismatch);
    }
    if let Some(identity) = identity {
        let dir = temp_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let outcome = async {
            let (enc, dump) = (dir.join("copie.dump.age"), dir.join("copie.dump"));
            tokio::fs::write(&enc, &bytes).await?;
            let (id, e, d) = (identity.to_string(), enc.clone(), dump.clone());
            tokio::task::spawn_blocking(move || decrypt_file(&id, &e, &d))
                .await
                .map_err(|_| BackupError::Decrypt)??;
            let none = PgTarget {
                host: String::new(),
                port: String::new(),
                user: String::new(),
                password: None,
                database: String::new(),
            };
            run_pg(
                pg_restore,
                &["--list".to_string(), dump.to_string_lossy().into_owned()],
                &none,
            )
            .await
            .map_err(BackupError::Restore)
        }
        .await;
        let _ = tokio::fs::remove_dir_all(&dir).await;
        outcome?;
    }
    Ok(())
}

// ───────── restauration ─────────

/// Durées mesurées d'une restauration (compte rendu de `DEPLOYMENT.md` §5.6).
#[derive(Debug, Clone, Copy)]
pub struct RestoreTimings {
    pub decrypt_ms: u128,
    pub restore_ms: u128,
}

/// Restaure une copie dans une base **vide et neuve** (jamais la production). Les rôles du dump doivent exister (`ops/sql/roles.sql`).
pub async fn restore<R: BackupReader>(
    reader: &R,
    manifest: &Manifest,
    identity: &str,
    target_url: &str,
    pg_restore: &Path,
) -> Result<RestoreTimings, BackupError> {
    let target = parse_pg_url(target_url)?;
    // refuse une base qui contient déjà des tables : une restauration ne se mélange pas à des données existantes
    let mut conn = connect(target_url).await?;
    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relkind = 'r'").fetch_one(&mut conn).await?;
    conn.close().await?;
    if existing > 0 {
        return Err(BackupError::Restore("la base cible n'est pas vide".to_string()));
    }
    verify(reader, manifest, None, pg_restore).await?;
    let bytes = reader.get(&manifest.object_key).await?;
    let dir = temp_dir();
    tokio::fs::create_dir_all(&dir).await?;
    let outcome = async {
        let (enc, dump) = (dir.join("copie.dump.age"), dir.join("copie.dump"));
        tokio::fs::write(&enc, &bytes).await?;
        let t0 = Instant::now();
        let (id, e, d) = (identity.to_string(), enc.clone(), dump.clone());
        tokio::task::spawn_blocking(move || decrypt_file(&id, &e, &d))
            .await
            .map_err(|_| BackupError::Decrypt)??;
        let decrypt_ms = t0.elapsed().as_millis();
        let t1 = Instant::now();
        let args: Vec<String> = [
            "--exit-on-error",
            "--no-password",
            "-h",
            &target.host,
            "-p",
            &target.port,
            "-U",
            &target.user,
            "-d",
            &target.database,
        ]
        .iter()
        .map(|s| (*s).to_string())
        .chain(std::iter::once(dump.to_string_lossy().into_owned()))
        .collect();
        run_pg(pg_restore, &args, &target).await.map_err(BackupError::Restore)?;
        Ok(RestoreTimings {
            decrypt_ms,
            restore_ms: t1.elapsed().as_millis(),
        })
    }
    .await;
    let _ = tokio::fs::remove_dir_all(&dir).await;
    outcome
}

// ───────── alertes ─────────

/// Écart détecté par le contrôle nocturne (DEPLOYMENT.md §5.5 et §7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alert {
    /// Aucune copie depuis plus de 24 heures (S2) ; `None` : aucune copie du tout.
    Missing { hours_since_last: Option<i64> },
    /// La dernière copie fait moins de la moitié de la précédente (S2).
    TooSmall { previous_bytes: u64, current_bytes: u64 },
}

impl Alert {
    /// Code technique stable (anglais, DP-67).
    pub fn code(&self) -> &'static str {
        match self {
            Alert::Missing { .. } => "backup.missing",
            Alert::TooSmall { .. } => "backup.too_small",
        }
    }
}

/// Contrôle pur, sans entrée-sortie : `manifests` triés du plus ancien au plus récent.
pub fn check_alerts(now: DateTime<Utc>, manifests: &[Manifest]) -> Vec<Alert> {
    let mut alerts = vec![];
    let Some(last) = manifests.last() else {
        return vec![Alert::Missing { hours_since_last: None }];
    };
    match last.created() {
        Some(created) if now - created <= Duration::hours(24) => {}
        Some(created) => alerts.push(Alert::Missing {
            hours_since_last: Some((now - created).num_hours()),
        }),
        None => alerts.push(Alert::Missing { hours_since_last: None }),
    }
    if let Some(previous) = manifests.len().checked_sub(2).and_then(|i| manifests.get(i)) {
        if last.size_bytes.saturating_mul(2) < previous.size_bytes {
            alerts.push(Alert::TooSmall {
                previous_bytes: previous.size_bytes,
                current_bytes: last.size_bytes,
            });
        }
    }
    alerts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::clock::FixedClock;

    fn m(created: &str, size: u64) -> Manifest {
        Manifest {
            name: "n".into(),
            object_key: "k".into(),
            created_at: created.into(),
            size_bytes: size,
            sha256: "x".into(),
            schema_version: None,
            rows_by_table: BTreeMap::new(),
        }
    }

    #[test]
    fn aucune_copie_est_une_alerte() {
        let now = FixedClock::at_ymd_hm(2026, 11, 13, 9, 15).unwrap().now();
        assert_eq!(check_alerts(now, &[]), vec![Alert::Missing { hours_since_last: None }]);
    }

    #[test]
    fn copie_de_moins_de_24_heures_pas_d_alerte_et_25_heures_alerte_pp15() {
        let copie = [m("2026-11-12T02:00:00Z", 1000)];
        assert!(
            check_alerts(FixedClock::at_ymd_hm(2026, 11, 13, 1, 59).unwrap().now(), &copie).is_empty(),
            "23 h 59"
        );
        assert!(
            check_alerts(FixedClock::at_ymd_hm(2026, 11, 13, 2, 0).unwrap().now(), &copie).is_empty(),
            "24 h pile"
        );
        let late = check_alerts(FixedClock::at_ymd_hm(2026, 11, 13, 3, 0).unwrap().now(), &copie);
        assert_eq!(
            late,
            vec![Alert::Missing {
                hours_since_last: Some(25)
            }]
        );
        assert_eq!(late[0].code(), "backup.missing");
    }

    #[test]
    fn copie_de_moins_de_la_moitie_de_la_precedente() {
        let now = FixedClock::at_ymd_hm(2026, 11, 13, 9, 0).unwrap().now();
        let petite = [m("2026-11-12T02:00:00Z", 1000), m("2026-11-13T02:00:00Z", 499)];
        assert_eq!(
            check_alerts(now, &petite),
            vec![Alert::TooSmall {
                previous_bytes: 1000,
                current_bytes: 499
            }]
        );
        let limite = [m("2026-11-12T02:00:00Z", 1000), m("2026-11-13T02:00:00Z", 500)];
        assert!(
            check_alerts(now, &limite).is_empty(),
            "exactement la moitié : pas d'alerte"
        );
    }

    #[test]
    fn url_de_connexion() {
        let t = parse_pg_url("postgres://u:p%40ss@hote.exemple:6543/ma_base?sslmode=require").unwrap();
        assert_eq!(
            (
                t.user.as_str(),
                t.password.as_deref(),
                t.host.as_str(),
                t.port.as_str(),
                t.database.as_str()
            ),
            ("u", Some("p%40ss"), "hote.exemple", "6543", "ma_base")
        );
        let t = parse_pg_url("postgres://postgres@127.0.0.1/x").unwrap();
        assert_eq!((t.password, t.port.as_str()), (None, "5432"));
        assert!(parse_pg_url("mysql://x").is_err() && parse_pg_url("postgres://u@h").is_err());
    }

    #[test]
    fn cle_du_manifeste() {
        assert_eq!(
            manifest_key("daily/2026/11/13/immo-x.dump.age"),
            "daily/2026/11/13/immo-x.manifest.json"
        );
    }
}
