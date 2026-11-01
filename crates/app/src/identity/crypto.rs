//! Chiffrement des champs sensibles et jetons à usage unique (SECURITY.md §5, DP-52, DP-88).
//!
//! - Champ chiffré : AES-256-GCM, nonce de 12 octets tiré à chaque valeur, identifiant de clé en premier octet
//!   (`[id][nonce 12][chiffré + étiquette]`). La clé vient de l'environnement seulement (`FIELD_ENCRYPTION_KEY`).
//! - Sous-clés : HMAC-SHA256 de la clé de champs avec une étiquette, pour ne jamais utiliser la même clé à deux usages
//!   (codes de secours TOTP).
//! - Jeton d'invitation ou de réinitialisation : 32 octets du système en base64url sans remplissage (43 caractères) ; seule
//!   l'empreinte SHA-256 est stockée.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use sha2::{Digest, Sha256};

/// Identifiant de la clé courante, placé en tête de chaque valeur chiffrée.
pub const KEY_ID: u8 = 1;
const NONCE_BYTES: usize = 12;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("clé de chiffrement invalide : 32 octets attendus")]
    BadKey,
    #[error("aucune source d'aléa disponible")]
    Random,
    #[error("valeur chiffrée illisible")]
    Unreadable,
}

/// Clé de chiffrement des champs. `Debug` ne montre jamais la clé.
#[derive(Clone)]
pub struct FieldKey {
    bytes: [u8; 32],
}

impl std::fmt::Debug for FieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldKey").finish_non_exhaustive()
    }
}

impl FieldKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| CryptoError::BadKey)?;
        Ok(Self { bytes })
    }

    /// 64 caractères hexadécimaux (format de `FIELD_ENCRYPTION_KEY`).
    pub fn from_hex(text: &str) -> Result<Self, CryptoError> {
        let text = text.trim();
        if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CryptoError::BadKey);
        }
        let bytes: Vec<u8> = (0..64)
            .step_by(2)
            .filter_map(|i| text.get(i..i + 2).and_then(|p| u8::from_str_radix(p, 16).ok()))
            .collect();
        Self::from_bytes(&bytes)
    }

    fn cipher(&self) -> Result<Aes256Gcm, CryptoError> {
        let key = Key::<Aes256Gcm>::try_from(self.bytes.as_slice()).map_err(|_| CryptoError::BadKey)?;
        Ok(Aes256Gcm::new(&key))
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0_u8; NONCE_BYTES];
        getrandom::fill(&mut nonce_bytes).map_err(|_| CryptoError::Random)?;
        let nonce = Nonce::try_from(nonce_bytes.as_slice()).map_err(|_| CryptoError::Unreadable)?;
        let sealed = self
            .cipher()?
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::Unreadable)?;
        let mut out = Vec::with_capacity(1 + NONCE_BYTES + sealed.len());
        out.push(KEY_ID);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&sealed);
        Ok(out)
    }

    pub fn decrypt(&self, blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let (Some(&id), Some(nonce_bytes), Some(sealed)) =
            (blob.first(), blob.get(1..=NONCE_BYTES), blob.get(1 + NONCE_BYTES..))
        else {
            return Err(CryptoError::Unreadable);
        };
        if id != KEY_ID {
            return Err(CryptoError::Unreadable);
        }
        let nonce = Nonce::try_from(nonce_bytes).map_err(|_| CryptoError::Unreadable)?;
        self.cipher()?
            .decrypt(&nonce, sealed)
            .map_err(|_| CryptoError::Unreadable)
    }

    /// Sous-clé de 32 octets pour un usage nommé.
    pub fn subkey(&self, label: &str) -> [u8; 32] {
        hmac_sha256(&self.bytes, format!("immo-subkey-v1:{label}").as_bytes())
    }
}

/// HMAC-SHA256 (RFC 2104) de `data` avec `key`. Écrit ici plutôt qu'avec une bibliothèque faillible : accepte toute longueur de clé
/// sans cas d'erreur. Vérifié sur les vecteurs de la RFC 4231.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut block = [0_u8; 64];
    if key.len() > 64 {
        for (slot, byte) in block.iter_mut().zip(Sha256::digest(key)) {
            *slot = byte;
        }
    } else {
        for (slot, byte) in block.iter_mut().zip(key) {
            *slot = *byte;
        }
    }
    let inner_pad: Vec<u8> = block.iter().map(|b| b ^ 0x36).collect();
    let outer_pad: Vec<u8> = block.iter().map(|b| b ^ 0x5c).collect();
    let mut inner = Sha256::new();
    inner.update(&inner_pad);
    inner.update(data);
    let mut outer = Sha256::new();
    outer.update(&outer_pad);
    outer.update(inner.finalize());
    outer.finalize().into()
}

// ───────── base64url et base32 ─────────

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64 URL-safe sans remplissage.
pub fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let symbols = chunk.len() + 1;
        for i in 0..symbols {
            let index = ((n >> (18 - 6 * i)) & 0x3f) as usize;
            if let Some(c) = B64.get(index) {
                out.push(char::from(*c));
            }
        }
    }
    out
}

/// Décodage inverse ; `None` si un caractère est hors alphabet ou si la longueur est impossible.
pub fn base64url_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut buffer = 0_u32;
    let mut bits = 0_u32;
    for c in text.bytes() {
        let value = B64.iter().position(|x| *x == c)?;
        buffer = (buffer << 6) | u32::try_from(value).ok()?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((buffer >> bits) & 0xff).ok()?);
        }
    }
    Some(out)
}

const B32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Base32 (RFC 4648) sans remplissage, majuscules : le format des clés d'application d'authentification.
pub fn base32(bytes: &[u8]) -> String {
    let mut out = String::with_capacity((bytes.len() * 8).div_ceil(5));
    let mut buffer = 0_u32;
    let mut bits = 0_u32;
    for byte in bytes {
        buffer = (buffer << 8) | u32::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            if let Some(c) = B32.get(((buffer >> bits) & 0x1f) as usize) {
                out.push(char::from(*c));
            }
        }
    }
    if bits > 0
        && let Some(c) = B32.get(((buffer << (5 - bits)) & 0x1f) as usize)
    {
        out.push(char::from(*c));
    }
    out
}

// ───────── jetons à usage unique ─────────

/// Nouveau jeton : 32 octets du système, 43 caractères base64url.
pub fn new_token() -> Result<String, CryptoError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| CryptoError::Random)?;
    Ok(base64url(&bytes))
}

/// Empreinte SHA-256 stockée à la place du jeton.
pub fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

/// Un jeton présenté a la forme attendue (43 caractères de l'alphabet base64url). Évite une requête pour une saisie absurde.
pub fn token_shape_ok(token: &str) -> bool {
    token.len() == 43 && token.bytes().all(|c| B64.contains(&c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> FieldKey {
        FieldKey::from_bytes(&[7_u8; 32]).unwrap()
    }

    #[test]
    fn le_chiffrement_est_reversible_et_different_a_chaque_fois() {
        let k = key();
        let a = k.encrypt(b"JBSWY3DPEHPK3PXP").unwrap();
        let b = k.encrypt(b"JBSWY3DPEHPK3PXP").unwrap();
        assert_ne!(a, b, "nonce différent à chaque valeur");
        assert_eq!(a.first(), Some(&KEY_ID), "identifiant de clé en tête");
        assert_eq!(k.decrypt(&a).unwrap(), b"JBSWY3DPEHPK3PXP");
        // le secret en clair n'apparaît pas dans la valeur stockée
        assert!(!a.windows(6).any(|w| w == b"JBSWY3"));
    }

    #[test]
    fn une_valeur_alteree_ou_une_autre_cle_est_refusee() {
        let k = key();
        let mut blob = k.encrypt(b"secret").unwrap();
        assert_eq!(
            FieldKey::from_bytes(&[8_u8; 32]).unwrap().decrypt(&blob),
            Err(CryptoError::Unreadable)
        );
        if let Some(last) = blob.last_mut() {
            *last ^= 1;
        }
        assert_eq!(k.decrypt(&blob), Err(CryptoError::Unreadable));
        assert_eq!(k.decrypt(&[]), Err(CryptoError::Unreadable));
        assert_eq!(k.decrypt(&[KEY_ID, 0, 0]), Err(CryptoError::Unreadable));
        let mut wrong_id = k.encrypt(b"x").unwrap();
        wrong_id[0] = 9;
        assert_eq!(k.decrypt(&wrong_id), Err(CryptoError::Unreadable));
    }

    #[test]
    fn la_cle_se_lit_en_hexadecimal_et_ne_s_affiche_pas() {
        assert!(FieldKey::from_hex(&"ab".repeat(32)).is_ok());
        for bad in ["", "ab", &"zz".repeat(32), &"ab".repeat(31), &"ab".repeat(33)] {
            assert_eq!(FieldKey::from_hex(bad).err(), Some(CryptoError::BadKey), "{bad:?}");
        }
        assert!(!format!("{:?}", key()).contains('7'));
    }

    #[test]
    fn hmac_sha256_suit_les_vecteurs_de_la_rfc_4231() {
        let hex = |bytes: [u8; 32]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        // cas 1 : clé de 20 octets 0x0b
        assert_eq!(
            hex(hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        // cas 2 : clé « Jefe »
        assert_eq!(
            hex(hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // cas 6 : clé de 131 octets (plus longue que le bloc)
        assert_eq!(
            hex(hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn les_sous_cles_different_selon_l_usage() {
        let k = key();
        assert_ne!(k.subkey("backup-codes"), k.subkey("autre"));
        assert_eq!(k.subkey("backup-codes"), k.subkey("backup-codes"));
        assert_ne!(k.subkey("backup-codes"), [7_u8; 32]);
    }

    #[test]
    fn base64url_et_base32_suivent_les_vecteurs_de_reference() {
        // RFC 4648 §10 (base64 sans remplissage, alphabet URL) et §10 (base32)
        assert_eq!(base64url(b""), "");
        assert_eq!(base64url(b"f"), "Zg");
        assert_eq!(base64url(b"fo"), "Zm8");
        assert_eq!(base64url(b"foo"), "Zm9v");
        assert_eq!(base64url(b"foob"), "Zm9vYg");
        assert_eq!(base64url(b"fooba"), "Zm9vYmE");
        assert_eq!(base64url(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
        assert_eq!(base32(b""), "");
        assert_eq!(base32(b"f"), "MY");
        assert_eq!(base32(b"fo"), "MZXQ");
        assert_eq!(base32(b"foo"), "MZXW6");
        assert_eq!(base32(b"foob"), "MZXW6YQ");
        assert_eq!(base32(b"fooba"), "MZXW6YTB");
        assert_eq!(base32(b"foobar"), "MZXW6YTBOI");
    }

    #[test]
    fn base64url_se_decode() {
        for sample in [&b""[..], b"f", b"fo", b"foo", b"foobar", &[0xfb, 0xff, 0x00, 0x10]] {
            assert_eq!(base64url_decode(&base64url(sample)).as_deref(), Some(sample));
        }
        assert_eq!(base64url_decode("Zm9v!"), None);
        assert_eq!(base64url_decode("A"), None);
    }

    #[test]
    fn un_jeton_fait_43_caracteres_et_seule_son_empreinte_est_stockable() {
        let a = new_token().unwrap();
        let b = new_token().unwrap();
        assert_eq!(a.len(), 43);
        assert!(token_shape_ok(&a));
        assert_ne!(a, b);
        assert_ne!(token_hash(&a), token_hash(&b));
        assert_eq!(token_hash(&a), token_hash(&a));
        for bad in [
            "",
            "abc",
            &"a".repeat(42),
            &"a".repeat(44),
            &format!("{}!", "a".repeat(42)),
        ] {
            assert!(!token_shape_ok(bad), "{bad:?}");
        }
    }
}
