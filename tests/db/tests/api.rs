//! API d'identité de l'étape 02 (critères 1 à 4 et 9, tests A1, A2, A4, A6, A11, A12 de TESTING_QA.md) : routeur réel, base réelle.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use std::time::Instant;

use axum::http::{Method, StatusCode};
use chrono::Duration;
use common::{GOOD, ORG_A, ORG_B, ORG_C, ORIGIN, api, call, fast_params, get_with, login, login_json};
use immo_app::identity::password::HashParams;
use serde_json::{Value, json};
use sqlx::{Executor, Row};

const JSON: (&str, &str) = ("content-type", "application/json");
const IP: &str = "10.0.0.1";

async fn post_login(a: &common::Api, identifier: &str, password: &str, ip: &str) -> common::Reply {
    call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        ip,
        &[JSON, ("origin", ORIGIN)],
        Some(&login_json(identifier, password)),
    )
    .await
}

fn write_headers<'a>(cookie: &'a str, csrf: &'a str) -> Vec<(&'a str, &'a str)> {
    vec![("cookie", cookie), ("x-csrf-token", csrf), ("origin", ORIGIN), JSON]
}

// ───────── critère 1 : identifiant inconnu et mot de passe faux ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a1_identifiant_inconnu_et_mot_de_passe_faux_donnent_la_meme_reponse_en_un_temps_comparable() {
    // coût argon2id réaliste (4 Mio, 2 itérations) pour que la vérification factice pèse comme une vraie
    let a = api(
        HashParams {
            memory_kib: 4096,
            iterations: 2,
            parallelism: 1,
        },
        "0.0.0",
    )
    .await;
    let mut unknown = vec![];
    let mut wrong = vec![];
    for i in 0..5 {
        // adresses différentes : la limitation par adresse ne s'en mêle pas ; identifiants différents à chaque tour
        let t = Instant::now();
        let u = post_login(&a, &format!("70 99 99 {i:02}"), GOOD, &format!("10.1.0.{i}")).await;
        unknown.push(t.elapsed());
        let t = Instant::now();
        let w = post_login(&a, "70 00 00 11", "mauvais mot de passe", &format!("10.2.0.{i}")).await;
        wrong.push(t.elapsed());
        assert_eq!(u.status, StatusCode::UNAUTHORIZED);
        assert_eq!(w.status, StatusCode::UNAUTHORIZED);
        let (mut uj, mut wj) = (u.json(), w.json());
        for j in [&mut uj, &mut wj] {
            j.as_object_mut().unwrap().remove("request_id");
        }
        assert_eq!(uj, wj, "corps identiques");
        assert_eq!(uj["code"], "auth.invalid_credentials");
        assert_eq!(u.header("content-type"), "application/problem+json");
    }
    unknown.sort();
    wrong.sort();
    let (mu, mw) = (unknown[2].as_secs_f64(), wrong[2].as_secs_f64());
    let ratio = if mu > mw { mu / mw } else { mw / mu };
    assert!(
        ratio < 1.6,
        "durées médianes comparables : inconnu {mu:.4} s, faux {mw:.4} s (rapport {ratio:.2})"
    );
    a.db.drop_db().await;
}

// ───────── critère 2 : 429 avec Retry-After ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a2_apres_cinq_echecs_la_sixieme_tentative_recoit_429_et_retry_after() {
    let a = api(fast_params(), "0.0.0").await;
    for _ in 0..5 {
        assert_eq!(
            post_login(&a, "70 00 00 11", "faux faux faux", IP).await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    let r = post_login(&a, "70 00 00 11", GOOD, IP).await;
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(r.header("retry-after"), "60");
    let j = r.json();
    assert_eq!(j["code"], "auth.rate_limited");
    assert_eq!(j["retry_after_seconds"], 60);
    assert_eq!(
        j["detail"],
        "Trop de tentatives. Réessayez dans 1 minute ou réinitialisez votre mot de passe."
    );
    // après une minute, le bon mot de passe passe
    a.clock.advance(Duration::seconds(61));
    assert_eq!(post_login(&a, "70 00 00 11", GOOD, IP).await.status, StatusCode::OK);
    a.db.drop_db().await;
}

// ───────── critère 3 : cookie ─────────

#[tokio::test(flavor = "multi_thread")]
async fn le_cookie_de_session_a_les_attributs_exiges() {
    let a = api(fast_params(), "0.0.0").await;
    let r = post_login(&a, "70 00 00 11", GOOD, IP).await;
    assert_eq!(r.status, StatusCode::OK);
    let cookie = r.session_cookie().unwrap();
    let parts: Vec<&str> = cookie.split("; ").collect();
    assert!(
        parts[0].starts_with("__Host-session=") && parts[0].len() == "__Host-session=".len() + 64,
        "{cookie}"
    );
    assert_eq!(
        &parts[1..],
        ["Path=/", "Secure", "HttpOnly", "SameSite=Lax", "Max-Age=43200"]
    );
    assert!(!cookie.to_lowercase().contains("domain"), "pas d'attribut Domain");
    // le corps ne répète pas le jeton de session
    assert!(!r.body.contains(parts[0].trim_start_matches("__Host-session=")));
    let j = r.json();
    assert_eq!(j["mfa_required"], false);
    assert_eq!(j["active_organization_id"], ORG_A);
    assert_eq!(j["organizations"][0]["role"], "manager");
    a.db.drop_db().await;
}

// ───────── critère 4 : autre organisation ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a4_demander_l_organisation_d_un_autre_donne_404_sans_donnee_audit_et_alerte() {
    let a = api(fast_params(), "0.0.0").await;
    let (cookie, csrf) = login(&a.app, "70 00 00 11", GOOD, IP).await; // gérant de A
    let r = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        IP,
        &write_headers(&cookie, &csrf),
        Some(&json!({ "organization_id": ORG_B }).to_string()),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert_eq!(r.header("content-type"), "application/problem+json");
    assert_eq!(r.json()["code"], "resource.not_found");
    assert!(
        !r.body.contains("Agence B") && !r.body.contains(ORG_B),
        "aucune donnée de B dans la réponse : {}",
        r.body
    );
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE org_id = $1::uuid AND action = 'security.cross_org_attempt' AND object_id = $2")
        .bind(ORG_A)
        .bind(ORG_B)
        .fetch_one(&a.db.admin)
        .await
        .unwrap();
    assert_eq!(n, 1, "une ligne dans l'audit de A");
    let b: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE org_id = $1::uuid")
        .bind(ORG_B)
        .fetch_one(&a.db.admin)
        .await
        .unwrap();
    assert_eq!(b, 0, "rien chez B");
    // la session reste valide, sur A
    let me = call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.json()["organization"]["id"], ORG_A);
    a.db.drop_db().await;
}

// ───────── jeton anti-falsification et Origin ─────────

#[tokio::test(flavor = "multi_thread")]
async fn les_ecritures_exigent_le_jeton_et_l_origine() {
    let a = api(fast_params(), "0.0.0").await;
    let (cookie, csrf) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    let body = json!({ "organization_id": ORG_A }).to_string();
    let cases: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "jeton absent",
            vec![("cookie", cookie.as_str()), ("origin", ORIGIN), JSON],
        ),
        (
            "jeton faux",
            vec![
                ("cookie", cookie.as_str()),
                ("x-csrf-token", "faux"),
                ("origin", ORIGIN),
                JSON,
            ],
        ),
        (
            "origine absente",
            vec![("cookie", cookie.as_str()), ("x-csrf-token", csrf.as_str()), JSON],
        ),
        (
            "origine étrangère",
            vec![
                ("cookie", cookie.as_str()),
                ("x-csrf-token", csrf.as_str()),
                ("origin", "https://pirate.test"),
                JSON,
            ],
        ),
    ];
    for (why, headers) in cases {
        let r = call(&a.app, Method::POST, "/api/me/organization", IP, &headers, Some(&body)).await;
        assert_eq!(r.status, StatusCode::FORBIDDEN, "{why}");
        assert_eq!(r.json()["code"], "auth.csrf_invalid", "{why}");
    }
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'security.permission_denied'")
        .fetch_one(&a.db.admin)
        .await
        .unwrap();
    assert_eq!(n, 4, "une ligne security.permission_denied par refus");
    // avec le bon jeton et la bonne origine : accepté
    let ok = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        IP,
        &write_headers(&cookie, &csrf),
        Some(&body),
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    // le jeton d'une autre session ne passe pas (le choix précédent a fait tourner la session : on en ouvre une neuve)
    let (cookie2, _) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    let (_, other_csrf) = login(&a.app, "70 00 00 14", GOOD, IP).await;
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/logout-all",
        IP,
        &write_headers(&cookie2, &other_csrf),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    a.db.drop_db().await;
}

// ───────── A6 : pas d'inscription publique ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a6_il_n_existe_aucune_route_d_inscription() {
    let a = api(fast_params(), "0.0.0").await;
    for (method, path) in [
        (Method::POST, "/api/auth/register"),
        (Method::GET, "/api/auth/register"),
        (Method::POST, "/auth/register"),
        (Method::GET, "/api/inconnue"),
    ] {
        let r = call(
            &a.app,
            method.clone(),
            path,
            IP,
            &[JSON, ("origin", ORIGIN)],
            Some("{}"),
        )
        .await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{method} {path}");
        assert_eq!(r.json()["code"], "resource.not_found");
    }
    a.db.drop_db().await;
}

// ───────── critère 9 / A12 : en-têtes sur chaque type de réponse ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a12_chaque_type_de_reponse_porte_les_en_tetes_de_securite() {
    let a = api(fast_params(), "1.0.0").await;
    let (cookie, _) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    let mut replies = vec![
        ("sonde", call(&a.app, Method::GET, "/health", IP, &[], None).await),
        (
            "200 authentifié",
            call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None).await,
        ),
        ("401", call(&a.app, Method::GET, "/api/me", IP, &[], None).await),
        ("404", call(&a.app, Method::GET, "/api/rien", IP, &[], None).await),
        ("login 200", post_login(&a, "70 00 00 11", GOOD, "10.3.0.1").await),
        (
            "login 401",
            post_login(&a, "70 00 00 11", "faux faux faux", "10.3.0.2").await,
        ),
        (
            "422",
            call(
                &a.app,
                Method::POST,
                "/api/auth/login",
                IP,
                &[JSON],
                Some(r#"{"identifier":"x","password":"y","extra":1}"#),
            )
            .await,
        ),
    ];
    for _ in 0..5 {
        post_login(&a, "70 00 00 12", "faux faux faux", "10.4.0.1").await;
    }
    replies.push(("429", post_login(&a, "70 00 00 12", GOOD, "10.4.0.1").await));
    let expected = [
        ("strict-transport-security", "max-age=31536000; includeSubDomains"),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        (
            "permissions-policy",
            "camera=(), microphone=(), geolocation=(), payment=()",
        ),
        ("cross-origin-opener-policy", "same-origin"),
        ("cross-origin-resource-policy", "same-origin"),
        ("x-frame-options", "DENY"),
        ("cache-control", "no-store"),
        ("x-min-client-version", "1.0.0"),
    ];
    for (name, r) in &replies {
        for (h, v) in expected {
            assert_eq!(r.header(h), v, "{name} : {h}");
        }
        let csp = r.header("content-security-policy");
        assert!(
            csp.starts_with("default-src 'self'")
                && csp.contains("frame-ancestors 'none'")
                && csp.contains("object-src 'none'"),
            "{name} : {csp}"
        );
        let id = r.header("x-request-id");
        assert!(id.starts_with("INC-") && id.len() == 10, "{name} : {id}");
    }
    // les erreurs portent le même code d'incident dans l'en-tête et dans le corps
    let r = &replies.iter().find(|(n, _)| *n == "401").unwrap().1;
    assert_eq!(r.json()["request_id"], r.header("x-request-id").as_str());
    a.db.drop_db().await;
}

// ───────── A11 : session, déconnexion ─────────

#[tokio::test(flavor = "multi_thread")]
async fn deconnexion_efface_le_cookie_et_invalide_la_session() {
    let a = api(fast_params(), "0.0.0").await;
    let (cookie, csrf) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    // sans jeton : refusée, la session reste valide
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/logout",
        IP,
        &[("cookie", cookie.as_str()), ("origin", ORIGIN)],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(
        call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None)
            .await
            .status,
        StatusCode::OK
    );
    // avec jeton : 204 et cookie effacé
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/logout",
        IP,
        &write_headers(&cookie, &csrf),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let cleared = r.session_cookie().unwrap();
    assert!(
        cleared.starts_with("__Host-session=;")
            && cleared.contains("Max-Age=0")
            && cleared.contains("Secure")
            && cleared.contains("HttpOnly"),
        "{cleared}"
    );
    let me = call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None).await;
    assert_eq!(me.status, StatusCode::UNAUTHORIZED);
    assert_eq!(me.json()["code"], "auth.session_expired");
    // sans session valide : 204 aussi (idempotent), cookie effacé
    for headers in [vec![], vec![("cookie", cookie.as_str())]] {
        let r = call(&a.app, Method::POST, "/api/auth/logout", IP, &headers, None).await;
        assert_eq!(r.status, StatusCode::NO_CONTENT);
    }
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn deconnexion_de_tous_les_appareils() {
    let a = api(fast_params(), "0.0.0").await;
    let (c1, csrf1) = login(&a.app, "70 00 00 11", GOOD, "10.0.0.1").await;
    let (c2, _) = login(&a.app, "70 00 00 11", GOOD, "10.0.0.2").await;
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/logout-all",
        IP,
        &write_headers(&c1, &csrf1),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    for c in [&c1, &c2] {
        assert_eq!(
            call(&a.app, Method::GET, "/api/me", IP, &get_with(c), None)
                .await
                .status,
            StatusCode::UNAUTHORIZED
        );
    }
    let row = sqlx::query("SELECT after->>'sessions' FROM audit_log WHERE action = 'session.revoked_all'")
        .fetch_one(&a.db.admin)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>(0), "2");
    a.db.drop_db().await;
}

// ───────── /api/me ─────────

#[tokio::test(flavor = "multi_thread")]
async fn me_donne_le_role_les_droits_et_la_date_du_jour_de_l_organisation() {
    let a = api(fast_params(), "0.5.0").await;
    let (cookie, csrf) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    let r = call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None).await;
    let j = r.json();
    assert_eq!(j["user"]["display_name"], "Un");
    assert_eq!(j["membership"]["role"], "manager");
    assert_eq!(j["organization"]["name"], "Agence A");
    assert_eq!(j["organization"]["plan"], "agency");
    assert_eq!(j["organization"]["timezone"], "Africa/Ouagadougou");
    assert_eq!(j["organization"]["currency"], "XOF");
    assert_eq!(j["today"], "2026-11-13");
    assert_eq!(j["min_client_version"], "0.5.0");
    assert_eq!(j["csrf_token"], csrf.as_str());
    assert!(j["server_time"].as_str().unwrap().starts_with("2026-11-13T09:15:00"));
    let perms: Vec<&str> = j["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert_eq!(perms.len(), 25, "le gérant a les 25 actions");
    // collaborateur du plan Propriétaire : aucune action du plan Agence
    let (co, _) = login(&a.app, "70 00 00 14", GOOD, "10.0.0.9").await;
    let jc = call(&a.app, Method::GET, "/api/me", "10.0.0.9", &get_with(&co), None)
        .await
        .json();
    let pc: Vec<&str> = jc["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap())
        .collect();
    assert!(
        !pc.iter()
            .any(|p| p.starts_with("statement.") || *p == "payout.record" || *p == "mandate.write"),
        "{pc:?}"
    );
    assert!(
        pc.contains(&"receipt.issue") && pc.contains(&"payment.request_correction") && !pc.contains(&"payment.record"),
        "{pc:?}"
    );
    assert_eq!(jc["organization"]["id"], ORG_C);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn plusieurs_organisations_le_choix_precede_tout_le_reste() {
    let a = api(fast_params(), "0.0.0").await;
    let r = post_login(&a, "70 00 00 12", GOOD, IP).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(r.json()["active_organization_id"].is_null());
    assert_eq!(r.json()["organizations"].as_array().unwrap().len(), 2);
    let cookie = r.session_cookie().unwrap().split(';').next().unwrap().to_string();
    let csrf = r.json()["csrf_token"].as_str().unwrap().to_string();
    // sans organisation active : le profil répond, une route d'organisation demande le choix
    let me = call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None).await;
    assert_eq!(me.status, StatusCode::OK);
    assert!(me.json()["organization"].is_null() && me.json()["permissions"].as_array().unwrap().is_empty());
    let org = call(&a.app, Method::GET, "/api/organization", IP, &get_with(&cookie), None).await;
    assert_eq!(
        (org.status, org.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.organization_required"))
    );
    // choix : nouveau cookie, l'ancien ne vaut plus
    let sel = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        IP,
        &write_headers(&cookie, &csrf),
        Some(&json!({ "organization_id": ORG_B }).to_string()),
    )
    .await;
    assert_eq!(sel.status, StatusCode::OK, "{}", sel.body);
    let new_cookie = sel.session_cookie().unwrap().split(';').next().unwrap().to_string();
    assert_ne!(new_cookie, cookie);
    assert_eq!(sel.json()["organization"]["name"], "Agence B");
    assert_eq!(
        call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let org = call(
        &a.app,
        Method::GET,
        "/api/organization",
        IP,
        &get_with(&new_cookie),
        None,
    )
    .await;
    assert_eq!(org.status, StatusCode::OK);
    assert_eq!(org.json()["name"], "Agence B");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn organisation_suspendue_et_compte_sans_acces() {
    let a = api(fast_params(), "0.0.0").await;
    let (cookie, _) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE organizations SET lifecycle = 'live', went_live_at = now() WHERE id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE organizations SET lifecycle = 'suspended' WHERE id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    let org = call(&a.app, Method::GET, "/api/organization", IP, &get_with(&cookie), None).await;
    assert_eq!(
        (org.status, org.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("org.suspended"))
    );
    assert_eq!(
        call(&a.app, Method::GET, "/api/me", IP, &get_with(&cookie), None)
            .await
            .status,
        StatusCode::OK,
        "le profil reste lisible"
    );
    // compte sans adhésion : 403 auth.no_active_membership, aucune session
    let r = post_login(&a, "70 00 00 13", GOOD, "10.5.0.1").await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.no_active_membership"))
    );
    assert!(r.session_cookie().is_none());
    a.db.drop_db().await;
}

// ───────── F02 : droits sur une route réelle ─────────

#[tokio::test(flavor = "multi_thread")]
async fn la_fiche_d_organisation_est_lisible_par_tous_les_roles_avec_le_droit_de_lecture() {
    let a = api(fast_params(), "0.0.0").await;
    for (id, who) in [
        ("70 00 00 11", "gérant"),
        ("70 00 00 16", "comptable"),
        ("70 00 00 14", "collaborateur"),
    ] {
        let (cookie, _) = login(&a.app, id, GOOD, "10.6.0.1").await;
        let r = call(
            &a.app,
            Method::GET,
            "/api/organization",
            "10.6.0.1",
            &get_with(&cookie),
            None,
        )
        .await;
        assert_eq!(r.status, StatusCode::OK, "{who} : {}", r.body);
        assert!(r.json()["name"].is_string());
    }
    let (cookie, _) = login(&a.app, "70 00 00 11", GOOD, "10.6.0.2").await;
    let r = call(
        &a.app,
        Method::GET,
        "/api/organization",
        "10.6.0.2",
        &get_with(&cookie),
        None,
    )
    .await;
    assert_eq!(
        (r.json()["name"].clone(), r.json()["city"].clone()),
        (json!("Agence A"), json!("Ouagadougou"))
    );
    a.db.drop_db().await;
}

// ───────── registre des routes ─────────

#[tokio::test(flavor = "multi_thread")]
async fn le_registre_des_routes_correspond_au_contrat_et_chaque_route_protegee_exige_une_session() {
    use immo_server::api::{Access, ROUTES};
    let a = api(fast_params(), "0.0.0").await;
    // 1. le registre et le contrat OpenAPI décrivent les mêmes routes (hors sondes)
    let spec: Value = serde_json::from_str(&immo_server::openapi::openapi_json()).unwrap();
    let mut in_contract: Vec<String> = spec["paths"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(p, _)| p.starts_with("/api/"))
        .flat_map(|(p, item)| {
            item.as_object()
                .unwrap()
                .keys()
                .map(move |m| format!("{} {p}", m.to_uppercase()))
        })
        .collect();
    let mut in_registry: Vec<String> = ROUTES.iter().map(|r| format!("{} {}", r.method, r.path)).collect();
    in_contract.sort();
    in_registry.sort();
    assert_eq!(in_registry, in_contract, "toute route est au registre ET au contrat");
    // 2. sans cookie, toute route non publique répond 401 ; aucune ne répond 200 ou 5xx
    for r in ROUTES {
        let method = Method::from_bytes(r.method.as_bytes()).unwrap();
        let reply = call(&a.app, method, r.path, IP, &[JSON, ("origin", ORIGIN)], Some("{}")).await;
        match r.access {
            Access::Public => assert!(
                reply.status != StatusCode::INTERNAL_SERVER_ERROR,
                "{} {}",
                r.method,
                r.path
            ),
            Access::Session | Access::Action(_) => {
                assert_eq!(
                    reply.status,
                    StatusCode::UNAUTHORIZED,
                    "{} {} sans session",
                    r.method,
                    r.path
                );
                assert_eq!(reply.json()["code"], "auth.unauthenticated");
            }
        }
    }
    // 3. une route publique ne l'est que si la liste fermée de API_CONTRACTS.md §4.1 la cite
    let public: Vec<&str> = ROUTES
        .iter()
        .filter(|r| r.access == Access::Public)
        .map(|r| r.path)
        .collect();
    assert_eq!(public, ["/api/auth/login", "/api/auth/logout"]);
    a.db.drop_db().await;
}

// ───────── corps de requête ─────────

#[tokio::test(flavor = "multi_thread")]
async fn les_corps_sont_stricts_et_bornes() {
    let a = api(fast_params(), "0.0.0").await;
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        IP,
        &[JSON],
        Some(r#"{"identifier":"70123456","password":"x","admin":true}"#),
    )
    .await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("request.unknown_field"))
    );
    assert!(!r.body.contains("admin"), "le nom du champ fautif n'est pas renvoyé");
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        IP,
        &[JSON],
        Some("{pas du json"),
    )
    .await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::BAD_REQUEST, json!("request.malformed"))
    );
    let r = call(
        &a.app,
        Method::POST,
        "/api/auth/login",
        IP,
        &[],
        Some(r#"{"identifier":"a","password":"b"}"#),
    )
    .await;
    assert_eq!(r.json()["code"], "request.malformed", "type de contenu absent");
    let big = format!(r#"{{"identifier":"{}","password":"x"}}"#, "a".repeat(20_000));
    let r = call(&a.app, Method::POST, "/api/auth/login", IP, &[JSON], Some(&big)).await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::PAYLOAD_TOO_LARGE, json!("request.too_large"))
    );
    // un mot de passe de 129 caractères est refusé comme les autres
    let r = post_login(&a, "70 00 00 11", &"x".repeat(129), IP).await;
    assert_eq!(r.json()["code"], "auth.invalid_credentials");
    a.db.drop_db().await;
}

// ───────── plafonds généraux, version du client ─────────

#[tokio::test(flavor = "multi_thread")]
async fn le_plafond_par_session_est_de_300_requetes_par_minute() {
    let a = api(fast_params(), "0.0.0").await;
    let (cookie, _) = login(&a.app, "70 00 00 11", GOOD, IP).await;
    let mut last = None;
    for i in 0..305 {
        let r = call(
            &a.app,
            Method::GET,
            "/api/me",
            &format!("10.7.{}.{}", i / 250, i % 250),
            &get_with(&cookie),
            None,
        )
        .await;
        if r.status == StatusCode::TOO_MANY_REQUESTS {
            last = Some((i, r));
            break;
        }
    }
    let (at, r) = last.expect("le plafond doit être atteint avant 305 appels");
    assert_eq!(
        at, 300,
        "les 300 premiers appels de la session passent, le 301ᵉ est refusé"
    );
    assert_eq!(r.json()["code"], "rate_limit.exceeded");
    assert!(r.header("retry-after").parse::<i64>().unwrap() >= 1);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_client_trop_ancien_recoit_426() {
    let a = api(fast_params(), "1.2.0").await;
    let r = call(
        &a.app,
        Method::GET,
        "/api/me",
        IP,
        &[("x-client-version", "1.1.9")],
        None,
    )
    .await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::UPGRADE_REQUIRED, json!("client.upgrade_required"))
    );
    let r = call(
        &a.app,
        Method::GET,
        "/api/me",
        IP,
        &[("x-client-version", "1.2.0")],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED, "à jour : la requête suit son cours");
    let r = call(&a.app, Method::GET, "/api/me", IP, &[], None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED, "sans en-tête : pas de blocage");
    a.db.drop_db().await;
}
