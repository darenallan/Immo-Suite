//! Identité, organisations et journal d'audit (étape 02, F01, F03, DATABASE_SCHEMA.md §3, §4.2, §6) : isolation (ISO-1, ISO-2),
//! écriture seule du journal pour trois rôles (G-05, ISO-4), cycle de vie, création d'organisation, détection inter-organisations (DP-86).

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use immo_db_tests::{TestDb, schema_audit};
use sqlx::{Executor, PgPool, Postgres, Row, Transaction};

const ORG_A: &str = "a0000000-0000-4000-8000-00000000000a";
const ORG_B: &str = "b0000000-0000-4000-8000-00000000000b";
const U1: &str = "11111111-0000-4000-8000-000000000001"; // membre de A et de B
const U2: &str = "22222222-0000-4000-8000-000000000002"; // membre de A
const U3: &str = "33333333-0000-4000-8000-000000000003"; // membre de B

fn code(e: &sqlx::Error) -> String {
    e.as_database_error()
        .and_then(|d| d.code().map(|c| c.to_string()))
        .unwrap_or_else(|| e.to_string())
}

/// Deux organisations, trois comptes, quatre adhésions, une ligne d'audit par organisation. Écrit par l'administrateur (contourne la RLS).
async fn seed(db: &TestDb) {
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO organizations (id, name, plan) VALUES ('{ORG_A}', 'Agence A', 'agency'), ('{ORG_B}', 'Agence B', 'agency');
             INSERT INTO users (id, login_phone, display_name) VALUES
               ('{U1}', '+22670000001', 'Un'), ('{U2}', '+22670000002', 'Deux'), ('{U3}', '+22670000003', 'Trois');
             INSERT INTO memberships (org_id, user_id, role) VALUES
               ('{ORG_A}', '{U1}', 'manager'), ('{ORG_B}', '{U1}', 'cashier'), ('{ORG_A}', '{U2}', 'cashier'), ('{ORG_B}', '{U3}', 'manager');
             INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES
               ('{ORG_A}', 'user', '{U1}', 'auth.login', 'user', '{U1}'), ('{ORG_B}', 'user', '{U3}', 'auth.login', 'user', '{U3}');"
        )))
        .await
        .unwrap();
}

/// Transaction avec contexte d'organisation (et d'utilisateur), comme `OrgTx::begin`.
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

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_string()))
        .fetch_one(pool)
        .await
        .unwrap()
}

// ───────── audit de schéma ─────────

#[tokio::test(flavor = "multi_thread")]
async fn l_audit_de_schema_passe_avec_les_tables_d_identite() {
    let db = TestDb::create().await;
    assert_eq!(schema_audit(&db.owner).await, Vec::<String>::new());
    for table in ["organizations", "memberships", "audit_log"] {
        let row = sqlx::query(
            "SELECT relrowsecurity, relforcerowsecurity FROM pg_class WHERE relname = $1 AND relkind = 'r'",
        )
        .bind(table)
        .fetch_one(&db.owner)
        .await
        .unwrap();
        assert!(
            row.get::<bool, _>(0) && row.get::<bool, _>(1),
            "{table} : RLS activée et forcée"
        );
    }
    // immo_backup lit toutes les tables d'identité (DP-78)
    for table in [
        "organizations",
        "users",
        "memberships",
        "sessions",
        "login_attempts",
        "audit_log",
    ] {
        let ok: bool = sqlx::query_scalar("SELECT has_table_privilege('immo_backup', $1, 'SELECT')")
            .bind(table)
            .fetch_one(&db.owner)
            .await
            .unwrap();
        assert!(ok, "immo_backup doit lire {table}");
    }
    db.drop_db().await;
}

// ───────── isolation (ISO-1, ISO-2) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn iso_1_sans_contexte_rien_ne_se_lit_ni_ne_s_ecrit() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    for table in ["organizations", "memberships", "audit_log"] {
        assert_eq!(
            count(&app, &format!("SELECT count(*) FROM {table}")).await,
            0,
            "{table} sans contexte"
        );
    }
    let e = sqlx::query(sqlx::AssertSqlSafe(format!(
        "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES ('{ORG_A}', 'user', '{U1}', 'auth.login', 'user', 'x')"
    )))
    .execute(&app)
    .await
    .unwrap_err();
    assert_eq!(code(&e), "42501", "insertion sans contexte refusée par la RLS : {e}");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn iso_2_une_organisation_ne_voit_ni_n_ecrit_chez_l_autre() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let mut tx = ctx(&app, ORG_A, U1).await;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM memberships")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 2, "A ne voit que ses deux adhésions");
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM memberships WHERE org_id = $1::uuid")
        .bind(ORG_B)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 0, "A ne voit pas les adhésions de B, même en les demandant");
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 1, "A ne voit que son audit");
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 1, "A ne voit que sa propre organisation");
    let done =
        sqlx::query("UPDATE memberships SET status = 'disabled', disabled_at = app_now() WHERE org_id = $1::uuid")
            .bind(ORG_B)
            .execute(&mut *tx)
            .await
            .unwrap();
    assert_eq!(done.rows_affected(), 0, "un UPDATE visant B touche 0 ligne");
    let e = sqlx::query("INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES ($1::uuid, 'user', $2::uuid, 'auth.login', 'user', 'x')")
        .bind(ORG_B)
        .bind(U1)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "écrire dans l'audit de B est refusé");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_utilisateur_retrouve_ses_adhesions_avant_de_choisir_une_organisation() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let mut tx = app.begin().await.unwrap();
    // seul l'utilisateur est connu, aucune organisation (politique own_memberships, DATABASE_SCHEMA.md §6.4)
    sqlx::query("SELECT set_config('app.user_id', $1, true)")
        .bind(U1)
        .execute(&mut *tx)
        .await
        .unwrap();
    let rows = sqlx::query("SELECT org_id::text, role FROM memberships ORDER BY role")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|r| r.get::<String, _>(0) == ORG_A || r.get::<String, _>(0) == ORG_B)
    );
    tx.rollback().await.unwrap();
    // un autre utilisateur ne voit que les siennes
    let mut tx = app.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.user_id', $1, true)")
        .bind(U2)
        .execute(&mut *tx)
        .await
        .unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM memberships")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(n, 1);
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

// ───────── journal en écriture seule (G-05, ISO-4) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn g_05_le_journal_refuse_update_delete_truncate_a_trois_roles() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let owner = &db.owner;
    let platform = &db.platform;

    // immo_app : privilèges SELECT et INSERT seulement
    for (stmt, name) in [
        ("UPDATE audit_log SET action = 'auth.logout'", "UPDATE"),
        ("DELETE FROM audit_log", "DELETE"),
        ("TRUNCATE audit_log", "TRUNCATE"),
    ] {
        let mut tx = ctx(&app, ORG_A, U1).await;
        let e = sqlx::query(stmt).execute(&mut *tx).await.unwrap_err();
        assert_eq!(code(&e), "42501", "immo_app {name} : droit absent ({e})");
    }
    // immo_platform : idem
    for (stmt, name) in [
        ("UPDATE audit_log SET action = 'auth.logout'", "UPDATE"),
        ("DELETE FROM audit_log", "DELETE"),
        ("TRUNCATE audit_log", "TRUNCATE"),
    ] {
        let e = sqlx::query(stmt).execute(platform).await.unwrap_err();
        assert_eq!(code(&e), "42501", "immo_platform {name} : droit absent ({e})");
    }
    // immo_owner : tous les droits SQL, mais le déclencheur refuse (barrière 2). Le contexte est posé pour que les lignes soient visibles.
    for (stmt, name) in [
        ("UPDATE audit_log SET action = 'auth.logout'", "UPDATE"),
        ("DELETE FROM audit_log", "DELETE"),
        ("TRUNCATE audit_log", "TRUNCATE"),
    ] {
        let mut tx = ctx(owner, ORG_A, U1).await;
        let e = sqlx::query(stmt).execute(&mut *tx).await.unwrap_err();
        assert_eq!(code(&e), "23001", "immo_owner {name} : refusé par le déclencheur ({e})");
        assert!(e.to_string().contains("ecriture seule"), "{e}");
    }
    assert_eq!(
        count(&db.admin, "SELECT count(*) FROM audit_log").await,
        2,
        "le journal est intact"
    );
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn les_gardes_du_journal_sont_presentes_et_actives() {
    let db = TestDb::create().await;
    let n = count(
        &db.owner,
        "SELECT count(*) FROM pg_trigger WHERE tgrelid = 'audit_log'::regclass AND NOT tgisinternal AND tgenabled = 'O'
            AND tgname IN ('audit_no_mutation', 'audit_no_truncate', 'audit_before_insert')",
    )
    .await;
    assert_eq!(n, 3);
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_heure_et_l_organisation_du_journal_sont_imposees_par_la_base() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let mut tx = ctx(&app, ORG_A, U1).await;
    sqlx::query("INSERT INTO audit_log (org_id, occurred_at, actor_kind, actor_user_id, action, object_type, object_id) VALUES ($1::uuid, '2020-01-01T00:00:00Z', 'user', $2::uuid, 'auth.login', 'user', 'y')")
        .bind(ORG_A)
        .bind(U1)
        .execute(&mut *tx)
        .await
        .unwrap();
    let recent: bool =
        sqlx::query_scalar("SELECT occurred_at > now() - interval '1 minute' FROM audit_log WHERE object_id = 'y'")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert!(recent, "l'heure envoyée (2020) est ignorée : la base écrit la sienne");
    tx.rollback().await.unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn r_008_une_ecriture_d_audit_en_echec_annule_l_action() {
    let db = TestDb::create().await;
    seed(&db).await;
    db.owner
        .execute("CREATE TABLE action_jouet (org_id uuid NOT NULL, n int); ALTER TABLE action_jouet ENABLE ROW LEVEL SECURITY; ALTER TABLE action_jouet FORCE ROW LEVEL SECURITY;
                  CREATE POLICY org_isolation ON action_jouet TO immo_app, immo_owner USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
                  GRANT SELECT, INSERT ON action_jouet TO immo_app")
        .await
        .unwrap();
    let app = db.app_pool(2).await;
    let mut tx = ctx(&app, ORG_A, U1).await;
    sqlx::query("INSERT INTO action_jouet (org_id, n) VALUES ($1::uuid, 1)")
        .bind(ORG_A)
        .execute(&mut *tx)
        .await
        .unwrap();
    // l'écriture d'audit échoue (code d'action mal formé) : l'appelant annule la transaction entière
    let e = sqlx::query("INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id) VALUES ($1::uuid, 'user', $2::uuid, 'Mauvais Code', 'x', 'y')")
        .bind(ORG_A)
        .bind(U1)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514");
    tx.rollback().await.unwrap();
    assert_eq!(
        count(&db.admin, "SELECT count(*) FROM action_jouet").await,
        0,
        "l'action jouet n'a pas eu lieu"
    );
    db.drop_db().await;
}

// ───────── création d'organisation, cycle de vie, démonstration ─────────

#[tokio::test(flavor = "multi_thread")]
async fn create_organization_cree_l_organisation_et_le_premier_membre_sans_deplacer_le_contexte() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let mut tx = app.begin().await.unwrap();
    let id: String =
        sqlx::query_scalar("SELECT create_organization('Agence Nouvelle', 'agency', $1::uuid, 'real')::text")
            .bind(U2)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    // le contexte de l'appelant n'a pas bougé (ici : aucun)
    let current: Option<String> = sqlx::query_scalar("SELECT current_org_id()::text")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(current, None, "create_organization restitue le contexte");
    tx.commit().await.unwrap();
    let row =
        sqlx::query("SELECT lifecycle, plan, kind, timezone, currency, locale FROM organizations WHERE id = $1::uuid")
            .bind(&id)
            .fetch_one(&db.admin)
            .await
            .unwrap();
    assert_eq!(row.get::<String, _>(0), "setup");
    assert_eq!(row.get::<String, _>(1), "agency");
    assert_eq!(row.get::<String, _>(2), "real");
    assert_eq!(row.get::<String, _>(3), "Africa/Ouagadougou");
    assert_eq!(
        (row.get::<String, _>(4), row.get::<String, _>(5)),
        ("XOF".to_string(), "fr".to_string())
    );
    let role: String =
        sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1::uuid AND user_id = $2::uuid")
            .bind(&id)
            .bind(U2)
            .fetch_one(&db.admin)
            .await
            .unwrap();
    assert_eq!(role, "manager", "le premier membre d'une agence est gérant");

    // plan Propriétaire : le premier membre est propriétaire
    let mut tx = app.begin().await.unwrap();
    let id2: String =
        sqlx::query_scalar("SELECT create_organization('Famille Sawadogo', 'owner', $1::uuid, 'real')::text")
            .bind(U3)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    let role: String = sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1::uuid")
        .bind(&id2)
        .fetch_one(&db.admin)
        .await
        .unwrap();
    assert_eq!(role, "owner");

    // plan inconnu, nom trop court, utilisateur inexistant : refusés
    for (name, plan, user) in [
        ("Agence", "gold", U1),
        ("A", "agency", U1),
        ("Agence Z", "agency", "99999999-0000-4000-8000-000000000009"),
    ] {
        let mut tx = app.begin().await.unwrap();
        let r = sqlx::query("SELECT create_organization($1, $2, $3::uuid, 'real')")
            .bind(name)
            .bind(plan)
            .bind(user)
            .execute(&mut *tx)
            .await;
        assert!(r.is_err(), "{name}/{plan}/{user} doit être refusé");
    }
    // immo_platform n'a pas le droit de l'appeler
    let e = sqlx::query("SELECT create_organization('X Y', 'agency', $1::uuid, 'real')")
        .bind(U1)
        .execute(&db.platform)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn les_comptes_de_demonstration_n_entrent_que_dans_les_organisations_de_demonstration() {
    let db = TestDb::create().await;
    seed(&db).await;
    db.admin
        .execute(sqlx::AssertSqlSafe(
            "INSERT INTO users (id, login_phone, display_name, is_demo) VALUES ('44444444-0000-4000-8000-000000000004', '+22670000004', 'Démo', true);
             INSERT INTO organizations (id, name, plan, kind) VALUES ('d0000000-0000-4000-8000-0000000000d0', 'Agence Démo', 'agency', 'demo')"
                .to_string(),
        ))
        .await
        .unwrap();
    // démo dans une organisation réelle : refusé ; réel dans une organisation de démo : refusé ; démo dans démo : accepté
    let e = db.admin.execute(sqlx::AssertSqlSafe(format!("INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_A}', '44444444-0000-4000-8000-000000000004', 'cashier')"))).await.unwrap_err();
    assert!(e.to_string().contains("demonstration"), "{e}");
    let e = db.admin.execute(sqlx::AssertSqlSafe(format!("INSERT INTO memberships (org_id, user_id, role) VALUES ('d0000000-0000-4000-8000-0000000000d0', '{U2}', 'cashier')"))).await.unwrap_err();
    assert!(e.to_string().contains("demonstration"), "{e}");
    db.admin
        .execute("INSERT INTO memberships (org_id, user_id, role) VALUES ('d0000000-0000-4000-8000-0000000000d0', '44444444-0000-4000-8000-000000000004', 'manager')")
        .await
        .unwrap();
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn le_cycle_de_vie_ne_revient_jamais_a_setup() {
    let db = TestDb::create().await;
    seed(&db).await;
    // setup → live exige une date de mise en service (contrainte)
    let e = sqlx::query("UPDATE organizations SET lifecycle = 'live' WHERE id = $1::uuid")
        .bind(ORG_A)
        .execute(&db.platform)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514", "{e}");
    sqlx::query("UPDATE organizations SET lifecycle = 'live', went_live_at = app_now() WHERE id = $1::uuid")
        .bind(ORG_A)
        .execute(&db.platform)
        .await
        .unwrap();
    // live → setup : refusé par le déclencheur, y compris pour la console
    let e = sqlx::query("UPDATE organizations SET lifecycle = 'setup' WHERE id = $1::uuid")
        .bind(ORG_A)
        .execute(&db.platform)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23001", "{e}");
    // suspended → live : permis
    sqlx::query("UPDATE organizations SET lifecycle = 'suspended' WHERE id = $1::uuid")
        .bind(ORG_A)
        .execute(&db.platform)
        .await
        .unwrap();
    sqlx::query("UPDATE organizations SET lifecycle = 'live' WHERE id = $1::uuid")
        .bind(ORG_A)
        .execute(&db.platform)
        .await
        .unwrap();
    // l'application ne change ni le cycle de vie, ni le plan, ni le type ; elle change le nom et la ville de sa propre organisation
    let app = db.app_pool(2).await;
    let mut tx = ctx(&app, ORG_A, U1).await;
    for col in [
        "lifecycle = 'closed'",
        "plan = 'owner'",
        "kind = 'test'",
        "timezone = 'Africa/Ouagadougou'",
    ] {
        let e = sqlx::query(sqlx::AssertSqlSafe(format!("UPDATE organizations SET {col}")))
            .execute(&mut *tx)
            .await
            .unwrap_err();
        assert_eq!(code(&e), "42501", "immo_app ne modifie pas {col}");
        tx.rollback().await.unwrap();
        tx = ctx(&app, ORG_A, U1).await;
    }
    let done = sqlx::query("UPDATE organizations SET name = 'Agence A bis', city = 'Ouagadougou'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(done.rows_affected(), 1, "seulement sa propre organisation");
    tx.commit().await.unwrap();
    db.drop_db().await;
}

// ───────── détection d'un accès à une autre organisation (DP-86) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn exists_in_other_org_repond_un_booleen_sur_une_liste_blanche() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    let mut tx = ctx(&app, ORG_A, U1).await;
    let b_exists: bool = sqlx::query_scalar("SELECT exists_in_other_org('organization', $1::uuid)")
        .bind(ORG_B)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(b_exists, "B existe et n'est pas l'organisation de l'appelant");
    let own: bool = sqlx::query_scalar("SELECT exists_in_other_org('organization', $1::uuid)")
        .bind(ORG_A)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(!own, "sa propre organisation n'est pas « une autre »");
    let nowhere: bool = sqlx::query_scalar("SELECT exists_in_other_org('organization', gen_random_uuid())")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert!(
        !nowhere,
        "un identifiant qui n'existe nulle part n'est pas une tentative"
    );
    let e = sqlx::query("SELECT exists_in_other_org('payments', gen_random_uuid())")
        .fetch_one(&mut *tx)
        .await
        .unwrap_err();
    assert!(
        e.to_string().contains("liste blanche"),
        "type hors liste blanche refusé : {e}"
    );
    tx.rollback().await.unwrap();

    // table jouet inscrite par le propriétaire : la liste blanche est une donnée
    db.owner
        .execute(
            "CREATE TABLE jouet_cross (id uuid PRIMARY KEY, org_id uuid NOT NULL);
             ALTER TABLE jouet_cross ENABLE ROW LEVEL SECURITY; ALTER TABLE jouet_cross FORCE ROW LEVEL SECURITY;
             CREATE POLICY org_isolation ON jouet_cross TO immo_app, immo_owner USING (org_id = current_org_id()) WITH CHECK (org_id = current_org_id());
             INSERT INTO cross_org_registry (object_type, table_name, org_column) VALUES ('jouet', 'jouet_cross', 'org_id'); GRANT SELECT ON jouet_cross TO immo_app;",
        )
        .await
        .unwrap();
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO jouet_cross (id, org_id) VALUES ('cccccccc-0000-4000-8000-00000000000c', '{ORG_B}')"
        )))
        .await
        .unwrap();
    let mut tx = ctx(&app, ORG_A, U1).await;
    let seen: bool =
        sqlx::query_scalar("SELECT exists_in_other_org('jouet', 'cccccccc-0000-4000-8000-00000000000c'::uuid)")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert!(
        seen,
        "la ligne de B est signalée à A comme existant ailleurs, sans qu'aucune donnée ne soit rendue"
    );
    let direct: i64 = sqlx::query_scalar("SELECT count(*) FROM jouet_cross")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(direct, 0, "A ne lit toujours rien de B");
    tx.rollback().await.unwrap();
    // l'application ne peut pas modifier la liste blanche
    let e =
        sqlx::query("INSERT INTO cross_org_registry (object_type, table_name, org_column) VALUES ('x', 'users', 'id')")
            .execute(&app)
            .await
            .unwrap_err();
    assert_eq!(code(&e), "42501");
    db.drop_db().await;
}

// ───────── comptes, sessions, tentatives : droits et contraintes ─────────

#[tokio::test(flavor = "multi_thread")]
async fn users_contraintes_de_format_et_droits_par_colonne() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    for (sql, why) in [
        (
            "INSERT INTO users (login_phone, display_name) VALUES ('+2267000', 'Court')",
            "téléphone mal formé",
        ),
        (
            "INSERT INTO users (login_email, display_name) VALUES ('Maj@Exemple.org', 'Maj')",
            "e-mail en majuscules",
        ),
        (
            "INSERT INTO users (display_name) VALUES ('Sans identifiant')",
            "ni téléphone ni e-mail",
        ),
        (
            "INSERT INTO users (login_phone, display_name, password_hash) VALUES ('+22670000009', 'Md5', 'md5abc')",
            "empreinte non argon2id",
        ),
        (
            "INSERT INTO users (login_phone, display_name) VALUES ('+22670000001', 'Doublon')",
            "téléphone déjà pris",
        ),
    ] {
        let e = sqlx::query(sql).execute(&app).await.unwrap_err();
        assert!(matches!(code(&e).as_str(), "23514" | "23505"), "{why} : {e}");
    }
    sqlx::query("INSERT INTO users (login_phone, display_name, password_hash) VALUES ('+22670000009', 'Bon', '$argon2id$v=19$m=19456,t=2,p=1$xxxx$yyyy')")
        .execute(&app)
        .await
        .unwrap();
    // colonnes modifiables : oui ; identifiant, is_demo, created_at : non
    sqlx::query(
        "UPDATE users SET display_name = 'Bon Nom', last_login_at = app_now() WHERE login_phone = '+22670000009'",
    )
    .execute(&app)
    .await
    .unwrap();
    for col in ["is_demo = true", "created_at = app_now()", "id = gen_random_uuid()"] {
        let e = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE users SET {col} WHERE login_phone = '+22670000009'"
        )))
        .execute(&app)
        .await
        .unwrap_err();
        assert_eq!(code(&e), "42501", "{col}");
    }
    // la console n'a aucun accès aux comptes
    let e = sqlx::query("SELECT count(*) FROM users")
        .fetch_one(&db.platform)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_et_tentatives_privileges_minimaux() {
    let db = TestDb::create().await;
    seed(&db).await;
    let app = db.app_pool(2).await;
    sqlx::query("INSERT INTO sessions (id_hash, user_id, org_id, csrf_hash, idle_expires_at, absolute_expires_at) VALUES (decode(repeat('ab', 32), 'hex'), $1::uuid, $2::uuid, decode(repeat('cd', 32), 'hex'), app_now() + interval '30 minutes', app_now() + interval '12 hours')")
        .bind(U1)
        .bind(ORG_A)
        .execute(&app)
        .await
        .unwrap();
    sqlx::query("UPDATE sessions SET revoked_at = app_now(), last_seen_at = app_now()")
        .execute(&app)
        .await
        .unwrap();
    for col in [
        "csrf_hash = decode(repeat('00', 32), 'hex')",
        "user_id = gen_random_uuid()",
        "absolute_expires_at = app_now()",
    ] {
        let e = sqlx::query(sqlx::AssertSqlSafe(format!("UPDATE sessions SET {col}")))
            .execute(&app)
            .await
            .unwrap_err();
        assert_eq!(code(&e), "42501", "{col}");
    }
    // expiration d'inactivité postérieure à l'absolue : refusée
    let e = sqlx::query("INSERT INTO sessions (id_hash, user_id, csrf_hash, idle_expires_at, absolute_expires_at) VALUES (decode(repeat('ee', 32), 'hex'), $1::uuid, decode(repeat('cd', 32), 'hex'), app_now() + interval '2 hours', app_now() + interval '1 hour')")
        .bind(U1)
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "23514");
    // tentatives : insertion et suppression (purge à 30 jours) ; jamais de modification
    sqlx::query("INSERT INTO login_attempts (identifier_hash, ip, success) VALUES (decode(repeat('11', 32), 'hex'), '10.0.0.1', false)").execute(&app).await.unwrap();
    let e = sqlx::query("UPDATE login_attempts SET success = true")
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501");
    let n = sqlx::query("DELETE FROM login_attempts WHERE attempted_at < app_now() - interval '30 days'")
        .execute(&app)
        .await
        .unwrap();
    assert_eq!(n.rows_affected(), 0);
    // la console ne voit ni sessions ni tentatives
    for t in ["sessions", "login_attempts"] {
        let e = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {t}")))
            .fetch_one(&db.platform)
            .await
            .unwrap_err();
        assert_eq!(code(&e), "42501", "{t}");
    }
    db.drop_db().await;
}
