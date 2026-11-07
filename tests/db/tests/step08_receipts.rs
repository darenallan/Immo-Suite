//! Étape 08 : quittance PDF, envois et liens de partage (F10, DP-98). Critères 1 à 7 de `roadmap/etape-08-…` §6 au niveau de
//! l'API et de la base : rendu en file et idempotent, échec de rendu sans effet sur le paiement (R-049), PDF en flux,
//! message WhatsApp minimal, envois consignés (R-055), page publique (E-12, A12), lecture par empreinte de jeton (ISO-9).
//! Le moteur Typst réel est testé à part (`crates/pdf/tests/render.rs`) ; ici un moteur factice et déterministe.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use chrono::TimeZone;
use common::{ORG_A, ORG_B, S, api, count, fast_params, req, sess};
use immo_app::delivery;
use immo_app::platform::clock::Clock;
use immo_app::platform::jobs::Job;
use immo_app::platform::pdf::{PaperFormat, PdfError, PdfRenderer, RenderRequest};
use serde_json::{Value, json};
use sqlx::Executor;

/// Moteur factice : un PDF minimal qui dépend du format, ou un échec.
struct Fake {
    fail: bool,
}

impl PdfRenderer for Fake {
    fn render(&self, request: &RenderRequest) -> Result<Vec<u8>, PdfError> {
        if self.fail {
            return Err(PdfError::RenderFailed);
        }
        let paper = match request.format {
            PaperFormat::A4 => "A4",
            PaperFormat::A5 => "A5",
        };
        Ok(format!("%PDF-1.7 factice {paper}").into_bytes())
    }

    fn version(&self) -> String {
        "factice-1".to_string()
    }
}

async fn goto(a: &common::Api, y: i32, m: u32, d: u32) -> S {
    let target = chrono::Utc.with_ymd_and_hms(y, m, d, 9, 0, 0).unwrap();
    let now = a.clock.now();
    a.clock.advance(target - now);
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "CREATE OR REPLACE FUNCTION app_now() RETURNS timestamptz LANGUAGE sql STABLE AS $$ SELECT timestamptz '{}' $$",
            target.to_rfc3339()
        )))
        .await
        .unwrap();
    sess(a, "70 00 00 11").await
}

async fn post(a: &common::Api, s: &S, path: &str, body: Value) -> common::Reply {
    req(a, s, Method::POST, path, Some(body), &[]).await
}

async fn get(a: &common::Api, s: &S, path: &str) -> common::Reply {
    req(a, s, Method::GET, path, None, &[]).await
}

fn id_of(r: &common::Reply) -> String {
    r.json()["id"]
        .as_str()
        .unwrap_or_else(|| panic!("pas d'identifiant : {}", r.body))
        .to_string()
}

fn code(r: &common::Reply) -> String {
    r.json()["code"].as_str().unwrap_or_default().to_string()
}

/// Bail actif de 75 000 au 5 mars 2027, locataire « Issouf Tapsoba {name} » (téléphone facultatif) ; paie 75 000.
/// Renvoie (identifiant de la quittance, numéro).
async fn paid_receipt(a: &common::Api, g: &S, name: &str, phone: Option<&str>) -> (String, String) {
    let p = post(a, g, "/api/properties", json!({ "name": format!("Zogona {name}") })).await;
    let u = post(
        a,
        g,
        "/api/units",
        json!({ "property_id": id_of(&p), "reference": "A1", "kind": "apartment", "usage": "residential" }),
    )
    .await;
    let mut tenant = json!({ "kind": "person", "first_name": "Issouf", "last_name": format!("Tapsoba {name}") });
    if let Some(ph) = phone {
        tenant["phone"] = json!(ph);
    }
    let t = post(a, g, "/api/tenants", tenant).await;
    assert_eq!(t.status, StatusCode::CREATED, "{}", t.body);
    let l = post(
        a,
        g,
        "/api/leases",
        json!({ "unit_id": id_of(&u), "tenant_ids": [id_of(&t)], "kind": "residential", "start_date": "2027-03-01",
                "rent": 75000, "due_day": 5 }),
    )
    .await;
    let lease = id_of(&l);
    post(a, g, &format!("/api/leases/{lease}/activate"), json!({})).await;
    let key = uuid::Uuid::new_v4().to_string();
    let r = req(
        a,
        g,
        Method::POST,
        "/api/payments",
        Some(json!({ "lease_id": lease, "amount": 75000, "paid_on": "2027-03-05", "method": "cash" })),
        &[("idempotency-key", &key)],
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    (
        r.json()["receipt"]["id"].as_str().unwrap().to_string(),
        r.json()["receipt"]["number"].as_str().unwrap().to_string(),
    )
}

fn job(receipt: &str) -> Job {
    Job {
        id: uuid::Uuid::new_v4(),
        job_type: delivery::RENDER_JOB.to_string(),
        org_id: Some(ORG_A.parse().unwrap()),
        payload: json!({ "receipt_id": receipt }),
        attempts: 1,
    }
}

async fn render(a: &common::Api, receipt: &str, fail: bool) -> Result<(), String> {
    let db = common::app_db(a).await;
    delivery::render_receipt(&db, &a.files, Arc::new(Fake { fail }), &job(receipt)).await
}

/// Appel public, sans cookie ni jeton anti-falsification.
async fn public(a: &common::Api, path: &str) -> common::Reply {
    common::call(&a.app, Method::GET, path, "203.0.113.9", &[], None).await
}

// ───────── critères 1 et 2 : rendu en file, PDF servi, échec sans effet (R-049) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c1_rendu_en_file_puis_pdf_servi_en_flux() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (receipt, number) = paid_receipt(&a, &g, "c1", None).await;
    assert_eq!(
        count(&a, "SELECT count(*) FROM jobs WHERE type = 'render_receipt'").await,
        1,
        "rendu mis en file dans la transaction du paiement"
    );
    let early = get(&a, &g, &format!("/api/receipts/{receipt}/pdf")).await;
    assert_eq!((early.status, code(&early)), (StatusCode::CONFLICT, "receipt.pdf_not_ready".into()));
    assert_eq!(early.header("retry-after"), "5");
    assert_eq!(get(&a, &g, &format!("/api/receipts/{receipt}")).await.json()["pdf"]["state"], "pending");
    render(&a, &receipt, false).await.unwrap();
    render(&a, &receipt, false).await.unwrap();
    assert_eq!(count(&a, "SELECT count(*) FROM receipt_files").await, 2, "A4 et A5, rendu idempotent");
    let r = get(&a, &g, &format!("/api/receipts/{receipt}")).await;
    assert_eq!(r.json()["pdf"]["state"], "ready");
    assert_eq!(r.json()["pdf"]["sha256"].as_str().unwrap().len(), 64);
    let pdf = get(&a, &g, &format!("/api/receipts/{receipt}/pdf")).await;
    assert_eq!(pdf.status, StatusCode::OK);
    assert_eq!(pdf.header("content-type"), "application/pdf");
    assert_eq!(
        pdf.header("content-disposition"),
        format!("attachment; filename=\"quittance-{number}.pdf\"")
    );
    assert_eq!(pdf.header("cache-control"), "no-store");
    assert_eq!(pdf.body, "%PDF-1.7 factice A5", "A5 par défaut (DP-64)");
    let a4 = get(&a, &g, &format!("/api/receipts/{receipt}/pdf?paper=a4")).await;
    assert_eq!(a4.body, "%PDF-1.7 factice A4");
    let bad = get(&a, &g, &format!("/api/receipts/{receipt}/pdf?paper=a3")).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn c2_echec_de_rendu_le_paiement_reste_et_le_nouvel_essai_reussit() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (receipt, _) = paid_receipt(&a, &g, "c2", None).await;
    assert!(render(&a, &receipt, true).await.is_err());
    assert_eq!(count(&a, "SELECT count(*) FROM payments").await, 1, "le paiement reste");
    assert_eq!(count(&a, "SELECT count(*) FROM receipt_files").await, 0);
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'receipt_pdf_failed'").await,
        1
    );
    assert_eq!(get(&a, &g, &format!("/api/receipts/{receipt}")).await.json()["pdf"]["state"], "pending");
    render(&a, &receipt, false).await.unwrap();
    assert_eq!(get(&a, &g, &format!("/api/receipts/{receipt}")).await.json()["pdf"]["state"], "ready");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn alerte_quittance_sans_pdf_apres_dix_minutes_une_seule_fois() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    paid_receipt(&a, &g, "al", None).await;
    let db = common::app_db(&a).await;
    let (alerts, _) = delivery::check_missing_pdfs(&db, a.clock.as_ref()).await.unwrap();
    assert_eq!(alerts, 0, "avant 10 minutes : pas d'alerte");
    a.clock.advance(chrono::Duration::minutes(11));
    let (alerts, _) = delivery::check_missing_pdfs(&db, a.clock.as_ref()).await.unwrap();
    assert_eq!(alerts, 1);
    let (again, _) = delivery::check_missing_pdfs(&db, a.clock.as_ref()).await.unwrap();
    assert_eq!(again, 0, "une alerte par quittance");
    assert_eq!(count(&a, "SELECT count(*) FROM jobs WHERE type = 'receipt.pdf_missing'").await, 1);
    a.db.drop_db().await;
}

// ───────── critères 5 et 6 : envois consignés, message minimal (R-055, SECURITY.md §7.8) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c5_c6_whatsapp_message_minimal_et_envois_consignes() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (receipt, number) = paid_receipt(&a, &g, "wa", Some("00 00 01 21")).await;
    let r = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "whatsapp" })).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let b = r.json();
    assert_eq!(b["recipient"]["display_name"], "Issouf Tapsoba wa");
    assert_eq!(b["recipient"]["phone_last2"], "21");
    let url = b["share_link"]["url"].as_str().unwrap().to_string();
    assert!(
        url.starts_with("https://app.test/s/") && url.len() == "https://app.test/s/".len() + 43,
        "{url}"
    );
    assert_eq!(
        b["message"]["text"],
        format!(
            "Bonjour Issouf Tapsoba wa, voici votre quittance n° {number} pour 75 000 FCFA (loyer de mars 2027) : {url} — Agence A"
        )
    );
    let wa = b["message"]["whatsapp_url"].as_str().unwrap();
    assert!(wa.starts_with("https://wa.me/22600000121?text=Bonjour%20Issouf"), "{wa}");
    for channel in ["print_a5", "link_copied"] {
        let x = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": channel })).await;
        assert_eq!(x.status, StatusCode::CREATED, "{}", x.body);
    }
    let sends = get(&a, &g, &format!("/api/receipts/{receipt}/sends")).await;
    assert_eq!(sends.json()["items"].as_array().unwrap().len(), 3);
    assert_eq!(count(&a, "SELECT count(*) FROM share_links").await, 2, "WhatsApp et lien copié");
    assert_eq!(
        count(&a, "SELECT count(*) FROM audit_log WHERE action = 'receipt.sent'").await,
        3
    );
    // un canal inconnu, et un locataire sans numéro
    let bad = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "email" })).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let (silent, _) = paid_receipt(&a, &g, "np", None).await;
    let np = post(&a, &g, &format!("/api/receipts/{silent}/sends"), json!({ "channel": "whatsapp" })).await;
    assert_eq!(
        (np.status, code(&np)),
        (StatusCode::UNPROCESSABLE_ENTITY, "receipt.recipient_has_no_phone".into())
    );
    a.db.drop_db().await;
}

// ───────── critère 3 : page publique, révocation, expiration (E-12, A12) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c3_page_publique_document_seul_puis_410_et_404() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (receipt, number) = paid_receipt(&a, &g, "<b>pub</b>", None).await;
    let link = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "link_copied" })).await;
    let url = link.json()["share_link"]["url"].as_str().unwrap().to_string();
    let link_id = link.json()["share_link"]["id"].as_str().unwrap().to_string();
    let path = url.trim_start_matches("https://app.test").to_string();
    let page = public(&a, &path).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    assert!(page.header("content-type").starts_with("text/html"));
    assert!(page.body.contains(&number));
    assert!(page.body.contains("Soixante-quinze mille francs CFA"));
    assert!(page.body.contains("&lt;b&gt;pub&lt;/b&gt;"), "nom échappé, jamais du balisage");
    assert!(!page.body.contains("<script"), "aucun JavaScript applicatif");
    assert_eq!(page.header("x-robots-tag"), "noindex, nofollow");
    assert_eq!(page.header("referrer-policy"), "no-referrer");
    assert_eq!(page.header("cache-control"), "no-store");
    // PDF public : en préparation, puis servi
    assert_eq!(public(&a, &format!("{path}/pdf")).await.status, StatusCode::CONFLICT);
    render(&a, &receipt, false).await.unwrap();
    let pdf = public(&a, &format!("{path}/pdf")).await;
    assert_eq!((pdf.status, pdf.body.as_str()), (StatusCode::OK, "%PDF-1.7 factice A5"));
    // révocation : 410 avec le texte de S26
    let rv = post(&a, &g, &format!("/api/share-links/{link_id}/revoke"), json!({})).await;
    assert_eq!(rv.status, StatusCode::OK, "{}", rv.body);
    let gone = public(&a, &path).await;
    assert_eq!(gone.status, StatusCode::GONE);
    assert!(gone.body.contains("Ce lien a expiré. Demandez un nouveau lien à Agence A."));
    assert!(!gone.body.contains(&number), "rien du document après révocation");
    // expiration (E-12) : 31 jours plus tard
    let fresh = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "link_copied" })).await;
    let fresh_path = fresh.json()["share_link"]["url"]
        .as_str()
        .unwrap()
        .trim_start_matches("https://app.test")
        .to_string();
    assert_eq!(public(&a, &fresh_path).await.status, StatusCode::OK);
    a.clock.advance(chrono::Duration::days(31));
    assert_eq!(public(&a, &fresh_path).await.status, StatusCode::GONE);
    // jeton inconnu (forme valide) ou mal formé : 404
    let unknown = format!("/s/{}", "A".repeat(43));
    assert_eq!(public(&a, &unknown).await.status, StatusCode::NOT_FOUND);
    assert_eq!(public(&a, "/s/abc").await.status, StatusCode::NOT_FOUND);
    a.db.drop_db().await;
}

// ───────── ISO-9 : lecture par empreinte de jeton, rien d'autre ─────────

#[tokio::test(flavor = "multi_thread")]
async fn iso9_le_jeton_ne_montre_que_sa_ligne() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (receipt, _) = paid_receipt(&a, &g, "iso", None).await;
    let link = post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "link_copied" })).await;
    post(&a, &g, &format!("/api/receipts/{receipt}/sends"), json!({ "channel": "link_copied" })).await;
    let url = link.json()["share_link"]["url"].as_str().unwrap().to_string();
    let token = url.rsplit('/').next().unwrap();
    let hex: String = immo_app::identity::crypto::token_hash(token)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let zeros = "0".repeat(64);
    let db = common::app_db(&a).await;
    for (hash, expected) in [(hex.as_str(), 1_i64), (zeros.as_str(), 0)] {
        let mut tx = db.begin_token(hash).await.unwrap();
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM share_links").fetch_one(tx.conn()).await.unwrap();
        let receipts: i64 = sqlx::query_scalar("SELECT count(*) FROM receipts")
            .fetch_one(tx.conn())
            .await
            .unwrap();
        tx.rollback().await.unwrap();
        assert_eq!((n, receipts), (expected, 0), "une ligne de lien au plus, aucune quittance");
    }
    // une autre organisation ne voit aucun lien de A
    let mut tx = db.begin(ORG_B.parse().unwrap(), uuid::Uuid::nil()).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM share_links").fetch_one(tx.conn()).await.unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(n, 0);
    a.db.drop_db().await;
}
