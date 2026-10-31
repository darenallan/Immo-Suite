//! Tests de base de l'étape 01 : migration de fondation, rôles (ISO-5), écriture seule, contexte (ISO-7), audit de schéma (ISO-6),
//! pool sous concurrence (PP-10, prototype b rejoué) et checksum des migrations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use immo_app::platform::db::Db;
use immo_db_tests::{TestDb, schema_audit};
use sqlx::{Executor, Row};
use uuid::Uuid;

fn code(e: &sqlx::Error) -> String {
    e.as_database_error().and_then(|d| d.code().map(|c| c.to_string())).unwrap_or_else(|| e.to_string())
}

// ───────── audit de schéma (ISO-6) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn audit_passe_sans_table_metier() {
    let db = TestDb::create().await;
    assert_eq!(schema_audit(&db.owner).await, Vec::<String>::new());
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn audit_echoue_pour_une_table_jouet_sans_rls() {
    let db = TestDb::create().await;
    // org_id présent, aucune RLS
    db.owner.execute("CREATE TABLE jouet_sans_rls (org_id uuid NOT NULL, x int)").await.unwrap();
    let e = schema_audit(&db.owner).await;
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].starts_with("jouet_sans_rls"), "{e:?}");
    // RLS activée mais pas forcée
    db.owner.execute("ALTER TABLE jouet_sans_rls ENABLE ROW LEVEL SECURITY").await.unwrap();
    assert_eq!(schema_audit(&db.owner).await.len(), 1);
    // forcée mais sans politique
    db.owner.execute("ALTER TABLE jouet_sans_rls FORCE ROW LEVEL SECURITY").await.unwrap();
    assert_eq!(schema_audit(&db.owner).await.len(), 1);
    // complète : RLS activée, forcée, politique
    db.owner.execute("CREATE POLICY org_isolation ON jouet_sans_rls USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id())").await.unwrap();
    assert_eq!(schema_audit(&db.owner).await, Vec::<String>::new());
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn audit_echoue_pour_une_table_sans_org_id_hors_liste_blanche() {
    let db = TestDb::create().await;
    db.owner.execute("CREATE TABLE jouet_globale (x int)").await.unwrap();
    let e = schema_audit(&db.owner).await;
    assert_eq!(e.len(), 1, "{e:?}");
    assert!(e[0].contains("liste blanche"), "{e:?}");
    db.drop_db().await;
}

// ───────── rôles (ISO-5) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn attributs_des_roles() {
    let db = TestDb::create().await;
    let rows = sqlx::query("SELECT rolname::text AS n, rolsuper, rolbypassrls, rolcreaterole, rolcreatedb FROM pg_roles WHERE rolname IN ('immo_owner', 'immo_app', 'immo_platform', 'immo_backup') ORDER BY 1").fetch_all(&db.admin).await.unwrap();
    assert_eq!(rows.len(), 4);
    for r in rows {
        let n: String = r.get("n");
        assert!(!r.get::<bool, _>("rolsuper"), "{n} est superutilisateur");
        assert!(!r.get::<bool, _>("rolcreaterole") && !r.get::<bool, _>("rolcreatedb"), "{n} crée des rôles ou des bases");
        // seul immo_backup contourne la RLS (décision n° 46, DP-78)
        assert_eq!(r.get::<bool, _>("rolbypassrls"), n == "immo_backup", "BYPASSRLS de {n}");
    }
    // immo_app, immo_platform, immo_backup ne possèdent aucun objet du schéma public
    let owned: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class c JOIN pg_namespace ns ON ns.oid = c.relnamespace
          WHERE ns.nspname = 'public' AND pg_get_userbyid(c.relowner) IN ('immo_app', 'immo_platform', 'immo_backup')",
    )
    .fetch_one(&db.admin)
    .await
    .unwrap();
    assert_eq!(owned, 0);
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_application_ne_peut_pas_creer_ni_redefinir_app_now() {
    let db = TestDb::create().await;
    let app = db.app_pool(1).await;
    let e = app.execute("CREATE OR REPLACE FUNCTION app_now() RETURNS timestamptz LANGUAGE sql AS $$ SELECT '2001-01-01'::timestamptz $$").await.unwrap_err();
    assert_eq!(code(&e), "42501", "{e:?}"); // permission refusée : pas de CREATE sur le schéma
    let e = app.execute("CREATE TABLE intrus (x int)").await.unwrap_err();
    assert_eq!(code(&e), "42501");
    let (a, b): (bool, bool) = sqlx::query_as("SELECT abs(extract(epoch FROM app_now() - now())) < 5, true").fetch_one(&app).await.unwrap();
    assert!(a && b, "app_now() doit valoir now() en production");
    db.drop_db().await;
}

// ───────── écriture seule : platform_events ─────────

#[tokio::test(flavor = "multi_thread")]
async fn platform_events_en_ajout_seul() {
    let db = TestDb::create().await;
    // immo_platform peut ajouter et lire
    db.platform.execute("INSERT INTO platform_events (action, detail) VALUES ('deploy.done', '{\"image\":\"sha256:test\"}')").await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM platform_events").fetch_one(&db.platform).await.unwrap();
    assert_eq!(n, 1);
    // ni UPDATE ni DELETE ni TRUNCATE : refusés par privilège (immo_platform) ET par déclencheur (propriétaire du schéma)
    for sql in ["UPDATE platform_events SET action = 'x.y'", "DELETE FROM platform_events", "TRUNCATE platform_events"] {
        let e = db.platform.execute(sql).await.unwrap_err();
        assert_eq!(code(&e), "42501", "{sql} : {e:?}");
    }
    for sql in ["UPDATE platform_events SET action = 'x.y'", "DELETE FROM platform_events", "TRUNCATE platform_events"] {
        let e = db.owner.execute(sql).await.unwrap_err();
        assert_eq!(code(&e), "23001", "{sql} : {e:?}"); // restrict_violation, levée par forbid_mutation / forbid_truncate
    }
    // l'application n'y a aucun accès
    let app = db.app_pool(1).await;
    assert_eq!(code(&app.execute("SELECT 1 FROM platform_events").await.unwrap_err()), "42501");
    assert_eq!(code(&app.execute("INSERT INTO platform_events (action) VALUES ('a.b')").await.unwrap_err()), "42501");
    // le format de l'action est contrôlé
    assert_eq!(code(&db.platform.execute("INSERT INTO platform_events (action) VALUES ('Mauvais Format')").await.unwrap_err()), "23514");
    db.drop_db().await;
}

// ───────── sauvegarde : immo_backup lit tout (DP-78) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn immo_backup_lit_toutes_les_tables_y_compris_les_nouvelles() {
    let db = TestDb::create().await;
    // une table créée APRÈS la migration par immo_owner : le droit vient de ALTER DEFAULT PRIVILEGES
    db.owner.execute("CREATE TABLE table_ajoutee_plus_tard (org_id uuid NOT NULL, x int)").await.unwrap();
    let rows = sqlx::query(
        "SELECT c.relname::text AS n, has_table_privilege('immo_backup', c.oid, 'SELECT') AS ok
           FROM pg_class c JOIN pg_namespace ns ON ns.oid = c.relnamespace WHERE ns.nspname = 'public' AND c.relkind = 'r'",
    )
    .fetch_all(&db.admin)
    .await
    .unwrap();
    assert!(rows.len() >= 5, "tables attendues : jobs, platform_events, purge_order, _sqlx_migrations, table_ajoutee_plus_tard");
    for r in rows {
        assert!(r.get::<bool, _>("ok"), "immo_backup ne peut pas lire {}", r.get::<String, _>("n"));
    }
    // et il contourne la RLS : il lit une table à RLS forcée sans contexte
    db.owner.execute("ALTER TABLE table_ajoutee_plus_tard ENABLE ROW LEVEL SECURITY").await.unwrap();
    db.owner.execute("ALTER TABLE table_ajoutee_plus_tard FORCE ROW LEVEL SECURITY").await.unwrap();
    db.owner.execute("CREATE POLICY p ON table_ajoutee_plus_tard USING (org_id = current_org_id())").await.unwrap();
    db.admin.execute("INSERT INTO table_ajoutee_plus_tard VALUES (gen_random_uuid(), 1)").await.unwrap(); // l'administrateur contourne aussi
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM table_ajoutee_plus_tard").fetch_one(&db.backup).await.unwrap();
    assert_eq!(n, 1);
    let n_owner: i64 = sqlx::query_scalar("SELECT count(*) FROM table_ajoutee_plus_tard").fetch_one(&db.owner).await.unwrap();
    assert_eq!(n_owner, 0, "le propriétaire est soumis à la RLS forcée");
    db.drop_db().await;
}

// ───────── file de tâches : contraintes ─────────

#[tokio::test(flavor = "multi_thread")]
async fn jobs_contraintes() {
    let db = TestDb::create().await;
    let app = db.app_pool(1).await;
    app.execute("INSERT INTO jobs (type, unique_key) VALUES ('demo', 'k1')").await.unwrap();
    assert_eq!(code(&app.execute("INSERT INTO jobs (type, unique_key) VALUES ('demo', 'k1')").await.unwrap_err()), "23505"); // clé d'unicité
    assert_eq!(code(&app.execute("INSERT INTO jobs (type, state) VALUES ('demo', 'inconnu')").await.unwrap_err()), "23514"); // état
    app.execute("INSERT INTO jobs (type) VALUES ('sans-cle'), ('sans-cle')").await.unwrap(); // plusieurs sans clé
    app.execute("DELETE FROM jobs WHERE type = 'sans-cle'").await.unwrap(); // purgeable par l'application
    db.drop_db().await;
}

// ───────── contexte d'organisation (ISO-7) et pool (PP-10) ─────────

async fn toy_units_table(db: &TestDb) {
    db.owner
        .execute(
            "CREATE TABLE units_jouet (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, label text NOT NULL);
             ALTER TABLE units_jouet ENABLE ROW LEVEL SECURITY;
             ALTER TABLE units_jouet FORCE ROW LEVEL SECURITY;
             CREATE POLICY org_isolation ON units_jouet USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
             GRANT SELECT, INSERT ON units_jouet TO immo_app;",
        )
        .await
        .unwrap();
}

async fn seed(db: &Db, org: Uuid, user: Uuid, n: i32) {
    let mut tx = db.begin(org, user).await.unwrap();
    sqlx::query("INSERT INTO units_jouet (org_id, label) SELECT $1::uuid, 'lot-' || g FROM generate_series(1, $2) g")
        .bind(org.to_string())
        .bind(n)
        .execute(tx.conn())
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn le_contexte_ne_survit_pas_a_la_transaction_iso7() {
    let db = TestDb::create().await;
    toy_units_table(&db).await;
    let app = Db::connect(&db.app_url, 1).await.unwrap(); // UNE connexion : la suivante est forcément la même
    let (a, u) = (Uuid::new_v4(), Uuid::new_v4());
    seed(&app, a, u, 3).await;
    // sous contexte : 3 lignes ; la transaction suivante, sur la même connexion, ne voit rien
    let mut tx = app.begin(a, u).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM units_jouet").fetch_one(tx.conn()).await.unwrap();
    assert_eq!(n, 3);
    tx.commit().await.unwrap();
    let raw = app.raw_pool_for_tests();
    let mut c = raw.begin().await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM units_jouet").fetch_one(&mut *c).await.unwrap();
    assert_eq!(n, 0, "sans contexte : 0 ligne");
    let ctx: Option<String> = sqlx::query_scalar("SELECT current_org_id()::text").fetch_one(&mut *c).await.unwrap();
    assert_eq!(ctx, None, "current_org_id() vaut NULL après la transaction (et non la chaîne vide)");
    // insertion sans contexte : refusée
    let e = sqlx::query("INSERT INTO units_jouet (org_id, label) VALUES (gen_random_uuid(), 'x')").execute(&mut *c).await.unwrap_err();
    assert_eq!(code(&e), "42501");
    c.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn pp10_pool_de_2_connexions_50_taches_10000_iterations_aucune_lecture_croisee() {
    let db = TestDb::create().await;
    toy_units_table(&db).await;
    let app = Db::connect(&db.app_url, 2).await.unwrap();
    let (a, b, u) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    seed(&app, a, u, 50).await;
    seed(&app, b, u, 50).await;
    let (reads, leaks) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let mut handles = vec![];
    for task in 0..50u32 {
        let (app, reads, leaks) = (app.clone(), reads.clone(), leaks.clone());
        handles.push(tokio::spawn(async move {
            for i in 0..200u32 {
                let me = if (task + i) % 2 == 0 { a } else { b };
                let mut tx = app.begin(me, u).await.unwrap();
                let rows = sqlx::query("SELECT org_id::text AS o FROM units_jouet").fetch_all(tx.conn()).await.unwrap();
                tx.commit().await.unwrap();
                reads.fetch_add(rows.len() as u64, Ordering::Relaxed);
                let foreign = rows.iter().filter(|r| r.get::<String, _>("o") != me.to_string()).count();
                leaks.fetch_add(foreign as u64, Ordering::Relaxed);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(reads.load(Ordering::Relaxed), 10_000 * 50, "chaque itération doit lire les 50 lignes de son organisation");
    assert_eq!(leaks.load(Ordering::Relaxed), 0, "AUCUNE lecture croisée");
    db.drop_db().await;
}

/// Test négatif obligatoire (DP-77) : la variante de SESSION fuit. Ce test PROUVE le piège ; s'il cessait de fuir, il échouerait
/// et il faudrait comprendre pourquoi avant de se réjouir.
#[tokio::test(flavor = "multi_thread")]
async fn pp10_negatif_la_variante_de_session_fuit() {
    let db = TestDb::create().await;
    toy_units_table(&db).await;
    let app = Db::connect(&db.app_url, 2).await.unwrap();
    let (a, b, u) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    seed(&app, a, u, 20).await;
    seed(&app, b, u, 20).await;
    let raw = app.raw_pool_for_tests().clone();
    let leaks = Arc::new(AtomicU64::new(0));
    let mut handles = vec![];
    for task in 0..10u32 {
        let (raw, leaks) = (raw.clone(), leaks.clone());
        handles.push(tokio::spawn(async move {
            for i in 0..100u32 {
                let me = if (task + i) % 2 == 0 { a } else { b };
                let mut conn = raw.acquire().await.unwrap();
                // 10 % d'oublis du contexte, et variante de session (troisième argument false) pour les autres
                if (task * 100 + i) % 10 != 9 {
                    sqlx::query("SELECT set_config('app.org_id', $1, false)").bind(me.to_string()).execute(&mut *conn).await.unwrap();
                }
                let rows = sqlx::query("SELECT org_id::text AS o FROM units_jouet").fetch_all(&mut *conn).await.unwrap();
                let foreign = rows.iter().filter(|r| r.get::<String, _>("o") != me.to_string()).count();
                leaks.fetch_add(foreign as u64, Ordering::Relaxed);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert!(leaks.load(Ordering::Relaxed) > 0, "la variante de session DOIT fuir : c'est la preuve du piège");
    db.drop_db().await;
}

// ───────── migrations : somme de contrôle (CODING_RULES.md §7 n° 10) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn une_migration_appliquee_puis_modifiee_est_refusee() {
    let dir = std::env::temp_dir().join(format!("immo-migrations-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/202610311800_foundation.sql");
    let target = dir.join("202610311800_foundation.sql");
    std::fs::copy(&source, &target).unwrap();

    let db = TestDb::create_without_migrations().await;
    db.migrate(&dir).await.expect("première application");
    db.migrate(&dir).await.expect("réapplication sans effet (idempotente)");
    // la migration déjà appliquée est modifiée : le lanceur refuse (divergence entre environnements)
    let mut text = std::fs::read_to_string(&target).unwrap();
    text.push_str("\n-- modification après application\n");
    std::fs::write(&target, text).unwrap();
    let err = db.migrate(&dir).await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("modified") || err.to_string().to_lowercase().contains("checksum") || matches!(err, sqlx::migrate::MigrateError::VersionMismatch(_)), "{err}");
    db.drop_db().await;
    std::fs::remove_dir_all(&dir).ok();
}
