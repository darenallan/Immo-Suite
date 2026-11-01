//! Banc des tests de base de données (étape 01, TESTING_QA.md §3).
//!
//! Chaque test crée une base jetable sur un serveur PostgreSQL de test, y exécute `ops/sql/roles.sql` avec l'administrateur,
//! applique `migrations/` avec `immo_owner`, puis se connecte avec les trois rôles de production. Aucune donnée réelle.
//!
//! Variables : `TEST_PG_HOST` (défaut 127.0.0.1), `TEST_PG_PORT` (5433), `TEST_PG_USER` (postgres), `TEST_PG_PASSWORD`
//! (absent), `PSQL` (chemin de psql, défaut `psql`).

// Banc de test : l'accès direct au pool et aux requêtes est nécessaire pour reproduire les erreurs d'usage que les lints interdisent.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::path::{Path, PathBuf};

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Executor, PgPool, Row};
use tokio::sync::OnceCell;

/// Tables globales sans `org_id` ni RLS (DATABASE_SCHEMA.md §6.5) ; `_sqlx_migrations` est le journal des migrations.
pub const GLOBAL_TABLES: [&str; 11] = [
    "cross_org_registry",
    "users",
    "sessions",
    "login_attempts",
    "password_resets",
    "totp_backup_codes",
    "platform_staff",
    "platform_events",
    "purge_order",
    "jobs",
    "_sqlx_migrations",
];

/// Tables avec `org_id` facultatif, sans RLS (DATABASE_SCHEMA.md §6.5).
pub const OPTIONAL_ORG_TABLES: [&str; 2] = ["sessions", "jobs"];

/// Table sans `org_id` mais protégée par une politique sur `id`.
pub const ORG_ROOT_TABLE: &str = "organizations";

struct Server {
    host: String,
    port: String,
    user: String,
    password: Option<String>,
    psql: String,
    pw_owner: String,
    pw_app: String,
    pw_platform: String,
    pw_backup: String,
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn server() -> Server {
    let secret = || uuid::Uuid::new_v4().simple().to_string();
    Server {
        host: env_or("TEST_PG_HOST", "127.0.0.1"),
        port: env_or("TEST_PG_PORT", "5433"),
        user: env_or("TEST_PG_USER", "postgres"),
        password: std::env::var("TEST_PG_PASSWORD").ok(),
        psql: env_or("PSQL", "psql"),
        pw_owner: secret(),
        pw_app: secret(),
        pw_platform: secret(),
        pw_backup: secret(),
    }
}

static SERVER: OnceCell<Server> = OnceCell::const_new();

async fn srv() -> &'static Server {
    SERVER.get_or_init(|| async { server() }).await
}

fn options(s: &Server, user: &str, password: Option<&str>, db: &str) -> PgConnectOptions {
    let mut o = PgConnectOptions::new()
        .host(&s.host)
        .port(s.port.parse().unwrap())
        .username(user)
        .database(db);
    if let Some(p) = password {
        o = o.password(p);
    }
    o
}

fn url(s: &Server, user: &str, password: &str, db: &str) -> String {
    format!("postgres://{user}:{password}@{}:{}/{db}", s.host, s.port)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Une base jetable avec ses rôles et ses migrations appliquées.
pub struct TestDb {
    pub name: String,
    pub admin: PgPool,
    pub owner: PgPool,
    pub platform: PgPool,
    pub backup: PgPool,
    /// URL du rôle applicatif (`immo_app`), pour `Db::connect`.
    pub app_url: String,
    /// URL du rôle de sauvegarde (immo_backup).
    pub backup_url: String,
    /// URL du rôle de la console (immo_platform).
    pub platform_url: String,
    maintenance: PgPool,
}

impl TestDb {
    /// Crée la base, les rôles et applique toutes les migrations.
    pub async fn create() -> TestDb {
        let db = Self::create_without_migrations().await;
        db.migrate(&repo_root().join("migrations")).await.expect("migrations");
        db
    }

    /// Crée la base et les rôles seulement.
    pub async fn create_without_migrations() -> TestDb {
        let s = srv().await;
        let name = format!("immo_test_{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
        let maintenance = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options(s, &s.user, s.password.as_deref(), "postgres"))
            .await
            .expect("connexion d'administration");
        // Nettoyage, une fois par exécution : les bases jetables orphelines (test interrompu) sans connexion active sont supprimées.
        static CLEANED: OnceCell<()> = OnceCell::const_new();
        CLEANED
            .get_or_init(|| async {
                let stale: Vec<String> = sqlx::query_scalar(
                    "SELECT d.datname::text FROM pg_database d WHERE d.datname LIKE 'immo\\_test\\_%'
                        AND NOT EXISTS (SELECT 1 FROM pg_stat_activity a WHERE a.datname = d.datname)",
                )
                .fetch_all(&maintenance)
                .await
                .unwrap_or_default();
                for db in stale {
                    let _ = maintenance
                        .execute(sqlx::AssertSqlSafe(format!(
                            "DROP DATABASE IF EXISTS {db} WITH (FORCE)"
                        )))
                        .await;
                }
            })
            .await;
        maintenance
            .execute(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .await
            .expect("CREATE DATABASE");
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options(s, &s.user, s.password.as_deref(), &name))
            .await
            .expect("administrateur");

        // Les rôles sont créés par le vrai script d'exploitation, avec psql (variables `-v`).
        let roles = repo_root().join("ops/sql/roles.sql");
        static ROLES_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
        let _guard = ROLES_LOCK.lock().await; // les rôles sont communs au serveur : une exécution à la fois
        let mut cmd = tokio::process::Command::new(&s.psql);
        cmd.args([
            "-X",
            "-q",
            "-v",
            "ON_ERROR_STOP=1",
            "-h",
            &s.host,
            "-p",
            &s.port,
            "-U",
            &s.user,
            "-d",
            &name,
        ]);
        for (k, v) in [
            ("owner_pw", &s.pw_owner),
            ("app_pw", &s.pw_app),
            ("platform_pw", &s.pw_platform),
            ("backup_pw", &s.pw_backup),
        ] {
            cmd.args(["-v", &format!("{k}={v}")]);
        }
        cmd.arg("-f").arg(&roles);
        if let Some(p) = &s.password {
            cmd.env("PGPASSWORD", p);
        }
        let out = cmd.output().await.expect("psql introuvable : définir PSQL");
        assert!(
            out.status.success(),
            "roles.sql a échoué : {}",
            String::from_utf8_lossy(&out.stderr)
        );
        drop(_guard);

        let pool = |user: &str, pw: &str| {
            PgPoolOptions::new()
                .max_connections(4)
                .connect_lazy_with(options(s, user, Some(pw), &name))
        };
        TestDb {
            owner: pool("immo_owner", &s.pw_owner),
            platform: pool("immo_platform", &s.pw_platform),
            backup: pool("immo_backup", &s.pw_backup),
            app_url: url(s, "immo_app", &s.pw_app, &name),
            backup_url: url(s, "immo_backup", &s.pw_backup, &name),
            platform_url: url(s, "immo_platform", &s.pw_platform, &name),
            admin,
            maintenance,
            name,
        }
    }

    /// Applique les migrations d'un dossier avec `immo_owner`, comme le fait le déploiement.
    pub async fn migrate(&self, dir: &Path) -> Result<(), sqlx::migrate::MigrateError> {
        let migrator = sqlx::migrate::Migrator::new(dir).await?;
        migrator.run(&self.owner).await
    }

    /// Un pool pour le rôle applicatif.
    pub async fn app_pool(&self, max: u32) -> PgPool {
        let s = srv().await;
        PgPoolOptions::new()
            .max_connections(max)
            .connect_with(options(s, "immo_app", Some(&s.pw_app), &self.name))
            .await
            .expect("immo_app")
    }

    /// Crée une base vide supplémentaire (cible d'une restauration) et renvoie son URL d'administration.
    pub async fn create_empty_database(&self, name: &str) -> String {
        let s = srv().await;
        self.maintenance
            .execute(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .await
            .expect("CREATE DATABASE");
        let auth = match &s.password {
            Some(p) => format!("{}:{p}", s.user),
            None => s.user.clone(),
        };
        format!("postgres://{auth}@{}:{}/{name}", s.host, s.port)
    }

    /// Supprime une base créée par `create_empty_database`.
    pub async fn drop_database(&self, name: &str) {
        let _ = self
            .maintenance
            .execute(sqlx::AssertSqlSafe(format!(
                "DROP DATABASE IF EXISTS {name} WITH (FORCE)"
            )))
            .await;
    }

    /// Pool d'administration sur une autre base du serveur.
    pub async fn admin_pool_for(&self, name: &str) -> PgPool {
        let s = srv().await;
        PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options(s, &s.user, s.password.as_deref(), name))
            .await
            .expect("administrateur")
    }

    /// Supprime la base jetable.
    pub async fn drop_db(self) {
        self.owner.close().await;
        self.platform.close().await;
        self.backup.close().await;
        self.admin.close().await;
        let _ = self
            .maintenance
            .execute(sqlx::AssertSqlSafe(format!(
                "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                self.name
            )))
            .await;
    }
}

/// Audit du schéma (CODING_RULES.md §7 n° 10, DATABASE_SCHEMA.md §6.5, ISO-6) : renvoie la liste des écarts.
///
/// - une table sans `org_id` doit figurer sur la liste blanche, sauf `organizations` qui doit être protégée ;
/// - une table avec `org_id` (hors `sessions` et `jobs`) doit avoir la RLS activée, forcée et au moins une politique.
pub async fn schema_audit(pool: &PgPool) -> Vec<String> {
    let rows = sqlx::query(
        "SELECT c.relname::text AS name, c.relrowsecurity AS rls, c.relforcerowsecurity AS forced,
                EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid AND a.attname = 'org_id' AND NOT a.attisdropped) AS has_org,
                (SELECT count(*) FROM pg_policy p WHERE p.polrelid = c.oid) AS policies
           FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
          WHERE n.nspname = 'public' AND c.relkind IN ('r', 'p') AND NOT c.relispartition
          ORDER BY 1",
    )
    .fetch_all(pool)
    .await
    .expect("audit");
    let mut errors = vec![];
    for r in rows {
        let name: String = r.get("name");
        let (rls, forced, has_org, policies): (bool, bool, bool, i64) =
            (r.get("rls"), r.get("forced"), r.get("has_org"), r.get("policies"));
        let protected = rls && forced && policies > 0;
        if name == ORG_ROOT_TABLE {
            if !protected {
                errors.push(format!("{name} : RLS activée, forcée et politique exigées"));
            }
        } else if !has_org {
            if !GLOBAL_TABLES.contains(&name.as_str()) {
                errors.push(format!(
                    "{name} : table sans org_id absente de la liste blanche des tables globales"
                ));
            }
        } else if !OPTIONAL_ORG_TABLES.contains(&name.as_str()) && !protected {
            errors.push(format!(
                "{name} : org_id présent mais RLS {}, forcée {}, politiques {policies}",
                yn(rls),
                yn(forced)
            ));
        }
    }
    errors
}

fn yn(b: bool) -> &'static str {
    if b { "oui" } else { "NON" }
}

/// Horloge de test que le test fait avancer (DP-50 : tout le code lit l'heure par `Clock`).
pub struct TestClock(std::sync::Mutex<chrono::DateTime<chrono::Utc>>);

impl TestClock {
    /// Le 13/11/2026 à 09:15 UTC, date du jeu de démonstration.
    pub fn new() -> Self {
        use chrono::TimeZone;
        Self(std::sync::Mutex::new(
            chrono::Utc.with_ymd_and_hms(2026, 11, 13, 9, 15, 0).unwrap(),
        ))
    }

    pub fn advance(&self, d: chrono::Duration) {
        *self.0.lock().unwrap() += d;
    }
}

impl Default for TestClock {
    fn default() -> Self {
        Self::new()
    }
}

impl immo_app::platform::clock::Clock for TestClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        *self.0.lock().unwrap()
    }
}
