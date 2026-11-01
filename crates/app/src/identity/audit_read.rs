//! Lecture du journal d'audit (F03 §8.1, R-007) : filtres combinables, pagination par curseur, adresse IP réservée.
//!
//! La lecture n'écrit rien. Les dates `from` et `to` sont des jours du fuseau de l'organisation, bornes incluses, convertis
//! en un intervalle semi-ouvert d'instants ; le code ne suppose pas que ce fuseau soit UTC.

// Requêtes sur les tables de l'organisation active, dans son contexte : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

use super::authorize::Role;
use super::crypto::{base64url, base64url_decode};
use super::error::{FieldIssue, ServiceError};
use super::service::{Authenticated, IdentityService, at, from_micros};
use crate::platform::clock::OUAGADOUGOU;

pub const DEFAULT_LIMIT: i64 = 25;
pub const MAX_LIMIT: i64 = 100;

/// Fragments de nom de clé retirés de `before` et `after` à la lecture, même s'ils sont arrivés dans la base (F03 règle 5 et §8.1 point 10).
const FORBIDDEN_KEYS: [&str; 6] = ["password", "token", "secret", "hash", "totp", "csrf"];

#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub actor_kind: Option<String>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditRowView {
    pub id: i64,
    pub occurred_at: DateTime<Utc>,
    pub actor_kind: String,
    pub actor_user_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub action: String,
    pub object_type: String,
    pub object_id: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    /// Absente (jamais vide) pour les rôles qui ne voient pas l'adresse IP.
    pub ip: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditPage {
    pub rows: Vec<AuditRowView>,
    pub next_cursor: Option<String>,
    pub actors: Vec<(Uuid, String)>,
}

fn strip(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(k, _)| {
                    let lower = k.to_lowercase();
                    !FORBIDDEN_KEYS.iter().any(|f| lower.contains(f))
                })
                .map(|(k, v)| (k, strip(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip).collect()),
        other => other,
    }
}

fn parse_json(text: Option<String>) -> Option<Value> {
    text.and_then(|t| serde_json::from_str::<Value>(&t).ok()).map(strip)
}

/// Les rôles qui voient l'adresse IP : ceux qui lisent le journal (gérant, comptable, propriétaire direct, DP-88).
fn sees_ip(role: Option<Role>) -> bool {
    matches!(role, Some(Role::Manager | Role::Accountant | Role::Owner))
}

/// Valeur liée à une requête dynamique : texte (converti par un ::type dans la requête) ou entier.
enum Bind {
    Text(String),
    Int(i64),
}

fn day_start(day: NaiveDate) -> Option<DateTime<Utc>> {
    OUAGADOUGOU
        .from_local_datetime(&day.and_hms_opt(0, 0, 0)?)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
}

impl IdentityService {
    /// `GET /audit` : une page du journal, du plus récent au plus ancien.
    pub async fn audit_page(&self, auth: &Authenticated, query: &AuditQuery) -> Result<AuditPage, ServiceError> {
        let org = auth
            .org
            .as_ref()
            .ok_or(ServiceError::Forbidden("organization_required"))?;
        let mut issues = vec![];
        let actor = match query.actor_id.as_deref() {
            None => None,
            Some(a) => match Uuid::parse_str(a) {
                Ok(u) => Some(u.to_string()),
                Err(_) => {
                    issues.push(FieldIssue {
                        field: "actor_id",
                        code: "validation.enum",
                    });
                    None
                }
            },
        };
        if let Some(a) = query.action.as_deref()
            && !(a.contains('.')
                && a.len() <= 60
                && a.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_')
                && a.matches('.').count() == 1)
        {
            issues.push(FieldIssue {
                field: "action",
                code: "validation.enum",
            });
        }
        if query
            .actor_kind
            .as_deref()
            .is_some_and(|k| !matches!(k, "user" | "system" | "support"))
        {
            issues.push(FieldIssue {
                field: "actor_kind",
                code: "validation.enum",
            });
        }
        if let (Some(from), Some(to)) = (query.from, query.to)
            && to < from
        {
            issues.push(FieldIssue {
                field: "to",
                code: "validation.date_order",
            });
        }
        if !issues.is_empty() {
            return Err(ServiceError::Fields(issues));
        }
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let cursor = match query.cursor.as_deref() {
            None => None,
            Some(c) => Some(read_cursor(c).ok_or(ServiceError::BadCursor)?),
        };
        // [from 00:00:00 local, lendemain de `to` 00:00:00 local[ : bornes incluses, intervalle semi-ouvert
        let start = query.from.and_then(day_start);
        let end = query.to.and_then(|d| d.succ_opt()).and_then(day_start);

        // Seuls les filtres présents entrent dans la requête : le planificateur peut alors employer l'index (org_id, occurred_at DESC)
        // pour un intervalle de dates, ce qu'il ne fait pas avec des conditions « $n IS NULL OR … » (F03 §8.1 point 9).
        let mut conditions: Vec<String> = vec![];
        let mut binds: Vec<Bind> = vec![];
        let mut add = |sql: &str, bind: Bind| {
            binds.push(bind);
            conditions.push(sql.replace("$n", &format!("${}", binds.len())));
        };
        if let Some(a) = actor {
            add("a.actor_user_id = $n::uuid", Bind::Text(a));
        }
        if let Some(a) = query.action.clone() {
            add("a.action = $n", Bind::Text(a));
        }
        if let Some(t) = query.object_type.clone() {
            add("a.object_type = $n", Bind::Text(t));
        }
        if let Some(i) = query.object_id.clone() {
            add("a.object_id = $n", Bind::Text(i));
        }
        if let Some(t) = start {
            add("a.occurred_at >= $n::timestamptz", Bind::Text(t.to_rfc3339()));
        }
        if let Some(t) = end {
            add("a.occurred_at < $n::timestamptz", Bind::Text(t.to_rfc3339()));
        }
        if let Some(k) = query.actor_kind.clone() {
            add("a.actor_kind = $n", Bind::Text(k));
        }
        if let Some((micros, id)) = cursor {
            let (m, i) = (binds.len() + 1, binds.len() + 2);
            binds.push(Bind::Int(micros));
            binds.push(Bind::Int(id));
            conditions.push(format!(
                "(a.occurred_at, a.id) < (timestamptz 'epoch' + ${m}::bigint * interval '1 microsecond', ${i}::bigint)"
            ));
        }
        binds.push(Bind::Int(limit + 1));
        let where_clause = if conditions.is_empty() {
            "TRUE".to_string()
        } else {
            conditions.join(" AND ")
        };
        let sql = format!(
            "SELECT a.id, {}, a.actor_kind, a.actor_user_id::text, u.display_name, a.action, a.object_type, a.object_id, \
                    a.before::text, a.after::text, host(a.ip), a.request_id \
               FROM audit_log a LEFT JOIN users u ON u.id = a.actor_user_id \
              WHERE {where_clause} \
              ORDER BY a.occurred_at DESC, a.id DESC LIMIT ${}",
            at("a.occurred_at"),
            binds.len()
        );
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let mut statement = sqlx::query(sqlx::AssertSqlSafe(sql));
        for bind in binds {
            statement = match bind {
                Bind::Text(t) => statement.bind(t),
                Bind::Int(n) => statement.bind(n),
            };
        }
        let rows = statement.fetch_all(tx.conn()).await?;
        let actors = sqlx::query(
            "SELECT DISTINCT u.id::text, u.display_name FROM memberships m JOIN users u ON u.id = m.user_id ORDER BY u.display_name, u.id::text",
        )
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;

        let show_ip = sees_ip(auth.role);
        let page_len = usize::try_from(limit).unwrap_or(25);
        let has_more = rows.len() > page_len;
        let mut views = Vec::with_capacity(rows.len().min(page_len));
        for r in rows.iter().take(page_len) {
            views.push(AuditRowView {
                id: r.get(0),
                occurred_at: from_micros(r.get(1))?,
                actor_kind: r.get(2),
                actor_user_id: r.get::<Option<String>, _>(3).and_then(|s| Uuid::parse_str(&s).ok()),
                actor_name: r.get(4),
                action: r.get(5),
                object_type: r.get(6),
                object_id: r.get(7),
                before: parse_json(r.get(8)),
                after: parse_json(r.get(9)),
                ip: if show_ip { r.get(10) } else { None },
                request_id: r.get(11),
            });
        }
        let next_cursor = if has_more {
            views.last().map(|v| write_cursor(v.occurred_at, v.id))
        } else {
            None
        };
        Ok(AuditPage {
            rows: views,
            next_cursor,
            actors: actors
                .iter()
                .filter_map(|r| Some((Uuid::parse_str(&r.get::<String, _>(0)).ok()?, r.get::<String, _>(1))))
                .collect(),
        })
    }
}

/// Curseur opaque : `(occurred_at en microsecondes, id)` en base64url.
fn write_cursor(occurred_at: DateTime<Utc>, id: i64) -> String {
    base64url(format!("{}:{id}", occurred_at.timestamp_micros()).as_bytes())
}

fn read_cursor(cursor: &str) -> Option<(i64, i64)> {
    let text = String::from_utf8(base64url_decode(cursor)?).ok()?;
    let (micros, id) = text.split_once(':')?;
    Some((micros.parse().ok()?, id.parse().ok()?))
}
