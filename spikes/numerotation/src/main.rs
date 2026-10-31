//! Prototype (c) : numérotation des quittances sans trou et idempotence sous concurrence.
//! Protocole : `spikes/numerotation/NOTE.md`. Hors build du produit, aucune donnée réelle.
//!
//! Usage :
//!   spike-numerotation setup <local|neon>
//!   spike-numerotation serve <local|neon> [port]
//!   spike-numerotation load  <local|neon> <e10|main|attempts|idem|samelease|e27|verify|stats> [url]

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::{
    Connection, Executor, PgConnection, PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ORGS: [&str; 3] = [
    "a0000000-0000-4000-8000-000000000001",
    "b0000000-0000-4000-8000-000000000002",
    "c0000000-0000-4000-8000-000000000003",
];
const ORG_D: &str = "d0000000-0000-4000-8000-000000000004"; // E-10, E-27 et idempotence
const USER: &str = "11111111-1111-4111-8111-111111111111";
const ROUTE: &str = "POST /spike/payments";

// ───────────────────────── configuration (comme le prototype b) ─────────────────────────

struct Target {
    name: String,
    env_path: PathBuf,
    env: HashMap<String, String>,
    admin: PgConnectOptions,
}

impl Target {
    fn load(name: &str) -> R<Self> {
        let (dir, key, default) = match name {
            "local" => ("immo-spikes-pg", "LOCAL_C_ADMIN_URL", Some("postgres://postgres@127.0.0.1:5433/spike_c")),
            "neon" => ("immo-spikes-neon", "NEON_C_ADMIN_URL", None),
            _ => return Err("cible inconnue : local ou neon".into()),
        };
        let env_path = PathBuf::from(std::env::var("LOCALAPPDATA")?).join(dir).join(".env.local");
        let mut env = HashMap::new();
        for line in std::fs::read_to_string(&env_path)?.lines() {
            if let Some((k, v)) = line.split_once('=') {
                env.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        let url = env.get(key).cloned().or(default.map(String::from)).ok_or_else(|| format!("{key} absent"))?;
        Ok(Self { name: name.into(), env_path, env, admin: PgConnectOptions::from_str(&url)? })
    }

    fn opts(&self, role: &str) -> R<PgConnectOptions> {
        let pw = self.env.get(&format!("PW_{}", role.to_uppercase())).ok_or("mot de passe de rôle absent : lancer le prototype (b) d'abord")?;
        Ok(self.admin.clone().username(role).password(pw))
    }
}

// ───────────────────────── horloge injectée ─────────────────────────

trait Clock: Send + Sync {
    /// Année d'émission au fuseau de l'organisation (Africa/Ouagadougou = UTC+0).
    fn year(&self) -> i32;
}

/// Horloge système, avec année imposée pour rejouer E-27 sans attendre le 1er janvier.
struct TestClock(Mutex<Option<i32>>);

impl Clock for TestClock {
    fn year(&self) -> i32 {
        if let Some(y) = *self.0.lock().unwrap() {
            return y;
        }
        let days = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / 86_400) as i64;
        // civil_from_days (H. Hinnant), suffisant pour l'année
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (yoe + era * 400 + i64::from(m <= 2)) as i32
    }
}

// ───────────────────────── serveur ─────────────────────────

#[derive(Default)]
struct Stats {
    server_ms: Mutex<Vec<f64>>,
    lock_hold_ms: Mutex<Vec<f64>>,
    validated: AtomicU64,
    forced_rollbacks: AtomicU64,
    replays: AtomicU64,
    rejected_422: AtomicU64,
    deadlocks: AtomicU64,
    other_errors: AtomicU64,
}

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    clock: Arc<TestClock>,
    stats: Arc<Stats>,
}

#[derive(Deserialize, Serialize)]
struct PaymentReq {
    lease_id: String,
    amount_fcfa: i64,
    paid_on: String,
    series: String,
}

#[derive(Serialize)]
struct PaymentResp<'a> {
    payment_id: &'a str,
    receipt_number: &'a str,
    sequence: i32,
    year: i32,
}

enum Outcome {
    Done(u16, String),
    Injected(&'static str),
    DropResponse,
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() as f64 - 1.0) * p).round() as usize]
}

fn err_code(e: &sqlx::Error) -> String {
    e.as_database_error().and_then(|d| d.code().map(|c| c.to_string())).unwrap_or_default()
}

async fn begin_org(pool: &PgPool, org: &str) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.org_id', $1, true), set_config('app.user_id', $2, true)")
        .bind(org)
        .bind(USER)
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

async fn validate(st: &AppState, org: &str, key: &str, hash: &str, req: &PaymentReq, fail: &str) -> Result<Outcome, sqlx::Error> {
    let t0 = Instant::now();
    let mut tx = begin_org(&st.pool, org).await?;

    // 1. idempotence : la ligne de clé est prise la première ; un doublon attend la fin de la première transaction
    let fresh = sqlx::query("INSERT INTO idempotency_keys (org_id, key, route, request_hash) VALUES (current_org_id(), $1, $2, $3) ON CONFLICT (org_id, key) DO NOTHING")
        .bind(key)
        .bind(ROUTE)
        .bind(hash)
        .execute(&mut *tx)
        .await?
        .rows_affected()
        == 1;
    if !fresh {
        let row = sqlx::query("SELECT route, request_hash, response_status, response_body FROM idempotency_keys WHERE key = $1")
            .bind(key)
            .fetch_one(&mut *tx)
            .await?;
        let (route, h, status, body): (String, String, Option<i32>, Option<String>) = (row.get(0), row.get(1), row.get(2), row.get(3));
        tx.rollback().await?;
        if route != ROUTE || h != hash {
            st.stats.rejected_422.fetch_add(1, Ordering::Relaxed);
            return Ok(Outcome::Done(422, r#"{"error":"idempotency.key_reused_with_different_body"}"#.into()));
        }
        st.stats.replays.fetch_add(1, Ordering::Relaxed);
        return Ok(Outcome::Done(status.unwrap_or(409) as u16, body.unwrap_or_default()));
    }

    // 2. verrou consultatif par bail, pris AVANT le compteur
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))").bind(&req.lease_id).execute(&mut *tx).await?;

    // 3. panne injectée avant toute validation (cas iv)
    if fail == "before" {
        tx.rollback().await?;
        return Ok(Outcome::Injected("before"));
    }

    let payment_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO payments (id, org_id, lease_id, amount_fcfa, paid_on, idempotency_key) VALUES ($1::uuid, current_org_id(), $2::uuid, $3, $4::date, $5)")
        .bind(&payment_id)
        .bind(&req.lease_id)
        .bind(req.amount_fcfa)
        .bind(&req.paid_on)
        .bind(key)
        .execute(&mut *tx)
        .await?;

    // 4. le compteur, en dernier (ARCHITECTURE.md §4.4). L'année est celle de l'émission (DP-61), lue par la Clock.
    let year = st.clock.year();
    let n: i32 = sqlx::query_scalar(
        "INSERT INTO receipt_counters (org_id, series, year, last_number) VALUES (current_org_id(), $1, $2, 1)
         ON CONFLICT (org_id, series, year) DO UPDATE SET last_number = receipt_counters.last_number + 1
         RETURNING last_number",
    )
    .bind(&req.series)
    .bind(year)
    .fetch_one(&mut *tx)
    .await?;
    let t_counter = Instant::now();

    // 5. annulation forcée APRÈS l'attribution du numéro : aucun trou ne doit rester
    if fail == "after_number" {
        tx.rollback().await?;
        st.stats.forced_rollbacks.fetch_add(1, Ordering::Relaxed);
        return Ok(Outcome::Injected("after_number"));
    }

    let receipt_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO receipts (id, org_id, series, year, sequence, payment_id, validated_at) VALUES ($1::uuid, current_org_id(), $2, $3, $4, $5::uuid, clock_timestamp())")
        .bind(&receipt_id)
        .bind(&req.series)
        .bind(year)
        .bind(n)
        .bind(&payment_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO audit_log (org_id, action, entity_id) VALUES (current_org_id(), 'receipt.created', $1::uuid)")
        .bind(&receipt_id)
        .execute(&mut *tx)
        .await?;

    let prefix = if req.series == "trial" { "ESSAI-" } else { "" };
    let number = format!("{prefix}{year}-{n:06}");
    let body = serde_json::to_string(&PaymentResp { payment_id: &payment_id, receipt_number: &number, sequence: n, year }).unwrap();
    sqlx::query("UPDATE idempotency_keys SET response_status = 201, response_body = $2 WHERE key = $1").bind(key).bind(&body).execute(&mut *tx).await?;
    tx.commit().await?;

    st.stats.validated.fetch_add(1, Ordering::Relaxed);
    st.stats.server_ms.lock().unwrap().push(t0.elapsed().as_secs_f64() * 1000.0);
    st.stats.lock_hold_ms.lock().unwrap().push(t_counter.elapsed().as_secs_f64() * 1000.0);
    if fail == "after_commit" {
        return Ok(Outcome::DropResponse); // la réponse « se perd » : le client réessaie avec la même clé
    }
    Ok(Outcome::Done(201, body))
}

async fn post_payment(State(st): State<AppState>, headers: HeaderMap, body: String) -> Response {
    let h = |n: &str| headers.get(n).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let (key, org, fail) = (h("idempotency-key"), h("x-org-id"), h("x-spike-fail"));
    if key.is_empty() || org.is_empty() {
        return (StatusCode::BAD_REQUEST, r#"{"error":"missing_header"}"#).into_response();
    }
    let Ok(req) = serde_json::from_str::<PaymentReq>(&body) else {
        return (StatusCode::BAD_REQUEST, r#"{"error":"bad_body"}"#).into_response();
    };
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    // une panne de sérialisation ou d'interblocage se rejoue une fois, comme le ferait le front avec la même clé
    match validate(&st, &org, &key, &hash, &req, &fail).await {
        Ok(Outcome::Done(code, body)) => (StatusCode::from_u16(code).unwrap(), [("content-type", "application/json")], body).into_response(),
        Ok(Outcome::Injected(w)) => (StatusCode::INTERNAL_SERVER_ERROR, format!(r#"{{"error":"injected_failure_{w}"}}"#)).into_response(),
        Ok(Outcome::DropResponse) => StatusCode::GATEWAY_TIMEOUT.into_response(),
        Err(e) => {
            if err_code(&e) == "40P01" {
                st.stats.deadlocks.fetch_add(1, Ordering::Relaxed);
            } else {
                st.stats.other_errors.fetch_add(1, Ordering::Relaxed);
            }
            (StatusCode::INTERNAL_SERVER_ERROR, format!(r#"{{"error":"db","code":"{}","detail":"{}"}}"#, err_code(&e), e.to_string().replace('"', "'"))).into_response()
        }
    }
}

async fn post_clock(State(st): State<AppState>, body: String) -> Response {
    let year = serde_json::from_str::<serde_json::Value>(&body).ok().and_then(|v| v["year"].as_i64()).map(|y| y as i32);
    *st.clock.0.lock().unwrap() = year;
    format!(r#"{{"year":{}}}"#, st.clock.year()).into_response()
}

async fn get_stats(State(st): State<AppState>) -> String {
    let s = &st.stats;
    let (mut a, mut b) = (s.server_ms.lock().unwrap().clone(), s.lock_hold_ms.lock().unwrap().clone());
    format!(
        r#"{{"validated":{},"forced_rollbacks":{},"replays":{},"rejected_422":{},"deadlocks":{},"other_errors":{},"server_p50":{:.1},"server_p95":{:.1},"server_p99":{:.1},"lock_p50":{:.1},"lock_p95":{:.1}}}"#,
        s.validated.load(Ordering::Relaxed),
        s.forced_rollbacks.load(Ordering::Relaxed),
        s.replays.load(Ordering::Relaxed),
        s.rejected_422.load(Ordering::Relaxed),
        s.deadlocks.load(Ordering::Relaxed),
        s.other_errors.load(Ordering::Relaxed),
        pct(&mut a, 0.5),
        pct(&mut a, 0.95),
        pct(&mut a, 0.99),
        pct(&mut b, 0.5),
        pct(&mut b, 0.95)
    )
}

async fn serve(t: &Target, port: u16) -> R<()> {
    let pool = PgPoolOptions::new().max_connections(20).connect_with(t.opts("immo_app")?).await?;
    let st = AppState { pool, clock: Arc::new(TestClock(Mutex::new(None))), stats: Arc::new(Stats::default()) };
    let app = Router::new()
        .route("/spike/payments", post(post_payment))
        .route("/spike/clock", post(post_clock))
        .route("/spike/stats", get(get_stats))
        .with_state(st);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    println!("serveur de {} sur http://127.0.0.1:{port}", t.name);
    axum::serve(listener, app).await?;
    Ok(())
}

// ───────────────────────── installation ─────────────────────────

async fn setup(t: &Target) -> R<()> {
    if t.name == "local" {
        let mut c = PgConnection::connect_with(&t.admin.clone().database("postgres")).await?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = 'spike_c')").fetch_one(&mut c).await?;
        if !exists {
            c.execute("CREATE DATABASE spike_c").await?;
        }
        let mut a = PgConnection::connect_with(&t.admin).await?;
        a.execute("GRANT ALL ON SCHEMA public TO immo_owner").await?;
        a.execute("GRANT USAGE ON SCHEMA public TO immo_app").await?;
    }
    let mut o = PgConnection::connect_with(&t.opts("immo_owner")?).await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(std::fs::read_to_string("sql/schema.sql")?)).execute(&mut o).await?;
    let app = PgPoolOptions::new().max_connections(2).connect_with(t.opts("immo_app")?).await?;
    for (i, org) in ORGS.iter().chain([&ORG_D]).enumerate() {
        let mut tx = begin_org(&app, org).await?;
        sqlx::query("INSERT INTO organizations (id, name) VALUES ($1::uuid, $2)").bind(org).bind(format!("Agence {}", i + 1)).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO leases (org_id, label) SELECT $1::uuid, 'bail-' || g FROM generate_series(1, 50) g").bind(org).execute(&mut *tx).await?;
        tx.commit().await?;
    }
    println!("schéma du prototype (c) créé ; 4 organisations de 50 baux chacune");
    Ok(())
}

// ───────────────────────── générateur de charge ─────────────────────────

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

async fn leases_of(t: &Target, org: &str) -> R<Vec<String>> {
    let mut a = PgConnection::connect_with(&t.admin).await?;
    Ok(sqlx::query_scalar("SELECT id::text FROM leases WHERE org_id = $1::uuid ORDER BY label").bind(org).fetch_all(&mut a).await?)
}

struct Reply {
    status: u16,
    body: String,
    ms: f64,
}

async fn send_payment(c: &reqwest::Client, url: &str, org: &str, key: &str, req: &PaymentReq, fail: &str) -> Reply {
    let t0 = Instant::now();
    let mut r = c
        .post(format!("{url}/spike/payments"))
        .header("idempotency-key", key)
        .header("x-org-id", org)
        .header("content-type", "application/json")
        .body(serde_json::to_string(req).unwrap());
    if !fail.is_empty() {
        r = r.header("x-spike-fail", fail);
    }
    match r.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            Reply { status, body: resp.text().await.unwrap_or_default(), ms: t0.elapsed().as_secs_f64() * 1000.0 }
        }
        Err(e) => Reply { status: 0, body: e.to_string(), ms: t0.elapsed().as_secs_f64() * 1000.0 },
    }
}

fn req(lease: &str, series: &str, amount: i64) -> PaymentReq {
    PaymentReq { lease_id: lease.into(), amount_fcfa: amount, paid_on: "2026-11-05".into(), series: series.into() }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder().pool_max_idle_per_host(32).build().unwrap()
}

/// Charge de référence : 20 clients sur 3 organisations, 10 % d'annulations forcées après le numéro.
/// `until_validated = Some(n)` continue jusqu'à n quittances validées ; sinon `attempts` tentatives par client.
async fn run_load(t: &Target, url: &str, series: &'static str, until_validated: Option<u64>, attempts: u64) -> R<()> {
    let mut leases = vec![];
    for org in ORGS {
        leases.push(leases_of(t, org).await?);
    }
    let leases = Arc::new(leases);
    let validated = Arc::new(AtomicU64::new(0));
    let sent = Arc::new(AtomicU64::new(0));
    let failed_unexpected = Arc::new(AtomicU64::new(0));
    let lat = Arc::new(Mutex::new(Vec::<f64>::new()));
    let c = http_client();
    let t0 = Instant::now();
    let mut hs = vec![];
    for client in 0..20u64 {
        let (c, url, leases, validated, sent, bad, lat) = (c.clone(), url.to_string(), leases.clone(), validated.clone(), sent.clone(), failed_unexpected.clone(), lat.clone());
        hs.push(tokio::spawn(async move {
            let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (client + 1) * 0x1000_0001);
            let mut i = 0u64;
            loop {
                match until_validated {
                    Some(n) if validated.load(Ordering::Relaxed) >= n => break,
                    None if i >= attempts => break,
                    _ => {}
                }
                i += 1;
                let org_idx = (client % 3) as usize;
                let lease = &leases[org_idx][(rng.next() % 50) as usize];
                let forced = rng.next() % 10 == 0;
                let key = uuid::Uuid::new_v4().to_string();
                let r = send_payment(&c, &url, ORGS[org_idx], &key, &req(lease, series, 50_000 + (rng.next() % 100) as i64 * 1000), if forced { "after_number" } else { "" }).await;
                sent.fetch_add(1, Ordering::Relaxed);
                lat.lock().unwrap().push(r.ms);
                match (forced, r.status) {
                    (false, 201) => {
                        validated.fetch_add(1, Ordering::Relaxed);
                    }
                    (true, 500) => {}
                    _ => {
                        bad.fetch_add(1, Ordering::Relaxed);
                        eprintln!("réponse inattendue (forcée={forced}) : {} {}", r.status, r.body);
                    }
                }
                let s = sent.load(Ordering::Relaxed);
                if s % 500 == 0 {
                    println!("  … {s} tentatives, {} validées, {:.0} s", validated.load(Ordering::Relaxed), t0.elapsed().as_secs_f64());
                }
            }
        }));
    }
    for h in hs {
        h.await?;
    }
    let total = t0.elapsed().as_secs_f64();
    let mut l = lat.lock().unwrap().clone();
    println!(
        "série {series} : {} tentatives, {} validées, {} réponses inattendues, {:.0} s, débit {:.1} validations/s\n  latence client (ms) p50 = {:.0}, p95 = {:.0}, p99 = {:.0}",
        sent.load(Ordering::Relaxed),
        validated.load(Ordering::Relaxed),
        failed_unexpected.load(Ordering::Relaxed),
        total,
        validated.load(Ordering::Relaxed) as f64 / total,
        pct(&mut l, 0.5),
        pct(&mut l, 0.95),
        pct(&mut l, 0.99)
    );
    Ok(())
}

async fn verify(t: &Target) -> R<()> {
    let mut a = PgConnection::connect_with(&t.admin).await?;
    let rows = sqlx::query(
        "SELECT r.org_id::text, r.series, r.year, count(*) AS n, min(r.sequence), max(r.sequence), count(DISTINCT r.sequence),
                (SELECT last_number FROM receipt_counters c WHERE c.org_id = r.org_id AND c.series = r.series AND c.year = r.year),
                (SELECT count(*) FROM (SELECT validated_at, lag(validated_at) OVER (ORDER BY sequence) AS prev
                                        FROM receipts x WHERE x.org_id = r.org_id AND x.series = r.series AND x.year = r.year) o
                  WHERE o.prev > o.validated_at)
         FROM receipts r GROUP BY r.org_id, r.series, r.year ORDER BY 1, 2, 3",
    )
    .fetch_all(&mut a)
    .await?;
    let mut ok = true;
    println!("{:<10}{:<10}{:<6}{:>8}{:>6}{:>8}{:>10}{:>8}{:>10}  verdict", "org", "série", "année", "quitt.", "min", "max", "distincts", "compteur", "ordre KO");
    for r in rows {
        let (org, series, year, n, min, max, distinct, last, order_ko): (String, String, i32, i64, i32, i32, i64, Option<i32>, i64) =
            (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4), r.get(5), r.get(6), r.get(7), r.get(8));
        let good = min == 1 && i64::from(max) == n && distinct == n && last == Some(max) && order_ko == 0;
        ok &= good;
        println!("{:<10}{:<10}{:<6}{:>8}{:>6}{:>8}{:>10}{:>8}{:>10}  {}", &org[..1], series, year, n, min, max, distinct, last.unwrap_or(-1), order_ko, if good { "OK" } else { "ÉCART" });
    }
    let (receipts, audits, payments): (i64, i64, i64) = (
        sqlx::query_scalar("SELECT count(*) FROM receipts").fetch_one(&mut a).await?,
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'receipt.created'").fetch_one(&mut a).await?,
        sqlx::query_scalar("SELECT count(*) FROM payments").fetch_one(&mut a).await?,
    );
    let orphans: i64 = sqlx::query_scalar("SELECT count(*) FROM payments p WHERE NOT EXISTS (SELECT 1 FROM receipts r WHERE r.payment_id = p.id)").fetch_one(&mut a).await?;
    println!("quittances = {receipts}, lignes d'audit = {audits}, paiements = {payments}, paiements sans quittance = {orphans}");
    ok &= receipts == audits && orphans == 0 && payments == receipts;
    println!("VERDICT : {}", if ok { "0 trou, 0 doublon, numéros consécutifs, ordre respecté" } else { "ÉCART DÉTECTÉ" });
    Ok(())
}

async fn count_payments(t: &Target, key: &str) -> R<i64> {
    let mut a = PgConnection::connect_with(&t.admin).await?;
    Ok(sqlx::query_scalar("SELECT count(*) FROM payments WHERE idempotency_key = $1").bind(key).fetch_one(&mut a).await?)
}

async fn load(t: &Target, scenario: &str, url: &str) -> R<()> {
    let c = http_client();
    let d_leases = leases_of(t, ORG_D).await?;
    match scenario {
        "e10" => {
            let r1 = send_payment(&c, url, ORG_D, &uuid::Uuid::new_v4().to_string(), &req(&d_leases[0], "trial", 75_000), "").await;
            let r2 = send_payment(&c, url, ORG_D, &uuid::Uuid::new_v4().to_string(), &req(&d_leases[0], "trial", 75_000), "").await;
            println!("E-10 : {} {}\n       {} {}", r1.status, r1.body, r2.status, r2.body);
        }
        "main" => run_load(t, url, "official", Some(10_000), 0).await?,
        "attempts" => run_load(t, url, "trial", None, 500).await?,
        "idem" => {
            let (mut same, mut one_payment, mut same_number) = (0, 0, 0);
            for i in 0..100 {
                let key = uuid::Uuid::new_v4().to_string();
                let lease = &d_leases[i % 50];
                let rq = req(lease, "trial", 60_000);
                let (a, b, cc) = tokio::join!(send_payment(&c, url, ORG_D, &key, &rq, ""), send_payment(&c, url, ORG_D, &key, &rq, ""), send_payment(&c, url, ORG_D, &key, &rq, ""));
                if a.status == 201 && a.body == b.body && b.body == cc.body && b.status == 201 && cc.status == 201 {
                    same += 1;
                    same_number += 1;
                } else {
                    println!("  écart (i) : {} {} | {} {} | {} {}", a.status, a.body, b.status, b.body, cc.status, cc.body);
                }
                if count_payments(t, &key).await? == 1 {
                    one_payment += 1;
                }
            }
            println!("(i) 100 clés × 3 requêtes simultanées : réponses identiques {same}/100, un seul paiement {one_payment}/100, même numéro {same_number}/100");
            let key = uuid::Uuid::new_v4().to_string();
            let first = send_payment(&c, url, ORG_D, &key, &req(&d_leases[0], "trial", 60_000), "").await;
            let other = send_payment(&c, url, ORG_D, &key, &req(&d_leases[0], "trial", 61_000), "").await;
            println!("(ii) même clé, contenu différent : {} {} ; paiements = {} (attendu 422 et 1)", other.status, other.body, count_payments(t, &key).await?);
            let _ = first;
            let key = uuid::Uuid::new_v4().to_string();
            let lost = send_payment(&c, url, ORG_D, &key, &req(&d_leases[1], "trial", 60_000), "after_commit").await;
            let retry = send_payment(&c, url, ORG_D, &key, &req(&d_leases[1], "trial", 60_000), "").await;
            println!("(iii) réponse perdue : essai 1 = {} ; essai 2 = {} {} ; paiements = {} (attendu 504, 201 et 1)", lost.status, retry.status, retry.body, count_payments(t, &key).await?);
            let key = uuid::Uuid::new_v4().to_string();
            let cut = send_payment(&c, url, ORG_D, &key, &req(&d_leases[2], "trial", 60_000), "before").await;
            let after_cut = count_payments(t, &key).await?;
            let retry = send_payment(&c, url, ORG_D, &key, &req(&d_leases[2], "trial", 60_000), "").await;
            println!("(iv) coupure avant validation : essai 1 = {} (paiements = {after_cut}) ; essai 2 = {} {} ; paiements = {} (attendu 500/0, 201, 1)", cut.status, retry.status, retry.body, count_payments(t, &key).await?);
        }
        "samelease" => {
            // 20 paiements simultanés sur le MÊME bail : sérialisation, aucun interblocage
            let t0 = Instant::now();
            let mut hs = vec![];
            for i in 0..20 {
                let (c, url, lease) = (c.clone(), url.to_string(), d_leases[10].clone());
                hs.push(tokio::spawn(async move { send_payment(&c, &url, ORG_D, &uuid::Uuid::new_v4().to_string(), &req(&lease, "trial", 1000 + i), "").await }));
            }
            let mut seqs = vec![];
            for h in hs {
                let r = h.await?;
                if r.status == 201 {
                    seqs.push(serde_json::from_str::<serde_json::Value>(&r.body)?["sequence"].as_i64().unwrap());
                } else {
                    println!("  échec : {} {}", r.status, r.body);
                }
            }
            seqs.sort();
            println!("20 paiements simultanés sur un bail : {} validés en {:.1} s ; numéros {:?}..{:?}, consécutifs : {}", seqs.len(), t0.elapsed().as_secs_f64(), seqs.first(), seqs.last(), seqs.windows(2).all(|w| w[1] == w[0] + 1));
        }
        "e27" => {
            // E-27 et DP-61 : paiement daté du 31/12/2026, saisi le 02/01/2027 -> numéro de 2027
            http_post_clock(&c, url, Some(2027)).await?;
            let mut rq = req(&d_leases[20], "official", 80_000);
            rq.paid_on = "2026-12-31".into();
            let r27 = send_payment(&c, url, ORG_D, &uuid::Uuid::new_v4().to_string(), &rq, "").await;
            let r27b = send_payment(&c, url, ORG_D, &uuid::Uuid::new_v4().to_string(), &rq, "").await;
            http_post_clock(&c, url, Some(2026)).await?;
            let r26 = send_payment(&c, url, ORG_D, &uuid::Uuid::new_v4().to_string(), &req(&d_leases[21], "official", 80_000), "").await;
            http_post_clock(&c, url, None).await?;
            println!("E-27 : horloge 2027-01-02, paiement du 31/12/2026 -> {} {}\n       deuxième en 2027 -> {} {}\n       retour en 2026 -> {} {}", r27.status, r27.body, r27b.status, r27b.body, r26.status, r26.body);
        }
        "verify" => verify(t).await?,
        "stats" => println!("{}", reqwest::get(format!("{url}/spike/stats")).await?.text().await?),
        _ => return Err("scénario : e10 | main | attempts | idem | samelease | e27 | verify | stats".into()),
    }
    Ok(())
}

async fn http_post_clock(c: &reqwest::Client, url: &str, year: Option<i32>) -> R<()> {
    let body = match year {
        Some(y) => format!(r#"{{"year":{y}}}"#),
        None => r#"{"year":null}"#.into(),
    };
    c.post(format!("{url}/spike/clock")).body(body).send().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> R<()> {
    let a: Vec<String> = std::env::args().collect();
    let get = |i: usize| a.get(i).map(String::as_str).unwrap_or("");
    let t = Target::load(if get(2).is_empty() { "local" } else { get(2) })?;
    match get(1) {
        "setup" => setup(&t).await,
        "serve" => serve(&t, get(3).parse().unwrap_or(3000)).await,
        "load" => load(&t, get(3), if get(4).is_empty() { "http://127.0.0.1:3000" } else { get(4) }).await,
        _ => Err("commande : setup | serve | load".into()),
    }
}
