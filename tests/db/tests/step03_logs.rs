//! Test A13 de l'étape 03 (SECURITY.md §7.7, PP-21, F03 §8.4) : un parcours complet (invitation, acceptation, confirmation du mot de
//! passe, changement de rôle, réinitialisation, double authentification, code de secours, accord support, lecture du journal) ne
//! laisse aucun jeton, mot de passe, code, clé, défi ni numéro dans les journaux `tracing`, ni dans `audit_log`.
//!
//! Dans son propre fichier : l'abonné `tracing` est global et le test est seul de son binaire.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use axum::http::{Method, StatusCode};
use chrono::Duration;
use common::{ORG_A, ORIGIN, S, api, call, fast_params, reauth, req, sess, token_of};
use immo_app::identity::crypto::FieldKey;
use immo_app::identity::totp;
use immo_app::platform::clock::Clock;
use serde_json::json;

static CAPTURE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl io::Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        if CAPTURE.load(Ordering::SeqCst) {
            self.0.lock().unwrap().extend_from_slice(b);
        }
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

const JSON: (&str, &str) = ("content-type", "application/json");

#[tokio::test(flavor = "multi_thread")]
async fn a13_aucun_secret_dans_les_journaux_d_un_parcours_complet_de_l_etape_03() {
    let buf = Buf::default();
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let a = api(fast_params(), "0.0.0").await;
    CAPTURE.store(true, Ordering::SeqCst);
    let ip = "10.40.0.1";
    let mut secrets: Vec<(&'static str, String)> = vec![];

    // ── invitation d'un caissier, acceptation avec un premier mot de passe ──
    let g = sess(&a, "70 00 00 11").await;
    let invite = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 12 34 56", "role": "cashier" })),
        &[],
    )
    .await;
    let invitation_token = token_of(invite.json()["url"].as_str().unwrap());
    secrets.push(("jeton d'invitation", invitation_token.clone()));
    let first_password = "Mot-de-passe-initial-A13";
    secrets.push(("premier mot de passe", first_password.to_string()));
    let accepted = call(
        &a.app,
        Method::POST,
        "/api/invitations/accept",
        ip,
        &[JSON],
        Some(&json!({ "token": invitation_token, "display_name": "Awa Test", "password": first_password }).to_string()),
    )
    .await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    secrets.push((
        "jeton de session de l'invité",
        accepted
            .session_cookie()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .trim_start_matches("__Host-session=")
            .to_string(),
    ));

    // ── changement de rôle (confirmation du mot de passe) et lien de réinitialisation ──
    reauth(&a, &g).await;
    let members = req(&a, &g, Method::GET, "/api/users", None, &[]).await;
    let awa = members.json()["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["display_name"] == "Awa Test")
        .unwrap()
        .clone();
    let mid = awa["membership_id"].as_str().unwrap().to_string();
    req(
        &a,
        &g,
        Method::PATCH,
        &format!("/api/users/{mid}"),
        Some(json!({ "role": "property_manager" })),
        &[],
    )
    .await;
    let reset = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{mid}/password-resets"),
        None,
        &[],
    )
    .await;
    let reset_token = token_of(reset.json()["url"].as_str().unwrap());
    secrets.push(("jeton de réinitialisation", reset_token.clone()));
    let second_password = "Mot-de-passe-reinitialise-A13";
    secrets.push(("mot de passe réinitialisé", second_password.to_string()));
    let done = call(
        &a.app,
        Method::POST,
        "/api/auth/password-reset/complete",
        ip,
        &[JSON],
        Some(&json!({ "token": reset_token, "new_password": second_password }).to_string()),
    )
    .await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{}", done.body);

    // ── double authentification du gérant, connexion avec le code puis avec un code de secours ──
    let begin = req(&a, &g, Method::POST, "/api/me/totp", None, &[]).await;
    let secret_b32 = begin.json()["secret_base32"].as_str().unwrap().to_string();
    secrets.push(("clé TOTP en base32", secret_b32.clone()));
    let blob: Vec<u8> =
        sqlx::query_scalar("SELECT totp_secret_enc FROM users WHERE id = '11111111-0000-4000-8000-000000000011'")
            .fetch_one(&a.db.admin)
            .await
            .unwrap();
    let secret = FieldKey::from_bytes(&[9_u8; 32]).unwrap().decrypt(&blob).unwrap();
    let step_code = |delta: i64| {
        format!(
            "{:06}",
            totp::code_for_step(&secret, u64::try_from(totp::step_at(a.clock.now()) + delta).unwrap())
        )
    };
    let activation_code = step_code(0);
    let confirm = req(
        &a,
        &g,
        Method::POST,
        "/api/me/totp/confirm",
        Some(json!({ "code": activation_code })),
        &[],
    )
    .await;
    assert_eq!(confirm.status, StatusCode::OK, "{}", confirm.body);
    let backup: Vec<String> = confirm.json()["backup_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    for code in &backup {
        secrets.push(("code de secours", code.clone()));
    }
    a.clock.advance(Duration::seconds(60));
    let login = |ident: &str| {
        let app = a.app.clone();
        let body = common::login_json(ident, common::GOOD);
        async move { call(&app, Method::POST, "/api/auth/login", ip, &[JSON], Some(&body)).await }
    };
    let first = login("70 00 00 11").await;
    let challenge = first.json()["challenge"].as_str().unwrap().to_string();
    secrets.push(("défi TOTP", challenge.clone()));
    let code = step_code(0);
    secrets.push(("code TOTP", code.clone()));
    let session = call(
        &a.app,
        Method::POST,
        "/api/auth/totp/verify",
        ip,
        &[JSON],
        Some(&json!({ "challenge": challenge, "code": code }).to_string()),
    )
    .await;
    assert_eq!(session.status, StatusCode::OK, "{}", session.body);
    let second = login("70 00 00 11").await;
    let backup_used = call(
        &a.app,
        Method::POST,
        "/api/auth/totp/verify",
        ip,
        &[JSON],
        Some(&json!({ "challenge": second.json()["challenge"], "code": backup[0] }).to_string()),
    )
    .await;
    assert_eq!(backup_used.status, StatusCode::OK, "{}", backup_used.body);
    let pair = backup_used
        .session_cookie()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let g2 = S {
        cookie: pair.clone(),
        csrf: backup_used.json()["csrf_token"].as_str().unwrap().to_string(),
    };
    secrets.push((
        "jeton de session après second facteur",
        pair.trim_start_matches("__Host-session=").to_string(),
    ));
    secrets.push(("jeton anti-falsification", g2.csrf.clone()));

    // ── accord support, lecture du journal ──
    reauth(&a, &g2).await;
    let grant = req(
        &a,
        &g2,
        Method::POST,
        "/api/support-grants",
        Some(json!({ "scope": "read", "reason": "Aide au démarrage A13", "duration_hours": 24 })),
        &[],
    )
    .await;
    assert_eq!(grant.status, StatusCode::OK, "{}", grant.body);
    let audit = req(&a, &g2, Method::GET, "/api/audit?limit=100", None, &[]).await;
    assert_eq!(audit.status, StatusCode::OK);
    CAPTURE.store(false, Ordering::SeqCst);

    // ── 1. journaux tracing ──
    let log = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        log.matches("\"requête\"").count() >= 15,
        "le journal d'accès existe : {} octets",
        log.len()
    );
    let mut forbidden: Vec<(&str, String)> = secrets.clone();
    forbidden.extend([
        ("numéro de l'invité", "+22670123456".to_string()),
        ("numéro de l'invité (chiffres)", "70123456".to_string()),
        ("numéro du gérant (chiffres)", "70000011".to_string()),
        ("indicatif", "+226".to_string()),
        ("mot de passe de connexion", common::GOOD.to_string()),
        ("empreinte de mot de passe", "$argon2id".to_string()),
        ("nom du cookie avec valeur", "__Host-session=".to_string()),
    ]);
    for (what, secret) in &forbidden {
        assert!(
            !log.contains(secret.as_str()),
            "{what} présent dans les journaux tracing : « {secret} »"
        );
    }

    // ── 2. journal d'audit : valeurs avant/après et identifiants d'objet ──
    let audit_text: String = sqlx::query_scalar(
        "SELECT coalesce(string_agg(coalesce(before::text, '') || ' ' || coalesce(after::text, '') || ' ' || object_id || ' ' || action, ' '), '') FROM audit_log",
    )
    .fetch_one(&a.db.admin)
    .await
    .unwrap();
    assert!(
        audit_text.contains("support.access_granted"),
        "le journal contient bien le parcours"
    );
    for (what, secret) in &forbidden {
        if *what == "indicatif" || *what == "nom du cookie avec valeur" || *what == "empreinte de mot de passe" {
            continue;
        }
        assert!(
            !audit_text.contains(secret.as_str()),
            "{what} présent dans audit_log : « {secret} »"
        );
    }
    // la réponse de lecture du journal ne contient rien non plus
    for (what, secret) in &secrets {
        assert!(
            !audit.body.contains(secret.as_str()) || *what == "jeton anti-falsification" && false,
            "{what} dans la réponse du journal"
        );
    }
    let _ = (ORG_A, ORIGIN);
    a.db.drop_db().await;
}
