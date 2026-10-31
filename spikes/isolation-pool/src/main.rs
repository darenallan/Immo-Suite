//! Prototype (b) : isolation entre organisations avec pool de connexions.
//! Protocole : `spikes/isolation-pool/NOTE.md`. Hors build du produit, aucune donnée réelle.
//!
//! Usage : spike-isolation-pool <setup|isolation|overhead|backup|restore-check> <local|neon>

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use sqlx::{
    Connection, Executor, PgConnection, PgPool, Postgres, Row, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const ORG_A: &str = "a0000000-0000-4000-8000-000000000001";
const ORG_B: &str = "b0000000-0000-4000-8000-000000000002";
const ORG_C: &str = "c0000000-0000-4000-8000-000000000003";
const USER_1: &str = "11111111-1111-4111-8111-111111111111";
const USER_2: &str = "22222222-2222-4222-8222-222222222222";
const ROLES: [&str; 4] = ["immo_owner", "immo_app", "immo_platform", "immo_backup"];

// ───────────────────────── configuration ─────────────────────────

struct Target {
    name: String,
    env_path: PathBuf,
    env: HashMap<String, String>,
    admin: PgConnectOptions,
    admin_pw: String,
}

impl Target {
    fn load(name: &str) -> R<Self> {
        let (dir, admin_key, default_admin) = match name {
            "local" => (
                "immo-spikes-pg",
                "LOCAL_ADMIN_URL",
                Some("postgres://postgres@127.0.0.1:5433/spike_b"),
            ),
            "neon" => ("immo-spikes-neon", "NEON_B_ADMIN_URL", None),
            _ => return Err("cible inconnue : local ou neon".into()),
        };
        let env_path = PathBuf::from(std::env::var("LOCALAPPDATA")?)
            .join(dir)
            .join(".env.local");
        let mut env = HashMap::new();
        if let Ok(text) = std::fs::read_to_string(&env_path) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    env.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        let url = env
            .get(admin_key)
            .cloned()
            .or(default_admin.map(String::from))
            .ok_or_else(|| format!("{admin_key} absent de {}", env_path.display()))?;
        // mot de passe de l'administrateur, lu dans l'URL (sqlx ne l'expose pas) pour pg_dump et pg_restore
        let admin_pw = url
            .split_once("://")
            .and_then(|(_, rest)| rest.split_once('@'))
            .and_then(|(cred, _)| cred.split_once(':'))
            .map(|(_, pw)| pw.to_string())
            .unwrap_or_default();
        Ok(Self {
            name: name.into(),
            env_path,
            env,
            admin: PgConnectOptions::from_str(&url)?,
            admin_pw,
        })
    }

    fn save(&mut self, key: &str, value: &str) -> R<()> {
        self.env.insert(key.into(), value.into());
        let mut lines: Vec<_> = self.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        lines.sort();
        std::fs::write(&self.env_path, lines.join("\n") + "\n")?;
        Ok(())
    }

    fn password(&mut self, role: &str) -> R<String> {
        let key = format!("PW_{}", role.to_uppercase());
        if let Some(v) = self.env.get(&key) {
            return Ok(v.clone());
        }
        let pw = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
        self.save(&key, &pw)?;
        Ok(pw)
    }

    fn opts(&mut self, role: &str) -> R<PgConnectOptions> {
        let pw = self.password(role)?;
        Ok(self.admin.clone().username(role).password(&pw))
    }

    fn is_neon(&self) -> bool {
        self.name == "neon"
    }
}

// ───────────────────────── OrgTx ─────────────────────────

/// Transaction qui porte le contexte d'organisation (ARCHITECTURE.md §5.3).
struct OrgTx(Transaction<'static, Postgres>);

impl OrgTx {
    async fn begin(pool: &PgPool, org: &str, user: &str) -> Result<Self, sqlx::Error> {
        let mut tx = pool.begin().await?;
        sqlx::query("SELECT set_config('app.org_id', $1, true), set_config('app.user_id', $2, true)")
            .bind(org)
            .bind(user)
            .execute(&mut *tx)
            .await?;
        Ok(Self(tx))
    }
    fn conn(&mut self) -> &mut PgConnection {
        &mut self.0
    }
    async fn commit(self) -> Result<(), sqlx::Error> {
        self.0.commit().await
    }
}

fn code(e: &sqlx::Error) -> String {
    e.as_database_error()
        .and_then(|d| d.code().map(|c| c.to_string()))
        .unwrap_or_else(|| e.to_string())
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() as f64 - 1.0) * p).round() as usize]
}

async fn pool_of(opts: PgConnectOptions, max: u32) -> R<PgPool> {
    Ok(PgPoolOptions::new().max_connections(max).connect_with(opts).await?)
}

// ───────────────────────── setup ─────────────────────────

async fn setup(t: &mut Target) -> R<()> {
    if !t.is_neon() {
        let mut c = PgConnection::connect_with(&t.admin.clone().database("postgres")).await?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_database WHERE datname = 'spike_b')")
            .fetch_one(&mut c)
            .await?;
        if !exists {
            c.execute("CREATE DATABASE spike_b").await?;
        }
    }
    let admin = t.admin.clone();
    let mut a = PgConnection::connect_with(&admin).await?;
    let who: String = sqlx::query_scalar("SELECT current_user || ' / ' || version()").fetch_one(&mut a).await?;
    println!("administrateur : {who}");

    create_roles(t, &mut a).await?;

    // Schéma exécuté par immo_owner, comme les migrations du produit
    let owner_opts = t.opts("immo_owner")?;
    let mut o = PgConnection::connect_with(&owner_opts).await?;
    let schema = std::fs::read_to_string("sql/schema.sql")?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(schema)).execute(&mut o).await?;
    println!("schéma jouet créé par immo_owner");

    // Données de départ : 3 organisations, écrites par immo_app sous contexte
    let app = pool_of(t.opts("immo_app")?, 2).await?;
    for (org, name, n) in [(ORG_A, "Agence A", 50), (ORG_B, "Agence B", 50), (ORG_C, "Agence C", 50)] {
        let mut tx = OrgTx::begin(&app, org, USER_1).await?;
        sqlx::query("INSERT INTO organizations (id, name) VALUES ($1::uuid, $2)")
            .bind(org)
            .bind(name)
            .execute(tx.conn())
            .await?;
        sqlx::query(
            "INSERT INTO units (org_id, label, rent_fcfa)
             SELECT $1::uuid, $2 || '-lot-' || g, 50000 + g * 1000 FROM generate_series(1, $3) g",
        )
        .bind(org)
        .bind(name)
        .bind(n)
        .execute(tx.conn())
        .await?;
        sqlx::query(
            "INSERT INTO plain_units (org_id, label, rent_fcfa)
             SELECT $1::uuid, $2 || '-lot-' || g, 50000 + g * 1000 FROM generate_series(1, $3) g",
        )
        .bind(org)
        .bind(name)
        .bind(n)
        .execute(tx.conn())
        .await?;
        let (m_user, m_role) = if org == ORG_C { (USER_2, "owner") } else { (USER_1, "manager") };
        sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1::uuid, $2::uuid, $3)")
            .bind(org)
            .bind(m_user)
            .bind(m_role)
            .execute(tx.conn())
            .await?;
        tx.commit().await?;
    }
    println!("3 organisations semées (50 lots chacune)");
    role_audit(t).await
}

/// Crée les rôles par SQL (jamais par la console). Consigne le message exact de BYPASSRLS.
async fn create_roles(t: &mut Target, a: &mut PgConnection) -> R<()> {
    a.execute("GRANT CREATE, USAGE ON SCHEMA public TO PUBLIC").await.ok();
    // les politiques du dump peuvent viser le rôle de l'essai V2 : il doit exister à destination
    let mut roles: Vec<&str> = ROLES.to_vec();
    if t.env.contains_key("PW_IMMO_BACKUP_V2") {
        roles.push("immo_backup_v2");
    }
    for role in roles {
        let pw = t.password(role)?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT FROM pg_roles WHERE rolname = $1)")
            .bind(role)
            .fetch_one(&mut *a)
            .await?;
        let mut recreated = false;
        if exists && role == "immo_backup" {
            // on recrée pour rejouer l'essai BYPASSRLS ; si des privilèges en dépendent (cas de Neon), on le garde
            a.execute("DROP OWNED BY immo_backup").await.ok();
            recreated = a.execute("DROP ROLE immo_backup").await.is_ok();
        }
        let exists = exists && !recreated;
        if exists {
            a.execute(sqlx::AssertSqlSafe(format!("ALTER ROLE {role} LOGIN PASSWORD '{pw}'")))
                .await?;
            continue;
        }
        if role == "immo_backup" {
            match a.execute(sqlx::AssertSqlSafe(format!("CREATE ROLE {role} LOGIN BYPASSRLS PASSWORD '{pw}'"))).await {
                Ok(_) => println!("CREATE ROLE immo_backup BYPASSRLS : ACCEPTÉ"),
                Err(e) => {
                    println!("CREATE ROLE immo_backup BYPASSRLS : REFUSÉ -> {e}");
                    a.execute(sqlx::AssertSqlSafe(format!("CREATE ROLE {role} LOGIN PASSWORD '{pw}'"))).await?;
                }
            }
        } else {
            a.execute(sqlx::AssertSqlSafe(format!("CREATE ROLE {role} LOGIN PASSWORD '{pw}'"))).await?;
        }
    }
    // immo_owner doit pouvoir créer des objets dans public ; les autres seulement s'en servir
    a.execute("GRANT ALL ON SCHEMA public TO immo_owner").await?;
    a.execute("REVOKE CREATE ON SCHEMA public FROM PUBLIC").await.ok();
    a.execute("GRANT USAGE ON SCHEMA public TO immo_app, immo_platform, immo_backup").await?;
    Ok(())
}

async fn role_audit(t: &mut Target) -> R<()> {
    let mut a = PgConnection::connect_with(&t.admin).await?;
    println!("\n-- SELECT rolname, rolbypassrls, rolsuper FROM pg_roles (rôles du prototype et administrateur)");
    let rows = sqlx::query(
        "SELECT rolname, rolbypassrls, rolsuper, rolcreaterole FROM pg_roles
         WHERE rolname LIKE 'immo\\_%' OR rolname = current_user ORDER BY 1",
    )
    .fetch_all(&mut a)
    .await?;
    for r in rows {
        println!(
            "{:<16} bypassrls={:<5} super={:<5} createrole={}",
            r.get::<String, _>(0),
            r.get::<bool, _>(1),
            r.get::<bool, _>(2),
            r.get::<bool, _>(3)
        );
    }
    let owned: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relowner = 'immo_app'::regrole AND n.nspname = 'public'",
    )
    .fetch_one(&mut a)
    .await?;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname || ' enable=' || c.relrowsecurity || ' force=' || c.relforcerowsecurity
         FROM pg_class c WHERE c.relnamespace = 'public'::regnamespace AND c.relkind = 'r' ORDER BY 1",
    )
    .fetch_all(&mut a)
    .await?;
    println!("objets de public possédés par immo_app : {owned}");
    for l in tables {
        println!("  {l}");
    }
    Ok(())
}

// ───────────────────────── isolation (T1 à T5) ─────────────────────────

async fn count_other(tx: &mut OrgTx, me: &str) -> Result<(u64, u64), sqlx::Error> {
    let rows = sqlx::query("SELECT org_id::text FROM units").fetch_all(tx.conn()).await?;
    let foreign = rows.iter().filter(|r| r.get::<String, _>(0) != me).count() as u64;
    Ok((rows.len() as u64, foreign))
}

async fn isolation(t: &mut Target) -> R<()> {
    role_audit(t).await?;
    let app = pool_of(t.opts("immo_app")?, 2).await?;

    println!("\n== T1 sans contexte");
    let mut tx = app.begin().await?;
    for table in ["organizations", "units", "leases", "memberships"] {
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
            .fetch_one(&mut *tx)
            .await?;
        println!("SELECT count(*) FROM {table:<14} -> {n}");
    }
    let ins = sqlx::query("INSERT INTO units (org_id, label, rent_fcfa) VALUES ($1::uuid, 'x', 1)")
        .bind(ORG_A)
        .execute(&mut *tx)
        .await;
    println!("INSERT sans contexte -> {}", ins.map(|_| "ACCEPTÉ (anomalie)".into()).unwrap_or_else(|e| code(&e)));
    drop(tx);

    println!("\n== T2 deux organisations");
    let mut ta = OrgTx::begin(&app, ORG_A, USER_1).await?;
    let b_seen: i64 = sqlx::query_scalar("SELECT count(*) FROM units WHERE org_id = $1::uuid")
        .bind(ORG_B)
        .fetch_one(ta.conn())
        .await?;
    println!("A lit les lots de B -> {b_seen} ligne(s)");
    let upd = sqlx::query("UPDATE units SET rent_fcfa = 0 WHERE org_id = $1::uuid")
        .bind(ORG_B)
        .execute(ta.conn())
        .await?;
    println!("A met à jour les lots de B -> {} ligne(s) touchée(s)", upd.rows_affected());
    let ins = sqlx::query("INSERT INTO units (org_id, label, rent_fcfa) VALUES ($1::uuid, 'pirate', 1)")
        .bind(ORG_B)
        .execute(ta.conn())
        .await;
    println!("A insère un lot avec l'org_id de B -> {}", ins.map(|_| "ACCEPTÉ (anomalie)".into()).unwrap_or_else(|e| code(&e)));
    ta.commit().await.ok(); // la transaction est déjà en échec : l'issue n'importe pas
    let mut tb = OrgTx::begin(&app, ORG_B, USER_1).await?;
    let b_unit: String = sqlx::query_scalar("SELECT id::text FROM units LIMIT 1").fetch_one(tb.conn()).await?;
    let b_zero: i64 = sqlx::query_scalar("SELECT count(*) FROM units WHERE rent_fcfa = 0").fetch_one(tb.conn()).await?;
    println!("B contrôle ses lots : {b_zero} loyer(s) à 0 (attendu 0)");
    tb.commit().await?;
    let mut ta = OrgTx::begin(&app, ORG_A, USER_1).await?;
    let lease = sqlx::query("INSERT INTO leases (org_id, unit_id, tenant) VALUES ($1::uuid, $2::uuid, 'X')")
        .bind(ORG_A)
        .bind(&b_unit)
        .execute(ta.conn())
        .await;
    println!("bail de A visant un lot de B (clé composite) -> {}", lease.map(|_| "ACCEPTÉ (anomalie)".into()).unwrap_or_else(|e| code(&e)));
    drop(ta);

    println!("\n== T3 concurrence : 50 tâches, pool de 2 connexions, 10 000 itérations");
    let (reads, leaks, empty) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let start = Instant::now();
    let mut hs = vec![];
    for task in 0..50u32 {
        let (pool, reads, leaks, empty) = (app.clone(), reads.clone(), leaks.clone(), empty.clone());
        hs.push(tokio::spawn(async move {
            for i in 0..200u32 {
                let me = if (task + i) % 2 == 0 { ORG_A } else { ORG_B };
                let mut tx = OrgTx::begin(&pool, me, USER_1).await.unwrap();
                let (n, foreign) = count_other(&mut tx, me).await.unwrap();
                tx.commit().await.unwrap();
                reads.fetch_add(n, Ordering::Relaxed);
                leaks.fetch_add(foreign, Ordering::Relaxed);
                if n == 0 {
                    empty.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for h in hs {
        h.await?;
    }
    println!(
        "10 000 itérations en {:.1} s : {} lignes lues, lectures croisées = {}, itérations vides = {}",
        start.elapsed().as_secs_f64(),
        reads.load(Ordering::Relaxed),
        leaks.load(Ordering::Relaxed),
        empty.load(Ordering::Relaxed)
    );

    println!("\n== T4 test négatif : SET app.org_id de SESSION, 10 % d'oublis (DOIT FUIR)");
    let (reads, leaks, leak_iters) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    // pool dédié et jeté : la variante de session pollue les connexions, elle ne doit pas fausser T5
    let leaky = pool_of(t.opts("immo_app")?, 2).await?;
    let mut hs = vec![];
    for task in 0..50u32 {
        let (pool, reads, leaks, leak_iters) = (leaky.clone(), reads.clone(), leaks.clone(), leak_iters.clone());
        hs.push(tokio::spawn(async move {
            for i in 0..200u32 {
                let me = if (task + i) % 2 == 0 { ORG_A } else { ORG_B };
                let forget = (task * 200 + i) % 10 == 9;
                let mut conn = pool.acquire().await.unwrap();
                if !forget {
                    // la fuite : set_config(..., false) vaut SET de session
                    sqlx::query("SELECT set_config('app.org_id', $1, false)").bind(me).execute(&mut *conn).await.unwrap();
                }
                let rows = sqlx::query("SELECT org_id::text FROM units").fetch_all(&mut *conn).await.unwrap();
                let foreign = rows.iter().filter(|r| r.get::<String, _>(0) != me).count() as u64;
                reads.fetch_add(rows.len() as u64, Ordering::Relaxed);
                leaks.fetch_add(foreign, Ordering::Relaxed);
                if foreign > 0 {
                    leak_iters.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for h in hs {
        h.await?;
    }
    println!(
        "lignes lues = {}, lectures croisées = {} (dans {} itérations sur 10 000) -> {}",
        reads.load(Ordering::Relaxed),
        leaks.load(Ordering::Relaxed),
        leak_iters.load(Ordering::Relaxed),
        if leaks.load(Ordering::Relaxed) > 0 { "FUITE OBSERVÉE, comme prévu" } else { "aucune fuite : le test négatif n'a rien prouvé" }
    );

    leaky.close().await;
    println!("\n== T4b SET LOCAL hors transaction (pool d'une connexion)");
    let one = pool_of(t.opts("immo_app")?, 1).await?;
    {
        let mut c = one.acquire().await?;
        sqlx::query("SELECT set_config('app.org_id', $1, false)").bind(ORG_B).execute(&mut *c).await?;
    }
    let mut c = one.acquire().await?;
    let r = c.execute(sqlx::AssertSqlSafe(format!("SET LOCAL app.org_id = '{ORG_A}'"))).await;
    println!("SET LOCAL hors transaction -> {}", r.map(|_| "accepté sans effet (avertissement)".into()).unwrap_or_else(|e| code(&e)));
    let seen: Vec<String> = sqlx::query_scalar("SELECT DISTINCT org_id::text FROM units").fetch_all(&mut *c).await?;
    println!("lots visibles ensuite : orgs {seen:?} (A voulue = {ORG_A}) -> {}", if seen.iter().any(|o| o == ORG_B) { "FUITE : la valeur de session précédente (B) survit" } else { "pas de fuite" });
    drop(c);

    println!("\n== T5 adhésions avant contexte");
    let mut tx = app.begin().await?;
    let n0: i64 = sqlx::query_scalar("SELECT count(*) FROM memberships").fetch_one(&mut *tx).await?;
    println!("sans aucun contexte : memberships visibles = {n0}");
    let n1: i64 = sqlx::query_scalar("SELECT count(*) FROM find_memberships($1::uuid)").bind(USER_1).fetch_one(&mut *tx).await?;
    let n2: i64 = sqlx::query_scalar("SELECT count(*) FROM find_memberships($1::uuid)").bind(USER_2).fetch_one(&mut *tx).await?;
    println!("find_memberships(USER_1) = {n1} (attendu 2), find_memberships(USER_2) = {n2} (attendu 1)");
    println!("  limite : la fonction accepte n'importe quel identifiant d'utilisateur ; l'appelant doit être la couche identity");
    sqlx::query("SELECT set_config('app.user_id', $1, true)").bind(USER_1).execute(&mut *tx).await?;
    let own: Vec<String> = sqlx::query_scalar("SELECT org_id::text FROM memberships ORDER BY 1").fetch_all(&mut *tx).await?;
    println!("politique own_memberships (app.user_id seul) : {} adhésion(s) -> {own:?}", own.len());
    drop(tx);
    Ok(())
}

// ───────────────────────── surcoût ─────────────────────────

async fn bench<F, Fut>(name: &str, n: usize, mut f: F) -> R<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<(), sqlx::Error>>,
{
    for _ in 0..100 {
        f().await?; // échauffement (Neon sort de veille)
    }
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let s = Instant::now();
        f().await?;
        v.push(s.elapsed().as_secs_f64() * 1000.0);
    }
    println!("{name:<58} p50 = {:>7.2} ms   p95 = {:>7.2} ms", pct(&mut v.clone(), 0.5), pct(&mut v, 0.95));
    Ok(())
}

async fn overhead(t: &mut Target) -> R<()> {
    let n = 1000;
    println!("cible : {} ({} requêtes mesurées par variante, 100 d'échauffement)", t.name, n);
    let plain = pool_of(t.opts("immo_app")?, 2).await?;
    let hooks = PgPoolOptions::new()
        .max_connections(2)
        .after_release(|conn, _| {
            Box::pin(async move {
                conn.execute("RESET app.org_id; RESET app.user_id").await?;
                Ok(true)
            })
        })
        .before_acquire(|conn, _| {
            Box::pin(async move {
                let v: Option<String> = sqlx::query_scalar("SELECT NULLIF(current_setting('app.org_id', true), '')")
                    .fetch_one(&mut *conn)
                    .await?;
                Ok(v.is_none())
            })
        })
        .connect_with(t.opts("immo_app")?)
        .await?;

    let one = pool_of(t.opts("immo_app")?, 1).await?;
    bench("aller-retour de base : SELECT 1 (1 connexion)", n, || async { sqlx::query("SELECT 1").execute(&one).await.map(|_| ()) }).await?;
    bench("(i) requête nue, table sans RLS", n, || async {
        sqlx::query("SELECT count(*), coalesce(sum(rent_fcfa), 0)::bigint FROM plain_units WHERE org_id = $1::uuid")
            .bind(ORG_A)
            .fetch_one(&plain)
            .await
            .map(|_| ())
    })
    .await?;
    bench("(ii) OrgTx : BEGIN, set_config, requête, COMMIT", n, || async {
        let mut tx = OrgTx::begin(&plain, ORG_A, USER_1).await?;
        sqlx::query("SELECT count(*), coalesce(sum(rent_fcfa), 0)::bigint FROM units").fetch_one(tx.conn()).await?;
        tx.commit().await
    })
    .await?;
    bench("(iii) OrgTx + after_release RESET + before_acquire", n, || async {
        let mut tx = OrgTx::begin(&hooks, ORG_A, USER_1).await?;
        sqlx::query("SELECT count(*), coalesce(sum(rent_fcfa), 0)::bigint FROM units").fetch_one(tx.conn()).await?;
        tx.commit().await
    })
    .await?;
    Ok(())
}

// ───────────────────────── sauvegarde logique ─────────────────────────

async fn run_tool(t: &mut Target, tool: &str, role: &str, args: &[String]) -> R<(i32, String, Vec<u8>)> {
    let pw = if role == "admin" { t.admin_pw.clone() } else { t.password(role)? };
    let user = if role == "admin" { t.admin.get_username().to_string() } else { role.to_string() };
    let bin = format!("C:\\Program Files\\PostgreSQL\\18\\bin\\{tool}.exe");
    let out = tokio::process::Command::new(bin)
        .env("PGPASSWORD", pw)
        .env("PGSSLMODE", if t.is_neon() { "require" } else { "prefer" })
        .args(["-h", t.admin.get_host(), "-p", &t.admin.get_port().to_string(), "-U", &user])
        .args(args)
        .output()
        .await?;
    let err = String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("").to_string();
    Ok((out.status.code().unwrap_or(-1), err, out.stdout))
}

/// Compte les lignes par table et par organisation dans un fichier `pg_dump -Fc`.
async fn dump_counts(t: &mut Target, file: &str) -> R<BTreeMap<String, u64>> {
    let (code, _, out) = run_tool(t, "pg_restore", "immo_backup", &["--data-only".into(), "-f".into(), "-".into(), file.into()]).await?;
    let mut counts = BTreeMap::new();
    if code != 0 {
        return Ok(counts);
    }
    let text = String::from_utf8_lossy(&out);
    let mut table: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("COPY public.") {
            table = rest.split(' ').next().map(String::from);
        } else if line == "\\." {
            table = None;
        } else if let Some(tb) = &table {
            let first = line.split('\t').next().unwrap_or("");
            // organizations a l'id en première colonne, les autres tables org_id ; plain_units : org_id en 2e colonne
            let org = if tb == "plain_units" { line.split('\t').nth(1).unwrap_or("") } else { first };
            let org = if tb == "units" || tb == "leases" { line.split('\t').nth(1).unwrap_or("") } else { org };
            *counts.entry(format!("{tb}|{}", org.chars().next().unwrap_or('?'))).or_default() += 1;
        }
    }
    Ok(counts)
}

/// Comptes par table et par organisation (initiale de l'org_id), lus par l'administrateur (BYPASSRLS).
async fn admin_counts(admin: &PgConnectOptions) -> R<BTreeMap<String, i64>> {
    let mut a = PgConnection::connect_with(admin).await?;
    let mut out = BTreeMap::new();
    for tb in ["organizations", "units", "leases", "memberships", "plain_units"] {
        let col = if tb == "organizations" { "id" } else { "org_id" };
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT {col}::text, count(*) FROM {tb} GROUP BY 1 ORDER BY 1")))
            .fetch_all(&mut a)
            .await?;
        for r in rows {
            out.insert(format!("{tb}|{}", &r.get::<String, _>(0)[..1]), r.get(1));
        }
    }
    Ok(out)
}

/// Restaure le dump de immo_backup (V1) dans une autre base, puis compare les comptes par organisation.
async fn restore(t: &mut Target) -> R<()> {
    let (url_key, default_url) = if t.is_neon() {
        ("NEON_RESTORE_ADMIN_URL", None)
    } else {
        ("LOCAL_RESTORE_ADMIN_URL", Some("postgres://postgres@127.0.0.1:5433/spike_b_restore"))
    };
    let url = t.env.get(url_key).cloned().or(default_url.map(String::from)).ok_or("URL de restauration absente")?;
    if !t.is_neon() {
        let mut c = PgConnection::connect_with(&t.admin.clone().database("postgres")).await?;
        c.execute("DROP DATABASE IF EXISTS spike_b_restore").await?;
        c.execute("CREATE DATABASE spike_b_restore").await?;
    }
    let admin_pw = url.split_once("://").and_then(|(_, r)| r.split_once('@')).and_then(|(c, _)| c.split_once(':')).map(|(_, p)| p.to_string()).unwrap_or_default();
    let mut dest = Target { name: t.name.clone(), env_path: t.env_path.clone(), env: t.env.clone(), admin: PgConnectOptions::from_str(&url)?, admin_pw };
    let start = Instant::now();
    if t.is_neon() {
        // projet séparé : mêmes rôles créés en SQL avant la restauration (les rôles locaux existent déjà dans le cluster)
        let mut a = PgConnection::connect_with(&dest.admin).await?;
        create_roles(&mut dest, &mut a).await?;
    }
    let file = format!("out/{}-immo_backup.dump", t.name);
    let db = dest.admin.get_database().unwrap_or("postgres").to_string();
    let (code, err, _) = run_tool(&mut dest, "pg_restore", "admin", &["--no-owner".into(), "--no-privileges".into(), "--clean".into(), "--if-exists".into(), "--exit-on-error".into(), "-d".into(), db, file.clone()]).await?;
    println!("pg_restore {file} -> code de sortie = {code} {err} ({:.1} s, rôles compris)", start.elapsed().as_secs_f64());
    let src = admin_counts(&t.admin).await?;
    let dst = admin_counts(&dest.admin).await?;
    println!("comptes source      : {src:?}\ncomptes restaurés   : {dst:?}\nidentiques : {}", src == dst);
    let mut a = PgConnection::connect_with(&dest.admin).await?;
    let forced: Vec<String> = sqlx::query_scalar("SELECT relname || ' force=' || relforcerowsecurity FROM pg_class WHERE relnamespace = 'public'::regnamespace AND relkind = 'r' ORDER BY 1").fetch_all(&mut a).await?;
    println!("RLS après restauration : {forced:?}");
    Ok(())
}

async fn backup(t: &mut Target) -> R<()> {
    // jeu d'environ 10 000 lignes sur 3 organisations (units + leases)
    let app = pool_of(t.opts("immo_app")?, 2).await?;
    for (org, total) in [(ORG_A, 5000i64), (ORG_B, 3000), (ORG_C, 2000)] {
        let mut tx = OrgTx::begin(&app, org, USER_1).await?;
        let have: i64 = sqlx::query_scalar("SELECT count(*) FROM units").fetch_one(tx.conn()).await?;
        if have < total {
            sqlx::query("INSERT INTO units (org_id, label, rent_fcfa) SELECT $1::uuid, 'bulk-' || g, 40000 + g FROM generate_series(1, $2) g")
                .bind(org)
                .bind(total - have)
                .execute(tx.conn())
                .await?;
            sqlx::query(
                "INSERT INTO leases (org_id, unit_id, tenant)
                 SELECT org_id, id, 'locataire-' || label FROM units WHERE label LIKE 'bulk-%' AND random() < 0.3",
            )
            .execute(tx.conn())
            .await?;
        }
        tx.commit().await?;
    }
    std::fs::create_dir_all("out")?;
    let ref_counts = admin_counts(&t.admin).await?;
    println!("comptes de référence (administrateur, table|initiale d'organisation) : {ref_counts:?}");

    // V2 : rôle sans BYPASSRLS, avec une politique de lecture totale
    {
        let mut a = PgConnection::connect_with(&t.admin).await?;
        let pw = t.password("immo_backup_v2")?;
        a.execute(sqlx::AssertSqlSafe(format!(
            "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname='immo_backup_v2') THEN CREATE ROLE immo_backup_v2 LOGIN; END IF; END $$;
             ALTER ROLE immo_backup_v2 PASSWORD '{pw}' NOBYPASSRLS;
             GRANT USAGE ON SCHEMA public TO immo_backup_v2;"
        )))
        .await?;
        let mut o = PgConnection::connect_with(&t.opts("immo_owner")?).await?;
        // le droit de lecture vient du propriétaire des tables (sur Neon, l'administrateur n'est pas propriétaire)
        o.execute("GRANT SELECT ON ALL TABLES IN SCHEMA public TO immo_backup_v2").await?;
        for tb in ["organizations", "units", "leases", "memberships"] {
            o.execute(sqlx::AssertSqlSafe(format!("DROP POLICY IF EXISTS backup_read ON {tb}; CREATE POLICY backup_read ON {tb} FOR SELECT TO immo_backup_v2 USING (true)"))).await?;
        }
    }

    let db = t.admin.get_database().unwrap_or("postgres").to_string();
    let mut results = vec![];
    for (label, role) in [
        ("V1 immo_backup (BYPASSRLS si accepté)", "immo_backup"),
        ("V2 immo_backup_v2 (sans BYPASSRLS + politique USING (true))", "immo_backup_v2"),
        ("V3a immo_owner (propriétaire, RLS forcée)", "immo_owner"),
        ("V3b immo_app", "immo_app"),
    ] {
        let file = format!("out/{}-{}.dump", t.name, role);
        let _ = std::fs::remove_file(&file);
        let (code, err, _) = run_tool(t, "pg_dump", role, &["-Fc".into(), "-d".into(), db.clone(), "-f".into(), file.clone()]).await?;
        let counts = if code == 0 { dump_counts(t, &file).await? } else { BTreeMap::new() };
        println!("\n{label}\n  commande : pg_dump -Fc -h <hôte> -U {role} -d {db} -f {file}\n  code de sortie = {code}   {err}");
        if code == 0 {
            let same = ref_counts.iter().all(|(k, v)| counts.get(k) == Some(&(*v as u64)));
            println!("  comptes dans le fichier : {counts:?}\n  identiques à la référence : {same}");
        }
        results.push((label, code));
    }
    println!("\nRésumé : {results:?}");
    Ok(())
}

#[tokio::main]
async fn main() -> R<()> {
    let args: Vec<String> = std::env::args().collect();
    let (cmd, target) = (args.get(1).map(String::as_str).unwrap_or(""), args.get(2).map(String::as_str).unwrap_or("local"));
    let mut t = Target::load(target)?;
    match cmd {
        "setup" => setup(&mut t).await,
        "isolation" => isolation(&mut t).await,
        "overhead" => overhead(&mut t).await,
        "backup" => backup(&mut t).await,
        "restore" => restore(&mut t).await,
        _ => Err("commande : setup | isolation | overhead | backup".into()),
    }
}
