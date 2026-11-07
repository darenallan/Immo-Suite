//! Étape 03, API : invitations, confirmation du mot de passe, rôles et droits réglables, désactivation, dernier gérant, lots assignés
//! (F01 §9.2, §9.6, F02 §7). Cas chiffrés : invitation du 01/11 à 10:00 acceptable jusqu'au 08/11 à 09:59:59.

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
    GOOD, IP, ORG_A, ORG_B, ORG_C, ORIGIN, Reply, S, actions, api, call, count, fast_params, login, reauth, req, sess,
    token_of,
};
use serde_json::{Value, json};
use sqlx::Executor;

// ───────── invitations : cas chiffré n° 1 et parcours du caissier ─────────

#[tokio::test(flavor = "multi_thread")]
async fn invitation_valable_sept_jours_a_la_seconde_pres_et_a_usage_unique() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let created = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 12 34 56", "role": "cashier" })),
        &[],
    )
    .await;
    assert_eq!(created.status, StatusCode::OK, "{}", created.body);
    let url = created.json()["url"].as_str().unwrap().to_string();
    assert!(url.starts_with("https://app.test/invitation/"), "{url}");
    let token = token_of(&url);
    assert_eq!(token.len(), 43);
    // le jeton n'est jamais stocké, ni renvoyé par la liste
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM invitations WHERE encode(token_hash, 'escape') LIKE '%{token}%'")
        )
        .await,
        0
    );
    let list = req(&a, &g, Method::GET, "/api/invitations", None, &[]).await;
    assert!(!list.body.contains(&token), "la liste ne renvoie jamais le lien");
    assert_eq!(list.json()["invitations"][0]["status"], "pending");

    let preview = |a: &common::Api| {
        let path = format!("/api/invitations/{token}");
        let app = a.app.clone();
        async move { call(&app, Method::GET, &path, "10.30.0.9", &[], None).await }
    };
    // 7 jours moins une seconde : encore valable
    a.clock.advance(Duration::days(7) - Duration::seconds(1));
    let ok = preview(&a).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(ok.json()["organization_name"], "Agence A");
    assert_eq!(ok.json()["role"], "cashier");
    assert_eq!(ok.json()["identifier_exists"], false);
    // à 7 jours exactement : expirée (même corps que révoquée ou utilisée)
    a.clock.advance(Duration::seconds(1));
    let gone = preview(&a).await;
    assert_eq!(
        (gone.status, gone.json()["code"].clone()),
        (StatusCode::GONE, json!("invitation.expired"))
    );
    assert_eq!(
        gone.json()["detail"],
        "Cette invitation n'est plus valable. Demandez au gérant de la renvoyer."
    );
    let g = sess(&a, "70 00 00 11").await; // la session de 12 heures a expiré depuis
    let list = req(&a, &g, Method::GET, "/api/invitations", None, &[]).await;
    assert_eq!(list.json()["invitations"][0]["status"], "expired");
    // jeton inconnu : 404
    let unknown = call(
        &a.app,
        Method::GET,
        &format!("/api/invitations/{}", "a".repeat(43)),
        "10.30.0.9",
        &[],
        None,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn le_caissier_invite_choisit_son_mot_de_passe_et_n_a_que_ses_droits() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let created = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 12 34 56", "role": "cashier" })),
        &[],
    )
    .await;
    let token = token_of(created.json()["url"].as_str().unwrap());
    let accept = |body: Value| {
        let app = a.app.clone();
        async move {
            call(
                &app,
                Method::POST,
                "/api/invitations/accept",
                "10.30.0.8",
                &[("content-type", "application/json")],
                Some(&body.to_string()),
            )
            .await
        }
    };
    // mot de passe courant ou trop court : refusé, l'invitation reste valable
    let weak = accept(json!({ "token": token, "display_name": "Awa", "password": "court" })).await;
    assert_eq!(
        (
            weak.status,
            weak.json()["code"].clone(),
            weak.json()["params"]["reason"].clone()
        ),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!("auth.password_weak"),
            json!("too_short")
        )
    );
    let phone = accept(json!({ "token": token, "display_name": "Awa", "password": "70123456abc" })).await;
    assert_eq!(
        phone.json()["params"]["reason"],
        "contains_identifier",
        "cas chiffré n° 5"
    );
    let missing = accept(json!({ "token": token })).await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
    // acceptation
    let ok = accept(json!({ "token": token, "display_name": "Awa", "password": "un bon mot de passe" })).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    let cookie = ok.session_cookie().expect("session ouverte");
    let pair = cookie.split(';').next().unwrap().to_string();
    let csrf = ok.json()["csrf_token"].as_str().unwrap().to_string();
    let s = S { cookie: pair, csrf };
    let me = req(&a, &s, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(me.json()["membership"]["role"], "cashier");
    let perms: Vec<String> = me.json()["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_string())
        .collect();
    assert!(perms.contains(&"payment.record".to_string()));
    assert!(
        !perms.contains(&"user.manage".to_string()),
        "le caissier n'administre rien"
    );
    // le caissier ne voit ni les utilisateurs ni le journal : 403 côté serveur
    for path in [
        "/api/users",
        "/api/invitations",
        "/api/audit",
        "/api/support-grants",
        "/api/role-settings",
    ] {
        let r = req(&a, &s, Method::GET, path, None, &[]).await;
        assert_eq!(
            (r.status, r.json()["code"].clone()),
            (StatusCode::FORBIDDEN, json!("auth.forbidden")),
            "{path}"
        );
    }
    // usage unique, même avec le bon mot de passe
    let again = accept(json!({ "token": token, "display_name": "Awa", "password": "un bon mot de passe" })).await;
    assert_eq!(
        (again.status, again.json()["code"].clone()),
        (StatusCode::GONE, json!("invitation.expired"))
    );
    // il peut se reconnecter avec ce mot de passe
    let (_, _) = login(&a.app, "70 12 34 56", "un bon mot de passe", "10.30.0.7").await;
    // audit : invitation, acceptation, connexion
    let log = actions(&a, ORG_A).await;
    assert!(
        log.contains(&"user.invited".to_string()) && log.contains(&"membership.accepted".to_string()),
        "{log:?}"
    );
    assert_eq!(log.iter().filter(|x| *x == "user.invited").count(), 1);
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn deux_acceptations_simultanees_une_seule_reussit() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let created = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 12 34 56", "role": "cashier" })),
        &[],
    )
    .await;
    let token = token_of(created.json()["url"].as_str().unwrap());
    let body = json!({ "token": token, "display_name": "Awa", "password": "un bon mot de passe" }).to_string();
    let (r1, r2) = tokio::join!(
        call(
            &a.app,
            Method::POST,
            "/api/invitations/accept",
            "10.30.0.5",
            &[("content-type", "application/json")],
            Some(&body)
        ),
        call(
            &a.app,
            Method::POST,
            "/api/invitations/accept",
            "10.30.0.6",
            &[("content-type", "application/json")],
            Some(&body)
        ),
    );
    let mut statuses = vec![r1.status, r2.status];
    statuses.sort();
    assert_eq!(
        statuses,
        vec![StatusCode::OK, StatusCode::GONE],
        "{} / {}",
        r1.body,
        r2.body
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM users WHERE login_phone = '+22670123456'").await,
        1
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn regles_d_invitation_doublons_roles_hors_plan_gerant_et_renvoi() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let post = |body: Value| req(&a, &g, Method::POST, "/api/invitations", Some(body), &[]);
    // déjà membre : on réactive, on n'invite pas ; un compte d'une AUTRE organisation est invité sans le révéler
    let member = post(json!({ "phone": "70 00 00 12", "role": "cashier" })).await;
    assert_eq!(
        (member.status, member.json()["code"].clone()),
        (StatusCode::CONFLICT, json!("auth.identifier_taken"))
    );
    let other_org = post(json!({ "email": "un.gerant@agence.bf", "role": "cashier" })).await;
    assert_eq!(other_org.status, StatusCode::OK, "{}", other_org.body);
    // identifiant invalide, rôle hors plan, owner, lots non vides
    let bad = post(json!({ "phone": "1234", "role": "cashier" })).await;
    assert_eq!(bad.json()["errors"][0]["code"], "validation.phone_format");
    for role in ["collaborator", "owner", "chef"] {
        let r = post(json!({ "phone": "70 55 55 55", "role": role })).await;
        assert_eq!(r.json()["code"], "validation.failed", "{role}");
        assert_eq!(r.json()["params"]["reason"], "role_not_in_plan", "{role}");
    }
    let units = post(
        json!({ "phone": "70 55 55 55", "role": "field_agent", "unit_ids": ["a0000000-0000-4000-8000-000000000001"] }),
    )
    .await;
    assert_eq!(units.json()["params"]["reason"], "units_unavailable");
    let both = post(json!({ "phone": "70 55 55 55", "email": "x@y.bf", "role": "cashier" })).await;
    assert_eq!(both.json()["code"], "validation.failed");
    // une invitation en attente : « Renvoyez-la »
    let first = post(json!({ "phone": "70 66 66 66", "role": "cashier" })).await;
    assert_eq!(first.status, StatusCode::OK);
    let dup = post(json!({ "phone": "70 66 66 66", "role": "accountant" })).await;
    assert_eq!(
        (dup.status, dup.json()["code"].clone()),
        (StatusCode::CONFLICT, json!("invitation.already_pending"))
    );
    // inviter un gérant exige la confirmation du mot de passe, avant tout effet
    let before = count(&a, "SELECT count(*) FROM invitations").await;
    let manager = post(json!({ "phone": "70 77 77 77", "role": "manager" })).await;
    assert_eq!(
        (manager.status, manager.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.reauth_required"))
    );
    assert_eq!(
        count(&a, "SELECT count(*) FROM invitations").await,
        before,
        "aucun effet sans confirmation"
    );
    reauth(&a, &g).await;
    assert_eq!(
        post(json!({ "phone": "70 77 77 77", "role": "manager" })).await.status,
        StatusCode::OK
    );

    // renvoi : l'ancien lien meurt, le nouveau vit, la trace garde le lien entre les deux
    let old_token = token_of(first.json()["url"].as_str().unwrap());
    let id = first.json()["invitation"]["id"].as_str().unwrap().to_string();
    let renewed = req(&a, &g, Method::POST, &format!("/api/invitations/{id}/renew"), None, &[]).await;
    assert_eq!(renewed.status, StatusCode::OK, "{}", renewed.body);
    let new_token = token_of(renewed.json()["url"].as_str().unwrap());
    assert_ne!(old_token, new_token);
    let old = call(
        &a.app,
        Method::GET,
        &format!("/api/invitations/{old_token}"),
        IP,
        &[],
        None,
    )
    .await;
    assert_eq!(old.status, StatusCode::GONE, "l'ancien lien est révoqué");
    let new = call(
        &a.app,
        Method::GET,
        &format!("/api/invitations/{new_token}"),
        IP,
        &[],
        None,
    )
    .await;
    assert_eq!(new.status, StatusCode::OK);
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM invitations WHERE renewed_from = '{id}'")
        )
        .await,
        1
    );
    // révocation
    let new_id = renewed.json()["invitation"]["id"].as_str().unwrap().to_string();
    let revoked = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/invitations/{new_id}/revoke"),
        None,
        &[],
    )
    .await;
    assert_eq!(revoked.json()["status"], "revoked");
    assert_eq!(
        call(
            &a.app,
            Method::GET,
            &format!("/api/invitations/{new_token}"),
            IP,
            &[],
            None
        )
        .await
        .status,
        StatusCode::GONE
    );
    let again = req(
        &a,
        &g,
        Method::POST,
        &format!("/api/invitations/{new_id}/renew"),
        None,
        &[],
    )
    .await;
    assert_eq!(again.json()["params"]["reason"], "not_renewable");
    let log = actions(&a, ORG_A).await;
    assert!(
        log.contains(&"invitation.renewed".to_string()) && log.contains(&"invitation.revoked".to_string()),
        "{log:?}"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn une_invitation_d_un_compte_existant_exige_sa_connexion_prealable() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    // « Trois » (+22670000013) a un compte mais aucune organisation
    let created = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 00 00 13", "role": "accountant" })),
        &[],
    )
    .await;
    let token = token_of(created.json()["url"].as_str().unwrap());
    let preview = call(&a.app, Method::GET, &format!("/api/invitations/{token}"), IP, &[], None).await;
    assert_eq!(preview.json()["identifier_exists"], true);
    let json = [("content-type", "application/json")];
    let body = json!({ "token": token }).to_string();
    // sans session : « Voulez-vous vous connecter ? »
    let anon = call(&a.app, Method::POST, "/api/invitations/accept", IP, &json, Some(&body)).await;
    assert_eq!(
        (anon.status, anon.json()["code"].clone()),
        (StatusCode::CONFLICT, json!("auth.identifier_taken"))
    );
    // la session d'un AUTRE compte est refusée
    let other = sess(&a, "70 00 00 16").await;
    let headers = [
        ("cookie", other.cookie.as_str()),
        ("origin", ORIGIN),
        ("content-type", "application/json"),
    ];
    let wrong = call(
        &a.app,
        Method::POST,
        "/api/invitations/accept",
        IP,
        &headers,
        Some(&body),
    )
    .await;
    assert_eq!(
        (wrong.status, wrong.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.forbidden"))
    );
    // une session sans origine autorisée est refusée comme toute écriture
    let no_origin = call(
        &a.app,
        Method::POST,
        "/api/invitations/accept",
        IP,
        &[("cookie", other.cookie.as_str()), ("content-type", "application/json")],
        Some(&body),
    )
    .await;
    assert_eq!(no_origin.json()["code"], "auth.csrf_invalid");
    // la connexion de « Trois » a échoué plus tôt faute d'adhésion : on l'ajoute ailleurs pour qu'il ait une session
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "INSERT INTO memberships (org_id, user_id, role) VALUES ('{ORG_B}', '33333333-0000-4000-8000-000000000013', 'cashier')"
        )))
        .await
        .unwrap();
    let third = sess(&a, "70 00 00 13").await;
    let headers = [
        ("cookie", third.cookie.as_str()),
        ("origin", ORIGIN),
        ("content-type", "application/json"),
    ];
    let joined = call(
        &a.app,
        Method::POST,
        "/api/invitations/accept",
        IP,
        &headers,
        Some(&body),
    )
    .await;
    assert_eq!(joined.status, StatusCode::OK, "{}", joined.body);
    assert_eq!(joined.json()["organizations"].as_array().unwrap().len(), 2);
    assert_eq!(count(&a, &format!("SELECT count(*) FROM memberships WHERE org_id = '{ORG_A}' AND role = 'accountant' AND user_id = '33333333-0000-4000-8000-000000000013'")).await, 1);
    // compte désactivé : refus comme une connexion refusée
    a.db.admin
        .execute("UPDATE users SET status = 'disabled' WHERE id = '66666666-0000-4000-8000-000000000016'")
        .await
        .unwrap();
    let c2 = req(
        &a,
        &g,
        Method::POST,
        "/api/invitations",
        Some(json!({ "phone": "70 00 00 16", "role": "cashier" })),
        &[],
    )
    .await;
    assert_eq!(
        c2.json()["code"],
        "auth.identifier_taken",
        "Six est déjà comptable de A"
    );
    a.db.drop_db().await;
}

// ───────── confirmation du mot de passe ─────────

#[tokio::test(flavor = "multi_thread")]
async fn la_confirmation_vaut_cinq_minutes_pas_une_seconde_de_plus() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let target = "/api/users/00000000-0000-4000-8000-000000000000/enable";
    // sans confirmation : refus AVANT toute recherche de la cible
    let r = req(&a, &g, Method::POST, target, None, &[]).await;
    assert_eq!(
        (r.status, r.json()["code"].clone()),
        (StatusCode::FORBIDDEN, json!("auth.reauth_required"))
    );
    // mauvais mot de passe : 401, compté dans la limitation de la connexion
    let bad = req(
        &a,
        &g,
        Method::POST,
        "/api/auth/reauth",
        Some(json!({ "password": "pas le bon" })),
        &[],
    )
    .await;
    assert_eq!(
        (bad.status, bad.json()["code"].clone()),
        (StatusCode::UNAUTHORIZED, json!("auth.invalid_credentials"))
    );
    reauth(&a, &g).await;
    a.clock.advance(Duration::minutes(4) + Duration::seconds(59));
    let r = req(&a, &g, Method::POST, target, None, &[]).await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "4 min 59 s : la confirmation vaut encore, la cible n'existe pas : {}",
        r.body
    );
    a.clock.advance(Duration::seconds(2));
    let r = req(&a, &g, Method::POST, target, None, &[]).await;
    assert_eq!(r.json()["code"], "auth.reauth_required", "5 min 01 s");
    // l'ouverture de session ne vaut pas confirmation : une nouvelle connexion doit confirmer aussi
    let fresh = sess(&a, "70 00 00 11").await;
    let r = req(&a, &fresh, Method::POST, target, None, &[]).await;
    assert_eq!(r.json()["code"], "auth.reauth_required");
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn cinq_mauvais_mots_de_passe_a_la_confirmation_bloquent_comme_a_la_connexion() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    a.clock.advance(Duration::seconds(1)); // les échecs comptent depuis la dernière réussite, strictement
    for _ in 0..5 {
        let r = req(
            &a,
            &g,
            Method::POST,
            "/api/auth/reauth",
            Some(json!({ "password": "pas le bon" })),
            &[],
        )
        .await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    }
    let blocked = req(
        &a,
        &g,
        Method::POST,
        "/api/auth/reauth",
        Some(json!({ "password": GOOD })),
        &[],
    )
    .await;
    assert_eq!(
        (blocked.status, blocked.json()["code"].clone()),
        (StatusCode::TOO_MANY_REQUESTS, json!("auth.rate_limited"))
    );
    a.db.drop_db().await;
}

// ───────── rôles, droits réglables, désactivation ─────────

fn permissions(me: &Reply) -> Vec<String> {
    me.json()["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_string())
        .collect()
}

async fn membership_id(a: &common::Api, org: &str, user: &str) -> String {
    sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(format!(
        "SELECT id::text FROM memberships WHERE org_id = '{org}' AND user_id = '{user}'"
    )))
    .fetch_one(&a.db.admin)
    .await
    .unwrap()
}

const DEUX: &str = "22222222-0000-4000-8000-000000000012";
const UN: &str = "11111111-0000-4000-8000-000000000011";

#[tokio::test(flavor = "multi_thread")]
async fn le_role_change_s_applique_sans_reconnexion_et_sans_revoquer_la_session() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    // « Deux » est caissier de A et de B : on le connecte, il choisit A
    let (cookie, _) = login(&a.app, "70 00 00 12", GOOD, "10.30.0.20").await;
    let chosen = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        "10.30.0.20",
        &[
            ("cookie", &cookie),
            ("origin", ORIGIN),
            ("content-type", "application/json"),
            ("x-csrf-token", &a.svc.csrf_for(cookie.split('=').nth(1).unwrap())),
        ],
        Some(&json!({ "organization_id": ORG_A }).to_string()),
    )
    .await;
    assert_eq!(chosen.status, StatusCode::OK, "{}", chosen.body);
    let pair = chosen.session_cookie().unwrap().split(';').next().unwrap().to_string();
    let csrf = chosen.json()["csrf_token"].as_str().unwrap().to_string();
    let deux = S { cookie: pair, csrf };
    let me = req(&a, &deux, Method::GET, "/api/me", None, &[]).await;
    assert!(!permissions(&me).contains(&"catalog.write".to_string()));

    let mid = membership_id(&a, ORG_A, DEUX).await;
    // sans confirmation : refusé, rien ne change
    let refused = req(
        &a,
        &g,
        Method::PATCH,
        &format!("/api/users/{mid}"),
        Some(json!({ "role": "property_manager" })),
        &[],
    )
    .await;
    assert_eq!(refused.json()["code"], "auth.reauth_required");
    reauth(&a, &g).await;
    let changed = req(
        &a,
        &g,
        Method::PATCH,
        &format!("/api/users/{mid}"),
        Some(json!({ "role": "property_manager" })),
        &[],
    )
    .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    assert_eq!(changed.json()["role"], "property_manager");
    // même session, requête suivante : les droits du gestionnaire (critère 5)
    let me = req(&a, &deux, Method::GET, "/api/me", None, &[]).await;
    assert_eq!(me.status, StatusCode::OK, "la session reste valable");
    assert_eq!(me.json()["membership"]["role"], "property_manager");
    assert!(permissions(&me).contains(&"catalog.write".to_string()));
    assert_eq!(count(&a, "SELECT count(*) FROM sessions WHERE revoked_at IS NOT NULL AND user_id = '22222222-0000-4000-8000-000000000012' AND org_id = 'a0000000-0000-4000-8000-00000000000a'").await, 0, "aucune session révoquée");
    let log = actions(&a, ORG_A).await;
    assert!(log.contains(&"membership.role_changed".to_string()));
    // rôle hors plan, owner : refusés ; le plan Propriétaire n'a aucun changement possible
    for bad in ["collaborator", "owner", "inconnu"] {
        let r = req(
            &a,
            &g,
            Method::PATCH,
            &format!("/api/users/{mid}"),
            Some(json!({ "role": bad })),
            &[],
        )
        .await;
        assert_eq!(r.json()["code"], "validation.failed", "{bad}");
    }
    // auto-retrait interdit
    let own = membership_id(&a, ORG_A, UN).await;
    let me_self = req(
        &a,
        &g,
        Method::PATCH,
        &format!("/api/users/{own}"),
        Some(json!({ "role": "cashier" })),
        &[],
    )
    .await;
    assert_eq!(
        (
            me_self.status,
            me_self.json()["code"].clone(),
            me_self.json()["detail"].clone()
        ),
        (
            StatusCode::FORBIDDEN,
            json!("auth.forbidden"),
            json!("Demandez à un autre gérant.")
        )
    );
    let me_self = req(&a, &g, Method::POST, &format!("/api/users/{own}/disable"), None, &[]).await;
    assert_eq!(me_self.status, StatusCode::FORBIDDEN);
    assert_eq!(
        count(
            &a,
            &format!("SELECT count(*) FROM memberships WHERE id = '{own}' AND status = 'active' AND role = 'manager'")
        )
        .await,
        1
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn les_droits_reglables_changent_la_decision_a_la_requete_suivante() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE memberships SET role = 'property_manager' WHERE org_id = '{ORG_A}' AND user_id = '{DEUX}'"
        )))
        .await
        .unwrap();
    let (cookie, _) = login(&a.app, "70 00 00 12", GOOD, "10.30.0.21").await;
    let csrf = a.svc.csrf_for(cookie.split('=').nth(1).unwrap());
    let chosen = call(
        &a.app,
        Method::POST,
        "/api/me/organization",
        "10.30.0.21",
        &[
            ("cookie", &cookie),
            ("origin", ORIGIN),
            ("content-type", "application/json"),
            ("x-csrf-token", &csrf),
        ],
        Some(&json!({ "organization_id": ORG_A }).to_string()),
    )
    .await;
    let deux = S {
        cookie: chosen.session_cookie().unwrap().split(';').next().unwrap().to_string(),
        csrf: chosen.json()["csrf_token"].as_str().unwrap().to_string(),
    };

    let list = req(&a, &g, Method::GET, "/api/role-settings", None, &[]).await;
    let items = list.json()["settings"].as_array().unwrap().clone();
    assert_eq!(items.len(), 3, "gestionnaire × encaisser, importer, exporter");
    assert!(
        items.iter().all(|i| i["allowed"] == false && i["version"] == 0),
        "défaut « non »"
    );
    let put = |allowed: bool, version: &'static str| {
        req_owned(
            &a,
            &g,
            Method::PUT,
            "/api/role-settings",
            Some(json!({ "role": "property_manager", "action": "record_payment", "allowed": allowed })),
            vec![("if-match", version)],
        )
    };
    let has = |me: Reply| permissions(&me).contains(&"payment.record".to_string());
    assert!(
        !has(req(&a, &deux, Method::GET, "/api/me", None, &[]).await),
        "défaut : le gestionnaire n'encaisse pas"
    );
    // If-Match absent : 428 ; version fausse : 412
    let none = req(
        &a,
        &g,
        Method::PUT,
        "/api/role-settings",
        Some(json!({ "role": "property_manager", "action": "record_payment", "allowed": true })),
        &[],
    )
    .await;
    assert_eq!(none.status, StatusCode::PRECONDITION_REQUIRED);
    assert_eq!(put(true, "\"5\"").await.status, StatusCode::PRECONDITION_FAILED);
    // « oui » : effet immédiat, même session
    let on = put(true, "\"0\"").await;
    assert_eq!(on.status, StatusCode::OK, "{}", on.body);
    assert_eq!(on.json()["version"], 1);
    assert!(
        has(req(&a, &deux, Method::GET, "/api/me", None, &[]).await),
        "oui : la requête suivante, sans reconnexion"
    );
    // « non » à nouveau
    let off = put(false, "\"1\"").await;
    assert_eq!(off.json()["version"], 2);
    assert!(!has(req(&a, &deux, Method::GET, "/api/me", None, &[]).await));
    // deux gérants simultanés : le second reçoit 412
    assert_eq!(put(true, "\"1\"").await.status, StatusCode::PRECONDITION_FAILED);
    // un couple non réglable : refusé
    let bad = req(
        &a,
        &g,
        Method::PUT,
        "/api/role-settings",
        Some(json!({ "role": "collaborator", "action": "import_data", "allowed": true })),
        &[("if-match", "0")],
    )
    .await;
    assert_eq!(bad.json()["params"]["reason"], "setting_not_available");
    let bad = req(
        &a,
        &g,
        Method::PUT,
        "/api/role-settings",
        Some(json!({ "role": "cashier", "action": "record_payment", "allowed": true })),
        &[("if-match", "0")],
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let log = actions(&a, ORG_A).await;
    assert_eq!(log.iter().filter(|x| *x == "settings.updated").count(), 2);
    a.db.drop_db().await;
}

async fn req_owned(
    a: &common::Api,
    s: &S,
    method: Method,
    path: &str,
    body: Option<Value>,
    extra: Vec<(&str, &str)>,
) -> Reply {
    req(a, s, method, path, body, &extra).await
}

#[tokio::test(flavor = "multi_thread")]
async fn desactiver_revoque_les_sessions_de_l_organisation_et_epargne_les_autres() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    // « Deux » ouvre une session sur A et une sur B
    let open = |org: &'static str, ip: &'static str| {
        let app = a.app.clone();
        let svc = a.svc.clone();
        async move {
            let (cookie, _) = login(&app, "70 00 00 12", GOOD, ip).await;
            let csrf = svc.csrf_for(cookie.split('=').nth(1).unwrap());
            let chosen = call(
                &app,
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
            S {
                cookie: chosen.session_cookie().unwrap().split(';').next().unwrap().to_string(),
                csrf: chosen.json()["csrf_token"].as_str().unwrap().to_string(),
            }
        }
    };
    let on_a = open(ORG_A, "10.30.0.31").await;
    let on_b = open(ORG_B, "10.30.0.32").await;
    let mid = membership_id(&a, ORG_A, DEUX).await;
    reauth(&a, &g).await;
    let off = req(&a, &g, Method::POST, &format!("/api/users/{mid}/disable"), None, &[]).await;
    assert_eq!(off.status, StatusCode::OK, "{}", off.body);
    assert_eq!(off.json()["status"], "disabled");
    assert_eq!(
        req(&a, &on_a, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::UNAUTHORIZED,
        "session de A révoquée"
    );
    assert_eq!(
        req(&a, &on_b, Method::GET, "/api/me", None, &[]).await.status,
        StatusCode::OK,
        "session de B intacte"
    );
    // il ne peut plus choisir A ; la liste le montre désactivé
    let users = req(&a, &g, Method::GET, "/api/users", None, &[]).await;
    let row = users.json()["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["membership_id"] == mid.as_str())
        .unwrap()
        .clone();
    assert_eq!(row["status"], "disabled");
    // réactiver exige la confirmation (valable 5 minutes, encore active ici)
    let on = req(&a, &g, Method::POST, &format!("/api/users/{mid}/enable"), None, &[]).await;
    assert_eq!(on.json()["status"], "active");
    a.clock.advance(Duration::minutes(6));
    let again = req(&a, &g, Method::POST, &format!("/api/users/{mid}/disable"), None, &[]).await;
    assert_eq!(again.json()["code"], "auth.reauth_required");
    let log = actions(&a, ORG_A).await;
    assert!(
        log.contains(&"user.disabled".to_string()) && log.contains(&"user.enabled".to_string()),
        "{log:?}"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn un_gerant_ne_touche_jamais_un_membre_d_une_autre_organisation() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let b_member = membership_id(&a, ORG_B, "55555555-0000-4000-8000-000000000015").await;
    reauth(&a, &g).await;
    for (method, path, body) in [
        (
            Method::PATCH,
            format!("/api/users/{b_member}"),
            Some(json!({ "role": "cashier" })),
        ),
        (Method::POST, format!("/api/users/{b_member}/disable"), None),
        (Method::POST, format!("/api/users/{b_member}/enable"), None),
        (Method::POST, format!("/api/users/{b_member}/password-resets"), None),
        (
            Method::PUT,
            format!("/api/users/{b_member}/unit-assignments"),
            Some(json!({ "unit_ids": [] })),
        ),
    ] {
        let r = req(&a, &g, method, &path, body, &[]).await;
        assert_eq!(
            (r.status, r.json()["code"].clone()),
            (StatusCode::NOT_FOUND, json!("resource.not_found")),
            "{path}"
        );
    }
    let log = actions(&a, ORG_A).await;
    assert_eq!(
        log.iter().filter(|x| *x == "security.cross_org_attempt").count(),
        5,
        "une ligne par tentative chez l'appelant : {log:?}"
    );
    assert!(
        !actions(&a, ORG_B)
            .await
            .contains(&"security.cross_org_attempt".to_string()),
        "rien chez l'autre"
    );
    let users = req(&a, &g, Method::GET, "/api/users", None, &[]).await;
    assert!(
        users.json()["users"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["user_id"] != "55555555-0000-4000-8000-000000000015"),
        "jamais un membre d'une autre organisation"
    );
    a.db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn lots_assignes_liste_vide_seulement_et_portee_de_l_agent() {
    let a = api(fast_params(), "0.0.0").await;
    let g = sess(&a, "70 00 00 11").await;
    let mid = membership_id(&a, ORG_A, DEUX).await;
    let path = format!("/api/users/{mid}/unit-assignments");
    let some = req(
        &a,
        &g,
        Method::PUT,
        &path,
        Some(json!({ "unit_ids": ["a0000000-0000-4000-8000-000000000001"] })),
        &[],
    )
    .await;
    assert_eq!(
        some.json()["params"]["reason"],
        "unit_ids",
        "étape 05 : un lot inconnu est refusé (DP-92)"
    );
    let empty = req(&a, &g, Method::PUT, &path, Some(json!({ "unit_ids": [] })), &[]).await;
    assert_eq!(empty.json()["unit_ids"], json!([]));

    // portée : 25 lots, 10 assignés à l'agent terrain
    a.db.admin
        .execute(sqlx::AssertSqlSafe(format!(
            "UPDATE memberships SET role = 'field_agent' WHERE org_id = '{ORG_A}' AND user_id = '{DEUX}';
             -- étape 05 : les 25 lots existent (clé unit_assignments → units)
             INSERT INTO properties (id, org_id, name) VALUES ('f0000000-0000-4000-8000-0000000000aa', '{ORG_A}', 'Bien d''essai');
             INSERT INTO units (id, org_id, property_id, reference, kind, usage)
               SELECT ('f0000000-0000-4000-8000-0000000000' || lpad(n::text, 2, '0'))::uuid, '{ORG_A}', 'f0000000-0000-4000-8000-0000000000aa',
                      'L' || n, 'room', 'residential' FROM generate_series(1, 25) n;
             INSERT INTO unit_assignments (org_id, unit_id, user_id, assigned_by)
               SELECT '{ORG_A}', ('f0000000-0000-4000-8000-0000000000' || lpad(n::text, 2, '0'))::uuid, '{DEUX}', '{UN}' FROM generate_series(1, 10) n;"
        )))
        .await
        .unwrap();
    let auth = {
        let (cookie, _) = login(&a.app, "70 00 00 12", GOOD, "10.30.0.40").await;
        let csrf = a.svc.csrf_for(cookie.split('=').nth(1).unwrap());
        let chosen = call(
            &a.app,
            Method::POST,
            "/api/me/organization",
            "10.30.0.40",
            &[
                ("cookie", &cookie),
                ("origin", ORIGIN),
                ("content-type", "application/json"),
                ("x-csrf-token", &csrf),
            ],
            Some(&json!({ "organization_id": ORG_A }).to_string()),
        )
        .await;
        let token = chosen
            .session_cookie()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .split('=')
            .nth(1)
            .unwrap()
            .to_string();
        a.svc.authenticate(&token).await.unwrap()
    };
    let scope = a.svc.scope_of(&auth).await.unwrap();
    match &scope {
        immo_app::identity::members::Scope::Units(units) => assert_eq!(units.len(), 10),
        other => panic!("portée attendue : 10 lots, obtenu {other:?}"),
    }
    // retrait de toutes les affectations
    let cleared = req(&a, &g, Method::PUT, &path, Some(json!({ "unit_ids": [] })), &[]).await;
    assert_eq!(cleared.status, StatusCode::OK);
    assert_eq!(
        a.svc.scope_of(&auth).await.unwrap(),
        immo_app::identity::members::Scope::Units(vec![])
    );
    // un gérant voit tout
    let manager = a.svc.authenticate(g.cookie.split('=').nth(1).unwrap()).await.unwrap();
    assert_eq!(
        a.svc.scope_of(&manager).await.unwrap(),
        immo_app::identity::members::Scope::All
    );
    let _ = ORG_C;
    a.db.drop_db().await;
}
