//! Étape 04 (F39-observation, DP-90) : observation des usages, erreurs et signalements, sur le routeur réel et la base réelle.
//!
//! Couvre les critères 1 à 5 : signalement (ligne, référence, contexte, alerte), aucun nom, numéro ni jeton, événement inconnu
//! écarté et compté (`204` en production, `422` en essai), erreur serveur agrégée par empreinte, isolation et ajout seul.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod common;

use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::{ORG_A, ORG_B, Reply, S, api, api_with, call, count, fast_params, req, sess};
use immo_app::observation::{feedback, retention};
use immo_app::platform::db::Db;
use immo_app::platform::jobs;
use serde_json::{Value, json};
use sqlx::Executor;

const MANAGER_A: &str = "70 00 00 11";
const CASHIER_A: &str = "70 00 00 12";
const ACCOUNTANT_A: &str = "70 00 00 16";
const MANAGER_B_EMAIL: &str = "un.gerant@agence.bf";

fn code(e: &sqlx::Error) -> String {
    e.as_database_error()
        .and_then(|d| d.code().map(|c| c.to_string()))
        .unwrap_or_else(|| e.to_string())
}

/// Le caissier appartient à A et à B : on ouvre sa session puis on choisit l'organisation A (nouveau cookie).
async fn cashier_a(a: &common::Api) -> S {
    let s = sess(a, CASHIER_A).await;
    let r = req(
        a,
        &s,
        Method::POST,
        "/api/me/organization",
        Some(json!({ "organization_id": ORG_A })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let cookie = r.session_cookie().unwrap().split(';').next().unwrap().to_string();
    S {
        cookie,
        csrf: r.json()["csrf_token"].as_str().unwrap().to_string(),
    }
}

/// Lit une valeur JSON (la colonne est lue en texte : le test n'active pas la fonction JSON de sqlx).
async fn jval(a: &common::Api, sql: &str) -> Value {
    let text: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT ({sql})::text")))
        .fetch_one(&a.db.admin)
        .await
        .unwrap();
    serde_json::from_str(&text).unwrap()
}

fn view(name: &str, screen: &str) -> Value {
    json!({ "name": name, "properties": { "screen": screen, "width_class": "mobile" }, "client_version": "1.0.0" })
}

async fn post_events(a: &common::Api, s: &S, events: Vec<Value>) -> Reply {
    req(
        a,
        s,
        Method::POST,
        "/api/events",
        Some(json!({ "events": events })),
        &[],
    )
    .await
}

/// Attend (au plus 5 s) qu'une requête de comptage atteigne la valeur attendue : les événements de connexion sont écrits en tâche
/// détachée, pour ne pas changer la durée de la réponse.
async fn wait_count(a: &common::Api, sql: &str, expected: i64) -> i64 {
    let mut n = count(a, sql).await;
    for _ in 0..50 {
        if n == expected {
            return n;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        n = count(a, sql).await;
    }
    n
}

fn feedback_body(description: &str) -> Value {
    json!({
        "category": "bug",
        "description": description,
        "can_call_back": true,
        "context": {
            "screen": "S05",
            "path": "/impayes?tel=70123456",
            "client_version": "1.0.0",
            "viewport": "390x844",
            "last_request_ids": ["INC-ABC123"],
            "last_error_code": "auth.forbidden"
        }
    })
}

// ───────── base : ajout seul, isolation, lecture par la console ─────────

#[tokio::test(flavor = "multi_thread")]
async fn product_events_ajout_seul_isolation_et_retention() {
    let a = api(fast_params(), "0.0.0").await;
    let admin = &a.db.admin;
    admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO product_events (org_id, name, properties) VALUES ('{ORG_A}', 'screen_viewed', '{{}}'), ('{ORG_B}', 'screen_viewed', '{{}}');
             INSERT INTO product_events (org_id, name, properties) VALUES ('{ORG_A}', 'screen_viewed', '{{\"vieux\": true}}');
             UPDATE product_events SET occurred_at = now() - interval '13 months 1 day' WHERE properties ? 'vieux';"
        )))
        .await
        .unwrap();
    let app = a.db.app_pool(2).await;
    let begin = |org: &'static str| {
        let app = app.clone();
        async move {
            let mut tx = app.begin().await.unwrap();
            sqlx::query("SELECT set_config('app.org_id', $1, true)")
                .bind(org)
                .execute(&mut *tx)
                .await
                .unwrap();
            tx
        }
    };

    // isolation : A ne voit pas B, ne peut pas écrire chez B
    let mut tx = begin(ORG_A).await;
    let seen: i64 = sqlx::query_scalar("SELECT count(*) FROM product_events")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(seen, 2, "les deux lignes de A, aucune de B");
    let e = sqlx::query("INSERT INTO product_events (org_id, name) VALUES ($1::uuid, 'screen_viewed')")
        .bind(ORG_B)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "écrire chez B est refusé : {e}");
    tx.rollback().await.unwrap();

    // ajout seul : ni UPDATE ni TRUNCATE ; DELETE d'une ligne récente sans effet, d'une ligne de plus de 13 mois accepté
    let mut tx = begin(ORG_A).await;
    let e = sqlx::query("UPDATE product_events SET name = 'autre_nom'")
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(code(&e), "42501", "UPDATE refusé : {e}");
    tx.rollback().await.unwrap();
    let e = app.execute("TRUNCATE product_events").await.unwrap_err();
    assert_eq!(code(&e), "42501", "TRUNCATE refusé à l'application : {e}");
    let e = a.db.owner.execute("TRUNCATE product_events").await.unwrap_err();
    assert_eq!(
        code(&e),
        "23001",
        "TRUNCATE refusé au propriétaire par déclencheur : {e}"
    );
    let mut tx = begin(ORG_A).await;
    let deleted = sqlx::query("DELETE FROM product_events WHERE occurred_at > now() - interval '1 day'")
        .execute(&mut *tx)
        .await
        .unwrap()
        .rows_affected();
    assert_eq!(deleted, 0, "une ligne récente ne se supprime pas");
    let deleted = sqlx::query("DELETE FROM product_events WHERE occurred_at < now() - interval '13 months'")
        .execute(&mut *tx)
        .await
        .unwrap()
        .rows_affected();
    assert_eq!(deleted, 1, "une ligne de 13 mois et 1 jour se supprime");
    tx.commit().await.unwrap();
    assert_eq!(count(&a, "SELECT count(*) FROM product_events").await, 2);

    // contraintes : nom en minuscules, taille des propriétés, pseudonyme de session
    for bad in [
        "INSERT INTO product_events (org_id, name) VALUES (current_org_id(), 'Mauvais Nom')",
        "INSERT INTO product_events (org_id, name, session_id) VALUES (current_org_id(), 'screen_viewed', 'pas-hexa')",
        "INSERT INTO product_events (org_id, name, properties) VALUES (current_org_id(), 'screen_viewed', '[]')",
    ] {
        let mut tx = begin(ORG_A).await;
        let e = sqlx::query(bad).execute(&mut *tx).await.unwrap_err();
        assert_eq!(code(&e), "23514", "{bad} : {e}");
        tx.rollback().await.unwrap();
    }
    let big = "x".repeat(3000);
    let mut tx = begin(ORG_A).await;
    let e = sqlx::query(
        "INSERT INTO product_events (org_id, name, properties) VALUES (current_org_id(), 'screen_viewed', $1::jsonb)",
    )
    .bind(json!({ "k": big }).to_string())
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(code(&e), "23514", "propriétés de plus de 2 Kio refusées : {e}");
    tx.rollback().await.unwrap();

    // l'heure est celle de la base : une date envoyée est ignorée
    let mut tx = begin(ORG_A).await;
    let at: bool = sqlx::query_scalar(
        "WITH i AS (INSERT INTO product_events (org_id, name, occurred_at) VALUES (current_org_id(), 'screen_viewed', '2001-01-01') RETURNING occurred_at)
         SELECT occurred_at > now() - interval '1 minute' FROM i",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert!(at, "occurred_at imposé par la base");
    tx.rollback().await.unwrap();
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_console_lit_la_telemetrie_sans_accord_support_et_ne_change_que_le_suivi() {
    let a = api(fast_params(), "0.0.0").await;
    let u = "11111111-0000-4000-8000-000000000011";
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO product_events (org_id, name) VALUES ('{ORG_A}', 'screen_viewed'), ('{ORG_B}', 'screen_viewed');
             INSERT INTO error_reports (org_id, fingerprint, source) VALUES ('{ORG_A}', repeat('a', 64), 'server');
             INSERT INTO feedback_reports (org_id, user_id, category, body) VALUES ('{ORG_A}', '{u}', 'bug', 'Le bouton ne répond pas');"
        )))
        .await
        .unwrap();
    // aucun accord support : la console lit quand même les trois tables, toutes organisations confondues
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM product_events")
        .fetch_one(&a.db.platform)
        .await
        .unwrap();
    assert_eq!(n, 2);
    for t in ["error_reports", "feedback_reports"] {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {t}")))
            .fetch_one(&a.db.platform)
            .await
            .unwrap();
        assert_eq!(n, 1, "{t}");
    }
    // mais aucune autre table de l'organisation : toujours soumise à l'accord (R-009)
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&a.db.platform)
        .await
        .unwrap();
    assert_eq!(n, 0, "audit_log reste soumis à l'accord support");
    // la console n'insère rien dans la télémétrie
    let e =
        a.db.platform
            .execute(sqlx::AssertSqlSafe(format!(
                "INSERT INTO product_events (org_id, name) VALUES ('{ORG_A}', 'screen_viewed')"
            )))
            .await
            .unwrap_err();
    assert_eq!(code(&e), "42501", "{e}");
    // elle change le suivi d'un signalement, rien d'autre
    let updated =
        a.db.platform
            .execute("UPDATE feedback_reports SET status = 'in_progress', handled_at = now()")
            .await
            .unwrap()
            .rows_affected();
    assert_eq!(updated, 1);
    let e =
        a.db.platform
            .execute("UPDATE feedback_reports SET body = 'Réécrit par la console'")
            .await
            .unwrap_err();
    assert_eq!(code(&e), "42501", "le texte du signalement ne se modifie pas : {e}");
    let e =
        a.db.platform
            .execute("UPDATE feedback_reports SET status = 'inconnu'")
            .await
            .unwrap_err();
    assert_eq!(code(&e), "23514", "statut hors liste : {e}");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_purge_de_conservation_supprime_les_lignes_de_plus_de_13_mois_dans_toutes_les_organisations() {
    let a = api(fast_params(), "0.0.0").await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO product_events (org_id, name, properties) VALUES
               ('{ORG_A}', 'screen_viewed', '{{\"vieux\":1}}'), ('{ORG_B}', 'screen_viewed', '{{\"vieux\":1}}'), ('{ORG_A}', 'screen_viewed', '{{}}');
             INSERT INTO error_reports (org_id, fingerprint, source) VALUES ('{ORG_A}', repeat('a', 64), 'server'), ('{ORG_B}', repeat('b', 64), 'client');
             UPDATE product_events SET occurred_at = now() - interval '14 months' WHERE properties ? 'vieux';
             UPDATE error_reports SET last_seen_at = now() - interval '14 months' WHERE source = 'server';"
        )))
        .await
        .unwrap();
    let db = Db::connect(&a.db.app_url, 2).await.unwrap();
    let purged = retention::purge_expired(&db).await.unwrap();
    assert_eq!(purged, 3, "deux événements et une erreur de plus de 13 mois");
    assert_eq!(count(&a, "SELECT count(*) FROM product_events").await, 1);
    assert_eq!(count(&a, "SELECT count(*) FROM error_reports").await, 1);
    assert_eq!(retention::purge_expired(&db).await.unwrap(), 0, "rejouable");
    a.db.drop_db().await;
}

// ───────── POST /events ─────────

#[tokio::test(flavor = "multi_thread")]
async fn un_lot_valide_est_ecrit_avec_l_identite_de_la_session_et_aucune_donnee_du_corps() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    let r = post_events(
        &a,
        &s,
        vec![
            view("screen_viewed", "S05"),
            json!({ "name": "validation_error", "properties": { "screen": "S09", "field": "amount", "rule_code": "R-041" } }),
        ],
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM product_events WHERE org_id = '{ORG_A}' AND name IN ('screen_viewed', 'validation_error') AND role = 'cashier' AND user_id = '22222222-0000-4000-8000-000000000012' AND session_id ~ '^[0-9a-f]{{16}}$'")).await,
        2,
        "organisation, utilisateur, rôle et pseudonyme de session viennent de la session"
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE client_version = '1.0.0'").await,
        1,
        "la version vient de l'événement qui la porte"
    );
    // le jeton de session n'est nulle part dans la ligne
    let cookie_value = s.cookie.split('=').nth(1).unwrap().to_string();
    assert_eq!(
        count(&a, &format!("SELECT count(*) FROM product_events WHERE properties::text LIKE '%{cookie_value}%' OR session_id = '{cookie_value}'")).await,
        0
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_evenement_inconnu_est_ecarte_et_compte_204_en_production_422_en_essai() {
    let bad_batch = vec![
        view("screen_viewed", "S05"),
        json!({ "name": "payment_hacked", "properties": {} }),
        json!({ "name": "login_failed", "properties": { "reason": "bad_credentials" } }),
        json!({ "name": "validation_error", "properties": { "screen": "S09", "field": "Traoré", "rule_code": "R-041" } }),
        json!({ "name": "validation_error", "properties": { "screen": "S09", "field": "tel_70123456", "rule_code": "R-041" } }),
    ];
    // production : 204, l'événement valide est écrit, les autres comptés par motif (jamais le nom reçu)
    let prod = api_with(fast_params(), "0.0.0", false, false).await;
    let s = cashier_a(&prod).await;
    let r = post_events(&prod, &s, bad_batch.clone()).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
    assert_eq!(
        count(
            &prod,
            "SELECT count(*) FROM product_events WHERE name = 'screen_viewed'"
        )
        .await,
        1
    );
    assert_eq!(
        count(&prod, "SELECT count(*) FROM product_events WHERE name = 'event_discarded' AND properties = '{\"reason\":\"unknown_name\",\"count\":1}'::jsonb").await,
        1
    );
    assert_eq!(
        count(&prod, "SELECT count(*) FROM product_events WHERE name = 'event_discarded' AND properties = '{\"reason\":\"server_origin\",\"count\":1}'::jsonb").await,
        1
    );
    assert_eq!(
        count(&prod, "SELECT count(*) FROM product_events WHERE name = 'event_discarded' AND properties = '{\"reason\":\"bad_property\",\"count\":2}'::jsonb").await,
        1
    );
    for forbidden in ["payment_hacked", "Traor", "70123456"] {
        assert_eq!(
            count(&prod, &format!("SELECT count(*) FROM product_events WHERE properties::text LIKE '%{forbidden}%' OR name LIKE '%{forbidden}%'")).await,
            0,
            "le nom reçu n'est jamais stocké : {forbidden}"
        );
    }
    prod.db.drop_db().await;

    // essai : 422 pour que l'erreur se voie ; l'écart est compté quand même
    let essai = api_with(fast_params(), "0.0.0", true, false).await;
    let s = cashier_a(&essai).await;
    let r = post_events(&essai, &s, vec![json!({ "name": "payment_hacked", "properties": {} })]).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    assert_eq!(r.json()["code"], "validation.failed");
    assert_eq!(
        count(
            &essai,
            "SELECT count(*) FROM product_events WHERE name = 'event_discarded'"
        )
        .await,
        1
    );
    let ok = post_events(&essai, &s, vec![view("screen_viewed", "S01")]).await;
    assert_eq!(
        ok.status,
        StatusCode::NO_CONTENT,
        "un lot valide reste accepté en essai"
    );
    essai.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn limites_du_lot_vingt_evenements_huit_kio_et_debit() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    // 21 événements : les 20 premiers sont écrits, le 21ᵉ écarté et compté
    let batch: Vec<Value> = (0..21).map(|_| view("screen_viewed", "S01")).collect();
    let r = post_events(&a, &s, batch).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'screen_viewed'").await,
        20
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'event_discarded' AND properties->>'reason' = 'batch_overflow'").await,
        1
    );
    // corps de plus de 8 Kio : refusé
    let big = json!({ "events": [{ "name": "screen_viewed", "properties": { "screen": "S01", "width_class": "mobile", "x": "y".repeat(9000) } }] });
    let r = req(&a, &s, Method::POST, "/api/events", Some(big), &[]).await;
    assert_eq!(r.status, StatusCode::PAYLOAD_TOO_LARGE, "{}", r.body);
    assert_eq!(r.json()["code"], "request.too_large");
    // champ inconnu au niveau du corps : refusé
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/events",
        Some(json!({ "events": [], "extra": 1 })),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    assert_eq!(r.json()["code"], "request.unknown_field");
    // débit : 30 requêtes par minute et par session (deux déjà faites)
    let mut last = StatusCode::NO_CONTENT;
    for _ in 0..30 {
        last = post_events(&a, &s, vec![]).await.status;
        if last == StatusCode::TOO_MANY_REQUESTS {
            break;
        }
    }
    assert_eq!(
        last,
        StatusCode::TOO_MANY_REQUESTS,
        "la 31ᵉ requête de la minute est limitée"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn trois_routes_sans_session_401_avec_session_tous_les_roles_ont_le_droit() {
    let a = api(fast_params(), "0.0.0").await;
    for path in ["/api/events", "/api/client-errors", "/api/feedback"] {
        let r = call(
            &a.app,
            Method::POST,
            path,
            "10.30.0.9",
            &[("content-type", "application/json")],
            Some("{}"),
        )
        .await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{path} : {}", r.body);
    }
    // gérant, caissier, comptable, collaborateur (plan Propriétaire) : autorisés
    let sessions = [
        ("gérant", sess(&a, MANAGER_A).await),
        ("caissier", cashier_a(&a).await),
        ("comptable", sess(&a, ACCOUNTANT_A).await),
        ("collaborateur", sess(&a, "70 00 00 14").await),
    ];
    for (who, s) in &sessions {
        assert_eq!(
            post_events(&a, s, vec![view("screen_viewed", "S01")]).await.status,
            StatusCode::NO_CONTENT,
            "{who}"
        );
        let r = req(
            &a,
            s,
            Method::POST,
            "/api/feedback",
            Some(feedback_body("Un message de test")),
            &[],
        )
        .await;
        assert_eq!(r.status, StatusCode::CREATED, "{who} : {}", r.body);
    }
    // l'écriture exige l'en-tête anti-falsification
    let s = cashier_a(&a).await;
    let r = call(
        &a.app,
        Method::POST,
        "/api/events",
        "10.30.0.9",
        &[
            ("cookie", &s.cookie),
            ("origin", common::ORIGIN),
            ("content-type", "application/json"),
        ],
        Some("{\"events\":[]}"),
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(r.json()["code"], "auth.csrf_invalid");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn isolation_une_session_ne_voit_que_les_lignes_de_son_organisation() {
    let a = api(fast_params(), "0.0.0").await;
    let sa = sess(&a, MANAGER_A).await;
    let sb = sess(&a, MANAGER_B_EMAIL).await;
    post_events(&a, &sa, vec![view("screen_viewed", "S01")]).await;
    post_events(
        &a,
        &sb,
        vec![view("screen_viewed", "S02"), view("screen_viewed", "S03")],
    )
    .await;
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM product_events WHERE org_id = '{ORG_A}'")
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM product_events WHERE org_id = '{ORG_B}'")
        )
        .await,
        2
    );
    a.db.drop_db().await;
}

// ───────── POST /client-errors ─────────

#[tokio::test(flavor = "multi_thread")]
async fn une_erreur_du_front_est_nettoyee_agregee_et_sans_pile() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    let payload = |n: &str| {
        json!({
            "message": format!("TypeError: échec pour +22670123456 et traore@agence.bf avec 0123456789abcdef0123456789abcdef {n}"),
            "screen": "S05",
            "client_version": "1.0.0",
            "request_ids": ["INC-ABC123", "pas un code"]
        })
    };
    for _ in 0..2 {
        let r = req(&a, &s, Method::POST, "/api/client-errors", Some(payload("")), &[]).await;
        assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
    }
    assert_eq!(
        count(&a, "SELECT count(*) FROM error_reports WHERE source = 'client'").await,
        1,
        "même empreinte"
    );
    assert_eq!(
        count(
            &a,
            "SELECT occurrences::bigint FROM error_reports WHERE source = 'client'"
        )
        .await,
        2
    );
    for forbidden in ["22670123456", "traore", "@", "0123456789abcdef"] {
        assert_eq!(
            count(
                &a,
                &format!("SELECT count(*) FROM error_reports WHERE sample::text LIKE '%{forbidden}%'")
            )
            .await,
            0,
            "{forbidden}"
        );
    }
    assert_eq!(
        count(&a, "SELECT count(*) FROM error_reports WHERE sample->'request_ids' = '[\"INC-ABC123\"]'::jsonb AND sample ? 'message' AND NOT sample ? 'stack'").await,
        1
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'client_error' AND properties = '{\"screen\":\"S05\",\"code\":\"TypeError\"}'::jsonb").await,
        2
    );
    a.db.drop_db().await;
}

// ───────── POST /feedback ─────────

#[tokio::test(flavor = "multi_thread")]
async fn un_signalement_cree_la_ligne_la_reference_le_contexte_l_evenement_et_la_tache_d_alerte() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    let description = "Le bouton « Encaisser » ne répond plus. Appelez-moi au +22670123456 (jeton SECRET-ABC).";
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body(description)),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let reference = r.json()["reference"].as_str().unwrap().to_string();
    assert!(reference.starts_with("SIG-") && reference.len() == 10, "{reference}");
    assert!(reference[4..].bytes().all(|c| c.is_ascii_digit()));

    // la ligne : texte tel quel, contexte minimal et propre
    let row = jval(&a, "SELECT to_jsonb(f) FROM feedback_reports f").await;
    assert_eq!(row["reference"], reference);
    assert_eq!(row["body"], description, "texte stocké tel quel");
    assert_eq!(row["callback_ok"], true);
    assert_eq!(row["status"], "new");
    let ctx = &row["context"];
    assert_eq!(ctx["screen"], "S05");
    assert_eq!(
        ctx["path"], "/impayes",
        "ni paramètre de requête ni numéro dans l'adresse"
    );
    assert_eq!(ctx["viewport"], "390x844");
    assert_eq!(ctx["last_request_ids"], json!(["INC-ABC123"]));
    assert_eq!(ctx["last_error_code"], "auth.forbidden");
    assert_eq!(ctx["role"], "cashier");
    assert!(!ctx.to_string().contains("70123456"));

    // l'événement ne porte ni description ni numéro
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'feedback_submitted' AND properties = '{\"screen\":\"S05\",\"category\":\"bug\"}'::jsonb").await,
        1
    );
    assert_eq!(count(&a, "SELECT count(*) FROM product_events WHERE properties::text LIKE '%Encaisser%' OR properties::text LIKE '%SECRET%'").await, 0);

    // la tâche d'alerte est dans la file, créée dans la même transaction
    assert_eq!(
        count(
            &a,
            &format!(
                "SELECT count(*) FROM jobs WHERE type = '{}' AND org_id = '{ORG_A}' AND state = 'queued'",
                feedback::NOTIFY_JOB
            )
        )
        .await,
        1
    );
    // l'alerte part : référence, catégorie, écran, nature de l'organisation, rappel ; jamais la description
    let db = Db::connect(&a.db.app_url, 2).await.unwrap();
    let job = jobs::claim(&db, a.clock.as_ref())
        .await
        .unwrap()
        .expect("tâche de notification");
    assert_eq!(job.job_type, feedback::NOTIFY_JOB);
    feedback::notify(&db, Some(&a.db.platform_url), &job).await.unwrap();
    let detail = jval(
        &a,
        "SELECT detail FROM platform_events WHERE action = 'feedback.received'",
    )
    .await;
    assert_eq!(
        detail,
        json!({ "reference": reference, "category": "bug", "screen": "S05", "org_kind": "real", "callback_ok": true })
    );
    let text = detail.to_string();
    for forbidden in ["Encaisser", "70123456", "SECRET", "Agence A", "Deux"] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn signalement_validation_idempotence_debit_et_isolation() {
    let a = api(fast_params(), "0.0.0").await;
    // la limite compte les tentatives : chaque partie du test a son propre utilisateur
    let s = sess(&a, ACCOUNTANT_A).await;
    // trop court, catégorie inconnue, caractères de contrôle seuls
    for body in [feedback_body("abc"), feedback_body("     "), {
        let mut b = feedback_body("Un message assez long");
        b["category"] = json!("autre");
        b
    }] {
        let r = req(&a, &s, Method::POST, "/api/feedback", Some(body), &[]).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
        assert_eq!(r.json()["code"], "validation.failed");
    }
    assert_eq!(
        count(&a, "SELECT count(*) FROM feedback_reports").await,
        0,
        "rien n'est créé"
    );
    // 4 001 caractères : refusé ; 4 000 : accepté
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body(&"a".repeat(4001))),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body(&"é".repeat(4000))),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);

    // idempotence : la même clé donne le même signalement
    let s = sess(&a, MANAGER_A).await;
    let key = "5b3e6a1c-0d4f-4c2e-9a7b-1f2e3d4c5b6a";
    let first = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body("Message avec une clé")),
        &[("idempotency-key", key)],
    )
    .await;
    let again = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body("Message avec une clé")),
        &[("idempotency-key", key)],
    )
    .await;
    assert_eq!(first.status, StatusCode::CREATED);
    assert_eq!(again.status, StatusCode::CREATED);
    assert_eq!(first.json()["reference"], again.json()["reference"]);
    assert_eq!(
        count(
            &a,
            "SELECT count(*) FROM feedback_reports WHERE body = 'Message avec une clé'"
        )
        .await,
        1
    );
    let bad = req(
        &a,
        &s,
        Method::POST,
        "/api/feedback",
        Some(feedback_body("Message avec une clé")),
        &[("idempotency-key", "pas-un-uuid")],
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);

    // débit : 5 par heure et par utilisateur
    let s = sess(&a, "70 00 00 14").await;
    let mut limited = None;
    for _ in 0..6 {
        let r = req(
            &a,
            &s,
            Method::POST,
            "/api/feedback",
            Some(feedback_body("Encore un message")),
            &[],
        )
        .await;
        if r.status == StatusCode::TOO_MANY_REQUESTS {
            limited = Some(r);
            break;
        }
    }
    let limited = limited.expect("le 6ᵉ signalement de l'heure est limité");
    assert_eq!(limited.json()["code"], "rate_limit.exceeded");

    // isolation : le gérant de B n'a aucun accès aux signalements de A (RLS) ; ses signalements restent chez B
    let sb = sess(&a, MANAGER_B_EMAIL).await;
    let r = req(
        &a,
        &sb,
        Method::POST,
        "/api/feedback",
        Some(feedback_body("Un signalement chez B")),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED);
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM feedback_reports WHERE org_id = '{ORG_B}'")
        )
        .await,
        1
    );
    a.db.drop_db().await;
}

// ───────── erreurs serveur ─────────

#[tokio::test(flavor = "multi_thread")]
async fn une_erreur_serveur_provoquee_apparait_une_fois_avec_son_nombre_d_occurrences() {
    let a = api_with(fast_params(), "0.0.0", true, true).await;
    let s = cashier_a(&a).await;
    let r = req(&a, &s, Method::GET, "/api/_test/server-error", None, &[]).await;
    assert_eq!(r.status, StatusCode::INTERNAL_SERVER_ERROR);
    let incident = r.json()["request_id"].as_str().unwrap().to_string();
    assert_eq!(
        count(&a, "SELECT count(*) FROM error_reports WHERE source = 'server'").await,
        1
    );
    assert_eq!(
        count(
            &a,
            "SELECT occurrences::bigint FROM error_reports WHERE source = 'server'"
        )
        .await,
        1,
        "1 occurrence"
    );
    let r2 = req(&a, &s, Method::GET, "/api/_test/server-error", None, &[]).await;
    assert_eq!(r2.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        count(&a, "SELECT count(*) FROM error_reports WHERE source = 'server'").await,
        1,
        "toujours une ligne"
    );
    assert_eq!(
        count(
            &a,
            "SELECT occurrences::bigint FROM error_reports WHERE source = 'server'"
        )
        .await,
        2,
        "2 occurrences"
    );

    // l'exemple : identifiant de requête du dernier incident, modèle de route, pile de notre code ; ni corps, ni paramètres
    let sample = jval(&a, "SELECT sample FROM error_reports WHERE source = 'server'").await;
    assert_eq!(sample["route"], "/api/_test/server-error");
    assert_eq!(sample["status"], 500);
    assert_ne!(
        sample["request_id"].as_str().unwrap(),
        incident,
        "le dernier exemple remplace le précédent"
    );
    assert!(
        sample["frames"].as_array().is_some_and(|f| !f.is_empty()),
        "pile : {sample}"
    );
    for key in ["body", "params", "headers", "query", "message"] {
        assert!(sample.get(key).is_none(), "{key} ne doit pas figurer : {sample}");
    }

    // une autre organisation a sa propre ligne
    let sb = sess(&a, MANAGER_B_EMAIL).await;
    req(&a, &sb, Method::GET, "/api/_test/server-error", None, &[]).await;
    assert_eq!(count(&a, "SELECT count(*) FROM error_reports").await, 2);
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM error_reports WHERE org_id = '{ORG_B}'")
        )
        .await,
        1
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_route_d_essai_n_existe_pas_en_production() {
    let a = api_with(fast_params(), "0.0.0", false, false).await;
    let s = cashier_a(&a).await;
    let r = req(&a, &s, Method::GET, "/api/_test/server-error", None, &[]).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.body);
    assert_eq!(count(&a, "SELECT count(*) FROM error_reports").await, 0);
    a.db.drop_db().await;
}

// ───────── événements du serveur ─────────

#[tokio::test(flavor = "multi_thread")]
async fn un_refus_de_droit_ecrit_permission_denied_avec_le_role_et_le_code_du_droit() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    let r = req(&a, &s, Method::GET, "/api/audit", None, &[]).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'permission_denied' AND properties = '{\"role\":\"cashier\",\"action\":\"audit.read\"}'::jsonb AND role = 'cashier'").await,
        1
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn login_failed_seulement_pour_un_compte_connu_d_une_seule_organisation() {
    let a = api(fast_params(), "0.0.0").await;
    let attempt = |who: &'static str| {
        let app = a.app.clone();
        async move {
            call(
                &app,
                Method::POST,
                "/api/auth/login",
                "10.50.0.1",
                &[("content-type", "application/json"), ("origin", common::ORIGIN)],
                Some(&common::login_json(who, "mauvais mot de passe")),
            )
            .await
        }
    };
    // gérant de A seulement : l'événement est écrit chez A
    assert_eq!(attempt(MANAGER_A).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        wait_count(&a, &format!("SELECT count(*) FROM product_events WHERE org_id = '{ORG_A}' AND name = 'login_failed' AND properties = '{{\"reason\":\"bad_credentials\"}}'::jsonb"), 1).await,
        1
    );
    // membre de deux organisations, ou compte inconnu : rien n'est écrit (la friction reste lisible dans login_attempts)
    assert_eq!(attempt(CASHIER_A).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(attempt("70 99 99 99").await.status, StatusCode::UNAUTHORIZED);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'login_failed'").await,
        1
    );
    // aucun identifiant ni mot de passe dans la ligne
    assert_eq!(count(&a, "SELECT count(*) FROM product_events WHERE properties::text LIKE '%mauvais%' OR properties::text LIKE '%70%0%'").await, 0);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn session_expired_est_ecrit_une_seule_fois() {
    let a = api(fast_params(), "0.0.0").await;
    let s = cashier_a(&a).await;
    // le caissier a deux organisations : session sans organisation active, donc rien à observer ; on prend le comptable de A
    let s = {
        let _ = s;
        sess(&a, ACCOUNTANT_A).await
    };
    a.clock.advance(chrono::Duration::hours(3));
    let r = req(&a, &s, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{}", r.body);
    let again = req(&a, &s, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'session_expired' AND properties = '{\"reason\":\"idle\"}'::jsonb AND role = 'accountant'").await,
        1,
        "une seule ligne malgré deux requêtes avec le cookie échu"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_demande_de_reinitialisation_ecrit_son_evenement_sans_identite_de_la_cible() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, MANAGER_A).await;
    common::reauth(&a, &g).await;
    let members = req(&a, &g, Method::GET, "/api/users", None, &[]).await;
    assert_eq!(members.status, StatusCode::OK, "{}", members.body);
    let list = members.json();
    let target = list["users"]
        .as_array()
        .and_then(|users| users.iter().find(|m| m["role"] == "cashier"))
        .expect("un caissier");
    let id = target["membership_id"].as_str().unwrap().to_string();
    let r = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/users/{id}/password-resets"),
        None,
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(
        count(&a, "SELECT count(*) FROM product_events WHERE name = 'password_reset_requested' AND properties = '{\"channel\":\"admin_link\"}'::jsonb").await,
        1
    );
    a.db.drop_db().await;
}

// ───────── indicateurs (ops/sql/indicateurs.sql) ─────────

#[tokio::test(flavor = "multi_thread")]
async fn les_indicateurs_s_executent_donnent_les_bons_chiffres_et_excluent_la_demonstration() {
    let a = api(fast_params(), "0.0.0").await;
    // B devient une organisation de démonstration : exclue des indicateurs
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE organizations SET kind = 'demo' WHERE id = '{ORG_B}';
             INSERT INTO product_events (org_id, user_id, name, properties) VALUES
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'screen_viewed', '{{\"screen\":\"S05\",\"width_class\":\"mobile\"}}'),
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'screen_viewed', '{{\"screen\":\"S05\",\"width_class\":\"mobile\"}}'),
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'screen_viewed', '{{\"screen\":\"S09\",\"width_class\":\"desktop\"}}'),
               ('{ORG_B}', '55555555-0000-4000-8000-000000000015', 'screen_viewed', '{{\"screen\":\"S99\",\"width_class\":\"mobile\"}}'),
               ('{ORG_A}', NULL, 'payment_recorded', '{{\"duration_ms\":20000,\"entry_delay_days\":0}}'),
               ('{ORG_A}', NULL, 'payment_recorded', '{{\"duration_ms\":30000,\"entry_delay_days\":2}}'),
               ('{ORG_A}', NULL, 'payment_recorded', '{{\"duration_ms\":40000,\"entry_delay_days\":5}}'),
               ('{ORG_B}', NULL, 'payment_recorded', '{{\"duration_ms\":999999,\"entry_delay_days\":0}}'),
               ('{ORG_A}', NULL, 'payment_form_abandoned', '{{\"step\":\"amount\"}}'),
               ('{ORG_A}', NULL, 'event_discarded', '{{\"reason\":\"unknown_name\",\"count\":4}}');
             INSERT INTO error_reports (org_id, fingerprint, source, route, occurrences) VALUES
               ('{ORG_A}', repeat('a', 64), 'server', '/api/x', 3), ('{ORG_B}', repeat('b', 64), 'server', '/api/x', 9);
             INSERT INTO feedback_reports (org_id, user_id, category, body) VALUES
               ('{ORG_A}', '11111111-0000-4000-8000-000000000011', 'bug', 'Un signalement ouvert'),
               ('{ORG_B}', '55555555-0000-4000-8000-000000000015', 'bug', 'Un signalement de démonstration');"
        )))
        .await
        .unwrap();

    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ops/sql/indicateurs.sql");
    let text = std::fs::read_to_string(file).unwrap();
    let queries: Vec<String> = text
        .split(";\n")
        .map(|chunk| {
            chunk
                .lines()
                .filter(|l| !l.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string()
        })
        .filter(|q| !q.is_empty())
        .collect();
    assert_eq!(queries.len(), 8, "huit requêtes fixes");

    // chaque requête s'exécute avec le rôle de la console ; on garde les lignes en JSON pour les relire
    let mut results: Vec<Vec<Value>> = vec![];
    for q in &queries {
        let sql = format!("SELECT coalesce(jsonb_agg(to_jsonb(t)), '[]'::jsonb)::text FROM ({q}) t");
        let text: String = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&a.db.platform)
            .await
            .unwrap();
        results.push(serde_json::from_str::<Vec<Value>>(&text).unwrap());
    }
    // 2. écrans : S05 (2) puis S09 (1) ; l'écran S99 de la démonstration n'apparaît pas
    let screens: Vec<(String, i64)> = results[1]
        .iter()
        .map(|r| {
            (
                r["ecran"].as_str().unwrap().to_string(),
                r["consultations"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(screens, vec![("S05".to_string(), 2), ("S09".to_string(), 1)]);
    // 3. médiane 30 000 ms (la démonstration, 999 999, est exclue)
    assert_eq!(results[2].len(), 1);
    assert_eq!(results[2][0]["paiements"], 3);
    assert_eq!(results[2][0]["mediane_ms"].as_f64(), Some(30000.0));
    // 4. 2 paiements sur 3 saisis sous 48 h
    assert_eq!(results[3][0]["sous_48_h"], 2);
    assert_eq!(results[3][0]["part_pct"].as_f64(), Some(66.7));
    // 5. un abandon à l'étape « amount »
    assert_eq!(results[4][0]["derniere_etape"], "amount");
    // 6. 3 occurrences (les 9 de la démonstration sont exclues)
    assert_eq!(results[5].len(), 1);
    assert_eq!(results[5][0]["occurrences"], 3);
    // 7. un signalement ouvert (celui de la démonstration est exclu)
    assert_eq!(results[6][0]["signalements"], 1);
    // 8. quatre événements écartés
    assert_eq!(results[7][0]["ecartes"], 4);
    // aucune colonne ne porte un nom, un numéro ou un texte de signalement
    let all = serde_json::to_string(&results).unwrap();
    for forbidden in ["Agence A", "Famille", "Un signalement", "+226", "11111111"] {
        assert!(!all.contains(forbidden), "{forbidden}");
    }
    a.db.drop_db().await;
}
