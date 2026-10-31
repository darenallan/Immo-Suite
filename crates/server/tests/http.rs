//! Tests HTTP de l'étape 01 : /health, /ready (réseau interne), journal à champs en liste blanche, fraîcheur du contrat OpenAPI.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use immo_server::http::{AppState, router};
use immo_server::probe::Probe;
use tower::ServiceExt;

struct Fake(bool);
impl Probe for Fake {
    fn name(&self) -> &'static str {
        "fake"
    }
    fn check(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { self.0 })
    }
}

fn app(probes: Vec<bool>) -> axum::Router {
    router(Arc::new(AppState {
        probes: probes
            .into_iter()
            .map(|ok| Arc::new(Fake(ok)) as Arc<dyn Probe>)
            .collect(),
        version: "test".into(),
    }))
}

async fn call(
    app: axum::Router,
    path: &str,
    peer: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let mut builder = Request::builder().uri(path);
    for (k, v) in headers {
        builder = builder.header(*k, *v);
    }
    let mut req = builder.body(Body::from(body.to_string())).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(peer.parse::<SocketAddr>().unwrap()));
    let resp = app.oneshot(req).await.unwrap();
    let (parts, body) = resp.into_parts();
    (
        parts.status,
        parts.headers,
        String::from_utf8(body.collect().await.unwrap().to_bytes().to_vec()).unwrap(),
    )
}

#[tokio::test]
async fn health_repond_ok_et_rien_d_autre() {
    let (status, headers, body) = call(app(vec![]), "/health", "203.0.113.9:5555", &[], "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok"}"#, "aucune donnée dans la réponse");
    assert_eq!(headers.get("cache-control").unwrap(), "no-store");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(
        headers.get("x-request-id").unwrap().to_str().unwrap().len(),
        36,
        "identifiant de requête (UUID)"
    );
}

#[tokio::test]
async fn health_est_joignable_depuis_l_exterieur() {
    let (status, ..) = call(app(vec![false]), "/health", "203.0.113.9:5555", &[], "").await;
    assert_eq!(status, StatusCode::OK, "la sonde externe ne dépend d'aucune ressource");
}

#[tokio::test]
async fn ready_depuis_le_reseau_interne() {
    for peer in [
        "127.0.0.1:1",
        "10.0.0.5:1",
        "192.168.1.8:1",
        "172.16.0.3:1",
        "[::1]:1",
        "[fd00::1]:1",
    ] {
        let (status, _, body) = call(app(vec![true, true]), "/ready", peer, &[], "").await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::OK, r#"{"status":"ready"}"#),
            "{peer}"
        );
    }
}

#[tokio::test]
async fn ready_503_si_une_ressource_manque_sans_dire_laquelle() {
    let (status, _, body) = call(app(vec![true, false]), "/ready", "127.0.0.1:1", &[], "").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, r#"{"status":"unavailable"}"#);
}

#[tokio::test]
async fn ready_est_invisible_depuis_l_exterieur() {
    for peer in ["203.0.113.9:1", "8.8.8.8:1", "[2001:db8::1]:1"] {
        let (status, ..) = call(app(vec![true]), "/ready", peer, &[], "").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{peer}");
    }
    // une requête relayée par le proxy (adresse interne, mais X-Forwarded-For présent) est traitée comme externe
    for h in [("x-forwarded-for", "203.0.113.9"), ("forwarded", "for=203.0.113.9")] {
        let (status, ..) = call(app(vec![true]), "/ready", "10.0.0.5:1", &[h], "").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{h:?}");
    }
}

#[tokio::test]
async fn route_inconnue_404() {
    let (status, ..) = call(app(vec![]), "/inconnue", "127.0.0.1:1", &[], "").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ───────── contrat ─────────

#[test]
fn le_contrat_openapi_versionne_est_a_jour() {
    let committed = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi.json"))
        .expect("openapi.json à la racine");
    assert_eq!(
        committed,
        immo_server::openapi::openapi_json(),
        "régénérer : immo openapi > openapi.json"
    );
}
