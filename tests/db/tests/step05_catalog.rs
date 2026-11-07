//! Étape 05 : biens, lots, locataires, fiches mandants, recherche unique (F04, F05, F06, DP-92). Critères 1 à 8 de
//! `roadmap/etape-05-biens-lots-locataires.md` §6 ; cas chiffrés des fiches. Fixtures : bloc fictif `+226 00 00 …` seulement.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use axum::http::{Method, StatusCode};
use common::{GOOD, ORG_A, ORG_C, ORIGIN, S, actions, api, call, count, fast_params, login, req, sess};
use serde_json::{Value, json};
use sqlx::Executor;

const UN: &str = "11111111-0000-4000-8000-000000000011";

async fn add_member(a: &common::Api, id: &str, phone: &str, org: &str, role: &str) {
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO users (id, login_phone, display_name, password_hash)
               SELECT '{id}', '{phone}', 'Membre', password_hash FROM users WHERE id = '{UN}';
             INSERT INTO memberships (org_id, user_id, role) VALUES ('{org}', '{id}', '{role}');"
        )))
        .await
        .unwrap();
}

/// Session sur une organisation choisie (compte membre de plusieurs organisations).
async fn on_org(a: &common::Api, identifier: &str, org: &str, ip: &str) -> S {
    let (cookie, _) = login(&a.app, identifier, GOOD, ip).await;
    let csrf = a.svc.csrf_for(cookie.split('=').nth(1).unwrap());
    let chosen = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        ip,
        &[
            ("cookie", &cookie),
            ("origin", ORIGIN),
            ("content-type", "application/json"),
            ("x-csrf-token", &csrf),
        ],
        Some(&json!({ "organization_id": org }).to_string()),
    )
    .await;
    assert_eq!(chosen.status, StatusCode::OK, "{}", chosen.body);
    S {
        cookie: chosen.session_cookie().unwrap().split(';').next().unwrap().to_string(),
        csrf: chosen.json()["csrf_token"].as_str().unwrap().to_string(),
    }
}

async fn post(a: &common::Api, s: &S, path: &str, body: Value) -> common::Reply {
    req(a, s, Method::POST, path, Some(body), &[]).await
}

async fn get(a: &common::Api, s: &S, path: &str) -> common::Reply {
    req(a, s, Method::GET, path, None, &[]).await
}

async fn patch(a: &common::Api, s: &S, path: &str, body: Value, version: Option<i64>) -> common::Reply {
    let v = version.map(|v| format!("\"{v}\""));
    let extra: Vec<(&str, &str)> = v.as_deref().map(|v| vec![("if-match", v)]).unwrap_or_default();
    req(a, s, Method::PATCH, path, Some(body), &extra).await
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

/// Bien « Zogona » avec deux lots : « A1 » (habitation) et « B1 » (boutique). Renvoie (bien, A1, B1).
async fn zogona(a: &common::Api, g: &S) -> (String, String, String) {
    let p = post(
        a,
        g,
        "/api/properties",
        json!({ "name": "Zogona", "district": "Zogona", "city": "Ouagadougou" }),
    )
    .await;
    assert_eq!(p.status, StatusCode::CREATED, "{}", p.body);
    let pid = id_of(&p);
    let a1 = post(
        a,
        g,
        "/api/units",
        json!({ "property_id": pid, "reference": "A1", "kind": "apartment", "usage": "residential", "reference_rent": 75000 }),
    )
    .await;
    assert_eq!(a1.status, StatusCode::CREATED, "{}", a1.body);
    let b1 = post(
        a,
        g,
        "/api/units",
        json!({ "property_id": pid, "reference": "B1", "kind": "shop", "usage": "commercial", "permitted_activity": "Quincaillerie", "surface_m2": "18,5" }),
    )
    .await;
    assert_eq!(b1.status, StatusCode::CREATED, "{}", b1.body);
    (pid, id_of(&a1), id_of(&b1))
}

// ───────── critère 1 : maison seule et référence unique dans le bien (R-010, R-011) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c1_maison_seule_donne_un_bien_et_un_lot_et_la_reference_est_unique_dans_le_bien() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;

    let villa = post(
        &a,
        &g,
        "/api/properties",
        json!({ "name": "Villa Kaboré", "single_unit": {} }),
    )
    .await;
    assert_eq!(villa.status, StatusCode::CREATED, "{}", villa.body);
    let v = villa.json();
    assert_eq!(v["unit_count"], 1);
    assert_eq!(v["units"].as_array().unwrap().len(), 1);
    assert_eq!(v["units"][0]["reference"], "1");
    assert_eq!(v["units"][0]["kind"], "house");
    assert_eq!(v["units"][0]["usage"], "residential");
    assert_eq!(v["units"][0]["status"], "vacant");
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM properties WHERE org_id = '{ORG_A}'")).await,
        1
    );
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM units WHERE org_id = '{ORG_A}'")).await,
        1
    );

    // la création du lot échoue (type inconnu) : aucun bien n'est créé non plus
    let bad = post(
        &a,
        &g,
        "/api/properties",
        json!({ "name": "Villa Ouédraogo", "single_unit": { "kind": "chateau" } }),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", bad.body);
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM properties WHERE org_id = '{ORG_A}'")).await,
        1
    );

    let (pid, _, _) = zogona(&a, &g).await;
    // deux « A1 » dans le même bien : refusé ; « a1 » aussi (casse ignorée, DP-92)
    for reference in ["A1", "a1", " A1 "] {
        let dup = post(
            &a,
            &g,
            "/api/units",
            json!({ "property_id": pid, "reference": reference, "kind": "apartment", "usage": "residential" }),
        )
        .await;
        assert_eq!(
            (dup.status, code(&dup)),
            (StatusCode::CONFLICT, "unit.reference_duplicate".into()),
            "{reference:?}"
        );
    }
    // un bien sans lot existe (DP-92), puis « A1 » dans cet autre bien : accepté
    let other = post(&a, &g, "/api/properties", json!({ "name": "Gounghin" })).await;
    assert_eq!(other.json()["unit_count"], 0, "{}", other.body);
    let oid = id_of(&other);
    let ok = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": oid, "reference": "A1", "kind": "room", "usage": "residential" }),
    )
    .await;
    assert_eq!(ok.status, StatusCode::CREATED, "{}", ok.body);
    assert_eq!(
        get(&a, &g, &format!("/api/properties/{oid}")).await.json()["unit_count"],
        1
    );
    a.db.drop_db().await;
}

// ───────── critère 2 : champs selon l'usage (R-012, R-094, DP-41) ; loyer de référence (R-001) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c2_activite_autorisee_seulement_hors_habitation_et_montants_entiers() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (pid, a1, b1) = zogona(&a, &g).await;
    let b = get(&a, &g, &format!("/api/units/{b1}")).await.json();
    assert_eq!(b["permitted_activity"], "Quincaillerie");
    assert_eq!(b["surface_m2"], "18.50");
    assert!(get(&a, &g, &format!("/api/units/{a1}")).await.json()["permitted_activity"].is_null());

    let refused = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": pid, "reference": "A2", "kind": "apartment", "usage": "residential", "permitted_activity": "Restauration" }),
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused.json()["errors"][0]["field"], "permitted_activity");
    assert_eq!(refused.json()["errors"][0]["code"], "validation.not_applicable");
    let mixed = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": pid, "reference": "C1", "kind": "premises", "usage": "mixed", "permitted_activity": "Couture" }),
    )
    .await;
    assert_eq!(mixed.status, StatusCode::CREATED, "{}", mixed.body);

    for (rent, want) in [
        (json!(75000.5), "validation.amount_not_integer"),
        (json!(0), "validation.amount_positive"),
        (json!(1_000_000_000_i64), "validation.amount_out_of_range"),
        (json!("75000"), "validation.amount_not_integer"),
    ] {
        let r = post(
            &a,
            &g,
            "/api/units",
            json!({ "property_id": pid, "reference": "D1", "kind": "room", "usage": "residential", "reference_rent": rent }),
        )
        .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{rent} : {}", r.body);
        assert_eq!(r.json()["errors"][0]["code"], want, "{rent}");
    }
    let surface = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": pid, "reference": "D1", "kind": "room", "usage": "residential", "surface_m2": "45.555" }),
    )
    .await;
    assert_eq!(surface.json()["errors"][0]["field"], "surface_m2", "{}", surface.body);
    a.db.drop_db().await;
}

// ───────── critère 3 : téléphone unique parmi les fiches non archivées (R-017, R-018, DP-92) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c3_les_trois_formes_d_un_numero_designent_la_meme_fiche_et_le_doublon_propose_la_fiche_existante() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let t = post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Tapsoba", "first_name": "Issouf", "phone": "70123456", "phone_is_whatsapp": true }),
    )
    .await;
    assert_eq!(t.status, StatusCode::CREATED, "{}", t.body);
    let tid = id_of(&t);
    assert_eq!(t.json()["phone"], "+22670123456");
    assert_eq!(t.json()["display_name"], "Issouf Tapsoba");
    for form in ["+226 70 12 34 56", "0022670123456"] {
        let dup = post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "I. Tapsoba", "phone": form }),
        )
        .await;
        assert_eq!(
            (dup.status, code(&dup)),
            (StatusCode::CONFLICT, "tenant.phone_duplicate".into()),
            "{form}"
        );
        assert_eq!(dup.json()["params"]["conflict"]["id"], tid.as_str());
        assert_eq!(dup.json()["params"]["conflict"]["display_name"], "Issouf Tapsoba");
    }
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM tenants WHERE org_id = '{ORG_A}'")).await,
        1,
        "rien n'est créé"
    );
    // 7 chiffres : message explicite
    let short = post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Kaboré", "phone": "7012345" }),
    )
    .await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(short.json()["errors"][0]["field"], "phone");
    assert_eq!(short.json()["errors"][0]["code"], "validation.phone_format");
    // `display_name` et la pièce d'identité ne s'envoient pas
    for body in [
        json!({ "kind": "person", "last_name": "Kaboré", "display_name": "X" }),
        json!({ "kind": "person", "last_name": "Kaboré", "id_document": "B1234567" }),
    ] {
        let r = post(&a, &g, "/api/tenants", body).await;
        assert_eq!(
            (r.status, code(&r)),
            (StatusCode::UNPROCESSABLE_ENTITY, "request.unknown_field".into())
        );
    }
    // changer le numéro d'une autre fiche pour celui-ci : refusé
    let other = post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Ouédraogo", "phone": "+226 00 00 02 02" }),
    )
    .await;
    let oid = id_of(&other);
    let steal = patch(
        &a,
        &g,
        &format!("/api/tenants/{oid}"),
        json!({ "phone": "70 12 34 56" }),
        Some(1),
    )
    .await;
    assert_eq!(code(&steal), "tenant.phone_duplicate", "{}", steal.body);
    // une fiche archivée libère son numéro
    let arch = post(&a, &g, &format!("/api/tenants/{tid}/archive"), json!({})).await;
    assert_eq!(arch.status, StatusCode::OK, "{}", arch.body);
    let again = post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Tapsoba", "phone": "70123456" }),
    )
    .await;
    assert_eq!(again.status, StatusCode::CREATED, "{}", again.body);
    // société : raison sociale, activité, pas de prénom
    let co = post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "company", "last_name": "Boutique Wend-Panga SARL", "trade_activity": "Quincaillerie" }),
    )
    .await;
    assert_eq!(co.json()["display_name"], "Boutique Wend-Panga SARL", "{}", co.body);
    let acts = actions(&a, ORG_A).await;
    assert!(acts.iter().filter(|x| *x == "tenant.created").count() >= 3);
    assert!(acts.contains(&"tenant.archived".to_string()));
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn fusion_de_deux_fiches_sans_recopie_et_lecture_de_l_ancienne() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let keep = id_of(
        &post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Tapsoba", "first_name": "Issouf", "phone": "+226 00 00 01 01" }),
        )
        .await,
    );
    let dup = id_of(
        &post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Tapsoba", "first_name": "I.", "email": "issouf@exemple.bf" }),
        )
        .await,
    );
    let merged = post(&a, &g, &format!("/api/tenants/{dup}/merge"), json!({ "into_id": keep })).await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.body);
    assert_eq!(merged.json()["id"], keep.as_str());
    assert!(merged.json()["email"].is_null(), "aucune recopie (DP-92)");
    let old = get(&a, &g, &format!("/api/tenants/{dup}")).await.json();
    assert_eq!(old["merged_into_id"], keep.as_str());
    assert!(!old["archived_at"].is_null());
    // conditions : même fiche, fiche archivée, déjà fusionnée
    for (from, into) in [
        (keep.clone(), keep.clone()),
        (dup.clone(), keep.clone()),
        (keep.clone(), dup.clone()),
    ] {
        let r = post(
            &a,
            &g,
            &format!("/api/tenants/{from}/merge"),
            json!({ "into_id": into }),
        )
        .await;
        assert_eq!(
            (r.status, code(&r)),
            (StatusCode::CONFLICT, "tenant.merge_invalid".into())
        );
    }
    assert!(actions(&a, ORG_A).await.contains(&"tenant.merged".to_string()));
    a.db.drop_db().await;
}

// ───────── critères 4 et 8 : pièce d'identité désactivée ; champs masqués selon le rôle (R-019, §4.5) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c4_c8_piece_d_identite_desactivee_et_contact_de_secours_absent_pour_le_caissier() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let tid = id_of(
        &post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Zongo", "phone": "+226 00 00 03 03", "backup_contact": { "name": "Awa Zongo", "phone": "00 00 03 04" }, "notes": "Paie le 5" }),
        )
        .await,
    );
    let full = get(&a, &g, &format!("/api/tenants/{tid}")).await.json();
    assert_eq!(full["backup_contact"]["phone"], "+22600000304");
    assert_eq!(full["notes"], "Paie le 5");
    let cashier = on_org(&a, "70 00 00 12", ORG_A, "10.50.0.2").await;
    let seen = get(&a, &cashier, &format!("/api/tenants/{tid}")).await;
    assert_eq!(seen.status, StatusCode::OK, "{}", seen.body);
    let seen = seen.json();
    assert_eq!(seen["phone"], "+22600000303");
    assert!(seen.get("backup_contact").is_none(), "absent, pas null : {seen}");
    assert!(seen.get("notes").is_none());
    let accountant = sess(&a, "70 00 00 16").await;
    assert!(
        get(&a, &accountant, &format!("/api/tenants/{tid}"))
            .await
            .json()
            .get("notes")
            .is_none()
    );
    // le caissier lit mais n'écrit pas
    let w = post(
        &a,
        &cashier,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "X" }),
    )
    .await;
    assert_eq!(code(&w), "auth.forbidden");
    // pièce d'identité : jamais lisible au pilote
    let doc = get(&a, &g, &format!("/api/tenants/{tid}/id-document")).await;
    assert_eq!(
        (doc.status, code(&doc)),
        (StatusCode::FORBIDDEN, "feature.disabled".into())
    );
    a.db.drop_db().await;
}

// ───────── critère 5 : l'agent terrain ne voit que ses lots (R-081, A5) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c5_agent_terrain_assigne_a_deux_lots_sur_cinq() {
    let a = api(fast_params(), "0.0.0").await;
    let agent_id = "77777777-0000-4000-8000-000000000071";
    add_member(&a, agent_id, "+22670000071", ORG_A, "field_agent").await;
    let g = sess(&a, "70 00 00 11").await;
    let (pid, a1, b1) = zogona(&a, &g).await;
    let mut others = vec![];
    for r in ["C1", "C2", "C3"] {
        others.push(id_of(
            &post(
                &a,
                &g,
                "/api/units",
                json!({ "property_id": pid, "reference": r, "kind": "room", "usage": "residential" }),
            )
            .await,
        ));
    }
    let lone = id_of(
        &post(
            &a,
            &g,
            "/api/properties",
            json!({ "name": "Tanghin", "single_unit": {} }),
        )
        .await,
    );
    let membership: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT id::text FROM memberships WHERE user_id = '{agent_id}'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .unwrap();
    let put = req(
        &a,
        &g,
        Method::PUT,
        &format!("/api/users/{membership}/unit-assignments"),
        Some(json!({ "unit_ids": [a1, b1] })),
        &[],
    )
    .await;
    assert_eq!(put.status, StatusCode::OK, "{}", put.body);

    let agent = sess(&a, "70 00 00 71").await;
    let units = get(&a, &agent, "/api/units").await.json();
    let mut refs: Vec<&str> = units["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["reference"].as_str().unwrap())
        .collect();
    refs.sort_unstable();
    assert_eq!(refs, ["A1", "B1"]);
    assert_eq!(
        get(&a, &agent, &format!("/api/units/{a1}")).await.status,
        StatusCode::OK
    );
    for hidden in &others {
        assert_eq!(
            get(&a, &agent, &format!("/api/units/{hidden}")).await.status,
            StatusCode::NOT_FOUND
        );
    }
    // les biens : seulement ceux qui contiennent un lot assigné
    let props = get(&a, &agent, "/api/properties").await.json();
    assert_eq!(props["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        get(&a, &agent, &format!("/api/properties/{lone}")).await.status,
        StatusCode::NOT_FOUND
    );
    // la recherche aussi
    let found = get(&a, &agent, "/api/search?q=c1").await.json();
    assert_eq!(found["units"].as_array().unwrap().len(), 0, "{found}");
    let found = get(&a, &agent, "/api/search?q=a1").await.json();
    assert_eq!(found["units"].as_array().unwrap().len(), 1, "{found}");
    // sans bail, aucun locataire n'est « de ses lots » à l'étape 05
    post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Sawadogo" }),
    )
    .await;
    assert_eq!(
        get(&a, &agent, "/api/tenants").await.json()["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    a.db.drop_db().await;
}

// ───────── critère 6 : modification concurrente (n° 64) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c6_le_second_gestionnaire_recoit_un_conflit_de_version() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (pid, a1, _) = zogona(&a, &g).await;
    let tid = id_of(
        &post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Compaoré" }),
        )
        .await,
    );
    for (path, body) in [
        (format!("/api/properties/{pid}"), json!({ "city": "Ouaga" })),
        (format!("/api/units/{a1}"), json!({ "reference_rent": 80000 })),
        (format!("/api/tenants/{tid}"), json!({ "notes": "x" })),
    ] {
        let missing = patch(&a, &g, &path, body.clone(), None).await;
        assert_eq!(
            (missing.status, code(&missing)),
            (StatusCode::PRECONDITION_REQUIRED, "concurrency.version_required".into()),
            "{path}"
        );
        let first = patch(&a, &g, &path, body.clone(), Some(1)).await;
        assert_eq!(first.status, StatusCode::OK, "{path} : {}", first.body);
        assert_eq!(first.json()["version"], 2);
        let second = patch(&a, &g, &path, body, Some(1)).await;
        assert_eq!(
            (second.status, code(&second)),
            (StatusCode::PRECONDITION_FAILED, "concurrency.version_mismatch".into()),
            "{path}"
        );
    }
    assert_eq!(
        get(&a, &g, &format!("/api/units/{a1}")).await.json()["reference_rent"],
        80000
    );
    // un lot ne change pas de bien
    let mv = patch(
        &a,
        &g,
        &format!("/api/units/{a1}"),
        json!({ "property_id": pid }),
        Some(2),
    )
    .await;
    assert_eq!(code(&mv), "request.unknown_field");
    a.db.drop_db().await;
}

// ───────── critère 7 : archivage, rien n'est supprimé (R-005) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn c7_un_bien_archive_archive_ses_lots_et_reste_consultable() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let (pid, a1, _) = zogona(&a, &g).await;
    let r = post(&a, &g, &format!("/api/properties/{pid}/archive"), json!({})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert!(!r.json()["archived_at"].is_null());
    let unit = get(&a, &g, &format!("/api/units/{a1}")).await;
    assert_eq!(unit.status, StatusCode::OK);
    assert_eq!(unit.json()["status"], "archived");
    assert_eq!(
        get(&a, &g, "/api/properties").await.json()["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        get(&a, &g, "/api/properties?archived=true").await.json()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let edit = patch(
        &a,
        &g,
        &format!("/api/units/{a1}"),
        json!({ "reference_rent": 1 }),
        Some(2),
    )
    .await;
    assert_eq!(code(&edit), "unit.archived", "{}", edit.body);
    let new_unit = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": pid, "reference": "Z9", "kind": "room", "usage": "residential" }),
    )
    .await;
    assert_eq!(code(&new_unit), "property.archived");
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM units WHERE org_id = '{ORG_A}'")).await,
        2
    );
    let acts = actions(&a, ORG_A).await;
    assert!(acts.contains(&"property.archived".to_string()));
    assert_eq!(
        acts.iter().filter(|x| *x == "unit.archived").count(),
        2,
        "une ligne par lot archivé"
    );
    // l'application ne peut rien supprimer
    let pool = a.db.app_pool(1).await;
    for table in ["properties", "units", "tenants", "owners"] {
        let e = sqlx::query(sqlx::AssertSqlSafe(format!("DELETE FROM {table}")))
            .execute(&pool)
            .await
            .unwrap_err();
        assert!(e.to_string().contains("permission denied"), "{table} : {e}");
    }
    a.db.drop_db().await;
}

// ───────── mandants (F05, R-016, DP-92) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn mandants_fiche_is_self_du_plan_proprietaire_droits_et_archivage() {
    let a = api(fast_params(), "0.0.0").await;
    // ORG_C (plan Propriétaire), insérée directement par le jeu de données : sa fiche `is_self` existe (déclencheur)
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM owners WHERE org_id = '{ORG_C}' AND is_self AND display_name = 'Famille C'")
        )
        .await,
        1
    );
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM owners WHERE org_id = '{ORG_A}'")).await,
        0
    );
    let collab = sess(&a, "70 00 00 14").await;
    let list = get(&a, &collab, "/api/owners").await.json();
    assert_eq!(list["items"][0]["is_self"], true, "{list}");
    let c = post(
        &a,
        &collab,
        "/api/owners",
        json!({ "kind": "person", "display_name": "Autre" }),
    )
    .await;
    assert_eq!(code(&c), "plan.feature_unavailable");
    // un bien du plan Propriétaire prend la fiche is_self par défaut
    let owner_id = "77777777-0000-4000-8000-000000000072";
    add_member(&a, owner_id, "+22670000072", ORG_C, "owner").await;
    let o = sess(&a, "70 00 00 72").await;
    let p = post(
        &a,
        &o,
        "/api/properties",
        json!({ "name": "Maison familiale", "single_unit": {} }),
    )
    .await;
    assert_eq!(p.json()["default_owner"]["display_name"], "Famille C", "{}", p.body);

    // plan Agence : le gérant crée ; le caissier voit le nom sans le téléphone
    let g = sess(&a, "70 00 00 11").await;
    let m = post(
        &a,
        &g,
        "/api/owners",
        json!({ "kind": "person", "display_name": "Salif Ouédraogo", "phone": "00 00 05 05", "payout_method": "mobile_money" }),
    )
    .await;
    assert_eq!(m.status, StatusCode::CREATED, "{}", m.body);
    let mid = id_of(&m);
    assert_eq!(m.json()["phone"], "+22600000505");
    let cashier = on_org(&a, "70 00 00 12", ORG_A, "10.50.0.3").await;
    let seen = get(&a, &cashier, &format!("/api/owners/{mid}")).await.json();
    assert_eq!(seen["display_name"], "Salif Ouédraogo");
    assert!(
        seen.get("phone").is_none() && seen.get("payout_method").is_none(),
        "{seen}"
    );
    // propriétaire d'un bien : archivage refusé
    let pid = id_of(
        &post(
            &a,
            &g,
            "/api/properties",
            json!({ "name": "Cité An III", "default_owner_id": mid }),
        )
        .await,
    );
    let arch = post(&a, &g, &format!("/api/owners/{mid}/archive"), json!({})).await;
    assert_eq!(
        (arch.status, code(&arch)),
        (StatusCode::CONFLICT, "owner.has_properties".into())
    );
    assert_eq!(arch.json()["params"]["count"], 1);
    let p = get(&a, &g, &format!("/api/properties/{pid}")).await.json();
    let moved = patch(
        &a,
        &g,
        &format!("/api/properties/{pid}"),
        json!({ "default_owner_id": null }),
        Some(p["version"].as_i64().unwrap()),
    )
    .await;
    assert_eq!(moved.status, StatusCode::OK, "{}", moved.body);
    assert_eq!(
        post(&a, &g, &format!("/api/owners/{mid}/archive"), json!({}))
            .await
            .status,
        StatusCode::OK
    );
    // mandant d'une autre organisation : introuvable
    let gb = sess(&a, "un.gerant@agence.bf").await;
    let bmid = id_of(
        &post(
            &a,
            &gb,
            "/api/owners",
            json!({ "kind": "company", "display_name": "SCI Nord" }),
        )
        .await,
    );
    let cross = post(
        &a,
        &g,
        "/api/properties",
        json!({ "name": "Volé", "default_owner_id": bmid }),
    )
    .await;
    assert_eq!(cross.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", cross.body);
    assert_eq!(cross.json()["errors"][0]["code"], "owner.not_found");
    a.db.drop_db().await;
}

// ───────── recherche unique (§2.6, F04 §4.1) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn recherche_insensible_aux_accents_et_par_numero() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    zogona(&a, &g).await;
    let tid = id_of(
        &post(
            &a,
            &g,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Tapsoba", "first_name": "Issouf", "phone": "+226 00 00 01 01" }),
        )
        .await,
    );
    post(
        &a,
        &g,
        "/api/tenants",
        json!({ "kind": "person", "last_name": "Ouédraogo", "first_name": "Aminata" }),
    )
    .await;
    for q in [
        "tapsoba",
        "TAPSOBA",
        "Tapsóba",
        "issouf",
        "00000101",
        "+226 00 00 01 01",
        "0000 0101",
    ] {
        let r = get(&a, &g, &format!("/api/search?q={}", urlencode(q))).await;
        assert_eq!(r.status, StatusCode::OK, "{q} : {}", r.body);
        let tenants = r.json()["tenants"].clone();
        assert_eq!(tenants.as_array().unwrap().len(), 1, "{q} : {tenants}");
        assert_eq!(tenants[0]["id"], tid.as_str());
    }
    let r = get(&a, &g, "/api/search?q=ouedraogo").await.json();
    assert_eq!(r["tenants"][0]["display_name"], "Aminata Ouédraogo");
    let r = get(&a, &g, "/api/search?q=zogona").await.json();
    assert_eq!(r["units"].as_array().unwrap().len(), 2, "{r}");
    assert_eq!(r["units"][0]["property_name"], "Zogona");
    assert_eq!(r["receipts"], json!([]));
    let none = get(&a, &g, "/api/search?q=inconnu").await.json();
    assert_eq!(none, json!({ "tenants": [], "units": [], "receipts": [] }));
    let short = get(&a, &g, "/api/search?q=a").await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    a.db.drop_db().await;
}

fn urlencode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' => char::from(b).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

// ───────── isolation (A4, ISO-3) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn a4_un_identifiant_d_une_autre_organisation_repond_404_et_trace_la_tentative() {
    let a = api(fast_params(), "0.0.0").await;
    let gb = sess(&a, "un.gerant@agence.bf").await;
    let (bp, bu, _) = zogona(&a, &gb).await;
    let bt = id_of(
        &post(
            &a,
            &gb,
            "/api/tenants",
            json!({ "kind": "person", "last_name": "Kiemdé" }),
        )
        .await,
    );
    let g = sess(&a, "70 00 00 11").await;
    for path in [
        format!("/api/properties/{bp}"),
        format!("/api/units/{bu}"),
        format!("/api/tenants/{bt}"),
    ] {
        let r = get(&a, &g, &path).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{path}");
    }
    let unit = post(
        &a,
        &g,
        "/api/units",
        json!({ "property_id": bp, "reference": "X1", "kind": "room", "usage": "residential" }),
    )
    .await;
    assert_eq!(unit.status, StatusCode::NOT_FOUND, "{}", unit.body);
    assert!(
        actions(&a, ORG_A)
            .await
            .iter()
            .filter(|x| *x == "security.cross_org_attempt")
            .count()
            >= 3,
        "chaque tentative est tracée"
    );
    // la base refuse un lot qui vise le bien d'une autre organisation (clé composite, ISO-3)
    let e = a
        .db
        .admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO units (org_id, property_id, reference, kind, usage) VALUES ('{ORG_A}', '{bp}', 'X2', 'room', 'residential')"
        )))
        .await
        .unwrap_err();
    assert!(e.to_string().contains("foreign key"), "{e}");
    a.db.drop_db().await;
}
