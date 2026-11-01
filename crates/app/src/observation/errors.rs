//! Suivi d'erreurs sans service tiers (F39-observation §2 règles 12 et 13, décision n° 28).
//!
//! Toute erreur non prévue est agrégée par **empreinte** : SHA-256 de la source, du modèle de route, du type d'erreur et des cinq
//! premiers cadres de la pile. Le message d'une bibliothèque n'entre jamais dans l'empreinte ni dans l'exemple (il peut citer une
//! valeur). Aucun corps de requête, aucun paramètre. Une erreur du front est nettoyée : suites de 6 chiffres, adresses électroniques,
//! jetons et numéros remplacés par « … », 200 caractères au plus, ni pile ni adresse.

// Requêtes non vérifiées du module, comme celles de `audit` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::platform::db::{DbError, OrgTx};

/// Empreintes distinctes nouvelles par organisation et par jour ; au-delà, seules les occurrences des existantes sont comptées.
pub const MAX_NEW_FINGERPRINTS_PER_DAY: i64 = 200;
/// Longueur maximale du message d'une erreur du front.
pub const MAX_CLIENT_MESSAGE_CHARS: usize = 200;
/// Cadres de pile gardés.
pub const MAX_FRAMES: usize = 5;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Empreinte SHA-256 (64 caractères hexadécimaux) des éléments donnés, séparés sans ambiguïté.
pub fn fingerprint(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.len().to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hex(&hasher.finalize())
}

fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '@' | '.' | '_' | '-')
}

fn sensitive_token(token: &str) -> bool {
    if super::events::looks_personal(token) {
        return true;
    }
    let hexa = token.len() >= 20 && token.chars().all(|c| c.is_ascii_hexdigit());
    hexa || token.chars().count() >= 32
}

/// Message d'une erreur du front rendu inoffensif : jetons, numéros et adresses remplacés, longueur bornée.
pub fn sanitize_client_message(message: &str) -> String {
    let mut out = String::new();
    let mut token = String::new();
    let flush = |token: &mut String, out: &mut String| {
        if token.is_empty() {
            return;
        }
        if sensitive_token(token) {
            out.push('…');
        } else {
            out.push_str(token);
        }
        token.clear();
    };
    for c in message.chars() {
        if token_char(c) {
            token.push(c);
        } else {
            flush(&mut token, &mut out);
            if c.is_control() {
                out.push(' ');
            } else {
                out.push(c);
            }
        }
    }
    flush(&mut token, &mut out);
    out.trim().chars().take(MAX_CLIENT_MESSAGE_CHARS).collect()
}

/// Erreur à agréger.
#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub fingerprint: String,
    pub source: &'static str,
    pub route: Option<String>,
    pub sample: Value,
}

/// Cadres de pile utiles : noms de fonctions de notre code, sans chemin de fichier, 80 caractères au plus.
pub fn frames_of(backtrace: &str) -> Vec<String> {
    let mut frames = vec![];
    for line in backtrace.lines() {
        let line = line.trim();
        // « 3: immo_server::api::handler » ; les lignes « at fichier:ligne » sont ignorées
        let Some((_, name)) = line.split_once(": ") else {
            continue;
        };
        if line.starts_with("at ") || !(name.contains("immo_") || name.contains("immo-")) {
            continue;
        }
        let name: String = name.chars().take(80).collect();
        if !frames.contains(&name) {
            frames.push(name);
        }
        if frames.len() >= MAX_FRAMES {
            break;
        }
    }
    frames
}

/// Erreur serveur non prévue (réponse 500).
pub fn server_error(
    route: &str,
    method: &str,
    status: u16,
    request_id: &str,
    version: &str,
    frames: &[String],
) -> ErrorReport {
    let mut parts: Vec<&str> = vec!["server", route, "server.unexpected"];
    parts.extend(frames.iter().take(MAX_FRAMES).map(String::as_str));
    ErrorReport {
        fingerprint: fingerprint(&parts),
        source: "server",
        route: Some(route.chars().take(200).collect()),
        sample: json!({
            "request_id": request_id,
            "route": route,
            "method": method,
            "status": status,
            "server_version": version,
            "kind": "server.unexpected",
            "frames": frames.iter().take(MAX_FRAMES).collect::<Vec<_>>(),
        }),
    }
}

/// Erreur du front (`POST /client-errors`) : message nettoyé, écran de la liste, version, identifiants de requête valides.
pub fn client_error(message: &str, screen: &str, client_version: &str, request_ids: &[String]) -> ErrorReport {
    let clean = sanitize_client_message(message);
    ErrorReport {
        fingerprint: fingerprint(&["client", screen, &clean, client_version]),
        source: "client",
        route: Some(screen.to_string()),
        sample: json!({
            "screen": screen,
            "message": clean,
            "client_version": client_version,
            "request_ids": request_ids.iter().take(5).collect::<Vec<_>>(),
        }),
    }
}

const RECORD: &str = "INSERT INTO error_reports (org_id, fingerprint, source, route, sample) \
     SELECT current_org_id(), $1::text, $2::text, $3::text, $4::jsonb \
      WHERE (SELECT count(*) FROM error_reports WHERE first_seen_at > app_now() - interval '1 day') < $5 \
         OR EXISTS (SELECT 1 FROM error_reports WHERE fingerprint = $1::text) \
     ON CONFLICT (org_id, fingerprint) DO UPDATE \
        SET occurrences = error_reports.occurrences + 1, last_seen_at = app_now(), sample = EXCLUDED.sample";

/// Agrège l'erreur dans la transaction de l'organisation : première occurrence → une ligne ; suivantes → `occurrences + 1`.
pub async fn record(tx: &mut OrgTx, report: &ErrorReport) -> Result<(), DbError> {
    sqlx::query(RECORD)
        .bind(&report.fingerprint)
        .bind(report.source)
        .bind(report.route.as_deref())
        .bind(report.sample.to_string())
        .bind(MAX_NEW_FINGERPRINTS_PER_DAY)
        .execute(tx.conn())
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empreinte_stable_et_sans_ambiguite_de_separation() {
        let a = fingerprint(&["server", "/api/x", "e"]);
        assert_eq!(a, fingerprint(&["server", "/api/x", "e"]));
        assert_eq!(a.len(), 64);
        assert_ne!(fingerprint(&["ab", "c"]), fingerprint(&["a", "bc"]));
    }

    #[test]
    fn le_message_du_front_est_nettoye() {
        let m = sanitize_client_message(
            "Échec pour +22670123456 et traore@agence.bf : jeton 0123456789abcdef0123456789abcdef (réf 123456)",
        );
        assert!(!m.contains("22670123456"), "{m}");
        assert!(!m.contains('@'), "{m}");
        assert!(!m.contains("0123456789abcdef"), "{m}");
        assert!(!m.contains("123456"), "{m}");
        assert!(m.contains("Échec"), "le texte courant reste lisible : {m}");
        assert_eq!(
            sanitize_client_message("TypeError: x is undefined"),
            "TypeError: x is undefined"
        );
        let long = "a ".repeat(300);
        assert_eq!(sanitize_client_message(&long).chars().count(), MAX_CLIENT_MESSAGE_CHARS);
        assert_eq!(sanitize_client_message("a\nb\u{0}c"), "a b c");
    }

    #[test]
    fn cadres_de_pile_limites_a_notre_code() {
        let bt = "   0: std::backtrace::Backtrace::force_capture\n   1: immo_server::problem::ApiError::unexpected\n             at ./crates/server/src/problem.rs:436\n   2: immo_server::api::login\n   3: tokio::runtime::task::core\n";
        let frames = frames_of(bt);
        assert_eq!(
            frames,
            vec!["immo_server::problem::ApiError::unexpected", "immo_server::api::login"]
        );
        assert!(frames_of("").is_empty());
    }

    #[test]
    fn deux_erreurs_serveur_identiques_ont_la_meme_empreinte_et_l_exemple_est_sans_corps() {
        let f = vec!["immo_server::api::login".to_string()];
        let a = server_error("/api/x", "GET", 500, "INC-AAAAAA", "1.0.0", &f);
        let b = server_error("/api/x", "GET", 500, "INC-BBBBBB", "1.0.0", &f);
        assert_eq!(a.fingerprint, b.fingerprint);
        assert_ne!(
            a.fingerprint,
            server_error("/api/y", "GET", 500, "INC-AAAAAA", "1.0.0", &f).fingerprint
        );
        let keys: Vec<&String> = a.sample.as_object().map(|o| o.keys().collect()).unwrap_or_default();
        for forbidden in ["body", "params", "headers", "message", "query"] {
            assert!(!keys.iter().any(|k| k.as_str() == forbidden), "{forbidden}");
        }
    }
}
