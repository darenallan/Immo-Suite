//! Service d'identité (F01, F02, F03 ; DP-85, DP-86) : connexion, limitation des tentatives, sessions, choix d'organisation,
//! déconnexion, audit dans la même transaction (R-008), détection inter-organisations. Base réelle, horloge de test.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::sync::Arc;

use chrono::Duration;
use immo_app::identity::authorize::{Plan, Role};
use immo_app::identity::password::{HashParams, PasswordService};
use immo_app::identity::service::{AuthError, ClientInfo, IdentityService, LoginError, LoginRequest, SelectError};
use immo_app::platform::db::Db;
use immo_db_tests::{TestClock, TestDb};
use sqlx::{Executor, PgPool, Row};

const ORG_A: &str = "a0000000-0000-4000-8000-00000000000a";
const ORG_B: &str = "b0000000-0000-4000-8000-00000000000b";
const ORG_C: &str = "c0000000-0000-4000-8000-00000000000c";
const GOOD: &str = "un mot de passe correct";
const SECRET: &[u8] = b"secret de session de test, au moins trente-deux octets";

struct Env {
    db: TestDb,
    svc: IdentityService,
    clock: Arc<TestClock>,
    hash: String,
}

fn client(ip: &str) -> ClientInfo {
    ClientInfo {
        ip: Some(ip.parse().unwrap()),
        user_agent: Some("test".to_string()),
        request_id: Some("req-1".to_string()),
    }
}

fn login_req(identifier: &str, password: &str, ip: &str) -> LoginRequest {
    LoginRequest {
        identifier: identifier.to_string(),
        password: password.to_string(),
        client: client(ip),
    }
}

/// Organisations A, B (agences) et C (propriétaire) ; comptes :
/// - +22670000011 « Un » : gérant de A seulement ;
/// - +22670000012 « Deux » : caissier de A et de B (plusieurs organisations) ;
/// - +22670000013 « Trois » : aucun accès ;
/// - +22670000014 « Quatre » : collaborateur de C (plan Propriétaire) ;
/// - un.gerant@agence.bf : gérant de B, connexion par e-mail.
async fn env() -> Env {
    let db = TestDb::create().await;
    let passwords = PasswordService::new(HashParams {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    })
    .unwrap();
    let hash = passwords.hash(GOOD).await.unwrap();
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO organizations (id, name, plan, kind) VALUES ('{ORG_A}', 'Agence A', 'agency', 'real'), ('{ORG_B}', 'Agence B', 'agency', 'real'), ('{ORG_C}', 'Famille C', 'owner', 'real');
             INSERT INTO users (id, login_phone, login_email, display_name, password_hash) VALUES
               ('11111111-0000-4000-8000-000000000011', '+22670000011', NULL, 'Un', '{hash}'),
               ('22222222-0000-4000-8000-000000000012', '+22670000012', NULL, 'Deux', '{hash}'),
               ('33333333-0000-4000-8000-000000000013', '+22670000013', NULL, 'Trois', '{hash}'),
               ('44444444-0000-4000-8000-000000000014', '+22670000014', NULL, 'Quatre', '{hash}'),
               ('55555555-0000-4000-8000-000000000015', NULL, 'un.gerant@agence.bf', 'Cinq', '{hash}');
             INSERT INTO memberships (org_id, user_id, role) VALUES
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'manager'),
               ('{ORG_A}', '22222222-0000-4000-8000-000000000012', 'cashier'),
               ('{ORG_B}', '22222222-0000-4000-8000-000000000012', 'cashier'),
               ('{ORG_C}', '44444444-0000-4000-8000-000000000014', 'collaborator'),
               ('{ORG_B}', '55555555-0000-4000-8000-000000000015', 'manager');"
        )))
        .await
        .unwrap();
    let app = Db::connect(&db.app_url, 4).await.unwrap();
    let clock = Arc::new(TestClock::new());
    let svc = IdentityService::new(app, passwords, clock.clone(), SECRET.to_vec());
    Env { db, svc, clock, hash }
}

/// Un second gérant dans l'organisation A : la garde du dernier gérant (G-16) laisse alors retirer le premier.
async fn add_second_manager(db: &TestDb) {
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO users (id, login_phone, display_name) VALUES ('77777777-0000-4000-8000-000000000017', '+22670000017', 'Sept');
             INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_A}', '77777777-0000-4000-8000-000000000017', 'manager');"
        )))
        .await
        .unwrap();
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_string()))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn audit_actions(pool: &PgPool, org: &str) -> Vec<String> {
    sqlx::query("SELECT action FROM audit_log WHERE org_id = $1::uuid ORDER BY id")
        .bind(org)
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|r| r.get::<String, _>(0))
        .collect()
}

// ───────── connexion ─────────

#[tokio::test(flavor = "multi_thread")]
async fn connexion_reussie_une_organisation_session_audit_et_empreintes() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    assert_eq!(ok.active_org.unwrap().to_string(), ORG_A);
    assert_eq!(ok.organizations.len(), 1);
    assert_eq!(ok.user.display_name, "Un");
    assert_eq!(ok.session_token.len(), 64);
    assert_eq!(ok.csrf_token.len(), 64);
    // seule l'empreinte du jeton est en base, jamais le jeton ni le jeton anti-falsification
    let stored: Vec<u8> = sqlx::query_scalar("SELECT id_hash FROM sessions")
        .fetch_one(&e.db.admin)
        .await
        .unwrap();
    assert_eq!(stored.len(), 32);
    assert_ne!(immo_app::identity::session::to_hex(&stored), ok.session_token);
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sessions WHERE encode(id_hash, 'hex') = $1 OR encode(csrf_hash, 'hex') = $2",
    )
    .bind(&ok.session_token)
    .bind(&ok.csrf_token)
    .fetch_one(&e.db.admin)
    .await
    .unwrap();
    assert_eq!(leaked, 0);
    assert_eq!(audit_actions(&e.db.admin, ORG_A).await, ["auth.login"]);
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM login_attempts WHERE success").await,
        1
    );
    // l'identifiant n'est jamais en clair dans les tentatives
    let plain: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM login_attempts WHERE encode(identifier_hash, 'escape') LIKE '%70000011%'",
    )
    .fetch_one(&e.db.admin)
    .await
    .unwrap();
    assert_eq!(plain, 0);
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn connexion_par_e_mail_et_par_telephone_sous_toutes_ses_formes() {
    let e = env().await;
    e.svc
        .login(login_req("  Un.Gerant@Agence.BF ", GOOD, "10.0.0.1"))
        .await
        .unwrap();
    e.svc
        .login(login_req("+226 70 00 00 11", GOOD, "10.0.0.1"))
        .await
        .unwrap();
    e.svc.login(login_req("0022670000011", GOOD, "10.0.0.1")).await.unwrap();
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn identifiant_inconnu_mot_de_passe_faux_et_forme_invalide_donnent_la_meme_erreur() {
    let e = env().await;
    for (id, pw) in [
        ("70 99 99 99", GOOD),
        ("70 00 00 11", "mauvais mot de passe"),
        ("pas un identifiant", GOOD),
        ("", GOOD),
    ] {
        let err = e.svc.login(login_req(id, pw, "10.0.0.2")).await.unwrap_err();
        assert!(matches!(err, LoginError::InvalidCredentials), "{id:?} : {err:?}");
    }
    // un compte désactivé ne se connecte pas non plus, avec la même erreur
    e.db.admin
        .execute("UPDATE users SET status = 'disabled' WHERE login_phone = '+22670000011'")
        .await
        .unwrap();
    let err = e
        .svc
        .login(login_req("70 00 00 11", GOOD, "10.0.0.2"))
        .await
        .unwrap_err();
    assert!(matches!(err, LoginError::InvalidCredentials));
    assert_eq!(count(&e.db.admin, "SELECT count(*) FROM sessions").await, 0);
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn aucune_adhesion_active_pas_de_session() {
    let e = env().await;
    let err = e
        .svc
        .login(login_req("70 00 00 13", GOOD, "10.0.0.1"))
        .await
        .unwrap_err();
    assert!(matches!(err, LoginError::NoActiveMembership), "{err:?}");
    assert_eq!(count(&e.db.admin, "SELECT count(*) FROM sessions").await, 0);
    // adhésion désactivée : idem (un second gérant existe : la garde du dernier gérant, étape 03, ne refuse pas)
    add_second_manager(&e.db).await;
    e.db.admin
        .execute("UPDATE memberships SET status = 'disabled', disabled_at = now() WHERE user_id = '11111111-0000-4000-8000-000000000011'")
        .await
        .unwrap();
    let err = e
        .svc
        .login(login_req("70 00 00 11", GOOD, "10.0.0.1"))
        .await
        .unwrap_err();
    assert!(matches!(err, LoginError::NoActiveMembership), "{err:?}");
    e.db.drop_db().await;
}

// ───────── limitation (cas chiffrés de F01 §7) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn cinq_echecs_puis_attentes_de_1_5_et_15_minutes() {
    let e = env().await;
    for _ in 0..5 {
        assert!(matches!(
            e.svc
                .login(login_req("70 00 00 11", "faux faux faux", "10.0.0.3"))
                .await,
            Err(LoginError::InvalidCredentials)
        ));
    }
    // 6ᵉ tentative, même avec le bon mot de passe : 429 environ 60 s, sans rien enregistrer
    let attempts_before = count(&e.db.admin, "SELECT count(*) FROM login_attempts").await;
    match e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await {
        Err(LoginError::RateLimited { retry_after_seconds }) => assert_eq!(retry_after_seconds, 60),
        other => panic!("attendu RateLimited, obtenu {other:?}"),
    }
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM login_attempts").await,
        attempts_before,
        "une requête bloquée ne compte pas"
    );
    // le même identifiant écrit autrement est bloqué aussi
    assert!(matches!(
        e.svc.login(login_req("+226 70000011", GOOD, "10.0.0.4")).await,
        Err(LoginError::RateLimited { .. })
    ));
    // l'attente diminue
    e.clock.advance(Duration::seconds(30));
    match e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await {
        Err(LoginError::RateLimited { retry_after_seconds }) => assert_eq!(retry_after_seconds, 30),
        other => panic!("{other:?}"),
    }
    // après une minute : un 6ᵉ échec impose 5 minutes
    e.clock.advance(Duration::seconds(30));
    assert!(matches!(
        e.svc
            .login(login_req("70 00 00 11", "faux faux faux", "10.0.0.3"))
            .await,
        Err(LoginError::InvalidCredentials)
    ));
    match e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await {
        Err(LoginError::RateLimited { retry_after_seconds }) => assert_eq!(retry_after_seconds, 300),
        other => panic!("{other:?}"),
    }
    // après 5 minutes : un 7ᵉ échec impose 15 minutes
    e.clock.advance(Duration::minutes(5));
    assert!(matches!(
        e.svc
            .login(login_req("70 00 00 11", "faux faux faux", "10.0.0.3"))
            .await,
        Err(LoginError::InvalidCredentials)
    ));
    match e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await {
        Err(LoginError::RateLimited { retry_after_seconds }) => assert_eq!(retry_after_seconds, 900),
        other => panic!("{other:?}"),
    }
    // après 15 minutes : le bon mot de passe passe et remet le compteur à zéro
    e.clock.advance(Duration::minutes(15));
    e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await.unwrap();
    for _ in 0..4 {
        assert!(matches!(
            e.svc
                .login(login_req("70 00 00 11", "faux faux faux", "10.0.0.3"))
                .await,
            Err(LoginError::InvalidCredentials)
        ));
    }
    e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await.unwrap();
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_limitation_ne_distingue_pas_un_identifiant_inconnu() {
    let e = env().await;
    for _ in 0..5 {
        assert!(matches!(
            e.svc.login(login_req("70 99 99 99", GOOD, "10.0.0.5")).await,
            Err(LoginError::InvalidCredentials)
        ));
    }
    assert!(matches!(
        e.svc.login(login_req("70 99 99 99", GOOD, "10.0.0.5")).await,
        Err(LoginError::RateLimited { .. })
    ));
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn vingt_echecs_depuis_une_adresse_bloquent_toute_connexion_de_cette_adresse() {
    let e = env().await;
    // 20 identifiants différents (aucun n'atteint 5 échecs), même adresse, en moins de dix minutes
    for i in 0..20 {
        let id = format!("70 10 00 {i:02}");
        assert!(matches!(
            e.svc.login(login_req(&id, GOOD, "10.9.9.9")).await,
            Err(LoginError::InvalidCredentials)
        ));
        e.clock.advance(Duration::seconds(10));
    }
    match e.svc.login(login_req("70 00 00 11", GOOD, "10.9.9.9")).await {
        Err(LoginError::RateLimited { retry_after_seconds }) => assert!(
            retry_after_seconds > 800 && retry_after_seconds <= 900,
            "{retry_after_seconds}"
        ),
        other => panic!("{other:?}"),
    }
    // une autre adresse n'est pas touchée
    e.svc.login(login_req("70 00 00 11", GOOD, "10.9.9.10")).await.unwrap();
    // 15 minutes après le dernier échec : débloquée
    e.clock.advance(Duration::minutes(16));
    e.svc.login(login_req("70 00 00 11", GOOD, "10.9.9.9")).await.unwrap();
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn le_cinquieme_echec_ecrit_security_rate_limited_dans_les_organisations_du_compte_seulement() {
    let e = env().await;
    // compte connu, membre de A et de B
    for _ in 0..5 {
        let _ = e
            .svc
            .login(login_req("70 00 00 12", "faux faux faux", "10.0.0.6"))
            .await;
    }
    assert_eq!(audit_actions(&e.db.admin, ORG_A).await, ["security.rate_limited"]);
    assert_eq!(audit_actions(&e.db.admin, ORG_B).await, ["security.rate_limited"]);
    // identifiant inconnu : nulle part
    for _ in 0..5 {
        let _ = e.svc.login(login_req("70 99 99 98", GOOD, "10.0.0.7")).await;
    }
    assert_eq!(count(&e.db.admin, "SELECT count(*) FROM audit_log").await, 2);
    // un sixième échec n'écrit pas une nouvelle ligne
    e.clock.advance(Duration::minutes(2));
    let _ = e
        .svc
        .login(login_req("70 00 00 12", "faux faux faux", "10.0.0.6"))
        .await;
    assert_eq!(
        count(
            &e.db.admin,
            "SELECT count(*) FROM audit_log WHERE action = 'security.rate_limited'"
        )
        .await,
        2
    );
    e.db.drop_db().await;
}

// ───────── plusieurs organisations, choix, rotation ─────────

#[tokio::test(flavor = "multi_thread")]
async fn plusieurs_organisations_choix_rotation_et_audit_au_premier_choix() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 12", GOOD, "10.0.0.1")).await.unwrap();
    assert_eq!(ok.active_org, None);
    assert_eq!(ok.organizations.len(), 2);
    assert_eq!(ok.organizations[0].name, "Agence A");
    assert_eq!(ok.organizations[1].name, "Agence B");
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM audit_log").await,
        0,
        "aucune organisation active : pas encore d'audit"
    );
    // sans organisation active, la session est reconnue mais sans rôle
    let a = e.svc.authenticate(&ok.session_token).await.unwrap();
    assert!(a.org.is_none() && a.role.is_none());

    let b = ORG_B.parse().unwrap();
    let switched = e
        .svc
        .select_organization(&ok.session_token, b, &client("10.0.0.1"))
        .await
        .unwrap();
    assert_ne!(
        switched.session_token, ok.session_token,
        "nouvel identifiant de session"
    );
    assert_eq!(audit_actions(&e.db.admin, ORG_B).await, ["auth.login"]);
    assert!(
        matches!(
            e.svc.authenticate(&ok.session_token).await,
            Err(AuthError::Unauthenticated)
        ),
        "l'ancien jeton est révoqué"
    );
    let a = e.svc.authenticate(&switched.session_token).await.unwrap();
    assert_eq!(a.org.as_ref().unwrap().id, b);
    assert_eq!(a.role, Some(Role::Cashier));
    assert_eq!(a.org.as_ref().unwrap().plan, Plan::Agency);

    // second changement (B → A) : nouvelle rotation, pas de nouvelle ligne auth.login
    let a_id = ORG_A.parse().unwrap();
    let again = e
        .svc
        .select_organization(&switched.session_token, a_id, &client("10.0.0.1"))
        .await
        .unwrap();
    assert_ne!(again.session_token, switched.session_token);
    assert!(audit_actions(&e.db.admin, ORG_A).await.is_empty());
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn choisir_une_organisation_dont_on_n_est_pas_membre_est_introuvable_et_tracee_chez_l_appelant() {
    let e = env().await;
    // « Un » est gérant de A seulement ; B existe
    let ok = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    let err = e
        .svc
        .select_organization(&ok.session_token, ORG_B.parse().unwrap(), &client("10.0.0.1"))
        .await
        .unwrap_err();
    assert!(matches!(err, SelectError::NotFound), "{err:?}");
    // trace dans l'audit de A (l'appelant), rien chez B
    let a = audit_actions(&e.db.admin, ORG_A).await;
    assert_eq!(a, ["auth.login", "security.cross_org_attempt"]);
    assert!(audit_actions(&e.db.admin, ORG_B).await.is_empty());
    let object: String =
        sqlx::query_scalar("SELECT object_id FROM audit_log WHERE action = 'security.cross_org_attempt'")
            .fetch_one(&e.db.admin)
            .await
            .unwrap();
    assert_eq!(object, ORG_B);
    // la session de l'appelant est intacte (même jeton, même organisation)
    assert_eq!(
        e.svc
            .authenticate(&ok.session_token)
            .await
            .unwrap()
            .org
            .unwrap()
            .id
            .to_string(),
        ORG_A
    );
    // une organisation qui n'existe nulle part : même réponse, aucune trace
    let err = e
        .svc
        .select_organization(
            &ok.session_token,
            "99999999-0000-4000-8000-000000000099".parse().unwrap(),
            &client("10.0.0.1"),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, SelectError::NotFound));
    assert_eq!(audit_actions(&e.db.admin, ORG_A).await.len(), 2);
    e.db.drop_db().await;
}

// ───────── authentification d'une requête ─────────

#[tokio::test(flavor = "multi_thread")]
async fn le_role_est_relu_a_chaque_requete_et_une_adhesion_retiree_coupe_la_session() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    let a = e.svc.authenticate(&ok.session_token).await.unwrap();
    assert_eq!(a.role, Some(Role::Manager));
    assert_eq!(a.user.display_name, "Un");
    // changement de rôle en base : effectif à la requête suivante, sans reconnexion (un second gérant existe : G-16)
    add_second_manager(&e.db).await;
    e.db.admin
        .execute("UPDATE memberships SET role = 'cashier' WHERE user_id = '11111111-0000-4000-8000-000000000011'")
        .await
        .unwrap();
    assert_eq!(
        e.svc.authenticate(&ok.session_token).await.unwrap().role,
        Some(Role::Cashier)
    );
    // adhésion désactivée : la requête suivante est refusée et la session révoquée
    e.db.admin
        .execute("UPDATE memberships SET status = 'disabled', disabled_at = now() WHERE user_id = '11111111-0000-4000-8000-000000000011'")
        .await
        .unwrap();
    assert!(matches!(
        e.svc.authenticate(&ok.session_token).await,
        Err(AuthError::Unauthenticated)
    ));
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM sessions WHERE revoked_at IS NULL").await,
        0
    );
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn compte_desactive_organisation_fermee_jeton_inconnu_ou_mal_forme() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    e.db.admin
        .execute("UPDATE users SET status = 'disabled' WHERE login_phone = '+22670000011'")
        .await
        .unwrap();
    assert!(matches!(
        e.svc.authenticate(&ok.session_token).await,
        Err(AuthError::Unauthenticated)
    ));
    for bad in ["", "abc", &"zz".repeat(32), &"ab".repeat(32)] {
        assert!(
            matches!(e.svc.authenticate(bad).await, Err(AuthError::Unauthenticated)),
            "{bad:?}"
        );
    }
    e.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn expiration_par_inactivite_glissante_et_expiration_absolue() {
    let e = env().await;
    // caissier : 30 minutes d'inactivité
    let ok = e.svc.login(login_req("70 00 00 12", GOOD, "10.0.0.1")).await.unwrap();
    let s = e
        .svc
        .select_organization(&ok.session_token, ORG_A.parse().unwrap(), &client("10.0.0.1"))
        .await
        .unwrap();
    e.clock.advance(Duration::minutes(25));
    e.svc.authenticate(&s.session_token).await.unwrap(); // l'activité repousse l'échéance
    e.clock.advance(Duration::minutes(25));
    e.svc.authenticate(&s.session_token).await.unwrap(); // 50 min depuis la connexion, 25 depuis la dernière activité
    e.clock.advance(Duration::minutes(31));
    assert!(
        matches!(
            e.svc.authenticate(&s.session_token).await,
            Err(AuthError::Unauthenticated)
        ),
        "inactif depuis 31 minutes"
    );

    // gérant : 60 minutes ; expiration absolue à 12 heures même en restant actif
    let g = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    for _ in 0..12 {
        e.clock.advance(Duration::minutes(55));
        e.svc.authenticate(&g.session_token).await.unwrap();
    }
    // 11 h d'activité continue : encore valide, et l'inactivité est plafonnée à l'expiration absolue (12 h)
    e.clock.advance(Duration::minutes(55));
    e.svc.authenticate(&g.session_token).await.unwrap();
    // 11 h 55 : 10 minutes plus tard on dépasse 12 h, malgré l'activité récente
    e.clock.advance(Duration::minutes(10));
    assert!(
        matches!(
            e.svc.authenticate(&g.session_token).await,
            Err(AuthError::Unauthenticated)
        ),
        "au-delà de 12 heures"
    );
    e.db.drop_db().await;
}

// ───────── déconnexion ─────────

#[tokio::test(flavor = "multi_thread")]
async fn deconnexion_et_deconnexion_de_tous_les_appareils() {
    let e = env().await;
    let s1 = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    let s2 = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.2")).await.unwrap();
    let s3 = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.3")).await.unwrap();
    e.svc.logout(&s1.session_token).await.unwrap();
    assert!(matches!(
        e.svc.authenticate(&s1.session_token).await,
        Err(AuthError::Unauthenticated)
    ));
    e.svc.authenticate(&s2.session_token).await.unwrap();
    // idempotent
    e.svc.logout(&s1.session_token).await.unwrap();
    e.svc.logout_all(&s2.session_token, &client("10.0.0.2")).await.unwrap();
    for t in [&s2.session_token, &s3.session_token] {
        assert!(matches!(e.svc.authenticate(t).await, Err(AuthError::Unauthenticated)));
    }
    let row = sqlx::query("SELECT action, after->>'sessions' FROM audit_log WHERE action = 'session.revoked_all'")
        .fetch_one(&e.db.admin)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>(1), "2", "deux sessions encore actives révoquées");
    e.db.drop_db().await;
}

// ───────── organisation suspendue, plan Propriétaire ─────────

#[tokio::test(flavor = "multi_thread")]
async fn organisation_suspendue_et_plan_proprietaire_sont_exposes_a_l_appelant() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 14", GOOD, "10.0.0.1")).await.unwrap();
    let a = e.svc.authenticate(&ok.session_token).await.unwrap();
    assert_eq!(a.role, Some(Role::Collaborator));
    assert_eq!(a.org.as_ref().unwrap().plan, Plan::Owner);
    e.db.admin
        .execute("UPDATE organizations SET lifecycle = 'live', went_live_at = now() WHERE name = 'Famille C'")
        .await
        .unwrap();
    e.db.admin
        .execute("UPDATE organizations SET lifecycle = 'suspended' WHERE name = 'Famille C'")
        .await
        .unwrap();
    let a = e.svc.authenticate(&ok.session_token).await.unwrap();
    assert_eq!(
        a.org.unwrap().lifecycle,
        "suspended",
        "la couche HTTP répond org.suspended"
    );
    // organisation fermée : la session ne tient plus
    e.db.admin
        .execute("UPDATE organizations SET lifecycle = 'closed' WHERE name = 'Famille C'")
        .await
        .unwrap();
    assert!(matches!(
        e.svc.authenticate(&ok.session_token).await,
        Err(AuthError::Unauthenticated)
    ));
    e.db.drop_db().await;
}

// ───────── R-008 : l'audit en échec annule la connexion ─────────

#[tokio::test(flavor = "multi_thread")]
async fn r_008_si_l_audit_echoue_la_connexion_echoue_et_rien_n_est_enregistre() {
    let e = env().await;
    e.db.owner
        .execute("REVOKE INSERT ON audit_log FROM immo_app")
        .await
        .unwrap();
    let err = e
        .svc
        .login(login_req("70 00 00 11", GOOD, "10.0.0.1"))
        .await
        .unwrap_err();
    assert!(matches!(err, LoginError::Internal), "{err:?}");
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM sessions").await,
        0,
        "pas de session sans trace"
    );
    assert_eq!(
        count(&e.db.admin, "SELECT count(*) FROM login_attempts WHERE success").await,
        0
    );
    e.db.owner
        .execute("GRANT INSERT ON audit_log TO immo_app")
        .await
        .unwrap();
    e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    assert!(!e.hash.is_empty());
    e.db.drop_db().await;
}

// ───────── jeton anti-falsification ─────────

#[tokio::test(flavor = "multi_thread")]
async fn le_jeton_anti_falsification_est_celui_de_la_session() {
    let e = env().await;
    let ok = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    assert!(e.svc.csrf_valid(&ok.session_token, &ok.csrf_token));
    assert!(!e.svc.csrf_valid(&ok.session_token, "x"));
    let other = e.svc.login(login_req("70 00 00 11", GOOD, "10.0.0.1")).await.unwrap();
    assert!(!e.svc.csrf_valid(&ok.session_token, &other.csrf_token));
    // empreinte stockée = empreinte du jeton rendu
    let h = immo_app::identity::session::csrf_hash(&ok.csrf_token);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions WHERE csrf_hash = $1")
        .bind(h.to_vec())
        .fetch_one(&e.db.admin)
        .await
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(e.svc.csrf_for(&ok.session_token), ok.csrf_token);
    e.db.drop_db().await;
}
