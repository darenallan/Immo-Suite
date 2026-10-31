//! File de tâches (ARCHITECTURE.md §3.9, étape 01) : atomicité avec l'écriture métier, clé d'unicité, prise concurrente sans doublon,
//! attente croissante, alerte au cinquième échec, reprise d'une tâche orpheline.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, TimeZone, Utc};
use immo_app::platform::clock::{Clock, OUAGADOUGOU};
use immo_app::platform::db::Db;
use immo_app::platform::jobs::{self, Job, JobHandler, MAX_ATTEMPTS, NewJob, RunOutcome};
use immo_db_tests::TestDb;
use sqlx::{Executor, Row};
use uuid::Uuid;

/// Horloge que le test fait avancer.
struct TestClock(Mutex<DateTime<Utc>>);

impl TestClock {
    fn new() -> Self {
        Self(Mutex::new(Utc.with_ymd_and_hms(2026, 11, 13, 9, 15, 0).unwrap()))
    }
    fn advance(&self, d: Duration) {
        let mut t = self.0.lock().unwrap();
        *t += d;
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

/// Réussit toujours et note les tâches vues.
#[derive(Default)]
struct Recorder(Mutex<Vec<Uuid>>);

impl JobHandler for Recorder {
    async fn handle(&self, job: &Job) -> Result<(), String> {
        self.0.lock().unwrap().push(job.id);
        Ok(())
    }
}

/// Échoue toujours.
struct Failing;

impl JobHandler for Failing {
    async fn handle(&self, _job: &Job) -> Result<(), String> {
        Err("échec simulé".to_string())
    }
}

async fn state_of(db: &TestDb, id: Uuid) -> (String, i32) {
    let row = sqlx::query("SELECT state, attempts FROM jobs WHERE id = $1::uuid")
        .bind(id.to_string())
        .fetch_one(&db.owner)
        .await
        .unwrap();
    (row.get(0), row.get(1))
}

#[tokio::test(flavor = "multi_thread")]
async fn la_tache_suit_la_transaction_metier() {
    let db = TestDb::create().await;
    let app = Db::connect(&db.app_url, 2).await.unwrap();
    let clock = TestClock::new();
    let org = Uuid::new_v4();

    // transaction annulée : aucune tâche orpheline
    let mut tx = app.begin(org, Uuid::new_v4()).await.unwrap();
    jobs::enqueue(
        &mut tx,
        &clock,
        NewJob::new("receipt.render", serde_json::json!({"receipt_id": "r1"})),
    )
    .await
    .unwrap()
    .unwrap();
    tx.rollback().await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs")
        .fetch_one(&db.owner)
        .await
        .unwrap();
    assert_eq!(n, 0, "rollback : pas de tâche");

    // transaction validée : la tâche existe, rattachée à l'organisation du contexte
    let mut tx = app.begin(org, Uuid::new_v4()).await.unwrap();
    let id = jobs::enqueue(
        &mut tx,
        &clock,
        NewJob::new("receipt.render", serde_json::json!({"receipt_id": "r1"})),
    )
    .await
    .unwrap()
    .unwrap();
    tx.commit().await.unwrap();
    let row = sqlx::query("SELECT org_id::text, state, payload->>'receipt_id' FROM jobs WHERE id = $1::uuid")
        .bind(id.to_string())
        .fetch_one(&db.owner)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>(0), org.to_string());
    assert_eq!(row.get::<String, _>(1), "queued");
    assert_eq!(row.get::<String, _>(2), "r1");
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn la_cle_d_unicite_empeche_le_doublon_du_jour() {
    let db = TestDb::create().await;
    let app = Db::connect(&db.app_url, 4).await.unwrap();
    let clock = TestClock::new();
    let org = Uuid::new_v4();
    let key = jobs::daily_key("rent.generate_due", Some(org), &clock, OUAGADOUGOU);

    // dix « instances » du planificateur en même temps : une seule tâche
    let mut handles = Vec::new();
    for _ in 0..10 {
        let app = app.clone();
        let key = key.clone();
        let now = clock.now();
        handles.push(tokio::spawn(async move {
            let fixed = immo_app::platform::clock::FixedClock::at(now);
            jobs::schedule(
                &app,
                &fixed,
                Some(org),
                NewJob::new("rent.generate_due", serde_json::json!({})).unique(key),
            )
            .await
            .unwrap()
        }));
    }
    let mut created = 0;
    for h in handles {
        created += usize::from(h.await.unwrap().is_some());
    }
    assert_eq!(created, 1, "une seule instance crée la tâche du jour");

    // le lendemain : une nouvelle tâche
    clock.advance(Duration::days(1));
    let next = jobs::daily_key("rent.generate_due", Some(org), &clock, OUAGADOUGOU);
    assert_ne!(next, key);
    let id = jobs::schedule(
        &app,
        &clock,
        Some(org),
        NewJob::new("rent.generate_due", serde_json::json!({})).unique(next),
    )
    .await
    .unwrap();
    assert!(id.is_some());
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn huit_travailleurs_ne_prennent_jamais_la_meme_tache() {
    let db = TestDb::create().await;
    let app = Db::connect(&db.app_url, 10).await.unwrap();
    let clock = Arc::new(TestClock::new());
    let recorder = Arc::new(Recorder::default());
    for i in 0..200 {
        jobs::schedule(
            &app,
            clock.as_ref(),
            None,
            NewJob::new("demo.noop", serde_json::json!({ "n": i })),
        )
        .await
        .unwrap();
    }
    let mut handles = Vec::new();
    for _ in 0..8 {
        let (app, clock, recorder) = (app.clone(), clock.clone(), recorder.clone());
        handles.push(tokio::spawn(async move {
            while jobs::run_once(&app, clock.as_ref(), recorder.as_ref()).await.unwrap() != RunOutcome::Idle {}
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    let seen = recorder.0.lock().unwrap().clone();
    let distinct: HashSet<_> = seen.iter().copied().collect();
    assert_eq!(seen.len(), 200, "chaque tâche est exécutée");
    assert_eq!(distinct.len(), 200, "aucune tâche n'est exécutée deux fois");
    let done: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs WHERE state = 'done' AND attempts = 1")
        .fetch_one(&db.owner)
        .await
        .unwrap();
    assert_eq!(done, 200);
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn echecs_attente_croissante_puis_abandon_au_cinquieme() {
    let db = TestDb::create().await;
    let app = Db::connect(&db.app_url, 2).await.unwrap();
    let clock = TestClock::new();
    let id = jobs::schedule(&app, &clock, None, NewJob::new("demo.noop", serde_json::json!({})))
        .await
        .unwrap()
        .unwrap();

    let expected_wait = [30, 60, 120, 240];
    for (i, wait) in expected_wait.iter().enumerate() {
        let outcome = jobs::run_once(&app, &clock, &Failing).await.unwrap();
        let RunOutcome::Retry(got, at) = outcome else {
            panic!("essai {} : {outcome:?}", i + 1)
        };
        assert_eq!(got, id);
        assert_eq!(at, clock.now() + Duration::seconds(*wait));
        // avant l'heure prévue, la tâche n'est pas reprise
        assert_eq!(jobs::run_once(&app, &clock, &Failing).await.unwrap(), RunOutcome::Idle);
        assert_eq!(
            state_of(&db, id).await,
            ("queued".to_string(), i32::try_from(i + 1).unwrap())
        );
        clock.advance(Duration::seconds(*wait));
    }
    // cinquième échec : abandon définitif et message conservé
    assert_eq!(
        jobs::run_once(&app, &clock, &Failing).await.unwrap(),
        RunOutcome::Dead(id)
    );
    assert_eq!(state_of(&db, id).await, ("failed".to_string(), MAX_ATTEMPTS));
    let error: String = sqlx::query_scalar("SELECT last_error FROM jobs WHERE id = $1::uuid")
        .bind(id.to_string())
        .fetch_one(&db.owner)
        .await
        .unwrap();
    assert_eq!(error, "échec simulé");
    clock.advance(Duration::days(1));
    assert_eq!(
        jobs::run_once(&app, &clock, &Failing).await.unwrap(),
        RunOutcome::Idle,
        "plus jamais reprise"
    );
    db.drop_db().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn une_tache_orpheline_est_reprise_puis_abandonnee_a_la_limite() {
    let db = TestDb::create().await;
    let app = Db::connect(&db.app_url, 2).await.unwrap();
    let clock = TestClock::new();
    let recorder = Recorder::default();
    let stale = (clock.now() - Duration::minutes(11)).to_rfc3339();
    let fresh = (clock.now() - Duration::minutes(2)).to_rfc3339();
    // trois tâches « running » : orpheline au 1er essai, récente, orpheline au dernier essai
    for (attempts, locked) in [(1, &stale), (1, &fresh), (MAX_ATTEMPTS, &stale)] {
        db.owner
            .execute(
                sqlx::query(
                    "INSERT INTO jobs (type, state, attempts, locked_at) VALUES ('demo.noop', 'running', $1, $2::timestamptz)",
                )
                .bind(attempts)
                .bind(locked),
            )
            .await
            .unwrap();
    }
    assert!(matches!(
        jobs::run_once(&app, &clock, &recorder).await.unwrap(),
        RunOutcome::Done(_)
    ));
    assert_eq!(jobs::run_once(&app, &clock, &recorder).await.unwrap(), RunOutcome::Idle);
    assert_eq!(
        recorder.0.lock().unwrap().len(),
        1,
        "seule l'orpheline du 1er essai est reprise"
    );
    let rows = sqlx::query("SELECT state, attempts FROM jobs ORDER BY attempts, state")
        .fetch_all(&db.owner)
        .await
        .unwrap();
    let states: Vec<(String, i32)> = rows.iter().map(|r| (r.get(0), r.get(1))).collect();
    assert_eq!(
        states,
        vec![
            ("running".to_string(), 1),
            ("done".to_string(), 2),
            ("failed".to_string(), MAX_ATTEMPTS),
        ]
    );
    db.drop_db().await;
}
