//! Étape 03, API : mot de passe, réinitialisation par lien, double authentification, profil (F01 §9.3 à §9.7).
//! Cas chiffrés : lien de réinitialisation accepté à 10:14:59, refusé à 10:15:00 ; codes TOTP aux pas −1, 0, +1, ±2 ; 6 essais en 60 s.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use axum::http::{Method, StatusCode};
use chrono::{Duration, TimeZone, Utc};
use common::{GOOD, ORG_A, ORG_B, ORIGIN, S, actions, api, call, count, fast_params, login, reauth, req, sess};
use immo_app::identity::crypto::FieldKey;
use immo_app::identity::totp;
use immo_app::platform::clock::Clock;
use serde_json::{Value, json};
use sqlx::Executor;

const DEUX: &str = "22222222-0000-4000-8000-000000000012";
const UN: &str = "11111111-0000-4000-8000-000000000011";
const SIX: &str = "66666666-0000-4000-8000-000000000016";

async fn choose(a: &common::Api, identifier: &str, org: &str, ip: &str) -> S {
    let (cookie, _) = login(&a.app, identifier, GOOD, ip).await;
    let csrf = a.svc.csrf_for(cookie.split('=').nth(1).unwrap());
    let chosen = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        ip,
        &[
            ("cookie", &cookie),
            ("origin", ORIGIN),
            ("content-type", "application/json"),
            ("x-csrf-token", &csrf),
        ],
        Some(&json!({ "organization_id": org }).to_string()),
    )
    .await;
    assert_eq!(chosen.status, StatusCode::OK, "{}", chosen.body);
    S {
        cookie: chosen.session_cookie().unwrap().split(';').next().unwrap().to_string(),
        csrf: chosen.json()["csrf_token"].as_str().unwrap().to_string(),
    }
}

async fn membership_id(a: &common::Api, org: &str, user: &str) -> String {
    sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(format!(
        "SELECT id::text FROM memberships WHERE org_id = '{org}' AND user_id = '{user}'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .unwrap()
}

// ───────── changement de mot de passe ─────────

#[tokio::test(flavor = "multi_thread")]
async fn changer_son_mot_de_passe_revoque_les_autres_sessions_et_renouvelle_la_courante() {
    let a = api(fast_params(), "0.0.0").await;
    let first = choose(&a, "70 00 00 12", ORG_A, "10.31.0.1").await;
    let second = choose(&a, "70 00 00 12", ORG_B, "10.31.0.2").await;
    let put = |body: Value| req(&a, &first, Method::PUT, "/api/me/password", Some(body), &[]);
    // règles de choix : 9, 10, 128 et 129 caractères (cas chiffré n° 4), mot de passe courant, identifiant
    let weak = |r: &common::Reply| r.json()["params"]["reason"].as_str().unwrap_or_default().to_string();
    assert_eq!(
        weak(&put(json!({ "current_password": GOOD, "new_password": "123456789a" })).await),
        "common",
        "mot de passe courant"
    );
    assert_eq!(
        weak(&put(json!({ "current_password": GOOD, "new_password": "neuf cars" })).await),
        "too_short"
    );
    assert_eq!(
        weak(&put(json!({ "current_password": GOOD, "new_password": "x".repeat(129) })).await),
        "too_long"
    );
    assert_eq!(
        weak(&put(json!({ "current_password": GOOD, "new_password": "mon 70000012 ici" })).await),
        "contains_identifier"
    );
    assert_eq!(
        weak(&put(json!({ "current_password": GOOD, "new_password": GOOD })).await),
        "same_as_current"
    );
    let wrong = put(json!({ "current_password": "pas le bon", "new_password": "un nouveau mot de passe" })).await;
    assert_eq!(
        (wrong.status, wrong.json()["code"].clone()),
        (StatusCode::UNAUTHORIZED, json!("auth.invalid_credentials"))
    );
    // 10 caractères exactement : accepté
    let ok = put(json!({ "current_password": GOOD, "new_password": "dix chars!" })).await;
    assert_eq!(ok.status, StatusCode::NO_CONTENT, "{}", ok.body);
    let cookie = ok
        .session_cookie()
        .expect("cookie renouvelé")
        .split(';')
        .next()
        .unwrap()
        .to_string();
    assert_ne!(cookie, first.cookie, "nouvel identifiant de session");
    // l'ancien identifiant est mort, l'autre session aussi, la nouvelle vit
    assert_eq!(
        req(&a, &first, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        req(&a, &second, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::UNAUTHORIZED
    );
    let fresh = S {
        cookie,
        csrf: a.svc.csrf_for(
            ok.session_cookie()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .split('=')
                .nth(1)
                .unwrap(),
        ),
    };
    assert_eq!(
        req(&a, &fresh, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::OK
    );
    // ancien mot de passe refusé, nouveau accepté
    let old = call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        "10.31.0.3",
        &[("content-type", "application/json")],
        Some(&common::login_json("70 00 00 12", GOOD)),
    )
    .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    login(&a.app, "70 00 00 12", "dix chars!", "10.31.0.4").await;
    // une ligne dans l'audit de CHAQUE organisation du compte (compte global, audit par organisation)
    assert_eq!(
        actions(&a, ORG_A)
            .await
            .iter()
            .filter(|x| *x == "auth.password_changed")
            .count(),
        1
    );
    assert_eq!(
        actions(&a, ORG_B)
            .await
            .iter()
            .filter(|x| *x == "auth.password_changed")
            .count(),
        1
    );
    let after: Value = sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(format!(
        "SELECT after::text FROM audit_log WHERE org_id = '{ORG_A}' AND action = 'auth.password_changed'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .map(|t| serde_json::from_str(&t).unwrap())
    .unwrap();
    assert_eq!(after, json!({ "sessions_revoked": 1 }));
    a.db.drop_db().await;
}

// ───────── réinitialisation par lien ─────────

fn reset_token(r: &common::Reply) -> String {
    common::token_of(r.json()["url"].as_str().unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn lien_de_reinitialisation_quinze_minutes_a_usage_unique() {
    let a = api(fast_params(), "0.0.0").await;
    // le jeu de test commence le 13/11/2026 à 09:15:00 ; on se place à 10:00:00 pour le cas chiffré n° 2
    a.clock.advance(Duration::minutes(45));
    let g = sess(&a, "70 00 00 11").await;
    let mid = membership_id(&a, ORG_A, DEUX).await;
    // sans confirmation : refus avant tout effet
    let r = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{mid}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(r.json()["code"], "auth.reauth_required");
    assert_eq!(count(&a, "SELECT count(*) FROM password_resets").await, 0);
    reauth(&a, &g).await;
    let issued = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{mid}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(issued.status, StatusCode::OK, "{}", issued.body);
    assert!(
        issued.json()["url"]
            .as_str()
            .unwrap()
            .starts_with("https://app.test/reinitialisation/")
    );
    assert_eq!(issued.json()["expires_at"], "2026-11-13T10:15:00+00:00");
    let token = reset_token(&issued);
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM password_resets WHERE encode(token_hash, 'escape') LIKE '%{token}%'")
        )
        .await,
        0,
        "seule l'empreinte est stockée"
    );

    let complete = |token: String, password: &'static str, ip: &'static str| {
        let app = a.app.clone();
        async move {
            call(
                &app,
                Method::POST,
                "/api/auth/password-reset/complete",
                ip,
                &[("content-type", "application/json")],
                Some(&json!({ "token": token, "new_password": password }).to_string()),
            )
            .await
        }
    };
    // mot de passe faible : le lien reste valable
    let weak = complete(token.clone(), "court", "10.31.1.1").await;
    assert_eq!(
        (weak.status, weak.json()["params"]["reason"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("too_short"))
    );
    // 10:14:59 : accepté
    a.clock.advance(Duration::minutes(14) + Duration::seconds(59));
    // « Deux » avait une session ouverte : elle sera révoquée
    let open = sess_for(&a, "70 00 00 12", ORG_A, "10.31.1.2").await;
    assert_eq!(
        req(&a, &open, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::OK
    );
    let done = complete(token.clone(), "un nouveau mot de passe", "10.31.1.3").await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.body);
    assert!(done.session_cookie().is_none(), "aucune session ouverte par le lien");
    assert_eq!(
        req(&a, &open, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::UNAUTHORIZED,
        "sessions révoquées"
    );
    // second usage : 410, même corps qu'un lien inconnu
    let again = complete(token.clone(), "un autre mot de passe", "10.31.1.4").await;
    let unknown = complete("a".repeat(43), "un autre mot de passe", "10.31.1.5").await;
    assert_eq!(
        (again.status, again.json()["code"].clone()),
        (StatusCode::GONE, json!("auth.reset_expired"))
    );
    assert_eq!(
        again.body.replace(again.json()["request_id"].as_str().unwrap(), ""),
        unknown.body.replace(unknown.json()["request_id"].as_str().unwrap(), ""),
        "même corps pour tous les liens inutilisables"
    );
    // connexion avec le nouveau mot de passe
    login(&a.app, "70 00 00 12", "un nouveau mot de passe", "10.31.1.6").await;
    let log = actions(&a, ORG_A).await;
    assert!(
        log.contains(&"auth.reset_issued".to_string()) && log.contains(&"auth.reset_used".to_string()),
        "{log:?}"
    );
    a.db.drop_db().await;
}

async fn sess_for(a: &common::Api, identifier: &str, org: &str, ip: &str) -> S {
    choose(a, identifier, org, ip).await
}

#[tokio::test(flavor = "multi_thread")]
async fn le_lien_est_refuse_a_dix_heures_quinze_pile() {
    let a = api(fast_params(), "0.0.0").await;
    a.clock.advance(Duration::minutes(45));
    assert_eq!(a.clock.now(), Utc.with_ymd_and_hms(2026, 11, 13, 10, 0, 0).unwrap());
    let g = sess(&a, "70 00 00 11").await;
    reauth(&a, &g).await;
    let mid = membership_id(&a, ORG_A, DEUX).await;
    let token = reset_token(
        &req(
            &a,
            &g,
            Method::POST,
            &format!("/api/users/{mid}/password-resets"),
            None,
            &[],
        )
        .await,
    );
    a.clock.advance(Duration::minutes(15));
    let late = call(
        &a.app,
        Method::POST,
        "/api/auth/password-reset/complete",
        "10.31.2.1",
        &[("content-type", "application/json")],
        Some(&json!({ "token": token, "new_password": "un nouveau mot de passe" }).to_string()),
    )
    .await;
    assert_eq!(
        (late.status, late.json()["code"].clone()),
        (StatusCode::GONE, json!("auth.reset_expired")),
        "10:15:00 : refusé"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_nouveau_lien_annule_le_precedent_et_le_sixieme_en_une_heure_est_refuse() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    reauth(&a, &g).await;
    let mid = membership_id(&a, ORG_A, DEUX).await;
    let path = format!("/api/users/{mid}/password-resets");
    let mut tokens = vec![];
    for _ in 0..5 {
        let r = req(&a, &g, Method::POST, &path, None, &[]).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        tokens.push(reset_token(&r));
    }
    let sixth = req(&a, &g, Method::POST, &path, None, &[]).await;
    assert_eq!(
        (sixth.status, sixth.json()["code"].clone()),
        (StatusCode::TOO_MANY_REQUESTS, json!("auth.rate_limited")),
        "cas chiffré n° 3"
    );
    // seul le dernier lien est vivant
    let try_with = |token: String| {
        let app = a.app.clone();
        async move {
            call(
                &app,
                Method::POST,
                "/api/auth/password-reset/complete",
                "10.31.3.1",
                &[("content-type", "application/json")],
                Some(&json!({ "token": token, "new_password": "court" }).to_string()),
            )
            .await
        }
    };
    assert_eq!(
        try_with(tokens[0].clone()).await.status,
        StatusCode::GONE,
        "révoqué par le suivant"
    );
    assert_eq!(
        try_with(tokens[4].clone()).await.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "le dernier est vivant (mot de passe faible)"
    );
    // une heure plus tard, un lien est de nouveau possible
    a.clock.advance(Duration::hours(1) + Duration::seconds(1));
    let g = sess(&a, "70 00 00 11").await;
    reauth(&a, &g).await;
    assert_eq!(req(&a, &g, Method::POST, &path, None, &[]).await.status, StatusCode::OK);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_gerant_n_est_jamais_reinitialise_par_un_autre_ni_soi_meme_ni_un_membre_desactive() {
    let a = api(fast_params(), "0.0.0").await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO users (id, login_phone, display_name, password_hash) SELECT '77777777-0000-4000-8000-000000000017', '+22670000017', 'Sept', password_hash FROM users WHERE id = '{UN}';
             INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_A}', '77777777-0000-4000-8000-000000000017', 'manager');"
        )))
        .await
        .unwrap();
    let g = sess(&a, "70 00 00 11").await;
    reauth(&a, &g).await;
    let other_manager = membership_id(&a, ORG_A, "77777777-0000-4000-8000-000000000017").await;
    let r = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{other_manager}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(
        (r.status, r.json()["code"].clone(), r.json()["detail"].clone()),
        (
            StatusCode::FORBIDDEN,
            json!("auth.forbidden"),
            json!("Pour un gérant, contactez le support HABEK.")
        )
    );
    let own = membership_id(&a, ORG_A, UN).await;
    let r = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{own}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    // un membre désactivé : introuvable
    let mid = membership_id(&a, ORG_A, DEUX).await;
    assert_eq!(
        req(&a, &g, Method::POST, &format!("/api/users/{mid}/disable"), None, &[])
            .await
            .status,
        StatusCode::OK
    );
    let r = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{mid}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert_eq!(count(&a, "SELECT count(*) FROM password_resets").await, 0);
    a.db.drop_db().await;
}

// ───────── double authentification ─────────

fn key() -> FieldKey {
    FieldKey::from_bytes(&[9_u8; 32]).unwrap()
}

async fn secret_of(a: &common::Api, user: &str) -> Vec<u8> {
    let blob: Vec<u8> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT totp_secret_enc FROM users WHERE id = '{user}'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .unwrap();
    key().decrypt(&blob).unwrap()
}

fn code_at(a: &common::Api, secret: &[u8], delta: i64) -> String {
    let step = totp::step_at(a.clock.now()) + delta;
    format!("{:06}", totp::code_for_step(secret, u64::try_from(step).unwrap()))
}

async fn enable_totp(a: &common::Api, s: &S, user: &str) -> (Vec<u8>, Vec<String>) {
    let begin = req(a, s, Method::POST, "/api/me/totp", None, &[]).await;
    assert_eq!(begin.status, StatusCode::OK, "{}", begin.body);
    assert!(
        begin.json()["otpauth_url"]
            .as_str()
            .unwrap()
            .starts_with("otpauth://totp/Immo%20Suite:")
    );
    let secret = secret_of(a, user).await;
    // le secret en clair n'est ni en base ni dans la réponse hors cette seule fois
    let blob: Vec<u8> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT totp_secret_enc FROM users WHERE id = '{user}'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .unwrap();
    assert!(
        !blob.windows(secret.len()).any(|w| w == secret.as_slice()),
        "secret chiffré en base"
    );
    assert!(begin.json()["secret_base32"].as_str().unwrap().len() >= 32);
    let bad = req(
        a,
        s,
        Method::POST,
        "/api/me/totp/confirm",
        Some(json!({ "code": "000000" })),
        &[],
    )
    .await;
    assert_eq!(
        (bad.status, bad.json()["code"].clone()),
        (StatusCode::UNAUTHORIZED, json!("auth.totp_invalid"))
    );
    let confirm = req(
        a,
        s,
        Method::POST,
        "/api/me/totp/confirm",
        Some(json!({ "code": code_at(a, &secret, 0) })),
        &[],
    )
    .await;
    assert_eq!(confirm.status, StatusCode::OK, "{}", confirm.body);
    let codes: Vec<String> = confirm.json()["backup_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    assert_eq!(codes.len(), 10);
    (secret, codes)
}

fn login_for_challenge(
    a: &common::Api,
    identifier: &str,
    ip: &'static str,
) -> impl std::future::Future<Output = common::Reply> + use<> {
    let app = a.app.clone();
    let body = common::login_json(identifier, GOOD);
    async move {
        call(
            &app,
            Method::POST,
            "/api/auth/login",
            ip,
            &[("content-type", "application/json")],
            Some(&body),
        )
        .await
    }
}

async fn verify(a: &common::Api, challenge: &str, code: &str, ip: &str) -> common::Reply {
    call(
        &a.app,
        Method::POST,
        "/api/auth/totp/verify",
        ip,
        &[("content-type", "application/json")],
        Some(&json!({ "challenge": challenge, "code": code }).to_string()),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn activer_la_double_authentification_puis_se_connecter_avec_le_code() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (secret, _codes) = enable_totp(&a, &g, UN).await;
    // déjà activée : refus
    let again = req(&a, &g, Method::POST, "/api/me/totp", None, &[]).await;
    assert_eq!(again.json()["params"]["reason"], "already_enabled");

    a.clock.advance(Duration::seconds(30)); // un nouveau pas, pour que le pas de l'activation ne soit pas rejoué
    let first = login_for_challenge(&a, "70 00 00 11", "10.32.0.1").await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(first.json()["mfa_required"], true);
    assert!(first.session_cookie().is_none(), "aucune session avant le code");
    assert_eq!(count(&a, "SELECT count(*) FROM sessions WHERE revoked_at IS NULL AND user_id = '11111111-0000-4000-8000-000000000011'").await, 1, "seule la session de l'activation");
    let challenge = first.json()["challenge"].as_str().unwrap().to_string();

    // un défi altéré ou venu d'ailleurs : session expirée
    let tampered = format!("{}x", challenge);
    assert_eq!(
        verify(&a, &tampered, &code_at(&a, &secret, 0), "10.32.0.2")
            .await
            .json()["code"],
        "auth.session_expired"
    );
    // mauvais code : refusé
    assert_eq!(
        verify(&a, &challenge, "000000", "10.32.0.2").await.json()["code"],
        "auth.totp_invalid"
    );
    // bon code : session ouverte
    let ok = verify(&a, &challenge, &code_at(&a, &secret, 0), "10.32.0.2").await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert!(ok.session_cookie().is_some());
    assert_eq!(ok.json()["mfa_required"], false);
    // rejeu du même code : refusé (cas chiffré n° 6)
    let second = login_for_challenge(&a, "70 00 00 11", "10.32.0.3").await;
    let replay = verify(
        &a,
        second.json()["challenge"].as_str().unwrap(),
        &code_at(&a, &secret, 0),
        "10.32.0.3",
    )
    .await;
    assert_eq!(
        (replay.status, replay.json()["code"].clone()),
        (StatusCode::UNAUTHORIZED, json!("auth.totp_invalid"))
    );
    // le défi expire après 5 minutes
    a.clock.advance(Duration::minutes(5) + Duration::seconds(1));
    let late = verify(
        &a,
        second.json()["challenge"].as_str().unwrap(),
        &code_at(&a, &secret, 0),
        "10.32.0.3",
    )
    .await;
    assert_eq!(late.json()["code"], "auth.session_expired");
    let log = actions(&a, ORG_A).await;
    assert!(log.contains(&"auth.mfa_enabled".to_string()), "{log:?}");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn decalage_d_horloge_d_un_pas_accepte_de_deux_pas_refuse() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (secret, _) = enable_totp(&a, &g, UN).await;
    for (delta, accepted) in [(-1_i64, true), (1, true), (-2, false), (2, false)] {
        a.clock.advance(Duration::seconds(30 * 10)); // loin du dernier pas accepté
        let c = login_for_challenge(&a, "70 00 00 11", "10.32.1.1").await;
        let r = verify(
            &a,
            c.json()["challenge"].as_str().unwrap(),
            &code_at(&a, &secret, delta),
            "10.32.1.1",
        )
        .await;
        assert_eq!(r.status == StatusCode::OK, accepted, "pas {delta:+} : {}", r.body);
    }
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn six_codes_faux_en_soixante_secondes_la_sixieme_est_limitee_sans_tester_le_code() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (secret, _) = enable_totp(&a, &g, UN).await;
    a.clock.advance(Duration::seconds(60));
    let c = login_for_challenge(&a, "70 00 00 11", "10.32.2.1").await;
    let challenge = c.json()["challenge"].as_str().unwrap().to_string();
    for _ in 0..5 {
        assert_eq!(
            verify(&a, &challenge, "111111", "10.32.2.1").await.json()["code"],
            "auth.totp_invalid"
        );
    }
    // le sixième essai porte le BON code : il n'est pas testé
    let sixth = verify(&a, &challenge, &code_at(&a, &secret, 0), "10.32.2.1").await;
    assert_eq!(
        (sixth.status, sixth.json()["code"].clone()),
        (StatusCode::TOO_MANY_REQUESTS, json!("auth.rate_limited"))
    );
    assert!(sixth.headers.get("retry-after").is_some());
    // l'attente ne compte pas : une minute plus tard le bon code passe
    a.clock.advance(Duration::seconds(61));
    let c = login_for_challenge(&a, "70 00 00 11", "10.32.2.2").await;
    let ok = verify(
        &a,
        c.json()["challenge"].as_str().unwrap(),
        &code_at(&a, &secret, 0),
        "10.32.2.2",
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_code_de_secours_ne_sert_qu_une_fois_et_laisse_sa_trace() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (_, codes) = enable_totp(&a, &g, UN).await;
    assert_eq!(
        count(&a, "SELECT count(*) FROM totp_backup_codes WHERE used_at IS NULL").await,
        10
    );
    // les empreintes seules sont en base
    let stored: Vec<Vec<u8>> = sqlx::query_scalar("SELECT code_hash FROM totp_backup_codes")
        .fetch_all(&a.db.admin)
        .await
        .unwrap();
    for c in &codes {
        let plain = c.replace('-', "");
        assert!(
            stored
                .iter()
                .all(|h| !h.windows(plain.len()).any(|w| w == plain.as_bytes())),
            "code en clair en base"
        );
    }
    let c = login_for_challenge(&a, "70 00 00 11", "10.32.3.1").await;
    let used = verify(&a, c.json()["challenge"].as_str().unwrap(), &codes[0], "10.32.3.1").await;
    assert_eq!(used.status, StatusCode::OK, "{}", used.body);
    assert_eq!(
        count(&a, "SELECT count(*) FROM totp_backup_codes WHERE used_at IS NULL").await,
        9
    );
    // la seconde fois : refusé
    let c = login_for_challenge(&a, "70 00 00 11", "10.32.3.2").await;
    let twice = verify(&a, c.json()["challenge"].as_str().unwrap(), &codes[0], "10.32.3.2").await;
    assert_eq!(
        (twice.status, twice.json()["code"].clone()),
        (StatusCode::UNAUTHORIZED, json!("auth.totp_invalid"))
    );
    // un autre code passe, sans tirets ni espaces superflus
    let other = codes[1].replace('-', " ");
    let c = login_for_challenge(&a, "70 00 00 11", "10.32.3.3").await;
    assert_eq!(
        verify(&a, c.json()["challenge"].as_str().unwrap(), &other, "10.32.3.3")
            .await
            .status,
        StatusCode::OK
    );
    let log: Vec<(String, String)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT action, after::text FROM audit_log WHERE org_id = '{ORG_A}' AND action = 'auth.mfa_backup_used' ORDER BY id"
    )))
    .fetch_all(&a.db.admin)
    .await
    .unwrap();
    assert_eq!(log.len(), 2);
    assert_eq!(
        serde_json::from_str::<Value>(&log[0].1).unwrap(),
        json!({ "remaining": 9 })
    );
    assert_eq!(
        serde_json::from_str::<Value>(&log[1].1).unwrap(),
        json!({ "remaining": 8 })
    );
    a.db.drop_db().await;
}

// ───────── profil ─────────

#[tokio::test(flavor = "multi_thread")]
async fn changer_de_telephone_exige_la_confirmation_et_un_numero_libre() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 16").await; // Six, comptable de A
    let name = req(
        &a,
        &g,
        Method::PATCH,
        "/api/me",
        Some(json!({ "display_name": "  Six Bis  " })),
        &[],
    )
    .await;
    assert_eq!(
        (name.status, name.json()["display_name"].clone()),
        (StatusCode::OK, json!("Six Bis"))
    );
    let short = req(
        &a,
        &g,
        Method::PATCH,
        "/api/me",
        Some(json!({ "display_name": "x" })),
        &[],
    )
    .await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    let phone = json!({ "login_phone": "70 12 34 99" });
    let r = req(&a, &g, Method::PATCH, "/api/me", Some(phone.clone()), &[]).await;
    assert_eq!(r.json()["code"], "auth.reauth_required");
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM users WHERE id = '{SIX}' AND login_phone = '+22670000016'")
        )
        .await,
        1,
        "aucun effet sans confirmation"
    );
    reauth(&a, &g).await;
    let taken = req(
        &a,
        &g,
        Method::PATCH,
        "/api/me",
        Some(json!({ "login_phone": "70 00 00 11" })),
        &[],
    )
    .await;
    assert_eq!(
        (taken.status, taken.json()["code"].clone()),
        (StatusCode::CONFLICT, json!("auth.identifier_taken"))
    );
    let invalid = req(
        &a,
        &g,
        Method::PATCH,
        "/api/me",
        Some(json!({ "login_phone": "12" })),
        &[],
    )
    .await;
    assert_eq!(invalid.json()["errors"][0]["code"], "validation.phone_format");
    let ok = req(&a, &g, Method::PATCH, "/api/me", Some(phone), &[]).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    // l'ancien numéro cesse de fonctionner, le nouveau marche
    let old = call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        "10.33.0.1",
        &[("content-type", "application/json")],
        Some(&common::login_json("70 00 00 16", GOOD)),
    )
    .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    login(&a.app, "70 12 34 99", GOOD, "10.33.0.2").await;
    // la trace dit « téléphone modifié » sans les numéros
    let rows: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT after::text FROM audit_log WHERE org_id = '{ORG_A}' AND action = 'user.updated' ORDER BY id"
    )))
    .fetch_all(&a.db.admin)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter().all(|r| !r.contains("7000") && !r.contains("1234")),
        "aucun numéro dans l'audit : {rows:?}"
    );
    assert!(rows[1].contains("login_phone"));
    let _ = ORG_B;
    a.db.drop_db().await;
}
