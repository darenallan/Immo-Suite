//! Service d'identité (F01, F02, F03 ; DP-85, DP-86) : connexion, authentification d'une requête, choix d'organisation, déconnexion.
//!
//! - Les tables globales (`users`, `sessions`, `login_attempts`) se lisent hors contexte d'organisation (`SystemTx`, `UserTx`) ; tout
//!   ce qui touche une organisation passe par `OrgTx`. Une action d'une organisation et sa ligne d'audit partagent la même transaction
//!   (R-008) : si l'audit échoue, la connexion échoue et rien n'est enregistré.
//! - Toutes les heures viennent de `Clock` (DP-50), jamais de la base ni du système.
//! - Aucun mot de passe, jeton ni identifiant en clair dans les journaux (A13).

// Requêtes sur les tables globales d'identité, hors contexte d'organisation : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::net::IpAddr;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::{PgConnection, Row};
use uuid::Uuid;

use super::authorize::{Plan, Role};
use super::identifier::Identifier;
use super::password::{MAX_CHARS, PasswordService};
use super::session::{
    self, absolute_expiry, csrf_hash, csrf_matches, csrf_token, generate_token, hash_token, idle_expiry_minutes,
};
use super::throttle::{identifier_wait, ip_wait, retry_after_seconds};
use crate::audit::{self, AuditAction, AuditContext, AuditEntry};
use crate::platform::clock::Clock;
use crate::platform::db::{Db, DbError};

/// D'où vient la requête : renseigné par la couche HTTP, jamais par le corps de la requête.
#[derive(Debug, Clone, Default)]
pub struct ClientInfo {
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
    pub client: ClientInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserView {
    pub id: Uuid,
    pub display_name: String,
}

/// Une organisation proposée à l'utilisateur, avec son rôle dans celle-ci.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgChoice {
    pub id: Uuid,
    pub name: String,
    pub plan: Plan,
    pub kind: String,
    pub lifecycle: String,
    pub role: Role,
}

/// Résultat d'une connexion ou d'un changement d'organisation : la valeur du cookie et le jeton anti-falsification.
#[derive(Debug, Clone)]
pub struct SessionGrant {
    pub session_token: String,
    pub csrf_token: String,
    pub user: UserView,
    pub organizations: Vec<OrgChoice>,
    pub active_org: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveOrg {
    pub id: Uuid,
    pub name: String,
    pub plan: Plan,
    pub kind: String,
    pub lifecycle: String,
}

/// Fiche de l'organisation active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgSummary {
    pub name: String,
    pub city: Option<String>,
    pub plan: Plan,
    pub kind: String,
    pub lifecycle: String,
}

/// Une requête authentifiée : qui, dans quelle organisation, avec quel rôle (relu en base à chaque requête).
#[derive(Debug, Clone)]
pub struct Authenticated {
    pub user: UserView,
    pub org: Option<ActiveOrg>,
    pub role: Option<Role>,
    pub session_absolute_expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum LoginError {
    /// Identifiant inconnu, mauvais mot de passe, forme invalide ou compte désactivé : même réponse (SECURITY.md §3.3).
    InvalidCredentials,
    RateLimited {
        retry_after_seconds: i64,
    },
    NoActiveMembership,
    Internal,
}

#[derive(Debug)]
pub enum AuthError {
    Unauthenticated,
    Internal,
}

#[derive(Debug)]
pub enum SelectError {
    /// Pas d'adhésion active à cette organisation : 404, même réponse que l'organisation soit inconnue ou celle d'un autre.
    NotFound,
    Unauthenticated,
    Internal,
}

impl From<DbError> for LoginError {
    fn from(e: DbError) -> Self {
        tracing::error!(error = %e, "identité : erreur de base de données");
        LoginError::Internal
    }
}

fn internal<E: std::fmt::Display>(e: E) -> DbError {
    // Les erreurs de décodage ne citent jamais de valeur.
    DbError::Query(sqlx::Error::Decode(e.to_string().into()))
}

/// Service d'identité. Cloner est peu coûteux.
#[derive(Clone)]
pub struct IdentityService {
    db: Db,
    passwords: PasswordService,
    clock: Arc<dyn Clock>,
    secret: Arc<Vec<u8>>,
    /// Durée d'inactivité forcée, en minutes (essai seulement : exercer l'expiration).
    idle_override: Option<i64>,
}

impl std::fmt::Debug for IdentityService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityService").finish_non_exhaustive()
    }
}

fn micros(t: DateTime<Utc>) -> String {
    t.to_rfc3339()
}

fn from_micros(value: i64) -> Result<DateTime<Utc>, DbError> {
    DateTime::from_timestamp_micros(value).ok_or_else(|| internal("heure illisible"))
}

const EPOCH_MICROS: &str = "(extract(epoch from {c}) * 1000000)::bigint";

fn at(column: &str) -> String {
    EPOCH_MICROS.replace("{c}", column)
}

impl IdentityService {
    pub fn new(db: Db, passwords: PasswordService, clock: Arc<dyn Clock>, secret: Vec<u8>) -> Self {
        Self {
            db,
            passwords,
            clock,
            secret: Arc::new(secret),
            idle_override: None,
        }
    }

    /// Force la durée d'inactivité (minutes), pour exercer l'expiration en essai. Les réglages l'interdisent en production.
    pub fn with_idle_minutes(mut self, minutes: Option<i64>) -> Self {
        self.idle_override = minutes;
        self
    }

    fn idle_expiry(&self, now: DateTime<Utc>, role: Role, absolute: DateTime<Utc>) -> DateTime<Utc> {
        idle_expiry_minutes(now, self.idle_override.unwrap_or_else(|| role.idle_minutes()), absolute)
    }

    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Les organisations dont l'utilisateur est membre actif, avec son rôle dans chacune.
    pub async fn organizations_of(&self, user_id: Uuid) -> Result<Vec<OrgChoice>, DbError> {
        self.my_organizations(user_id).await
    }

    /// Fiche de l'organisation active de la session (lue sous la RLS : seulement la sienne).
    pub async fn organization_summary(&self, auth: &Authenticated) -> Result<OrgSummary, DbError> {
        let org = auth
            .org
            .as_ref()
            .ok_or_else(|| internal("aucune organisation active"))?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let row =
            sqlx::query("SELECT name, city, plan, kind, lifecycle FROM organizations WHERE id = current_org_id()")
                .fetch_one(tx.conn())
                .await?;
        tx.commit().await?;
        Ok(OrgSummary {
            name: row.get(0),
            city: row.get(1),
            plan: Plan::parse(&row.get::<String, _>(2)).ok_or_else(|| internal("plan inconnu"))?,
            kind: row.get(3),
            lifecycle: row.get(4),
        })
    }

    /// Meilleur effort : trace `security.permission_denied` (jeton anti-falsification ou `Origin` absent ou faux) chez l'organisation
    /// active. Un échec d'écriture est journalisé et ne change pas la réponse de refus.
    pub async fn trace_permission_denied(&self, auth: &Authenticated, route: &str, reason: &str, client: &ClientInfo) {
        let Some(org) = &auth.org else { return };
        let result: Result<(), DbError> = async {
            let mut tx = self.db.begin(org.id, auth.user.id).await?;
            let entry = AuditEntry::new(AuditAction::SecurityPermissionDenied, "route", route)
                .after(json!({ "reason": reason }))
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
            tx.commit().await
        }
        .await;
        if let Err(e) = result {
            tracing::error!(error = %e, "identité : refus non tracé");
        }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Jeton anti-falsification attendu pour cette session (à renvoyer à la connexion et par `GET /me`).
    pub fn csrf_for(&self, session_token: &str) -> String {
        csrf_token(&self.secret, session_token).unwrap_or_default()
    }

    /// Le jeton présenté dans `X-CSRF-Token` est-il celui de la session ? (comparaison en temps constant)
    pub fn csrf_valid(&self, session_token: &str, presented: &str) -> bool {
        csrf_matches(&self.secret, session_token, presented)
    }

    // ───────── connexion ─────────

    pub async fn login(&self, req: LoginRequest) -> Result<SessionGrant, LoginError> {
        let now = self.clock.now();
        let parsed = Identifier::parse(&req.identifier);
        // Clé de suivi : l'identifiant normalisé, ou à défaut la saisie en minuscules (une saisie invalide est suivie aussi).
        let key = parsed.as_ref().map_or_else(
            || Identifier::Email(req.identifier.trim().to_lowercase()).hash(),
            Identifier::hash,
        );
        let ip = req.client.ip;

        // 1. limitation : aucune vérification de mot de passe pendant une attente, et la requête ne compte pas
        {
            let mut tx = self.db.begin_system().await?;
            let id_failures = self.consecutive_failures(tx.conn(), &key).await?;
            let ip_failures = match ip {
                Some(ip) => self.recent_ip_failures(tx.conn(), ip, now).await?,
                None => vec![],
            };
            tx.commit().await?;
            let wait = retry_after_seconds(identifier_wait(&id_failures, now), ip_wait(&ip_failures, now));
            if let Some(seconds) = wait {
                return Err(LoginError::RateLimited {
                    retry_after_seconds: seconds,
                });
            }
        }

        // 2. recherche du compte et vérification (factice si le compte n'existe pas, est désactivé ou n'a pas de mot de passe)
        let user = match &parsed {
            Some(id) => self.find_user(id).await?,
            None => None,
        };
        let too_long = req.password.chars().count() > MAX_CHARS;
        let stored_hash = user
            .as_ref()
            .filter(|u| u.active)
            .and_then(|u| u.password_hash.as_deref());
        let verified = if too_long {
            false
        } else {
            self.passwords.verify(&req.password, stored_hash).await.map_err(|e| {
                tracing::error!(error = %e, "identité : vérification du mot de passe impossible");
                LoginError::Internal
            })?
        };

        let Some(user) = user.filter(|_| verified) else {
            self.record_failure(&key, ip, now, parsed.as_ref()).await?;
            return Err(LoginError::InvalidCredentials);
        };

        // 3. organisations de l'utilisateur
        let orgs = self.my_organizations(user.id).await?;
        let Some(first) = orgs.first() else {
            // mot de passe juste : la tentative réussie remet le compteur à zéro, mais aucune session n'est créée
            let mut tx = self.db.begin_system().await?;
            self.insert_attempt(tx.conn(), &key, ip, true, now).await?;
            tx.commit().await?;
            return Err(LoginError::NoActiveMembership);
        };

        let token = generate_token().map_err(|_| LoginError::Internal)?;
        let csrf = self.csrf_for(&token);
        let absolute = absolute_expiry(now);

        if let [only] = orgs.as_slice() {
            // une seule organisation : tout (tentative, session, audit) dans la transaction de cette organisation (R-008)
            let mut tx = self.db.begin(only.id, user.id).await?;
            self.insert_attempt(tx.conn(), &key, ip, true, now).await?;
            self.insert_session(
                tx.conn(),
                &token,
                user.id,
                Some(only.id),
                only.role,
                now,
                absolute,
                &req.client,
            )
            .await?;
            sqlx::query("UPDATE users SET last_login_at = $2::timestamptz WHERE id = $1::uuid")
                .bind(user.id.to_string())
                .bind(micros(now))
                .execute(tx.conn())
                .await
                .map_err(DbError::from)?;
            let entry = AuditEntry::new(AuditAction::AuthLogin, "user", user.id)
                .after(json!({ "org_role": only.role.as_str() }))
                .map_err(|_| LoginError::Internal)?;
            audit::record(&mut tx, &audit_context(user.id, &req.client), entry).await?;
            tx.commit().await?;
            Ok(SessionGrant {
                session_token: token,
                csrf_token: csrf,
                user: UserView {
                    id: user.id,
                    display_name: user.display_name,
                },
                active_org: Some(only.id),
                organizations: orgs,
            })
        } else {
            // plusieurs organisations : session sans organisation active ; `auth.login` est écrit au premier choix
            let _ = first;
            let mut tx = self.db.begin_system().await?;
            self.insert_attempt(tx.conn(), &key, ip, true, now).await?;
            self.insert_session(
                tx.conn(),
                &token,
                user.id,
                None,
                Role::Cashier,
                now,
                absolute,
                &req.client,
            )
            .await?;
            sqlx::query("UPDATE users SET last_login_at = $2::timestamptz WHERE id = $1::uuid")
                .bind(user.id.to_string())
                .bind(micros(now))
                .execute(tx.conn())
                .await
                .map_err(DbError::from)?;
            tx.commit().await?;
            Ok(SessionGrant {
                session_token: token,
                csrf_token: csrf,
                user: UserView {
                    id: user.id,
                    display_name: user.display_name,
                },
                active_org: None,
                organizations: orgs,
            })
        }
    }

    async fn find_user(&self, id: &Identifier) -> Result<Option<UserRow>, DbError> {
        let column = match id {
            Identifier::Phone(_) => "login_phone",
            Identifier::Email(_) => "login_email",
        };
        let mut tx = self.db.begin_system().await?;
        let row = sqlx::query(sqlx::AssertSqlSafe(format!(
            "SELECT id::text, display_name, password_hash, status FROM users WHERE {column} = $1"
        )))
        .bind(id.as_str())
        .fetch_optional(tx.conn())
        .await?;
        tx.commit().await?;
        row.map(|r| {
            Ok(UserRow {
                id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
                display_name: r.get(1),
                password_hash: r.get(2),
                active: r.get::<String, _>(3) == "active",
            })
        })
        .transpose()
    }

    /// Échecs consécutifs depuis la dernière réussite, du plus récent au plus ancien (20 au plus).
    async fn consecutive_failures(
        &self,
        conn: &mut PgConnection,
        key: &[u8; 32],
    ) -> Result<Vec<DateTime<Utc>>, DbError> {
        let sql = format!(
            "SELECT {} FROM login_attempts WHERE identifier_hash = $1 AND NOT success \
               AND attempted_at > COALESCE((SELECT max(attempted_at) FROM login_attempts WHERE identifier_hash = $1 AND success), '-infinity'::timestamptz) \
             ORDER BY attempted_at DESC LIMIT 20",
            at("attempted_at")
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(key.to_vec())
            .fetch_all(conn)
            .await?;
        rows.iter().map(|r| from_micros(r.get::<i64, _>(0))).collect()
    }

    /// Échecs des 25 dernières minutes depuis cette adresse (200 au plus).
    async fn recent_ip_failures(
        &self,
        conn: &mut PgConnection,
        ip: IpAddr,
        now: DateTime<Utc>,
    ) -> Result<Vec<DateTime<Utc>>, DbError> {
        let sql = format!(
            "SELECT {} FROM login_attempts WHERE ip = $1::inet AND NOT success AND attempted_at > $2::timestamptz - interval '25 minutes' \
             ORDER BY attempted_at DESC LIMIT 200",
            at("attempted_at")
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(ip.to_string())
            .bind(micros(now))
            .fetch_all(conn)
            .await?;
        rows.iter().map(|r| from_micros(r.get::<i64, _>(0))).collect()
    }

    async fn insert_attempt(
        &self,
        conn: &mut PgConnection,
        key: &[u8; 32],
        ip: Option<IpAddr>,
        success: bool,
        now: DateTime<Utc>,
    ) -> Result<(), DbError> {
        sqlx::query("INSERT INTO login_attempts (identifier_hash, ip, success, attempted_at) VALUES ($1, $2::inet, $3, $4::timestamptz)")
            .bind(key.to_vec())
            .bind(ip.map(|i| i.to_string()))
            .bind(success)
            .bind(micros(now))
            .execute(conn)
            .await?;
        Ok(())
    }

    /// Enregistre l'échec ; au cinquième échec consécutif, trace `security.rate_limited` chez les organisations du compte (F01 §4).
    async fn record_failure(
        &self,
        key: &[u8; 32],
        ip: Option<IpAddr>,
        now: DateTime<Utc>,
        parsed: Option<&Identifier>,
    ) -> Result<(), LoginError> {
        let mut tx = self.db.begin_system().await?;
        self.insert_attempt(tx.conn(), key, ip, false, now).await?;
        let failures = self.consecutive_failures(tx.conn(), key).await?;
        tx.commit().await?;
        if failures.len() == super::throttle::FAILURES_BEFORE_WAIT
            && let Some(id) = parsed
            && let Ok(Some(user)) = self.find_user(id).await
        {
            self.trace_rate_limit(user.id, ip).await;
        }
        Ok(())
    }

    /// Meilleur effort : un échec d'écriture ici ne change pas la réponse à l'appelant, mais il est journalisé.
    async fn trace_rate_limit(&self, user_id: Uuid, ip: Option<IpAddr>) {
        let Ok(orgs) = self.my_organizations(user_id).await else {
            return;
        };
        for org in orgs {
            let result: Result<(), DbError> = async {
                let mut tx = self.db.begin(org.id, user_id).await?;
                let entry = AuditEntry::new(AuditAction::SecurityRateLimited, "user", user_id)
                    .after(json!({ "wait_seconds": 60 }))
                    .map_err(internal)?;
                let ctx = AuditContext {
                    actor_user_id: Some(user_id),
                    ip,
                    request_id: None,
                };
                audit::record(&mut tx, &ctx, entry).await?;
                tx.commit().await
            }
            .await;
            if let Err(e) = result {
                tracing::error!(error = %e, "identité : trace de limitation non écrite");
            }
        }
    }

    // ───────── organisations et sessions ─────────

    /// Adhésions actives de l'utilisateur, avec les champs d'affichage de leurs organisations (fonction `my_organizations`).
    async fn my_organizations(&self, user_id: Uuid) -> Result<Vec<OrgChoice>, DbError> {
        let mut tx = self.db.begin_user(user_id).await?;
        let rows = sqlx::query("SELECT org_id::text, org_name, plan, kind, lifecycle, role FROM my_organizations()")
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        rows.iter()
            .map(|r| {
                Ok(OrgChoice {
                    id: Uuid::parse_str(&r.get::<String, _>(0)).map_err(internal)?,
                    name: r.get(1),
                    plan: Plan::parse(&r.get::<String, _>(2)).ok_or_else(|| internal("plan inconnu"))?,
                    kind: r.get(3),
                    lifecycle: r.get(4),
                    role: Role::parse(&r.get::<String, _>(5)).ok_or_else(|| internal("rôle inconnu"))?,
                })
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_session(
        &self,
        conn: &mut PgConnection,
        token: &str,
        user_id: Uuid,
        org_id: Option<Uuid>,
        role: Role,
        now: DateTime<Utc>,
        absolute: DateTime<Utc>,
        client: &ClientInfo,
    ) -> Result<(), DbError> {
        let id_hash = hash_token(token).ok_or_else(|| internal("jeton"))?;
        let csrf = self.csrf_for(token);
        sqlx::query(
            "INSERT INTO sessions (id_hash, user_id, org_id, csrf_hash, created_at, last_seen_at, idle_expires_at, absolute_expires_at, ip, user_agent) \
             VALUES ($1, $2::uuid, $3::uuid, $4, $5::timestamptz, $5::timestamptz, $6::timestamptz, $7::timestamptz, $8::inet, $9)",
        )
        .bind(id_hash.to_vec())
        .bind(user_id.to_string())
        .bind(org_id.map(|o| o.to_string()))
        .bind(csrf_hash(&csrf).to_vec())
        .bind(micros(now))
        .bind(micros(self.idle_expiry(now, role, absolute)))
        .bind(micros(absolute))
        .bind(client.ip.map(|i| i.to_string()))
        .bind(client.user_agent.as_deref().map(|ua| ua.chars().take(300).collect::<String>()))
        .execute(conn)
        .await?;
        Ok(())
    }

    // ───────── authentification d'une requête ─────────

    /// Reconnaît la session du cookie. Le rôle, l'état de l'adhésion, celui du compte et le cycle de vie de l'organisation sont relus
    /// en base à chaque appel (F01 règle 9). Une adhésion retirée ou un compte désactivé révoque la session.
    pub async fn authenticate(&self, token: &str) -> Result<Authenticated, AuthError> {
        let Some(id_hash) = hash_token(token) else {
            return Err(AuthError::Unauthenticated);
        };
        let now = self.clock.now();
        let run = async {
            let mut tx = self.db.begin_system().await?;
            let sql = format!(
                "SELECT s.user_id::text, s.org_id::text, u.display_name, u.status, {} \
                   FROM sessions s JOIN users u ON u.id = s.user_id \
                  WHERE s.id_hash = $1 AND s.revoked_at IS NULL AND s.idle_expires_at > $2::timestamptz AND s.absolute_expires_at > $2::timestamptz",
                at("s.absolute_expires_at")
            );
            let row = sqlx::query(sqlx::AssertSqlSafe(sql))
                .bind(id_hash.to_vec())
                .bind(micros(now))
                .fetch_optional(tx.conn())
                .await?;
            tx.commit().await?;
            let Some(row) = row else { return Ok(None) };
            let user_id = Uuid::parse_str(&row.get::<String, _>(0)).map_err(internal)?;
            let org_id = row
                .get::<Option<String>, _>(1)
                .map(|s| Uuid::parse_str(&s).map_err(internal))
                .transpose()?;
            let user = UserView {
                id: user_id,
                display_name: row.get(2),
            };
            let absolute = from_micros(row.get::<i64, _>(4))?;
            if row.get::<String, _>(3) != "active" {
                self.revoke(&id_hash, now).await?;
                return Ok(None);
            }
            let (org, role) = match org_id {
                None => (None, None),
                Some(org_id) => {
                    let mut tx = self.db.begin(org_id, user_id).await?;
                    let found = sqlx::query(
                        "SELECT m.role, m.status, o.name, o.plan, o.kind, o.lifecycle FROM memberships m \
                           JOIN organizations o ON o.id = m.org_id WHERE m.user_id = $1::uuid AND m.org_id = current_org_id()",
                    )
                    .bind(user_id.to_string())
                    .fetch_optional(tx.conn())
                    .await?;
                    tx.commit().await?;
                    let usable = found
                        .as_ref()
                        .filter(|r| r.get::<String, _>(1) == "active" && r.get::<String, _>(5) != "closed");
                    let Some(r) = usable else {
                        self.revoke(&id_hash, now).await?;
                        return Ok(None);
                    };
                    let (Some(role), Some(plan)) =
                        (Role::parse(&r.get::<String, _>(0)), Plan::parse(&r.get::<String, _>(3)))
                    else {
                        self.revoke(&id_hash, now).await?;
                        return Ok(None);
                    };
                    if role.plan() != plan {
                        self.revoke(&id_hash, now).await?;
                        return Ok(None);
                    }
                    let active = ActiveOrg {
                        id: org_id,
                        name: r.get(2),
                        plan,
                        kind: r.get(4),
                        lifecycle: r.get(5),
                    };
                    (Some(active), Some(role))
                }
            };
            // l'activité repousse l'échéance d'inactivité, jamais au-delà de l'expiration absolue
            let idle = self.idle_expiry(now, role.unwrap_or(Role::Cashier), absolute);
            let mut tx = self.db.begin_system().await?;
            sqlx::query("UPDATE sessions SET last_seen_at = $2::timestamptz, idle_expires_at = $3::timestamptz WHERE id_hash = $1")
                .bind(id_hash.to_vec())
                .bind(micros(now))
                .bind(micros(idle))
                .execute(tx.conn())
                .await?;
            tx.commit().await?;
            Ok::<_, DbError>(Some(Authenticated {
                user,
                org,
                role,
                session_absolute_expires_at: absolute,
            }))
        };
        match run.await {
            Ok(Some(a)) => Ok(a),
            Ok(None) => Err(AuthError::Unauthenticated),
            Err(e) => {
                tracing::error!(error = %e, "identité : authentification impossible");
                Err(AuthError::Internal)
            }
        }
    }

    async fn revoke(&self, id_hash: &[u8; 32], now: DateTime<Utc>) -> Result<(), DbError> {
        let mut tx = self.db.begin_system().await?;
        sqlx::query("UPDATE sessions SET revoked_at = $2::timestamptz WHERE id_hash = $1 AND revoked_at IS NULL")
            .bind(id_hash.to_vec())
            .bind(micros(now))
            .execute(tx.conn())
            .await?;
        tx.commit().await
    }

    // ───────── choix d'organisation ─────────

    /// Change l'organisation active (revérifie l'adhésion), avec un nouvel identifiant de session. Au premier choix d'une session
    /// sans organisation, écrit `auth.login`. Une organisation dont l'utilisateur n'est pas membre : 404 ; si elle existe chez un
    /// autre, `security.cross_org_attempt` est écrit chez l'organisation active de l'appelant (DP-86).
    pub async fn select_organization(
        &self,
        token: &str,
        org_id: Uuid,
        client: &ClientInfo,
    ) -> Result<SessionGrant, SelectError> {
        let current = self.authenticate(token).await.map_err(|e| match e {
            AuthError::Unauthenticated => SelectError::Unauthenticated,
            AuthError::Internal => SelectError::Internal,
        })?;
        let to_internal = |e: DbError| {
            tracing::error!(error = %e, "identité : changement d'organisation impossible");
            SelectError::Internal
        };
        let orgs = self.my_organizations(current.user.id).await.map_err(to_internal)?;
        let Some(target) = orgs.iter().find(|o| o.id == org_id).cloned() else {
            if let Some(active) = &current.org {
                self.trace_cross_org(active.id, current.user.id, org_id, client)
                    .await
                    .map_err(to_internal)?;
            }
            return Err(SelectError::NotFound);
        };

        let now = self.clock.now();
        let new_token = generate_token().map_err(|_| SelectError::Internal)?;
        let old_hash = hash_token(token).ok_or(SelectError::Unauthenticated)?;
        let result: Result<(), DbError> = async {
            let mut tx = self.db.begin(target.id, current.user.id).await?;
            self.insert_session(
                tx.conn(),
                &new_token,
                current.user.id,
                Some(target.id),
                target.role,
                now,
                current.session_absolute_expires_at,
                client,
            )
            .await?;
            sqlx::query("UPDATE sessions SET revoked_at = $2::timestamptz WHERE id_hash = $1 AND revoked_at IS NULL")
                .bind(old_hash.to_vec())
                .bind(micros(now))
                .execute(tx.conn())
                .await?;
            if current.org.is_none() {
                let entry = AuditEntry::new(AuditAction::AuthLogin, "user", current.user.id)
                    .after(json!({ "org_role": target.role.as_str() }))
                    .map_err(internal)?;
                audit::record(&mut tx, &audit_context(current.user.id, client), entry).await?;
            }
            tx.commit().await
        }
        .await;
        result.map_err(to_internal)?;
        Ok(SessionGrant {
            csrf_token: self.csrf_for(&new_token),
            session_token: new_token,
            user: current.user,
            active_org: Some(target.id),
            organizations: orgs,
        })
    }

    /// `security.cross_org_attempt` : seulement si l'organisation demandée existe réellement chez un autre (DP-86).
    async fn trace_cross_org(
        &self,
        active_org: Uuid,
        user_id: Uuid,
        wanted: Uuid,
        client: &ClientInfo,
    ) -> Result<(), DbError> {
        let mut tx = self.db.begin(active_org, user_id).await?;
        let exists: bool = sqlx::query_scalar("SELECT exists_in_other_org('organization', $1::uuid)")
            .bind(wanted.to_string())
            .fetch_one(tx.conn())
            .await?;
        if exists {
            let entry = AuditEntry::new(AuditAction::SecurityCrossOrgAttempt, "organization", wanted);
            audit::record(&mut tx, &audit_context(user_id, client), entry).await?;
            // alerte immédiate (S1) : ligne de journal structurée, sans donnée personnelle ; le canal viendra avec la dette de l'étape 01
            tracing::error!(alert = "security.cross_org_attempt", org_id = %active_org, "tentative d'accès à une autre organisation");
        }
        tx.commit().await
    }

    // ───────── déconnexion ─────────

    /// Révoque cette session. Idempotent : une session déjà révoquée, expirée ou inconnue répond « fait ».
    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        let Some(id_hash) = hash_token(token) else {
            return Ok(());
        };
        self.revoke(&id_hash, self.clock.now()).await.map_err(|e| {
            tracing::error!(error = %e, "identité : déconnexion impossible");
            AuthError::Internal
        })
    }

    /// Révoque toutes les sessions actives de l'utilisateur et trace `session.revoked_all` chez l'organisation active (F01 §3).
    pub async fn logout_all(&self, token: &str, client: &ClientInfo) -> Result<(), AuthError> {
        let current = self.authenticate(token).await?;
        let now = self.clock.now();
        let run: Result<(), DbError> = async {
            match &current.org {
                Some(org) => {
                    let mut tx = self.db.begin(org.id, current.user.id).await?;
                    let done = sqlx::query(
                        "UPDATE sessions SET revoked_at = $2::timestamptz WHERE user_id = $1::uuid AND revoked_at IS NULL AND absolute_expires_at > $2::timestamptz",
                    )
                    .bind(current.user.id.to_string())
                    .bind(micros(now))
                    .execute(tx.conn())
                    .await?;
                    let entry = AuditEntry::new(AuditAction::SessionRevokedAll, "user", current.user.id)
                        .after(json!({ "sessions": done.rows_affected() }))
                        .map_err(internal)?;
                    audit::record(&mut tx, &audit_context(current.user.id, client), entry).await?;
                    tx.commit().await
                }
                None => {
                    let mut tx = self.db.begin_system().await?;
                    sqlx::query("UPDATE sessions SET revoked_at = $2::timestamptz WHERE user_id = $1::uuid AND revoked_at IS NULL")
                        .bind(current.user.id.to_string())
                        .bind(micros(now))
                        .execute(tx.conn())
                        .await?;
                    tx.commit().await
                }
            }
        }
        .await;
        run.map_err(|e| {
            tracing::error!(error = %e, "identité : déconnexion générale impossible");
            AuthError::Internal
        })
    }

    /// Supprime les tentatives de plus de 30 jours (tâche de fond, SECURITY.md §6.3). Retourne le nombre de lignes supprimées.
    pub async fn purge_old_attempts(&self) -> Result<u64, DbError> {
        let cutoff = self.clock.now() - Duration::days(30);
        let mut tx = self.db.begin_system().await?;
        let done = sqlx::query("DELETE FROM login_attempts WHERE attempted_at < $1::timestamptz")
            .bind(micros(cutoff))
            .execute(tx.conn())
            .await?;
        tx.commit().await?;
        Ok(done.rows_affected())
    }
}

struct UserRow {
    id: Uuid,
    display_name: String,
    password_hash: Option<String>,
    active: bool,
}

fn audit_context(user_id: Uuid, client: &ClientInfo) -> AuditContext {
    AuditContext {
        actor_user_id: Some(user_id),
        ip: client.ip,
        request_id: client.request_id.clone(),
    }
}

/// Le cookie `Set-Cookie` de la session (F01 §3 point 6) : `__Host-session`, `Secure`, `HttpOnly`, `SameSite=Lax`, chemin `/`,
/// sans `Domain`. `max_age_seconds` = 12 heures au plus.
pub fn session_cookie(token: &str, max_age_seconds: i64) -> String {
    format!(
        "{}={token}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}",
        session::COOKIE_NAME
    )
}

/// Cookie d'effacement (déconnexion).
pub fn clear_session_cookie() -> String {
    format!(
        "{}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0",
        session::COOKIE_NAME
    )
}
