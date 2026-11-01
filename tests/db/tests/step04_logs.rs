//! Test A13 de l'étape 04 (SECURITY.md §7.7, PP-21, F39-observation critère 2) : un parcours complet d'observation (écrans, erreur
//! du front, signalement dont la description contient un numéro et un jeton, événements piégés, erreur serveur, alerte) ne laisse ni
//! la description, ni le numéro, ni le jeton dans les journaux `tracing`, ni dans `product_events`, `error_reports`, `platform_events`.
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
use common::{api_with, fast_params, req, sess};
use immo_app::observation::feedback;
use immo_app::platform::db::Db;
use immo_app::platform::jobs;
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
async fn a13_aucune_description_numero_ni_jeton_dans_les_journaux_ni_dans_les_tables_d_observation() {
    let buf = Buf::default();
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let a = api_with(fast_params(), "0.0.0", false, true).await;
    CAPTURE.store(true, Ordering::SeqCst);

    let phone = "+22670123456";
    let phone_local = "70123456";
    let marker = "SECRET-ABC-777";
    let leaked_token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let description = format!("Le reçu ne s'imprime pas. Rappelez {phone} ({marker}) ; jeton {leaked_token}.");

    let s = sess(&a, "70 00 00 16").await;

    // écrans, erreurs de saisie, événements piégés (nom, numéro, adresse électronique dans les propriétés)
    let events = vec![
        json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "mobile" }, "client_version": "1.0.0" }),
        json!({ "name": "validation_error", "properties": { "screen": "S09", "field": "amount", "rule_code": "R-041" } }),
        json!({ "name": "validation_error", "properties": { "screen": "S09", "field": format!("tel_{phone_local}"), "rule_code": "R-041" } }),
        json!({ "name": format!("achat_{marker}"), "properties": { "tel": phone, "mail": "traore@agence.bf" } }),
    ];
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/events",
        Some(json!({ "events": events })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);

    // erreur du front dont le message cite le numéro, une adresse et un jeton
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/client-errors",
        Some(json!({
            // un mot libre ne se reconnaît pas (limite assumée, DP-91) : le front n'envoie que des messages de son propre code
            "message": format!("TypeError: client {phone} traore@agence.bf {leaked_token}"),
            "screen": "S05",
            "client_version": "1.0.0",
            "request_ids": []
        })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);

    // signalement avec description à risque et adresse portant un numéro
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(json!({
            "category": "bug",
            "description": description,
            "can_call_back": true,
            "context": {
                "screen": "S05",
                "path": format!("/locataires/{phone_local}?tel={phone}&jeton={leaked_token}"),
                "client_version": "1.0.0",
                "viewport": "390x844",
                "last_request_ids": [],
                "last_error_code": null
            }
        })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);

    // erreur serveur réelle
    let r = req(&a, &s, Method::GET, "/api/_test/server-error", None, &[]).await;
    assert_eq!(r.status, StatusCode::INTERNAL_SERVER_ERROR);

    // alerte
    let db = Db::connect(&a.db.app_url, 2).await.unwrap();
    let job = jobs::claim(&db, a.clock.as_ref())
        .await
        .unwrap()
        .expect("tâche de notification");
    feedback::notify(&db, Some(&a.db.platform_url), &job).await.unwrap();

    CAPTURE.store(false, Ordering::SeqCst);
    let logs = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(
        logs.contains("feedback.received"),
        "l'alerte est bien émise dans les journaux"
    );
    assert!(logs.contains("\"route\""), "le journal d'accès tourne");

    let forbidden = [
        phone,
        phone_local,
        marker,
        leaked_token,
        "traore",
        "ne s'imprime pas",
        "Rappelez",
    ];
    for needle in forbidden {
        assert!(
            !logs.contains(needle),
            "journaux tracing : {needle:?} ne doit pas y figurer"
        );
    }
    // les tables d'observation, hors le texte du signalement lui-même (que l'utilisateur écrit pour nous)
    for (table, column) in [
        ("product_events", "properties::text || name"),
        ("error_reports", "coalesce(sample::text, '') || coalesce(route, '')"),
        ("platform_events", "coalesce(detail::text, '') || action"),
        ("feedback_reports", "context::text"),
        ("jobs", "payload::text || coalesce(last_error, '')"),
    ] {
        for needle in forbidden {
            let n = common::count(
                &a,
                &format!(
                    "SELECT count(*) FROM {table} WHERE ({column}) ILIKE '%{}%'",
                    needle.replace('\'', "''")
                ),
            )
            .await;
            assert_eq!(n, 0, "{table} : {needle:?} ne doit pas y figurer");
        }
    }
    // la description, elle, est bien stockée (c'est son seul lieu)
    assert_eq!(
        common::count(
            &a,
            &format!("SELECT count(*) FROM feedback_reports WHERE body LIKE '%{marker}%'")
        )
        .await,
        1
    );
    a.db.drop_db().await;
}
