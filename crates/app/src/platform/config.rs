//! Configuration typée et validée au démarrage (DEPLOYMENT.md §2.4, décision n° 92).
//!
//! Aucun secret dans le dépôt : tout vient de l'environnement. Un secret n'apparaît jamais dans `Debug` ni dans un message d'erreur.

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Environnement d'exécution (`APP_ENV`) : développement local, essai, production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "development" => Ok(Self::Development),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(ConfigError::Invalid {
                name: "APP_ENV",
                reason: "attendu : development, staging ou production",
            }),
        }
    }
}

/// Valeur secrète : affichée sous la forme `***`, lisible seulement par `expose`.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    /// Le secret en clair, à n'utiliser qu'au point de connexion.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Erreurs de configuration ; le message ne cite jamais la valeur fautive.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("variable d'environnement absente : {0}")]
    Missing(&'static str),
    #[error("variable {name} invalide : {reason}")]
    Invalid { name: &'static str, reason: &'static str },
}

/// Configuration de l'application.
#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub bind_addr: SocketAddr,
    pub public_base_url: Option<String>,
    pub database_url: Secret,
    /// Dossier des fichiers : réservé au développement local (DP-70) ; refusé hors développement.
    pub file_store_dir: Option<PathBuf>,
}

impl Config {
    /// Lit la configuration depuis l'environnement du processus.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Lit la configuration depuis une fonction de recherche (testable sans toucher à l'environnement).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let non_empty = |name: &str| get(name).filter(|v| !v.trim().is_empty());
        let environment = Environment::parse(&non_empty("APP_ENV").ok_or(ConfigError::Missing("APP_ENV"))?)?;
        let bind_addr = non_empty("BIND_ADDR")
            .unwrap_or_else(|| "127.0.0.1:8080".to_string())
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::Invalid {
                name: "BIND_ADDR",
                reason: "attendu : adresse:port",
            })?;
        let database_url = non_empty("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;
        if !(database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")) {
            return Err(ConfigError::Invalid {
                name: "DATABASE_URL",
                reason: "attendu : postgres://…",
            });
        }
        let public_base_url = non_empty("PUBLIC_BASE_URL");
        let file_store_dir = non_empty("FILE_STORE_DIR").map(PathBuf::from);

        if environment != Environment::Development {
            // essai et production : TLS vérifié vers la base, adresse publique connue, aucun stockage sur disque
            if !database_url.contains("sslmode=verify-full") {
                return Err(ConfigError::Invalid {
                    name: "DATABASE_URL",
                    reason: "sslmode=verify-full exigé hors développement",
                });
            }
            if public_base_url.is_none() {
                return Err(ConfigError::Missing("PUBLIC_BASE_URL"));
            }
            if file_store_dir.is_some() {
                return Err(ConfigError::Invalid {
                    name: "FILE_STORE_DIR",
                    reason: "stockage sur disque réservé au développement (DP-70)",
                });
            }
        }
        Ok(Self {
            environment,
            bind_addr,
            public_base_url,
            database_url: Secret(database_url),
            file_store_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    const DEV_DB: &str = "postgres://immo_app:motdepasse@127.0.0.1:5433/immo";

    #[test]
    fn developpement_minimal() {
        let c = Config::from_lookup(lookup(&[("APP_ENV", "development"), ("DATABASE_URL", DEV_DB)])).unwrap();
        assert_eq!(c.environment, Environment::Development);
        assert_eq!(c.bind_addr.to_string(), "127.0.0.1:8080");
        assert!(c.file_store_dir.is_none());
    }

    #[test]
    fn le_secret_n_apparait_jamais_dans_debug() {
        let c = Config::from_lookup(lookup(&[("APP_ENV", "development"), ("DATABASE_URL", DEV_DB)])).unwrap();
        assert!(!format!("{c:?}").contains("motdepasse"));
        assert_eq!(c.database_url.expose(), DEV_DB);
    }

    #[test]
    fn variables_obligatoires() {
        assert_eq!(
            Config::from_lookup(lookup(&[])).unwrap_err(),
            ConfigError::Missing("APP_ENV")
        );
        assert_eq!(
            Config::from_lookup(lookup(&[("APP_ENV", "development")])).unwrap_err(),
            ConfigError::Missing("DATABASE_URL")
        );
        assert_eq!(
            Config::from_lookup(lookup(&[("APP_ENV", ""), ("DATABASE_URL", DEV_DB)])).unwrap_err(),
            ConfigError::Missing("APP_ENV")
        );
    }

    #[test]
    fn valeurs_invalides() {
        assert!(matches!(
            Config::from_lookup(lookup(&[("APP_ENV", "prod"), ("DATABASE_URL", DEV_DB)])),
            Err(ConfigError::Invalid { name: "APP_ENV", .. })
        ));
        assert!(matches!(
            Config::from_lookup(lookup(&[("APP_ENV", "development"), ("DATABASE_URL", "mysql://x")])),
            Err(ConfigError::Invalid {
                name: "DATABASE_URL",
                ..
            })
        ));
        assert!(matches!(
            Config::from_lookup(lookup(&[
                ("APP_ENV", "development"),
                ("DATABASE_URL", DEV_DB),
                ("BIND_ADDR", "partout")
            ])),
            Err(ConfigError::Invalid { name: "BIND_ADDR", .. })
        ));
    }

    #[test]
    fn le_message_d_erreur_ne_cite_pas_la_valeur() {
        let e = Config::from_lookup(lookup(&[
            ("APP_ENV", "development"),
            ("DATABASE_URL", "mysql://utilisateur:secret@h/x"),
        ]))
        .unwrap_err();
        assert!(!e.to_string().contains("secret"));
    }

    #[test]
    fn production_exige_tls_verifie_adresse_publique_et_pas_de_disque() {
        let base = [("APP_ENV", "production")];
        let secure = "postgres://u:p@h/db?sslmode=verify-full";
        // sans verify-full
        assert!(matches!(
            Config::from_lookup(lookup(&[
                base[0],
                ("DATABASE_URL", DEV_DB),
                ("PUBLIC_BASE_URL", "https://app.exemple.invalid")
            ])),
            Err(ConfigError::Invalid {
                name: "DATABASE_URL",
                ..
            })
        ));
        // sans adresse publique
        assert_eq!(
            Config::from_lookup(lookup(&[base[0], ("DATABASE_URL", secure)])).unwrap_err(),
            ConfigError::Missing("PUBLIC_BASE_URL")
        );
        // stockage sur disque
        assert!(matches!(
            Config::from_lookup(lookup(&[
                base[0],
                ("DATABASE_URL", secure),
                ("PUBLIC_BASE_URL", "https://app.exemple.invalid"),
                ("FILE_STORE_DIR", "/tmp/x")
            ])),
            Err(ConfigError::Invalid {
                name: "FILE_STORE_DIR",
                ..
            })
        ));
        // configuration valide
        let ok = Config::from_lookup(lookup(&[
            base[0],
            ("DATABASE_URL", secure),
            ("PUBLIC_BASE_URL", "https://app.exemple.invalid"),
        ]))
        .unwrap();
        assert_eq!(ok.environment, Environment::Production);
    }
}
