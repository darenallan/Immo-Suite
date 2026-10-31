//! Commandes d'exploitation `immo backup …` (F39, `specs/F39-sauvegardes.md` §3). Aucun écran.
//!
//! Variables : `BACKUP_DATABASE_URL` (rôle `immo_backup`), `BACKUP_EVENTS_URL` (rôle `immo_platform`, facultative),
//! `BACKUP_AGE_RECIPIENT` (clé publique), `BACKUP_SINK_DIR` (second fournisseur simulé par un dossier, DP-82),
//! `BACKUP_AGE_IDENTITY_FILE` (clé privée : vérification en profondeur et restauration seulement), `PG_DUMP`, `PG_RESTORE`.

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use immo_app::platform::backup::{self, BackupConfig, DirSink, Manifest};
use immo_app::platform::clock::{Clock, SystemClock};

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn out(text: &str) {
    let _ = std::io::stdout().write_all(text.as_bytes());
}

fn fail(message: &str) -> ExitCode {
    tracing::error!("{message}");
    ExitCode::FAILURE
}

fn sink() -> Option<DirSink> {
    env("BACKUP_SINK_DIR").map(DirSink::new)
}

fn config() -> Result<BackupConfig, &'static str> {
    Ok(BackupConfig {
        dump_url: env("BACKUP_DATABASE_URL").ok_or("BACKUP_DATABASE_URL absente")?,
        events_url: env("BACKUP_EVENTS_URL"),
        recipient: env("BACKUP_AGE_RECIPIENT").ok_or("BACKUP_AGE_RECIPIENT absente")?,
        pg_dump: PathBuf::from(env("PG_DUMP").unwrap_or_else(|| "pg_dump".to_string())),
    })
}

fn identity() -> Result<String, &'static str> {
    let path = env("BACKUP_AGE_IDENTITY_FILE").ok_or("BACKUP_AGE_IDENTITY_FILE absente : la clé privée est requise")?;
    std::fs::read_to_string(path).map_err(|_| "clé privée illisible")
}

fn pg_restore() -> PathBuf {
    PathBuf::from(env("PG_RESTORE").unwrap_or_else(|| "pg_restore".to_string()))
}

async fn latest(sink: &DirSink) -> Result<Manifest, String> {
    let manifests = backup::latest_manifests(sink).await.map_err(|e| e.to_string())?;
    manifests
        .into_iter()
        .last()
        .ok_or_else(|| "aucune copie chez le second fournisseur".to_string())
}

/// Point d'entrée de `immo backup <commande>`.
pub async fn run(args: &[String]) -> ExitCode {
    let command = args.first().map(String::as_str);
    if command == Some("keygen") {
        // immo backup keygen <fichier> : écrit la clé privée dans le fichier (refuse d'écraser) et affiche la clé publique
        let Some(path) = args.get(1) else {
            return fail("usage : immo backup keygen <fichier de la clé privée>");
        };
        let (public, secret) = backup::generate_keypair();
        let written = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .and_then(|mut f| f.write_all(format!("{secret}\n").as_bytes()));
        return match written {
            Ok(()) => {
                out(&format!("{public}\n"));
                ExitCode::SUCCESS
            }
            Err(_) => fail("clé privée non écrite (le fichier existe déjà ou n'est pas accessible)"),
        };
    }
    let Some(sink) = sink() else {
        return fail("BACKUP_SINK_DIR absente : le second fournisseur n'est pas configuré");
    };
    match command {
        Some("run") => {
            let cfg = match config() {
                Ok(c) => c,
                Err(e) => return fail(e),
            };
            match backup::run(&SystemClock, &cfg, &sink).await {
                Ok(m) => {
                    out(&format!("{}\n", serde_json::to_string(&m).unwrap_or_default()));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&format!("sauvegarde en échec : {e}")),
            }
        }
        Some("list") => match backup::latest_manifests(&sink).await {
            Ok(ms) => {
                for m in ms {
                    out(&format!(
                        "{}  {:>12} octets  {}  schéma {:?}\n",
                        m.created_at, m.size_bytes, m.sha256, m.schema_version
                    ));
                }
                ExitCode::SUCCESS
            }
            Err(e) => fail(&e.to_string()),
        },
        Some("verify") => {
            let deep = args.iter().any(|a| a == "--deep");
            let manifest = match latest(&sink).await {
                Ok(m) => m,
                Err(e) => return fail(&e),
            };
            let key = if deep {
                match identity() {
                    Ok(k) => Some(k),
                    Err(e) => return fail(e),
                }
            } else {
                None
            };
            match backup::verify(&sink, &manifest, key.as_deref(), &pg_restore()).await {
                Ok(()) => {
                    out(&format!(
                        "copie {} conforme ({})\n",
                        manifest.name,
                        if deep {
                            "déchiffrée et lisible"
                        } else {
                            "empreinte et taille"
                        }
                    ));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&format!("copie {} non conforme : {e}", manifest.name)),
            }
        }
        // contrôle nocturne : une alerte par écart, code de sortie non nul s'il y en a (DEPLOYMENT.md §5.5, §7.3)
        Some("check") => match backup::latest_manifests(&sink).await {
            Ok(ms) => {
                let alerts = backup::check_alerts(SystemClock.now(), &ms);
                for a in &alerts {
                    tracing::error!(alert = a.code(), "alerte de sauvegarde");
                }
                if alerts.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(e) => fail(&e.to_string()),
        },
        Some("restore") => {
            let Some(target) = args.get(1) else {
                return fail("usage : immo backup restore <adresse de la base NEUVE>");
            };
            let (manifest, key) = match (latest(&sink).await, identity()) {
                (Ok(m), Ok(k)) => (m, k),
                (Err(e), _) => return fail(&e),
                (_, Err(e)) => return fail(e),
            };
            match backup::restore(&sink, &manifest, &key, target, &pg_restore()).await {
                Ok(t) => {
                    out(&format!(
                        "{{\"copie\":\"{}\",\"dechiffrement_ms\":{},\"restauration_ms\":{}}}\n",
                        manifest.name, t.decrypt_ms, t.restore_ms
                    ));
                    ExitCode::SUCCESS
                }
                Err(e) => fail(&format!("restauration en échec : {e}")),
            }
        }
        _ => fail("usage : immo backup keygen <fichier> | run | list | verify [--deep] | check | restore <base neuve>"),
    }
}
