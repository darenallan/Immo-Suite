//! Test A13 (SECURITY.md §7.7, PP-21) : un parcours de connexion complet ne laisse aucun numéro `+226`, mot de passe, jeton, e-mail
//! ni empreinte dans les journaux, à tous les niveaux.
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

use axum::http::Method;
use chrono::Duration;
use common::{GOOD, ORG_B, ORIGIN, api, call, fast_params, login_json};
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

#[tokio::test(flavor = "multi_thread")]
async fn a13_aucun_secret_ni_numero_dans_les_journaux_d_un_parcours_complet() {
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
    let ip = "10.0.0.1";
    let hdr = |extra: &[(&'static str, String)]| -> Vec<(&'static str, String)> {
        let mut v = vec![
            ("content-type", "application/json".to_string()),
            ("origin", ORIGIN.to_string()),
        ];
        v.extend_from_slice(extra);
        v
    };
    let send = |method: Method, path: &'static str, headers: Vec<(&'static str, String)>, body: Option<String>| {
        let app = a.app.clone();
        async move {
            let refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
            call(&app, method, path, ip, &refs, body.as_deref()).await
        }
    };

    // connexion par téléphone, par e-mail, échecs (bon et mauvais identifiants), blocage, reprise
    let wrong = "mot de passe faux secret-xyz";
    for _ in 0..5 {
        send(
            Method::POST,
            "/api/auth/login",
            hdr(&[]),
            Some(login_json("70 00 00 11", wrong)),
        )
        .await;
    }
    send(
        Method::POST,
        "/api/auth/login",
        hdr(&[]),
        Some(login_json("70 00 00 11", GOOD)),
    )
    .await; // 429
    a.clock.advance(Duration::seconds(61));
    let ok = send(
        Method::POST,
        "/api/auth/login",
        hdr(&[]),
        Some(login_json("70 00 00 11", GOOD)),
    )
    .await;
    let cookie = ok.session_cookie().unwrap().split(';').next().unwrap().to_string();
    let session_token = cookie.trim_start_matches("__Host-session=").to_string();
    let csrf = ok.json()["csrf_token"].as_str().unwrap().to_string();
    let email_ok = send(
        Method::POST,
        "/api/auth/login",
        hdr(&[]),
        Some(login_json("Un.Gerant@Agence.BF", GOOD)),
    )
    .await;
    let email_cookie = email_ok
        .session_cookie()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    // profil, organisation, autre organisation (tentative inter-organisations), refus de jeton, déconnexions
    send(Method::GET, "/api/me", vec![("cookie", cookie.clone())], None).await;
    send(Method::GET, "/api/organization", vec![("cookie", cookie.clone())], None).await;
    let writes = hdr(&[("cookie", cookie.clone()), ("x-csrf-token", csrf.clone())]);
    send(
        Method::POST,
        "/api/me/organization",
        writes.clone(),
        Some(json!({ "organization_id": ORG_B }).to_string()),
    )
    .await;
    send(
        Method::POST,
        "/api/me/organization",
        hdr(&[("cookie", cookie.clone())]),
        Some(json!({ "organization_id": ORG_B }).to_string()),
    )
    .await; // sans jeton
    send(Method::POST, "/api/auth/logout-all", writes, None).await;
    send(Method::GET, "/api/me", vec![("cookie", cookie.clone())], None).await; // session révoquée
    CAPTURE.store(false, Ordering::SeqCst);

    let log = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        log.matches("\"requête\"").count() >= 10,
        "le journal d'accès existe : {} octets",
        log.len()
    );
    let email_session = email_cookie.trim_start_matches("__Host-session=");
    for (what, secret) in [
        ("numéro de téléphone (forme normalisée)", "+22670000011"),
        ("numéro de téléphone (chiffres)", "70000011"),
        ("indicatif", "+226"),
        ("e-mail", "agence.bf"),
        ("e-mail (partie locale)", "un.gerant"),
        ("mot de passe juste", GOOD),
        ("mot de passe faux", "secret-xyz"),
        ("jeton de session", session_token.as_str()),
        ("jeton de session (e-mail)", email_session),
        ("jeton anti-falsification", csrf.as_str()),
        ("empreinte de mot de passe", "$argon2id"),
        ("nom du cookie avec valeur", "__Host-session="),
    ] {
        assert!(!log.contains(secret), "{what} présent dans les journaux : « {secret} »");
    }
    a.db.drop_db().await;
}
