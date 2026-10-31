//! Travailleur de fond `immo work` (ARCHITECTURE.md §3.9) : planificateur interne puis exécution des tâches échues.
//!
//! Plusieurs instances peuvent tourner ensemble : la prise est `SKIP LOCKED` et la clé d'unicité évite les doublons du planificateur.
//! La seule tâche de l'étape 01 est `demo.noop`, idempotente par nature (elle ne fait qu'écrire au journal).

use std::process::ExitCode;
use std::time::Duration;

use immo_app::platform::clock::{Clock, OUAGADOUGOU, SystemClock};
use immo_app::platform::config::Config;
use immo_app::platform::db::Db;
use immo_app::platform::jobs::{self, Job, JobHandler, NewJob, RunOutcome};

/// Tâches connues du binaire. Un type inconnu échoue : il sera réessayé puis alerté, jamais ignoré en silence.
struct Handler;

impl JobHandler for Handler {
    async fn handle(&self, job: &Job) -> Result<(), String> {
        match job.job_type.as_str() {
            "demo.noop" => {
                tracing::info!(job_id = %job.id, "tâche de démonstration exécutée");
                Ok(())
            }
            other => Err(format!("type de tâche inconnu : {other}")),
        }
    }
}

/// Une tâche de démonstration par jour local, créée par n'importe quelle instance (clé d'unicité).
async fn plan(db: &Db, clock: &dyn Clock) -> Result<(), immo_app::platform::db::DbError> {
    let key = jobs::daily_key("demo.noop", None, clock, OUAGADOUGOU);
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new("demo.noop", serde_json::json!({})).unique(key),
    )
    .await?;
    Ok(())
}

/// Boucle : planifie, vide la file, dort. S'arrête proprement sur Ctrl+C entre deux passes.
pub async fn run() -> ExitCode {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "configuration invalide");
            return ExitCode::FAILURE;
        }
    };
    let db = match Db::connect(config.database_url.expose(), 3).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!(error = %e, "connexion à la base impossible");
            return ExitCode::FAILURE;
        }
    };
    let clock = SystemClock;
    tracing::info!("travailleur démarré");
    let mut stop = std::pin::pin!(tokio::signal::ctrl_c());
    loop {
        if let Err(e) = plan(&db, &clock).await {
            tracing::error!(error = %e, "planification impossible");
        }
        loop {
            match jobs::run_once(&db, &clock, &Handler).await {
                Ok(RunOutcome::Idle) => break,
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "passe du travailleur en erreur");
                    break;
                }
            }
        }
        tokio::select! {
            _ = &mut stop => {
                tracing::info!("arrêt demandé : travailleur arrêté entre deux passes");
                return ExitCode::SUCCESS;
            }
            () = tokio::time::sleep(Duration::from_secs(2)) => {}
        }
    }
}
