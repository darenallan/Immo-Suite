//! Accès à la base : le seul module qui connaît `PgPool` (DP-77).
//!
//! Toute requête d'une organisation passe par `OrgTx`, qui pose `app.org_id` et `app.user_id` avec
//! `set_config(…, true)` : la valeur est locale à la transaction et disparaît au `COMMIT` ou au `ROLLBACK`.
//! La variante de session (`SET app.org_id`, `set_config(…, false)`) fuit entre organisations et est interdite.

// `PgPool`, `PgConnection` et `Pool` sont interdits partout ailleurs (clippy.toml) : ce module est leur unique usage.
#![allow(clippy::disallowed_types)]

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use std::str::FromStr;
use uuid::Uuid;

/// Erreurs d'accès à la base.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("connexion à la base impossible")]
    Connect(#[source] sqlx::Error),
    #[error("erreur de base de données")]
    Query(#[from] sqlx::Error),
}

/// Pool de connexions de l'application (rôle `immo_app`). Connexion directe, sans pooler externe (DP-55).
#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Ouvre un pool. `max_connections` borne la charge sur le plan gratuit de Neon.
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self, DbError> {
        let options = PgConnectOptions::from_str(url).map_err(DbError::Connect)?;
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await
            .map_err(DbError::Connect)?;
        Ok(Self { pool })
    }

    /// Contrôle de disponibilité (`/ready`) : une requête triviale hors contexte d'organisation.
    pub async fn ping(&self) -> Result<(), DbError> {
        // Requête sans donnée métier et sans organisation : seul usage direct du pool, réservé à ce module.
        #[allow(clippy::disallowed_methods)]
        let result = sqlx::query("SELECT 1").execute(&self.pool).await;
        result.map(|_| ()).map_err(DbError::from)
    }

    /// Ouvre une transaction `OrgTx` pour l'organisation et l'utilisateur donnés.
    pub async fn begin(&self, org_id: Uuid, user_id: Uuid) -> Result<OrgTx, DbError> {
        OrgTx::begin(self, org_id, user_id).await
    }

    /// Transaction sans contexte d'organisation, pour les tables globales (`jobs`, `sessions`). Réservée à `platform` :
    /// sur une table sous RLS, l'absence de contexte ne renvoie aucune ligne.
    pub(crate) async fn begin_system(&self) -> Result<SystemTx, DbError> {
        Ok(SystemTx {
            tx: self.pool.begin().await?,
        })
    }

    /// Transaction avec `app.user_id` seul (aucune organisation). Réservée à `identity`, pour la connexion.
    pub(crate) async fn begin_user(&self, user_id: Uuid) -> Result<UserTx, DbError> {
        let mut tx = self.pool.begin().await?;
        #[allow(clippy::disallowed_methods)]
        let query = sqlx::query("SELECT set_config('app.user_id', $1, true)").bind(user_id.to_string());
        query.execute(&mut *tx).await?;
        Ok(UserTx { tx })
    }

    /// Le pool brut, pour les tests d'étanchéité qui doivent reproduire des erreurs d'usage (variante de session).
    #[doc(hidden)]
    pub fn raw_pool_for_tests(&self) -> &PgPool {
        &self.pool
    }
}

/// Transaction sans organisation (tables globales). Visible seulement dans la caisse : jamais pour des données d'organisation.
#[derive(Debug)]
pub(crate) struct SystemTx {
    tx: Transaction<'static, Postgres>,
}

impl SystemTx {
    pub(crate) fn conn(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    pub(crate) async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await.map_err(DbError::from)
    }
}

/// Transaction avec l'utilisateur pour seul contexte (aucune organisation active) : sert à retrouver ses propres adhésions avant
/// le choix d'une organisation (politique `own_memberships`, DATABASE_SCHEMA.md §6.4).
#[derive(Debug)]
pub(crate) struct UserTx {
    tx: Transaction<'static, Postgres>,
}

impl UserTx {
    pub(crate) fn conn(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    pub(crate) async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await.map_err(DbError::from)
    }
}

/// Transaction qui porte le contexte d'organisation. Seul moyen de lire ou d'écrire des données d'une organisation.
#[derive(Debug)]
pub struct OrgTx {
    tx: Transaction<'static, Postgres>,
    org_id: Uuid,
}

impl OrgTx {
    /// Première instruction de la transaction : `set_config` local, selon DP-77.
    pub async fn begin(db: &Db, org_id: Uuid, user_id: Uuid) -> Result<Self, DbError> {
        let mut tx = db.pool.begin().await?;
        // Pose du contexte : requête paramétrée, valeurs en texte, portée de transaction (troisième argument `true`).
        #[allow(clippy::disallowed_methods)]
        let query = sqlx::query("SELECT set_config('app.org_id', $1, true), set_config('app.user_id', $2, true)")
            .bind(org_id.to_string())
            .bind(user_id.to_string());
        query.execute(&mut *tx).await?;
        Ok(Self { tx, org_id })
    }

    /// L'organisation du contexte posé.
    pub fn org_id(&self) -> Uuid {
        self.org_id
    }

    /// La connexion de la transaction, pour exécuter des requêtes dans le contexte posé.
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.tx
    }

    /// Valide la transaction : le contexte disparaît.
    pub async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await.map_err(DbError::from)
    }

    /// Annule la transaction : le contexte disparaît.
    pub async fn rollback(self) -> Result<(), DbError> {
        self.tx.rollback().await.map_err(DbError::from)
    }
}
