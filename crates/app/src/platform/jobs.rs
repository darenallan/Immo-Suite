//! File de tâches dans PostgreSQL (ARCHITECTURE.md §3.9, étape 01).
//!
//! - La tâche s'insère dans la **même transaction** que l'écriture métier (`enqueue` prend l'`OrgTx`) : pas de tâche orpheline.
//! - Prise par `SELECT … FOR UPDATE SKIP LOCKED` : plusieurs travailleurs ne prennent jamais la même tâche.
//! - Échec : nouvel essai avec attente croissante (30 s, 1 min, 2 min… plafonnée à 1 h) ; **alerte** au cinquième échec [valeurs proposées].
//! - Tâche `running` dont le travailleur a disparu (plus de 10 min) : reprise, donc au moins une exécution : toute tâche est idempotente.
//! - Planificateur : la clé d'unicité (`type:organisation:date locale`) empêche deux instances de créer le même jour deux fois la tâche.
//! - La charge utile ne contient que des identifiants (DATABASE_SCHEMA.md §6.5) et `last_error` aucune donnée personnelle.

// Les tables globales (`jobs`) se lisent hors contexte d'organisation : requêtes réservées à `platform` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::future::Future;

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

use super::clock::Clock;
use super::db::{Db, DbError, OrgTx};

/// Nombre d'échecs après lequel la tâche est abandonnée et l'alerte levée.
pub const MAX_ATTEMPTS: i32 = 5;
/// Une tâche `running` plus ancienne que cela est considérée comme orpheline.
pub const STALE_AFTER_MINUTES: i64 = 10;

/// Attente avant le prochain essai, après `attempts` essais déjà faits (1 pour le premier échec).
pub fn backoff(attempts: i32) -> Duration {
    let exponent = u32::try_from(attempts.saturating_sub(1).clamp(0, 7)).unwrap_or(7);
    let seconds = 30_i64.saturating_mul(1_i64 << exponent);
    Duration::seconds(seconds.min(3600))
}

/// Tâche à créer.
#[derive(Debug, Clone)]
pub struct NewJob {
    pub job_type: String,
    pub payload: Value,
    pub run_after: Option<DateTime<Utc>>,
    pub unique_key: Option<String>,
}

impl NewJob {
    pub fn new(job_type: impl Into<String>, payload: Value) -> Self {
        Self {
            job_type: job_type.into(),
            payload,
            run_after: None,
            unique_key: None,
        }
    }

    pub fn unique(mut self, key: impl Into<String>) -> Self {
        self.unique_key = Some(key.into());
        self
    }

    pub fn after(mut self, instant: DateTime<Utc>) -> Self {
        self.run_after = Some(instant);
        self
    }
}

/// Tâche prise par un travailleur.
#[derive(Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub org_id: Option<Uuid>,
    pub payload: Value,
    /// Numéro de l'essai en cours (1 au premier).
    pub attempts: i32,
}

/// Résultat d'une passe du travailleur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// Rien à faire.
    Idle,
    Done(Uuid),
    /// Échec, nouvel essai prévu.
    Retry(Uuid, DateTime<Utc>),
    /// Échec définitif (cinquième échec) : l'alerte `job.failed_permanently` a été journalisée.
    Dead(Uuid),
}

/// Exécute une tâche. Retourne un message d'erreur sans donnée personnelle en cas d'échec.
pub trait JobHandler: Send + Sync {
    fn handle(&self, job: &Job) -> impl Future<Output = Result<(), String>> + Send;
}

fn parse_uuid(text: &str) -> Result<Uuid, DbError> {
    Uuid::parse_str(text).map_err(|e| DbError::Query(sqlx::Error::Decode(Box::new(e))))
}

const INSERT_JOB: &str = "INSERT INTO jobs (type, org_id, payload, run_after, unique_key) \
     VALUES ($1, $2::uuid, $3::jsonb, COALESCE($4::timestamptz, $5::timestamptz), $6) \
     ON CONFLICT (unique_key) DO NOTHING RETURNING id::text";

/// Crée une tâche **dans la transaction de l'organisation** (même `COMMIT` que l'écriture métier).
/// Retourne `None` si la clé d'unicité existe déjà.
pub async fn enqueue(tx: &mut OrgTx, clock: &dyn Clock, job: NewJob) -> Result<Option<Uuid>, DbError> {
    let org = tx.org_id().to_string();
    insert(tx.conn(), clock, Some(org), job).await
}

/// Crée une tâche globale ou d'organisation hors transaction métier (planificateur).
pub async fn schedule(db: &Db, clock: &dyn Clock, org_id: Option<Uuid>, job: NewJob) -> Result<Option<Uuid>, DbError> {
    let mut tx = db.begin_system().await?;
    let id = insert(tx.conn(), clock, org_id.map(|o| o.to_string()), job).await?;
    tx.commit().await?;
    Ok(id)
}

async fn insert(
    conn: &mut sqlx::PgConnection,
    clock: &dyn Clock,
    org_id: Option<String>,
    job: NewJob,
) -> Result<Option<Uuid>, DbError> {
    let row = sqlx::query(INSERT_JOB)
        .bind(job.job_type)
        .bind(org_id)
        .bind(job.payload.to_string())
        .bind(job.run_after.map(|t| t.to_rfc3339()))
        .bind(clock.now().to_rfc3339())
        .bind(job.unique_key)
        .fetch_optional(conn)
        .await?;
    row.map(|r| parse_uuid(&r.get::<String, _>(0))).transpose()
}

/// Clé d'unicité d'une tâche quotidienne : une seule par type, par organisation et par jour local.
pub fn daily_key(job_type: &str, org_id: Option<Uuid>, clock: &dyn Clock, tz: Tz) -> String {
    let org = org_id.map_or_else(|| "-".to_string(), |o| o.to_string());
    format!("{job_type}:{org}:{}", clock.local_date(tz))
}

/// Instant UTC d'une heure locale du jour local courant (planifier « à 02 h 00, heure de l'organisation »).
pub fn local_time_today(clock: &dyn Clock, tz: Tz, hour: u32, minute: u32) -> DateTime<Utc> {
    use chrono::TimeZone;
    clock
        .local_date(tz)
        .and_hms_opt(hour, minute, 0)
        .and_then(|t| tz.from_local_datetime(&t).earliest())
        .map_or_else(|| clock.now(), |t| t.with_timezone(&Utc))
}

/// Vrai si la tâche quotidienne `job_type` du jour local (clé de `daily_key`) est terminée.
pub async fn daily_done(
    db: &Db,
    clock: &dyn Clock,
    job_type: &str,
    org_id: Option<Uuid>,
    tz: Tz,
) -> Result<bool, DbError> {
    let mut tx = db.begin_system().await?;
    let done: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM jobs WHERE unique_key = $1 AND state = 'done')")
        .bind(daily_key(job_type, org_id, clock, tz))
        .fetch_one(tx.conn())
        .await?;
    tx.commit().await?;
    Ok(done)
}

/// Prend la tâche suivante, ou `None`. Abandonne d'abord les tâches orphelines qui ont déjà épuisé leurs essais.
pub async fn claim(db: &Db, clock: &dyn Clock) -> Result<Option<Job>, DbError> {
    let now = clock.now();
    let stale = now - Duration::minutes(STALE_AFTER_MINUTES);
    let mut tx = db.begin_system().await?;
    let abandoned = sqlx::query(
        "UPDATE jobs SET state = 'failed', last_error = 'abandonnée : travailleur disparu après le dernier essai' \
         WHERE state = 'running' AND locked_at < $1::timestamptz AND attempts >= $2 RETURNING id::text, type",
    )
    .bind(stale.to_rfc3339())
    .bind(MAX_ATTEMPTS)
    .fetch_all(tx.conn())
    .await?;
    for row in &abandoned {
        tracing::error!(
            alert = "job.failed_permanently",
            job_id = %row.get::<String, _>(0),
            job_type = %row.get::<String, _>(1),
            "tâche abandonnée"
        );
    }
    let row = sqlx::query(
        "UPDATE jobs SET state = 'running', locked_at = $1::timestamptz, attempts = attempts + 1 \
         WHERE id = (SELECT id FROM jobs \
                      WHERE (state = 'queued' AND run_after <= $1::timestamptz) \
                         OR (state = 'running' AND locked_at < $2::timestamptz AND attempts < $3) \
                      ORDER BY run_after, created_at LIMIT 1 FOR UPDATE SKIP LOCKED) \
         RETURNING id::text, type, org_id::text, payload::text, attempts",
    )
    .bind(now.to_rfc3339())
    .bind(stale.to_rfc3339())
    .bind(MAX_ATTEMPTS)
    .fetch_optional(tx.conn())
    .await?;
    tx.commit().await?;
    let Some(row) = row else { return Ok(None) };
    let org_id = row.get::<Option<String>, _>(2).as_deref().map(parse_uuid).transpose()?;
    let payload =
        serde_json::from_str(&row.get::<String, _>(3)).map_err(|e| DbError::Query(sqlx::Error::Decode(Box::new(e))))?;
    Ok(Some(Job {
        id: parse_uuid(&row.get::<String, _>(0))?,
        job_type: row.get(1),
        org_id,
        payload,
        attempts: row.get(4),
    }))
}

/// Marque la tâche terminée.
pub async fn complete(db: &Db, job: &Job) -> Result<(), DbError> {
    let mut tx = db.begin_system().await?;
    sqlx::query("UPDATE jobs SET state = 'done', locked_at = NULL, last_error = NULL WHERE id = $1::uuid")
        .bind(job.id.to_string())
        .execute(tx.conn())
        .await?;
    tx.commit().await
}

/// Enregistre un échec : nouvel essai avec attente croissante, ou abandon et alerte au cinquième échec.
pub async fn fail(db: &Db, clock: &dyn Clock, job: &Job, error: &str) -> Result<RunOutcome, DbError> {
    let message: String = error.chars().take(500).collect();
    let mut tx = db.begin_system().await?;
    let outcome = if job.attempts >= MAX_ATTEMPTS {
        sqlx::query("UPDATE jobs SET state = 'failed', locked_at = NULL, last_error = $2 WHERE id = $1::uuid")
            .bind(job.id.to_string())
            .bind(&message)
            .execute(tx.conn())
            .await?;
        tracing::error!(
            alert = "job.failed_permanently",
            job_id = %job.id,
            job_type = %job.job_type,
            attempts = job.attempts,
            "tâche en échec définitif"
        );
        RunOutcome::Dead(job.id)
    } else {
        let at = clock.now() + backoff(job.attempts);
        sqlx::query(
            "UPDATE jobs SET state = 'queued', locked_at = NULL, last_error = $2, run_after = $3::timestamptz WHERE id = $1::uuid",
        )
        .bind(job.id.to_string())
        .bind(&message)
        .bind(at.to_rfc3339())
        .execute(tx.conn())
        .await?;
        RunOutcome::Retry(job.id, at)
    };
    tx.commit().await?;
    Ok(outcome)
}

/// Une passe : prend une tâche, l'exécute, enregistre le résultat.
pub async fn run_once<H: JobHandler>(db: &Db, clock: &dyn Clock, handler: &H) -> Result<RunOutcome, DbError> {
    let Some(job) = claim(db, clock).await? else {
        return Ok(RunOutcome::Idle);
    };
    match handler.handle(&job).await {
        Ok(()) => {
            complete(db, &job).await?;
            tracing::info!(job_id = %job.id, job_type = %job.job_type, attempts = job.attempts, "tâche terminée");
            Ok(RunOutcome::Done(job.id))
        }
        Err(error) => {
            tracing::warn!(job_id = %job.id, job_type = %job.job_type, attempts = job.attempts, "tâche en échec");
            fail(db, clock, &job, &error).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attente_croissante_plafonnee_a_une_heure() {
        let secondes: Vec<i64> = (1..=9).map(|n| backoff(n).num_seconds()).collect();
        assert_eq!(secondes, vec![30, 60, 120, 240, 480, 960, 1920, 3600, 3600]);
        assert_eq!(backoff(0).num_seconds(), 30);
    }

    #[test]
    fn cle_quotidienne_par_type_organisation_et_jour_local() {
        use crate::platform::clock::{FixedClock, OUAGADOUGOU};
        let clock = FixedClock::at_ymd_hm(2026, 11, 13, 23, 30).unwrap();
        assert_eq!(daily_key("demo", None, &clock, OUAGADOUGOU), "demo:-:2026-11-13");
        let org = Uuid::nil();
        assert_eq!(
            daily_key("rent.due", Some(org), &clock, OUAGADOUGOU),
            format!("rent.due:{org}:2026-11-13")
        );
    }
}
