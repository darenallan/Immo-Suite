//! Double authentification par code temporel (SECURITY.md §3.4, F01 §9.5, DP-88) : RFC 6238 (HOTP de la RFC 4226, SHA-1,
//! 6 chiffres, pas de 30 secondes), décalage d'un pas toléré de chaque côté, rejeu refusé, codes de secours.

use chrono::{DateTime, Utc};
use sha1::Sha1;
use subtle::ConstantTimeEq;

use super::crypto::{FieldKey, base32, hmac_sha256};

/// Durée d'un pas, en secondes.
pub const STEP_SECONDS: i64 = 30;
/// Nombre de pas acceptés de chaque côté du pas courant (horloge de téléphone décalée d'une minute au plus).
pub const SKEW_STEPS: i64 = 1;
/// Octets du secret (160 bits, la taille de la sortie de SHA-1 recommandée par la RFC 4226).
pub const SECRET_BYTES: usize = 20;
/// Nombre de codes de secours remis à l'activation.
pub const BACKUP_CODES: usize = 10;
/// Alphabet des codes de secours : 32 symboles sans ambiguïté (ni 0/O, ni 1/I/L, ni U).
const BACKUP_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKMNPQRSTVWXYZab";

/// HMAC-SHA1 (RFC 2104), écrit ici comme `hmac_sha256` : aucune erreur possible.
fn hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    use sha1::Digest;
    let mut block = [0_u8; 64];
    if key.len() > 64 {
        for (slot, byte) in block.iter_mut().zip(Sha1::digest(key)) {
            *slot = byte;
        }
    } else {
        for (slot, byte) in block.iter_mut().zip(key) {
            *slot = *byte;
        }
    }
    let inner_pad: Vec<u8> = block.iter().map(|b| b ^ 0x36).collect();
    let outer_pad: Vec<u8> = block.iter().map(|b| b ^ 0x5c).collect();
    let mut inner = Sha1::new();
    inner.update(&inner_pad);
    inner.update(data);
    let mut outer = Sha1::new();
    outer.update(&outer_pad);
    outer.update(inner.finalize());
    outer.finalize().into()
}

/// Code à 6 chiffres du pas `step` (HOTP, RFC 4226 §5.3).
pub fn code_for_step(secret: &[u8], step: u64) -> u32 {
    let mac = hmac_sha1(secret, &step.to_be_bytes());
    let offset = usize::from(mac.last().copied().unwrap_or(0) & 0x0f);
    let word = mac
        .get(offset..offset + 4)
        .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]) & 0x7fff_ffff);
    word % 1_000_000
}

/// Pas courant pour un instant donné.
pub fn step_at(now: DateTime<Utc>) -> i64 {
    now.timestamp().div_euclid(STEP_SECONDS)
}

/// Un code saisi est-il valide ? Renvoie le pas qui correspond (au plus récent des pas qui correspondent) à condition qu'il soit
/// **strictement supérieur** à `last_step`, le dernier pas déjà accepté pour ce compte (rejeu refusé).
pub fn verify(secret: &[u8], now: DateTime<Utc>, code: &str, last_step: Option<i64>) -> Option<i64> {
    let code = code.trim();
    if code.len() != 6 || !code.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let current = step_at(now);
    let mut found = None;
    // Parcours complet et comparaison en temps constant : le temps de réponse ne dit pas quel pas correspondait.
    for delta in -SKEW_STEPS..=SKEW_STEPS {
        let step = current + delta;
        let Ok(unsigned) = u64::try_from(step) else { continue };
        let expected = format!("{:06}", code_for_step(secret, unsigned));
        let matches: bool = expected.as_bytes().ct_eq(code.as_bytes()).into();
        if matches && last_step.is_none_or(|last| step > last) {
            found = Some(step);
        }
    }
    found
}

/// Nouveau secret aléatoire.
pub fn generate_secret() -> Result<[u8; SECRET_BYTES], getrandom::Error> {
    let mut secret = [0_u8; SECRET_BYTES];
    getrandom::fill(&mut secret)?;
    Ok(secret)
}

/// Clé en base32, par groupes de 4 pour la saisie à la main.
pub fn grouped_base32(secret: &[u8]) -> String {
    let text = base32(secret);
    text.as_bytes()
        .chunks(4)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lien `otpauth://` (SHA-1, 6 chiffres, 30 secondes ; émetteur « Immo Suite », libellé = identifiant de connexion).
pub fn otpauth_url(secret: &[u8], account: &str) -> String {
    let encode = |text: &str| {
        text.bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                    char::from(b).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect::<String>()
    };
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period={}",
        encode("Immo Suite"),
        encode(account),
        base32(secret),
        encode("Immo Suite"),
        STEP_SECONDS
    )
}

// ───────── codes de secours ─────────

/// Dix codes de 12 caractères (60 bits chacun), à afficher une fois.
pub fn generate_backup_codes() -> Result<Vec<String>, getrandom::Error> {
    let mut codes = Vec::with_capacity(BACKUP_CODES);
    for _ in 0..BACKUP_CODES {
        let mut bytes = [0_u8; 12];
        getrandom::fill(&mut bytes)?;
        // 32 symboles : `& 0x1f` est sans biais
        let code: String = bytes
            .iter()
            .filter_map(|b| BACKUP_ALPHABET.get(usize::from(*b & 0x1f)).map(|c| char::from(*c)))
            .collect();
        codes.push(code);
    }
    Ok(codes)
}

/// Affichage par groupes de 4 : `ABCD-EFGH-JKMN`.
pub fn display_backup_code(code: &str) -> String {
    code.as_bytes()
        .chunks(4)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("-")
}

/// Forme canonique d'une saisie : sans tirets ni espaces. Les codes de secours sont sensibles à la casse (alphabet mixte) : seule
/// la mise en forme est tolérée. `None` si la longueur n'est pas 12.
pub fn normalize_backup_code(input: &str) -> Option<String> {
    let cleaned: String = input.chars().filter(|c| !matches!(c, '-' | ' ')).collect();
    (cleaned.chars().count() == 12 && cleaned.bytes().all(|b| BACKUP_ALPHABET.contains(&b))).then_some(cleaned)
}

/// Une saisie ressemble-t-elle à un code de secours plutôt qu'à un code à 6 chiffres ?
pub fn looks_like_backup_code(input: &str) -> bool {
    normalize_backup_code(input).is_some()
}

/// Empreinte stockée d'un code de secours : HMAC-SHA256 avec une sous-clé de la clé de champs, liée à l'utilisateur
/// (deux comptes ne partagent jamais une empreinte). Une fuite de la base seule ne permet pas de deviner les codes.
pub fn backup_code_hash(key: &FieldKey, user_id: &str, canonical_code: &str) -> [u8; 32] {
    hmac_sha256(
        &key.subkey("totp-backup-codes"),
        format!("{user_id}:{canonical_code}").as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// Secret de la RFC 6238 (annexe B) : « 12345678901234567890 ».
    const RFC_SECRET: &[u8] = b"12345678901234567890";

    #[test]
    fn les_codes_suivent_les_vecteurs_de_la_rfc_6238_sha1() {
        // les vecteurs de l'annexe B sont à 8 chiffres ; on garde les 6 derniers chiffres
        for (time, expected) in [
            (59_i64, 287_082_u32),
            (1_111_111_109, 81_804),
            (1_111_111_111, 50_471),
            (1_234_567_890, 5_924),
            (2_000_000_000, 279_037),
        ] {
            let step = u64::try_from(time / STEP_SECONDS).unwrap();
            assert_eq!(code_for_step(RFC_SECRET, step), expected, "t = {time}");
        }
    }

    #[test]
    fn les_codes_hotp_suivent_les_vecteurs_de_la_rfc_4226() {
        for (counter, expected) in [(0_u64, 755_224_u32), (1, 287_082), (2, 359_152), (9, 520_489)] {
            assert_eq!(code_for_step(RFC_SECRET, counter), expected, "compteur {counter}");
        }
    }

    fn at(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).unwrap()
    }

    #[test]
    fn le_decalage_d_un_pas_est_accepte_deux_pas_sont_refuses() {
        let now = at(1_111_111_109); // pas 37 037 037
        let step = step_at(now);
        let code = |s: i64| format!("{:06}", code_for_step(RFC_SECRET, u64::try_from(s).unwrap()));
        assert_eq!(verify(RFC_SECRET, now, &code(step), None), Some(step));
        assert_eq!(verify(RFC_SECRET, now, &code(step - 1), None), Some(step - 1));
        assert_eq!(verify(RFC_SECRET, now, &code(step + 1), None), Some(step + 1));
        assert_eq!(verify(RFC_SECRET, now, &code(step - 2), None), None);
        assert_eq!(verify(RFC_SECRET, now, &code(step + 2), None), None);
    }

    #[test]
    fn un_code_deja_accepte_ne_sert_pas_deux_fois() {
        let now = at(1_111_111_109);
        let step = step_at(now);
        let code = format!("{:06}", code_for_step(RFC_SECRET, u64::try_from(step).unwrap()));
        assert_eq!(verify(RFC_SECRET, now, &code, Some(step - 1)), Some(step));
        assert_eq!(verify(RFC_SECRET, now, &code, Some(step)), None, "rejeu au même pas");
        assert_eq!(
            verify(RFC_SECRET, now, &code, Some(step + 5)),
            None,
            "pas plus ancien que le dernier accepté"
        );
    }

    #[test]
    fn les_saisies_mal_formees_sont_refusees() {
        let now = at(59);
        for bad in ["", "12345", "1234567", "12a456", "      ", "-12345", "287 082"] {
            assert_eq!(verify(RFC_SECRET, now, bad, None), None, "{bad:?}");
        }
        // espaces de bord tolérés
        assert_eq!(verify(RFC_SECRET, now, " 287082 ", None), Some(1));
    }

    #[test]
    fn la_cle_et_le_lien_otpauth() {
        assert_eq!(
            grouped_base32(b"12345678901234567890"),
            "GEZD GNBV GY3T QOJQ GEZD GNBV GY3T QOJQ"
        );
        let url = otpauth_url(b"12345678901234567890", "+22670123456");
        assert_eq!(
            url,
            "otpauth://totp/Immo%20Suite:%2B22670123456?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Immo%20Suite&algorithm=SHA1&digits=6&period=30"
        );
    }

    #[test]
    fn dix_codes_de_secours_de_douze_caracteres_tous_differents() {
        let codes = generate_backup_codes().unwrap();
        assert_eq!(codes.len(), BACKUP_CODES);
        let mut seen = std::collections::HashSet::new();
        for c in &codes {
            assert_eq!(c.chars().count(), 12);
            assert!(normalize_backup_code(c).is_some(), "{c}");
            assert!(seen.insert(c.clone()));
        }
    }

    #[test]
    fn les_codes_de_secours_se_normalisent_et_s_empreignent_par_utilisateur() {
        assert_eq!(display_backup_code("ABCDEFGHJKMN"), "ABCD-EFGH-JKMN");
        assert_eq!(
            normalize_backup_code("abcd-efgh-jkmn"),
            None,
            "la casse compte (alphabet mixte)"
        );
        assert_eq!(normalize_backup_code("ABCD-EFGH JKMN").as_deref(), Some("ABCDEFGHJKMN"));
        assert_eq!(normalize_backup_code("ABCDEFGHJKM"), None, "11 caractères");
        assert_eq!(normalize_backup_code("ABCDEFGHJKM0"), None, "0 hors alphabet");
        assert!(looks_like_backup_code("ABCD-EFGH-JKMN"));
        assert!(!looks_like_backup_code("287082"));
        let key = FieldKey::from_bytes(&[3_u8; 32]).unwrap();
        let a = backup_code_hash(&key, "user-1", "ABCDEFGHJKMN");
        assert_eq!(a, backup_code_hash(&key, "user-1", "ABCDEFGHJKMN"));
        assert_ne!(
            a,
            backup_code_hash(&key, "user-2", "ABCDEFGHJKMN"),
            "lié à l'utilisateur"
        );
        assert_ne!(
            a,
            backup_code_hash(&FieldKey::from_bytes(&[4_u8; 32]).unwrap(), "user-1", "ABCDEFGHJKMN")
        );
    }
}
