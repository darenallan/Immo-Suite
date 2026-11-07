//! Travailleur de fond `immo work` (ARCHITECTURE.md §3.9) : planificateur interne puis exécution des tâches échues.
//!
//! Plusieurs instances peuvent tourner ensemble : la prise est `SKIP LOCKED` et la clé d'unicité évite les doublons du planificateur.
//! La seule tâche de l'étape 01 est `demo.noop`, idempotente par nature (elle ne fait qu'écrire au journal).

use std::process::ExitCode;
use std::time::Duration;

use immo_app::lease;
use immo_app::ledger;
use immo_app::observation::feedback;
use immo_app::observation::retention;
use immo_app::platform::clock::{Clock, OUAGADOUGOU, SystemClock};
use immo_app::platform::config::Config;
use immo_app::delivery;
use immo_app::platform::db::Db;
use immo_app::platform::file_store::DiskFileStore;
use immo_app::platform::pdf::ProcessRenderer;
use immo_app::platform::jobs::{self, Job, JobHandler, NewJob, RunOutcome};

/// Purge quotidienne des clés d'idempotence (`API_CONTRACTS.md` §5.1).
const IDEMPOTENCY_PURGE_JOB: &str = "idempotency.purge";

/// Tâches connues du binaire. Un type inconnu échoue : il sera réessayé puis alerté, jamais ignoré en silence.
struct Handler {
    db: Db,
    /// Connexion de la console (`PLATFORM_DATABASE_URL`) pour écrire les alertes dans `platform_events` ; facultative.
    platform_url: Option<String>,
    /// Stockage des PDF (`FILE_STORE_DIR`) ; absent : le rendu échoue et sera réessayé (R-049).
    store: Option<DiskFileStore>,
    /// Rendu dans le processus isolé `immo pdf-worker` (DP-76).
    renderer: std::sync::Arc<ProcessRenderer>,
}

impl JobHandler for Handler {
    async fn handle(&self, job: &Job) -> Result<(), String> {
        match job.job_type.as_str() {
            "demo.noop" => {
                tracing::info!(job_id = %job.id, "tâche de démonstration exécutée");
                Ok(())
            }
            feedback::NOTIFY_JOB => feedback::notify(&self.db, self.platform_url.as_deref(), job).await,
            retention::PURGE_JOB => {
                let n = retention::purge_expired(&self.db).await.map_err(|e| e.to_string())?;
                tracing::info!(job_id = %job.id, purged = n, "conservation de 13 mois appliquée");
                Ok(())
            }
            lease::GENERATE_JOB => {
                let n = lease::generate_due_installments(&self.db, &immo_app::platform::clock::SystemClock)
                    .await
                    .map_err(|e| e.to_string())?;
                tracing::info!(job_id = %job.id, created = n, "échéances des 30 jours à venir générées (R-029)");
                Ok(())
            }
            lease::LATE_CHECK_JOB => lease::check_generation_late(&self.db, &SystemClock, self.platform_url.as_deref())
                .await
                .map(|_| ()),
            ledger::GAPS_JOB => {
                let n = ledger::check_receipt_gaps(&self.db, self.platform_url.as_deref()).await?;
                tracing::info!(job_id = %job.id, orgs_with_gaps = n, "contrôle des trous de numérotation (DP-46)");
                Ok(())
            }
            IDEMPOTENCY_PURGE_JOB => {
                let n = ledger::purge_idempotency_keys(&self.db)
                    .await
                    .map_err(|e| e.to_string())?;
                tracing::info!(job_id = %job.id, purged = n, "clés d'idempotence de plus de 30 jours purgées");
                Ok(())
            }
            delivery::RENDER_JOB => match self.store.as_ref() {
                Some(store) => delivery::render_receipt(&self.db, store, std::sync::Arc::clone(&self.renderer), job).await,
                None => Err("stockage de fichiers non configuré (FILE_STORE_DIR)".to_string()),
            },
            delivery::PDF_CHECK_JOB => {
                let (alerts, queued) = delivery::check_missing_pdfs(&self.db, &SystemClock)
                    .await
                    .map_err(|e| e.to_string())?;
                tracing::info!(job_id = %job.id, alerts, queued, "contrôle des quittances sans PDF (R-049)");
                Ok(())
            }
            delivery::PDF_MISSING_JOB => {
                let receipt = job.payload.get("receipt_id").and_then(|v| v.as_str()).unwrap_or("");
                tracing::error!(alert = "receipt.pdf_missing", severity = "S2", receipt_id = %receipt, "quittance sans PDF après 10 minutes");
                if let Some(url) = self.platform_url.as_deref() {
                    immo_app::platform::ops::record_event(url, "receipt.pdf_missing", &serde_json::json!({ "receipt_id": receipt }))
                        .await
                        .map_err(|e| e.to_string())?;
                }
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
    // échéances des 30 jours à venir : une génération par jour local, idempotente (R-029, DP-94)
    let gen_key = jobs::daily_key(lease::GENERATE_JOB, None, clock, OUAGADOUGOU);
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(lease::GENERATE_JOB, serde_json::json!({})).unique(gen_key),
    )
    .await?;
    // contrôle de 02 h 00 (heure locale) ; un travailleur démarré plus tard laisse 5 minutes à la génération du jour
    let check_at = jobs::local_time_today(clock, OUAGADOUGOU, lease::LATE_CHECK_HOUR, 0)
        .max(clock.now() + chrono::Duration::minutes(5));
    let check_key = jobs::daily_key(lease::LATE_CHECK_JOB, None, clock, OUAGADOUGOU);
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(lease::LATE_CHECK_JOB, serde_json::json!({}))
            .unique(check_key)
            .after(check_at),
    )
    .await?;
    // contrôle des trous de numérotation à 03 h 00 locale (F10 règle 8, DP-96) et purge des clés d'idempotence
    let gaps_at = jobs::local_time_today(clock, OUAGADOUGOU, ledger::GAPS_HOUR, 0).max(clock.now());
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(ledger::GAPS_JOB, serde_json::json!({}))
            .unique(jobs::daily_key(ledger::GAPS_JOB, None, clock, OUAGADOUGOU))
            .after(gaps_at),
    )
    .await?;
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(IDEMPOTENCY_PURGE_JOB, serde_json::json!({})).unique(jobs::daily_key(
            IDEMPOTENCY_PURGE_JOB,
            None,
            clock,
            OUAGADOUGOU,
        )),
    )
    .await?;
    // quittances sans PDF : contrôle toutes les 5 minutes (DP-98), une tâche par tranche de 5 minutes
    let slot = clock.now().timestamp() / 300;
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(delivery::PDF_CHECK_JOB, serde_json::json!({})).unique(format!("{}:{slot}", delivery::PDF_CHECK_JOB)),
    )
    .await?;
    // conservation de 13 mois : une purge par jour local (F39-observation règle 10)
    let purge_key = jobs::daily_key(retention::PURGE_JOB, None, clock, OUAGADOUGOU);
    jobs::schedule(
        db,
        clock,
        None,
        NewJob::new(retention::PURGE_JOB, serde_json::json!({})).unique(purge_key),
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
    let program = std::env::current_exe().map_or_else(|_| "immo".into(), |p| p.to_string_lossy().into_owned());
    let renderer = std::sync::Arc::new(ProcessRenderer::new(
        program,
        vec!["pdf-worker".to_string()],
        Duration::from_secs(10),
        5_000_000,
        immo_pdf::VERSION,
    ));
    let handler = Handler {
        store: config.file_store_dir.clone().map(DiskFileStore::new),
        renderer,
        db: db.clone(),
        platform_url: std::env::var("PLATFORM_DATABASE_URL")
            .ok()
            .filter(|v| !v.trim().is_empty()),
    };
    tracing::info!("travailleur démarré");
    let mut stop = std::pin::pin!(tokio::signal::ctrl_c());
    loop {
        if let Err(e) = plan(&db, &clock).await {
            tracing::error!(error = %e, "planification impossible");
        }
        loop {
            match jobs::run_once(&db, &clock, &handler).await {
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
