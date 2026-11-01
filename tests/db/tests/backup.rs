//! Sauvegarde chiffrée, vérification et restauration complète chronométrée (F39, PP-14, PP-15 ; DEPLOYMENT.md §5.4 à §5.6).
//!
//! Jeu synthétique : 3 organisations fictives, 1 200 lignes, tables à RLS forcée. `PG_DUMP` et `PG_RESTORE` désignent les outils
//! PostgreSQL (défaut : dans le PATH). Aucune donnée réelle.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use std::path::PathBuf;

use age::secrecy::ExposeSecret;
use immo_app::platform::backup::{self, Alert, BackupConfig, BackupError, BackupReader, DirSink};
use immo_app::platform::clock::{Clock, FixedClock};
use immo_app::platform::file_store::FileStoreError;
use immo_db_tests::{TestDb, schema_audit};
use sqlx::{Executor, PgPool, Row};

const ORGS: [&str; 3] = [
    "a0000000-0000-4000-8000-000000000001",
    "b0000000-0000-4000-8000-000000000002",
    "c0000000-0000-4000-8000-000000000003",
];

fn tool(var: &str, default: &str) -> PathBuf {
    PathBuf::from(std::env::var(var).unwrap_or_else(|_| default.to_string()))
}

async fn seed(db: &TestDb) {
    db.owner
        .execute(
            "CREATE TABLE paiements_jouet (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, montant bigint NOT NULL CHECK (montant > 0));
             ALTER TABLE paiements_jouet ENABLE ROW LEVEL SECURITY; ALTER TABLE paiements_jouet FORCE ROW LEVEL SECURITY;
             CREATE POLICY org_isolation ON paiements_jouet USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
             GRANT SELECT, INSERT ON paiements_jouet TO immo_app;",
        )
        .await
        .unwrap();
    for (i, org) in ORGS.iter().enumerate() {
        // l'administrateur contourne la RLS ; 400 lignes par organisation, montants distincts par organisation
        db.admin
            .execute(sqlx::AssertSqlSafe(format!(
                "INSERT INTO paiements_jouet (org_id, montant) SELECT '{org}', {} + g FROM generate_series(1, 400) g",
                10_000 * (i + 1)
            )))
            .await
            .unwrap();
    }
}

/// (organisation, lignes, somme) lues par l'administrateur, qui contourne la RLS.
async fn totals(pool: &PgPool) -> Vec<(String, i64, i64)> {
    sqlx::query(
        "SELECT org_id::text AS o, count(*) AS n, sum(montant)::bigint AS s FROM paiements_jouet GROUP BY 1 ORDER BY 1",
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .iter()
    .map(|r| (r.get("o"), r.get("n"), r.get("s")))
    .collect()
}

fn temp_sink() -> (DirSink, PathBuf) {
    let dir = std::env::temp_dir().join(format!("immo-second-fournisseur-{}", uuid::Uuid::new_v4()));
    (DirSink::new(&dir), dir)
}

fn new_keys() -> (String, String) {
    let id = age::x25519::Identity::generate();
    (id.to_public().to_string(), id.to_string().expose_secret().to_string())
}

#[tokio::test(flavor = "multi_thread")]
async fn sauvegarde_chiffree_verifiee_et_restauree_a_l_identique() {
    let db = TestDb::create().await;
    seed(&db).await;
    let source = totals(&db.admin).await;
    assert_eq!(source.len(), 3);
    assert!(source.iter().all(|(_, n, _)| *n == 400));

    let (recipient, secret) = new_keys();
    let (sink, dir) = temp_sink();
    let cfg = BackupConfig {
        dump_url: db.backup_url.clone(),
        events_url: Some(db.platform_url.clone()),
        recipient,
        pg_dump: tool("PG_DUMP", "pg_dump"),
    };
    let pg_restore = tool("PG_RESTORE", "pg_restore");
    let clock = FixedClock::at_ymd_hm(2026, 11, 13, 2, 0).unwrap();

    // ───── sauvegarde ─────
    let manifest = backup::run(&clock, &cfg, &sink).await.expect("sauvegarde");
    assert_eq!(manifest.rows_by_table.get("paiements_jouet"), Some(&1200));
    assert_eq!(
        manifest.schema_version,
        Some(latest_migration_version()),
        "version de schéma = dernière migration"
    );
    assert_eq!(manifest.object_key, "daily/2026/11/13/immo-20261113T020000Z.dump.age");
    let keys = sink.list("daily/").await.unwrap();
    assert_eq!(keys.len(), 2, "un objet chiffré et son manifeste : {keys:?}");

    // ───── illisible sans la clé privée ─────
    let encrypted = sink.get(&manifest.object_key).await.unwrap();
    assert!(encrypted.starts_with(b"age-encryption.org/"), "format age");
    let text = String::from_utf8_lossy(&encrypted);
    assert!(
        !text.contains("PGDMP") && !text.contains("paiements_jouet") && !text.contains("immo_owner"),
        "aucun clair dans la copie"
    );
    let (_, autre_cle) = new_keys();
    assert!(
        matches!(
            backup::verify(&sink, &manifest, Some(&autre_cle), &pg_restore).await,
            Err(BackupError::Decrypt)
        ),
        "mauvaise clé refusée"
    );

    // ───── vérification (empreinte, puis en profondeur avec la clé privée) ─────
    backup::verify(&sink, &manifest, None, &pg_restore)
        .await
        .expect("empreinte");
    backup::verify(&sink, &manifest, Some(&secret), &pg_restore)
        .await
        .expect("déchiffrement et lecture du sommaire");

    // ───── écriture seule : jamais d'écrasement ─────
    assert!(
        matches!(
            backup::run(&clock, &cfg, &sink).await,
            Err(BackupError::Sink(FileStoreError::AlreadyExists))
        ),
        "une copie existante n'est jamais réécrite"
    );

    // ───── journal de plateforme ─────
    let done: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM platform_events WHERE action = 'backup.done' AND detail->>'sha256' = $1",
    )
    .bind(&manifest.sha256)
    .fetch_one(&db.admin)
    .await
    .unwrap();
    assert_eq!(done, 1, "une ligne backup.done, avec l'empreinte");
    let failed: i64 = sqlx::query_scalar("SELECT count(*) FROM platform_events WHERE action = 'backup.failed'")
        .fetch_one(&db.admin)
        .await
        .unwrap();
    assert_eq!(failed, 1, "la seconde tentative (refusée) est journalisée");

    // ───── restauration complète sur une base neuve, chronométrée ─────
    let target_name = format!("immo_restore_{}", &uuid::Uuid::new_v4().simple().to_string()[..10]);
    let target_url = db.create_empty_database(&target_name).await;
    let total = std::time::Instant::now();
    let timings = backup::restore(&sink, &manifest, &secret, &target_url, &pg_restore)
        .await
        .expect("restauration");
    let restored = db.admin_pool_for(&target_name).await;
    eprintln!(
        "RESTAURATION (jeu synthétique, 3 organisations, 1 200 lignes) : déchiffrement {} ms, pg_restore {} ms, total {} ms",
        timings.decrypt_ms,
        timings.restore_ms,
        total.elapsed().as_millis()
    );

    // comptes et sommes par organisation identiques
    assert_eq!(
        totals(&restored).await,
        source,
        "comptes de lignes et sommes par organisation identiques"
    );
    // audit d'isolation de la base restaurée : RLS activée et forcée, rôle applicatif sans BYPASSRLS
    assert_eq!(schema_audit(&restored).await, Vec::<String>::new());
    let forced: bool = sqlx::query_scalar("SELECT relforcerowsecurity FROM pg_class WHERE relname = 'paiements_jouet'")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert!(forced);
    let bypass: bool = sqlx::query_scalar("SELECT rolbypassrls FROM pg_roles WHERE rolname = 'immo_app'")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert!(!bypass);
    // les gardes d'écriture seule sont restaurées et actives
    let guards: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger WHERE tgname IN ('platform_events_no_mutation', 'platform_events_no_truncate') AND NOT tgisinternal").fetch_one(&restored).await.unwrap();
    assert_eq!(guards, 2);
    // les droits sont restaurés : immo_backup lit toujours tout
    let can: bool = sqlx::query_scalar("SELECT has_table_privilege('immo_backup', 'paiements_jouet', 'SELECT')")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert!(can);
    // une restauration dans une base non vide est refusée
    assert!(matches!(
        backup::restore(&sink, &manifest, &secret, &target_url, &pg_restore).await,
        Err(BackupError::Restore(_))
    ));

    restored.close().await;
    db.drop_database(&target_name).await;
    std::fs::remove_dir_all(&dir).ok();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn une_copie_alteree_est_detectee() {
    let db = TestDb::create().await;
    seed(&db).await;
    let (recipient, secret) = new_keys();
    let (sink, dir) = temp_sink();
    let cfg = BackupConfig {
        dump_url: db.backup_url.clone(),
        events_url: None,
        recipient,
        pg_dump: tool("PG_DUMP", "pg_dump"),
    };
    let pg_restore = tool("PG_RESTORE", "pg_restore");
    let manifest = backup::run(&FixedClock::at_ymd_hm(2026, 11, 13, 2, 0).unwrap(), &cfg, &sink)
        .await
        .unwrap();
    // un octet change dans l'objet chiffré chez le second fournisseur
    let path = dir.join(&manifest.object_key);
    let mut bytes = std::fs::read(&path).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xff;
    std::fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        backup::verify(&sink, &manifest, None, &pg_restore).await,
        Err(BackupError::HashMismatch)
    ));
    assert!(matches!(
        backup::verify(&sink, &manifest, Some(&secret), &pg_restore).await,
        Err(BackupError::HashMismatch)
    ));
    // tronquée : taille différente
    std::fs::write(&path, &bytes[..bytes.len() / 2]).unwrap();
    assert!(matches!(
        backup::verify(&sink, &manifest, None, &pg_restore).await,
        Err(BackupError::SizeMismatch)
    ));
    std::fs::remove_dir_all(&dir).ok();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn alerte_de_sauvegarde_manquante_apres_25_heures_pp15() {
    let db = TestDb::create().await;
    seed(&db).await;
    let (recipient, _) = new_keys();
    let (sink, dir) = temp_sink();
    let cfg = BackupConfig {
        dump_url: db.backup_url.clone(),
        events_url: None,
        recipient,
        pg_dump: tool("PG_DUMP", "pg_dump"),
    };
    let fait = FixedClock::at_ymd_hm(2026, 11, 13, 2, 0).unwrap();
    backup::run(&fait, &cfg, &sink).await.unwrap();
    let manifests = backup::latest_manifests(&sink).await.unwrap();
    assert_eq!(manifests.len(), 1);
    // 23 h plus tard : rien ; 25 h plus tard : l'alerte
    assert!(backup::check_alerts(FixedClock::at_ymd_hm(2026, 11, 14, 1, 0).unwrap().now(), &manifests).is_empty());
    assert_eq!(
        backup::check_alerts(FixedClock::at_ymd_hm(2026, 11, 14, 3, 0).unwrap().now(), &manifests),
        vec![Alert::Missing {
            hours_since_last: Some(25)
        }]
    );
    std::fs::remove_dir_all(&dir).ok();
    db.drop_db().await;
}

/// Version de la dernière migration du dépôt (préfixe numérique du dernier fichier de `migrations/`).
fn latest_migration_version() -> i64 {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .and_then(|n| n.split('_').next().map(str::to_string))
        })
        .filter_map(|v| v.parse::<i64>().ok())
        .max()
        .unwrap()
}
