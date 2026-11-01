//! Commandes d'administration `immo user create` et `immo org create` (F01 règle 10, DP-68). Aucun écran, aucune inscription publique.
//!
//! - `immo user create --phone +22670123456 | --email x@y --name "Nom affiché" [--password-file F] [--demo]` : sans fichier de mot de
//!   passe, un mot de passe provisoire est engendré et affiché **une seule fois**. **Refusé si `APP_ENV=production`** : le premier gérant
//!   de production arrive par invitation (étape 03).
//! - `immo org create --name "Agence" --plan agency|owner --user <téléphone ou e-mail> [--kind real|demo|test]` : organisation en phase
//!   `setup` ; le compte désigné en devient le premier membre (gérant d'une agence, propriétaire du plan Propriétaire).
//!
//! Le mot de passe n'est jamais lu d'un argument de ligne de commande (historique du shell) ni journalisé.

use std::io::Write;
use std::process::ExitCode;

use immo_app::identity::admin::{self, AdminError, NewUser};
use immo_app::identity::authorize::Plan;
use immo_app::identity::password::{HashParams, PasswordService};
use immo_app::platform::config::{Config, Environment};
use immo_app::platform::db::Db;
use immo_app::platform::ops;

fn out(text: &str) {
    let _ = std::io::stdout().write_all(text.as_bytes());
}

fn fail(message: &str) -> ExitCode {
    tracing::error!("{message}");
    ExitCode::FAILURE
}

/// Valeur d'une option `--nom valeur`.
fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

/// La création d'un compte en ligne de commande est interdite en production.
pub fn user_creation_allowed(environment: Environment) -> bool {
    environment != Environment::Production
}

async fn event(action: &str, detail: serde_json::Value) {
    if let Ok(url) = std::env::var("PLATFORM_DATABASE_URL")
        && !url.trim().is_empty()
        && let Err(e) = ops::record_event(&url, action, &detail).await
    {
        tracing::warn!(error = %e, "événement de plateforme non écrit");
    }
}

pub async fn user_create(args: &[String]) -> ExitCode {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => return fail(&format!("configuration invalide : {e}")),
    };
    if !user_creation_allowed(config.environment) {
        return fail(
            "refusé : la création d'un compte en ligne de commande est interdite en production (le premier gérant arrive par invitation)",
        );
    }
    let identifier = match (option(args, "--phone"), option(args, "--email")) {
        (Some(p), None) => p,
        (None, Some(e)) => e,
        _ => {
            return fail(
                "usage : immo user create (--phone <+22670123456> | --email <adresse>) --name <nom affiché> [--password-file <fichier>] [--demo]",
            );
        }
    };
    let Some(name) = option(args, "--name") else {
        return fail("--name requis");
    };
    let (password, generated) = match option(args, "--password-file") {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(text) => (text.lines().next().unwrap_or("").trim_end().to_string(), false),
            Err(_) => return fail("fichier du mot de passe illisible"),
        },
        None => match admin::generate_password() {
            Ok(p) => (p, true),
            Err(e) => return fail(&e.to_string()),
        },
    };
    let db = match Db::connect(config.database_url.expose(), 2).await {
        Ok(db) => db,
        Err(e) => return fail(&format!("connexion à la base impossible : {e}")),
    };
    let passwords = match PasswordService::new(argon_params()) {
        Ok(p) => p,
        Err(e) => return fail(&e.to_string()),
    };
    let new = NewUser {
        identifier: identifier.to_string(),
        display_name: name.to_string(),
        password: password.clone(),
        is_demo: flag(args, "--demo"),
    };
    match admin::create_user(&db, &passwords, new).await {
        Ok(id) => {
            event("user.created", serde_json::json!({ "user_id": id.to_string() })).await;
            out(&format!("compte créé : {id}\n"));
            if generated {
                out(&format!(
                    "mot de passe provisoire (affiché une seule fois) : {password}\n"
                ));
            }
            ExitCode::SUCCESS
        }
        Err(e) => fail(&describe(&e)),
    }
}

pub async fn org_create(args: &[String]) -> ExitCode {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => return fail(&format!("configuration invalide : {e}")),
    };
    let (Some(name), Some(plan), Some(user)) = (option(args, "--name"), option(args, "--plan"), option(args, "--user"))
    else {
        return fail(
            "usage : immo org create --name <nom> --plan agency|owner --user <téléphone ou e-mail du premier membre> [--kind real|demo|test]",
        );
    };
    let Some(plan) = Plan::parse(plan) else {
        return fail("--plan : agency ou owner");
    };
    let kind = option(args, "--kind").unwrap_or("real");
    let db = match Db::connect(config.database_url.expose(), 2).await {
        Ok(db) => db,
        Err(e) => return fail(&format!("connexion à la base impossible : {e}")),
    };
    match admin::create_organization(&db, name, plan, kind, user).await {
        Ok(id) => {
            event(
                "org.created",
                serde_json::json!({ "org_id": id.to_string(), "plan": plan.as_str(), "kind": kind }),
            )
            .await;
            out(&format!("organisation créée : {id} (phase setup)\n"));
            ExitCode::SUCCESS
        }
        Err(e) => fail(&describe(&e)),
    }
}

fn argon_params() -> HashParams {
    let defaults = HashParams::default();
    let get = |name: &str, default: u32| {
        std::env::var(name)
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(default)
    };
    HashParams {
        memory_kib: get("ARGON2_MEMORY_KIB", defaults.memory_kib),
        iterations: get("ARGON2_ITERATIONS", defaults.iterations),
        parallelism: get("ARGON2_PARALLELISM", defaults.parallelism),
    }
}

fn describe(e: &AdminError) -> String {
    // les messages d'AdminError ne citent ni mot de passe ni identifiant
    e.to_string()
}

/// `immo password calibrate [--memory-kib N] [--iterations N] [--parallelism N]` : mesure la durée d'une vérification argon2id sur
/// CETTE machine (décision n° 48 : viser 100 à 500 ms sur le serveur d'essai). Affiche les cinq mesures et la médiane.
pub async fn password_calibrate(args: &[String]) -> ExitCode {
    let defaults = HashParams::default();
    let get = |name: &str, default: u32| option(args, name).and_then(|v| v.parse().ok()).unwrap_or(default);
    let params = HashParams {
        memory_kib: get("--memory-kib", defaults.memory_kib),
        iterations: get("--iterations", defaults.iterations),
        parallelism: get("--parallelism", defaults.parallelism),
    };
    let service = match PasswordService::new(params) {
        Ok(s) => s,
        Err(e) => return fail(&e.to_string()),
    };
    let Ok(hash) = service.hash("mot de passe de mesure").await else {
        return fail("calcul impossible");
    };
    let mut times = vec![];
    for _ in 0..5 {
        let started = std::time::Instant::now();
        if !matches!(service.verify("mot de passe de mesure", Some(&hash)).await, Ok(true)) {
            return fail("vérification inattendue");
        }
        times.push(started.elapsed().as_millis());
    }
    times.sort_unstable();
    let median = times.get(2).copied().unwrap_or(0);
    out(&format!(
        "argon2id m={} Kio t={} p={} : mesures {:?} ms, médiane {median} ms ({})\n",
        params.memory_kib,
        params.iterations,
        params.parallelism,
        times,
        if (100..=500).contains(&median) {
            "dans la cible 100–500 ms"
        } else {
            "HORS de la cible 100–500 ms"
        }
    ));
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_creation_de_compte_est_interdite_en_production_seulement() {
        assert!(user_creation_allowed(Environment::Development));
        assert!(user_creation_allowed(Environment::Staging));
        assert!(!user_creation_allowed(Environment::Production));
    }

    #[test]
    fn lecture_des_options() {
        let args: Vec<String> = ["--name", "Mme Kaboré", "--demo", "--plan", "agency"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(option(&args, "--name"), Some("Mme Kaboré"));
        assert_eq!(option(&args, "--plan"), Some("agency"));
        assert_eq!(option(&args, "--phone"), None);
        assert!(flag(&args, "--demo") && !flag(&args, "--absent"));
    }
}
