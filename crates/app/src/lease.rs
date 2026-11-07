//! Baux et échéancier (étape 06 ; F07, F08, DP-94). Les règles de calendrier et le statut d'une échéance viennent de
//! `immo_domain::schedule` ; ce module lit et écrit dans un `OrgTx`, avec l'audit dans la même transaction (R-008).

// Requêtes sur les tables de l'organisation active, dans son contexte (OrgTx) : même exception documentée de DP-77 que `catalog`.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

use immo_domain::schedule::{self, Amendment, InstallmentStatus, LeaseWindow};

use crate::audit::{self, AuditAction};
use crate::catalog::{Input, Reader, agent_of, audit_entry, opt_time, org_of, parse_id, unit_scope};
use crate::identity::error::{FieldIssue, ServiceError, sql_code};
use crate::identity::service::{Authenticated, ClientInfo, IdentityService, at, audit_context, internal, micros};
use crate::platform::clock::{Clock, OUAGADOUGOU};
use crate::platform::db::{Db, DbError, OrgTx};

/// Tâche quotidienne de génération des échéances (R-029).
pub const GENERATE_JOB: &str = "installments.generate";
/// Contrôle quotidien : alerte si la génération du jour n'est pas terminée à 02 h 00, heure locale (F08, DP-95).
pub const LATE_CHECK_JOB: &str = "installments.check_late";
/// Heure locale du contrôle.
pub const LATE_CHECK_HOUR: u32 = 2;
/// Fenêtre de génération (R-029).
pub const HORIZON_DAYS: u32 = 30;

// ───────── vues ─────────

#[derive(Debug, Clone)]
pub struct LeaseTenantView {
    pub id: Uuid,
    pub display_name: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct AmendmentView {
    pub id: Uuid,
    pub effective_from: NaiveDate,
    pub entered_on: NaiveDate,
    pub new_rent: Option<i64>,
    pub new_charges: Option<i64>,
    pub new_end_date: Option<NaiveDate>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LeaseView {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub unit_reference: String,
    pub property_name: String,
    pub kind: String,
    pub status: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub rent: i64,
    pub charges: i64,
    /// Loyer et charges en vigueur aujourd'hui (dernier avenant applicable, R-104).
    pub current_rent: i64,
    pub current_charges: i64,
    pub due_day: i16,
    pub due_day_defaulted: bool,
    pub deposit_amount: i64,
    pub vat_included: bool,
    pub duration_months: Option<i32>,
    pub indexation_note: Option<String>,
    pub terminated_on: Option<NaiveDate>,
    pub termination_reason: Option<String>,
    pub activated_at: Option<DateTime<Utc>>,
    pub tenants: Vec<LeaseTenantView>,
    pub amendments: Vec<AmendmentView>,
    pub version: i32,
    /// Avertissements non bloquants (`lease.deposit_above_two_months`, `lease.due_day_defaulted`).
    pub warnings: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct LineView {
    pub id: Uuid,
    pub kind: String,
    pub label: Option<String>,
    pub amount: i64,
    pub due_date: NaiveDate,
    pub voided: bool,
    pub void_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallmentView {
    pub id: Uuid,
    pub period: NaiveDate,
    pub due_date: NaiveDate,
    pub due_amount: i64,
    pub allocated_amount: i64,
    pub remaining_amount: i64,
    /// Calculé par `immo_domain::schedule::installment_status` avec la date du jour de l'organisation.
    pub status: InstallmentStatus,
    pub days_late: i64,
    pub lines: Vec<LineView>,
}

/// Filtres de `GET /leases`.
#[derive(Debug, Clone, Default)]
pub struct LeaseQuery {
    pub status: Option<String>,
    pub unit_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
}

// ───────── outils ─────────

fn date(text: &str) -> Result<NaiveDate, ServiceError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| ServiceError::from(internal("date illisible")))
}

fn opt_date(text: Option<String>) -> Result<Option<NaiveDate>, ServiceError> {
    text.map(|t| date(&t)).transpose()
}

fn fr_date(d: NaiveDate) -> String {
    d.format("%d/%m/%Y").to_string()
}

/// Une date `AAAA-MM-JJ` (règle des dates métier, `API_CONTRACTS.md` §1).
fn read_date(r: &mut Reader<'_>, field: &'static str) -> Option<Option<NaiveDate>> {
    match r.map.get(field) {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(s)) => match NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d") {
            Ok(d) => Some(Some(d)),
            Err(_) => {
                r.fail(field, "validation.invalid");
                None
            }
        },
        Some(_) => {
            r.fail(field, "validation.invalid");
            None
        }
    }
}

/// Montant entier FCFA borné (R-001) ; `min` vaut 1 pour un loyer (R-022), 0 pour des charges ou un dépôt.
fn read_amount(r: &mut Reader<'_>, field: &'static str, min: i64, below_code: &'static str) -> Option<Option<i64>> {
    match r.map.get(field) {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::Number(n)) => match n.as_i64() {
            Some(v) if v < min => {
                r.fail(field, below_code);
                None
            }
            Some(v) if v > immo_domain::MAX_ABS_FCFA => {
                r.fail(field, "validation.amount_out_of_range");
                None
            }
            Some(v) => Some(Some(v)),
            None => {
                r.fail(
                    field,
                    if n.as_u64().is_some() {
                        "validation.amount_out_of_range"
                    } else {
                        "validation.amount_not_integer"
                    },
                );
                None
            }
        },
        Some(_) => {
            r.fail(field, "validation.amount_not_integer");
            None
        }
    }
}

fn read_int(r: &mut Reader<'_>, field: &'static str, min: i64, max: i64) -> Option<Option<i64>> {
    match r.map.get(field) {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::Number(n)) => match n.as_i64() {
            Some(v) if (min..=max).contains(&v) => Some(Some(v)),
            _ => {
                r.fail(field, "validation.invalid");
                None
            }
        },
        Some(_) => {
            r.fail(field, "validation.invalid");
            None
        }
    }
}

/// Message d'une garde de base (`RAISE EXCEPTION 'code'`, SQLSTATE 23001) traduit en conflit du catalogue.
fn guard_conflict(e: &sqlx::Error) -> Option<ServiceError> {
    if sql_code(e).as_deref() == Some("23P01") {
        return Some(ServiceError::conflict("lease.overlap"));
    }
    if sql_code(e).as_deref() != Some("23001") {
        return None;
    }
    let message = e
        .as_database_error()
        .map(|d| d.message().to_string())
        .unwrap_or_default();
    let code: &'static str = match message.as_str() {
        "unit.under_works" => "unit.under_works",
        "unit.archived" => "unit.archived",
        "unit.has_active_lease" => "unit.has_active_lease",
        "lease.not_draft" => "lease.not_draft",
        "lease.not_in_progress" => "lease.not_in_progress",
        "lease.end_date_only_later" => "lease.end_date_only_later",
        "lease.amendment_before_start" => "lease.amendment_before_start",
        // gardes posées avec les paiements (étape 07, R-023, R-036)
        "lease.amendment_paid_installment" => "lease.amendment_paid_installment",
        "installment.line_has_payments" => "installment.line_has_payments",
        _ => return None,
    };
    Some(ServiceError::conflict(code))
}

fn sql_err(e: sqlx::Error) -> ServiceError {
    guard_conflict(&e).unwrap_or_else(|| e.into())
}

/// « Aujourd'hui » au fuseau de l'organisation (R-002, DP-50).
fn today(clock: &dyn Clock) -> NaiveDate {
    clock.local_date(OUAGADOUGOU)
}

// ───────── lectures ─────────

const LEASE_COLUMNS: &str = "l.id::text, l.unit_id::text, u.reference, p.name, l.kind, l.status, l.start_date::text, l.end_date::text, \
     l.rent, l.charges, l.due_day, l.due_day_defaulted, l.deposit_amount, l.vat_included, ct.duration_months, ct.indexation_note, \
     l.terminated_on::text, l.termination_reason, {activated}, l.row_version";

fn lease_columns() -> String {
    LEASE_COLUMNS.replace("{activated}", &at("l.activated_at"))
}

const LEASE_FROM: &str = "FROM leases l JOIN units u ON u.org_id = l.org_id AND u.id = l.unit_id \
     JOIN properties p ON p.org_id = u.org_id AND p.id = u.property_id \
     LEFT JOIN lease_commercial_terms ct ON ct.org_id = l.org_id AND ct.lease_id = l.id";

async fn amendments_of(tx: &mut OrgTx, lease_id: Uuid) -> Result<Vec<AmendmentView>, ServiceError> {
    let rows = sqlx::query(
        "SELECT id::text, effective_from::text, entered_on::text, new_rent, new_charges, new_end_date::text, note \
           FROM lease_amendments WHERE lease_id = $1::uuid ORDER BY effective_from, created_at",
    )
    .bind(lease_id.to_string())
    .fetch_all(tx.conn())
    .await?;
    rows.iter()
        .map(|r| {
            Ok(AmendmentView {
                id: parse_id(&r.get::<String, _>(0))?,
                effective_from: date(&r.get::<String, _>(1))?,
                entered_on: date(&r.get::<String, _>(2))?,
                new_rent: r.get(3),
                new_charges: r.get(4),
                new_end_date: opt_date(r.get(5))?,
                note: r.get(6),
            })
        })
        .collect()
}

fn domain_amendments(list: &[AmendmentView]) -> Vec<Amendment<Uuid>> {
    list.iter()
        .map(|a| Amendment {
            id: a.id,
            effective_from: a.effective_from,
            entered_on: a.entered_on,
            rent: a.new_rent,
            charges: a.new_charges,
        })
        .collect()
}

async fn lease_in(
    tx: &mut OrgTx,
    id: Uuid,
    agent: Option<&str>,
    today: NaiveDate,
) -> Result<Option<LeaseView>, ServiceError> {
    let sql = format!(
        "SELECT {} {LEASE_FROM} WHERE l.id = $1::uuid AND {}",
        lease_columns(),
        unit_scope(2)
    );
    let Some(r) = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id.to_string())
        .bind(agent)
        .fetch_optional(tx.conn())
        .await?
    else {
        return Ok(None);
    };
    let tenants = sqlx::query(
        "SELECT t.id::text, t.last_name, t.first_name, lt.is_primary FROM lease_tenants lt \
           JOIN tenants t ON t.org_id = lt.org_id AND t.id = lt.tenant_id WHERE lt.lease_id = $1::uuid \
          ORDER BY lt.is_primary DESC, t.search_key",
    )
    .bind(id.to_string())
    .fetch_all(tx.conn())
    .await?
    .iter()
    .map(|t| {
        let (last, first): (String, Option<String>) = (t.get(1), t.get(2));
        Ok(LeaseTenantView {
            id: parse_id(&t.get::<String, _>(0))?,
            display_name: crate::catalog::display_name(first.as_deref(), &last),
            is_primary: t.get(3),
        })
    })
    .collect::<Result<Vec<_>, ServiceError>>()?;
    let amendments = amendments_of(tx, id).await?;
    let (rent, charges): (i64, i64) = (r.get(8), r.get(9));
    let terms = schedule::terms_at(rent, charges, &domain_amendments(&amendments), today);
    let kind: String = r.get(4);
    let deposit: i64 = r.get(12);
    let defaulted: bool = r.get(11);
    let mut warnings = vec![];
    if schedule::deposit_above_two_months(kind == "residential", deposit, rent) {
        warnings.push("lease.deposit_above_two_months");
    }
    if defaulted {
        warnings.push("lease.due_day_defaulted");
    }
    Ok(Some(LeaseView {
        id: parse_id(&r.get::<String, _>(0))?,
        unit_id: parse_id(&r.get::<String, _>(1))?,
        unit_reference: r.get(2),
        property_name: r.get(3),
        kind,
        status: r.get(5),
        start_date: date(&r.get::<String, _>(6))?,
        end_date: opt_date(r.get(7))?,
        rent,
        charges,
        current_rent: terms.rent.0,
        current_charges: terms.charges.0,
        due_day: r.get(10),
        due_day_defaulted: defaulted,
        deposit_amount: deposit,
        vat_included: r.get(13),
        duration_months: r.get(14),
        indexation_note: r.get(15),
        terminated_on: opt_date(r.get(16))?,
        termination_reason: r.get(17),
        activated_at: opt_time(r.get(18))?,
        tenants,
        amendments,
        version: r.get(19),
        warnings,
    }))
}

// ───────── génération (R-027 à R-030, R-098 à R-100, R-104) ─────────

/// Crée les échéances manquantes d'un bail en cours, dans la transaction de l'appelant. Idempotent : une période existante n'est
/// jamais recréée (unicité `(lease_id, period)`, `ON CONFLICT DO NOTHING`, E-03). Renvoie le nombre d'échéances créées.
pub(crate) async fn generate_for_lease(
    tx: &mut OrgTx,
    lease_id: Uuid,
    today: NaiveDate,
    actor: Option<Uuid>,
) -> Result<usize, ServiceError> {
    let Some(r) = sqlx::query(
        "SELECT status, start_date::text, (activated_at AT TIME ZONE 'Africa/Ouagadougou')::date::text, effective_end::text, due_day, rent, charges \
           FROM leases WHERE id = $1::uuid",
    )
    .bind(lease_id.to_string())
    .fetch_optional(tx.conn())
    .await?
    else {
        return Ok(0);
    };
    let status: String = r.get(0);
    if status != "active" && status != "notice" {
        return Ok(0);
    }
    let Some(activated_on) = opt_date(r.get(2))? else {
        return Ok(0);
    };
    let due_day = u8::try_from(r.get::<i16, _>(4)).map_err(internal)?;
    let window = LeaseWindow {
        start_date: date(&r.get::<String, _>(1))?,
        activated_on,
        effective_end: opt_date(r.get(3))?,
        due_day,
    };
    let (rent, charges): (i64, i64) = (r.get(5), r.get(6));
    let amendments = domain_amendments(&amendments_of(tx, lease_id).await?);
    let mut created = 0_usize;
    for (period, due) in schedule::periods_to_create(&window, today, HORIZON_DAYS) {
        let new_id: Option<String> = sqlx::query_scalar(
            "INSERT INTO installments (org_id, lease_id, period, due_date, created_by) \
             VALUES (current_org_id(), $1::uuid, $2::date, $3::date, $4::uuid) \
             ON CONFLICT (org_id, lease_id, period) DO NOTHING RETURNING id::text",
        )
        .bind(lease_id.to_string())
        .bind(period.to_string())
        .bind(due.to_string())
        .bind(actor.map(|a| a.to_string()))
        .fetch_optional(tx.conn())
        .await?;
        let Some(installment) = new_id else { continue };
        let terms = schedule::terms_at(rent, charges, &amendments, due);
        insert_line(
            tx,
            &installment,
            lease_id,
            "rent",
            None,
            terms.rent.0,
            due,
            terms.rent.1,
            actor,
        )
        .await?;
        if terms.charges.0 > 0 {
            insert_line(
                tx,
                &installment,
                lease_id,
                "charges",
                None,
                terms.charges.0,
                due,
                terms.charges.1,
                actor,
            )
            .await?;
        }
        created = created.saturating_add(1);
    }
    // R-086, E-31 : le crédit du bail s'affecte aux échéances créées, dans la même transaction (DP-96)
    if created > 0 {
        crate::ledger::apply_lease_credit(tx, lease_id).await?;
    }
    Ok(created)
}

#[allow(clippy::too_many_arguments)]
async fn insert_line(
    tx: &mut OrgTx,
    installment: &str,
    lease_id: Uuid,
    kind: &str,
    label: Option<&str>,
    amount: i64,
    due: NaiveDate,
    amendment: Option<Uuid>,
    actor: Option<Uuid>,
) -> Result<Uuid, ServiceError> {
    let id: String = sqlx::query_scalar(
        "INSERT INTO installment_lines (org_id, lease_id, installment_id, kind, label, amount, due_date, amendment_id, created_by) \
         VALUES (current_org_id(), $1::uuid, $2::uuid, $3, $4, $5, $6::date, $7::uuid, $8::uuid) RETURNING id::text",
    )
    .bind(lease_id.to_string())
    .bind(installment)
    .bind(kind)
    .bind(label)
    .bind(amount)
    .bind(due.to_string())
    .bind(amendment.map(|a| a.to_string()))
    .bind(actor.map(|a| a.to_string()))
    .fetch_one(tx.conn())
    .await?;
    parse_id(&id)
}

async fn void_line(tx: &mut OrgTx, line: Uuid, reason: &str, actor: Uuid) -> Result<(), ServiceError> {
    sqlx::query("INSERT INTO installment_line_voids (org_id, line_id, reason, voided_by) VALUES (current_org_id(), $1::uuid, $2, $3::uuid)")
        .bind(line.to_string())
        .bind(reason)
        .bind(actor.to_string())
        .execute(tx.conn())
        .await
        // une ligne qui porte un paiement ne s'annule pas (garde de l'étape 07, R-036)
        .map_err(sql_err)?;
    Ok(())
}

/// Tâche quotidienne : pour chaque organisation qui a un bail en cours, crée les échéances des 30 jours à venir (R-029). Une
/// organisation en échec n'empêche pas les autres ; l'échec est journalisé, la tâche est réessayée par la file.
pub async fn generate_due_installments(db: &Db, clock: &dyn Clock) -> Result<usize, DbError> {
    let orgs: Vec<String> = {
        let mut tx = db.begin(Uuid::nil(), Uuid::nil()).await?;
        let ids = sqlx::query_scalar("SELECT id::text FROM orgs_with_leases_in_progress() AS id")
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        ids
    };
    let day = today(clock);
    let mut total = 0_usize;
    let mut failures = 0_usize;
    for org in orgs {
        let Ok(org_id) = Uuid::parse_str(&org) else { continue };
        let outcome = async {
            let mut tx = db.begin(org_id, Uuid::nil()).await?;
            let leases: Vec<String> =
                sqlx::query_scalar("SELECT id::text FROM leases WHERE status IN ('active', 'notice')")
                    .fetch_all(tx.conn())
                    .await?;
            let mut n = 0_usize;
            for l in leases {
                n = n.saturating_add(generate_for_lease(&mut tx, parse_id(&l)?, day, None).await?);
            }
            tx.commit().await?;
            Ok::<usize, ServiceError>(n)
        }
        .await;
        match outcome {
            Ok(n) => total = total.saturating_add(n),
            Err(_) => {
                failures = failures.saturating_add(1);
                tracing::error!(org_id = %org_id, "génération des échéances en échec pour une organisation");
            }
        }
    }
    if failures > 0 {
        return Err(internal(format!("{failures} organisation(s) en échec")));
    }
    Ok(total)
}

/// Contrôle de 02 h 00 : vrai (et alerte `installments.generation_late`) si la génération du jour local n'est pas terminée.
/// L'alerte va au journal et, si la connexion de la console est configurée, dans `platform_events`.
pub async fn check_generation_late(db: &Db, clock: &dyn Clock, platform_url: Option<&str>) -> Result<bool, String> {
    let done = crate::platform::jobs::daily_done(db, clock, GENERATE_JOB, None, OUAGADOUGOU)
        .await
        .map_err(|e| e.to_string())?;
    if done {
        return Ok(false);
    }
    let day = today(clock).to_string();
    tracing::error!(
        alert = "installments.generation_late",
        date = %day,
        "aucune génération des échéances terminée à 02 h 00"
    );
    if let Some(url) = platform_url {
        crate::platform::ops::record_event(url, "installments.generation_late", &json!({ "date": day }))
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(true)
}

// ───────── service ─────────

impl IdentityService {
    pub(crate) async fn lease_or_404(
        &self,
        auth: &Authenticated,
        tx: &mut OrgTx,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        match lease_in(tx, id, agent_of(auth).as_deref(), today(self.clock.as_ref())).await? {
            Some(l) => Ok(l),
            None => {
                if let Some(org) = auth.org.as_ref() {
                    self.trace_missing(org.id, auth.user.id, "lease", id, client).await?;
                }
                Err(ServiceError::NotFound)
            }
        }
    }

    /// `GET /leases` (l'agent terrain : baux de ses lots).
    pub async fn list_leases(&self, auth: &Authenticated, q: &LeaseQuery) -> Result<Vec<LeaseView>, ServiceError> {
        let org = org_of(auth)?;
        if q.status
            .as_deref()
            .is_some_and(|s| !["draft", "active", "notice", "ended"].contains(&s))
        {
            return Err(ServiceError::Invalid("filter_value"));
        }
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let sql = format!(
            "SELECT l.id::text {LEASE_FROM} WHERE {} AND ($2::text IS NULL OR l.status = $2) AND ($3::uuid IS NULL OR l.unit_id = $3::uuid) \
               AND ($4::uuid IS NULL OR EXISTS (SELECT 1 FROM lease_tenants lt WHERE lt.lease_id = l.id AND lt.tenant_id = $4::uuid)) \
             ORDER BY l.start_date DESC, l.id LIMIT 200",
            unit_scope(1)
        );
        let ids: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(agent_of(auth))
            .bind(q.status.as_deref())
            .bind(q.unit_id.map(|u| u.to_string()))
            .bind(q.tenant_id.map(|t| t.to_string()))
            .fetch_all(tx.conn())
            .await?;
        let day = today(self.clock.as_ref());
        let mut out = vec![];
        for id in ids {
            if let Some(l) = lease_in(&mut tx, parse_id(&id)?, None, day).await? {
                out.push(l);
            }
        }
        tx.commit().await?;
        Ok(out)
    }

    /// `GET /leases/{id}`.
    pub async fn lease(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let view = self.lease_or_404(auth, &mut tx, id, client).await;
        tx.commit().await?;
        view
    }

    /// `POST /leases` : bail brouillon et avertissements (R-021, R-022, R-024, R-031, DP-94).
    pub async fn create_lease(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let unit = r.uuid("unit_id").flatten();
        if unit.is_none() && !r.issues.iter().any(|i| i.field == "unit_id") {
            r.fail("unit_id", "validation.required");
        }
        let tenants: Vec<Uuid> = match input.get("tenant_ids") {
            Some(Value::Array(a)) if !a.is_empty() => {
                let parsed: Vec<Option<Uuid>> = a
                    .iter()
                    .map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
                    .collect();
                if parsed.iter().any(Option::is_none) {
                    r.fail("tenant_ids", "validation.invalid");
                }
                let mut out: Vec<Uuid> = vec![];
                for id in parsed.into_iter().flatten() {
                    if !out.contains(&id) {
                        out.push(id);
                    }
                }
                out
            }
            _ => {
                r.fail("tenant_ids", "validation.required");
                vec![]
            }
        };
        let primary = r.uuid("primary_tenant_id").flatten();
        let kind = match r.choice("kind", &["residential", "commercial"]) {
            Some(Some(k)) => Some(k),
            _ => {
                if !r.issues.iter().any(|i| i.field == "kind") {
                    r.fail("kind", "validation.required");
                }
                None
            }
        };
        let start = read_date(&mut r, "start_date").flatten();
        if start.is_none() && !r.issues.iter().any(|i| i.field == "start_date") {
            r.fail("start_date", "validation.required");
        }
        let end = read_date(&mut r, "end_date").flatten();
        if let (Some(s), Some(e)) = (start, end)
            && e < s
        {
            r.fail("end_date", "validation.date_order");
        }
        let rent = read_amount(&mut r, "rent", 1, "validation.rent_positive").flatten();
        if rent.is_none() && !r.issues.iter().any(|i| i.field == "rent") {
            r.fail("rent", "validation.required");
        }
        let charges = read_amount(&mut r, "charges", 0, "validation.amount_positive")
            .flatten()
            .unwrap_or(0);
        let deposit = read_amount(&mut r, "deposit_amount", 0, "validation.amount_positive")
            .flatten()
            .unwrap_or(0);
        let due_day = read_int(&mut r, "due_day", 1, 31).flatten();
        let vat = r.boolean("vat_included").unwrap_or(false);
        let commercial = r.object("commercial_terms").flatten();
        let (duration, note) = match &commercial {
            Some(m) => {
                let mut cr = Reader::new(m);
                let d = read_int(&mut cr, "duration_months", 1, 600).flatten();
                let n = cr.text("indexation_note", 1000).flatten();
                for i in cr.issues {
                    r.fail(
                        if i.field == "duration_months" {
                            "commercial_terms.duration_months"
                        } else {
                            "commercial_terms.indexation_note"
                        },
                        i.code,
                    );
                }
                (d, n)
            }
            None => (None, None),
        };
        if kind.as_deref() == Some("residential") {
            if vat {
                r.fail("vat_included", "validation.not_applicable");
            }
            if commercial.is_some() {
                r.fail("commercial_terms", "validation.not_applicable");
            }
        }
        r.finish()?;
        let (Some(unit), Some(kind), Some(start), Some(rent)) = (unit, kind, start, rent) else {
            return Err(ServiceError::Invalid("lease"));
        };
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(u) = sqlx::query("SELECT usage, archived_at IS NOT NULL FROM units WHERE id = $1::uuid")
            .bind(unit.to_string())
            .fetch_optional(tx.conn())
            .await?
        else {
            drop(tx);
            if let Err(e) = self.trace_missing(org.id, auth.user.id, "unit", unit, client).await {
                return Err(e.into());
            }
            return Err(ServiceError::NotFound);
        };
        let usage: String = u.get(0);
        if u.get::<bool, _>(1) {
            return Err(ServiceError::conflict("unit.archived"));
        }
        // DP-94 : le type du bail suit l'usage du lot ; un lot mixte accepte les deux
        if usage != "mixed" && usage != kind {
            return Err(ServiceError::conflict("lease.kind_not_allowed"));
        }
        let ok: i64 =
            sqlx::query_scalar("SELECT count(*) FROM tenants WHERE id = ANY($1::uuid[]) AND archived_at IS NULL")
                .bind(tenants.iter().map(Uuid::to_string).collect::<Vec<_>>())
                .fetch_one(tx.conn())
                .await?;
        if usize::try_from(ok).ok() != Some(tenants.len()) {
            return Err(ServiceError::Fields(vec![FieldIssue {
                field: "tenant_ids",
                code: "validation.invalid",
            }]));
        }
        let primary = primary
            .filter(|p| tenants.contains(p))
            .or_else(|| tenants.first().copied());
        let lid: String = sqlx::query_scalar(
            "INSERT INTO leases (org_id, unit_id, kind, start_date, end_date, rent, charges, due_day, due_day_defaulted, deposit_amount, vat_included, created_by) \
             VALUES (current_org_id(), $1::uuid, $2, $3::date, $4::date, $5, $6, $7, $8, $9, $10, $11::uuid) RETURNING id::text",
        )
        .bind(unit.to_string())
        .bind(&kind)
        .bind(start.to_string())
        .bind(end.map(|e| e.to_string()))
        .bind(rent)
        .bind(charges)
        .bind(i16::try_from(due_day.unwrap_or(1)).map_err(internal)?)
        .bind(due_day.is_none())
        .bind(deposit)
        .bind(vat)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await
        .map_err(sql_err)?;
        let lid = parse_id(&lid)?;
        for t in &tenants {
            sqlx::query("INSERT INTO lease_tenants (org_id, lease_id, tenant_id, is_primary) VALUES (current_org_id(), $1::uuid, $2::uuid, $3)")
                .bind(lid.to_string())
                .bind(t.to_string())
                .bind(Some(*t) == primary)
                .execute(tx.conn())
                .await?;
        }
        if kind == "commercial" && (duration.is_some() || note.is_some()) {
            sqlx::query("INSERT INTO lease_commercial_terms (org_id, lease_id, duration_months, indexation_note) VALUES (current_org_id(), $1::uuid, $2, $3)")
                .bind(lid.to_string())
                .bind(duration.map(|d| i32::try_from(d).unwrap_or(1)))
                .bind(&note)
                .execute(tx.conn())
                .await?;
        }
        let entry = audit_entry(
            AuditAction::LeaseCreated,
            "lease",
            lid,
            None,
            json!({ "unit_id": unit.to_string(), "rent": rent, "kind": kind }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = lease_in(&mut tx, lid, None, today(self.clock.as_ref()))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `PATCH /leases/{id}` : seulement en brouillon (`If-Match`).
    pub async fn patch_lease(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let start = read_date(&mut r, "start_date");
        let end = read_date(&mut r, "end_date");
        let rent = read_amount(&mut r, "rent", 1, "validation.rent_positive");
        let charges = read_amount(&mut r, "charges", 0, "validation.amount_positive");
        let deposit = read_amount(&mut r, "deposit_amount", 0, "validation.amount_positive");
        let due_day = read_int(&mut r, "due_day", 1, 31);
        r.finish()?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current = self.lease_or_404(auth, &mut tx, id, client).await?;
        if current.status != "draft" {
            return Err(ServiceError::conflict("lease.not_draft"));
        }
        if current.version != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let new_start = start.flatten().unwrap_or(current.start_date);
        let new_end = match end {
            None => current.end_date,
            Some(v) => v,
        };
        if new_end.is_some_and(|e| e < new_start) {
            return Err(ServiceError::Fields(vec![FieldIssue {
                field: "end_date",
                code: "validation.date_order",
            }]));
        }
        let new_rent = rent.flatten().unwrap_or(current.rent);
        let new_charges = charges.flatten().unwrap_or(current.charges);
        let new_deposit = deposit.flatten().unwrap_or(current.deposit_amount);
        let (new_due, defaulted) = match due_day.flatten() {
            Some(d) => (i16::try_from(d).map_err(internal)?, false),
            None => (current.due_day, current.due_day_defaulted),
        };
        sqlx::query(
            "UPDATE leases SET start_date = $2::date, end_date = $3::date, rent = $4, charges = $5, deposit_amount = $6, due_day = $7, \
                    due_day_defaulted = $8, row_version = row_version + 1 WHERE id = $1::uuid",
        )
        .bind(id.to_string())
        .bind(new_start.to_string())
        .bind(new_end.map(|e| e.to_string()))
        .bind(new_rent)
        .bind(new_charges)
        .bind(new_deposit)
        .bind(new_due)
        .bind(defaulted)
        .execute(tx.conn())
        .await
        .map_err(sql_err)?;
        let entry = audit_entry(
            AuditAction::LeaseUpdated,
            "lease",
            id,
            Some(json!({ "rent": current.rent, "charges": current.charges, "deposit_amount": current.deposit_amount })),
            json!({ "rent": new_rent, "charges": new_charges, "deposit_amount": new_deposit }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = lease_in(&mut tx, id, None, today(self.clock.as_ref()))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /leases/{id}/activate` (R-020, R-028, R-093, E-36). `replay` : la requête porte une clé d'idempotence ; un bail déjà
    /// actif est alors renvoyé tel quel au lieu de `lease.not_draft`.
    pub async fn activate_lease(
        &self,
        auth: &Authenticated,
        id: Uuid,
        replay: bool,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current = self.lease_or_404(auth, &mut tx, id, client).await?;
        // verrou du lot : deux activations simultanées sur un même lot se succèdent
        let unit: (bool, bool) = {
            let row =
                sqlx::query("SELECT under_works, archived_at IS NOT NULL FROM units WHERE id = $1::uuid FOR UPDATE")
                    .bind(current.unit_id.to_string())
                    .fetch_one(tx.conn())
                    .await?;
            (row.get(0), row.get(1))
        };
        if current.status != "draft" {
            if replay && current.status == "active" {
                tx.commit().await?;
                return Ok(current);
            }
            return Err(ServiceError::conflict("lease.not_draft"));
        }
        if unit.0 {
            return Err(ServiceError::conflict("unit.under_works"));
        }
        if unit.1 {
            return Err(ServiceError::conflict("unit.archived"));
        }
        let clash = sqlx::query(
            "SELECT start_date::text, effective_end::text FROM leases \
              WHERE unit_id = $1::uuid AND id <> $2::uuid AND status IN ('active', 'notice', 'ended') \
                AND daterange(start_date, effective_end, '[]') && daterange($3::date, $4::date, '[]') LIMIT 1",
        )
        .bind(current.unit_id.to_string())
        .bind(id.to_string())
        .bind(current.start_date.to_string())
        .bind(current.end_date.map(|e| e.to_string()))
        .fetch_optional(tx.conn())
        .await?;
        if let Some(c) = clash {
            let (from, to): (String, Option<String>) = (c.get(0), c.get(1));
            return Err(ServiceError::Conflict {
                code: "lease.overlap",
                params: Some(json!({ "start_date": from, "end_date": to })),
            });
        }
        sqlx::query(
            "UPDATE leases SET status = 'active', activated_at = $2::timestamptz, activated_by = $3::uuid, row_version = row_version + 1 WHERE id = $1::uuid",
        )
        .bind(id.to_string())
        .bind(micros(self.clock.now()))
        .bind(auth.user.id.to_string())
        .execute(tx.conn())
        .await
        .map_err(sql_err)?;
        let created = generate_for_lease(&mut tx, id, day, Some(auth.user.id)).await?;
        let entry = audit_entry(
            AuditAction::LeaseActivated,
            "lease",
            id,
            None,
            json!({ "installments_created": created }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = lease_in(&mut tx, id, None, day).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /leases/{id}/amendments` (R-023, R-091, R-104, E-02, E-35, E-44, DP-94).
    pub async fn amend_lease(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let effective = read_date(&mut r, "effective_from").flatten();
        if effective.is_none() && !r.issues.iter().any(|i| i.field == "effective_from") {
            r.fail("effective_from", "validation.required");
        }
        let rent = read_amount(&mut r, "rent", 1, "validation.rent_positive").flatten();
        let charges = read_amount(&mut r, "charges", 0, "validation.amount_positive").flatten();
        let new_end = read_date(&mut r, "new_end_date").flatten();
        let note = r.text("note", 500).flatten();
        if rent.is_none() && charges.is_none() && new_end.is_none() && r.issues.is_empty() {
            r.fail("rent", "validation.required");
        }
        r.finish()?;
        let Some(effective) = effective else {
            return Err(ServiceError::Invalid("effective_from"));
        };
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current = self.lease_or_404(auth, &mut tx, id, client).await?;
        if current.status != "active" && current.status != "notice" {
            return Err(ServiceError::conflict("lease.not_in_progress"));
        }
        if let Some(e) = new_end
            && current.end_date.is_none_or(|old| e <= old)
        {
            return Err(ServiceError::Fields(vec![FieldIssue {
                field: "new_end_date",
                code: "validation.date_order",
            }]));
        }
        let aid: String = sqlx::query_scalar(
            "INSERT INTO lease_amendments (org_id, lease_id, effective_from, new_rent, new_charges, new_end_date, note, entered_on, created_by) \
             VALUES (current_org_id(), $1::uuid, $2::date, $3, $4, $5::date, $6, $7::date, $8::uuid) RETURNING id::text",
        )
        .bind(id.to_string())
        .bind(effective.to_string())
        .bind(rent)
        .bind(charges)
        .bind(new_end.map(|e| e.to_string()))
        .bind(&note)
        .bind(day.to_string())
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await
        .map_err(|e| {
            if crate::catalog::is_unique_violation(&e, "lease_amendments_org_id_lease_id_effective_from_key") {
                ServiceError::conflict("lease.amendment_same_date")
            } else {
                sql_err(e)
            }
        })?;
        let aid = parse_id(&aid)?;
        if let Some(e) = new_end {
            sqlx::query("UPDATE leases SET end_date = $2::date, row_version = row_version + 1 WHERE id = $1::uuid")
                .bind(id.to_string())
                .bind(e.to_string())
                .execute(tx.conn())
                .await
                .map_err(sql_err)?;
        }
        // DP-94 : une échéance déjà générée, non échue, à laquelle l'avenant s'applique : la ligne est annulée avec un motif et
        // remplacée par une ligne au nouveau montant ; rien n'est modifié en place.
        let all = domain_amendments(&amendments_of(&mut tx, id).await?);
        let reason = format!("avenant du {}", fr_date(day));
        let pending = sqlx::query(
            "SELECT i.id::text, i.due_date::text, l.line_id::text, l.kind, l.amount FROM installments i \
               JOIN v_installment_lines l ON l.org_id = i.org_id AND l.installment_id = i.id \
              WHERE i.lease_id = $1::uuid AND i.due_date > $2::date AND NOT l.voided AND l.kind IN ('rent', 'charges')",
        )
        .bind(id.to_string())
        .bind(day.to_string())
        .fetch_all(tx.conn())
        .await?;
        let mut seen_charges: Vec<String> = vec![];
        for p in &pending {
            let (inst, due, line, kind, amount): (String, String, String, String, i64) =
                (p.get(0), p.get(1), p.get(2), p.get(3), p.get(4));
            let due = date(&due)?;
            let terms = schedule::terms_at(current.rent, current.charges, &all, due);
            let (wanted, source) = if kind == "rent" { terms.rent } else { terms.charges };
            if kind == "charges" {
                seen_charges.push(inst.clone());
            }
            if wanted != amount {
                void_line(&mut tx, parse_id(&line)?, &reason, auth.user.id).await?;
                if wanted > 0 {
                    insert_line(&mut tx, &inst, id, &kind, None, wanted, due, source, Some(auth.user.id)).await?;
                }
            }
        }
        // charges nouvellement non nulles sur une échéance qui n'en avait pas
        if charges.is_some_and(|c| c > 0) {
            let without = sqlx::query(
                "SELECT id::text, due_date::text FROM installments WHERE lease_id = $1::uuid AND due_date > $2::date",
            )
            .bind(id.to_string())
            .bind(day.to_string())
            .fetch_all(tx.conn())
            .await?;
            for w in &without {
                let (inst, due): (String, String) = (w.get(0), w.get(1));
                if seen_charges.contains(&inst) {
                    continue;
                }
                let due = date(&due)?;
                let terms = schedule::terms_at(current.rent, current.charges, &all, due);
                if terms.charges.0 > 0 {
                    insert_line(
                        &mut tx,
                        &inst,
                        id,
                        "charges",
                        None,
                        terms.charges.0,
                        due,
                        terms.charges.1,
                        Some(auth.user.id),
                    )
                    .await?;
                }
            }
        }
        generate_for_lease(&mut tx, id, day, Some(auth.user.id)).await?;
        let entry = audit_entry(
            AuditAction::LeaseAmended,
            "lease",
            id,
            None,
            json!({ "amendment_id": aid.to_string(), "effective_from": effective.to_string(), "rent": rent, "charges": charges,
                    "new_end_date": new_end.map(|e| e.to_string()) }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = lease_in(&mut tx, id, None, day).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /leases/{id}/terminate` (R-030) : plus aucune échéance après le mois de la résiliation ; celles déjà générées au-delà
    /// sont annulées avec un motif.
    pub async fn terminate_lease(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<LeaseView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let on = read_date(&mut r, "terminated_on").flatten();
        if on.is_none() && !r.issues.iter().any(|i| i.field == "terminated_on") {
            r.fail("terminated_on", "validation.required");
        }
        let reason = r.required("reason", 3, 500, true);
        r.finish()?;
        let (Some(on), Some(reason)) = (on, reason) else {
            return Err(ServiceError::Invalid("terminate"));
        };
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let current = self.lease_or_404(auth, &mut tx, id, client).await?;
        if current.status != "active" && current.status != "notice" {
            return Err(ServiceError::conflict("lease.not_in_progress"));
        }
        if on < current.start_date {
            return Err(ServiceError::Fields(vec![FieldIssue {
                field: "terminated_on",
                code: "validation.date_order",
            }]));
        }
        sqlx::query(
            "UPDATE leases SET status = 'ended', terminated_on = $2::date, termination_reason = $3, departure_date = NULL, notice_given_on = NULL, \
                    row_version = row_version + 1 WHERE id = $1::uuid",
        )
        .bind(id.to_string())
        .bind(on.to_string())
        .bind(&reason)
        .execute(tx.conn())
        .await
        .map_err(sql_err)?;
        let first_after = schedule::first_of_month(on)
            .and_then(|m| m.checked_add_months(chrono::Months::new(1)))
            .ok_or_else(|| ServiceError::from(internal("date hors bornes")))?;
        let lines: Vec<String> = sqlx::query_scalar(
            "SELECT l.line_id::text FROM v_installment_lines l JOIN installments i ON i.org_id = l.org_id AND i.id = l.installment_id \
              WHERE i.lease_id = $1::uuid AND i.period >= $2::date AND NOT l.voided",
        )
        .bind(id.to_string())
        .bind(first_after.to_string())
        .fetch_all(tx.conn())
        .await?;
        let why = format!("résiliation du {}", fr_date(on));
        for l in lines {
            void_line(&mut tx, parse_id(&l)?, &why, auth.user.id).await?;
        }
        let entry = audit_entry(
            AuditAction::LeaseEnded,
            "lease",
            id,
            None,
            json!({ "terminated_on": on.to_string() }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = lease_in(&mut tx, id, None, day).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `GET /leases/{id}/installments` : montants de la base, statut du domaine (une seule fonction, ARCHITECTURE §4.9).
    pub async fn lease_installments(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<Vec<InstallmentView>, ServiceError> {
        let org = org_of(auth)?;
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        self.lease_or_404(auth, &mut tx, id, client).await?;
        let rows = sqlx::query(
            "SELECT installment_id::text, period::text, due_date::text, due_amount, allocated_amount, remaining_amount \
               FROM v_installment_balances WHERE lease_id = $1::uuid ORDER BY period",
        )
        .bind(id.to_string())
        .fetch_all(tx.conn())
        .await?;
        let lines = sqlx::query(
            "SELECT installment_id::text, line_id::text, kind, label, amount, due_date::text, voided, void_reason \
               FROM v_installment_lines WHERE lease_id = $1::uuid ORDER BY due_date, kind",
        )
        .bind(id.to_string())
        .fetch_all(tx.conn())
        .await?;
        tx.commit().await?;
        let mut out = vec![];
        for r in &rows {
            let inst: String = r.get(0);
            let due_date = date(&r.get::<String, _>(2))?;
            let (due, allocated): (i64, i64) = (r.get(3), r.get(4));
            let mut ls = vec![];
            for l in lines.iter().filter(|l| l.get::<String, _>(0) == inst) {
                ls.push(LineView {
                    id: parse_id(&l.get::<String, _>(1))?,
                    kind: l.get(2),
                    label: l.get(3),
                    amount: l.get(4),
                    due_date: date(&l.get::<String, _>(5))?,
                    voided: l.get(6),
                    void_reason: l.get(7),
                });
            }
            out.push(InstallmentView {
                id: parse_id(&inst)?,
                period: date(&r.get::<String, _>(1))?,
                due_date,
                due_amount: due,
                allocated_amount: allocated,
                remaining_amount: r.get(5),
                status: schedule::installment_status(due, allocated, due_date, day),
                days_late: if due > allocated {
                    schedule::days_late(due_date, day)
                } else {
                    0
                },
                lines: ls,
            });
        }
        Ok(out)
    }

    /// `POST /leases/{id}/installment-lines` : arriéré ou ligne exceptionnelle (R-035, R-036).
    pub async fn add_installment_line(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<LineView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let kind = match r.choice("kind", &["arrears", "exceptional"]) {
            Some(Some(k)) => Some(k),
            _ => {
                if !r.issues.iter().any(|i| i.field == "kind") {
                    r.fail("kind", "validation.required");
                }
                None
            }
        };
        let label = r.required("label", 2, 200, true);
        let amount = read_amount(&mut r, "amount", 1, "validation.amount_positive").flatten();
        if amount.is_none() && !r.issues.iter().any(|i| i.field == "amount") {
            r.fail("amount", "validation.required");
        }
        let due = read_date(&mut r, "due_date").flatten();
        if due.is_none() && !r.issues.iter().any(|i| i.field == "due_date") {
            r.fail("due_date", "validation.required");
        }
        r.finish()?;
        let (Some(kind), Some(label), Some(amount), Some(due)) = (kind, label, amount, due) else {
            return Err(ServiceError::Invalid("line"));
        };
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let lease = self.lease_or_404(auth, &mut tx, id, client).await?;
        if lease.status == "draft" {
            return Err(ServiceError::conflict("lease.not_in_progress"));
        }
        let period = schedule::first_of_month(due).ok_or_else(|| ServiceError::from(internal("date")))?;
        // l'échéance de la période : créée si besoin (arriéré d'une période antérieure à l'activation, R-100)
        let inst_due = schedule::due_date_for(period, u8::try_from(lease.due_day).map_err(internal)?)
            .ok_or_else(|| ServiceError::from(internal("jour")))?;
        sqlx::query(
            "INSERT INTO installments (org_id, lease_id, period, due_date, created_by) VALUES (current_org_id(), $1::uuid, $2::date, $3::date, $4::uuid) \
             ON CONFLICT (org_id, lease_id, period) DO NOTHING",
        )
        .bind(id.to_string())
        .bind(period.to_string())
        .bind(inst_due.to_string())
        .bind(auth.user.id.to_string())
        .execute(tx.conn())
        .await?;
        let inst: String =
            sqlx::query_scalar("SELECT id::text FROM installments WHERE lease_id = $1::uuid AND period = $2::date")
                .bind(id.to_string())
                .bind(period.to_string())
                .fetch_one(tx.conn())
                .await?;
        let line = insert_line(
            &mut tx,
            &inst,
            id,
            &kind,
            Some(&label),
            amount,
            due,
            None,
            Some(auth.user.id),
        )
        .await?;
        let entry = audit_entry(
            AuditAction::InstallmentLineAdded,
            "installment_line",
            line,
            None,
            json!({ "lease_id": id.to_string(), "kind": kind, "amount": amount }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        tx.commit().await?;
        Ok(LineView {
            id: line,
            kind,
            label: Some(label),
            amount,
            due_date: due,
            voided: false,
            void_reason: None,
        })
    }

    /// `POST /installment-lines/{id}/void` (R-036 : motif obligatoire ; le refus d'une ligne payée arrive à l'étape 07).
    pub async fn void_installment_line(
        &self,
        auth: &Authenticated,
        line: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<LineView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let reason = r.required("reason", 3, 500, true);
        r.finish()?;
        let Some(reason) = reason else {
            return Err(ServiceError::Invalid("reason"));
        };
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let sql = format!(
            "SELECT l.kind, l.label, l.amount, l.due_date::text, l.voided FROM v_installment_lines l JOIN leases ls ON ls.org_id = l.org_id AND ls.id = l.lease_id \
               JOIN units u ON u.org_id = ls.org_id AND u.id = ls.unit_id WHERE l.line_id = $1::uuid AND {}",
            unit_scope(2)
        );
        let Some(row) = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(line.to_string())
            .bind(agent_of(auth))
            .fetch_optional(tx.conn())
            .await?
        else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "installment_line", line, client)
                .await?;
            return Err(ServiceError::NotFound);
        };
        if !row.get::<bool, _>(4) {
            void_line(&mut tx, line, &reason, auth.user.id).await?;
            let entry = audit_entry(
                AuditAction::InstallmentLineVoided,
                "installment_line",
                line,
                None,
                json!({ "reason_length": reason.chars().count() }),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        tx.commit().await?;
        Ok(LineView {
            id: line,
            kind: row.get(0),
            label: row.get(1),
            amount: row.get(2),
            due_date: date(&row.get::<String, _>(3))?,
            voided: true,
            void_reason: Some(reason),
        })
    }

    /// `POST /units/{id}/works` (R-093, DP-25) : « en travaux » sur un lot vacant seulement ; retrait libre.
    pub async fn set_unit_works(
        &self,
        auth: &Authenticated,
        unit: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<crate::catalog::UnitView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let works = r.boolean("under_works");
        if works.is_none() && !r.issues.iter().any(|i| i.field == "under_works") {
            r.fail("under_works", "validation.required");
        }
        r.finish()?;
        let works = works.unwrap_or(false);
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) =
            sqlx::query("SELECT under_works, archived_at IS NOT NULL FROM units WHERE id = $1::uuid FOR UPDATE")
                .bind(unit.to_string())
                .fetch_optional(tx.conn())
                .await?
        else {
            drop(tx);
            self.trace_missing(org.id, auth.user.id, "unit", unit, client).await?;
            return Err(ServiceError::NotFound);
        };
        if cur.get::<bool, _>(1) {
            return Err(ServiceError::conflict("unit.archived"));
        }
        if works != cur.get::<bool, _>(0) {
            if works {
                let busy: bool = sqlx::query_scalar(
                    "SELECT EXISTS (SELECT 1 FROM leases WHERE unit_id = $1::uuid AND status IN ('active', 'notice'))",
                )
                .bind(unit.to_string())
                .fetch_one(tx.conn())
                .await?;
                if busy {
                    return Err(ServiceError::conflict("unit.works_requires_vacant"));
                }
            }
            sqlx::query("UPDATE units SET under_works = $2, works_since = $3::date, row_version = row_version + 1 WHERE id = $1::uuid")
                .bind(unit.to_string())
                .bind(works)
                .bind(works.then(|| day.to_string()))
                .execute(tx.conn())
                .await
                .map_err(sql_err)?;
            let entry = audit_entry(
                AuditAction::UnitWorksSet,
                "unit",
                unit,
                None,
                json!({ "under_works": works }),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        tx.commit().await?;
        self.unit(auth, unit, client).await
    }
}
