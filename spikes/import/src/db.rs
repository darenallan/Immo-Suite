//! Garanties de processus de l'import sur PostgreSQL (R-080, E-21) : rien avant confirmation, tout disparaît
//! après annulation, double confirmation sans effet, annulation refusée après un paiement.
//! Tables jouets, sans RLS (ce n'est pas l'objet de ce prototype) ; l'annulation est une exception encadrée simplifiée.

use std::path::Path;
use std::str::FromStr;

use sqlx::{Connection, Executor, PgConnection, Row, postgres::PgConnectOptions};

use crate::analyze::{self, Report};

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SCHEMA: &str = "
DROP TABLE IF EXISTS payments, leases, tenants, units, import_rows, import_batches CASCADE;
CREATE TABLE import_batches (id uuid PRIMARY KEY, org_id uuid NOT NULL, file_sha256 text NOT NULL,
  status text NOT NULL CHECK (status IN ('staged','confirmed','cancelled')), rows_total int NOT NULL, rows_creatable int NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(), confirmed_at timestamptz, cancelled_at timestamptz);
CREATE TABLE import_rows (batch_id uuid NOT NULL REFERENCES import_batches(id), line int NOT NULL, status text NOT NULL, payload jsonb NOT NULL, PRIMARY KEY (batch_id, line));
CREATE TABLE tenants (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, name text NOT NULL, phone text, import_batch_id uuid REFERENCES import_batches(id));
CREATE TABLE units (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, district text, ref text NOT NULL, import_batch_id uuid REFERENCES import_batches(id), UNIQUE (org_id, district, ref));
CREATE TABLE leases (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, tenant_id uuid NOT NULL REFERENCES tenants(id), unit_id uuid NOT NULL REFERENCES units(id),
  rent bigint NOT NULL CHECK (rent > 0), due_day int NOT NULL, start_date date NOT NULL, end_date date, import_batch_id uuid REFERENCES import_batches(id));
CREATE TABLE payments (id uuid PRIMARY KEY DEFAULT gen_random_uuid(), org_id uuid NOT NULL, lease_id uuid NOT NULL REFERENCES leases(id), amount bigint NOT NULL);
";

async fn counts(c: &mut PgConnection, batch: Option<&str>) -> R<String> {
    let mut out = vec![];
    for t in ["tenants", "units", "leases"] {
        let n: i64 = match batch {
            Some(b) => sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {t} WHERE import_batch_id = $1::uuid"))).bind(b).fetch_one(&mut *c).await?,
            None => sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {t}"))).fetch_one(&mut *c).await?,
        };
        out.push(format!("{t}={n}"));
    }
    if let Some(b) = batch {
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM import_rows WHERE batch_id = $1::uuid").bind(b).fetch_one(&mut *c).await?;
        out.push(format!("import_rows={n}"));
    }
    Ok(out.join(" "))
}

async fn stage(c: &mut PgConnection, org: &str, sha: &str, report: &Report) -> R<String> {
    let id = uuid::Uuid::new_v4().to_string();
    let mut tx = c.begin().await?;
    sqlx::query("INSERT INTO import_batches (id, org_id, file_sha256, status, rows_total, rows_creatable) VALUES ($1::uuid, $2::uuid, $3, 'staged', $4, $5)")
        .bind(&id).bind(org).bind(sha).bind(report.rows.len() as i32).bind(report.creatable() as i32).execute(&mut *tx).await?;
    let canon = report.canonical();
    for r in canon["rows"].as_array().cloned().unwrap_or_default() {
        sqlx::query("INSERT INTO import_rows (batch_id, line, status, payload) VALUES ($1::uuid, $2, $3, $4)")
            .bind(&id).bind(r["line"].as_i64().unwrap() as i32).bind(r["status"].as_str().unwrap()).bind(&r).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(id)
}

/// Confirmation : crée locataires, lots et baux des lignes sans erreur, dans une transaction. Sans effet si déjà confirmé.
async fn confirm(c: &mut PgConnection, org: &str, batch: &str) -> R<(bool, i64)> {
    let mut tx = c.begin().await?;
    // verrou de ligne : deux confirmations simultanées ne créent qu'une fois
    let status: String = sqlx::query_scalar("SELECT status FROM import_batches WHERE id = $1::uuid FOR UPDATE").bind(batch).fetch_one(&mut *tx).await?;
    if status != "staged" {
        tx.rollback().await?;
        return Ok((false, 0));
    }
    let rows = sqlx::query("SELECT payload FROM import_rows WHERE batch_id = $1::uuid AND status <> 'error' ORDER BY line").bind(batch).fetch_all(&mut *tx).await?;
    let mut created = 0;
    for r in rows {
        let p: serde_json::Value = r.get(0);
        let tenant: String = sqlx::query_scalar("INSERT INTO tenants (org_id, name, phone, import_batch_id) VALUES ($1::uuid, $2, $3, $4::uuid) RETURNING id::text")
            .bind(org).bind(p["name"].as_str()).bind(p["phone"].as_str()).bind(batch).fetch_one(&mut *tx).await?;
        // le lot est cherché avant d'être créé : une même référence n'est créée qu'une fois (proposée : clé quartier + référence)
        let district = p["district"].as_str().unwrap_or("");
        let uref = p["unit_ref"].as_str().unwrap_or("");
        let unit: String = sqlx::query_scalar(
            "INSERT INTO units (org_id, district, ref, import_batch_id) VALUES ($1::uuid, $2, $3, $4::uuid)
             ON CONFLICT (org_id, district, ref) DO UPDATE SET ref = units.ref RETURNING id::text",
        )
        .bind(org).bind(district).bind(uref).bind(batch).fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO leases (org_id, tenant_id, unit_id, rent, due_day, start_date, end_date, import_batch_id) VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6::date, $7::date, $8::uuid)")
            .bind(org).bind(&tenant).bind(&unit).bind(p["rent"].as_i64()).bind(p["due_day"].as_i64().unwrap_or(1) as i32)
            .bind(p["start"].as_str()).bind(p["end"].as_str()).bind(batch).execute(&mut *tx).await?;
        created += 1;
    }
    sqlx::query("UPDATE import_batches SET status = 'confirmed', confirmed_at = now() WHERE id = $1::uuid").bind(batch).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok((true, created))
}

/// Annulation (R-080, E-21) : refusée si un paiement existe sur un bail du lot ; sinon tout disparaît, l'en-tête du lot
/// reste avec le statut « cancelled » (trace d'audit, proposée).
async fn cancel(c: &mut PgConnection, batch: &str) -> R<Result<(), String>> {
    let mut tx = c.begin().await?;
    let status: String = sqlx::query_scalar("SELECT status FROM import_batches WHERE id = $1::uuid FOR UPDATE").bind(batch).fetch_one(&mut *tx).await?;
    if status == "cancelled" {
        tx.rollback().await?;
        return Ok(Err("déjà annulé".into()));
    }
    let used: i64 = sqlx::query_scalar("SELECT count(*) FROM payments p JOIN leases l ON l.id = p.lease_id WHERE l.import_batch_id = $1::uuid").bind(batch).fetch_one(&mut *tx).await?;
    if used > 0 {
        tx.rollback().await?;
        return Ok(Err(format!("{used} paiement(s) sur des baux du lot : annulation refusée (R-080)")));
    }
    for t in ["leases", "tenants", "units"] {
        sqlx::query(sqlx::AssertSqlSafe(format!("DELETE FROM {t} WHERE import_batch_id = $1::uuid"))).bind(batch).execute(&mut *tx).await?;
    }
    sqlx::query("DELETE FROM import_rows WHERE batch_id = $1::uuid").bind(batch).execute(&mut *tx).await?;
    sqlx::query("UPDATE import_batches SET status = 'cancelled', cancelled_at = now() WHERE id = $1::uuid").bind(batch).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

pub async fn scenario(file: &Path, target: &str) -> R<()> {
    use sha2::{Digest, Sha256};
    let url = match target {
        "local" => "postgres://postgres@127.0.0.1:5433/spike_d".to_string(),
        _ => {
            let p = std::path::PathBuf::from(std::env::var("LOCALAPPDATA")?).join("immo-spikes-neon").join(".env.local");
            std::fs::read_to_string(p)?.lines().find_map(|l| l.strip_prefix("NEON_D_ADMIN_URL=").map(String::from)).ok_or("NEON_D_ADMIN_URL absent")?
        }
    };
    if target == "local" {
        let mut c = PgConnection::connect_with(&PgConnectOptions::from_str(&url)?.database("postgres")).await?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = 'spike_d')").fetch_one(&mut c).await?;
        if !exists {
            c.execute("CREATE DATABASE spike_d").await?;
        }
    }
    let mut c = PgConnection::connect_with(&PgConnectOptions::from_str(&url)?).await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(SCHEMA.to_string())).execute(&mut c).await?;
    let org = "d0000000-0000-4000-8000-0000000000d1";
    let sha: String = Sha256::digest(std::fs::read(file)?).iter().map(|b| format!("{b:02x}")).collect();
    let report = analyze::analyze(file);
    if report.refusal.is_some() {
        return Err(format!("fichier refusé : {}", report.refusal.unwrap()).into());
    }
    println!("fichier : {} ({} lignes, {} créables, {} erreurs)", file.display(), report.rows.len(), report.creatable(), report.rows.len() - report.creatable());

    println!("\n== 1. analyse puis mise en attente : aucune ligne créée avant confirmation");
    let b1 = stage(&mut c, org, &sha, &report).await?;
    println!("avant confirmation : {}", counts(&mut c, None).await?);

    println!("\n== 2. confirmation, puis seconde confirmation (sans effet attendu)");
    let (done, created) = confirm(&mut c, org, &b1).await?;
    println!("confirmation 1 : effectuée={done}, baux créés={created} ; {}", counts(&mut c, Some(&b1)).await?);
    let (done2, created2) = confirm(&mut c, org, &b1).await?;
    println!("confirmation 2 : effectuée={done2}, baux créés={created2} ; {}", counts(&mut c, Some(&b1)).await?);

    println!("\n== 3. annulation (E-21) : rien ne doit rester du lot");
    match cancel(&mut c, &b1).await? {
        Ok(()) => println!("annulation acceptée ; {}", counts(&mut c, Some(&b1)).await?),
        Err(e) => println!("annulation refusée : {e}"),
    }
    println!("total après annulation : {}", counts(&mut c, None).await?);
    let st: String = sqlx::query_scalar("SELECT status FROM import_batches WHERE id = $1::uuid").bind(&b1).fetch_one(&mut c).await?;
    println!("en-tête du lot conservé : statut = {st}");
    println!("seconde annulation : {:?}", cancel(&mut c, &b1).await?);

    println!("\n== 4. nouveau lot, confirmé, puis un paiement sur un de ses baux : l'annulation doit être refusée");
    let b2 = stage(&mut c, org, &sha, &report).await?;
    confirm(&mut c, org, &b2).await?;
    let lease: String = sqlx::query_scalar("SELECT id::text FROM leases WHERE import_batch_id = $1::uuid ORDER BY id LIMIT 1").bind(&b2).fetch_one(&mut c).await?;
    sqlx::query("INSERT INTO payments (org_id, lease_id, amount) VALUES ($1::uuid, $2::uuid, 75000)").bind(org).bind(&lease).execute(&mut c).await?;
    println!("avant tentative : {}", counts(&mut c, Some(&b2)).await?);
    match cancel(&mut c, &b2).await? {
        Ok(()) => println!("ANOMALIE : annulation acceptée malgré le paiement"),
        Err(e) => println!("annulation refusée : {e}"),
    }
    println!("après tentative : {}", counts(&mut c, Some(&b2)).await?);
    Ok(())
}
