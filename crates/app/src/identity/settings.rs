//! Paramètres de l'organisation et droits réglables (F01 §9.8, F02 §7.1, DP-88).
//!
//! `If-Match` porte la version de la ligne (`row_version`) : périmée → `412`, absente → `428`. Chaque modification écrit
//! `settings.updated` avec les seules clés changées, dans la même transaction (R-008).

// Requêtes sur les tables de l'organisation active, dans son contexte : exception documentée de DP-77, réservée à ce module.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::DateTime;
use serde_json::{Map, Value, json};
use sqlx::Row;

use super::authorize::{Plan, Role};
use super::error::{FieldIssue, ServiceError, sql_code};
use super::service::{ActiveOrg, Authenticated, ClientInfo, IdentityService, audit_context, internal, micros};
use crate::audit::{self, AuditAction, AuditEntry};

/// Fiche de l'organisation (`GET /organization`).
#[derive(Debug, Clone)]
pub struct OrgDetail {
    pub name: String,
    pub city: Option<String>,
    pub plan: Plan,
    pub kind: String,
    pub lifecycle: String,
    pub version: i32,
}

/// Paramètres (`GET /settings`).
#[derive(Debug, Clone)]
pub struct OrgSettingsView {
    pub receipt_prefix: String,
    pub receipt_header: Option<String>,
    pub address: Option<String>,
    pub legal_ids: Value,
    pub default_paper: String,
    pub share_link_ttl_days: i32,
    pub payment_confirm_threshold: i64,
    pub version: i32,
}

/// Modification partielle des paramètres. Une valeur absente ne change rien ; une chaîne vide efface un texte facultatif.
/// `share_link_ttl_days` et `payment_confirm_threshold` arrivent en JSON brut : un décimal ou un négatif est refusé
/// par validation (règle d'or 1), pas par une erreur de format.
#[derive(Debug, Clone, Default)]
pub struct SettingsPatch {
    pub receipt_prefix: Option<String>,
    pub receipt_header: Option<String>,
    pub address: Option<String>,
    pub legal_ids: Option<Value>,
    pub default_paper: Option<String>,
    pub share_link_ttl_days: Option<Value>,
    pub payment_confirm_threshold: Option<Value>,
}

/// Un droit réglable et son état (`GET /role-settings`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSettingView {
    pub role: Role,
    pub action: &'static str,
    pub allowed: bool,
    /// 0 tant qu'aucune ligne n'existe : la valeur d'`If-Match` pour créer la ligne.
    pub version: i32,
}

/// Les quatre réglages applicables, par plan (F02 §7.1) : gestionnaire × 3 en Agence, collaborateur × encaisser en Propriétaire.
pub fn applicable_settings(plan: Plan) -> &'static [(Role, &'static str)] {
    match plan {
        Plan::Agency => &[
            (Role::PropertyManager, "record_payment"),
            (Role::PropertyManager, "import_data"),
            (Role::PropertyManager, "export_data"),
        ],
        Plan::Owner => &[(Role::Collaborator, "record_payment")],
    }
}

fn org_of(auth: &Authenticated) -> Result<&ActiveOrg, ServiceError> {
    auth.org
        .as_ref()
        .ok_or(ServiceError::Forbidden("organization_required"))
}

fn issue(field: &'static str, code: &'static str) -> FieldIssue {
    FieldIssue { field, code }
}

fn trimmed_or_none(text: &str) -> Option<String> {
    let t = text.trim();
    (!t.is_empty()).then(|| t.to_string())
}

impl IdentityService {
    /// `GET /organization`.
    pub async fn organization_detail(&self, auth: &Authenticated) -> Result<OrgDetail, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let r = sqlx::query(
            "SELECT name, city, plan, kind, lifecycle, row_version FROM organizations WHERE id = current_org_id()",
        )
        .fetch_one(tx.conn())
        .await?;
        tx.commit().await?;
        Ok(OrgDetail {
            name: r.get(0),
            city: r.get(1),
            plan: Plan::parse(&r.get::<String, _>(2)).ok_or_else(|| ServiceError::from(internal("plan inconnu")))?,
            kind: r.get(3),
            lifecycle: r.get(4),
            version: r.get(5),
        })
    }

    /// `PATCH /organization` : nom (2 à 120 caractères) et ville ; `If-Match` obligatoire.
    pub async fn patch_organization(
        &self,
        auth: &Authenticated,
        name: Option<&str>,
        city: Option<&str>,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<OrgDetail, ServiceError> {
        let org = org_of(auth)?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut issues = vec![];
        let new_name = name.map(str::trim).map(str::to_string);
        if let Some(n) = &new_name {
            let length = n.chars().count();
            if length < 2 {
                issues.push(issue("name", "validation.required"));
            } else if length > 120 {
                issues.push(issue("name", "validation.too_long"));
            }
        }
        let new_city = city.map(trimmed_or_none);
        if new_city
            .as_ref()
            .and_then(Option::as_ref)
            .is_some_and(|c| c.chars().count() > 120)
        {
            issues.push(issue("city", "validation.too_long"));
        }
        if !issues.is_empty() {
            return Err(ServiceError::Fields(issues));
        }
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current =
            sqlx::query("SELECT name, city, row_version FROM organizations WHERE id = current_org_id() FOR UPDATE")
                .fetch_one(tx.conn())
                .await?;
        if current.get::<i32, _>(2) != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let (old_name, old_city): (String, Option<String>) = (current.get(0), current.get(1));
        let final_name = new_name.clone().unwrap_or_else(|| old_name.clone());
        let final_city = new_city.clone().unwrap_or_else(|| old_city.clone());
        let (mut before, mut after) = (Map::new(), Map::new());
        if final_name != old_name {
            before.insert("name".into(), json!(old_name));
            after.insert("name".into(), json!(final_name));
        }
        if final_city != old_city {
            before.insert("city".into(), json!(old_city));
            after.insert("city".into(), json!(final_city));
        }
        if !after.is_empty() {
            sqlx::query("UPDATE organizations SET name = $1, city = $2, row_version = row_version + 1 WHERE id = current_org_id()")
                .bind(&final_name)
                .bind(&final_city)
                .execute(tx.conn())
                .await?;
            let entry = AuditEntry::new(AuditAction::SettingsUpdated, "organization", org.id)
                .before(Value::Object(before))
                .and_then(|e| e.after(Value::Object(after)))
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        tx.commit().await?;
        self.organization_detail(auth).await
    }

    fn settings_row(r: &sqlx::postgres::PgRow) -> OrgSettingsView {
        let letterhead: Value = serde_json::from_str(&r.get::<String, _>(1)).unwrap_or(Value::Null);
        let text = |key: &str| letterhead.get(key).and_then(Value::as_str).map(str::to_string);
        OrgSettingsView {
            receipt_prefix: r.get(0),
            receipt_header: text("header"),
            address: text("address"),
            legal_ids: serde_json::from_str(&r.get::<String, _>(2)).unwrap_or(Value::Null),
            default_paper: r.get(3),
            share_link_ttl_days: r.get(4),
            payment_confirm_threshold: r.get(5),
            version: r.get(6),
        }
    }

    const SETTINGS_SQL: &'static str = "SELECT receipt_prefix, letterhead::text, legal_ids::text, default_paper, share_link_ttl_days, payment_confirm_threshold, row_version \
                                        FROM org_settings WHERE org_id = current_org_id()";

    /// `GET /settings` (et valeurs de `GET /me`).
    pub async fn org_settings(&self, auth: &Authenticated) -> Result<OrgSettingsView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let r = sqlx::query(Self::SETTINGS_SQL).fetch_one(tx.conn()).await?;
        tx.commit().await?;
        Ok(Self::settings_row(&r))
    }

    /// `PATCH /settings` : validation de tous les champs avant l'écriture ; toutes les erreurs reviennent ensemble.
    pub async fn patch_settings(
        &self,
        auth: &Authenticated,
        patch: &SettingsPatch,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<OrgSettingsView, ServiceError> {
        let org = org_of(auth)?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut issues = vec![];

        let prefix = patch.receipt_prefix.as_deref().map(str::trim);
        if let Some(p) = prefix
            && (!(2..=8).contains(&p.chars().count())
                || !p.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
        {
            issues.push(issue("receipt_prefix", "validation.enum"));
        }
        let header = patch.receipt_header.as_deref().map(trimmed_or_none);
        if header
            .as_ref()
            .and_then(Option::as_ref)
            .is_some_and(|h| h.chars().count() > 200)
        {
            issues.push(issue("receipt_header", "validation.too_long"));
        }
        let address = patch.address.as_deref().map(trimmed_or_none);
        if address
            .as_ref()
            .and_then(Option::as_ref)
            .is_some_and(|a| a.chars().count() > 300)
        {
            issues.push(issue("address", "validation.too_long"));
        }
        let paper = patch.default_paper.as_deref();
        if paper.is_some_and(|p| !matches!(p, "a4" | "a5")) {
            issues.push(issue("default_paper", "validation.enum"));
        }
        // un décimal (1000000.5), un négatif ou un texte n'est jamais accepté : entier seulement (règle d'or 1)
        let ttl = match &patch.share_link_ttl_days {
            None => None,
            Some(v) => match v
                .as_i64()
                .filter(|n| (1..=365).contains(n))
                .and_then(|n| i32::try_from(n).ok())
            {
                Some(n) => Some(n),
                None => {
                    issues.push(issue("share_link_ttl_days", "validation.amount_out_of_range"));
                    None
                }
            },
        };
        let threshold = match &patch.payment_confirm_threshold {
            None => None,
            Some(v) => match v {
                Value::Number(n) if n.is_i64() || n.is_u64() => {
                    match n.as_i64().filter(|x| (1..=999_999_999).contains(x)) {
                        Some(x) => Some(x),
                        None => {
                            issues.push(issue("payment_confirm_threshold", "validation.amount_out_of_range"));
                            None
                        }
                    }
                }
                _ => {
                    issues.push(issue("payment_confirm_threshold", "validation.amount_not_integer"));
                    None
                }
            },
        };
        let legal_ids = match &patch.legal_ids {
            None => None,
            Some(Value::Object(map))
                if map.len() <= 10
                    && map.iter().all(|(k, v)| {
                        k.chars().count() <= 40 && v.as_str().is_some_and(|s| s.chars().count() <= 80)
                    }) =>
            {
                Some(Value::Object(map.clone()))
            }
            Some(_) => {
                issues.push(issue("legal_ids", "validation.enum"));
                None
            }
        };
        if !issues.is_empty() {
            return Err(ServiceError::Fields(issues));
        }

        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let locked = sqlx::query(sqlx::AssertSqlSafe(format!("{} FOR UPDATE", Self::SETTINGS_SQL)))
            .fetch_one(tx.conn())
            .await?;
        let current = Self::settings_row(&locked);
        if current.version != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let mut letterhead = Map::new();
        let new_header = header.clone().unwrap_or_else(|| current.receipt_header.clone());
        let new_address = address.clone().unwrap_or_else(|| current.address.clone());
        if let Some(h) = &new_header {
            letterhead.insert("header".into(), json!(h));
        }
        if let Some(a) = &new_address {
            letterhead.insert("address".into(), json!(a));
        }
        let new = OrgSettingsView {
            receipt_prefix: prefix.map_or_else(|| current.receipt_prefix.clone(), str::to_string),
            receipt_header: new_header,
            address: new_address,
            legal_ids: legal_ids.unwrap_or_else(|| current.legal_ids.clone()),
            default_paper: paper.map_or_else(|| current.default_paper.clone(), str::to_string),
            share_link_ttl_days: ttl.unwrap_or(current.share_link_ttl_days),
            payment_confirm_threshold: threshold.unwrap_or(current.payment_confirm_threshold),
            version: current.version,
        };
        let (mut before, mut after) = (Map::new(), Map::new());
        let mut note = |key: &str, old: Value, newv: Value| {
            if old != newv {
                before.insert(key.to_string(), old);
                after.insert(key.to_string(), newv);
            }
        };
        note(
            "receipt_prefix",
            json!(current.receipt_prefix),
            json!(new.receipt_prefix),
        );
        note(
            "receipt_header",
            json!(current.receipt_header),
            json!(new.receipt_header),
        );
        note("address", json!(current.address), json!(new.address));
        note("legal_ids", current.legal_ids.clone(), new.legal_ids.clone());
        note("default_paper", json!(current.default_paper), json!(new.default_paper));
        note(
            "share_link_ttl_days",
            json!(current.share_link_ttl_days),
            json!(new.share_link_ttl_days),
        );
        note(
            "payment_confirm_threshold",
            json!(current.payment_confirm_threshold),
            json!(new.payment_confirm_threshold),
        );
        if !after.is_empty() {
            sqlx::query(
                "UPDATE org_settings SET receipt_prefix = $1, letterhead = $2::jsonb, legal_ids = $3::jsonb, default_paper = $4, \
                        share_link_ttl_days = $5, payment_confirm_threshold = $6, updated_at = $7::timestamptz, updated_by = $8::uuid, \
                        row_version = row_version + 1 WHERE org_id = current_org_id()",
            )
            .bind(&new.receipt_prefix)
            .bind(Value::Object(letterhead).to_string())
            .bind(new.legal_ids.to_string())
            .bind(&new.default_paper)
            .bind(new.share_link_ttl_days)
            .bind(new.payment_confirm_threshold)
            .bind(micros(now))
            .bind(auth.user.id.to_string())
            .execute(tx.conn())
            .await?;
            let entry = AuditEntry::new(AuditAction::SettingsUpdated, "settings", org.id)
                .before(Value::Object(before))
                .and_then(|e| e.after(Value::Object(after)))
                .map_err(internal)?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let r = sqlx::query(Self::SETTINGS_SQL).fetch_one(tx.conn()).await?;
        tx.commit().await?;
        Ok(Self::settings_row(&r))
    }

    // ───────── droits réglables ─────────

    /// `GET /role-settings` : les réglages applicables au plan, avec leur état et leur version (0 : aucune ligne).
    pub async fn role_settings(&self, auth: &Authenticated) -> Result<Vec<RoleSettingView>, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query("SELECT role, action, allowed, row_version FROM role_settings")
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        Ok(applicable_settings(org.plan)
            .iter()
            .map(|(role, action)| {
                let found = rows
                    .iter()
                    .find(|r| r.get::<String, _>(0) == role.as_str() && r.get::<String, _>(1) == *action);
                RoleSettingView {
                    role: *role,
                    action,
                    allowed: found.is_some_and(|r| r.get::<bool, _>(2)),
                    version: found.map_or(0, |r| r.get::<i32, _>(3)),
                }
            })
            .collect())
    }

    /// `PUT /role-settings` : `If-Match` porte la version (`0` crée la ligne). Effet à la requête suivante, même session.
    pub async fn put_role_setting(
        &self,
        auth: &Authenticated,
        role: &str,
        action: &str,
        allowed: bool,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<RoleSettingView, ServiceError> {
        let org = org_of(auth)?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let (role, action) = Role::parse(role)
            .and_then(|r| {
                applicable_settings(org.plan)
                    .iter()
                    .find(|(ar, aa)| *ar == r && *aa == action)
                    .copied()
            })
            .ok_or(ServiceError::Invalid("setting_not_available"))?;
        let now = self.clock.now();
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current =
            sqlx::query("SELECT allowed, row_version FROM role_settings WHERE role = $1 AND action = $2 FOR UPDATE")
                .bind(role.as_str())
                .bind(action)
                .fetch_optional(tx.conn())
                .await?;
        let (before_allowed, new_version) = match current {
            None => {
                if expected != 0 {
                    return Err(ServiceError::VersionMismatch);
                }
                let inserted = sqlx::query(
                    "INSERT INTO role_settings (org_id, role, action, allowed, set_by, set_at) \
                     VALUES (current_org_id(), $1, $2, $3, $4::uuid, $5::timestamptz)",
                )
                .bind(role.as_str())
                .bind(action)
                .bind(allowed)
                .bind(auth.user.id.to_string())
                .bind(micros(now))
                .execute(tx.conn())
                .await;
                match inserted {
                    Ok(_) => (false, 1),
                    Err(e) if sql_code(&e).as_deref() == Some("23505") => return Err(ServiceError::VersionMismatch),
                    Err(e) => return Err(e.into()),
                }
            }
            Some(r) => {
                if r.get::<i32, _>(1) != expected {
                    return Err(ServiceError::VersionMismatch);
                }
                let version = r.get::<i32, _>(1) + 1;
                sqlx::query(
                    "UPDATE role_settings SET allowed = $3, set_by = $4::uuid, set_at = $5::timestamptz, row_version = $6 \
                      WHERE role = $1 AND action = $2",
                )
                .bind(role.as_str())
                .bind(action)
                .bind(allowed)
                .bind(auth.user.id.to_string())
                .bind(micros(now))
                .bind(version)
                .execute(tx.conn())
                .await?;
                (r.get::<bool, _>(0), version)
            }
        };
        let entry = AuditEntry::new(
            AuditAction::SettingsUpdated,
            "role_settings",
            format!("{}.{action}", role.as_str()),
        )
        .before(json!({ "allowed": before_allowed }))
        .and_then(|e| e.after(json!({ "allowed": allowed })))
        .map_err(internal)?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        tx.commit().await?;
        Ok(RoleSettingView {
            role,
            action,
            allowed,
            version: new_version,
        })
    }
}

/// Date lisible d'un instant, pour les réponses (RFC 3339).
pub fn rfc3339(t: DateTime<chrono::Utc>) -> String {
    t.to_rfc3339()
}
