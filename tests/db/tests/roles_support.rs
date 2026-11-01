//! Étape 03, base de données (F01 §9.1, F02 §7.2, F40 §3, DATABASE_SCHEMA.md §6.5, §6.6, §6.12) : garde du dernier gérant (G-16),
//! accès support (ISO-8), colonnes modifiables de `support_grants`, paramètres par défaut, invitations par jeton, droits réglables.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::time::Duration;

use immo_db_tests::{TestDb, schema_audit};
use sqlx::{Executor, PgPool, Postgres, Row, Transaction};

const ORG_A: &str = "a0000000-0000-4000-8000-00000000000a";
const ORG_B: &str = "b0000000-0000-4000-8000-00000000000b";
const ORG_C: &str = "c0000000-0000-4000-8000-00000000000c"; // plan Propriétaire
const ORG_D: &str = "d0000000-0000-4000-8000-00000000000d"; // démonstration
const M1: &str = "11111111-0000-4000-8000-000000000001"; // gérant de A
const M2: &str = "22222222-0000-4000-8000-000000000002"; // gérant de A
const CA: &str = "33333333-0000-4000-8000-000000000003"; // caissier de A
const MB: &str = "44444444-0000-4000-8000-000000000004"; // gérant de B
const OW: &str = "55555555-0000-4000-8000-000000000005"; // owner de C
const DM: &str = "66666666-0000-4000-8000-000000000006"; // gérant de la démonstration

fn code(e: &sqlx::Error) -> String {
    e.as_database_error()
        .and_then(|d| d.code().map(|c| c.to_string()))
        .unwrap_or_else(|| e.to_string())
}

fn message(e: &sqlx::Error) -> String {
    e.as_database_error()
        .map(|d| d.message().to_string())
        .unwrap_or_else(|| e.to_string())
}

async fn seed(db: &TestDb) {
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO organizations (id, name, plan, kind) VALUES
               ('{ORG_A}', 'Agence A', 'agency', 'real'), ('{ORG_B}', 'Agence B', 'agency', 'real'),
               ('{ORG_C}', 'Famille C', 'owner', 'real'), ('{ORG_D}', 'Démo', 'agency', 'demo');
             INSERT INTO users (id, login_phone, display_name, is_demo) VALUES
               ('{M1}', '+22670000001', 'Un', false), ('{M2}', '+22670000002', 'Deux', false), ('{CA}', '+22670000003', 'Trois', false),
               ('{MB}', '+22670000004', 'Quatre', false), ('{OW}', '+22670000005', 'Cinq', false), ('{DM}', '+22670000006', 'Six', true);
             INSERT INTO memberships (org_id, user_id, role) VALUES
               ('{ORG_A}', '{M1}', 'manager'), ('{ORG_A}', '{CA}', 'cashier'), ('{ORG_B}', '{MB}', 'manager'),
               ('{ORG_C}', '{OW}', 'owner'), ('{ORG_D}', '{DM}', 'manager');"
        )))
        .await
        .unwrap();
}

async fn ctx(pool: &PgPool, org: &str, user: &str) -> Transaction<'static, Postgres> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.org_id', $1, true), set_config('app.user_id', $2, true)")
        .bind(org)
        .bind(user)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx
}

async fn disable(tx: &mut Transaction<'static, Postgres>, org: &str, user: &str) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE memberships SET status = 'disabled', disabled_at = app_now() WHERE org_id = $1::uuid AND user_id = $2::uuid",
    )
    .bind(org)
    .bind(user)
    .execute(&mut **tx)
    .await
    .map(|r| r.rows_affected())
}

// ───────── G-16 : dernier gérant ─────────

#[tokio::test(flavor = "multi_thread")]
async fn g16_le_seul_gerant_ne_peut_etre_ni_desactive_ni_retrograde() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(4).await;

    let mut tx = ctx(&app, ORG_A, CA).await;
    let e = disable(&mut tx, ORG_A, M1).await.unwrap_err();
    assert_eq!(code(&e), "23001", "restrict_violation : {e}");
    assert_eq!(message(&e), "user.last_manager");
    tx.rollback().await.unwrap();

    let mut tx = ctx(&app, ORG_A, CA).await;
    let e = sqlx::query("UPDATE memberships SET role = 'cashier' WHERE org_id = $1::uuid AND user_id = $2::uuid")
        .bind(ORG_A)
        .bind(M1)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(message(&e), "user.last_manager", "rétrogradation du seul gérant");
    tx.rollback().await.unwrap();

    // un UPDATE direct de l'administrateur est aussi refusé : la garde est un déclencheur, pas du code applicatif
    let e = db
        .admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE memberships SET status = 'disabled', disabled_at = now() WHERE org_id = '{ORG_A}' AND user_id = '{M1}'"
        )))
        .await
        .unwrap_err();
    assert_eq!(message(&e), "user.last_manager");

    // le caissier, lui, se désactive sans difficulté
    let mut tx = ctx(&app, ORG_A, M1).await;
    assert_eq!(disable(&mut tx, ORG_A, CA).await.unwrap(), 1);
    tx.commit().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn g16_avec_deux_gerants_l_un_se_retire_l_autre_non() {
    let db = TestDb::create().await;
    seed(&db).await;
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_A}', '{M2}', 'manager')"
        )))
        .await
        .unwrap();
    let app = db.app_pool(4).await;
    let mut tx = ctx(&app, ORG_A, M1).await;
    assert_eq!(disable(&mut tx, ORG_A, M2).await.unwrap(), 1, "A désactive B");
    tx.commit().await.unwrap();
    let mut tx = ctx(&app, ORG_A, M1).await;
    let e = disable(&mut tx, ORG_A, M1).await.unwrap_err();
    assert_eq!(message(&e), "user.last_manager", "A est maintenant le dernier");
    tx.rollback().await.unwrap();
    // B, déjà désactivé, peut être désactivé de nouveau sans effet de bord : la garde ne regarde que les gérants actifs
    let mut tx = ctx(&app, ORG_A, M1).await;
    assert_eq!(disable(&mut tx, ORG_A, M2).await.unwrap(), 1);
    tx.commit().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn g16_deux_retraits_simultanes_un_seul_reussit() {
    let db = TestDb::create().await;
    seed(&db).await;
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_A}', '{M2}', 'manager')"
        )))
        .await
        .unwrap();
    let app = db.app_pool(4).await;

    let mut tx1 = ctx(&app, ORG_A, M1).await;
    assert_eq!(
        disable(&mut tx1, ORG_A, M2).await.unwrap(),
        1,
        "M1 retire M2, sans valider encore"
    );

    let app2 = app.clone();
    let second = tokio::spawn(async move {
        let mut tx2 = ctx(&app2, ORG_A, M2).await;
        let result = disable(&mut tx2, ORG_A, M1).await;
        match result {
            Ok(_) => {
                tx2.commit().await.unwrap();
                Ok(())
            }
            Err(e) => {
                tx2.rollback().await.ok();
                Err(e)
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(
        !second.is_finished(),
        "le second retrait attend le verrou de l'organisation"
    );
    tx1.commit().await.unwrap();
    let e = second.await.unwrap().unwrap_err();
    assert_eq!(
        message(&e),
        "user.last_manager",
        "le second voit l'état validé du premier"
    );
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM memberships WHERE org_id = $1::uuid AND role = 'manager' AND status = 'active'",
    )
    .bind(ORG_A)
    .fetch_one(&db.admin)
    .await
    .unwrap();
    assert_eq!(active, 1, "exactement un gérant actif");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn g16_plan_proprietaire_et_organisation_de_demonstration() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(4).await;
    let mut tx = ctx(&app, ORG_C, OW).await;
    let e = disable(&mut tx, ORG_C, OW).await.unwrap_err();
    assert_eq!(message(&e), "user.last_manager", "dernier owner du plan Propriétaire");
    tx.rollback().await.unwrap();
    let mut tx = ctx(&app, ORG_D, DM).await;
    let e = disable(&mut tx, ORG_D, DM).await.unwrap_err();
    assert_eq!(message(&e), "user.last_manager", "organisation de démonstration");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

// ───────── ISO-8 : accès support ─────────

const ORG_TABLES: [&str; 6] = [
    "memberships",
    "audit_log",
    "invitations",
    "role_settings",
    "unit_assignments",
    "org_settings",
];

async fn seed_rows(db: &TestDb) {
    seed(db).await;
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES
               ('{ORG_A}', 'user', '{M1}', 'auth.login', 'user', '{M1}'), ('{ORG_B}', 'user', '{MB}', 'auth.login', 'user', '{MB}');
             INSERT INTO invitations (org_id, invitee_phone, role, token_hash, created_by, expires_at)
               VALUES ('{ORG_A}', '+22670000099', 'cashier', decode(repeat('aa', 32), 'hex'), '{M1}', now() + interval '7 days'),
                      ('{ORG_B}', '+22670000098', 'cashier', decode(repeat('bb', 32), 'hex'), '{MB}', now() + interval '7 days');
             INSERT INTO role_settings (org_id, role, action, allowed, set_by) VALUES
               ('{ORG_A}', 'property_manager', 'record_payment', true, '{M1}'), ('{ORG_B}', 'property_manager', 'record_payment', true, '{MB}');
             INSERT INTO unit_assignments (org_id, unit_id, user_id, assigned_by) VALUES
               ('{ORG_A}', gen_random_uuid(), '{CA}', '{M1}');"
        )))
        .await
        .unwrap();
}

async fn platform_counts(db: &TestDb, org: &str) -> Vec<(String, i64)> {
    let mut out = vec![];
    for table in ORG_TABLES {
        let mut tx = ctx(&db.platform, org, M1).await;
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        out.push((table.to_string(), n));
    }
    out
}

async fn grant(db: &TestDb, org: &str, scope: &str, granted: &str, expires: &str) {
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO support_grants (org_id, scope, reason, granted_by, granted_at, expires_at)
             VALUES ('{org}', '{scope}', 'Aide à l''import', '{M1}', {granted}, {expires})"
        )))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn iso_8_sans_accord_zero_ligne_avec_accord_des_lignes_apres_revocation_zero() {
    let db = TestDb::create().await;
    seed_rows(&db).await;

    for (table, n) in platform_counts(&db, ORG_A).await {
        assert_eq!(n, 0, "{table} : sans accord, la console ne voit rien");
    }

    grant(&db, ORG_A, "read", "now()", "now() + interval '24 hours'").await;
    for (table, n) in platform_counts(&db, ORG_A).await {
        assert!(n > 0, "{table} : avec accord, la console lit");
    }
    for (table, n) in platform_counts(&db, ORG_B).await {
        assert_eq!(n, 0, "{table} : l'accord de A ne couvre pas B");
    }

    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE support_grants SET revoked_at = now() WHERE org_id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    for (table, n) in platform_counts(&db, ORG_A).await {
        assert_eq!(n, 0, "{table} : après révocation, rien");
    }
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn iso_8_un_accord_expire_ne_donne_rien_et_30_jours_sont_refuses() {
    let db = TestDb::create().await;
    seed_rows(&db).await;
    grant(
        &db,
        ORG_A,
        "read",
        "now() - interval '3 days'",
        "now() - interval '1 day'",
    )
    .await;
    for (table, n) in platform_counts(&db, ORG_A).await {
        assert_eq!(n, 0, "{table} : accord expiré");
    }
    let e = db
        .admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO support_grants (org_id, reason, granted_by, expires_at) VALUES ('{ORG_A}', 'trente jours', '{M1}', now() + interval '30 days')"
        )))
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514", "support_grants_duration : {e}");
    // 7 jours exactement : accepté ; 7 jours et une seconde : refusé
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO support_grants (org_id, reason, granted_by, granted_at, expires_at) VALUES ('{ORG_A}', 'sept jours', '{M1}', now(), now() + interval '7 days')"
        )))
        .await
        .unwrap();
    let e = db
        .admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO support_grants (org_id, reason, granted_by, granted_at, expires_at) VALUES ('{ORG_A}', 'sept jours et une seconde', '{M1}', now(), now() + interval '7 days 1 second')"
        )))
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514");
    // deux accords actifs : la révocation d'un seul laisse l'accès
    grant(&db, ORG_A, "read", "now()", "now() + interval '24 hours'").await;
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE support_grants SET revoked_at = now() WHERE org_id = '{ORG_A}' AND reason = 'sept jours'"
        )))
        .await
        .unwrap();
    for (table, n) in platform_counts(&db, ORG_A).await {
        assert!(n > 0, "{table} : l'autre accord reste valable");
    }
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_portee_read_interdit_l_ecriture_import_assist_l_autorise() {
    let db = TestDb::create().await;
    seed_rows(&db).await;
    db.owner
        .execute(
            "CREATE TABLE toy (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, v text);
             SELECT apply_org_policies('toy');
             GRANT SELECT, INSERT, UPDATE ON toy TO immo_platform, immo_app",
        )
        .await
        .unwrap();
    let insert = |org: &'static str| {
        let pool = db.platform.clone();
        async move {
            let mut tx = ctx(&pool, org, M1).await;
            let r = sqlx::query("INSERT INTO toy (org_id, v) VALUES ($1::uuid, 'x')")
                .bind(org)
                .execute(&mut *tx)
                .await;
            tx.rollback().await.unwrap();
            r
        }
    };
    grant(&db, ORG_A, "read", "now()", "now() + interval '24 hours'").await;
    let e = insert(ORG_A).await.unwrap_err();
    assert_eq!(
        code(&e),
        "42501",
        "accord read : écriture refusée par la politique : {e}"
    );
    grant(&db, ORG_A, "import_assist", "now()", "now() + interval '24 hours'").await;
    insert(ORG_A).await.unwrap();
    let e = insert(ORG_B).await.unwrap_err();
    assert_eq!(code(&e), "42501", "l'accord de A ne permet pas d'écrire chez B");

    // audit : la console écrit ses lignes `support` avec n'importe quel accord actif, jamais une ligne d'utilisateur
    let mut tx = ctx(&db.platform, ORG_B, M1).await;
    let e = sqlx::query(
        "INSERT INTO audit_log (org_id, actor_kind, actor_staff_id, action, object_type, object_id) VALUES ($1::uuid, 'support', gen_random_uuid(), 'support.record_viewed', 'x', 'y')",
    )
    .bind(ORG_B)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(code(&e), "42501", "sans accord pour B : refusé");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_audit_de_la_console_exige_un_accord_et_la_nature_support() {
    let db = TestDb::create().await;
    seed_rows(&db).await;
    grant(&db, ORG_A, "read", "now()", "now() + interval '24 hours'").await;
    let mut tx = ctx(&db.platform, ORG_A, M1).await;
    sqlx::query(
        "INSERT INTO audit_log (org_id, actor_kind, actor_staff_id, action, object_type, object_id) VALUES ($1::uuid, 'support', gen_random_uuid(), 'support.record_viewed', 'x', 'y')",
    )
    .bind(ORG_A)
    .execute(&mut *tx)
    .await
    .unwrap();
    let e = sqlx::query(
        "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES ($1::uuid, 'user', $2::uuid, 'auth.login', 'x', 'y')",
    )
    .bind(ORG_A)
    .bind(M1)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(
        code(&e),
        "42501",
        "la console ne se fait pas passer pour un utilisateur"
    );
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn support_grants_colonnes_modifiables_et_aucun_droit_pour_la_console() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(4).await;

    let mut tx = ctx(&app, ORG_A, M1).await;
    let id: String = sqlx::query_scalar(
        "INSERT INTO support_grants (org_id, scope, reason, granted_by, expires_at) VALUES ($1::uuid, 'read', 'Aide au démarrage', $2::uuid, app_now() + interval '168 hours') RETURNING id::text",
    )
    .bind(ORG_A)
    .bind(M1)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    for column in [
        "reason = 'autre'",
        "expires_at = expires_at + interval '1 hour'",
        "granted_at = now()",
        "scope = 'import_assist'",
    ] {
        let mut tx = ctx(&app, ORG_A, M1).await;
        let e = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE support_grants SET {column} WHERE id = '{id}'"
        )))
        .execute(&mut *tx)
        .await
        .unwrap_err();
        assert_eq!(code(&e), "42501", "{column} : {e}");
        tx.rollback().await.unwrap();
    }
    let mut tx = ctx(&app, ORG_A, M1).await;
    let e = sqlx::query("DELETE FROM support_grants")
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "DELETE refusé");
    tx.rollback().await.unwrap();
    let e = app.execute("TRUNCATE support_grants").await.unwrap_err();
    assert_eq!(code(&e), "42501", "TRUNCATE refusé");

    // la révocation se pose une fois
    let mut tx = ctx(&app, ORG_A, M1).await;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE support_grants SET revoked_at = app_now() WHERE id = '{id}'"
    )))
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let mut tx = ctx(&app, ORG_A, M1).await;
    let e = sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE support_grants SET revoked_at = NULL WHERE id = '{id}'"
    )))
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(code(&e), "23001", "un accord révoqué ne se rouvre pas : {e}");
    tx.rollback().await.unwrap();

    // l'organisation B ne voit pas l'accord de A
    let mut tx = ctx(&app, ORG_B, MB).await;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM support_grants")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 0);
    tx.rollback().await.unwrap();

    // la console n'a aucun droit sur la table
    let mut tx = ctx(&db.platform, ORG_A, M1).await;
    let e = sqlx::query("SELECT count(*) FROM support_grants")
        .fetch_one(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "immo_platform sans droit sur support_grants : {e}");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn toute_table_a_org_id_porte_les_politiques_de_la_console_sauf_support_grants() {
    let db = TestDb::create().await;
    assert_eq!(schema_audit(&db.owner).await, Vec::<String>::new());
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
          WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relname <> 'sessions' AND c.relname <> 'jobs'
            AND EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid AND a.attname = 'org_id' AND NOT a.attisdropped)
          ORDER BY 1",
    )
    .fetch_all(&db.owner)
    .await
    .unwrap();
    assert!(tables.len() >= 7, "tables à org_id : {tables:?}");
    for table in tables {
        let policies: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_policy p WHERE p.polrelid = $1::regclass
               AND p.polroles = ARRAY[(SELECT oid FROM pg_roles WHERE rolname = 'immo_platform')]",
        )
        .bind(&table)
        .fetch_one(&db.owner)
        .await
        .unwrap();
        if table == "support_grants" {
            assert_eq!(policies, 0, "support_grants : aucune politique pour la console");
            let ok: bool =
                sqlx::query_scalar("SELECT has_table_privilege('immo_platform', 'support_grants', 'SELECT')")
                    .fetch_one(&db.owner)
                    .await
                    .unwrap();
            assert!(!ok, "support_grants : aucun droit pour la console");
        } else {
            assert!(
                policies >= 1,
                "{table} : politique de la console attendue (trouvé {policies})"
            );
        }
    }
    db.drop_db().await;
}

// ───────── paramètres, invitations, droits réglables ─────────

#[tokio::test(flavor = "multi_thread")]
async fn toute_organisation_recoit_ses_parametres_par_defaut() {
    let db = TestDb::create().await;
    seed(&db).await;
    let row = sqlx::query(
        "SELECT receipt_prefix, default_paper, share_link_ttl_days, payment_confirm_threshold, row_version FROM org_settings WHERE org_id = $1::uuid",
    )
    .bind(ORG_A)
    .fetch_one(&db.admin)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>(0), "REC");
    assert_eq!(row.get::<String, _>(1), "a5");
    assert_eq!(row.get::<i32, _>(2), 30);
    assert_eq!(row.get::<i64, _>(3), 1_000_000);
    assert_eq!(row.get::<i32, _>(4), 1);

    // création par la fonction applicative : même résultat
    let app = db.app_pool(2).await;
    let mut tx = app.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.user_id', $1, true)")
        .bind(M2)
        .execute(&mut *tx)
        .await
        .unwrap();
    let id: String = sqlx::query_scalar("SELECT create_organization('Famille D', 'owner', $1::uuid)::text")
        .bind(M2)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM org_settings WHERE org_id = $1::uuid AND receipt_prefix = 'REC'")
            .bind(id)
            .fetch_one(&db.admin)
            .await
            .unwrap();
    assert_eq!(n, 1);

    // contraintes : seuil, durée des liens, préfixe, papier
    for set in [
        "payment_confirm_threshold = 0",
        "payment_confirm_threshold = 1000000000",
        "share_link_ttl_days = 366",
        "receipt_prefix = 'rec'",
        "receipt_prefix = 'A'",
        "default_paper = 'a3'",
    ] {
        let mut tx = ctx(&app, ORG_A, M1).await;
        let e = sqlx::query(sqlx::AssertSqlSafe(format!("UPDATE org_settings SET {set}")))
            .execute(&mut *tx)
            .await
            .unwrap_err();
        assert_eq!(code(&e), "23514", "{set} : {e}");
        tx.rollback().await.unwrap();
    }
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_invitation_se_lit_par_son_empreinte_et_nulle_part_ailleurs() {
    let db = TestDb::create().await;
    seed_rows(&db).await;
    let app = db.app_pool(4).await;
    let hash = vec![0xaa_u8; 32];

    let row = sqlx::query(
        "SELECT org_id::text, org_name, role, identifier_exists, accepted_at IS NULL FROM invitation_by_token($1)",
    )
    .bind(&hash)
    .fetch_one(&app)
    .await
    .unwrap();
    assert_eq!(row.get::<String, _>(0), ORG_A);
    assert_eq!(row.get::<String, _>(1), "Agence A");
    assert_eq!(row.get::<String, _>(2), "cashier");
    assert!(!row.get::<bool, _>(3), "aucun compte pour +22670000099");
    assert!(row.get::<bool, _>(4));

    let none = sqlx::query("SELECT * FROM invitation_by_token($1)")
        .bind(vec![0x01_u8; 32])
        .fetch_all(&app)
        .await
        .unwrap();
    assert!(none.is_empty(), "jeton inconnu : aucune ligne");

    // ni sans contexte, ni en posant soi-même app.token_hash : l'application ne lit pas la table
    let direct: i64 = sqlx::query_scalar("SELECT count(*) FROM invitations")
        .fetch_one(&app)
        .await
        .unwrap();
    assert_eq!(direct, 0);
    let mut tx = app.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.token_hash', $1, true)")
        .bind(format!("\\x{}", "aa".repeat(32)))
        .execute(&mut *tx)
        .await
        .unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM invitations")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 0, "poser app.token_hash ne donne aucun droit à immo_app");
    tx.rollback().await.unwrap();

    // dans le contexte de A, B reste invisible
    let mut tx = ctx(&app, ORG_A, M1).await;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM invitations")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 1);
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn une_seule_invitation_en_attente_par_identifiant_et_validite_de_7_jours() {
    let db = TestDb::create().await;
    seed(&db).await;
    let insert = |hash: &str, phone: &str, expires: &str| {
        format!(
            "INSERT INTO invitations (org_id, invitee_phone, role, token_hash, created_by, created_at, expires_at)
             VALUES ('{ORG_A}', '{phone}', 'cashier', decode('{hash}', 'hex'), '{M1}', '2026-11-01 10:00:00+00', '{expires}')"
        )
    };
    let h = |c: &str| c.repeat(32);
    db.admin
        .execute(sqlx::AssertSqlSafe(insert(
            &h("a1"),
            "+22670000077",
            "2026-11-08 10:00:00+00",
        )))
        .await
        .unwrap();
    let e = db
        .admin
        .execute(sqlx::AssertSqlSafe(insert(
            &h("a2"),
            "+22670000077",
            "2026-11-05 10:00:00+00",
        )))
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23505", "une invitation en attente existe déjà : {e}");
    // après révocation, une nouvelle est possible
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE invitations SET revoked_at = now() WHERE org_id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    db.admin
        .execute(sqlx::AssertSqlSafe(insert(
            &h("a2"),
            "+22670000077",
            "2026-11-05 10:00:00+00",
        )))
        .await
        .unwrap();
    // 7 jours et une seconde : refusé
    let e = db
        .admin
        .execute(sqlx::AssertSqlSafe(insert(
            &h("a3"),
            "+22670000078",
            "2026-11-08 10:00:01+00",
        )))
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514", "invitations_expiry : {e}");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn les_droits_reglables_ne_concernent_que_deux_roles_et_trois_actions() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    for (role, action, ok) in [
        ("property_manager", "record_payment", true),
        ("property_manager", "import_data", true),
        ("property_manager", "export_data", true),
        ("collaborator", "record_payment", true),
        ("cashier", "record_payment", false),
        ("manager", "record_payment", false),
        ("property_manager", "settings_write", false),
    ] {
        let mut tx = ctx(&app, ORG_A, M1).await;
        let r = sqlx::query(
            "INSERT INTO role_settings (org_id, role, action, allowed, set_by) VALUES ($1::uuid, $2, $3, true, $4::uuid)",
        )
        .bind(ORG_A)
        .bind(role)
        .bind(action)
        .bind(M1)
        .execute(&mut *tx)
        .await;
        assert_eq!(r.is_ok(), ok, "{role} × {action}");
        tx.rollback().await.unwrap();
    }
    // les colonnes modifiables : allowed, set_at, set_by, row_version ; pas la clé
    let mut tx = ctx(&app, ORG_A, M1).await;
    sqlx::query("INSERT INTO role_settings (org_id, role, action, allowed, set_by) VALUES ($1::uuid, 'collaborator', 'record_payment', false, $2::uuid)")
        .bind(ORG_A)
        .bind(M1)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE role_settings SET allowed = true, row_version = row_version + 1, set_by = $1::uuid, set_at = app_now()",
    )
    .bind(M1)
    .execute(&mut *tx)
    .await
    .unwrap();
    let e = sqlx::query("UPDATE role_settings SET role = 'property_manager'")
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "la clé ne se modifie pas");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}
