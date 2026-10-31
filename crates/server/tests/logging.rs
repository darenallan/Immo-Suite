//! Test du journal d'accès : champs en liste blanche, aucune donnée sensible (CODING_RULES.md §7 n° 13, SECURITY.md §7.7).
//!
//! Dans son propre fichier : `tracing` met en cache l'intérêt des événements ; les autres tests du serveur émettent avant qu'un
//! abonné existe, ce qui rendrait ce test dépendant de l'ordre d'exécution.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

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

async fn call(app: axum::Router, path: &str, peer: &str, headers: &[(&str, &str)], body: &str) -> (StatusCode, String) {
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
        String::from_utf8(body.collect().await.unwrap().to_bytes().to_vec()).unwrap(),
    )
}

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);
impl io::Write for Buf {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn le_journal_n_ecrit_que_des_champs_de_la_liste_blanche() {
    let buf = Buf::default();
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(move || writer.clone())
        .with_current_span(false)
        .with_span_list(false)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    // requêtes qui portent des données sensibles : corps, jeton, chaîne de requête, identifiant dans le chemin
    call(
        app(vec![true]),
        "/health?token=SECRET-JETON-123",
        "127.0.0.1:1",
        &[("authorization", "Bearer SECRET-BEARER")],
        "corps-secret-0612345678",
    )
    .await;
    call(
        app(vec![true]),
        "/inconnue/42/jeton-abc",
        "127.0.0.1:1",
        &[("cookie", "session=SECRET-COOKIE")],
        "+226 70 12 34 56",
    )
    .await;

    let text = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    let lines: Vec<serde_json::Value> = text.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 2, "une ligne par requête : {text}");
    let allowed = [
        "request_id",
        "method",
        "route",
        "status",
        "duration_ms",
        "version",
        "message",
    ];
    for l in &lines {
        for key in l["fields"].as_object().unwrap().keys() {
            assert!(allowed.contains(&key.as_str()), "champ hors liste blanche : {key}");
        }
        for key in l.as_object().unwrap().keys() {
            assert!(
                ["timestamp", "level", "fields", "target"].contains(&key.as_str()),
                "clé de niveau supérieur inattendue : {key}"
            );
        }
    }
    assert_eq!(lines[0]["fields"]["route"], "/health");
    assert_eq!(
        lines[1]["fields"]["route"], "unmatched",
        "le chemin brut n'est jamais journalisé"
    );
    for secret in [
        "SECRET",
        "corps-secret",
        "0612345678",
        "70 12 34 56",
        "jeton-abc",
        "/inconnue/42",
    ] {
        assert!(!text.contains(secret), "donnée sensible dans le journal : {secret}");
    }
}
