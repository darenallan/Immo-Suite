//! Réglages propres au serveur HTTP (DEPLOYMENT.md §2.4, DP-85) : secret de session, origines autorisées, proxy de confiance.
//! Lus de l'environnement avec `Config` ; aucun secret dans `Debug`, aucune valeur fautive dans les messages.

use immo_app::identity::password::HashParams;
use immo_app::platform::config::{Config, ConfigError, Environment, Secret};

/// Longueur minimale du secret de session, en octets.
pub const MIN_SECRET_BYTES: usize = 32;

#[derive(Debug, Clone)]
pub struct HttpSettings {
    pub session_secret: Secret,
    /// Origines acceptées dans l'en-tête `Origin` des requêtes qui écrivent (API_CONTRACTS.md §4.2).
    pub allowed_origins: Vec<String>,
    /// Derrière Caddy : l'adresse du client est la dernière entrée de `X-Forwarded-For`.
    pub trust_proxy: bool,
    pub min_client_version: String,
    /// Durée d'inactivité forcée, en minutes, pour exercer l'expiration en essai ; refusée en production.
    pub session_idle_minutes: Option<i64>,
    pub argon2: HashParams,
    /// Clé de chiffrement des champs (64 caractères hexadécimaux). Obligatoire en production ; sans elle, la double
    /// authentification est indisponible (DP-88).
    pub field_encryption_key: Option<Secret>,
}

fn origin_of(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.split('/').next().filter(|h| !h.is_empty())?;
    Some(format!("{scheme}://{host}"))
}

fn number<T: std::str::FromStr>(
    get: &impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: T,
) -> Result<T, ConfigError> {
    match get(name).filter(|v| !v.trim().is_empty()) {
        None => Ok(default),
        Some(v) => v.trim().parse().map_err(|_| ConfigError::Invalid {
            name,
            reason: "nombre attendu",
        }),
    }
}

impl HttpSettings {
    pub fn from_env(config: &Config) -> Result<Self, ConfigError> {
        Self::from_lookup(config, |name| std::env::var(name).ok())
    }

    pub fn from_lookup(config: &Config, get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let non_empty = |name: &str| get(name).filter(|v| !v.trim().is_empty());
        let secret = non_empty("SESSION_SECRET").ok_or(ConfigError::Missing("SESSION_SECRET"))?;
        if secret.len() < MIN_SECRET_BYTES {
            return Err(ConfigError::Invalid {
                name: "SESSION_SECRET",
                reason: "au moins 32 octets",
            });
        }
        let allowed_origins: Vec<String> = match non_empty("ALLOWED_ORIGINS") {
            Some(list) => list
                .split(',')
                .map(|o| o.trim().trim_end_matches('/').to_string())
                .filter(|o| !o.is_empty())
                .collect(),
            None => match config.public_base_url.as_deref().and_then(origin_of) {
                Some(origin) => vec![origin],
                None => vec![
                    "http://127.0.0.1:8080".to_string(),
                    "http://localhost:8080".to_string(),
                    "http://127.0.0.1:5173".to_string(),
                    "http://localhost:5173".to_string(),
                ],
            },
        };
        if allowed_origins.is_empty()
            || allowed_origins
                .iter()
                .any(|o| origin_of(o).as_deref() != Some(o.as_str()))
        {
            return Err(ConfigError::Invalid {
                name: "ALLOWED_ORIGINS",
                reason: "origines http(s)://hôte attendues, séparées par des virgules",
            });
        }
        let trust_proxy = match non_empty("TRUST_PROXY").as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    name: "TRUST_PROXY",
                    reason: "true ou false",
                });
            }
        };
        let session_idle_minutes = match non_empty("SESSION_IDLE_MINUTES") {
            None => None,
            Some(v) => {
                if config.environment == Environment::Production {
                    return Err(ConfigError::Invalid {
                        name: "SESSION_IDLE_MINUTES",
                        reason: "interdite en production",
                    });
                }
                let minutes: i64 = v.trim().parse().map_err(|_| ConfigError::Invalid {
                    name: "SESSION_IDLE_MINUTES",
                    reason: "nombre de minutes attendu",
                })?;
                if !(1..=720).contains(&minutes) {
                    return Err(ConfigError::Invalid {
                        name: "SESSION_IDLE_MINUTES",
                        reason: "entre 1 et 720",
                    });
                }
                Some(minutes)
            }
        };
        let defaults = HashParams::default();
        let argon2 = HashParams {
            memory_kib: number(&get, "ARGON2_MEMORY_KIB", defaults.memory_kib)?,
            iterations: number(&get, "ARGON2_ITERATIONS", defaults.iterations)?,
            parallelism: number(&get, "ARGON2_PARALLELISM", defaults.parallelism)?,
        };
        if !(8..=1_048_576).contains(&argon2.memory_kib)
            || !(1..=10).contains(&argon2.iterations)
            || !(1..=8).contains(&argon2.parallelism)
        {
            return Err(ConfigError::Invalid {
                name: "ARGON2_MEMORY_KIB",
                reason: "paramètres argon2id hors bornes",
            });
        }
        let field_encryption_key = match non_empty("FIELD_ENCRYPTION_KEY") {
            Some(k) => {
                if immo_app::identity::crypto::FieldKey::from_hex(&k).is_err() {
                    return Err(ConfigError::Invalid {
                        name: "FIELD_ENCRYPTION_KEY",
                        reason: "64 caractères hexadécimaux (32 octets)",
                    });
                }
                Some(Secret::new(k))
            }
            None if config.environment == Environment::Production => {
                return Err(ConfigError::Missing("FIELD_ENCRYPTION_KEY"));
            }
            None => None,
        };
        Ok(Self {
            session_secret: Secret::new(secret),
            allowed_origins,
            trust_proxy,
            min_client_version: non_empty("MIN_CLIENT_VERSION").unwrap_or_else(|| "0.0.0".to_string()),
            session_idle_minutes,
            argon2,
            field_encryption_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const DB: &str = "postgres://u:p@127.0.0.1:5433/immo";
    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn config(env: &str, extra: &[(&str, &str)]) -> Config {
        let mut pairs: HashMap<String, String> = [("APP_ENV", env), ("DATABASE_URL", DB)]
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        for (k, v) in extra {
            pairs.insert((*k).to_string(), (*v).to_string());
        }
        Config::from_lookup(move |n| pairs.get(n).cloned()).unwrap()
    }

    fn settings(c: &Config, pairs: &[(&str, &str)]) -> Result<HttpSettings, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        HttpSettings::from_lookup(c, move |n| map.get(n).cloned())
    }

    #[test]
    fn le_secret_de_session_est_obligatoire_et_assez_long() {
        let c = config("development", &[]);
        assert_eq!(settings(&c, &[]).unwrap_err(), ConfigError::Missing("SESSION_SECRET"));
        assert!(matches!(
            settings(&c, &[("SESSION_SECRET", "trop court")]),
            Err(ConfigError::Invalid {
                name: "SESSION_SECRET",
                ..
            })
        ));
        let ok = settings(&c, &[("SESSION_SECRET", SECRET)]).unwrap();
        assert!(
            !format!("{ok:?}").contains(SECRET),
            "le secret n'apparaît pas dans Debug"
        );
        assert!(!ok.trust_proxy);
        assert_eq!(ok.min_client_version, "0.0.0");
        assert_eq!(ok.argon2, HashParams::default());
        assert_eq!(ok.session_idle_minutes, None);
    }

    #[test]
    fn les_origines_viennent_de_l_adresse_publique_ou_de_la_liste() {
        let c = config(
            "staging",
            &[
                ("PUBLIC_BASE_URL", "https://essai.exemple.invalid/app"),
                ("DATABASE_URL", "postgres://u:p@h/db?sslmode=verify-full"),
            ],
        );
        let s = settings(&c, &[("SESSION_SECRET", SECRET)]).unwrap();
        assert_eq!(s.allowed_origins, ["https://essai.exemple.invalid"]);
        let s = settings(
            &c,
            &[
                ("SESSION_SECRET", SECRET),
                ("ALLOWED_ORIGINS", "https://a.invalid, https://b.invalid/"),
            ],
        )
        .unwrap();
        assert_eq!(s.allowed_origins, ["https://a.invalid", "https://b.invalid"]);
        assert!(
            settings(
                &c,
                &[("SESSION_SECRET", SECRET), ("ALLOWED_ORIGINS", "pas-une-origine")]
            )
            .is_err()
        );
        let dev = config("development", &[]);
        assert!(
            settings(&dev, &[("SESSION_SECRET", SECRET)])
                .unwrap()
                .allowed_origins
                .contains(&"http://127.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn la_duree_d_inactivite_forcee_est_interdite_en_production() {
        let prod = config(
            "production",
            &[
                ("PUBLIC_BASE_URL", "https://app.exemple.invalid"),
                ("DATABASE_URL", "postgres://u:p@h/db?sslmode=verify-full"),
            ],
        );
        assert!(matches!(
            settings(&prod, &[("SESSION_SECRET", SECRET), ("SESSION_IDLE_MINUTES", "2")]),
            Err(ConfigError::Invalid {
                name: "SESSION_IDLE_MINUTES",
                ..
            })
        ));
        let dev = config("development", &[]);
        assert_eq!(
            settings(&dev, &[("SESSION_SECRET", SECRET), ("SESSION_IDLE_MINUTES", "2")])
                .unwrap()
                .session_idle_minutes,
            Some(2)
        );
        assert!(settings(&dev, &[("SESSION_SECRET", SECRET), ("SESSION_IDLE_MINUTES", "0")]).is_err());
        assert!(settings(&dev, &[("SESSION_SECRET", SECRET), ("SESSION_IDLE_MINUTES", "abc")]).is_err());
    }

    #[test]
    fn proxy_de_confiance_et_parametres_argon2() {
        let dev = config("development", &[]);
        assert!(
            settings(&dev, &[("SESSION_SECRET", SECRET), ("TRUST_PROXY", "true")])
                .unwrap()
                .trust_proxy
        );
        assert!(settings(&dev, &[("SESSION_SECRET", SECRET), ("TRUST_PROXY", "oui")]).is_err());
        let s = settings(
            &dev,
            &[
                ("SESSION_SECRET", SECRET),
                ("ARGON2_MEMORY_KIB", "32768"),
                ("ARGON2_ITERATIONS", "3"),
            ],
        )
        .unwrap();
        assert_eq!(
            (s.argon2.memory_kib, s.argon2.iterations, s.argon2.parallelism),
            (32768, 3, 1)
        );
        assert!(settings(&dev, &[("SESSION_SECRET", SECRET), ("ARGON2_MEMORY_KIB", "1")]).is_err());
    }
}
