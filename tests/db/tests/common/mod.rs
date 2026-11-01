//! Outils communs aux tests d'API : jeu de données d'identité, routeur réel, appels HTTP sans réseau.

#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use http_body_util::BodyExt;
use immo_app::identity::password::{HashParams, PasswordService};
use immo_app::identity::service::IdentityService;
use immo_app::platform::db::Db;
use immo_db_tests::{TestClock, TestDb};
use immo_server::http::{AppState, router};
use serde_json::{Value, json};
use sqlx::Executor;

pub const ORG_A: &str = "a0000000-0000-4000-8000-00000000000a";
pub const ORG_B: &str = "b0000000-0000-4000-8000-00000000000b";
pub const ORG_C: &str = "c0000000-0000-4000-8000-00000000000c";
pub const GOOD: &str = "un mot de passe correct";
pub const SECRET: &[u8] = b"secret de session de test, au moins trente-deux octets";
pub const ORIGIN: &str = "https://app.test";

pub struct Api {
    pub db: TestDb,
    pub app: Router,
    pub svc: IdentityService,
    pub clock: Arc<TestClock>,
}

/// Mêmes comptes que `login.rs` : « Un » (+22670000011) gérant de A ; « Deux » (+22670000012) caissier de A et de B ;
/// « Trois » (+22670000013) sans accès ; « Quatre » (+22670000014) collaborateur de C (plan Propriétaire) ;
/// « Cinq » (un.gerant@agence.bf) gérant de B ; « Six » (+22670000016) comptable de A.
pub async fn api(params: HashParams, min_client_version: &str) -> Api {
    api_with(params, min_client_version, false, false).await
}

/// Comme `api`, avec les réglages d'observation : `strict` (événement écarté = 422, hors production) et `test_routes`
/// (route d'essai qui provoque une erreur 500).
pub async fn api_with(params: HashParams, min_client_version: &str, strict: bool, test_routes: bool) -> Api {
    let db = TestDb::create().await;
    let passwords = PasswordService::new(params).unwrap();
    let hash = passwords.hash(GOOD).await.unwrap();
    db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO organizations (id, name, plan, kind) VALUES ('{ORG_A}', 'Agence A', 'agency', 'real'), ('{ORG_B}', 'Agence B', 'agency', 'real'), ('{ORG_C}', 'Famille C', 'owner', 'real');
             UPDATE organizations SET city = 'Ouagadougou' WHERE id = '{ORG_A}';
             INSERT INTO users (id, login_phone, login_email, display_name, password_hash) VALUES
               ('11111111-0000-4000-8000-000000000011', '+22670000011', NULL, 'Un', '{hash}'),
               ('22222222-0000-4000-8000-000000000012', '+22670000012', NULL, 'Deux', '{hash}'),
               ('33333333-0000-4000-8000-000000000013', '+22670000013', NULL, 'Trois', '{hash}'),
               ('44444444-0000-4000-8000-000000000014', '+22670000014', NULL, 'Quatre', '{hash}'),
               ('55555555-0000-4000-8000-000000000015', NULL, 'un.gerant@agence.bf', 'Cinq', '{hash}'),
               ('66666666-0000-4000-8000-000000000016', '+22670000016', NULL, 'Six', '{hash}');
             INSERT INTO memberships (org_id, user_id, role) VALUES
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'manager'),
               ('{ORG_A}', '22222222-0000-4000-8000-000000000012', 'cashier'),
               ('{ORG_B}', '22222222-0000-4000-8000-000000000012', 'cashier'),
               ('{ORG_C}', '44444444-0000-4000-8000-000000000014', 'collaborator'),
               ('{ORG_B}', '55555555-0000-4000-8000-000000000015', 'manager'),
               ('{ORG_A}', '66666666-0000-4000-8000-000000000016', 'accountant');"
        )))
        .await
        .unwrap();
    let app_db = Db::connect(&db.app_url, 6).await.unwrap();
    let clock = Arc::new(TestClock::new());
    let svc = IdentityService::new(app_db, passwords, clock.clone(), SECRET.to_vec()).with_field_key(Some(
        immo_app::identity::crypto::FieldKey::from_bytes(&[9_u8; 32]).unwrap(),
    ));
    let state = AppState::new(vec![], "test".to_string())
        .with_identity(
            svc.clone(),
            vec![ORIGIN.to_string()],
            false,
            min_client_version.to_string(),
        )
        .with_public_base_url("https://app.test")
        .with_strict_events(strict)
        .with_test_routes(test_routes);
    Api {
        db,
        app: router(Arc::new(state)),
        svc,
        clock,
    }
}

pub fn fast_params() -> HashParams {
    HashParams {
        memory_kib: 64,
        iterations: 1,
        parallelism: 1,
    }
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }

    pub fn header(&self, name: &str) -> String {
        self.headers
            .get(name)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default()
    }

    /// Valeur du cookie de session posé par la réponse (`__Host-session=…`).
    pub fn session_cookie(&self) -> Option<String> {
        self.headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|c| c.starts_with("__Host-session="))
            .map(str::to_string)
    }
}

/// Un appel au routeur réel, avec adresse de connexion simulée.
pub async fn call(
    app: &Router,
    method: Method,
    path: &str,
    ip: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Reply {
    let mut builder = Request::builder().method(method).uri(path);
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let mut request = builder.body(Body::from(body.unwrap_or("").to_string())).unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(ip.parse().unwrap(), 40000)));
    let response = tower::ServiceExt::oneshot(app.clone(), request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = body.collect().await.unwrap().to_bytes();
    Reply {
        status: parts.status,
        headers: parts.headers,
        body: String::from_utf8_lossy(&bytes).to_string(),
    }
}

pub fn login_json(identifier: &str, password: &str) -> String {
    serde_json::json!({ "identifier": identifier, "password": password }).to_string()
}

/// Connexion réussie : renvoie (valeur du cookie « __Host-session=… » sans attributs, jeton anti-falsification).
pub async fn login(app: &Router, identifier: &str, password: &str, ip: &str) -> (String, String) {
    let r = call(
        app,
        Method::POST,
        "/api/auth/login",
        ip,
        &[("content-type", "application/json"), ("origin", ORIGIN)],
        Some(&login_json(identifier, password)),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let cookie = r.session_cookie().expect("cookie de session");
    let pair = cookie.split(';').next().unwrap().to_string();
    (pair, r.json()["csrf_token"].as_str().unwrap().to_string())
}

pub fn get_with(cookie: &str) -> Vec<(&str, &str)> {
    vec![("cookie", cookie)]
}

// ───────── session de test et outils de l'étape 03 ─────────

pub const IP: &str = "10.30.0.1";

pub struct S {
    pub cookie: String,
    pub csrf: String,
}

pub async fn sess(a: &Api, identifier: &str) -> S {
    let (cookie, csrf) = login(&a.app, identifier, GOOD, IP).await;
    S { cookie, csrf }
}

pub async fn req(a: &Api, s: &S, method: Method, path: &str, body: Option<Value>, extra: &[(&str, &str)]) -> Reply {
    let mut headers: Vec<(&str, &str)> = vec![
        ("cookie", &s.cookie),
        ("x-csrf-token", &s.csrf),
        ("origin", ORIGIN),
        ("content-type", "application/json"),
    ];
    headers.extend_from_slice(extra);
    let text = body.map(|b| b.to_string());
    call(&a.app, method, path, IP, &headers, text.as_deref()).await
}

pub async fn reauth(a: &Api, s: &S) {
    let r = req(
        a,
        s,
        Method::POST,
        "/api/auth/reauth",
        Some(json!({ "password": GOOD })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
}

pub async fn actions(a: &Api, org: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(format!(
        "SELECT action FROM audit_log WHERE org_id = '{org}' ORDER BY id"
    )))
    .fetch_all(&a.db.admin)
    .await
    .unwrap()
}

pub async fn count(a: &Api, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(sql.to_string()))
        .fetch_one(&a.db.admin)
        .await
        .unwrap()
}

/// Jeton d'un lien « https://app.test/invitation/<jeton> ».
pub fn token_of(url: &str) -> String {
    url.rsplit('/').next().unwrap().to_string()
}
