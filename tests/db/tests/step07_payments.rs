//! Étape 07 : encaisser (F09, F10, DP-96). Critères 1 à 8 de `roadmap/etape-07-encaisser.md` §6 au niveau de l'API et de la
//! base ; cas d'or E-05, E-06, E-07, E-09, E-10, E-11, E-25, E-27, E-31, E-32, E-33, E-42 ; gardes G-04 à G-09, G-12, G-13 ;
//! concurrence (n° 37) ; horloge fixée.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use axum::http::{Method, StatusCode};
use chrono::TimeZone;
use common::{ORG_A, S, api, count, fast_params, req, sess};
use immo_app::platform::clock::Clock;
use serde_json::{Value, json};
use sqlx::Executor;

/// Place l'horloge du service ET `app_now()` de la base au même instant (09:00 UTC = 09:00 à Ouagadougou), puis rouvre une session.
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

/// `POST /api/payments` avec une clé d'idempotence.
async fn pay(a: &common::Api, s: &S, key: &str, body: Value) -> common::Reply {
    req(
        a,
        s,
        Method::POST,
        "/api/payments",
        Some(body),
        &[("idempotency-key", key)],
    )
    .await
}

fn key() -> String {
    uuid::Uuid::new_v4().to_string()
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

/// Un bien, un lot d'habitation et un locataire ; renvoie (lot, locataire).
async fn unit_and_tenant(a: &common::Api, g: &S, name: &str) -> (String, String) {
    let p = post(a, g, "/api/properties", json!({ "name": format!("Zogona {name}") })).await;
    let u = post(
        a,
        g,
        "/api/units",
        json!({ "property_id": id_of(&p), "reference": "A1", "kind": "apartment", "usage": "residential" }),
    )
    .await;
    assert_eq!(u.status, StatusCode::CREATED, "{}", u.body);
    let t = post(
        a,
        g,
        "/api/tenants",
        json!({ "kind": "person", "first_name": "Issouf", "last_name": format!("Tapsoba {name}") }),
    )
    .await;
    (id_of(&u), id_of(&t))
}

/// Bail actif de 75 000 au 5 commençant le 1er mars 2027 (charges facultatives) ; renvoie (bail, locataire).
async fn active_lease(a: &common::Api, g: &S, name: &str, charges: i64) -> (String, String) {
    let (unit, tenant) = unit_and_tenant(a, g, name).await;
    let r = post(
        a,
        g,
        "/api/leases",
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": "2027-03-01",
                "rent": 75000, "charges": charges, "due_day": 5 }),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let id = id_of(&r);
    let act = post(a, g, &format!("/api/leases/{id}/activate"), json!({})).await;
    assert_eq!(act.status, StatusCode::OK, "{}", act.body);
    (id, tenant)
}

fn cash(lease: &str, amount: i64, on: &str) -> Value {
    json!({ "lease_id": lease, "amount": amount, "paid_on": on, "method": "cash" })
}

/// (période, statut, dû, reste) des échéances du bail.
async fn schedule(a: &common::Api, g: &S, id: &str) -> Vec<(String, String, i64, i64)> {
    let r = get(a, g, &format!("/api/leases/{id}/installments")).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    r.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["period"].as_str().unwrap().to_string(),
                i["status"].as_str().unwrap().to_string(),
                i["due_amount"].as_i64().unwrap(),
                i["remaining_amount"].as_i64().unwrap(),
            )
        })
        .collect()
}

// ───────── critère 1 : partiel, puis en retard à J+1 (E-05, E-25) ; E-11 ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c1_partiel_50000_puis_en_retard_a_j_plus_1_et_quittance_d_essai() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c1", 0).await;
    let ctx = get(&a, &g, &format!("/api/leases/{lease}/payment-context")).await;
    assert_eq!(ctx.status, StatusCode::OK, "{}", ctx.body);
    assert_eq!(ctx.json()["suggested_amount"], 75000);
    assert_eq!(ctx.json()["today"], "2027-03-05");
    assert_eq!(ctx.json()["confirm_threshold"], 1_000_000);
    let r = pay(&a, &g, &key(), cash(&lease, 50000, "2027-03-05")).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let b = r.json();
    assert_eq!(b["installments"][0]["status"], "partial");
    assert_eq!(b["installments"][0]["remaining_amount"], 25000);
    assert_eq!(b["receipt"]["remaining_after"], 25000);
    // organisation en phase setup : série d'essai (DP-61, DP-96)
    assert_eq!(b["receipt"]["number"], "ESSAI-REC-2027-000001");
    assert_eq!(b["receipt"]["series"], "trial");
    assert_eq!(b["receipt"]["snapshot"]["receipt"]["watermark"], "ESSAI");
    assert_eq!(b["receipt"]["amount_in_words"], "cinquante mille francs CFA");
    assert_eq!(b["payment"]["cash_day"], "2027-03-05");
    assert_eq!(b["payment"]["payer"]["kind"], "tenant");
    let g = goto(&a, 2027, 3, 6).await;
    let s = schedule(&a, &g, &lease).await;
    assert_eq!(s[0].1, "late", "J+1 : en retard même partiellement payée (DP-13)");
    assert_eq!((s[0].2, s[0].3), (75000, 25000));
    // journal S11 et détail
    let list = get(&a, &g, "/api/payments").await;
    assert_eq!(list.json()["items"].as_array().unwrap().len(), 1, "{}", list.body);
    let pid = b["payment"]["id"].as_str().unwrap();
    let one = get(&a, &g, &format!("/api/payments/{pid}")).await;
    assert_eq!(one.json()["receipt"]["number"], "ESSAI-REC-2027-000001", "{}", one.body);
    assert_eq!(one.json()["allocations"][0]["amount"], 50000);
    let rid = b["receipt"]["id"].as_str().unwrap();
    let rec = get(&a, &g, &format!("/api/receipts/{rid}")).await;
    assert_eq!(rec.json()["status"], "issued", "{}", rec.body);
    a.db.drop_db().await;
}

// ───────── critère 2 : avance (E-06) et crédit du bail (E-31) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c2_e06_avance_sur_l_echeance_suivante_deja_generee() {
    let a = api(fast_params(), "0.0.0").await;
    // le 10/03, avril (05/04) est dans la fenêtre de 30 jours : il est généré
    let g = goto(&a, 2027, 3, 10).await;
    let (lease, _) = active_lease(&a, &g, "c2a", 0).await;
    let refused = pay(&a, &g, &key(), cash(&lease, 100000, "2027-03-10")).await;
    assert_eq!(
        (refused.status, code(&refused)),
        (StatusCode::UNPROCESSABLE_ENTITY, "payment.surplus_not_confirmed".into()),
        "{}",
        refused.body
    );
    assert_eq!(refused.json()["params"]["surplus_amount"], 25000);
    let mut body = cash(&lease, 100000, "2027-03-10");
    body["surplus"] = json!("advance");
    let r = pay(&a, &g, &key(), body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let b = r.json();
    let parts: Vec<i64> = b["payment"]["allocations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["amount"].as_i64().unwrap())
        .collect();
    assert_eq!(parts, vec![75000, 25000]);
    assert_eq!(b["payment"]["unallocated_amount"], 0);
    assert_eq!(b["receipt"]["advance_amount"], 25000);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn c2_e31_sans_echeance_suivante_credit_du_bail_affecte_a_la_generation() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c2b", 0).await;
    let mut body = cash(&lease, 100000, "2027-03-05");
    body["surplus"] = json!("advance");
    let r = pay(&a, &g, &key(), body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.json()["payment"]["unallocated_amount"], 25000);
    let ctx = get(&a, &g, &format!("/api/leases/{lease}/payment-context")).await;
    assert_eq!(ctx.json()["balance"]["credit_amount"], 25000);
    // génération d'avril : le crédit s'y affecte, reste dû 50 000
    let g = goto(&a, 2027, 4, 10).await;
    immo_app::lease::generate_due_installments(&common::app_db(&a).await, a.clock.as_ref())
        .await
        .unwrap();
    let s = schedule(&a, &g, &lease).await;
    // le 10/04, avril et mai (05/05, dans la fenêtre de 30 jours) sont générés ; le crédit va à avril, le plus ancien
    assert_eq!(s.len(), 3);
    assert_eq!((s[1].0.as_str(), s[1].2, s[1].3), ("2027-04-01", 75000, 50000));
    assert_eq!((s[2].0.as_str(), s[2].3), ("2027-05-01", 75000));
    let ctx = get(&a, &g, &format!("/api/leases/{lease}/payment-context")).await;
    assert_eq!(ctx.json()["balance"]["credit_amount"], 0);
    assert_eq!(
        count(&a, "SELECT count(*) FROM allocations WHERE created_by IS NULL").await,
        1,
        "affectation système"
    );
    a.db.drop_db().await;
}

// ───────── critère 3 : double clic (E-07, R-042, DP-49) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c3_e07_meme_cle_un_seul_paiement_et_refus_des_cles_fautives() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c3", 0).await;
    let k = key();
    let first = pay(&a, &g, &k, cash(&lease, 75000, "2027-03-05")).await;
    let second = pay(&a, &g, &k, cash(&lease, 75000, "2027-03-05")).await;
    assert_eq!(
        (first.status, second.status),
        (StatusCode::CREATED, StatusCode::CREATED),
        "{}",
        second.body
    );
    assert_eq!(second.header("idempotent-replay"), "true");
    assert_eq!(first.json(), second.json(), "même corps");
    assert_eq!(count(&a, "SELECT count(*) FROM payments").await, 1);
    assert_eq!(count(&a, "SELECT count(*) FROM receipts").await, 1);
    // même clé, contenu différent
    let other = pay(&a, &g, &k, cash(&lease, 70000, "2027-03-05")).await;
    assert_eq!(
        (other.status, code(&other)),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotency.key_reused_with_different_body".into()
        )
    );
    // clé absente
    let none = req(
        &a,
        &g,
        Method::POST,
        "/api/payments",
        Some(cash(&lease, 1000, "2027-03-05")),
        &[],
    )
    .await;
    assert_eq!(
        (none.status, code(&none)),
        (StatusCode::BAD_REQUEST, "idempotency.key_missing".into())
    );
    // même clé trois fois en parallèle : un seul paiement de plus
    let k2 = key();
    let mut body = cash(&lease, 1000, "2027-03-05");
    body["surplus"] = json!("advance");
    let (r1, r2, r3) = tokio::join!(
        pay(&a, &g, &k2, body.clone()),
        pay(&a, &g, &k2, body.clone()),
        pay(&a, &g, &k2, body.clone()),
    );
    for r in [&r1, &r2, &r3] {
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }
    assert_eq!(
        count(&a, "SELECT count(*) FROM payments").await,
        2,
        "un seul paiement pour la clé jouée trois fois"
    );
    a.db.drop_db().await;
}

// ───────── critère 4 : numérotation (E-10, E-27) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c4_e10_e27_numeros_consecutifs_puis_000001_en_2027_serie_officielle() {
    let a = api(fast_params(), "0.0.0").await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE organizations SET lifecycle = 'live', went_live_at = now() WHERE id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE org_settings SET receipt_prefix = 'HOR' WHERE org_id = '{ORG_A}'"
        )))
        .await
        .unwrap();
    let g = goto(&a, 2026, 12, 31).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "c4").await;
    let l = post(
        &a,
        &g,
        "/api/leases",
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": "2026-12-01",
                "rent": 75000, "due_day": 5 }),
    )
    .await;
    let lease = id_of(&l);
    let act = post(&a, &g, &format!("/api/leases/{lease}/activate"), json!({})).await;
    assert_eq!(act.status, StatusCode::OK, "{}", act.body);
    let n1 = pay(&a, &g, &key(), cash(&lease, 10000, "2026-12-31")).await;
    let n2 = pay(&a, &g, &key(), cash(&lease, 10000, "2026-12-31")).await;
    assert_eq!(n1.json()["receipt"]["number"], "HOR-2026-000001", "{}", n1.body);
    assert_eq!(n2.json()["receipt"]["number"], "HOR-2026-000002");
    // paiement du 31/12 saisi le 02/01 : numéroté en 2027 (année d'émission, DP-61)
    let g = goto(&a, 2027, 1, 2).await;
    let n3 = pay(&a, &g, &key(), cash(&lease, 10000, "2026-12-31")).await;
    assert_eq!(n3.json()["receipt"]["number"], "HOR-2027-000001", "{}", n3.body);
    assert_eq!(n3.json()["receipt"]["issued_on"], "2027-01-02");
    assert_eq!(n3.json()["payment"]["paid_on"], "2026-12-31");
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM (SELECT org_id, series, numbering_year FROM receipts GROUP BY 1, 2, 3 \
             HAVING count(*) <> max(sequence)) t"
        )
        .await,
        0,
        "requête de trous : zéro ligne"
    );
    a.db.drop_db().await;
}

// ───────── critères 5 et 6 : seuil (E-09), doublon de référence (E-42), montant en lettres (E-11) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c6_e09_seuil_et_e42_reference_deja_utilisee() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c6", 0).await;
    let mut big = cash(&lease, 1_250_000, "2027-03-05");
    big["surplus"] = json!("advance");
    let refused = pay(&a, &g, &key(), big.clone()).await;
    assert_eq!(
        (refused.status, code(&refused)),
        (StatusCode::UNPROCESSABLE_ENTITY, "payment.confirmation_required".into())
    );
    assert_eq!(refused.json()["params"]["threshold"], 1_000_000);
    big["confirm_high_amount"] = json!(true);
    let ok = pay(&a, &g, &key(), big).await;
    assert_eq!(ok.status, StatusCode::CREATED, "{}", ok.body);
    assert_eq!(
        ok.json()["receipt"]["amount_in_words"],
        "un million deux cent cinquante mille francs CFA"
    );
    // référence mobile money : comparée sans casse ni espaces (DP-96)
    let mut mm = json!({ "lease_id": lease, "amount": 25000, "paid_on": "2027-03-05", "method": "mobile_money",
                         "external_ref": "MP 2611.AB", "surplus": "advance" });
    let first = pay(&a, &g, &key(), mm.clone()).await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.body);
    mm["external_ref"] = json!("mp2611.ab");
    let dup = pay(&a, &g, &key(), mm.clone()).await;
    assert_eq!(
        (dup.status, code(&dup)),
        (StatusCode::CONFLICT, "payment.external_ref_duplicate".into()),
        "{}",
        dup.body
    );
    assert_eq!(
        dup.json()["params"]["receipt_number"],
        first.json()["receipt"]["number"]
    );
    mm["acknowledge_external_ref_duplicate"] = json!(true);
    let confirmed = pay(&a, &g, &key(), mm).await;
    assert_eq!(confirmed.status, StatusCode::CREATED, "{}", confirmed.body);
    a.db.drop_db().await;
}

// ───────── critère 8 : loyer puis charges (E-32), payé par un tiers (E-33) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c8_e32_loyer_avant_charges_et_e33_paiement_par_un_tiers() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c8", 10000).await;
    let mut body = cash(&lease, 50000, "2027-03-05");
    body["payer"] =
        json!({ "kind": "third_party", "third_party_name": "Moussa Tapsoba", "third_party_phone": "70 00 00 99" });
    let r = pay(&a, &g, &key(), body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let b = r.json();
    assert_eq!(b["payment"]["allocations"].as_array().unwrap().len(), 1);
    assert_eq!(b["payment"]["allocations"][0]["kind"], "rent");
    assert_eq!(b["payment"]["allocations"][0]["amount"], 50000);
    assert_eq!(
        b["installments"][0]["remaining_amount"], 35000,
        "25 000 de loyer et 10 000 de charges"
    );
    assert_eq!(b["payment"]["payer"]["third_party_phone"], "+22670000099");
    let mention = b["receipt"]["snapshot"]["payer"]["mention"].as_str().unwrap();
    assert!(
        mention.starts_with("Payé par Moussa Tapsoba pour le compte de "),
        "{mention}"
    );
    a.db.drop_db().await;
}

// ───────── critère 7 : immuable (R-047, E-08, G-05) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c7_paiement_immuable_par_l_api_et_par_la_base() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "c7", 0).await;
    let r = pay(&a, &g, &key(), cash(&lease, 75000, "2027-03-05")).await;
    let pid = r.json()["payment"]["id"].as_str().unwrap().to_string();
    for m in [Method::PUT, Method::PATCH, Method::DELETE] {
        let x = req(&a, &g, m.clone(), &format!("/api/payments/{pid}"), Some(json!({})), &[]).await;
        assert_eq!(
            (x.status, code(&x)),
            (StatusCode::METHOD_NOT_ALLOWED, "payment.immutable".into()),
            "{m} {}",
            x.body
        );
    }
    let db = common::app_db(&a).await;
    let org: uuid::Uuid = ORG_A.parse().unwrap();
    for sql in [
        "UPDATE payments SET amount = 1",
        "DELETE FROM allocations",
        "UPDATE receipts SET amount = 1",
        "TRUNCATE receipts",
        "UPDATE receipt_counters SET last_number = last_number + 5",
        "UPDATE receipt_counters SET last_number = last_number - 1",
        "DELETE FROM receipt_counters",
    ] {
        let mut tx = db.begin(org, uuid::Uuid::nil()).await.unwrap();
        let res = sqlx::query(sqlx::AssertSqlSafe(sql)).execute(tx.conn()).await;
        assert!(res.is_err(), "refusé pour immo_app : {sql}");
        tx.rollback().await.unwrap();
    }
    // numéro 9 demandé à la place du suivant : refusé (G-07 à G-09)
    let mut tx = db.begin(org, uuid::Uuid::nil()).await.unwrap();
    let res = sqlx::query(
        "INSERT INTO receipts (org_id, payment_id, series, numbering_year, sequence, number, amount, issued_on, issued_by, snapshot) \
         SELECT org_id, id, 'trial', 2027, 9, 'ESSAI-REC-2027-000009', amount, '2027-03-05', created_by, '{}' FROM payments LIMIT 1",
    )
    .execute(tx.conn())
    .await;
    assert!(res.is_err());
    tx.rollback().await.unwrap();
    a.db.drop_db().await;
}

// ───────── gardes reportées de l'étape 06 (R-023, R-036) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn ligne_payee_non_annulable_et_avenant_retroactif_refuse() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "g6", 0).await;
    let r = pay(&a, &g, &key(), cash(&lease, 30000, "2027-03-05")).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let line = r.json()["payment"]["allocations"][0]["installment_line_id"]
        .as_str()
        .unwrap()
        .to_string();
    let v = post(
        &a,
        &g,
        &format!("/api/installment-lines/{line}/void"),
        json!({ "reason": "erreur de saisie" }),
    )
    .await;
    assert_eq!(
        (v.status, code(&v)),
        (StatusCode::CONFLICT, "installment.line_has_payments".into()),
        "{}",
        v.body
    );
    let am = post(
        &a,
        &g,
        &format!("/api/leases/{lease}/amendments"),
        json!({ "effective_from": "2027-03-02", "rent": 80000 }),
    )
    .await;
    assert_eq!(
        (am.status, code(&am)),
        (StatusCode::CONFLICT, "lease.amendment_paid_installment".into()),
        "{}",
        am.body
    );
    a.db.drop_db().await;
}

// ───────── refus de saisie (A8 à A10, R-001, R-039) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn refus_montant_decimal_date_future_champ_inconnu_bail_brouillon_payeur_etranger() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "rf", 0).await;
    let dec = pay(
        &a,
        &g,
        &key(),
        json!({ "lease_id": lease, "amount": 75000.5, "paid_on": "2027-03-05" }),
    )
    .await;
    assert_eq!(dec.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", dec.body);
    assert!(dec.body.contains("validation.amount_not_integer"), "{}", dec.body);
    let text = pay(
        &a,
        &g,
        &key(),
        json!({ "lease_id": lease, "amount": "75000", "paid_on": "2027-03-05" }),
    )
    .await;
    assert!(text.body.contains("validation.amount_not_integer"), "{}", text.body);
    let big = pay(
        &a,
        &g,
        &key(),
        json!({ "lease_id": lease, "amount": 1_000_000_000_i64, "paid_on": "2027-03-05" }),
    )
    .await;
    assert!(big.body.contains("validation.amount_out_of_range"), "{}", big.body);
    let future = pay(&a, &g, &key(), cash(&lease, 1000, "2027-03-06")).await;
    assert_eq!(future.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(future.body.contains("validation.date_in_future"), "{}", future.body);
    let mut unknown = cash(&lease, 1000, "2027-03-05");
    unknown["discount"] = json!(10);
    let u = pay(&a, &g, &key(), unknown).await;
    assert_eq!(u.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", u.body);
    // bail brouillon
    let (unit, tenant) = unit_and_tenant(&a, &g, "rf2").await;
    let draft = post(
        &a,
        &g,
        "/api/leases",
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": "2027-03-01", "rent": 75000 }),
    )
    .await;
    let d = pay(&a, &g, &key(), cash(&id_of(&draft), 1000, "2027-03-05")).await;
    assert_eq!(
        (d.status, code(&d)),
        (StatusCode::CONFLICT, "payment.lease_not_in_progress".into())
    );
    // payeur : locataire d'un autre bail
    let mut other = cash(&lease, 1000, "2027-03-05");
    other["payer"] = json!({ "kind": "tenant", "tenant_id": tenant });
    let o = pay(&a, &g, &key(), other).await;
    assert!(o.body.contains("payer.not_on_lease"), "{}", o.body);
    assert_eq!(
        count(&a, "SELECT count(*) FROM payments").await,
        0,
        "aucun refus n'a écrit"
    );
    a.db.drop_db().await;
}

// ───────── aperçu et répartition ajustée (R-043, R-044) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn apercu_sans_ecriture_et_repartition_ajustee() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "ap", 10000).await;
    let p = post(
        &a,
        &g,
        "/api/payments/preview",
        json!({ "lease_id": lease, "amount": 80000 }),
    )
    .await;
    assert_eq!(p.status, StatusCode::OK, "{}", p.body);
    let parts: Vec<i64> = p.json()["allocations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["amount"].as_i64().unwrap())
        .collect();
    assert_eq!(parts, vec![75000, 5000]);
    assert_eq!(p.json()["remaining_after"], 5000);
    assert_eq!(count(&a, "SELECT count(*) FROM payments").await, 0);
    let ctx = get(&a, &g, &format!("/api/leases/{lease}/payment-context")).await;
    assert_eq!(ctx.json()["open_lines"][1]["kind"], "charges", "{}", ctx.body);
    let charges = ctx.json()["open_lines"][1]["installment_line_id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut body = cash(&lease, 10000, "2027-03-05");
    body["allocations"] = json!([{ "installment_line_id": charges, "amount": 10000 }]);
    let r = pay(&a, &g, &key(), body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.json()["payment"]["allocations"][0]["kind"], "charges");
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM audit_log WHERE action = 'allocation.adjusted'"
        )
        .await,
        1
    );
    // ligne soldée entre-temps : aperçu périmé
    let mut again = cash(&lease, 5000, "2027-03-05");
    again["allocations"] = json!([{ "installment_line_id": charges, "amount": 5000 }]);
    let x = pay(&a, &g, &key(), again).await;
    assert_eq!(
        (x.status, code(&x)),
        (StatusCode::CONFLICT, "payment.stale_allocation".into()),
        "{}",
        x.body
    );
    a.db.drop_db().await;
}

// ───────── concurrence (n° 37, DP-96, P13) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn concurrence_quatre_caissiers_sur_un_bail_sans_depassement_ni_trou() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 5).await;
    let (lease, _) = active_lease(&a, &g, "cc", 0).await;
    let isolation: String = {
        let db = common::app_db(&a).await;
        let mut tx = db.begin(ORG_A.parse().unwrap(), uuid::Uuid::nil()).await.unwrap();
        let v = sqlx::query_scalar("SHOW transaction_isolation")
            .fetch_one(tx.conn())
            .await
            .unwrap();
        tx.commit().await.unwrap();
        v
    };
    assert_eq!(isolation, "read committed", "DB §4.5 exige READ COMMITTED");
    let worker = |n: i64| {
        let a = &a;
        let g = &g;
        let lease = lease.clone();
        async move {
            for _ in 0..10 {
                let mut body = cash(&lease, 7000 + n, "2027-03-05");
                body["surplus"] = json!("advance");
                let r = pay(a, g, &key(), body).await;
                assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
            }
        }
    };
    tokio::join!(worker(0), worker(1), worker(2), worker(3));
    assert_eq!(count(&a, "SELECT count(*) FROM payments").await, 40);
    assert_eq!(
        count(&a, "SELECT max(sequence)::bigint FROM receipts").await,
        40,
        "1 à 40"
    );
    assert_eq!(
        count(&a, "SELECT count(DISTINCT sequence) FROM receipts").await,
        40,
        "aucun doublon"
    );
    // la ligne de mars n'est jamais dépassée (P4) ; aucun crédit négatif (P2) ; somme des écritures (P3)
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM v_installment_lines WHERE allocated_amount > due_amount"
        )
        .await,
        0
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM v_payment_credit WHERE credit_amount < 0").await,
        0
    );
    assert_eq!(
        count(
            &a,
            "SELECT ((SELECT sum(amount) FROM payments) - (SELECT sum(amount) FROM allocations) \
                    - (SELECT sum(credit_amount) FROM v_lease_credit))::bigint"
        )
        .await,
        0,
        "P3"
    );
    a.db.drop_db().await;
}
