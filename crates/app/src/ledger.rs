//! Encaisser (étape 07 ; F09, F10, DP-96) : paiement, affectations, crédit du bail et quittance numérotée, en **une** transaction.
//!
//! Ordre (F09 règle 18, `API_CONTRACTS.md` §5.3) : clé d'idempotence (verrou), contrôles métier, verrou consultatif du bail,
//! relecture des lignes ouvertes, paiement, affectations, quittance (compteur **en dernier**, instantané), audit, réponse enregistrée
//! avec la clé. Aucun appel au stockage ni au moteur PDF (pas de tâche de rendu à cette étape, DP-96).
//!
//! Seul ce module écrit `payments`, `allocations`, `receipt_counters`, `receipts` et `idempotency_keys` (F10 règle 12).

// Requêtes sur les tables de l'organisation active, dans son contexte (OrgTx) : même exception documentée de DP-77 que `catalog`.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{Datelike, NaiveDate};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use immo_domain::allocation::{self, AllocationError, LineKind, OpenLine, Part, Split};
use immo_domain::{receipt as numbering, schedule, words};

use crate::audit::{self, AuditAction};
use crate::catalog::{Input, Reader, audit_entry, org_of, parse_id};
use crate::identity::error::{FieldIssue, ServiceError, sql_code};
use crate::identity::service::{Authenticated, ClientInfo, IdentityService, audit_context, internal};
use crate::lease::LeaseView;
use crate::observation::events::{self, ProductEvent};
use crate::platform::clock::{Clock, OUAGADOUGOU};
use crate::platform::db::{Db, DbError, OrgTx};

/// Route enregistrée avec la clé d'idempotence (une même clé sur une autre route est refusée, A11).
pub const PAYMENTS_ROUTE: &str = "POST /api/payments";
/// Tâche quotidienne du contrôle des trous de numérotation (F10 règle 8, DP-96).
pub const GAPS_JOB: &str = "receipts.check_gaps";
/// Heure locale du contrôle des trous.
pub const GAPS_HOUR: u32 = 3;
/// Conservation des clés d'idempotence (jours, `API_CONTRACTS.md` §5.1).
pub const IDEMPOTENCY_DAYS: i32 = 30;
/// Champs acceptés par `POST /payments` (le serveur refuse les autres avant l'appel).
pub const PAYMENT_FIELDS: &[&str] = &[
    "lease_id",
    "amount",
    "paid_on",
    "method",
    "external_ref",
    "payer",
    "note",
    "allocations",
    "surplus",
    "confirm_high_amount",
    "acknowledge_external_ref_duplicate",
    "replaces_payment_id",
];

const METHODS: &[&str] = &["cash", "mobile_money", "bank_transfer", "cheque"];

/// Réponse d'un enregistrement : corps JSON du contrat (§6.1) et indicateur de rejeu (`Idempotent-Replay: true`).
#[derive(Debug, Clone)]
pub struct Recorded {
    pub body: Value,
    pub replay: bool,
}

/// Filtres de `GET /payments`.
#[derive(Debug, Clone, Default)]
pub struct PaymentQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub method: Option<String>,
    pub lease_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
    pub unit_id: Option<Uuid>,
    pub q: Option<String>,
}

// ───────── outils ─────────

fn today(clock: &dyn Clock) -> NaiveDate {
    clock.local_date(OUAGADOUGOU)
}

fn date(text: &str) -> Result<NaiveDate, ServiceError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| ServiceError::from(internal("date illisible")))
}

fn unprocessable(code: &'static str, params: Value) -> ServiceError {
    ServiceError::Unprocessable {
        code,
        params: Some(params),
    }
}

fn field(field: &'static str, code: &'static str) -> ServiceError {
    ServiceError::Fields(vec![FieldIssue { field, code }])
}

/// Référence externe normalisée pour la comparaison (DP-96 : sans casse ni espaces).
pub fn normalized_ref(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Empreinte du corps (clés triées par `serde_json::Map`) et de la route.
fn request_hash(route: &str, input: &Input) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(route.as_bytes());
    h.update([0]);
    h.update(Value::Object(input.clone()).to_string().as_bytes());
    h.finalize().to_vec()
}

/// Verrou consultatif de transaction (même fonction de hachage que la base).
async fn advisory(tx: &mut OrgTx, key: &str, seed: i64) -> Result<(), ServiceError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, $2))")
        .bind(key)
        .bind(seed)
        .execute(tx.conn())
        .await?;
    Ok(())
}

/// Garde de la base traduite en erreur du contrat (`RAISE EXCEPTION 'code'`, SQLSTATE 23001).
fn guard_error(e: sqlx::Error) -> ServiceError {
    if sql_code(&e).as_deref() == Some("23001") {
        let message = e
            .as_database_error()
            .map(|d| d.message().to_string())
            .unwrap_or_default();
        match message.as_str() {
            "payment.lease_not_in_progress" => return ServiceError::conflict("payment.lease_not_in_progress"),
            "payment.allocation_invalid" => return unprocessable("payment.allocation_invalid", json!({})),
            "payer.not_on_lease" => return field("payer", "payer.not_on_lease"),
            "validation.date_in_future" => return field("paid_on", "validation.date_in_future"),
            _ => {}
        }
    }
    e.into()
}

fn fcfa(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let len = digits.len();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && len.saturating_sub(i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    format!("{out} FCFA")
}

// ───────── lignes ouvertes ─────────

#[derive(Debug, Clone)]
struct OpenRow {
    line: Uuid,
    installment: Uuid,
    period: NaiveDate,
    due_date: NaiveDate,
    kind: LineKind,
    label: Option<String>,
    remaining: i64,
}

async fn open_rows(tx: &mut OrgTx, lease_id: Uuid) -> Result<Vec<OpenRow>, ServiceError> {
    let rows = sqlx::query(
        "SELECT l.line_id::text, l.installment_id::text, i.period::text, l.due_date::text, l.kind, l.label, \
                (l.due_amount - l.allocated_amount)::bigint \
           FROM v_installment_lines l JOIN installments i ON i.org_id = l.org_id AND i.id = l.installment_id \
          WHERE l.lease_id = $1::uuid AND NOT l.voided AND l.due_amount - l.allocated_amount > 0",
    )
    .bind(lease_id.to_string())
    .fetch_all(tx.conn())
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(OpenRow {
            line: parse_id(&r.get::<String, _>(0))?,
            installment: parse_id(&r.get::<String, _>(1))?,
            period: date(&r.get::<String, _>(2))?,
            due_date: date(&r.get::<String, _>(3))?,
            kind: LineKind::from_code(&r.get::<String, _>(4)).ok_or_else(|| ServiceError::from(internal("nature")))?,
            label: r.get(5),
            remaining: r.get(6),
        });
    }
    Ok(out)
}

fn as_open(rows: &[OpenRow]) -> Vec<OpenLine<Uuid>> {
    rows.iter()
        .map(|r| OpenLine {
            id: r.line,
            due_date: r.due_date,
            kind: r.kind,
            remaining: r.remaining,
        })
        .collect()
}

fn line_status(due_date: NaiveDate, day: NaiveDate) -> &'static str {
    if due_date < day {
        "late"
    } else if due_date == day {
        "due"
    } else {
        "upcoming"
    }
}

fn line_json(r: &OpenRow, day: NaiveDate) -> Value {
    json!({
        "installment_id": r.installment.to_string(),
        "installment_line_id": r.line.to_string(),
        "period": r.period.to_string(),
        "due_date": r.due_date.to_string(),
        "kind": r.kind.code(),
        "label": r.label,
        "remaining_amount": r.remaining,
        "status": line_status(r.due_date, day),
    })
}

/// Lignes ouvertes dans l'ordre d'affectation (R-087), avec les informations de l'écran.
fn ordered_rows(rows: &[OpenRow]) -> Vec<OpenRow> {
    allocation::ordered(&as_open(rows))
        .into_iter()
        .filter_map(|o| rows.iter().find(|r| r.line == o.id).cloned())
        .collect()
}

/// Échéances touchées, avec leur statut calculé par le domaine (F08 règle 11).
async fn installments_json(tx: &mut OrgTx, ids: &[Uuid], day: NaiveDate) -> Result<Vec<Value>, ServiceError> {
    let ids: Vec<String> = ids.iter().map(Uuid::to_string).collect();
    let rows = sqlx::query(
        "SELECT installment_id::text, period::text, due_date::text, due_amount, allocated_amount, remaining_amount \
           FROM v_installment_balances WHERE installment_id::text = ANY($1) ORDER BY period",
    )
    .bind(&ids)
    .fetch_all(tx.conn())
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let due_date = date(&r.get::<String, _>(2))?;
        let (due, allocated): (i64, i64) = (r.get(3), r.get(4));
        out.push(json!({
            "id": r.get::<String, _>(0),
            "period": r.get::<String, _>(1),
            "due_date": due_date.to_string(),
            "status": schedule::installment_status(due, allocated, due_date, day).as_str(),
            "due_amount": due,
            "allocated_amount": allocated,
            "remaining_amount": r.get::<i64, _>(5),
        }));
    }
    Ok(out)
}

/// Crédit du bail (vue `v_lease_credit`, R-086).
pub async fn lease_credit(tx: &mut OrgTx, lease_id: Uuid) -> Result<i64, ServiceError> {
    let credit: Option<i64> = sqlx::query_scalar("SELECT credit_amount FROM v_lease_credit WHERE lease_id = $1::uuid")
        .bind(lease_id.to_string())
        .fetch_optional(tx.conn())
        .await?;
    Ok(credit.unwrap_or(0))
}

async fn confirm_threshold(tx: &mut OrgTx) -> Result<i64, ServiceError> {
    Ok(sqlx::query_scalar("SELECT payment_confirm_threshold FROM org_settings")
        .fetch_one(tx.conn())
        .await?)
}

// ───────── crédit du bail appliqué à la génération (R-086, E-31, DP-96) ─────────

/// Affecte le crédit du bail (part non affectée des paiements, la plus ancienne d'abord) aux lignes ouvertes. Appelée après chaque
/// création d'échéances, dans la même transaction ; affectations « système » (`created_by` nul). Renvoie le montant affecté.
pub async fn apply_lease_credit(tx: &mut OrgTx, lease_id: Uuid) -> Result<i64, ServiceError> {
    let credits = sqlx::query(
        "SELECT payment_id::text, credit_amount FROM v_payment_credit \
          WHERE lease_id = $1::uuid AND kind = 'payment' AND credit_amount > 0 ORDER BY recorded_at, payment_id",
    )
    .bind(lease_id.to_string())
    .fetch_all(tx.conn())
    .await?;
    if credits.is_empty() {
        return Ok(0);
    }
    advisory(tx, &lease_id.to_string(), 0).await?;
    let mut pool = Vec::with_capacity(credits.len());
    for r in credits {
        pool.push((parse_id(&r.get::<String, _>(0))?, r.get::<i64, _>(1)));
    }
    let rows = open_rows(tx, lease_id).await?;
    let mut total = 0_i64;
    for (payment, part) in allocation::apply_credit(&pool, &as_open(&rows)) {
        insert_allocation(tx, lease_id, payment, part, None).await?;
        total = total.saturating_add(part.amount);
    }
    Ok(total)
}

async fn insert_allocation(
    tx: &mut OrgTx,
    lease_id: Uuid,
    payment: Uuid,
    part: Part<Uuid>,
    actor: Option<Uuid>,
) -> Result<(), ServiceError> {
    sqlx::query(
        "INSERT INTO allocations (org_id, lease_id, payment_id, payment_kind, installment_line_id, amount, created_by) \
         VALUES (current_org_id(), $1::uuid, $2::uuid, 'payment', $3::uuid, $4, $5::uuid)",
    )
    .bind(lease_id.to_string())
    .bind(payment.to_string())
    .bind(part.line.to_string())
    .bind(part.amount)
    .bind(actor.map(|a| a.to_string()))
    .execute(tx.conn())
    .await
    .map_err(guard_error)?;
    Ok(())
}

// ───────── lecture du corps ─────────

#[derive(Debug, Clone)]
enum Payer {
    Tenant(Uuid),
    ThirdParty { name: String, phone: Option<String> },
}

#[derive(Debug, Clone)]
struct Command {
    lease_id: Uuid,
    amount: i64,
    paid_on: NaiveDate,
    method: String,
    external_ref: Option<String>,
    payer: Option<Payer>,
    note: Option<String>,
    allocations: Option<Vec<Part<Uuid>>>,
    surplus_advance: bool,
    confirm_high_amount: bool,
    acknowledge_duplicate: bool,
}

/// Montant entier strict (R-001, PP-17) : `75000.5`, `"75000"`, `1e5` refusés.
fn read_amount(r: &mut Reader<'_>, name: &'static str) -> Option<i64> {
    match r.map.get(name) {
        Some(Value::Number(n)) => match n.as_i64() {
            Some(v) if (1..=immo_domain::MAX_ABS_FCFA).contains(&v) => Some(v),
            Some(_) => {
                r.fail(name, "validation.amount_out_of_range");
                None
            }
            None => {
                let code = if n.as_u64().is_some() {
                    "validation.amount_out_of_range"
                } else {
                    "validation.amount_not_integer"
                };
                r.fail(name, code);
                None
            }
        },
        None | Some(Value::Null) => {
            r.fail(name, "validation.required");
            None
        }
        Some(_) => {
            r.fail(name, "validation.amount_not_integer");
            None
        }
    }
}

fn read_date(r: &mut Reader<'_>, name: &'static str) -> Option<NaiveDate> {
    match r.map.get(name) {
        Some(Value::String(s)) => match NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                r.fail(name, "validation.invalid");
                None
            }
        },
        None | Some(Value::Null) => {
            r.fail(name, "validation.required");
            None
        }
        Some(_) => {
            r.fail(name, "validation.invalid");
            None
        }
    }
}

fn flag(r: &mut Reader<'_>, name: &'static str) -> bool {
    match r.map.get(name) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            r.fail(name, "validation.invalid");
            false
        }
    }
}

fn read_parts(r: &mut Reader<'_>) -> Option<Vec<Part<Uuid>>> {
    let items = match r.map.get("allocations") {
        None | Some(Value::Null) => return None,
        Some(Value::Array(items)) if !items.is_empty() && items.len() <= 200 => items.clone(),
        Some(_) => {
            r.fail("allocations", "validation.invalid");
            return None;
        }
    };
    let mut out = Vec::with_capacity(items.len());
    for item in &items {
        let line = item
            .get("installment_line_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok());
        let amount = item.get("amount").and_then(Value::as_i64);
        let keys_ok = item
            .as_object()
            .is_some_and(|o| o.keys().all(|k| k == "installment_line_id" || k == "amount"));
        match (line, amount, keys_ok) {
            (Some(line), Some(amount), true) => out.push(Part { line, amount }),
            _ => {
                r.fail("allocations", "validation.invalid");
                return None;
            }
        }
    }
    Some(out)
}

fn read_payer(r: &mut Reader<'_>) -> Option<Payer> {
    let obj = match r.map.get("payer") {
        None | Some(Value::Null) => return None,
        Some(Value::Object(o)) => o.clone(),
        Some(_) => {
            r.fail("payer", "validation.invalid");
            return None;
        }
    };
    let allowed = ["kind", "tenant_id", "third_party_name", "third_party_phone"];
    if obj.keys().any(|k| !allowed.contains(&k.as_str())) {
        r.fail("payer", "validation.invalid");
        return None;
    }
    match obj.get("kind").and_then(Value::as_str) {
        Some("tenant") => {
            let id = obj
                .get("tenant_id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            if id.is_none() {
                r.fail("payer", "validation.invalid");
            }
            id.map(Payer::Tenant)
        }
        Some("third_party") => {
            let name = obj
                .get("third_party_name")
                .and_then(Value::as_str)
                .map_or("", str::trim);
            if name.is_empty() || name.chars().count() > 200 {
                r.fail("payer", "payer.third_party_name_required");
                return None;
            }
            let phone = match obj.get("third_party_phone") {
                None | Some(Value::Null) => None,
                Some(Value::String(s)) if s.trim().is_empty() => None,
                Some(Value::String(s)) => {
                    let Some(p) = immo_domain::phone::normalize(s) else {
                        r.fail("payer", "validation.phone_invalid");
                        return None;
                    };
                    Some(p)
                }
                Some(_) => {
                    r.fail("payer", "validation.invalid");
                    return None;
                }
            };
            Some(Payer::ThirdParty {
                name: name.to_string(),
                phone,
            })
        }
        _ => {
            r.fail("payer", "validation.invalid");
            None
        }
    }
}

fn read_command(input: &Input, day: NaiveDate) -> Result<Command, ServiceError> {
    let mut r = Reader::new(input);
    let lease_id = match r.uuid("lease_id") {
        Some(Some(id)) => Some(id),
        _ => {
            r.fail("lease_id", "validation.required");
            None
        }
    };
    let amount = read_amount(&mut r, "amount");
    let paid_on = read_date(&mut r, "paid_on");
    if paid_on.is_some_and(|d| d > day) {
        r.fail("paid_on", "validation.date_in_future");
    }
    let method = match r.choice("method", METHODS) {
        Some(Some(m)) => m,
        _ => "cash".to_string(),
    };
    let external_ref = r.text("external_ref", 100).flatten();
    let payer = read_payer(&mut r);
    let note = r.text("note", 500).flatten();
    let allocations = read_parts(&mut r);
    let surplus_advance = match r.map.get("surplus") {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) if s == "advance" => true,
        Some(_) => {
            r.fail("surplus", "validation.invalid");
            false
        }
    };
    let confirm_high_amount = flag(&mut r, "confirm_high_amount");
    let acknowledge_duplicate = flag(&mut r, "acknowledge_external_ref_duplicate");
    // le paiement de remplacement d'une contre-passation viendra avec l'étape 09
    if r.map.get("replaces_payment_id").is_some_and(|v| !v.is_null()) {
        r.fail("replaces_payment_id", "validation.not_available");
    }
    r.finish()?;
    let (Some(lease_id), Some(amount), Some(paid_on)) = (lease_id, amount, paid_on) else {
        return Err(ServiceError::Invalid("payment"));
    };
    Ok(Command {
        lease_id,
        amount,
        paid_on,
        method,
        external_ref,
        payer,
        note,
        allocations,
        surplus_advance,
        confirm_high_amount,
        acknowledge_duplicate,
    })
}

fn split_error(e: AllocationError) -> ServiceError {
    match e {
        AllocationError::Amount => field("amount", "validation.amount_out_of_range"),
        _ => unprocessable("payment.allocation_invalid", json!({})),
    }
}

fn split_of(
    amount: i64,
    parts: Option<&[Part<Uuid>]>,
    rows: &[OpenRow],
    day: NaiveDate,
) -> Result<Split<Uuid>, ServiceError> {
    let open = as_open(rows);
    match parts {
        None => allocation::allocate(amount, &open, day).map_err(split_error),
        Some(p) => allocation::check_manual(amount, &open, p, day).map_err(split_error),
    }
}

fn parts_json(split: &Split<Uuid>, rows: &[OpenRow]) -> Vec<Value> {
    split
        .parts
        .iter()
        .map(|p| {
            let row = rows.iter().find(|r| r.line == p.line);
            json!({
                "installment_line_id": p.line.to_string(),
                "installment_id": row.map(|r| r.installment.to_string()),
                "period": row.map(|r| r.period.to_string()),
                "due_date": row.map(|r| r.due_date.to_string()),
                "kind": row.map(|r| r.kind.code()),
                "label": row.and_then(|r| r.label.clone()),
                "amount": p.amount,
            })
        })
        .collect()
}

fn preview_json(split: &Split<Uuid>, rows: &[OpenRow]) -> Value {
    json!({
        "allocations": parts_json(split, rows),
        "surplus_amount": split.advance,
        "credit_amount": split.credit,
        "remaining_after": split.remaining_due_after,
        "requires_surplus_confirmation": split.requires_surplus_confirmation(),
    })
}

// ───────── service ─────────

impl IdentityService {
    /// `GET /leases/{id}/payment-context` : tout ce qu'il faut à S04 en un aller-retour (§6.1).
    pub async fn payment_context(
        &self,
        auth: &Authenticated,
        lease_id: Uuid,
        client: &ClientInfo,
    ) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        let day = today(self.clock.as_ref());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let lease = self.lease_or_404(auth, &mut tx, lease_id, client).await?;
        let rows = ordered_rows(&open_rows(&mut tx, lease_id).await?);
        let credit = lease_credit(&mut tx, lease_id).await?;
        let threshold = confirm_threshold(&mut tx).await?;
        let phones = tenant_phones(&mut tx, &lease).await?;
        tx.commit().await?;
        let mut remaining = 0_i64;
        let mut overdue = 0_i64;
        for r in rows.iter().filter(|r| r.due_date <= day) {
            remaining = remaining.saturating_add(r.remaining);
            if r.due_date < day {
                overdue = overdue.saturating_add(r.remaining);
            }
        }
        // montant proposé : reste de l'échéance non soldée la plus ancienne (§6.1)
        let suggested = rows.first().map_or(0, |first| {
            rows.iter()
                .filter(|r| r.installment == first.installment)
                .map(|r| r.remaining)
                .fold(0_i64, i64::saturating_add)
        });
        let tenants: Vec<Value> = lease
            .tenants
            .iter()
            .map(|t| {
                json!({
                    "id": t.id.to_string(), "display_name": t.display_name, "is_primary": t.is_primary,
                    "phone": phones.iter().find(|(id, _)| *id == t.id).and_then(|(_, p)| p.clone()),
                })
            })
            .collect();
        Ok(json!({
            "lease": {
                "id": lease.id.to_string(), "status": lease.status, "kind": lease.kind, "due_day": lease.due_day,
                "unit": { "id": lease.unit_id.to_string(), "label": lease.unit_reference, "property_name": lease.property_name },
            },
            "tenants": tenants,
            "balance": { "remaining_amount": remaining, "overdue_amount": overdue, "credit_amount": credit },
            "open_lines": rows.iter().map(|r| line_json(r, day)).collect::<Vec<_>>(),
            "suggested_amount": suggested,
            "default_method": "cash",
            "today": day.to_string(),
            "confirm_threshold": threshold,
        }))
    }

    /// `POST /payments/preview` : répartition calculée par le serveur, sans écriture ni clé (§7.6).
    pub async fn preview_payment(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        let day = today(self.clock.as_ref());
        let mut r = Reader::new(input);
        let lease_id = match r.uuid("lease_id") {
            Some(Some(id)) => Some(id),
            _ => {
                r.fail("lease_id", "validation.required");
                None
            }
        };
        let amount = read_amount(&mut r, "amount");
        let parts = read_parts(&mut r);
        r.finish()?;
        let (Some(lease_id), Some(amount)) = (lease_id, amount) else {
            return Err(ServiceError::Invalid("preview"));
        };
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        self.lease_or_404(auth, &mut tx, lease_id, client).await?;
        let rows = open_rows(&mut tx, lease_id).await?;
        let threshold = confirm_threshold(&mut tx).await?;
        tx.commit().await?;
        let split = split_of(amount, parts.as_deref(), &rows, day)?;
        let mut v = preview_json(&split, &rows);
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "requires_high_amount_confirmation".into(),
                Value::Bool(amount > threshold),
            );
        }
        Ok(v)
    }

    /// `POST /payments` (F09, F10) : une transaction ; `key` est l'en-tête `Idempotency-Key`.
    pub async fn record_payment(
        &self,
        auth: &Authenticated,
        key: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<Recorded, ServiceError> {
        let org = org_of(auth)?;
        let day = today(self.clock.as_ref());
        let hash = request_hash(PAYMENTS_ROUTE, input);
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        // R-042 : deux requêtes de même clé se suivent ; la seconde lit la réponse de la première
        advisory(&mut tx, &format!("idem:{}:{key}", org.id), 2).await?;
        let stored =
            sqlx::query("SELECT route, request_hash, response_body::text FROM idempotency_keys WHERE key = $1::uuid")
                .bind(key.to_string())
                .fetch_optional(tx.conn())
                .await?;
        if let Some(row) = stored {
            let (route, stored_hash): (String, Vec<u8>) = (row.get(0), row.get(1));
            if route != PAYMENTS_ROUTE || stored_hash != hash {
                return Err(unprocessable("idempotency.key_reused_with_different_body", json!({})));
            }
            let body: Value = serde_json::from_str(&row.get::<String, _>(2)).map_err(internal)?;
            if let Some(event) = ProductEvent::payment_idempotent_replay() {
                let ctx = self.event_context(auth, "", None);
                events::emit(&mut tx, &ctx, &event).await?;
            }
            tx.commit().await?;
            return Ok(Recorded { body, replay: true });
        }
        let cmd = read_command(input, day)?;
        let lease = self.lease_or_404(auth, &mut tx, cmd.lease_id, client).await?;
        if lease.status != "active" && lease.status != "notice" {
            return Err(ServiceError::conflict("payment.lease_not_in_progress"));
        }
        let threshold = confirm_threshold(&mut tx).await?;
        if cmd.amount > threshold && !cmd.confirm_high_amount {
            return Err(unprocessable(
                "payment.confirmation_required",
                json!({ "threshold": threshold }),
            ));
        }
        // payé par : un locataire du bail (le principal par défaut) ou un tiers (R-088)
        let payer = match cmd.payer.clone() {
            Some(Payer::Tenant(id)) => {
                if !lease.tenants.iter().any(|t| t.id == id) {
                    return Err(field("payer", "payer.not_on_lease"));
                }
                Payer::Tenant(id)
            }
            Some(p) => p,
            None => Payer::Tenant(
                lease
                    .tenants
                    .iter()
                    .find(|t| t.is_primary)
                    .or_else(|| lease.tenants.first())
                    .map(|t| t.id)
                    .ok_or_else(|| ServiceError::from(internal("bail sans locataire")))?,
            ),
        };
        // verrou du bail avant la relecture des soldes (n° 37, DP-96) ; même clé que chk_allocation
        advisory(&mut tx, &cmd.lease_id.to_string(), 0).await?;
        if let Some(reference) = cmd.external_ref.as_deref() {
            let norm = normalized_ref(reference);
            advisory(&mut tx, &format!("ref:{}:{}:{norm}", org.id, cmd.method), 1).await?;
            let existing = sqlx::query(
                "SELECT p.id::text, r.number FROM payments p \
                   LEFT JOIN receipts r ON r.org_id = p.org_id AND r.payment_id = p.id \
                  WHERE p.kind = 'payment' AND p.method = $1 AND p.external_ref IS NOT NULL \
                    AND lower(regexp_replace(p.external_ref, '\\s', '', 'g')) = $2 \
                  ORDER BY p.recorded_at LIMIT 1",
            )
            .bind(&cmd.method)
            .bind(&norm)
            .fetch_optional(tx.conn())
            .await?;
            if let Some(row) = existing
                && !cmd.acknowledge_duplicate
            {
                if let Some(event) = ProductEvent::payment_external_ref_duplicate(&cmd.method) {
                    let ctx = self.event_context(auth, "", None);
                    events::emit(&mut tx, &ctx, &event).await?;
                    tx.commit().await?;
                }
                return Err(ServiceError::Conflict {
                    code: "payment.external_ref_duplicate",
                    params: Some(json!({
                        "conflict": { "kind": "payment", "id": row.get::<String, _>(0) },
                        "receipt_number": row.get::<Option<String>, _>(1),
                    })),
                });
            }
        }
        let rows = open_rows(&mut tx, cmd.lease_id).await?;
        if let Some(parts) = cmd.allocations.as_deref() {
            // une ligne du bail qui n'est plus ouverte : l'aperçu est périmé (§6.1)
            for p in parts {
                if rows.iter().any(|r| r.line == p.line) {
                    continue;
                }
                let of_lease: bool = sqlx::query_scalar(
                    "SELECT EXISTS (SELECT 1 FROM installment_lines WHERE id = $1::uuid AND lease_id = $2::uuid)",
                )
                .bind(p.line.to_string())
                .bind(cmd.lease_id.to_string())
                .fetch_one(tx.conn())
                .await?;
                if of_lease {
                    let fresh = split_of(cmd.amount, None, &rows, day)?;
                    return Err(ServiceError::Conflict {
                        code: "payment.stale_allocation",
                        params: Some(json!({ "preview": preview_json(&fresh, &rows) })),
                    });
                }
            }
        }
        let split = split_of(cmd.amount, cmd.allocations.as_deref(), &rows, day)?;
        if split.requires_surplus_confirmation() && !cmd.surplus_advance {
            return Err(unprocessable(
                "payment.surplus_not_confirmed",
                json!({ "surplus_amount": split.advance }),
            ));
        }
        let adjusted = cmd.allocations.is_some()
            && split_of(cmd.amount, None, &rows, day).map_or(true, |d| d.parts != split.parts);

        // paiement
        let (payer_kind, payer_tenant, payer_name, payer_phone) = match &payer {
            Payer::Tenant(id) => ("tenant", Some(id.to_string()), None, None),
            Payer::ThirdParty { name, phone } => ("third_party", None, Some(name.clone()), phone.clone()),
        };
        let payment_row = sqlx::query(
            "INSERT INTO payments (org_id, lease_id, kind, amount, method, external_ref, paid_on, cash_day, created_by, \
                                   payer_kind, payer_tenant_id, payer_third_party_name, payer_third_party_phone, note) \
             VALUES (current_org_id(), $1::uuid, 'payment', $2, $3, $4, $5::date, $5::date, $6::uuid, $7, $8::uuid, $9, $10, $11) \
             RETURNING id::text, cash_day::text, to_char(recorded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')",
        )
        .bind(cmd.lease_id.to_string())
        .bind(cmd.amount)
        .bind(&cmd.method)
        .bind(&cmd.external_ref)
        .bind(cmd.paid_on.to_string())
        .bind(auth.user.id.to_string())
        .bind(payer_kind)
        .bind(&payer_tenant)
        .bind(&payer_name)
        .bind(&payer_phone)
        .bind(&cmd.note)
        .fetch_one(tx.conn())
        .await
        .map_err(guard_error)?;
        let payment_id = parse_id(&payment_row.get::<String, _>(0))?;
        let cash_day: String = payment_row.get(1);
        let recorded_at: String = payment_row.get(2);
        for p in &split.parts {
            insert_allocation(&mut tx, cmd.lease_id, payment_id, *p, Some(auth.user.id)).await?;
        }

        // quittance : compteur en dernier (DP-46)
        let receiver: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1::uuid")
            .bind(auth.user.id.to_string())
            .fetch_one(tx.conn())
            .await?;
        let receipt = issue_receipt(
            &mut tx,
            &ReceiptInput {
                payment_id,
                lease: &lease,
                cmd: &cmd,
                payer: &payer,
                split: &split,
                rows: &rows,
                issued_on: day,
                issued_by: auth.user.id,
                issued_by_name: &receiver,
                clock: self.clock.as_ref(),
            },
        )
        .await?;

        // audit (R-008) et événement du serveur
        let actx = audit_context(auth.user.id, client);
        let entry = audit_entry(
            AuditAction::PaymentRecorded,
            "payment",
            payment_id,
            None,
            json!({ "lease_id": cmd.lease_id.to_string(), "amount": cmd.amount, "method": cmd.method,
                    "paid_on": cmd.paid_on.to_string(), "advance": split.advance, "credit": split.credit }),
        )?;
        audit::record(&mut tx, &actx, entry).await?;
        let entry = audit_entry(
            AuditAction::ReceiptIssued,
            "receipt",
            receipt.id,
            None,
            json!({ "number": receipt.number, "payment_id": payment_id.to_string() }),
        )?;
        audit::record(&mut tx, &actx, entry).await?;
        if adjusted {
            let parts: Vec<Value> = split
                .parts
                .iter()
                .map(|p| json!({ "installment_line_id": p.line.to_string(), "amount": p.amount }))
                .collect();
            let entry = audit_entry(
                AuditAction::AllocationAdjusted,
                "payment",
                payment_id,
                None,
                json!({ "allocations": parts }),
            )?;
            audit::record(&mut tx, &actx, entry).await?;
        }
        let late_days = (day - cmd.paid_on).num_days().clamp(0, 3650);
        if let Some(event) = ProductEvent::payment_recorded(
            &cmd.method,
            split.remaining_due_after > 0,
            split.advance > 0,
            split.credit > 0,
            late_days,
        ) {
            let ctx = self.event_context(auth, "", None);
            events::emit(&mut tx, &ctx, &event).await?;
        }

        // réponse (§6.1), enregistrée avec la clé dans la même transaction (DP-49)
        let mut touched: Vec<Uuid> = Vec::new();
        for p in &split.parts {
            if let Some(r) = rows.iter().find(|r| r.line == p.line)
                && !touched.contains(&r.installment)
            {
                touched.push(r.installment);
            }
        }
        let installments = installments_json(&mut tx, &touched, day).await?;
        let body = json!({
            "payment": {
                "id": payment_id.to_string(), "kind": "payment", "status": "validated",
                "lease_id": cmd.lease_id.to_string(), "amount": cmd.amount, "method": cmd.method,
                "external_ref": cmd.external_ref, "paid_on": cmd.paid_on.to_string(),
                "recorded_at": recorded_at, "cash_day": cash_day,
                "received_by": { "id": auth.user.id.to_string(), "display_name": receiver },
                "payer": payer_json(&payer, &lease), "note": cmd.note,
                "allocations": parts_json(&split, &rows),
                "unallocated_amount": split.credit,
            },
            "receipt": receipt.json,
            "installments": installments,
            "warnings": [],
        });
        sqlx::query(
            "INSERT INTO idempotency_keys (org_id, key, user_id, route, request_hash, response_status, response_body) \
             VALUES (current_org_id(), $1::uuid, $2::uuid, $3, $4, 201, $5::jsonb)",
        )
        .bind(key.to_string())
        .bind(auth.user.id.to_string())
        .bind(PAYMENTS_ROUTE)
        .bind(&hash)
        .bind(body.to_string())
        .execute(tx.conn())
        .await?;
        tx.commit().await.map_err(|e| match e {
            DbError::Query(q) => guard_error(q),
            other => other.into(),
        })?;
        Ok(Recorded { body, replay: false })
    }

    /// `GET /payments` : journal (S11), 100 au plus, du plus récent au plus ancien.
    pub async fn list_payments(&self, auth: &Authenticated, q: &PaymentQuery) -> Result<Vec<Value>, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let sql = format!(
            "{PAYMENT_SELECT} WHERE p.kind = 'payment' \
               AND ($1::date IS NULL OR p.paid_on >= $1::date) AND ($2::date IS NULL OR p.paid_on <= $2::date) \
               AND ($3::text IS NULL OR p.method = $3) AND ($4::uuid IS NULL OR p.lease_id = $4::uuid) \
               AND ($5::uuid IS NULL OR l.unit_id = $5::uuid) \
               AND ($6::uuid IS NULL OR EXISTS (SELECT 1 FROM lease_tenants lt WHERE lt.lease_id = p.lease_id AND lt.tenant_id = $6::uuid)) \
               AND ($7::text IS NULL OR r.number ILIKE '%' || $7 || '%' OR p.external_ref ILIKE '%' || $7 || '%') \
             ORDER BY p.recorded_at DESC, p.id LIMIT 100"
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(q.from.map(|d| d.to_string()))
            .bind(q.to.map(|d| d.to_string()))
            .bind(&q.method)
            .bind(q.lease_id.map(|d| d.to_string()))
            .bind(q.unit_id.map(|d| d.to_string()))
            .bind(q.tenant_id.map(|d| d.to_string()))
            .bind(q.q.as_ref().map(|s| crate::catalog::like_escape(s.trim())))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        Ok(rows.iter().map(payment_summary).collect())
    }

    /// `GET /payments/{id}` : paiement, affectations et quittance.
    pub async fn payment(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = sqlx::query(sqlx::AssertSqlSafe(format!("{PAYMENT_SELECT} WHERE p.id = $1::uuid")))
            .bind(id.to_string())
            .fetch_optional(tx.conn())
            .await?;
        let Some(row) = found else {
            tx.commit().await?;
            self.trace_missing(org.id, auth.user.id, "payment", id, client).await?;
            return Err(ServiceError::NotFound);
        };
        let allocs = sqlx::query(
            "SELECT a.installment_line_id::text, il.installment_id::text, i.period::text, il.due_date::text, il.kind, il.label, \
                    a.amount, (a.created_by IS NULL) \
               FROM allocations a JOIN installment_lines il ON il.org_id = a.org_id AND il.id = a.installment_line_id \
               JOIN installments i ON i.org_id = il.org_id AND i.id = il.installment_id \
              WHERE a.payment_id = $1::uuid ORDER BY il.due_date, il.kind, a.allocated_at",
        )
        .bind(id.to_string())
        .fetch_all(tx.conn())
        .await?;
        let receipt = receipt_by(&mut tx, true, id).await?;
        tx.commit().await?;
        let allocations: Vec<Value> = allocs
            .iter()
            .map(|a| {
                json!({
                    "installment_line_id": a.get::<String, _>(0), "installment_id": a.get::<String, _>(1),
                    "period": a.get::<String, _>(2), "due_date": a.get::<String, _>(3), "kind": a.get::<String, _>(4),
                    "label": a.get::<Option<String>, _>(5), "amount": a.get::<i64, _>(6),
                    "from_credit": a.get::<bool, _>(7),
                })
            })
            .collect();
        let mut v = payment_summary(&row);
        if let Some(obj) = v.as_object_mut() {
            obj.insert("allocations".into(), Value::Array(allocations));
            obj.insert("receipt".into(), receipt.unwrap_or(Value::Null));
        }
        Ok(v)
    }

    /// `GET /receipts` : quittances, 100 au plus.
    pub async fn list_receipts(
        &self,
        auth: &Authenticated,
        q: Option<&str>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<Value>, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let sql = format!(
            "{RECEIPT_SELECT} WHERE ($1::text IS NULL OR r.number ILIKE '%' || $1 || '%') \
               AND ($2::date IS NULL OR r.issued_on >= $2::date) AND ($3::date IS NULL OR r.issued_on <= $3::date) \
             ORDER BY r.issued_at DESC, r.id LIMIT 100"
        );
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(q.map(|s| crate::catalog::like_escape(s.trim())))
            .bind(from.map(|d| d.to_string()))
            .bind(to.map(|d| d.to_string()))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        rows.iter().map(receipt_json).collect()
    }

    /// `GET /receipts/{id}`.
    pub async fn receipt(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<Value, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = receipt_by(&mut tx, false, id).await?;
        tx.commit().await?;
        match found {
            Some(v) => Ok(v),
            None => {
                self.trace_missing(org.id, auth.user.id, "receipt", id, client).await?;
                Err(ServiceError::NotFound)
            }
        }
    }
}

async fn tenant_phones(tx: &mut OrgTx, lease: &LeaseView) -> Result<Vec<(Uuid, Option<String>)>, ServiceError> {
    let ids: Vec<String> = lease.tenants.iter().map(|t| t.id.to_string()).collect();
    let rows = sqlx::query("SELECT id::text, phone FROM tenants WHERE id::text = ANY($1)")
        .bind(&ids)
        .fetch_all(tx.conn())
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push((parse_id(&r.get::<String, _>(0))?, r.get(1)));
    }
    Ok(out)
}

fn payer_json(payer: &Payer, lease: &LeaseView) -> Value {
    match payer {
        Payer::Tenant(id) => json!({
            "kind": "tenant", "tenant_id": id.to_string(),
            "display_name": lease.tenants.iter().find(|t| t.id == *id).map(|t| t.display_name.clone()),
        }),
        Payer::ThirdParty { name, phone } => json!({
            "kind": "third_party", "third_party_name": name, "third_party_phone": phone,
        }),
    }
}

const PAYMENT_SELECT: &str = "SELECT p.id::text, p.lease_id::text, p.amount, p.method, p.external_ref, p.paid_on::text, \
        p.cash_day::text, to_char(p.recorded_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), p.created_by::text, uc.display_name, p.payer_kind, p.payer_tenant_id::text, \
        p.payer_third_party_name, p.note, r.id::text, r.number, u.reference, pr.name, \
        (SELECT string_agg(concat_ws(' ', t.first_name, t.last_name), ', ' ORDER BY lt.is_primary DESC) \
           FROM lease_tenants lt JOIN tenants t ON t.org_id = lt.org_id AND t.id = lt.tenant_id WHERE lt.lease_id = p.lease_id) \
   FROM payments p \
   JOIN leases l ON l.org_id = p.org_id AND l.id = p.lease_id \
   JOIN units u ON u.org_id = l.org_id AND u.id = l.unit_id \
   JOIN properties pr ON pr.org_id = u.org_id AND pr.id = u.property_id \
   JOIN users uc ON uc.id = p.created_by \
   LEFT JOIN receipts r ON r.org_id = p.org_id AND r.payment_id = p.id";

fn payment_summary(r: &sqlx::postgres::PgRow) -> Value {
    let recorded: String = r.get(7);
    json!({
        "id": r.get::<String, _>(0), "kind": "payment", "status": "validated",
        "lease_id": r.get::<String, _>(1), "amount": r.get::<i64, _>(2), "method": r.get::<String, _>(3),
        "external_ref": r.get::<Option<String>, _>(4), "paid_on": r.get::<String, _>(5), "cash_day": r.get::<String, _>(6),
        "recorded_at": recorded,
        "received_by": { "id": r.get::<String, _>(8), "display_name": r.get::<String, _>(9) },
        "payer": { "kind": r.get::<Option<String>, _>(10), "tenant_id": r.get::<Option<String>, _>(11),
                   "third_party_name": r.get::<Option<String>, _>(12) },
        "note": r.get::<Option<String>, _>(13),
        "receipt_id": r.get::<Option<String>, _>(14), "receipt_number": r.get::<Option<String>, _>(15),
        "unit_reference": r.get::<String, _>(16), "property_name": r.get::<String, _>(17),
        "tenants": r.get::<Option<String>, _>(18),
    })
}

// ───────── quittances (F10) ─────────

struct ReceiptInput<'a> {
    payment_id: Uuid,
    lease: &'a LeaseView,
    cmd: &'a Command,
    payer: &'a Payer,
    split: &'a Split<Uuid>,
    rows: &'a [OpenRow],
    issued_on: NaiveDate,
    issued_by: Uuid,
    issued_by_name: &'a str,
    clock: &'a dyn Clock,
}

struct Issued {
    id: Uuid,
    number: String,
    json: Value,
}

const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

fn month_label(period: NaiveDate) -> String {
    let name = usize::try_from(period.month0())
        .ok()
        .and_then(|i| MONTHS.get(i))
        .copied()
        .unwrap_or("");
    format!("{name} {}", period.year())
}

fn kind_label(kind: LineKind) -> &'static str {
    match kind {
        LineKind::Rent => "Loyer",
        LineKind::Charges => "Charges",
        LineKind::Exceptional => "Ligne exceptionnelle",
        LineKind::Arrears => "Arriéré",
    }
}

fn method_label(method: &str) -> &'static str {
    match method {
        "cash" => "Espèces",
        "mobile_money" => "Mobile money",
        "bank_transfer" => "Virement",
        "cheque" => "Chèque",
        _ => "",
    }
}

/// Numéro, compteur et instantané, puis insertion (F10). Le compteur est pris ici, en dernier ; seule écriture de
/// `receipt_counters` et `receipts`.
async fn issue_receipt(tx: &mut OrgTx, input: &ReceiptInput<'_>) -> Result<Issued, ServiceError> {
    let org = sqlx::query(
        "SELECT o.name, o.lifecycle, s.receipt_prefix, s.letterhead::text, s.legal_ids::text, s.extra_mentions, o.kind \
           FROM organizations o JOIN org_settings s ON s.org_id = o.id WHERE o.id = current_org_id()",
    )
    .fetch_one(tx.conn())
    .await?;
    let (org_name, lifecycle, prefix): (String, String, String) = (org.get(0), org.get(1), org.get(2));
    let trial = lifecycle == "setup";
    let series = if trial { "trial" } else { "official" };
    let year = input.issued_on.year();
    let year_small = i16::try_from(year).map_err(internal)?;
    let place = sqlx::query(
        "SELECT p.address, p.district, p.city, u.usage FROM units u \
           JOIN properties p ON p.org_id = u.org_id AND p.id = u.property_id WHERE u.id = $1::uuid",
    )
    .bind(input.lease.unit_id.to_string())
    .fetch_one(tx.conn())
    .await?;
    let in_words = words::amount_in_words(u32::try_from(input.cmd.amount).map_err(internal)?).map_err(internal)?;

    // ventilation par échéance (période), puis par ligne
    let mut periods: Vec<(Uuid, NaiveDate)> = Vec::new();
    for p in &input.split.parts {
        if let Some(r) = input.rows.iter().find(|r| r.line == p.line)
            && !periods.iter().any(|(i, _)| *i == r.installment)
        {
            periods.push((r.installment, r.period));
        }
    }
    let mut breakdown: Vec<Value> = Vec::new();
    for (installment, period) in &periods {
        let mut lines: Vec<Value> = Vec::new();
        for p in &input.split.parts {
            if let Some(r) = input
                .rows
                .iter()
                .find(|r| r.line == p.line && r.installment == *installment)
            {
                lines.push(json!({
                    "kind": r.kind.code(),
                    "label": r.label.clone().unwrap_or_else(|| kind_label(r.kind).to_string()),
                    "due_date": r.due_date.to_string(),
                    "amount": p.amount,
                }));
            }
        }
        breakdown.push(json!({ "period": period.to_string(), "period_label": month_label(*period), "lines": lines }));
    }
    let tenants: Vec<String> = input.lease.tenants.iter().map(|t| t.display_name.clone()).collect();
    let payer = match input.payer {
        Payer::Tenant(id) => json!({
            "kind": "tenant",
            "name": input.lease.tenants.iter().find(|t| t.id == *id).map(|t| t.display_name.clone()),
        }),
        Payer::ThirdParty { name, phone } => json!({
            "kind": "third_party", "name": name, "phone": phone,
            "mention": format!("Payé par {name} pour le compte de {}", tenants.first().cloned().unwrap_or_default()),
        }),
    };

    // compteur en dernier (DP-46) : la ligne reste verrouillée jusqu'au COMMIT
    let sequence: i32 = sqlx::query_scalar(
        "INSERT INTO receipt_counters (org_id, series, year, last_number) VALUES (current_org_id(), $1, $2, 1) \
         ON CONFLICT (org_id, series, year) DO UPDATE SET last_number = receipt_counters.last_number + 1 RETURNING last_number",
    )
    .bind(series)
    .bind(year_small)
    .fetch_one(tx.conn())
    .await?;
    let number =
        numbering::number(&prefix, year, u32::try_from(sequence).map_err(internal)?, trial).map_err(internal)?;
    let snapshot = json!({
        "v": 2,
        "issuer": {
            "name": org_name,
            // version 2 (DP-98) : nature de l'organisation, pour le filigrane « DÉMONSTRATION »
            "kind": org.get::<String, _>(6),
            "letterhead": serde_json::from_str::<Value>(&org.get::<String, _>(3)).unwrap_or(Value::Null),
            "legal_ids": serde_json::from_str::<Value>(&org.get::<String, _>(4)).unwrap_or(Value::Null),
            "extra_mentions": org.get::<Option<String>, _>(5),
        },
        "receipt": {
            "number": number, "series": series, "watermark": if trial { Some("ESSAI") } else { None },
            "issued_on": input.issued_on.to_string(), "issued_by": input.issued_by_name,
        },
        "tenants": tenants,
        "payer": payer,
        "unit": {
            "reference": input.lease.unit_reference, "property_name": input.lease.property_name,
            "address": place.get::<Option<String>, _>(0), "district": place.get::<Option<String>, _>(1),
            "city": place.get::<Option<String>, _>(2), "usage": place.get::<String, _>(3),
            "vat_included": input.lease.kind == "commercial" && input.lease.vat_included,
        },
        "payment": {
            "amount": input.cmd.amount, "amount_text": fcfa(input.cmd.amount), "amount_in_words": in_words,
            "paid_on": input.cmd.paid_on.to_string(), "method": input.cmd.method,
            "method_label": method_label(&input.cmd.method), "external_ref": input.cmd.external_ref,
        },
        "breakdown": breakdown,
        "advance_amount": input.split.advance,
        "remaining_after": input.split.remaining_due_after,
    });
    let id: String = sqlx::query_scalar(
        "INSERT INTO receipts (org_id, payment_id, series, numbering_year, sequence, number, amount, issued_on, issued_by, snapshot) \
         VALUES (current_org_id(), $1::uuid, $2, $3, $4, $5, $6, $7::date, $8::uuid, $9::jsonb) RETURNING id::text",
    )
    .bind(input.payment_id.to_string())
    .bind(series)
    .bind(year_small)
    .bind(sequence)
    .bind(&number)
    .bind(input.cmd.amount)
    .bind(input.issued_on.to_string())
    .bind(input.issued_by.to_string())
    .bind(snapshot.to_string())
    .fetch_one(tx.conn())
    .await?;
    let id = parse_id(&id)?;
    // rendu PDF en tâche de fond, mis en file dans la transaction qui émet la quittance (étape 08, R-049)
    crate::delivery::enqueue_render(tx, input.clock, id).await?;
    let words_value = snapshot["payment"]["amount_in_words"].clone();
    Ok(Issued {
        id,
        number: number.clone(),
        json: json!({
            "id": id.to_string(), "number": number, "series": series, "kind": "standard", "status": "issued",
            "issued_on": input.issued_on.to_string(), "amount": input.cmd.amount, "amount_in_words": words_value,
            "remaining_after": input.split.remaining_due_after, "advance_amount": input.split.advance,
            "payment_id": input.payment_id.to_string(),
            "snapshot": snapshot,
            "pdf": { "state": "pending", "papers": ["a4", "a5"], "sha256": null },
        }),
    })
}

const RECEIPT_SELECT: &str = "SELECT r.id::text, r.number, r.series, r.kind, r.issued_on::text, r.amount, r.snapshot::text, \
        r.payment_id::text, (c.receipt_id IS NOT NULL), \
        (SELECT count(DISTINCT f.paper) FROM receipt_files f WHERE f.org_id = r.org_id AND f.receipt_id = r.id), \
        (SELECT encode(f.sha256, 'hex') FROM receipt_files f WHERE f.org_id = r.org_id AND f.receipt_id = r.id AND f.paper = 'a5' \
          ORDER BY f.rendered_at DESC LIMIT 1) \
   FROM receipts r LEFT JOIN receipt_cancellations c ON c.org_id = r.org_id AND c.receipt_id = r.id";

fn receipt_json(r: &sqlx::postgres::PgRow) -> Result<Value, ServiceError> {
    let snapshot: Value = serde_json::from_str(&r.get::<String, _>(6)).map_err(internal)?;
    Ok(json!({
        "id": r.get::<String, _>(0), "number": r.get::<String, _>(1), "series": r.get::<String, _>(2),
        "kind": r.get::<String, _>(3), "status": if r.get::<bool, _>(8) { "cancelled" } else { "issued" },
        "issued_on": r.get::<String, _>(4), "amount": r.get::<i64, _>(5),
        "amount_in_words": snapshot["payment"]["amount_in_words"].clone(),
        "remaining_after": snapshot["remaining_after"].clone(), "advance_amount": snapshot["advance_amount"].clone(),
        "payment_id": r.get::<String, _>(7),
        "snapshot": snapshot,
        "pdf": {
            "state": if r.get::<i64, _>(9) >= 2 { "ready" } else { "pending" },
            "papers": ["a4", "a5"],
            "sha256": r.get::<Option<String>, _>(10),
        },
    }))
}

async fn receipt_by(tx: &mut OrgTx, by_payment: bool, id: Uuid) -> Result<Option<Value>, ServiceError> {
    let sql = if by_payment {
        format!("{RECEIPT_SELECT} WHERE r.payment_id = $1::uuid")
    } else {
        format!("{RECEIPT_SELECT} WHERE r.id = $1::uuid")
    };
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?;
    row.as_ref().map(receipt_json).transpose()
}

// ───────── tâches de plateforme ─────────

/// Contrôle des trous de numérotation (F10 règle 8, DP-96) : alerte S1 `receipts.numbering_gap` par organisation fautive.
pub async fn check_receipt_gaps(db: &Db, platform_url: Option<&str>) -> Result<usize, String> {
    let orgs: Vec<String> = {
        let mut tx = db.begin(Uuid::nil(), Uuid::nil()).await.map_err(|e| e.to_string())?;
        let ids = sqlx::query_scalar("SELECT id::text FROM orgs_with_receipt_gaps() AS id")
            .fetch_all(tx.conn())
            .await
            .map_err(|e| e.to_string())?;
        tx.commit().await.map_err(|e| e.to_string())?;
        ids
    };
    for org in &orgs {
        tracing::error!(alert = "receipts.numbering_gap", severity = "S1", org_id = %org, "trou de numérotation des quittances");
        if let Some(url) = platform_url {
            crate::platform::ops::record_event(url, "receipts.numbering_gap", &json!({ "org_id": org }))
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(orgs.len())
}

/// Purge des clés d'idempotence de plus de 30 jours (`API_CONTRACTS.md` §5.1), dans chaque organisation qui a un bail en cours.
pub async fn purge_idempotency_keys(db: &Db) -> Result<u64, DbError> {
    let orgs: Vec<String> = {
        let mut tx = db.begin(Uuid::nil(), Uuid::nil()).await?;
        let ids = sqlx::query_scalar("SELECT id::text FROM orgs_with_leases_in_progress() AS id")
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        ids
    };
    let mut total = 0_u64;
    for org in orgs {
        let Ok(org_id) = Uuid::parse_str(&org) else { continue };
        let mut tx = db.begin(org_id, Uuid::nil()).await?;
        let n = sqlx::query("DELETE FROM idempotency_keys WHERE created_at < app_now() - make_interval(days => $1)")
            .bind(IDEMPOTENCY_DAYS)
            .execute(tx.conn())
            .await?
            .rows_affected();
        tx.commit().await?;
        total = total.saturating_add(n);
    }
    Ok(total)
}
