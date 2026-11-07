//! Étape 06 : baux et échéancier (F07, F08, DP-94). Critères 1 à 10 de `roadmap/etape-06-baux-et-echeancier.md` §6 ; tests d'or
//! E-01, E-02, E-03, E-04, E-22, E-35, E-36, E-37, E-39, E-40, E-44 au niveau de l'API, horloge fixée.

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
use common::{ORG_A, S, actions, api, count, fast_params, req, sess};
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

fn id_of(r: &common::Reply) -> String {
    r.json()["id"]
        .as_str()
        .unwrap_or_else(|| panic!("pas d'identifiant : {}", r.body))
        .to_string()
}

fn code(r: &common::Reply) -> String {
    r.json()["code"].as_str().unwrap_or_default().to_string()
}

/// Un lot d'habitation « A1 » et un locataire ; renvoie (lot, locataire).
async fn unit_and_tenant(a: &common::Api, g: &S, usage: &str) -> (String, String) {
    let p = post(a, g, "/api/properties", json!({ "name": format!("Zogona {usage}") })).await;
    let pid = id_of(&p);
    let kind = if usage == "residential" { "apartment" } else { "shop" };
    let u = post(
        a,
        g,
        "/api/units",
        json!({ "property_id": pid, "reference": "A1", "kind": kind, "usage": usage }),
    )
    .await;
    assert_eq!(u.status, StatusCode::CREATED, "{}", u.body);
    let t = post(
        a,
        g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": format!("Tapsoba {usage}") }),
    )
    .await;
    (id_of(&u), id_of(&t))
}

async fn lease(
    a: &common::Api,
    g: &S,
    unit: &str,
    tenant: &str,
    start: &str,
    due_day: Option<i64>,
    extra: Value,
) -> String {
    let mut body =
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": start, "rent": 75000 });
    if let Some(d) = due_day {
        body["due_day"] = json!(d);
    }
    if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            b.insert(k.clone(), v.clone());
        }
    }
    let r = post(a, g, "/api/leases", body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    id_of(&r)
}

async fn activate(a: &common::Api, g: &S, id: &str) -> common::Reply {
    post(a, g, &format!("/api/leases/{id}/activate"), json!({})).await
}

/// (période, date d'échéance, montant dû, statut) de chaque échéance.
async fn schedule(a: &common::Api, g: &S, id: &str) -> Vec<(String, String, i64, String)> {
    let r = get(a, g, &format!("/api/leases/{id}/installments")).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    r.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["period"].as_str().unwrap().to_string(),
                i["due_date"].as_str().unwrap().to_string(),
                i["due_amount"].as_i64().unwrap(),
                i["status"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

// ───────── critères 1 et 4 : première échéance, statuts (E-01, E-04, E-37) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c1_c4_bail_de_75000_au_5_active_le_1er_mars_puis_a_venir_due_en_retard() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2027-03-01", Some(5), json!({})).await;
    let r = activate(&a, &g, &id).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.json()["status"], "active");
    assert_eq!(
        schedule(&a, &g, &id).await,
        vec![("2027-03-01".into(), "2027-03-05".into(), 75000, "upcoming".into())]
    );
    // le lot est occupé
    assert_eq!(
        get(&a, &g, &format!("/api/units/{unit}")).await.json()["status"],
        "occupied"
    );
    for (day, want) in [(4, "upcoming"), (5, "due"), (6, "late")] {
        let g = goto(&a, 2027, 3, day).await;
        let s = schedule(&a, &g, &id).await;
        assert_eq!(s[0].3, want, "le {day} mars");
    }
    let g = goto(&a, 2027, 3, 12).await;
    let r = get(&a, &g, &format!("/api/leases/{id}/installments")).await;
    assert_eq!(r.json()["items"][0]["days_late"], 7);
    assert!(actions(&a, ORG_A).await.contains(&"lease.activated".to_string()));
    a.db.drop_db().await;
}

// ───────── critère 2 : avenant (E-02, E-44) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c2_avenant_a_80000_mars_inchange_avril_mis_a_jour() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2027-03-01", Some(5), json!({})).await;
    activate(&a, &g, &id).await;
    let g = goto(&a, 2027, 3, 15).await;
    let r = post(
        &a,
        &g,
        &format!("/api/leases/{id}/amendments"),
        json!({ "effective_from": "2027-04-01", "rent": 80000 }),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let s = schedule(&a, &g, &id).await;
    assert_eq!(
        s.iter().map(|x| (x.0.as_str(), x.2)).collect::<Vec<_>>(),
        vec![("2027-03-01", 75000), ("2027-04-01", 80000)]
    );
    // le loyer en vigueur passe à 80 000 le 05/04
    let g = goto(&a, 2027, 4, 5).await;
    assert_eq!(
        get(&a, &g, &format!("/api/leases/{id}")).await.json()["current_rent"],
        80000
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn e44_avenant_saisi_le_15_12_d_effet_au_01_12_decembre_reste_janvier_passe() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2026, 11, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2026-11-01", Some(5), json!({})).await;
    activate(&a, &g, &id).await;
    let g = goto(&a, 2026, 12, 10).await;
    immo_app::lease::generate_due_installments(&common::app_db(&a).await, a.clock.as_ref())
        .await
        .unwrap();
    let g2 = goto(&a, 2026, 12, 15).await;
    let _ = g;
    let r = post(
        &a,
        &g2,
        &format!("/api/leases/{id}/amendments"),
        json!({ "effective_from": "2026-12-01", "rent": 80000 }),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let s = schedule(&a, &g2, &id).await;
    let by: Vec<(&str, i64)> = s.iter().map(|x| (x.1.as_str(), x.2)).collect();
    assert!(by.contains(&("2026-12-05", 75000)), "{by:?}");
    assert!(by.contains(&("2027-01-05", 80000)), "{by:?}");
    // la ligne de janvier générée à 75 000 a été annulée avec un motif, une ligne à 80 000 l'a remplacée (DP-94)
    let r = get(&a, &g2, &format!("/api/leases/{id}/installments")).await;
    let jan = r.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["due_date"] == "2027-01-05")
        .cloned()
        .unwrap();
    let lines = jan["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2, "{jan}");
    assert!(
        lines
            .iter()
            .any(|l| l["voided"] == true && l["void_reason"] == "avenant du 15/12/2026")
    );
    a.db.drop_db().await;
}

// ───────── critère 3 : génération idempotente (E-03) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c3_la_tache_executee_deux_fois_ne_cree_aucun_doublon() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2027-03-01", Some(5), json!({})).await;
    activate(&a, &g, &id).await;
    goto(&a, 2027, 4, 20).await;
    let db = common::app_db(&a).await;
    let first = immo_app::lease::generate_due_installments(&db, a.clock.as_ref())
        .await
        .unwrap();
    let second = immo_app::lease::generate_due_installments(&db, a.clock.as_ref())
        .await
        .unwrap();
    assert_eq!((first, second), (2, 0), "avril et mai, puis rien");
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM installments WHERE lease_id = '{id}'")
        )
        .await,
        3
    );
    a.db.drop_db().await;
}

// ───────── critère 5 : calendrier (E-39, E-40, R-100) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c5_jour_31_en_fevrier_et_premier_mois_entier() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 2, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2027-02-01", Some(31), json!({})).await;
    activate(&a, &g, &id).await;
    assert_eq!(schedule(&a, &g, &id).await[0].1, "2027-02-28");
    // activé le 20 mars : mars dû en entier ; jour d'échéance absent : le 1er, signalé (R-031)
    let (unit2, tenant2) = unit_and_tenant(&a, &g, "mixed").await;
    let g = goto(&a, 2027, 3, 20).await;
    let id2 = lease(&a, &g, &unit2, &tenant2, "2027-03-20", None, json!({})).await;
    let r = activate(&a, &g, &id2).await;
    assert!(
        r.json()["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w == "lease.due_day_defaulted"),
        "{}",
        r.body
    );
    let s = schedule(&a, &g, &id2).await;
    assert_eq!((s[0].0.as_str(), s[0].2), ("2027-03-01", 75000));
    a.db.drop_db().await;
}

// ───────── critère 6 : chevauchement, y compris avec un bail terminé (R-020, n° 41) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c6_second_bail_en_cours_ou_chevauchant_un_bail_termine_refuse_avec_les_dates() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let first = lease(&a, &g, &unit, &tenant, "2027-03-01", Some(5), json!({})).await;
    activate(&a, &g, &first).await;
    let second = lease(&a, &g, &unit, &tenant, "2027-06-01", Some(5), json!({})).await;
    let r = activate(&a, &g, &second).await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "lease.overlap".into()));
    assert_eq!(r.json()["params"]["start_date"], "2027-03-01");
    // résilié le 31/03 : un bail qui commence le 31/03 chevauche encore, le 01/04 non (bornes incluses)
    let t = post(
        &a,
        &g,
        &format!("/api/leases/{first}/terminate"),
        json!({ "terminated_on": "2027-03-31", "reason": "Départ du locataire" }),
    )
    .await;
    assert_eq!(t.status, StatusCode::OK, "{}", t.body);
    assert_eq!(
        get(&a, &g, &format!("/api/units/{unit}")).await.json()["status"],
        "vacant"
    );
    let clash = lease(&a, &g, &unit, &tenant, "2027-03-31", Some(5), json!({})).await;
    assert_eq!(code(&activate(&a, &g, &clash).await), "lease.overlap");
    let ok = lease(&a, &g, &unit, &tenant, "2027-04-01", Some(5), json!({})).await;
    assert_eq!(activate(&a, &g, &ok).await.status, StatusCode::OK);
    a.db.drop_db().await;
}

// ───────── critère 7 : dépôt (E-22), lot en travaux (E-36) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c7_depot_superieur_a_deux_mois_et_lot_en_travaux() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let r = post(
        &a,
        &g,
        "/api/leases",
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": "2027-03-01", "rent": 75000, "due_day": 5, "deposit_amount": 200000 }),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert!(
        r.json()["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w == "lease.deposit_above_two_months")
    );
    let id = id_of(&r);
    let w = post(
        &a,
        &g,
        &format!("/api/units/{unit}/works"),
        json!({ "under_works": true }),
    )
    .await;
    assert_eq!(w.json()["status"], "under_works", "{}", w.body);
    assert_eq!(code(&activate(&a, &g, &id).await), "unit.under_works");
    post(
        &a,
        &g,
        &format!("/api/units/{unit}/works"),
        json!({ "under_works": false }),
    )
    .await;
    assert_eq!(activate(&a, &g, &id).await.status, StatusCode::OK);
    // un lot occupé ne passe pas en travaux
    let again = post(
        &a,
        &g,
        &format!("/api/units/{unit}/works"),
        json!({ "under_works": true }),
    )
    .await;
    assert_eq!(code(&again), "unit.works_requires_vacant");
    a.db.drop_db().await;
}

// ───────── critère 8 : prolongation (E-35) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c8_prolonger_la_fin_genere_les_echeances_jusqu_a_la_nouvelle_fin() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2026, 11, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(
        &a,
        &g,
        &unit,
        &tenant,
        "2026-11-01",
        Some(5),
        json!({ "end_date": "2026-12-31", "deposit_amount": 150000 }),
    )
    .await;
    activate(&a, &g, &id).await;
    let g = goto(&a, 2026, 12, 20).await;
    let r = post(
        &a,
        &g,
        &format!("/api/leases/{id}/amendments"),
        json!({ "effective_from": "2027-01-01", "new_end_date": "2027-06-30" }),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.json()["end_date"], "2027-06-30");
    assert_eq!(r.json()["deposit_amount"], 150000, "dépôt inchangé");
    assert!(schedule(&a, &g, &id).await.iter().any(|x| x.0 == "2027-01-01"));
    // raccourcir la fin : refusé
    let r = post(
        &a,
        &g,
        &format!("/api/leases/{id}/amendments"),
        json!({ "effective_from": "2027-02-01", "new_end_date": "2027-03-31" }),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    a.db.drop_db().await;
}

// ───────── critère 9 et autres gardes ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c9_archiver_un_lot_ou_un_locataire_a_bail_en_cours_est_refuse() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 1).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    let id = lease(&a, &g, &unit, &tenant, "2027-03-01", Some(5), json!({})).await;
    activate(&a, &g, &id).await;
    assert_eq!(
        code(&post(&a, &g, &format!("/api/units/{unit}/archive"), json!({})).await),
        "unit.has_active_lease"
    );
    assert_eq!(
        code(&post(&a, &g, &format!("/api/tenants/{tenant}/archive"), json!({})).await),
        "lease.active_exists"
    );
    // un bail actif ne se modifie plus par PATCH ; le type suit l'usage du lot (DP-94)
    let p = req(
        &a,
        &g,
        Method::PATCH,
        &format!("/api/leases/{id}"),
        Some(json!({ "rent": 1 })),
        &[("if-match", "2")],
    )
    .await;
    assert_eq!(code(&p), "lease.not_draft", "{}", p.body);
    let wrong = post(
        &a,
        &g,
        "/api/leases",
        json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "commercial", "start_date": "2027-09-01", "rent": 1 }),
    )
    .await;
    assert_eq!(code(&wrong), "lease.kind_not_allowed");
    // loyer nul, dates inversées (A10)
    let bad = post(&a, &g, "/api/leases", json!({ "unit_id": unit, "tenant_ids": [tenant], "kind": "residential", "start_date": "2027-09-01", "end_date": "2027-08-01", "rent": 0 })).await;
    let codes: Vec<String> = bad.json()["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["code"].as_str().unwrap().to_string())
        .collect();
    assert!(
        codes.contains(&"validation.rent_positive".to_string()) && codes.contains(&"validation.date_order".to_string()),
        "{codes:?}"
    );
    // rejouée avec une clé d'idempotence, l'activation renvoie le même bail ; sans clé, elle est refusée
    let again = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/leases/{id}/activate"),
        Some(json!({})),
        &[("idempotency-key", "11111111-2222-4333-8444-555555555555")],
    )
    .await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.body);
    assert_eq!(code(&activate(&a, &g, &id).await), "lease.not_draft");
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM installments WHERE lease_id = '{id}'")
        )
        .await,
        1
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn lignes_arriere_et_annulation_motivee() {
    let a = api(fast_params(), "0.0.0").await;
    let g = goto(&a, 2027, 3, 10).await;
    let (unit, tenant) = unit_and_tenant(&a, &g, "residential").await;
    // commencé en janvier, activé en mars : janvier et février se saisissent en arriérés (R-100)
    let id = lease(&a, &g, &unit, &tenant, "2027-01-01", Some(5), json!({})).await;
    activate(&a, &g, &id).await;
    assert_eq!(schedule(&a, &g, &id).await[0].0, "2027-03-01");
    let l = post(
        &a,
        &g,
        &format!("/api/leases/{id}/installment-lines"),
        json!({ "kind": "arrears", "label": "Loyer de janvier", "amount": 75000, "due_date": "2027-01-05" }),
    )
    .await;
    assert_eq!(l.status, StatusCode::CREATED, "{}", l.body);
    let s = schedule(&a, &g, &id).await;
    assert_eq!(s[0], ("2027-01-01".into(), "2027-01-05".into(), 75000, "late".into()));
    let line = id_of(&l);
    let short = post(
        &a,
        &g,
        &format!("/api/installment-lines/{line}/void"),
        json!({ "reason": "x" }),
    )
    .await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    let v = post(
        &a,
        &g,
        &format!("/api/installment-lines/{line}/void"),
        json!({ "reason": "Saisie en double" }),
    )
    .await;
    assert_eq!(v.status, StatusCode::OK, "{}", v.body);
    let s = schedule(&a, &g, &id).await;
    assert_eq!((s[0].2, s[0].3.as_str()), (0, "paid"), "dû nul : réglé (DP-94)");
    assert!(
        actions(&a, ORG_A)
            .await
            .contains(&"installment.line_voided".to_string())
    );
    a.db.drop_db().await;
}

// ───────── critère 10 et écriture seule (G-05) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c10_aucune_colonne_de_statut_et_echeances_en_ecriture_seule() {
    let a = api(fast_params(), "0.0.0").await;
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM information_schema.columns WHERE table_name IN ('installments', 'installment_lines') AND column_name ILIKE '%status%'"
        )
        .await,
        0
    );
    let pool = a.db.app_pool(1).await;
    for sql in [
        "UPDATE installments SET due_date = due_date",
        "DELETE FROM installment_lines",
        "UPDATE lease_amendments SET note = 'x'",
    ] {
        let e = sqlx::query(sqlx::AssertSqlSafe(sql.to_string()))
            .execute(&pool)
            .await
            .unwrap_err();
        assert!(e.to_string().contains("permission denied"), "{sql} : {e}");
    }
    a.db.drop_db().await;
}

// ───────── alerte « génération tardive » (F08, DP-95) ─────────

struct Generator(immo_app::platform::db::Db, std::sync::Arc<immo_db_tests::TestClock>);

impl immo_app::platform::jobs::JobHandler for Generator {
    async fn handle(&self, _job: &immo_app::platform::jobs::Job) -> Result<(), String> {
        immo_app::lease::generate_due_installments(&self.0, self.1.as_ref())
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn controle_de_02h00_alerte_seulement_si_la_generation_du_jour_n_est_pas_terminee() {
    use immo_app::platform::clock::OUAGADOUGOU;
    use immo_app::platform::jobs::{self, NewJob, RunOutcome};
    let a = api(fast_params(), "0.0.0").await;
    goto(&a, 2027, 3, 2).await;
    let db = common::app_db(&a).await;
    let clock = a.clock.as_ref();
    // 02 h 00 à Ouagadougou = 02:00 UTC
    assert_eq!(
        jobs::local_time_today(clock, OUAGADOUGOU, immo_app::lease::LATE_CHECK_HOUR, 0).to_rfc3339(),
        "2027-03-02T02:00:00+00:00"
    );
    // aucune génération du jour : alerte
    assert!(immo_app::lease::check_generation_late(&db, clock, None).await.unwrap());
    // génération du jour planifiée puis exécutée : plus d'alerte
    let key = jobs::daily_key(immo_app::lease::GENERATE_JOB, None, clock, OUAGADOUGOU);
    jobs::schedule(
        &db,
        clock,
        None,
        NewJob::new(immo_app::lease::GENERATE_JOB, json!({})).unique(key),
    )
    .await
    .unwrap();
    assert!(
        immo_app::lease::check_generation_late(&db, clock, None).await.unwrap(),
        "en file, pas terminée"
    );
    let runner = Generator(db.clone(), a.clock.clone());
    while jobs::run_once(&db, clock, &runner).await.unwrap() != RunOutcome::Idle {}
    assert!(!immo_app::lease::check_generation_late(&db, clock, None).await.unwrap());
    // le lendemain, la génération de la veille ne compte pas
    goto(&a, 2027, 3, 3).await;
    assert!(
        immo_app::lease::check_generation_late(&db, a.clock.as_ref(), None)
            .await
            .unwrap()
    );
    a.db.drop_db().await;
}
