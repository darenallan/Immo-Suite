//! Sessions côté serveur (SECURITY.md §3.2, DP-85) : identifiant de 256 bits dont seule l'empreinte est stockée, jeton
//! anti-falsification dérivé par HMAC, durées d'expiration.

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use super::authorize::Role;

/// Expiration absolue d'une session, en heures (tous les rôles).
pub const ABSOLUTE_HOURS: i64 = 12;
/// Nom du cookie : le préfixe `__Host-` impose `Secure`, le chemin `/` et l'absence de `Domain`.
pub const COOKIE_NAME: &str = "__Host-session";

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("aucune source d'aléa disponible")]
    Random,
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| text.get(i..i + 2).and_then(|pair| u8::from_str_radix(pair, 16).ok()))
        .collect()
}

/// Nouvel identifiant de session : 32 octets aléatoires du système, en hexadécimal (64 caractères). C'est la valeur du cookie.
pub fn generate_token() -> Result<String, SessionError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| SessionError::Random)?;
    Ok(to_hex(&bytes))
}

/// Empreinte SHA-256 du jeton lu dans le cookie. `None` si le jeton n'a pas la forme attendue (64 caractères hexadécimaux minuscules).
pub fn hash_token(token: &str) -> Option<[u8; 32]> {
    if token.len() != 64 {
        return None;
    }
    let bytes = from_hex(token)?;
    Some(Sha256::digest(&bytes).into())
}

/// Jeton anti-falsification de la session : HMAC-SHA256 du jeton de session avec `SESSION_SECRET` (DP-85).
/// Changer `SESSION_SECRET` rend tous les jetons invalides.
pub fn csrf_token(secret: &[u8], session_token: &str) -> Option<String> {
    // Un HMAC accepte une clé de toute longueur ; `None` n'est jamais renvoyé en pratique, et un jeton absent ne correspond à rien.
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(secret).ok()?;
    mac.update(b"immo-csrf-v1:");
    mac.update(session_token.as_bytes());
    Some(to_hex(&mac.finalize().into_bytes()))
}

/// Empreinte du jeton anti-falsification, stockée à la création de la session (`sessions.csrf_hash`).
pub fn csrf_hash(csrf: &str) -> [u8; 32] {
    Sha256::digest(csrf.as_bytes()).into()
}

/// Compare en temps constant le jeton présenté dans `X-CSRF-Token` au jeton attendu pour cette session.
pub fn csrf_matches(secret: &[u8], session_token: &str, presented: &str) -> bool {
    let Some(expected) = csrf_token(secret, session_token) else {
        return false;
    };
    !presented.is_empty() && bool::from(expected.as_bytes().ct_eq(presented.as_bytes()))
}

/// Fin de session par inactivité : 30 minutes (60 pour gérant et propriétaire), jamais au-delà de l'expiration absolue.
pub fn idle_expiry(now: DateTime<Utc>, role: Role, absolute: DateTime<Utc>) -> DateTime<Utc> {
    (now + Duration::minutes(role.idle_minutes())).min(absolute)
}

/// Comme `idle_expiry`, avec une durée d'inactivité forcée en minutes (essai : exercer l'expiration, interdit en production).
pub fn idle_expiry_minutes(now: DateTime<Utc>, minutes: i64, absolute: DateTime<Utc>) -> DateTime<Utc> {
    (now + Duration::minutes(minutes)).min(absolute)
}

pub fn absolute_expiry(created: DateTime<Utc>) -> DateTime<Utc> {
    created + Duration::hours(ABSOLUTE_HOURS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn le_jeton_fait_256_bits_en_hexadecimal_et_change_a_chaque_fois() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert_ne!(a, b);
    }

    #[test]
    fn seule_l_empreinte_du_jeton_est_calculable_et_stable() {
        let token = "ab".repeat(32);
        let h1 = hash_token(&token).unwrap();
        assert_eq!(h1, hash_token(&token).unwrap());
        assert_ne!(h1, hash_token(&"cd".repeat(32)).unwrap());
        // le jeton lui-même n'est pas l'empreinte
        assert_ne!(to_hex(&h1), token);
    }

    #[test]
    fn un_jeton_mal_forme_n_a_pas_d_empreinte() {
        for bad in [
            "",
            "abc",
            &"zz".repeat(32),
            &"AB".repeat(32),
            &"ab".repeat(31),
            &"ab".repeat(33),
        ] {
            assert_eq!(hash_token(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn le_jeton_anti_falsification_est_lie_a_la_session_et_au_secret() {
        let (s1, s2) = ("ab".repeat(32), "cd".repeat(32));
        let c1 = csrf_token(b"secret-un", &s1).unwrap();
        assert_eq!(c1, csrf_token(b"secret-un", &s1).unwrap(), "déterministe");
        assert_ne!(c1, csrf_token(b"secret-un", &s2).unwrap(), "lié à la session");
        assert_ne!(c1, csrf_token(b"secret-deux", &s1).unwrap(), "lié au secret");
        assert!(csrf_matches(b"secret-un", &s1, &c1));
        assert!(!csrf_matches(b"secret-un", &s2, &c1));
        assert!(
            !csrf_matches(b"secret-deux", &s1, &c1),
            "changer SESSION_SECRET invalide les jetons"
        );
        assert!(!csrf_matches(b"secret-un", &s1, ""));
        assert!(!csrf_matches(b"secret-un", &s1, &c1[..63]));
        assert_eq!(csrf_hash(&c1).len(), 32);
    }

    #[test]
    fn durees_d_inactivite_et_expiration_absolue() {
        let now = Utc.with_ymd_and_hms(2026, 11, 13, 9, 15, 0).unwrap();
        let absolute = absolute_expiry(now);
        assert_eq!(absolute, Utc.with_ymd_and_hms(2026, 11, 13, 21, 15, 0).unwrap());
        assert_eq!(idle_expiry(now, Role::Cashier, absolute), now + Duration::minutes(30));
        assert_eq!(idle_expiry(now, Role::Manager, absolute), now + Duration::minutes(60));
        assert_eq!(idle_expiry(now, Role::Owner, absolute), now + Duration::minutes(60));
        assert_eq!(
            idle_expiry(now, Role::FieldAgent, absolute),
            now + Duration::minutes(30)
        );
        // jamais au-delà de l'expiration absolue
        let late = now + Duration::hours(11) + Duration::minutes(50);
        assert_eq!(idle_expiry(late, Role::Manager, absolute), absolute);
    }
}
