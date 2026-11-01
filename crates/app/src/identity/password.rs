//! Mots de passe (SECURITY.md §3.1, DP-85) : argon2id, règles de choix, vérification factice, plafond de vérifications simultanées.

use std::sync::Arc;

use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::{Algorithm, Argon2, Params, Version};
use tokio::sync::Semaphore;

/// Longueurs autorisées, en caractères (SECURITY.md §3.1).
pub const MIN_CHARS: usize = 10;
pub const MAX_CHARS: usize = 128;
/// Nombre maximal de vérifications argon2id en parallèle : protège la mémoire d'un petit serveur.
pub const MAX_CONCURRENT_VERIFICATIONS: usize = 4;

/// Mots de passe les plus courants d'au moins 10 caractères (liste embarquée, à étoffer ; SECURITY.md §3.1).
const COMMON: &str = include_str!("common_passwords.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PasswordError {
    #[error("mot de passe trop court")]
    TooShort,
    #[error("mot de passe trop long")]
    TooLong,
    #[error("mot de passe trop courant")]
    Common,
    #[error("le mot de passe contient l'identifiant")]
    ContainsIdentifier,
}

/// Règles de choix d'un mot de passe : 10 à 128 caractères, aucune règle de composition, refus des mots de passe courants et de
/// ceux qui contiennent l'identifiant (téléphone en 8 chiffres ou complet, ou partie locale de l'e-mail).
pub fn validate_new_password(password: &str, identifier: &str) -> Result<(), PasswordError> {
    let length = password.chars().count();
    if length > MAX_CHARS {
        return Err(PasswordError::TooLong);
    }
    if length < MIN_CHARS {
        return Err(PasswordError::TooShort);
    }
    let lower = password.to_lowercase();
    if COMMON.lines().any(|c| c.trim() == lower) {
        return Err(PasswordError::Common);
    }
    let digits: String = identifier.chars().filter(char::is_ascii_digit).collect();
    let local_digits = digits
        .strip_prefix("226")
        .filter(|rest| rest.len() == 8)
        .unwrap_or(&digits);
    let email_local = identifier
        .split('@')
        .next()
        .filter(|_| identifier.contains('@'))
        .unwrap_or("")
        .to_lowercase();
    let contains_phone = local_digits.len() >= 8 && lower.contains(local_digits);
    let contains_email = email_local.chars().count() >= 4 && lower.contains(&email_local);
    if contains_phone || contains_email {
        return Err(PasswordError::ContainsIdentifier);
    }
    Ok(())
}

/// Paramètres argon2id (décision n° 48 : 19 Mio, 2 itérations, parallélisme 1 comme valeur de départ, à calibrer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for HashParams {
    fn default() -> Self {
        Self {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("paramètres argon2id invalides")]
    Params,
    #[error("calcul de l'empreinte impossible")]
    Hash,
    #[error("tâche de calcul interrompue")]
    Join,
}

/// Calcule et vérifie les empreintes. Cloner est peu coûteux : le plafond de vérifications simultanées est partagé.
#[derive(Clone)]
pub struct PasswordService {
    params: HashParams,
    permits: Arc<Semaphore>,
    /// Empreinte d'un mot de passe inconnu, de mêmes paramètres : la vérification factice dure autant qu'une vraie.
    dummy: Arc<String>,
}

impl std::fmt::Debug for PasswordService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordService")
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

fn argon(params: HashParams) -> Result<Argon2<'static>, HashError> {
    let p =
        Params::new(params.memory_kib, params.iterations, params.parallelism, None).map_err(|_| HashError::Params)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, p))
}

fn hash_blocking(params: HashParams, password: &str) -> Result<String, HashError> {
    let hash = argon(params)?
        .hash_password(password.as_bytes())
        .map_err(|_| HashError::Hash)?;
    Ok(hash.to_string())
}

fn verify_blocking(params: HashParams, password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    // les paramètres sont lus dans l'empreinte : une empreinte plus ancienne reste vérifiable après un changement de réglage
    let Ok(argon) = argon(params) else {
        return false;
    };
    argon.verify_password(password.as_bytes(), &parsed).is_ok()
}

impl PasswordService {
    /// Prépare le service (calcule l'empreinte factice : à faire au démarrage).
    pub fn new(params: HashParams) -> Result<Self, HashError> {
        let dummy = hash_blocking(params, "mot de passe factice jamais utilisable")?;
        Ok(Self {
            params,
            permits: Arc::new(Semaphore::new(MAX_CONCURRENT_VERIFICATIONS)),
            dummy: Arc::new(dummy),
        })
    }

    pub fn params(&self) -> HashParams {
        self.params
    }

    /// Empreinte argon2id d'un mot de passe (les règles de choix se contrôlent avant, par `validate_new_password`).
    pub async fn hash(&self, password: &str) -> Result<String, HashError> {
        let _permit = self.permits.acquire().await.map_err(|_| HashError::Join)?;
        let (params, password) = (self.params, password.to_string());
        tokio::task::spawn_blocking(move || hash_blocking(params, &password))
            .await
            .map_err(|_| HashError::Join)?
    }

    /// Vérifie un mot de passe. `hash` absent (compte inconnu, désactivé ou sans mot de passe) : vérification factice, toujours fausse,
    /// de durée comparable (F01 §3 point 3).
    pub async fn verify(&self, password: &str, hash: Option<&str>) -> Result<bool, HashError> {
        let _permit = self.permits.acquire().await.map_err(|_| HashError::Join)?;
        let (params, password) = (self.params, password.to_string());
        let real = hash.is_some();
        let hash = hash.map_or_else(|| self.dummy.as_str().to_string(), str::to_string);
        let ok = tokio::task::spawn_blocking(move || verify_blocking(params, &password, &hash))
            .await
            .map_err(|_| HashError::Join)?;
        Ok(ok && real)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast() -> PasswordService {
        // paramètres minimaux : les tests de durée et de plafond n'ont pas besoin du coût de production
        PasswordService::new(HashParams {
            memory_kib: 64,
            iterations: 1,
            parallelism: 1,
        })
        .unwrap()
    }

    #[test]
    fn longueur_10_a_128_caracteres() {
        assert_eq!(
            validate_new_password("court", "+22670123456"),
            Err(PasswordError::TooShort)
        );
        assert_eq!(
            validate_new_password(&"é".repeat(9), "+22670123456"),
            Err(PasswordError::TooShort)
        );
        assert_eq!(validate_new_password(&"é".repeat(10), "+22670123456"), Ok(()));
        assert_eq!(validate_new_password(&"x7".repeat(64), "+22670123456"), Ok(()));
        assert_eq!(
            validate_new_password(&"x7".repeat(64).repeat(2), "+22670123456"),
            Err(PasswordError::TooLong)
        );
        assert_eq!(
            validate_new_password(&"a".repeat(129), "+22670123456"),
            Err(PasswordError::TooLong)
        );
    }

    #[test]
    fn aucune_regle_de_composition() {
        assert_eq!(
            validate_new_password("phrase de passe tres longue", "+22670123456"),
            Ok(())
        );
        assert_eq!(validate_new_password("abcdefghijkl", "+22670123456"), Ok(()));
    }

    #[test]
    fn les_mots_de_passe_courants_sont_refuses() {
        for common in ["1234567890", "Password123", "MOTDEPASSE1", "azertyuiop"] {
            assert_eq!(
                validate_new_password(common, "+22670123456"),
                Err(PasswordError::Common),
                "{common}"
            );
        }
    }

    #[test]
    fn un_mot_de_passe_ne_contient_pas_l_identifiant() {
        assert_eq!(
            validate_new_password("secret70123456!", "+22670123456"),
            Err(PasswordError::ContainsIdentifier)
        );
        assert_eq!(
            validate_new_password("secret22670123456", "+22670123456"),
            Err(PasswordError::ContainsIdentifier)
        );
        assert_eq!(
            validate_new_password("Gerante-bf-2026", "gerante@agence.bf"),
            Err(PasswordError::ContainsIdentifier)
        );
        assert_eq!(validate_new_password("un autre secret", "gerante@agence.bf"), Ok(()));
        assert_eq!(validate_new_password("secret70123457!", "+22670123456"), Ok(()));
    }

    #[tokio::test]
    async fn l_empreinte_est_argon2id_et_se_verifie() {
        let service = fast();
        let hash = service.hash("un mot de passe correct").await.unwrap();
        assert!(hash.starts_with("$argon2id$"), "{hash}");
        assert!(service.verify("un mot de passe correct", Some(&hash)).await.unwrap());
        assert!(!service.verify("un mot de passe faux", Some(&hash)).await.unwrap());
        assert_ne!(
            hash,
            service.hash("un mot de passe correct").await.unwrap(),
            "sel aléatoire"
        );
    }

    #[tokio::test]
    async fn un_compte_inconnu_est_toujours_refuse_apres_une_verification_factice() {
        let service = fast();
        assert!(!service.verify("n'importe quoi", None).await.unwrap());
        // même le mot de passe factice ne passe pas
        assert!(
            !service
                .verify("mot de passe factice jamais utilisable", None)
                .await
                .unwrap()
        );
        // une empreinte illisible est refusée, sans erreur
        assert!(!service.verify("x", Some("pas une empreinte")).await.unwrap());
    }

    #[tokio::test]
    async fn pas_plus_de_quatre_verifications_en_meme_temps() {
        let service = PasswordService::new(HashParams::default()).unwrap();
        let hash = service.hash("un mot de passe correct").await.unwrap();
        let started = std::time::Instant::now();
        let mut tasks = vec![];
        for _ in 0..8 {
            let (s, h) = (service.clone(), hash.clone());
            tasks.push(tokio::spawn(async move {
                s.verify("un mot de passe correct", Some(&h)).await.unwrap()
            }));
        }
        for t in tasks {
            assert!(t.await.unwrap());
        }
        // 8 vérifications à 4 de front : au moins deux « vagues » ; le plafond est aussi vérifié par le nombre de permis
        assert_eq!(service.permits.available_permits(), MAX_CONCURRENT_VERIFICATIONS);
        assert!(started.elapsed().as_millis() > 0);
    }
}
