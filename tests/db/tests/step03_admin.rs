//! Étape 03, API : paramètres, accord d'accès support, journal d'audit, matrice des droits sur toutes les routes de l'étape
//! (F01 §9.8, F02 §7.6, F03 §8, F40). Cas chiffrés : `PATCH /organization` avec `If-Match: "3"` alors que la version est 4 → 412.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use axum::http::{Method, StatusCode};
use chrono::Duration;
use common::{
    GOOD, IP, ORG_A, ORG_B, ORG_C, ORIGIN, S, actions, api, call, count, fast_params, login, reauth, req, sess,
};
use immo_app::identity::authorize::{Action, Decision, DenyReason, Plan, Role, authorize};
use immo_server::api::{Access, ROUTES};
use serde_json::{Value, json};
use sqlx::Executor;

const UN: &str = "11111111-0000-4000-8000-000000000011";
const DEUX: &str = "22222222-0000-4000-8000-000000000012";

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

// ───────── organisation et paramètres ─────────

#[tokio::test(flavor = "multi_thread")]
async fn patch_organization_exige_la_bonne_version() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let org = req(&a, &g, Method::GET, "/api/organization", None, &[]).await;
    assert_eq!(org.json()["version"], 1);
    let patch = |body: Value, version: Option<&'static str>| {
        let extra: Vec<(&str, &str)> = version.map(|v| ("if-match", v)).into_iter().collect();
        let (a, g) = (&a, &g);
        async move { req(a, g, Method::PATCH, "/api/organization", Some(body), &extra).await }
    };
    assert_eq!(
        patch(json!({ "name": "Agence A+" }), None).await.status,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert_eq!(
        patch(json!({ "name": "Agence A+" }), Some("\"1\"")).await.status,
        StatusCode::OK
    );
    // cas chiffré n° 11 : version 2 en base, If-Match "3" → 412 sans modification
    let stale = patch(json!({ "name": "Autre nom" }), Some("\"3\"")).await;
    assert_eq!(
        (stale.status, stale.json()["code"].clone()),
        (StatusCode::PRECONDITION_FAILED, json!("concurrency.version_mismatch"))
    );
    let invalid = patch(json!({ "name": "A" }), Some("\"2\"")).await;
    assert_eq!(
        invalid.json()["errors"][0],
        json!({ "field": "name", "code": "validation.required", "params": {} })
    );
    let ok = patch(json!({ "city": "  Bobo-Dioulasso " }), Some("\"2\"")).await;
    assert_eq!(
        (
            ok.json()["name"].clone(),
            ok.json()["city"].clone(),
            ok.json()["version"].clone()
        ),
        (json!("Agence A+"), json!("Bobo-Dioulasso"), json!(3))
    );
    // l'audit ne garde que les clés changées
    let rows: Vec<(String, String)> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT before::text, after::text FROM audit_log WHERE org_id = '{ORG_A}' AND action = 'settings.updated' ORDER BY id"
    )))
    .fetch_all(&a.db.admin)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        serde_json::from_str::<Value>(&rows[1].0).unwrap(),
        json!({ "city": "Ouagadougou" })
    );
    assert_eq!(
        serde_json::from_str::<Value>(&rows[1].1).unwrap(),
        json!({ "city": "Bobo-Dioulasso" })
    );
    // un caissier lit (partiel) mais n'écrit pas
    let cashier = on_org(&a, "70 00 00 12", ORG_A, "10.34.0.1").await;
    assert_eq!(
        req(&a, &cashier, Method::GET, "/api/organization", None, &[])
            .await
            .status,
        StatusCode::OK
    );
    let denied = req(
        &a,
        &cashier,
        Method::PATCH,
        "/api/organization",
        Some(json!({ "name": "Piraté" })),
        &[("if-match", "3")],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn parametres_par_defaut_validation_des_montants_et_effet_sur_me() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let s = req(&a, &g, Method::GET, "/api/settings", None, &[]).await;
    assert_eq!(
        (
            s.json()["receipt_prefix"].clone(),
            s.json()["default_paper"].clone(),
            s.json()["share_link_ttl_days"].clone(),
            s.json()["payment_confirm_threshold"].clone(),
            s.json()["version"].clone()
        ),
        (json!("REC"), json!("a5"), json!(30), json!(1_000_000), json!(1))
    );
    assert_eq!(
        (s.json()["timezone"].clone(), s.json()["currency"].clone()),
        (json!("Africa/Ouagadougou"), json!("XOF"))
    );
    let me = req(&a, &g, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(
        me.json()["settings"]["payment_confirm_threshold"],
        1_000_000,
        "les vrais réglages, plus les valeurs fixes"
    );
    let patch = |body: Value, version: &'static str| {
        let (a, g) = (&a, &g);
        async move {
            req(
                a,
                g,
                Method::PATCH,
                "/api/settings",
                Some(body),
                &[("if-match", version)],
            )
            .await
        }
    };
    // cas chiffré n° 12 : décimal, négatif, trop grand ; préfixe en minuscules ; papier inconnu ; durée hors bornes
    for (body, field, code) in [
        (
            json!({ "payment_confirm_threshold": 1_000_000.5 }),
            "payment_confirm_threshold",
            "validation.amount_not_integer",
        ),
        (
            json!({ "payment_confirm_threshold": -1 }),
            "payment_confirm_threshold",
            "validation.amount_out_of_range",
        ),
        (
            json!({ "payment_confirm_threshold": 1_000_000_000 }),
            "payment_confirm_threshold",
            "validation.amount_out_of_range",
        ),
        (
            json!({ "payment_confirm_threshold": "beaucoup" }),
            "payment_confirm_threshold",
            "validation.amount_not_integer",
        ),
        (json!({ "receipt_prefix": "rec" }), "receipt_prefix", "validation.enum"),
        (json!({ "receipt_prefix": "A" }), "receipt_prefix", "validation.enum"),
        (json!({ "default_paper": "a3" }), "default_paper", "validation.enum"),
        (
            json!({ "share_link_ttl_days": 366 }),
            "share_link_ttl_days",
            "validation.amount_out_of_range",
        ),
    ] {
        let r = patch(body.clone(), "1").await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(r.json()["errors"][0]["field"], field, "{body}");
        assert_eq!(r.json()["errors"][0]["code"], code, "{body}");
    }
    // toutes les erreurs reviennent ensemble
    let many = patch(json!({ "receipt_prefix": "x", "default_paper": "a3" }), "1").await;
    assert_eq!(many.json()["errors"].as_array().unwrap().len(), 2);
    // champ inconnu (le logo vient à l'étape 08)
    let unknown = patch(json!({ "logo": "x" }), "1").await;
    assert_eq!(unknown.json()["code"], "request.unknown_field");
    // valide
    let ok = patch(json!({ "receipt_prefix": "AGB", "payment_confirm_threshold": 500_000, "default_paper": "a4", "receipt_header": "Agence A — Ouagadougou", "address": "Secteur 15", "legal_ids": { "IFU": "00012345X" } }), "1").await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(
        (
            ok.json()["receipt_prefix"].clone(),
            ok.json()["version"].clone(),
            ok.json()["address"].clone()
        ),
        (json!("AGB"), json!(2), json!("Secteur 15"))
    );
    let me = req(&a, &g, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(
        (
            me.json()["settings"]["payment_confirm_threshold"].clone(),
            me.json()["settings"]["default_paper"].clone()
        ),
        (json!(500_000), json!("a4"))
    );
    // version périmée
    assert_eq!(
        patch(json!({ "default_paper": "a5" }), "1").await.status,
        StatusCode::PRECONDITION_FAILED
    );
    // rien de changé : pas de nouvelle version, pas de ligne
    let same = patch(json!({ "default_paper": "a4" }), "2").await;
    assert_eq!(same.json()["version"], 2);
    let rows = actions(&a, ORG_A).await;
    assert_eq!(rows.iter().filter(|x| *x == "settings.updated").count(), 1);
    a.db.drop_db().await;
}

// ───────── accord d'accès support ─────────

async fn platform_sees(a: &common::Api, org: &str) -> i64 {
    let mut tx = a.db.platform.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.org_id', $1, true)")
        .bind(org)
        .execute(&mut *tx)
        .await
        .unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM memberships")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    n
}

#[tokio::test(flavor = "multi_thread")]
async fn accorder_puis_retirer_l_acces_support() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let grant = |body: Value| req(&a, &g, Method::POST, "/api/support-grants", Some(body), &[]);
    let valid = json!({ "scope": "read", "reason": "Aide au démarrage" });
    // confirmation exigée, avant tout effet
    assert_eq!(grant(valid.clone()).await.json()["code"], "auth.reauth_required");
    assert_eq!(count(&a, "SELECT count(*) FROM support_grants").await, 0);
    reauth(&a, &g).await;
    // cas limites : 0, 169, négatif, motif de 4 caractères ou de 5 espaces, portée inconnue
    for (body, field) in [
        (
            json!({ "scope": "read", "reason": "Aide", "duration_hours": 24 }),
            "reason",
        ),
        (
            json!({ "scope": "read", "reason": "     ", "duration_hours": 24 }),
            "reason",
        ),
        (
            json!({ "scope": "read", "reason": "Aide au démarrage", "duration_hours": 0 }),
            "duration_hours",
        ),
        (
            json!({ "scope": "read", "reason": "Aide au démarrage", "duration_hours": 169 }),
            "duration_hours",
        ),
        (
            json!({ "scope": "read", "reason": "Aide au démarrage", "duration_hours": -3 }),
            "duration_hours",
        ),
        (json!({ "scope": "tout", "reason": "Aide au démarrage" }), "scope"),
    ] {
        let r = grant(body.clone()).await;
        assert_eq!(
            (r.status, r.json()["errors"][0]["field"].clone()),
            (StatusCode::UNPROCESSABLE_ENTITY, json!(field)),
            "{body}"
        );
    }
    assert_eq!(count(&a, "SELECT count(*) FROM support_grants").await, 0);
    assert_eq!(
        platform_sees(&a, ORG_A).await,
        0,
        "sans accord, la console ne voit rien"
    );

    // 24 heures par défaut, expiration exacte
    let first = grant(valid).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.body);
    assert_eq!(first.json()["duration_hours"], 24);
    assert_eq!(first.json()["status"], "active");
    let granted = chrono::DateTime::parse_from_rfc3339(first.json()["granted_at"].as_str().unwrap()).unwrap();
    let expires = chrono::DateTime::parse_from_rfc3339(first.json()["expires_at"].as_str().unwrap()).unwrap();
    assert_eq!(expires - granted, Duration::hours(24));
    assert_eq!(first.json()["scope"], "read");
    assert!(platform_sees(&a, ORG_A).await > 0, "avec accord, la console lit");
    assert_eq!(platform_sees(&a, ORG_B).await, 0, "l'accord de A ne couvre pas B");
    // 168 heures : accepté ; deux accords simultanés permis
    let max = grant(json!({ "scope": "import_assist", "reason": "Import de départ", "duration_hours": 168 })).await;
    assert_eq!(max.status, StatusCode::OK, "{}", max.body);
    let list = req(&a, &g, Method::GET, "/api/support-grants", None, &[]).await;
    assert_eq!(list.json()["grants"].as_array().unwrap().len(), 2);
    assert_eq!(
        list.json()["accesses"],
        json!([]),
        "aucune consultation tant que la console n'existe pas"
    );

    // retrait immédiat de l'un : l'autre garde l'accès ; puis le second
    let id1 = first.json()["id"].as_str().unwrap().to_string();
    let id2 = max.json()["id"].as_str().unwrap().to_string();
    let revoked = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/support-grants/{id1}/revoke"),
        None,
        &[],
    )
    .await;
    assert_eq!(
        (revoked.status, revoked.json()["status"].clone()),
        (StatusCode::OK, json!("revoked"))
    );
    assert!(platform_sees(&a, ORG_A).await > 0, "l'autre accord reste valable");
    assert_eq!(
        req(
            &a,
            &g,
            Method::POST,
            &format!("/api/support-grants/{id1}/revoke"),
            None,
            &[]
        )
        .await
        .json()["params"]["reason"],
        "already_ended"
    );
    req(
        &a,
        &g,
        Method::POST,
        &format!("/api/support-grants/{id2}/revoke"),
        None,
        &[],
    )
    .await;
    assert_eq!(platform_sees(&a, ORG_A).await, 0, "après la dernière révocation, rien");
    let log = actions(&a, ORG_A).await;
    assert_eq!(log.iter().filter(|x| *x == "support.access_granted").count(), 2);
    assert_eq!(
        log.iter().filter(|x| *x == "support.access_revoked").count(),
        2,
        "la révocation refusée n'écrit rien"
    );

    // la confirmation expire : retirer demande de confirmer à nouveau
    a.clock.advance(Duration::minutes(6));
    let g2 = sess(&a, "70 00 00 11").await;
    let r = req(
        &a,
        &g2,
        Method::POST,
        &format!("/api/support-grants/{id1}/revoke"),
        None,
        &[],
    )
    .await;
    assert_eq!(r.json()["code"], "auth.reauth_required");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_historique_des_consultations_et_l_accord_d_une_autre_organisation() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO audit_log (org_id, actor_kind, actor_staff_id, action, object_type, object_id) VALUES
               ('{ORG_A}', 'support', gen_random_uuid(), 'support.record_viewed', 'tenant', 't1'),
               ('{ORG_B}', 'support', gen_random_uuid(), 'support.record_viewed', 'tenant', 'b1');
             INSERT INTO support_grants (id, org_id, reason, granted_by, granted_at, expires_at)
               VALUES ('bbbbbbbb-0000-4000-8000-000000000001', '{ORG_B}', 'Chez B', '55555555-0000-4000-8000-000000000015', now(), now() + interval '24 hours');"
        )))
        .await
        .unwrap();
    let list = req(&a, &g, Method::GET, "/api/support-grants", None, &[]).await;
    assert_eq!(
        list.json()["grants"],
        json!([]),
        "les accords de B ne sont pas visibles de A"
    );
    assert_eq!(
        list.json()["accesses"].as_array().unwrap().len(),
        1,
        "les consultations de l'organisation seule"
    );
    assert_eq!(list.json()["accesses"][0]["actor_label"], "Équipe HABEK");
    // révoquer l'accord de B depuis A : 404, ligne chez l'appelant, rien chez B
    reauth(&a, &g).await;
    let r = req(
        &a,
        &g,
        Method::POST,
        "/api/support-grants/bbbbbbbb-0000-4000-8000-000000000001/revoke",
        None,
        &[],
    )
    .await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::NOT_FOUND, json!("resource.not_found"))
    );
    assert!(
        actions(&a, ORG_A)
            .await
            .contains(&"security.cross_org_attempt".to_string())
    );
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM support_grants WHERE org_id = '{ORG_B}' AND revoked_at IS NULL")
        )
        .await,
        1,
        "l'accord de B est intact"
    );
    assert!(
        !actions(&a, ORG_B)
            .await
            .contains(&"security.cross_org_attempt".to_string())
    );
    a.db.drop_db().await;
}

// ───────── journal d'audit ─────────

/// Insère des lignes à des instants choisis : le déclencheur impose `app_now()`, qu'un réglage de transaction pilote ici.
async fn insert_audit_at(a: &common::Api, org: &str, actor: &str, action: &str, object_type: &str, at: &str, n: usize) {
    a.db.owner
        .execute(
            "CREATE OR REPLACE FUNCTION app_now() RETURNS timestamptz LANGUAGE sql STABLE
             AS $$ SELECT coalesce(nullif(current_setting('test.now', true), '')::timestamptz, now()) $$",
        )
        .await
        .unwrap();
    let mut tx = a.db.admin.begin().await.unwrap();
    sqlx::query("SELECT set_config('test.now', $1, true)")
        .bind(at)
        .execute(&mut *tx)
        .await
        .unwrap();
    for i in 0..n {
        sqlx::query(
            "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id, ip)
             VALUES ($1::uuid, 'user', $2::uuid, $3, $4, $5, '10.9.9.9')",
        )
        .bind(org)
        .bind(actor)
        .bind(action)
        .bind(object_type)
        .bind(format!("o{i}"))
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn filtres_pagination_et_bornes_du_journal() {
    let a = api(fast_params(), "0.0.0").await;
    // 61 lignes (60 + la connexion du test) : 3 acteurs, 3 types d'objet, 3 mois, dont 10 au même instant par lot
    let six = "66666666-0000-4000-8000-000000000016";
    insert_audit_at(
        &a,
        ORG_A,
        UN,
        "payment.recorded",
        "payment",
        "2026-09-10 08:00:00+00",
        20,
    )
    .await;
    insert_audit_at(
        &a,
        ORG_A,
        DEUX,
        "receipt.issued",
        "receipt",
        "2026-10-05 00:00:00+00",
        10,
    )
    .await; // borne basse
    insert_audit_at(
        &a,
        ORG_A,
        DEUX,
        "receipt.issued",
        "receipt",
        "2026-10-05 23:59:59+00",
        10,
    )
    .await; // borne haute
    insert_audit_at(&a, ORG_A, six, "tenant.created", "tenant", "2026-10-06 00:00:00+00", 10).await; // hors de la borne
    insert_audit_at(
        &a,
        ORG_A,
        UN,
        "payment.recorded",
        "payment",
        "2026-10-31 12:00:00+00",
        10,
    )
    .await;
    insert_audit_at(
        &a,
        ORG_B,
        UN,
        "payment.recorded",
        "payment",
        "2026-10-05 12:00:00+00",
        7,
    )
    .await; // autre organisation
    let g = sess(&a, "70 00 00 11").await;
    let get = |query: &str| {
        let path = format!("/api/audit{query}");
        let (a, g) = (&a, &g);
        async move { req(a, g, Method::GET, &path, None, &[]).await }
    };
    let n = |r: &common::Reply| r.json()["items"].as_array().unwrap().len();
    let mut seen = std::collections::HashSet::new();
    let mut cursor = String::new();
    let mut sizes = vec![];
    loop {
        let r = get(&format!("?limit=25{cursor}")).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        sizes.push(n(&r));
        for item in r.json()["items"].as_array().unwrap() {
            assert!(seen.insert(item["id"].as_i64().unwrap()), "doublon");
        }
        match r.json()["next_cursor"].as_str() {
            Some(c) => cursor = format!("&cursor={c}"),
            None => break,
        }
    }
    assert_eq!(sizes, vec![25, 25, 11], "61 lignes : pages de 25, 25 et 11");
    assert_eq!(
        seen.len(),
        61,
        "aucune ligne perdue, aucune doublée, rien de l'autre organisation"
    );
    // ordre : du plus récent au plus ancien, ex æquo par identifiant décroissant
    let all = get("?limit=100").await;
    let ids: Vec<i64> = all.json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids.len(), 61);
    assert_eq!(
        all.json()["items"][0]["action"],
        "auth.login",
        "la plus récente est la connexion du test"
    );
    // filtres : valeurs exactes
    assert_eq!(n(&get("?action=payment.recorded&limit=100").await), 30);
    assert_eq!(n(&get("?object_type=receipt").await), 20);
    assert_eq!(n(&get(&format!("?actor_id={DEUX}")).await), 20);
    assert_eq!(n(&get("?actor_id=66666666-0000-4000-8000-000000000016").await), 10);
    assert_eq!(
        n(&get("?object_id=o3").await),
        5,
        "un objet o3 par lot de 10 : cinq lots"
    );
    assert_eq!(n(&get("?actor_kind=support").await), 0);
    // bornes incluses, au fuseau de l'organisation : le 05/10 va de 00:00:00 à 23:59:59 ; le 06/10 00:00:00 n'en fait pas partie
    assert_eq!(
        n(&get("?from=2026-10-05&to=2026-10-05").await),
        20,
        "00:00:00 et 23:59:59 inclus"
    );
    assert_eq!(n(&get("?from=2026-10-06&to=2026-10-06").await), 10);
    assert_eq!(
        n(
            &get("?from=2026-10-01&to=2026-10-31&action=receipt.issued&actor_id=22222222-0000-4000-8000-000000000012")
                .await
        ),
        20,
        "filtres combinés par ET"
    );
    // valeurs mal formées : jamais ignorées en silence
    for bad in [
        "?from=05/10/2026",
        "?to=2026-13-01",
        "?action=n'importe",
        "?actor_kind=robot",
        "?actor_id=x",
        "?limit=abc",
        "?from=2026-10-06&to=2026-10-05",
    ] {
        let r = get(bad).await;
        assert_eq!(
            (r.status, r.json()["code"].clone()),
            (StatusCode::UNPROCESSABLE_ENTITY, json!("validation.failed")),
            "{bad}"
        );
    }
    // plus de 100 demandées : plafonné, sans erreur ; curseur altéré : 400
    assert_eq!(n(&get("?limit=500").await), 61);
    let first_page = get("?limit=5").await;
    let cursor = first_page.json()["next_cursor"].as_str().unwrap().to_string();
    let bad = get(&format!("?limit=5&cursor={cursor}x")).await;
    assert_eq!(
        (bad.status, bad.json()["code"].clone()),
        (StatusCode::BAD_REQUEST, json!("request.cursor_invalid"))
    );
    // la liste d'acteurs alimente le filtre « utilisateur »
    assert!(
        first_page.json()["meta"]["actors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["name"] == "Six")
    );
    // B voit les siennes seulement
    let b = on_org(&a, "un.gerant@agence.bf", ORG_B, "10.34.1.1").await;
    let rb = req(&a, &b, Method::GET, "/api/audit?limit=100", None, &[]).await;
    assert!(
        rb.json()["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["action"] != "receipt.issued"),
        "B ne lit rien de A"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn l_adresse_ip_se_lit_par_les_roles_prevus_et_les_secrets_sont_retires() {
    let a = api(fast_params(), "0.0.0").await;
    insert_audit_at(
        &a,
        ORG_A,
        UN,
        "payment.recorded",
        "payment",
        "2026-10-05 12:00:00+00",
        1,
    )
    .await;
    insert_audit_at(
        &a,
        ORG_C,
        "44444444-0000-4000-8000-000000000014",
        "payment.recorded",
        "payment",
        "2026-10-05 12:00:00+00",
        1,
    )
    .await;
    let ip_of = |r: &common::Reply| {
        r.json()["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["action"] == "payment.recorded")
            .unwrap()
            .clone()
    };
    let manager = sess(&a, "70 00 00 11").await;
    assert_eq!(
        ip_of(&req(&a, &manager, Method::GET, "/api/audit", None, &[]).await)["ip"],
        "10.9.9.9",
        "gérant"
    );
    let accountant = sess(&a, "70 00 00 16").await;
    assert_eq!(
        ip_of(&req(&a, &accountant, Method::GET, "/api/audit", None, &[]).await)["ip"],
        "10.9.9.9",
        "comptable"
    );
    // le propriétaire direct lit aussi l'adresse IP (DP-88, question 6 de F03 §8.7)
    add_member(
        &a,
        "88888888-0000-4000-8000-000000000003",
        "+22670000023",
        ORG_C,
        "owner",
    )
    .await;
    let owner = sess(&a, "70 00 00 23").await;
    assert_eq!(
        ip_of(&req(&a, &owner, Method::GET, "/api/audit", None, &[]).await)["ip"],
        "10.9.9.9",
        "propriétaire direct"
    );
    // le collaborateur du plan Propriétaire n'a pas `audit.read`
    let collaborator = sess(&a, "70 00 00 14").await;
    let denied = req(&a, &collaborator, Method::GET, "/api/audit", None, &[]).await;
    assert_eq!(
        (denied.status, denied.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.forbidden"))
    );
    let cashier = on_org(&a, "70 00 00 12", ORG_A, "10.34.2.1").await;
    assert_eq!(
        req(&a, &cashier, Method::GET, "/api/audit", None, &[]).await.status,
        StatusCode::FORBIDDEN
    );

    // une ligne ancienne qui contiendrait par erreur une clé interdite : la lecture la retire
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO audit_log (org_id, actor_kind, actor_user_id, action, object_type, object_id, after)
             VALUES ('{ORG_A}', 'user', '{UN}', 'settings.updated', 'settings', 'x', '{{\"ok\": 1, \"Password\": \"p\", \"a\": {{\"csrf_token\": \"c\", \"keep\": true}}, \"b\": [{{\"totp_secret\": \"s\"}}]}}')"
        )))
        .await
        .unwrap();
    let r = req(
        &a,
        &manager,
        Method::GET,
        "/api/audit?action=settings.updated",
        None,
        &[],
    )
    .await;
    let after = r.json()["items"][0]["after"].clone();
    assert_eq!(after, json!({ "ok": 1, "a": { "keep": true }, "b": [{}] }));
    assert!(
        !r.body.to_lowercase().contains("password")
            && !r.body.contains("csrf_token")
            && !r.body.contains("totp_secret")
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn le_journal_ne_se_modifie_pas() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let before = count(&a, "SELECT count(*) FROM audit_log").await;
    for method in [Method::PUT, Method::PATCH, Method::DELETE] {
        let r = req(&a, &g, method.clone(), "/api/audit/1", Some(json!({})), &[]).await;
        assert_eq!(
            (r.status, r.json()["code"].clone()),
            (StatusCode::METHOD_NOT_ALLOWED, json!("audit.immutable")),
            "{method}"
        );
        assert_eq!(r.header("allow"), "GET");
    }
    let log = actions(&a, ORG_A).await;
    assert_eq!(
        log.iter().filter(|x| *x == "security.permission_denied").count(),
        3,
        "une ligne par tentative chez l'appelant"
    );
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM audit_log WHERE action <> 'security.permission_denied' AND action <> 'auth.login'"
        )
        .await,
        0
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM audit_log").await,
        before + 3,
        "rien d'autre n'a changé"
    );
    // sans session : 401 d'abord
    let anon = call(&a.app, Method::DELETE, "/api/audit/1", IP, &[("origin", ORIGIN)], None).await;
    assert_eq!(anon.status, StatusCode::UNAUTHORIZED);
    // le droit SQL est la seconde barrière
    let e =
        a.db.app_pool(1)
            .await
            .execute("DELETE FROM audit_log")
            .await
            .unwrap_err();
    assert!(
        e.to_string().contains("permission")
            || e.to_string().contains("denied")
            || e.to_string().to_lowercase().contains("refus"),
        "{e}"
    );
    a.db.drop_db().await;
}

// ───────── matrice des droits sur les routes de l'étape (A3) ─────────

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

#[tokio::test(flavor = "multi_thread")]
async fn a3_chaque_route_de_l_etape_refuse_chaque_role_sans_le_droit() {
    let a = api(fast_params(), "0.0.0").await;
    add_member(
        &a,
        "88888888-0000-4000-8000-000000000001",
        "+22670000021",
        ORG_A,
        "property_manager",
    )
    .await;
    add_member(
        &a,
        "88888888-0000-4000-8000-000000000002",
        "+22670000022",
        ORG_A,
        "field_agent",
    )
    .await;
    add_member(
        &a,
        "88888888-0000-4000-8000-000000000003",
        "+22670000023",
        ORG_C,
        "owner",
    )
    .await;
    // un seul compte par rôle, chacun dans une seule organisation (sauf Deux : on choisit)
    let sessions: Vec<(Role, Plan, S)> = vec![
        (Role::Manager, Plan::Agency, sess(&a, "70 00 00 11").await),
        (Role::PropertyManager, Plan::Agency, sess(&a, "70 00 00 21").await),
        (
            Role::Cashier,
            Plan::Agency,
            on_org(&a, "70 00 00 12", ORG_A, "10.35.0.1").await,
        ),
        (Role::FieldAgent, Plan::Agency, sess(&a, "70 00 00 22").await),
        (Role::Accountant, Plan::Agency, sess(&a, "70 00 00 16").await),
        (Role::Owner, Plan::Owner, sess(&a, "70 00 00 23").await),
        (Role::Collaborator, Plan::Owner, sess(&a, "70 00 00 14").await),
    ];
    let unknown = "99999999-0000-4000-8000-000000000099";
    let mut checked = 0;
    for route in ROUTES {
        let Access::Action(action) = route.access else { continue };
        let path = route.path.replace("{membership_id}", unknown).replace("{id}", unknown);
        let method = Method::from_bytes(route.method.as_bytes()).unwrap();
        for (role, plan, s) in &sessions {
            let expected = authorize(*role, *plan, action);
            let allowed = matches!(
                expected,
                Decision::Allow | Decision::AllowScoped | Decision::AllowPartial
            );
            let r = req(&a, s, method.clone(), &path, Some(json!({})), &[("if-match", "1")]).await;
            let code = r.json()["code"].as_str().unwrap_or_default().to_string();
            if allowed {
                assert!(
                    code != "auth.forbidden" && code != "plan.feature_unavailable",
                    "{} {path} pour {role:?} : {code}",
                    route.method
                );
            } else {
                let want = if expected == Decision::Deny(DenyReason::PlanUnavailable) {
                    "plan.feature_unavailable"
                } else {
                    "auth.forbidden"
                };
                assert_eq!(
                    (r.status, code.as_str()),
                    (StatusCode::FORBIDDEN, want),
                    "{} {path} pour {role:?}",
                    route.method
                );
            }
            checked += 1;
        }
        let _ = (Action::UserManage, ORG_B);
    }
    assert_eq!(
        checked,
        7 * 23,
        "23 routes à droit (20 de l'étape 03, 3 d'observation), 7 rôles"
    );
    a.db.drop_db().await;
}

// ───────── performance de la lecture (F03 §8.1 point 9) ─────────

#[tokio::test(flavor = "multi_thread")]
#[allow(clippy::print_stdout)]
async fn journal_de_20000_lignes_filtre_par_periode_sous_150_ms_au_95e_centile() {
    let a = api(fast_params(), "0.0.0").await;
    // 20 000 lignes réparties sur neuf mois : le déclencheur d'horodatage est écarté pour poser des dates choisies
    a.db.admin
        .execute("ALTER TABLE audit_log DISABLE TRIGGER audit_before_insert")
        .await
        .unwrap();
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO audit_log (org_id, occurred_at, actor_kind, actor_user_id, action, object_type, object_id, ip)
             SELECT '{ORG_A}', timestamptz '2026-01-01 00:00:00+00' + (n * interval '20 minutes'), 'user', '{UN}',
                    'payment.recorded', 'payment', 'o' || n, '10.9.9.9' FROM generate_series(1, 20000) n"
        )))
        .await
        .unwrap();
    a.db.admin.execute("ANALYZE audit_log").await.unwrap();
    let g = sess(&a, "70 00 00 11").await;
    let mut times = vec![];
    for i in 0..40 {
        let path = format!("/api/audit?from=2026-03-01&to=2026-03-{:02}&limit=25", 10 + i % 20);
        let started = std::time::Instant::now();
        let r = req(&a, &g, Method::GET, &path, None, &[]).await;
        times.push(started.elapsed());
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        assert_eq!(r.json()["items"].as_array().unwrap().len(), 25);
    }
    times.sort();
    let p95 = times[(times.len() * 95 / 100).min(times.len() - 1)];
    println!(
        "lecture du journal, 20 000 lignes, filtre par période : p95 = {p95:?}, médiane = {:?}",
        times[times.len() / 2]
    );
    assert!(p95 < std::time::Duration::from_millis(150), "p95 = {p95:?}");
    a.db.drop_db().await;
}
