//! Patrimoine : biens, lots, locataires, fiches mandants et recherche unique (étape 05 ; F04, F05, F06, DP-92).
//!
//! Prolonge le service d'identité (base, horloge, audit, détection d'accès à une autre organisation). Chaque écriture passe par
//! `OrgTx` et écrit sa ligne d'audit dans la même transaction (R-008). Les règles pures viennent de `immo_domain`.

// Requêtes sur les tables de l'organisation active, dans son contexte (OrgTx) : même exception documentée de DP-77 que
// `identity::settings`.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

use immo_domain::catalog::{self as rules, CatalogError, UNIT_KINDS, UnitUsage};
use immo_domain::phone;

use crate::audit::{self, AuditAction, AuditEntry};
use crate::identity::authorize::{Plan, Role};
use crate::identity::error::{FieldIssue, ServiceError, sql_code};
use crate::identity::service::{
    ActiveOrg, Authenticated, ClientInfo, IdentityService, at, audit_context, from_micros, internal, micros,
};
use crate::platform::db::OrgTx;

/// Corps d'une requête : les clés présentes seulement (les clés inconnues ont été refusées par l'API).
pub type Input = Map<String, Value>;

const USAGES: [&str; 3] = ["residential", "commercial", "mixed"];
const PARTY_KINDS: [&str; 2] = ["person", "company"];
const PAYOUT_METHODS: [&str; 4] = ["cash", "mobile_money", "bank_transfer", "cheque"];
const UNIT_STATUSES: [&str; 4] = ["vacant", "occupied", "under_works", "archived"];

// ───────── vues ─────────

#[derive(Debug, Clone)]
pub struct OwnerRef {
    pub id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct UnitView {
    pub id: Uuid,
    pub property_id: Uuid,
    pub property_name: String,
    pub reference: String,
    pub kind: String,
    pub usage: String,
    /// Texte décimal (`"18.50"`), jamais un flottant.
    pub surface_m2: Option<String>,
    pub reference_rent: Option<i64>,
    pub permitted_activity: Option<String>,
    /// `vacant`, `under_works` ou `archived` (`occupied` viendra avec les baux, étape 06).
    pub status: String,
    pub archived_at: Option<DateTime<Utc>>,
    pub version: i32,
}

#[derive(Debug, Clone)]
pub struct PropertyView {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    pub default_owner: Option<OwnerRef>,
    pub notes: Option<String>,
    pub unit_count: i64,
    pub vacant_count: i64,
    pub archived_at: Option<DateTime<Utc>>,
    pub version: i32,
    /// Lots du bien : renseignés dans la fiche seulement.
    pub units: Vec<UnitView>,
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub name: Option<String>,
    pub phone: Option<String>,
}

/// Champs réservés au gérant, au gestionnaire et au propriétaire (`API_CONTRACTS.md` §4.5).
#[derive(Debug, Clone)]
pub struct TenantPrivate {
    pub backup_contact: Option<Contact>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TenantView {
    pub id: Uuid,
    pub kind: String,
    pub last_name: String,
    pub first_name: Option<String>,
    pub display_name: String,
    pub trade_activity: Option<String>,
    pub phone: Option<String>,
    pub phone_is_whatsapp: bool,
    pub email: Option<String>,
    /// `None` : champs masqués pour ce rôle (absents de la réponse).
    pub private: Option<TenantPrivate>,
    pub merged_into_id: Option<Uuid>,
    pub archived_at: Option<DateTime<Utc>>,
    pub version: i32,
}

#[derive(Debug, Clone)]
pub struct OwnerPrivate {
    pub phone: Option<String>,
    pub payout_method: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OwnerView {
    pub id: Uuid,
    pub kind: String,
    pub display_name: String,
    pub is_self: bool,
    /// `None` : téléphone et mode de reversement masqués (réservés à `mandate.write`, DP-92).
    pub private: Option<OwnerPrivate>,
    pub property_count: i64,
    pub archived_at: Option<DateTime<Utc>>,
    pub version: i32,
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

/// Filtres d'une liste ; un filtre absent ne filtre pas.
#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    pub q: Option<String>,
    pub archived: bool,
    pub district: Option<String>,
    pub owner_id: Option<Uuid>,
    pub property_id: Option<Uuid>,
    pub usage: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SearchTenant {
    pub id: Uuid,
    pub display_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchUnit {
    pub id: Uuid,
    pub label: String,
    pub property_name: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub tenants: Vec<SearchTenant>,
    pub units: Vec<SearchUnit>,
}

// ───────── lecture des champs ─────────

pub(crate) fn catalog_code(e: CatalogError) -> &'static str {
    match e {
        CatalogError::Required => "validation.required",
        CatalogError::TooLong => "validation.too_long",
        CatalogError::AmountNotPositive => "validation.amount_positive",
        CatalogError::AmountOutOfRange => "validation.amount_out_of_range",
        CatalogError::Invalid => "validation.invalid",
    }
}

/// Lit un corps champ par champ et accumule toutes les erreurs (API_CONTRACTS.md §3.3). Convention : `None` = clé absente
/// (aucun changement) ; `Some(None)` = `null` ou texte vide (effacer) ; `Some(Some(v))` = valeur.
pub(crate) struct Reader<'a> {
    pub(crate) map: &'a Input,
    pub(crate) issues: Vec<FieldIssue>,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(map: &'a Input) -> Self {
        Self { map, issues: vec![] }
    }

    pub(crate) fn fail(&mut self, field: &'static str, code: &'static str) {
        if !self.issues.iter().any(|i| i.field == field) {
            self.issues.push(FieldIssue { field, code });
        }
    }

    pub(crate) fn text(&mut self, field: &'static str, max: usize) -> Option<Option<String>> {
        match self.map.get(field) {
            None => None,
            Some(Value::Null) => Some(None),
            Some(Value::String(s)) => {
                let t = s.trim();
                if t.is_empty() {
                    Some(None)
                } else if t.chars().count() > max {
                    self.fail(field, "validation.too_long");
                    None
                } else {
                    Some(Some(t.to_string()))
                }
            }
            Some(_) => {
                self.fail(field, "validation.invalid");
                None
            }
        }
    }

    /// Texte obligatoire : à la création, absent ou vide est refusé ; à la modification, seul « vide » l'est.
    pub(crate) fn required(&mut self, field: &'static str, min: usize, max: usize, creating: bool) -> Option<String> {
        match self.text(field, max) {
            None => {
                if creating && !self.issues.iter().any(|i| i.field == field) {
                    self.fail(field, "validation.required");
                }
                None
            }
            Some(None) => {
                self.fail(field, "validation.required");
                None
            }
            Some(Some(t)) if t.chars().count() < min => {
                self.fail(field, "validation.required");
                None
            }
            Some(Some(t)) => Some(t),
        }
    }

    pub(crate) fn choice(&mut self, field: &'static str, allowed: &[&str]) -> Option<Option<String>> {
        match self.text(field, 40) {
            Some(Some(v)) if !allowed.contains(&v.as_str()) => {
                self.fail(field, "validation.enum");
                None
            }
            other => other,
        }
    }

    pub(crate) fn phone(&mut self, field: &'static str) -> Option<Option<String>> {
        match self.text(field, 40) {
            Some(Some(v)) => match phone::normalize(&v) {
                Some(p) => Some(Some(p)),
                None => {
                    self.fail(field, "validation.phone_format");
                    None
                }
            },
            other => other,
        }
    }

    pub(crate) fn email(&mut self, field: &'static str) -> Option<Option<String>> {
        match self.text(field, 254) {
            Some(Some(v)) => {
                let lower = v.to_lowercase();
                let plausible = lower
                    .split_once('@')
                    .is_some_and(|(a, d)| !a.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.'))
                    && !lower.chars().any(char::is_whitespace);
                if plausible {
                    Some(Some(lower))
                } else {
                    self.fail(field, "validation.email_format");
                    None
                }
            }
            other => other,
        }
    }

    pub(crate) fn boolean(&mut self, field: &'static str) -> Option<bool> {
        match self.map.get(field) {
            None => None,
            Some(Value::Bool(b)) => Some(*b),
            Some(_) => {
                self.fail(field, "validation.invalid");
                None
            }
        }
    }

    pub(crate) fn uuid(&mut self, field: &'static str) -> Option<Option<Uuid>> {
        match self.map.get(field) {
            None => None,
            Some(Value::Null) => Some(None),
            Some(Value::String(s)) => match Uuid::parse_str(s.trim()) {
                Ok(id) => Some(Some(id)),
                Err(_) => {
                    self.fail(field, "validation.invalid");
                    None
                }
            },
            Some(_) => {
                self.fail(field, "validation.invalid");
                None
            }
        }
    }

    /// Montant entier FCFA (R-001, P16) : un décimal ou un texte est refusé comme « non entier ».
    pub(crate) fn rent(&mut self, field: &'static str) -> Option<Option<i64>> {
        match self.map.get(field) {
            None => None,
            Some(Value::Null) => Some(None),
            Some(Value::Number(n)) => match n.as_i64() {
                Some(v) => match rules::reference_rent(v) {
                    Ok(f) => Some(Some(f.get())),
                    Err(e) => {
                        self.fail(field, catalog_code(e));
                        None
                    }
                },
                None => {
                    self.fail(
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
                self.fail(field, "validation.amount_not_integer");
                None
            }
        }
    }

    pub(crate) fn surface(&mut self, field: &'static str) -> Option<Option<String>> {
        let raw = match self.map.get(field) {
            None => return None,
            Some(Value::Null) => return Some(None),
            Some(Value::String(s)) if s.trim().is_empty() => return Some(None),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) if n.is_u64() => n.to_string(),
            Some(_) => {
                self.fail(field, "validation.invalid");
                return None;
            }
        };
        match rules::parse_surface(&raw) {
            Ok(s) => Some(Some(s)),
            Err(e) => {
                self.fail(field, catalog_code(e));
                None
            }
        }
    }

    pub(crate) fn object(&mut self, field: &'static str) -> Option<Option<Input>> {
        match self.map.get(field) {
            None => None,
            Some(Value::Null) => Some(None),
            Some(Value::Object(m)) => Some(Some(m.clone())),
            Some(_) => {
                self.fail(field, "validation.invalid");
                None
            }
        }
    }

    pub(crate) fn finish(self) -> Result<(), ServiceError> {
        if self.issues.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::Fields(self.issues))
        }
    }
}

// ───────── outils ─────────

pub(crate) fn org_of(auth: &Authenticated) -> Result<&ActiveOrg, ServiceError> {
    auth.org
        .as_ref()
        .ok_or(ServiceError::Forbidden("organization_required"))
}

/// R-081 : l'agent terrain ne voit que ses lots ; les autres rôles voient tout (`None`).
pub(crate) fn agent_of(auth: &Authenticated) -> Option<String> {
    (auth.role == Some(Role::FieldAgent)).then(|| auth.user.id.to_string())
}

pub(crate) fn sees_tenant_private(auth: &Authenticated) -> bool {
    matches!(auth.role, Some(Role::Manager | Role::PropertyManager | Role::Owner))
}

fn sees_owner_private(auth: &Authenticated) -> bool {
    matches!(auth.role, Some(Role::Manager | Role::Owner))
}

pub(crate) fn parse_id(text: &str) -> Result<Uuid, ServiceError> {
    Uuid::parse_str(text).map_err(|_| ServiceError::from(internal("identifiant illisible")))
}

pub(crate) fn opt_id(text: Option<String>) -> Result<Option<Uuid>, ServiceError> {
    text.map(|t| parse_id(&t)).transpose()
}

pub(crate) fn opt_time(value: Option<i64>) -> Result<Option<DateTime<Utc>>, ServiceError> {
    value.map(from_micros).transpose().map_err(ServiceError::from)
}

pub(crate) fn display_name(first: Option<&str>, last: &str) -> String {
    match first {
        Some(f) => format!("{f} {last}"),
        None => last.to_string(),
    }
}

/// Motif `LIKE` d'une saisie : caractères spéciaux échappés (`\`, `%`, `_`) ; le pliage des accents se fait en SQL (`app_fold`).
pub(crate) fn like_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || text.len() > 600 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| text.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
        .collect()
}

/// Curseur opaque : (clé de tri, identifiant) en hexadécimal.
fn write_cursor(key: &str, id: Uuid) -> String {
    hex(format!("{key}\u{1f}{id}").as_bytes())
}

fn read_cursor(cursor: Option<&str>) -> Result<(Option<String>, Option<String>), ServiceError> {
    let Some(c) = cursor else { return Ok((None, None)) };
    let text = unhex(c)
        .and_then(|b| String::from_utf8(b).ok())
        .ok_or(ServiceError::BadCursor)?;
    let (key, id) = text.split_once('\u{1f}').ok_or(ServiceError::BadCursor)?;
    Uuid::parse_str(id).map_err(|_| ServiceError::BadCursor)?;
    Ok((Some(key.to_string()), Some(id.to_string())))
}

fn limit_of(q: &ListQuery) -> i64 {
    q.limit.unwrap_or(25).clamp(1, 100)
}

fn query_pattern(q: Option<&str>) -> Option<String> {
    q.map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| format!("%{}%", like_escape(t)))
}

pub(crate) fn audit_entry(
    action: AuditAction,
    object_type: &'static str,
    id: Uuid,
    before: Option<Value>,
    after: Value,
) -> Result<AuditEntry, ServiceError> {
    let mut e = AuditEntry::new(action, object_type, id);
    if let Some(b) = before {
        e = e.before(b).map_err(internal)?;
    }
    e.after(after).map_err(|x| ServiceError::from(internal(x)))
}

/// Enregistre `field` dans before/after s'il change.
pub(crate) fn diff<T: PartialEq + serde::Serialize>(
    before: &mut Map<String, Value>,
    after: &mut Map<String, Value>,
    field: &str,
    old: &T,
    new: &T,
) {
    if old != new {
        before.insert(field.to_string(), json!(old));
        after.insert(field.to_string(), json!(new));
    }
}

/// Fusionne une valeur lue (`None` : inchangée) avec la valeur actuelle.
pub(crate) fn merged<T: Clone>(read: Option<Option<T>>, current: &Option<T>) -> Option<T> {
    match read {
        None => current.clone(),
        Some(v) => v,
    }
}

// ───────── requêtes ─────────

pub(crate) const UNIT_STATUS_SQL: &str = "CASE WHEN u.archived_at IS NOT NULL THEN 'archived' WHEN EXISTS (SELECT 1 FROM leases sl WHERE sl.org_id = u.org_id AND sl.unit_id = u.id AND sl.status IN ('active', 'notice') AND sl.start_date <= (app_now() AT TIME ZONE 'Africa/Ouagadougou')::date AND (sl.effective_end IS NULL OR sl.effective_end >= (app_now() AT TIME ZONE 'Africa/Ouagadougou')::date)) THEN 'occupied' WHEN u.under_works THEN 'under_works' ELSE 'vacant' END";

/// Portée de l'agent terrain sur un lot `u` ; paramètre `$N` = identifiant de l'utilisateur ou NULL.
pub(crate) fn unit_scope(param: usize) -> String {
    format!(
        "(${param}::uuid IS NULL OR u.id IN (SELECT ua.unit_id FROM unit_assignments ua WHERE ua.user_id = ${param}::uuid AND ua.revoked_at IS NULL))"
    )
}

/// Portée de l'agent terrain sur un bien `p` : il contient au moins un lot assigné.
fn property_scope(param: usize) -> String {
    format!(
        "(${param}::uuid IS NULL OR EXISTS (SELECT 1 FROM units su JOIN unit_assignments ua ON ua.unit_id = su.id AND ua.revoked_at IS NULL \
           AND ua.user_id = ${param}::uuid WHERE su.property_id = p.id))"
    )
}

fn unit_columns() -> String {
    format!(
        "u.id::text, u.property_id::text, p.name, u.reference, u.kind, u.usage, u.surface_m2::text, u.reference_rent, \
         u.permitted_activity, {UNIT_STATUS_SQL}, {}, u.row_version, u.search_key",
        at("u.archived_at")
    )
}

fn unit_from(r: &PgRow) -> Result<UnitView, ServiceError> {
    Ok(UnitView {
        id: parse_id(&r.get::<String, _>(0))?,
        property_id: parse_id(&r.get::<String, _>(1))?,
        property_name: r.get(2),
        reference: r.get(3),
        kind: r.get(4),
        usage: r.get(5),
        surface_m2: r.get(6),
        reference_rent: r.get(7),
        permitted_activity: r.get(8),
        status: r.get(9),
        archived_at: opt_time(r.get(10))?,
        version: r.get(11),
    })
}

fn property_columns() -> String {
    format!(
        "p.id::text, p.name, p.address, p.district, p.city, p.default_owner_id::text, o.display_name, p.notes, \
         (SELECT count(*) FROM units cu WHERE cu.property_id = p.id AND (cu.archived_at IS NULL OR p.archived_at IS NOT NULL)), \
         (SELECT count(*) FROM units cu WHERE cu.property_id = p.id AND cu.archived_at IS NULL AND NOT cu.under_works AND NOT EXISTS (SELECT 1 FROM leases cl WHERE cl.org_id = cu.org_id AND cl.unit_id = cu.id AND cl.status IN ('active', 'notice'))), \
         {}, p.row_version, p.search_key",
        at("p.archived_at")
    )
}

fn property_from(r: &PgRow) -> Result<PropertyView, ServiceError> {
    let owner_id = opt_id(r.get(5))?;
    let owner_name: Option<String> = r.get(6);
    Ok(PropertyView {
        id: parse_id(&r.get::<String, _>(0))?,
        name: r.get(1),
        address: r.get(2),
        district: r.get(3),
        city: r.get(4),
        default_owner: owner_id
            .zip(owner_name)
            .map(|(id, display_name)| OwnerRef { id, display_name }),
        notes: r.get(7),
        unit_count: r.get(8),
        vacant_count: r.get(9),
        archived_at: opt_time(r.get(10))?,
        version: r.get(11),
        units: vec![],
    })
}

fn tenant_columns() -> String {
    format!(
        "t.id::text, t.kind, t.last_name, t.first_name, t.trade_activity, t.phone, t.phone_is_whatsapp, t.email, \
         t.backup_contact_name, t.backup_contact_phone, t.notes, t.merged_into_id::text, {}, t.row_version, t.search_key",
        at("t.archived_at")
    )
}

fn tenant_from(r: &PgRow, private: bool) -> Result<TenantView, ServiceError> {
    let last: String = r.get(2);
    let first: Option<String> = r.get(3);
    let (bname, bphone): (Option<String>, Option<String>) = (r.get(8), r.get(9));
    Ok(TenantView {
        id: parse_id(&r.get::<String, _>(0))?,
        kind: r.get(1),
        display_name: display_name(first.as_deref(), &last),
        last_name: last,
        first_name: first,
        trade_activity: r.get(4),
        phone: r.get(5),
        phone_is_whatsapp: r.get(6),
        email: r.get(7),
        private: private.then(|| TenantPrivate {
            backup_contact: (bname.is_some() || bphone.is_some()).then_some(Contact {
                name: bname,
                phone: bphone,
            }),
            notes: r.get(10),
        }),
        merged_into_id: opt_id(r.get(11))?,
        archived_at: opt_time(r.get(12))?,
        version: r.get(13),
    })
}

fn owner_columns() -> String {
    format!(
        "o.id::text, o.kind, o.display_name, o.is_self, o.phone, o.payout_method, \
         (SELECT count(*) FROM properties op WHERE op.default_owner_id = o.id AND op.archived_at IS NULL), {}, o.row_version, o.search_key",
        at("o.archived_at")
    )
}

fn owner_from(r: &PgRow, private: bool) -> Result<OwnerView, ServiceError> {
    Ok(OwnerView {
        id: parse_id(&r.get::<String, _>(0))?,
        kind: r.get(1),
        display_name: r.get(2),
        is_self: r.get(3),
        private: private.then(|| OwnerPrivate {
            phone: r.get(4),
            payout_method: r.get(5),
        }),
        property_count: r.get(6),
        archived_at: opt_time(r.get(7))?,
        version: r.get(8),
    })
}

pub(crate) fn is_unique_violation(e: &sqlx::Error, constraint: &str) -> bool {
    sql_code(e).as_deref() == Some("23505")
        && e.as_database_error()
            .and_then(|d| d.constraint())
            .is_some_and(|c| c == constraint)
}

/// Lit une page : `rows` contient au plus `limit + 1` lignes ; la dernière colonne de chaque ligne est la clé de tri.
fn page<T>(
    rows: Vec<PgRow>,
    limit: i64,
    map: impl Fn(&PgRow) -> Result<T, ServiceError>,
    id: impl Fn(&T) -> Uuid,
) -> Result<Page<T>, ServiceError> {
    let limit = usize::try_from(limit).unwrap_or(25);
    let has_more = rows.len() > limit;
    let mut items = vec![];
    let mut last_key = None;
    for r in rows.iter().take(limit) {
        let key_index = r.len().saturating_sub(1);
        last_key = Some(r.get::<String, _>(key_index));
        items.push(map(r)?);
    }
    let next_cursor = if has_more {
        items
            .last()
            .zip(last_key)
            .map(|(item, key)| write_cursor(&key, id(item)))
    } else {
        None
    };
    Ok(Page { items, next_cursor })
}

// ───────── champs d'un lot ─────────

#[derive(Default)]
struct UnitFields {
    reference: Option<String>,
    kind: Option<String>,
    usage: Option<String>,
    surface: Option<Option<String>>,
    rent: Option<Option<i64>>,
    activity: Option<Option<String>>,
}

fn read_unit_fields(r: &mut Reader<'_>) -> UnitFields {
    let reference = match r.text("reference", rules::MAX_REFERENCE_CHARS) {
        None => None,
        Some(None) => {
            r.fail("reference", "validation.required");
            None
        }
        Some(Some(t)) => match rules::unit_reference(&t) {
            Ok(v) => Some(v),
            Err(e) => {
                r.fail("reference", catalog_code(e));
                None
            }
        },
    };
    let kind = match r.choice("kind", &UNIT_KINDS) {
        Some(None) => {
            r.fail("kind", "validation.required");
            None
        }
        other => other.flatten(),
    };
    let usage = match r.choice("usage", &USAGES) {
        Some(None) => {
            r.fail("usage", "validation.required");
            None
        }
        other => other.flatten(),
    };
    UnitFields {
        reference,
        kind,
        usage,
        surface: r.surface("surface_m2"),
        rent: r.rent("reference_rent"),
        activity: r.text("permitted_activity", 200),
    }
}

/// R-012 : l'activité autorisée n'existe pas pour un lot d'habitation.
fn check_activity(r: &mut Reader<'_>, usage: &str, activity: &Option<String>) {
    if activity.is_some() && UnitUsage::parse(usage).is_some_and(|u| !rules::permitted_activity_applies(u)) {
        r.fail("permitted_activity", "validation.not_applicable");
    }
}

// ───────── service ─────────

impl IdentityService {
    async fn trace_and_404(
        &self,
        auth: &Authenticated,
        object: &'static str,
        id: Uuid,
        client: &ClientInfo,
    ) -> ServiceError {
        if let Some(org) = auth.org.as_ref()
            && let Err(e) = self.trace_missing(org.id, auth.user.id, object, id, client).await
        {
            return e.into();
        }
        ServiceError::NotFound
    }

    async fn unit_in(&self, tx: &mut OrgTx, id: Uuid, agent: Option<&str>) -> Result<Option<UnitView>, ServiceError> {
        let sql = format!(
            "SELECT {} FROM units u JOIN properties p ON p.id = u.property_id WHERE u.id = $1::uuid AND {}",
            unit_columns(),
            unit_scope(2)
        );
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id.to_string())
            .bind(agent)
            .fetch_optional(tx.conn())
            .await?;
        row.as_ref().map(unit_from).transpose()
    }

    async fn property_in(
        &self,
        tx: &mut OrgTx,
        id: Uuid,
        agent: Option<&str>,
    ) -> Result<Option<PropertyView>, ServiceError> {
        let sql = format!(
            "SELECT {} FROM properties p LEFT JOIN owners o ON o.id = p.default_owner_id WHERE p.id = $1::uuid AND {}",
            property_columns(),
            property_scope(2)
        );
        let Some(row) = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id.to_string())
            .bind(agent)
            .fetch_optional(tx.conn())
            .await?
        else {
            return Ok(None);
        };
        let mut view = property_from(&row)?;
        let sql = format!(
            "SELECT {} FROM units u JOIN properties p ON p.id = u.property_id \
              WHERE u.property_id = $1::uuid AND (u.archived_at IS NULL OR p.archived_at IS NOT NULL) AND {} \
              ORDER BY u.search_key, u.id",
            unit_columns(),
            unit_scope(2)
        );
        let units = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id.to_string())
            .bind(agent)
            .fetch_all(tx.conn())
            .await?;
        view.units = units.iter().map(unit_from).collect::<Result<_, _>>()?;
        Ok(Some(view))
    }

    async fn tenant_in(
        &self,
        tx: &mut OrgTx,
        id: Uuid,
        agent: Option<&str>,
        private: bool,
    ) -> Result<Option<TenantView>, ServiceError> {
        // R-081 : l'agent terrain ne voit que les locataires des baux en cours de ses lots
        let sql = format!(
            "SELECT {} FROM tenants t WHERE t.id = $1::uuid AND ($2::uuid IS NULL OR EXISTS (SELECT 1 FROM lease_tenants lt JOIN leases l ON l.org_id = lt.org_id AND l.id = lt.lease_id JOIN unit_assignments ua ON ua.unit_id = l.unit_id AND ua.revoked_at IS NULL AND ua.user_id = $2::uuid WHERE lt.tenant_id = t.id AND l.status IN ('active', 'notice')))",
            tenant_columns()
        );
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id.to_string())
            .bind(agent)
            .fetch_optional(tx.conn())
            .await?;
        row.as_ref().map(|r| tenant_from(r, private)).transpose()
    }

    async fn owner_in(&self, tx: &mut OrgTx, id: Uuid, private: bool) -> Result<Option<OwnerView>, ServiceError> {
        let sql = format!("SELECT {} FROM owners o WHERE o.id = $1::uuid", owner_columns());
        let row = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(id.to_string())
            .fetch_optional(tx.conn())
            .await?;
        row.as_ref().map(|r| owner_from(r, private)).transpose()
    }

    /// Mandant actif de l'organisation (sinon `owner.not_found` sur le champ).
    async fn active_owner(&self, tx: &mut OrgTx, id: Uuid) -> Result<bool, ServiceError> {
        let found: Option<String> =
            sqlx::query_scalar("SELECT id::text FROM owners WHERE id = $1::uuid AND archived_at IS NULL")
                .bind(id.to_string())
                .fetch_optional(tx.conn())
                .await?;
        Ok(found.is_some())
    }

    // ───────── biens ─────────

    /// `GET /properties`.
    pub async fn list_properties(
        &self,
        auth: &Authenticated,
        q: &ListQuery,
    ) -> Result<Page<PropertyView>, ServiceError> {
        let org = org_of(auth)?;
        let (ck, cid) = read_cursor(q.cursor.as_deref())?;
        let limit = limit_of(q);
        let sql = format!(
            "SELECT {} FROM properties p LEFT JOIN owners o ON o.id = p.default_owner_id \
              WHERE {} AND (CASE WHEN $2::boolean THEN p.archived_at IS NOT NULL ELSE p.archived_at IS NULL END) \
                AND ($3::text IS NULL OR p.search_key LIKE app_fold($3) ESCAPE '\\') \
                AND ($4::text IS NULL OR app_fold(coalesce(p.district, '')) = app_fold($4)) \
                AND ($5::uuid IS NULL OR p.default_owner_id = $5::uuid) \
                AND ($6::text IS NULL OR (p.search_key, p.id::text) > ($6, $7)) \
              ORDER BY p.search_key, p.id::text LIMIT $8",
            property_columns(),
            property_scope(1)
        );
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(agent_of(auth))
            .bind(q.archived)
            .bind(query_pattern(q.q.as_deref()))
            .bind(q.district.as_deref())
            .bind(q.owner_id.map(|i| i.to_string()))
            .bind(ck)
            .bind(cid)
            .bind(limit.saturating_add(1))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        page(rows, limit, property_from, |p| p.id)
    }

    /// `GET /properties/{id}`.
    pub async fn property(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<PropertyView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = self.property_in(&mut tx, id, agent_of(auth).as_deref()).await?;
        tx.commit().await?;
        match found {
            Some(p) => Ok(p),
            None => Err(self.trace_and_404(auth, "property", id, client).await),
        }
    }

    /// `POST /properties` (R-010 : `single_unit` crée 1 bien et 1 lot dans la même transaction).
    pub async fn create_property(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<PropertyView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let name = r.required("name", 1, 160, true);
        let address = r.text("address", 300).flatten();
        let district = r.text("district", 120).flatten();
        let city = r.text("city", 120).flatten();
        let notes = r.text("notes", 4000).flatten();
        let owner = r.uuid("default_owner_id").flatten();
        let single = r.object("single_unit").flatten();
        let unit = single.as_ref().map(|m| {
            let mut ur = Reader::new(m);
            let fields = read_unit_fields(&mut ur);
            let usage = fields.usage.clone().unwrap_or_else(|| "residential".to_string());
            let activity = fields.activity.clone().flatten();
            check_activity(&mut ur, &usage, &activity);
            (fields, ur.issues)
        });
        if let Some((_, issues)) = &unit {
            for i in issues {
                r.fail(i.field, i.code);
            }
        }
        r.finish()?;
        let name = name.ok_or(ServiceError::Invalid("name"))?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let owner = match owner {
            Some(o) => {
                if !self.active_owner(&mut tx, o).await? {
                    return Err(ServiceError::Fields(vec![FieldIssue {
                        field: "default_owner_id",
                        code: "owner.not_found",
                    }]));
                }
                Some(o.to_string())
            }
            None if org.plan == Plan::Owner => {
                sqlx::query_scalar("SELECT id::text FROM owners WHERE is_self AND archived_at IS NULL")
                    .fetch_optional(tx.conn())
                    .await?
            }
            None => None,
        };
        let pid: String = sqlx::query_scalar(
            "INSERT INTO properties (org_id, name, address, district, city, default_owner_id, notes, created_at, created_by) \
             VALUES (current_org_id(), $1, $2, $3, $4, $5::uuid, $6, $7::timestamptz, $8::uuid) RETURNING id::text",
        )
        .bind(&name)
        .bind(&address)
        .bind(&district)
        .bind(&city)
        .bind(&owner)
        .bind(&notes)
        .bind(&now)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await?;
        let pid = parse_id(&pid)?;
        let ctx = audit_context(auth.user.id, client);
        audit::record(
            &mut tx,
            &ctx,
            audit_entry(
                AuditAction::PropertyCreated,
                "property",
                pid,
                None,
                json!({ "name": name }),
            )?,
        )
        .await?;
        if let Some((f, _)) = unit {
            let reference = f.reference.unwrap_or_else(|| "1".to_string());
            let kind = f.kind.unwrap_or_else(|| "house".to_string());
            let usage = f.usage.unwrap_or_else(|| "residential".to_string());
            let uid = self
                .insert_unit(
                    &mut tx,
                    pid,
                    &reference,
                    &kind,
                    &usage,
                    f.surface.flatten(),
                    f.rent.flatten(),
                    f.activity.flatten(),
                    &now,
                    auth,
                )
                .await?;
            audit::record(
                &mut tx,
                &ctx,
                audit_entry(
                    AuditAction::UnitCreated,
                    "unit",
                    uid,
                    None,
                    json!({ "reference": reference, "property_id": pid.to_string() }),
                )?,
            )
            .await?;
        }
        let view = self
            .property_in(&mut tx, pid, None)
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    #[allow(clippy::too_many_arguments)]
    async fn insert_unit(
        &self,
        tx: &mut OrgTx,
        property_id: Uuid,
        reference: &str,
        kind: &str,
        usage: &str,
        surface: Option<String>,
        rent: Option<i64>,
        activity: Option<String>,
        now: &str,
        auth: &Authenticated,
    ) -> Result<Uuid, ServiceError> {
        let inserted: Result<String, sqlx::Error> = sqlx::query_scalar(
            "INSERT INTO units (org_id, property_id, reference, kind, usage, surface_m2, reference_rent, permitted_activity, created_at, created_by) \
             VALUES (current_org_id(), $1::uuid, $2, $3, $4, $5::numeric(8,2), $6, $7, $8::timestamptz, $9::uuid) RETURNING id::text",
        )
        .bind(property_id.to_string())
        .bind(reference)
        .bind(kind)
        .bind(usage)
        .bind(surface)
        .bind(rent)
        .bind(activity)
        .bind(now)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await;
        match inserted {
            Ok(id) => parse_id(&id),
            Err(e) if is_unique_violation(&e, "units_reference_ci") => {
                Err(ServiceError::conflict("unit.reference_duplicate"))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// `PATCH /properties/{id}` (`If-Match`).
    pub async fn patch_property(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<PropertyView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let name = r.required("name", 1, 160, false);
        let address = r.text("address", 300);
        let district = r.text("district", 120);
        let city = r.text("city", 120);
        let notes = r.text("notes", 4000);
        let owner = r.uuid("default_owner_id");
        r.finish()?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) = sqlx::query(
            "SELECT name, address, district, city, default_owner_id::text, notes, archived_at IS NOT NULL, row_version \
               FROM properties WHERE id = $1::uuid FOR UPDATE",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "property", id, client).await);
        };
        if cur.get::<bool, _>(6) {
            return Err(ServiceError::conflict("property.archived"));
        }
        if cur.get::<i32, _>(7) != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let old_name: String = cur.get(0);
        let (old_address, old_district, old_city): (Option<String>, Option<String>, Option<String>) =
            (cur.get(1), cur.get(2), cur.get(3));
        let (old_owner, old_notes): (Option<String>, Option<String>) = (cur.get(4), cur.get(5));
        let new_name = name.unwrap_or_else(|| old_name.clone());
        let new_address = merged(address, &old_address);
        let new_district = merged(district, &old_district);
        let new_city = merged(city, &old_city);
        let new_notes = merged(notes, &old_notes);
        let new_owner = match owner {
            None => old_owner.clone(),
            Some(None) => None,
            Some(Some(o)) => {
                if !self.active_owner(&mut tx, o).await? {
                    return Err(ServiceError::Fields(vec![FieldIssue {
                        field: "default_owner_id",
                        code: "owner.not_found",
                    }]));
                }
                Some(o.to_string())
            }
        };
        let (mut before, mut after) = (Map::new(), Map::new());
        diff(&mut before, &mut after, "name", &old_name, &new_name);
        diff(&mut before, &mut after, "address", &old_address, &new_address);
        diff(&mut before, &mut after, "district", &old_district, &new_district);
        diff(&mut before, &mut after, "city", &old_city, &new_city);
        diff(&mut before, &mut after, "default_owner_id", &old_owner, &new_owner);
        if old_notes != new_notes {
            after.insert("notes".into(), json!("modifié"));
        }
        if !after.is_empty() {
            sqlx::query(
                "UPDATE properties SET name = $2, address = $3, district = $4, city = $5, default_owner_id = $6::uuid, notes = $7, \
                        row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&new_name)
            .bind(&new_address)
            .bind(&new_district)
            .bind(&new_city)
            .bind(&new_owner)
            .bind(&new_notes)
            .execute(tx.conn())
            .await?;
            let entry = audit_entry(
                AuditAction::PropertyUpdated,
                "property",
                id,
                Some(Value::Object(before)),
                Value::Object(after),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self
            .property_in(&mut tx, id, None)
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /properties/{id}/archive` : le bien et ses lots non archivés, dans la même transaction (R-005).
    pub async fn archive_property(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<PropertyView, ServiceError> {
        let org = org_of(auth)?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(archived) = sqlx::query_scalar::<_, bool>(
            "SELECT archived_at IS NOT NULL FROM properties WHERE id = $1::uuid FOR UPDATE",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "property", id, client).await);
        };
        if !archived {
            // R-006 : rien ne s'archive tant qu'un bail est en cours (étape 06)
            let busy: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM leases l JOIN units u ON u.org_id = l.org_id AND u.id = l.unit_id WHERE u.property_id = $1::uuid AND l.status IN ('active', 'notice'))")
                .bind(id.to_string())
                .fetch_one(tx.conn())
                .await?;
            if busy {
                return Err(ServiceError::conflict("unit.has_active_lease"));
            }
            let ctx = audit_context(auth.user.id, client);
            let units: Vec<String> = sqlx::query_scalar(
                "UPDATE units SET archived_at = $2::timestamptz, archived_by = $3::uuid, row_version = row_version + 1 \
                  WHERE property_id = $1::uuid AND archived_at IS NULL RETURNING id::text",
            )
            .bind(id.to_string())
            .bind(&now)
            .bind(auth.user.id.to_string())
            .fetch_all(tx.conn())
            .await?;
            for u in units {
                let uid = parse_id(&u)?;
                audit::record(
                    &mut tx,
                    &ctx,
                    audit_entry(
                        AuditAction::UnitArchived,
                        "unit",
                        uid,
                        None,
                        json!({ "with_property": id.to_string() }),
                    )?,
                )
                .await?;
            }
            sqlx::query(
                "UPDATE properties SET archived_at = $2::timestamptz, archived_by = $3::uuid, row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&now)
            .bind(auth.user.id.to_string())
            .execute(tx.conn())
            .await?;
            audit::record(
                &mut tx,
                &ctx,
                audit_entry(AuditAction::PropertyArchived, "property", id, None, json!({}))?,
            )
            .await?;
        }
        let view = self
            .property_in(&mut tx, id, None)
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    // ───────── lots ─────────

    /// `GET /units`.
    pub async fn list_units(&self, auth: &Authenticated, q: &ListQuery) -> Result<Page<UnitView>, ServiceError> {
        let org = org_of(auth)?;
        if q.usage.as_deref().is_some_and(|u| !USAGES.contains(&u))
            || q.kind.as_deref().is_some_and(|k| !UNIT_KINDS.contains(&k))
            || q.status.as_deref().is_some_and(|s| !UNIT_STATUSES.contains(&s))
        {
            return Err(ServiceError::Invalid("filter_value"));
        }
        let (ck, cid) = read_cursor(q.cursor.as_deref())?;
        let limit = limit_of(q);
        let archived = q.archived || q.status.as_deref() == Some("archived");
        let sql = format!(
            "SELECT {} FROM units u JOIN properties p ON p.id = u.property_id \
              WHERE {} AND (CASE WHEN $2::boolean THEN u.archived_at IS NOT NULL ELSE u.archived_at IS NULL END) \
                AND ($3::text IS NULL OR u.search_key LIKE app_fold($3) ESCAPE '\\' OR p.search_key LIKE app_fold($3) ESCAPE '\\') \
                AND ($4::uuid IS NULL OR u.property_id = $4::uuid) \
                AND ($5::text IS NULL OR u.usage = $5) AND ($6::text IS NULL OR u.kind = $6) \
                AND ($7::text IS NULL OR {UNIT_STATUS_SQL} = $7) \
                AND ($8::text IS NULL OR (u.search_key, u.id::text) > ($8, $9)) \
              ORDER BY u.search_key, u.id::text LIMIT $10",
            unit_columns(),
            unit_scope(1)
        );
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(agent_of(auth))
            .bind(archived)
            .bind(query_pattern(q.q.as_deref()))
            .bind(q.property_id.map(|i| i.to_string()))
            .bind(q.usage.as_deref())
            .bind(q.kind.as_deref())
            .bind(q.status.as_deref())
            .bind(ck)
            .bind(cid)
            .bind(limit.saturating_add(1))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        page(rows, limit, unit_from, |u| u.id)
    }

    /// `GET /units/{id}`.
    pub async fn unit(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<UnitView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = self.unit_in(&mut tx, id, agent_of(auth).as_deref()).await?;
        tx.commit().await?;
        match found {
            Some(u) => Ok(u),
            None => Err(self.trace_and_404(auth, "unit", id, client).await),
        }
    }

    /// `POST /units` (R-011, R-012).
    pub async fn create_unit(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<UnitView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let property = r.uuid("property_id").flatten();
        if property.is_none() && !r.issues.iter().any(|i| i.field == "property_id") {
            r.fail("property_id", "validation.required");
        }
        let f = read_unit_fields(&mut r);
        for (field, present) in [
            ("reference", f.reference.is_some()),
            ("kind", f.kind.is_some()),
            ("usage", f.usage.is_some()),
        ] {
            if !present && !input.contains_key(field) {
                r.fail(field, "validation.required");
            }
        }
        if let Some(usage) = &f.usage {
            check_activity(&mut r, usage, &f.activity.clone().flatten());
        }
        r.finish()?;
        let (Some(pid), Some(reference), Some(kind), Some(usage)) = (property, f.reference, f.kind, f.usage) else {
            return Err(ServiceError::Invalid("unit"));
        };
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(archived) = sqlx::query_scalar::<_, bool>(
            "SELECT archived_at IS NOT NULL FROM properties WHERE id = $1::uuid FOR SHARE",
        )
        .bind(pid.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "property", pid, client).await);
        };
        if archived {
            return Err(ServiceError::conflict("property.archived"));
        }
        let uid = self
            .insert_unit(
                &mut tx,
                pid,
                &reference,
                &kind,
                &usage,
                f.surface.flatten(),
                f.rent.flatten(),
                f.activity.flatten(),
                &now,
                auth,
            )
            .await?;
        let entry = audit_entry(
            AuditAction::UnitCreated,
            "unit",
            uid,
            None,
            json!({ "reference": reference, "property_id": pid.to_string() }),
        )?;
        audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        let view = self.unit_in(&mut tx, uid, None).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `PATCH /units/{id}` (`If-Match`) ; un lot ne change pas de bien.
    pub async fn patch_unit(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<UnitView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let f = read_unit_fields(&mut r);
        r.finish()?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) = sqlx::query(
            "SELECT reference, kind, usage, surface_m2::text, reference_rent, permitted_activity, archived_at IS NOT NULL, row_version \
               FROM units WHERE id = $1::uuid FOR UPDATE",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "unit", id, client).await);
        };
        if cur.get::<bool, _>(6) {
            return Err(ServiceError::conflict("unit.archived"));
        }
        if cur.get::<i32, _>(7) != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let (old_ref, old_kind, old_usage): (String, String, String) = (cur.get(0), cur.get(1), cur.get(2));
        let (old_surface, old_rent, old_activity): (Option<String>, Option<i64>, Option<String>) =
            (cur.get(3), cur.get(4), cur.get(5));
        let new_ref = f.reference.unwrap_or_else(|| old_ref.clone());
        let new_kind = f.kind.unwrap_or_else(|| old_kind.clone());
        let new_usage = f.usage.unwrap_or_else(|| old_usage.clone());
        let new_surface = merged(f.surface, &old_surface);
        let new_rent = merged(f.rent, &old_rent);
        let new_activity = merged(f.activity, &old_activity);
        let mut check = Reader::new(input);
        check_activity(&mut check, &new_usage, &new_activity);
        check.finish()?;
        let (mut before, mut after) = (Map::new(), Map::new());
        diff(&mut before, &mut after, "reference", &old_ref, &new_ref);
        diff(&mut before, &mut after, "kind", &old_kind, &new_kind);
        diff(&mut before, &mut after, "usage", &old_usage, &new_usage);
        let surface_changed = input.contains_key("surface_m2");
        if surface_changed {
            before.insert("surface_m2".into(), json!(old_surface));
            after.insert("surface_m2".into(), json!(new_surface));
        }
        diff(&mut before, &mut after, "reference_rent", &old_rent, &new_rent);
        diff(
            &mut before,
            &mut after,
            "permitted_activity",
            &old_activity,
            &new_activity,
        );
        if !after.is_empty() {
            let updated = sqlx::query(
                "UPDATE units SET reference = $2, kind = $3, usage = $4, surface_m2 = CASE WHEN $8 THEN $5::numeric(8,2) ELSE surface_m2 END, \
                        reference_rent = $6, permitted_activity = $7, row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&new_ref)
            .bind(&new_kind)
            .bind(&new_usage)
            .bind(&new_surface)
            .bind(new_rent)
            .bind(&new_activity)
            .bind(surface_changed)
            .execute(tx.conn())
            .await;
            match updated {
                Ok(_) => {}
                Err(e) if is_unique_violation(&e, "units_reference_ci") => {
                    return Err(ServiceError::conflict("unit.reference_duplicate"));
                }
                Err(e) => return Err(e.into()),
            }
            let entry = audit_entry(
                AuditAction::UnitUpdated,
                "unit",
                id,
                Some(Value::Object(before)),
                Value::Object(after),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self.unit_in(&mut tx, id, None).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /units/{id}/archive` (le refus d'un lot à bail en cours viendra avec les baux, étape 06).
    pub async fn archive_unit(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<UnitView, ServiceError> {
        let org = org_of(auth)?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(archived) =
            sqlx::query_scalar::<_, bool>("SELECT archived_at IS NOT NULL FROM units WHERE id = $1::uuid FOR UPDATE")
                .bind(id.to_string())
                .fetch_optional(tx.conn())
                .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "unit", id, client).await);
        };
        if !archived {
            // R-006 : rien ne s'archive tant qu'un bail est en cours (étape 06)
            let busy: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM leases WHERE unit_id = $1::uuid AND status IN ('active', 'notice'))",
            )
            .bind(id.to_string())
            .fetch_one(tx.conn())
            .await?;
            if busy {
                return Err(ServiceError::conflict("unit.has_active_lease"));
            }
            sqlx::query(
                "UPDATE units SET archived_at = $2::timestamptz, archived_by = $3::uuid, under_works = false, works_since = NULL, \
                        row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&now)
            .bind(auth.user.id.to_string())
            .execute(tx.conn())
            .await?;
            audit::record(
                &mut tx,
                &audit_context(auth.user.id, client),
                audit_entry(AuditAction::UnitArchived, "unit", id, None, json!({}))?,
            )
            .await?;
        }
        let view = self.unit_in(&mut tx, id, None).await?.ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    // ───────── locataires ─────────

    /// `GET /tenants`.
    pub async fn list_tenants(&self, auth: &Authenticated, q: &ListQuery) -> Result<Page<TenantView>, ServiceError> {
        let org = org_of(auth)?;
        let (ck, cid) = read_cursor(q.cursor.as_deref())?;
        let limit = limit_of(q);
        let digits = q.q.as_deref().and_then(phone::search_digits);
        let sql = format!(
            "SELECT {} FROM tenants t \
              WHERE ($1::uuid IS NULL OR EXISTS (SELECT 1 FROM lease_tenants lt JOIN leases l ON l.org_id = lt.org_id AND l.id = lt.lease_id JOIN unit_assignments ua ON ua.unit_id = l.unit_id AND ua.revoked_at IS NULL AND ua.user_id = $1::uuid WHERE lt.tenant_id = t.id AND l.status IN ('active', 'notice'))) AND (CASE WHEN $2::boolean THEN t.archived_at IS NOT NULL ELSE t.archived_at IS NULL END) \
                AND ($3::text IS NULL OR t.search_key LIKE app_fold($3) ESCAPE '\\' OR ($4::text IS NOT NULL AND t.phone LIKE '%' || $4 || '%')) \
                AND ($5::text IS NULL OR (t.search_key, t.id::text) > ($5, $6)) \
              ORDER BY t.search_key, t.id::text LIMIT $7",
            tenant_columns()
        );
        let private = sees_tenant_private(auth);
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(agent_of(auth))
            .bind(q.archived)
            .bind(query_pattern(q.q.as_deref()))
            .bind(digits)
            .bind(ck)
            .bind(cid)
            .bind(limit.saturating_add(1))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        page(rows, limit, |r| tenant_from(r, private), |t| t.id)
    }

    /// `GET /tenants/{id}` (champs masqués selon le rôle, §4.5).
    pub async fn tenant(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<TenantView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = self
            .tenant_in(&mut tx, id, agent_of(auth).as_deref(), sees_tenant_private(auth))
            .await?;
        tx.commit().await?;
        match found {
            Some(t) => Ok(t),
            None => Err(self.trace_and_404(auth, "tenant", id, client).await),
        }
    }

    /// Fiche non archivée qui porte déjà ce numéro (R-017), hors `except`.
    async fn phone_holder(
        &self,
        tx: &mut OrgTx,
        phone: &str,
        except: Option<Uuid>,
    ) -> Result<Option<(String, String)>, ServiceError> {
        let row = sqlx::query(
            "SELECT id::text, last_name, first_name FROM tenants \
              WHERE phone = $1 AND archived_at IS NULL AND ($2::uuid IS NULL OR id <> $2::uuid) LIMIT 1",
        )
        .bind(phone)
        .bind(except.map(|e| e.to_string()))
        .fetch_optional(tx.conn())
        .await?;
        Ok(row.map(|r| {
            let (id, last, first): (String, String, Option<String>) = (r.get(0), r.get(1), r.get(2));
            (id, display_name(first.as_deref(), &last))
        }))
    }

    fn duplicate(holder: (String, String)) -> ServiceError {
        ServiceError::Conflict {
            code: "tenant.phone_duplicate",
            params: Some(json!({ "conflict": { "id": holder.0, "display_name": holder.1 } })),
        }
    }

    /// Le doublon trouvé après une violation d'unicité (insertion concurrente) : relu dans une transaction neuve.
    async fn duplicate_after_race(&self, auth: &Authenticated, phone: &str, except: Option<Uuid>) -> ServiceError {
        let Some(org) = auth.org.as_ref() else {
            return ServiceError::Internal;
        };
        let found = async {
            let mut tx = self.db.begin(org.id, auth.user.id).await?;
            let h = self.phone_holder(&mut tx, phone, except).await?;
            tx.commit().await?;
            Ok::<_, ServiceError>(h)
        }
        .await;
        match found {
            Ok(Some(h)) => Self::duplicate(h),
            Ok(None) => ServiceError::conflict("tenant.phone_duplicate"),
            Err(e) => e,
        }
    }

    /// `POST /tenants` (R-017, R-018 ; pièce d'identité refusée par l'API : champ inconnu).
    pub async fn create_tenant(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<TenantView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let kind = r
            .choice("kind", &PARTY_KINDS)
            .flatten()
            .unwrap_or_else(|| "person".to_string());
        let last = r.required("last_name", 1, 120, true);
        let first = r.text("first_name", 120).flatten();
        if kind == "company" && first.is_some() {
            r.fail("first_name", "validation.not_applicable");
        }
        let trade = r.text("trade_activity", 200).flatten();
        let phone_v = r.phone("phone").flatten();
        let whatsapp = r.boolean("phone_is_whatsapp").unwrap_or(false);
        if whatsapp && phone_v.is_none() {
            r.fail("phone_is_whatsapp", "validation.invalid");
        }
        let email = r.email("email").flatten();
        let backup = r.object("backup_contact").flatten();
        let (bname, bphone) = match &backup {
            Some(m) => {
                let mut br = Reader::new(m);
                let n = br.text("name", 120).flatten();
                let p = br.phone("phone").flatten();
                for i in br.issues {
                    r.fail(
                        if i.field == "phone" {
                            "backup_contact.phone"
                        } else {
                            "backup_contact.name"
                        },
                        i.code,
                    );
                }
                (n, p)
            }
            None => (None, None),
        };
        let notes = r.text("notes", 4000).flatten();
        r.finish()?;
        let last = last.ok_or(ServiceError::Invalid("last_name"))?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        if let Some(p) = &phone_v
            && let Some(h) = self.phone_holder(&mut tx, p, None).await?
        {
            return Err(Self::duplicate(h));
        }
        let inserted: Result<String, sqlx::Error> = sqlx::query_scalar(
            "INSERT INTO tenants (org_id, kind, last_name, first_name, trade_activity, phone, phone_is_whatsapp, email, \
                                  backup_contact_name, backup_contact_phone, notes, created_at, created_by) \
             VALUES (current_org_id(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::timestamptz, $12::uuid) RETURNING id::text",
        )
        .bind(&kind)
        .bind(&last)
        .bind(&first)
        .bind(&trade)
        .bind(&phone_v)
        .bind(whatsapp)
        .bind(&email)
        .bind(&bname)
        .bind(&bphone)
        .bind(&notes)
        .bind(&now)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await;
        let tid = match inserted {
            Ok(id) => parse_id(&id)?,
            Err(e) if is_unique_violation(&e, "tenants_phone_unique") => {
                drop(tx);
                return Err(self
                    .duplicate_after_race(auth, phone_v.as_deref().unwrap_or_default(), None)
                    .await);
            }
            Err(e) => return Err(e.into()),
        };
        audit::record(
            &mut tx,
            &audit_context(auth.user.id, client),
            audit_entry(
                AuditAction::TenantCreated,
                "tenant",
                tid,
                None,
                json!({ "kind": kind, "phone": phone_v }),
            )?,
        )
        .await?;
        let view = self
            .tenant_in(&mut tx, tid, None, sees_tenant_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `PATCH /tenants/{id}` (`If-Match`) ; l'audit garde l'ancien et le nouveau numéro (DP-92).
    pub async fn patch_tenant(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<TenantView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let kind = r.choice("kind", &PARTY_KINDS);
        let last = r.required("last_name", 1, 120, false);
        let first = r.text("first_name", 120);
        let trade = r.text("trade_activity", 200);
        let phone_v = r.phone("phone");
        let whatsapp = r.boolean("phone_is_whatsapp");
        let email = r.email("email");
        let backup = r.object("backup_contact");
        let backup_fields = match &backup {
            None => None,
            Some(None) => Some((None, None)),
            Some(Some(m)) => {
                let mut br = Reader::new(m);
                let n = br.text("name", 120).flatten();
                let p = br.phone("phone").flatten();
                for i in br.issues {
                    r.fail(
                        if i.field == "phone" {
                            "backup_contact.phone"
                        } else {
                            "backup_contact.name"
                        },
                        i.code,
                    );
                }
                Some((n, p))
            }
        };
        let notes = r.text("notes", 4000);
        r.finish()?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) = sqlx::query(
            "SELECT kind, last_name, first_name, trade_activity, phone, phone_is_whatsapp, email, backup_contact_name, backup_contact_phone, \
                    notes, archived_at IS NOT NULL, row_version FROM tenants WHERE id = $1::uuid FOR UPDATE",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "tenant", id, client).await);
        };
        if cur.get::<bool, _>(10) {
            return Err(ServiceError::conflict("tenant.archived"));
        }
        if cur.get::<i32, _>(11) != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let old_kind: String = cur.get(0);
        let old_last: String = cur.get(1);
        let (old_first, old_trade, old_phone): (Option<String>, Option<String>, Option<String>) =
            (cur.get(2), cur.get(3), cur.get(4));
        let old_wa: bool = cur.get(5);
        let (old_email, old_bname, old_bphone, old_notes): (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = (cur.get(6), cur.get(7), cur.get(8), cur.get(9));
        let new_kind = kind.flatten().unwrap_or_else(|| old_kind.clone());
        let new_last = last.unwrap_or_else(|| old_last.clone());
        let new_first = merged(first, &old_first);
        let new_trade = merged(trade, &old_trade);
        let new_phone = merged(phone_v, &old_phone);
        let new_wa = whatsapp.unwrap_or(old_wa) && new_phone.is_some();
        let new_email = merged(email, &old_email);
        let (new_bname, new_bphone) = backup_fields.unwrap_or_else(|| (old_bname.clone(), old_bphone.clone()));
        let new_notes = merged(notes, &old_notes);
        let mut check = Reader::new(input);
        if new_kind == "company" && new_first.is_some() {
            check.fail("first_name", "validation.not_applicable");
        }
        check.finish()?;
        if let Some(p) = &new_phone
            && new_phone != old_phone
            && let Some(h) = self.phone_holder(&mut tx, p, Some(id)).await?
        {
            return Err(Self::duplicate(h));
        }
        let (mut before, mut after) = (Map::new(), Map::new());
        diff(&mut before, &mut after, "kind", &old_kind, &new_kind);
        diff(&mut before, &mut after, "last_name", &old_last, &new_last);
        diff(&mut before, &mut after, "first_name", &old_first, &new_first);
        diff(&mut before, &mut after, "trade_activity", &old_trade, &new_trade);
        diff(&mut before, &mut after, "phone", &old_phone, &new_phone);
        diff(&mut before, &mut after, "phone_is_whatsapp", &old_wa, &new_wa);
        diff(&mut before, &mut after, "email", &old_email, &new_email);
        diff(&mut before, &mut after, "backup_contact_name", &old_bname, &new_bname);
        diff(
            &mut before,
            &mut after,
            "backup_contact_phone",
            &old_bphone,
            &new_bphone,
        );
        if old_notes != new_notes {
            after.insert("notes".into(), json!("modifié"));
        }
        if !after.is_empty() {
            let updated = sqlx::query(
                "UPDATE tenants SET kind = $2, last_name = $3, first_name = $4, trade_activity = $5, phone = $6, phone_is_whatsapp = $7, \
                        email = $8, backup_contact_name = $9, backup_contact_phone = $10, notes = $11, row_version = row_version + 1 \
                  WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&new_kind)
            .bind(&new_last)
            .bind(&new_first)
            .bind(&new_trade)
            .bind(&new_phone)
            .bind(new_wa)
            .bind(&new_email)
            .bind(&new_bname)
            .bind(&new_bphone)
            .bind(&new_notes)
            .execute(tx.conn())
            .await;
            match updated {
                Ok(_) => {}
                Err(e) if is_unique_violation(&e, "tenants_phone_unique") => {
                    drop(tx);
                    return Err(self
                        .duplicate_after_race(auth, new_phone.as_deref().unwrap_or_default(), Some(id))
                        .await);
                }
                Err(e) => return Err(e.into()),
            }
            let entry = audit_entry(
                AuditAction::TenantUpdated,
                "tenant",
                id,
                Some(Value::Object(before)),
                Value::Object(after),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self
            .tenant_in(&mut tx, id, None, sees_tenant_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /tenants/{id}/merge` : la fiche `id` est archivée et pointe vers `into` ; rien n'est recopié (R-017, DP-92).
    pub async fn merge_tenant(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<TenantView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let into = r.uuid("into_id").flatten();
        if into.is_none() && !r.issues.iter().any(|i| i.field == "into_id") {
            r.fail("into_id", "validation.required");
        }
        r.finish()?;
        let into = into.ok_or(ServiceError::Invalid("into_id"))?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        // verrou dans un ordre fixe : deux fusions croisées ne s'interbloquent pas
        let rows = sqlx::query(
            "SELECT id::text, archived_at IS NOT NULL, merged_into_id IS NOT NULL FROM tenants \
              WHERE id IN ($1::uuid, $2::uuid) ORDER BY id FOR UPDATE",
        )
        .bind(id.to_string())
        .bind(into.to_string())
        .fetch_all(tx.conn())
        .await?;
        let state = |wanted: Uuid| {
            rows.iter()
                .find(|r| r.get::<String, _>(0) == wanted.to_string())
                .map(|r| (r.get::<bool, _>(1), r.get::<bool, _>(2)))
        };
        let (Some(source), Some(target)) = (state(id), state(into)) else {
            let missing = if state(id).is_none() { id } else { into };
            drop(tx);
            return Err(self.trace_and_404(auth, "tenant", missing, client).await);
        };
        if id == into || source.0 || source.1 || target.0 {
            return Err(ServiceError::conflict("tenant.merge_invalid"));
        }
        sqlx::query(
            "UPDATE tenants SET merged_into_id = $2::uuid, archived_at = $3::timestamptz, archived_by = $4::uuid, \
                    row_version = row_version + 1 WHERE id = $1::uuid",
        )
        .bind(id.to_string())
        .bind(into.to_string())
        .bind(&now)
        .bind(auth.user.id.to_string())
        .execute(tx.conn())
        .await?;
        audit::record(
            &mut tx,
            &audit_context(auth.user.id, client),
            audit_entry(
                AuditAction::TenantMerged,
                "tenant",
                id,
                None,
                json!({ "into_id": into.to_string() }),
            )?,
        )
        .await?;
        let view = self
            .tenant_in(&mut tx, into, None, sees_tenant_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /tenants/{id}/archive` (le refus d'un locataire à bail en cours viendra avec les baux, étape 06).
    pub async fn archive_tenant(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<TenantView, ServiceError> {
        let org = org_of(auth)?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(archived) =
            sqlx::query_scalar::<_, bool>("SELECT archived_at IS NOT NULL FROM tenants WHERE id = $1::uuid FOR UPDATE")
                .bind(id.to_string())
                .fetch_optional(tx.conn())
                .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "tenant", id, client).await);
        };
        if !archived {
            // R-006 : rien ne s'archive tant qu'un bail est en cours (étape 06)
            let busy: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM lease_tenants lt JOIN leases l ON l.org_id = lt.org_id AND l.id = lt.lease_id WHERE lt.tenant_id = $1::uuid AND l.status IN ('active', 'notice'))")
                .bind(id.to_string())
                .fetch_one(tx.conn())
                .await?;
            if busy {
                return Err(ServiceError::conflict("lease.active_exists"));
            }
            sqlx::query(
                "UPDATE tenants SET archived_at = $2::timestamptz, archived_by = $3::uuid, row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&now)
            .bind(auth.user.id.to_string())
            .execute(tx.conn())
            .await?;
            audit::record(
                &mut tx,
                &audit_context(auth.user.id, client),
                audit_entry(AuditAction::TenantArchived, "tenant", id, None, json!({}))?,
            )
            .await?;
        }
        let view = self
            .tenant_in(&mut tx, id, None, sees_tenant_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `GET /tenants/{id}/id-document` : désactivé au pilote (R-019).
    pub fn tenant_id_document(&self) -> Result<(), ServiceError> {
        Err(ServiceError::FeatureDisabled)
    }

    // ───────── mandants ─────────

    /// `GET /owners` : nom et type pour tous ; téléphone et mode de reversement pour `mandate.write` (DP-92).
    pub async fn list_owners(&self, auth: &Authenticated, q: &ListQuery) -> Result<Page<OwnerView>, ServiceError> {
        let org = org_of(auth)?;
        let (ck, cid) = read_cursor(q.cursor.as_deref())?;
        let limit = limit_of(q);
        let sql = format!(
            "SELECT {} FROM owners o \
              WHERE (CASE WHEN $1::boolean THEN o.archived_at IS NOT NULL ELSE o.archived_at IS NULL END) \
                AND ($2::text IS NULL OR o.search_key LIKE app_fold($2) ESCAPE '\\') \
                AND ($3::text IS NULL OR (o.search_key, o.id::text) > ($3, $4)) \
              ORDER BY o.search_key, o.id::text LIMIT $5",
            owner_columns()
        );
        let private = sees_owner_private(auth);
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let rows = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(q.archived)
            .bind(query_pattern(q.q.as_deref()))
            .bind(ck)
            .bind(cid)
            .bind(limit.saturating_add(1))
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        page(rows, limit, |r| owner_from(r, private), |o| o.id)
    }

    /// `GET /owners/{id}`.
    pub async fn owner(&self, auth: &Authenticated, id: Uuid, client: &ClientInfo) -> Result<OwnerView, ServiceError> {
        let org = org_of(auth)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let found = self.owner_in(&mut tx, id, sees_owner_private(auth)).await?;
        tx.commit().await?;
        match found {
            Some(o) => Ok(o),
            None => Err(self.trace_and_404(auth, "owner", id, client).await),
        }
    }

    /// `POST /owners` (gérant du plan Agence ; le plan Propriétaire est refusé par le droit `mandate.write`).
    pub async fn create_owner(
        &self,
        auth: &Authenticated,
        input: &Input,
        client: &ClientInfo,
    ) -> Result<OwnerView, ServiceError> {
        let org = org_of(auth)?;
        if org.plan == Plan::Owner {
            return Err(ServiceError::PlanUnavailable);
        }
        let mut r = Reader::new(input);
        let kind = r
            .choice("kind", &PARTY_KINDS)
            .flatten()
            .unwrap_or_else(|| "person".to_string());
        let name = r.required("display_name", 2, 160, true);
        let phone_v = r.phone("phone").flatten();
        let payout = r.choice("payout_method", &PAYOUT_METHODS).flatten();
        r.finish()?;
        let name = name.ok_or(ServiceError::Invalid("display_name"))?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let oid: String = sqlx::query_scalar(
            "INSERT INTO owners (org_id, kind, display_name, phone, payout_method, created_at, created_by) \
             VALUES (current_org_id(), $1, $2, $3, $4, $5::timestamptz, $6::uuid) RETURNING id::text",
        )
        .bind(&kind)
        .bind(&name)
        .bind(&phone_v)
        .bind(&payout)
        .bind(&now)
        .bind(auth.user.id.to_string())
        .fetch_one(tx.conn())
        .await?;
        let oid = parse_id(&oid)?;
        audit::record(
            &mut tx,
            &audit_context(auth.user.id, client),
            audit_entry(
                AuditAction::OwnerCreated,
                "owner",
                oid,
                None,
                json!({ "kind": kind, "display_name": name }),
            )?,
        )
        .await?;
        let view = self
            .owner_in(&mut tx, oid, sees_owner_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `PATCH /owners/{id}` (`If-Match`).
    pub async fn patch_owner(
        &self,
        auth: &Authenticated,
        id: Uuid,
        input: &Input,
        if_match: Option<i32>,
        client: &ClientInfo,
    ) -> Result<OwnerView, ServiceError> {
        let org = org_of(auth)?;
        let mut r = Reader::new(input);
        let kind = r.choice("kind", &PARTY_KINDS);
        let name = r.required("display_name", 2, 160, false);
        let phone_v = r.phone("phone");
        let payout = r.choice("payout_method", &PAYOUT_METHODS);
        r.finish()?;
        let expected = if_match.ok_or(ServiceError::VersionRequired)?;
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) = sqlx::query(
            "SELECT kind, display_name, phone, payout_method, archived_at IS NOT NULL, row_version FROM owners WHERE id = $1::uuid FOR UPDATE",
        )
        .bind(id.to_string())
        .fetch_optional(tx.conn())
        .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "owner", id, client).await);
        };
        if cur.get::<bool, _>(4) {
            return Err(ServiceError::conflict("owner.archived"));
        }
        if cur.get::<i32, _>(5) != expected {
            return Err(ServiceError::VersionMismatch);
        }
        let (old_kind, old_name): (String, String) = (cur.get(0), cur.get(1));
        let (old_phone, old_payout): (Option<String>, Option<String>) = (cur.get(2), cur.get(3));
        let new_kind = kind.flatten().unwrap_or_else(|| old_kind.clone());
        let new_name = name.unwrap_or_else(|| old_name.clone());
        let new_phone = merged(phone_v, &old_phone);
        let new_payout = merged(payout, &old_payout);
        let (mut before, mut after) = (Map::new(), Map::new());
        diff(&mut before, &mut after, "kind", &old_kind, &new_kind);
        diff(&mut before, &mut after, "display_name", &old_name, &new_name);
        diff(&mut before, &mut after, "phone", &old_phone, &new_phone);
        diff(&mut before, &mut after, "payout_method", &old_payout, &new_payout);
        if !after.is_empty() {
            sqlx::query(
                "UPDATE owners SET kind = $2, display_name = $3, phone = $4, payout_method = $5, row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&new_kind)
            .bind(&new_name)
            .bind(&new_phone)
            .bind(&new_payout)
            .execute(tx.conn())
            .await?;
            let entry = audit_entry(
                AuditAction::OwnerUpdated,
                "owner",
                id,
                Some(Value::Object(before)),
                Value::Object(after),
            )?;
            audit::record(&mut tx, &audit_context(auth.user.id, client), entry).await?;
        }
        let view = self
            .owner_in(&mut tx, id, sees_owner_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    /// `POST /owners/{id}/archive` : refusé pour la fiche du propriétaire direct, et tant qu'un bien non archivé l'a pour propriétaire.
    pub async fn archive_owner(
        &self,
        auth: &Authenticated,
        id: Uuid,
        client: &ClientInfo,
    ) -> Result<OwnerView, ServiceError> {
        let org = org_of(auth)?;
        let now = micros(self.clock.now());
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let Some(cur) =
            sqlx::query("SELECT is_self, archived_at IS NOT NULL FROM owners WHERE id = $1::uuid FOR UPDATE")
                .bind(id.to_string())
                .fetch_optional(tx.conn())
                .await?
        else {
            drop(tx);
            return Err(self.trace_and_404(auth, "owner", id, client).await);
        };
        if cur.get::<bool, _>(0) {
            return Err(ServiceError::conflict("owner.is_self"));
        }
        if !cur.get::<bool, _>(1) {
            let n: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM properties WHERE default_owner_id = $1::uuid AND archived_at IS NULL",
            )
            .bind(id.to_string())
            .fetch_one(tx.conn())
            .await?;
            if n > 0 {
                return Err(ServiceError::Conflict {
                    code: "owner.has_properties",
                    params: Some(json!({ "count": n })),
                });
            }
            sqlx::query(
                "UPDATE owners SET archived_at = $2::timestamptz, archived_by = $3::uuid, row_version = row_version + 1 WHERE id = $1::uuid",
            )
            .bind(id.to_string())
            .bind(&now)
            .bind(auth.user.id.to_string())
            .execute(tx.conn())
            .await?;
            audit::record(
                &mut tx,
                &audit_context(auth.user.id, client),
                audit_entry(AuditAction::OwnerArchived, "owner", id, None, json!({}))?,
            )
            .await?;
        }
        let view = self
            .owner_in(&mut tx, id, sees_owner_private(auth))
            .await?
            .ok_or(ServiceError::NotFound)?;
        tx.commit().await?;
        Ok(view)
    }

    // ───────── recherche unique ─────────

    /// `GET /search` : huit résultats au plus par groupe, insensible aux accents et à la casse, téléphone avec ou sans `+226`
    /// (API_CONTRACTS.md §2.6). Portée de l'agent terrain appliquée ; fiches archivées exclues.
    pub async fn search(&self, auth: &Authenticated, q: &str) -> Result<SearchResult, ServiceError> {
        let org = org_of(auth)?;
        let text = q.trim();
        let n = text.chars().count();
        if !(2..=80).contains(&n) {
            return Err(ServiceError::Fields(vec![FieldIssue {
                field: "q",
                code: "validation.too_short",
            }]));
        }
        let pattern = format!("%{}%", like_escape(text));
        let digits = phone::search_digits(text);
        let agent = agent_of(auth);
        let mut tx = self.db.begin(org.id, auth.user.id).await?;
        let tenants = sqlx::query(
            "SELECT t.id::text, t.last_name, t.first_name, t.phone FROM tenants t \
              WHERE ($1::uuid IS NULL OR EXISTS (SELECT 1 FROM lease_tenants lt JOIN leases l ON l.org_id = lt.org_id AND l.id = lt.lease_id JOIN unit_assignments ua ON ua.unit_id = l.unit_id AND ua.revoked_at IS NULL AND ua.user_id = $1::uuid WHERE lt.tenant_id = t.id AND l.status IN ('active', 'notice'))) AND t.archived_at IS NULL \
                AND (t.search_key LIKE app_fold($2) ESCAPE '\\' OR ($3::text IS NOT NULL AND t.phone LIKE '%' || $3 || '%')) \
              ORDER BY t.search_key, t.id LIMIT 8",
        )
        .bind(&agent)
        .bind(&pattern)
        .bind(&digits)
        .fetch_all(tx.conn())
        .await?;
        let sql = format!(
            "SELECT u.id::text, u.reference, p.name, {UNIT_STATUS_SQL} FROM units u JOIN properties p ON p.id = u.property_id \
              WHERE {} AND u.archived_at IS NULL \
                AND (u.search_key LIKE app_fold($2) ESCAPE '\\' OR p.search_key LIKE app_fold($2) ESCAPE '\\') \
              ORDER BY p.search_key, u.search_key, u.id LIMIT 8",
            unit_scope(1)
        );
        let units = sqlx::query(sqlx::AssertSqlSafe(sql))
            .bind(&agent)
            .bind(&pattern)
            .fetch_all(tx.conn())
            .await?;
        tx.commit().await?;
        Ok(SearchResult {
            tenants: tenants
                .iter()
                .map(|r| {
                    let (last, first): (String, Option<String>) = (r.get(1), r.get(2));
                    Ok(SearchTenant {
                        id: parse_id(&r.get::<String, _>(0))?,
                        display_name: display_name(first.as_deref(), &last),
                        phone: r.get(3),
                    })
                })
                .collect::<Result<_, ServiceError>>()?,
            units: units
                .iter()
                .map(|r| {
                    Ok(SearchUnit {
                        id: parse_id(&r.get::<String, _>(0))?,
                        label: r.get(1),
                        property_name: r.get(2),
                        status: r.get(3),
                    })
                })
                .collect::<Result<_, ServiceError>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_curseur_fait_l_aller_retour_et_refuse_un_texte_quelconque() {
        let id = Uuid::nil();
        let c = write_cursor("zogona a1", id);
        assert_eq!(
            read_cursor(Some(&c)).ok(),
            Some((Some("zogona a1".to_string()), Some(id.to_string())))
        );
        assert!(read_cursor(Some("zz")).is_err());
        assert!(read_cursor(Some("41")).is_err());
    }

    #[test]
    fn les_caracteres_de_motif_sont_echappes() {
        assert_eq!(like_escape("50%_a\\b"), "50\\%\\_a\\\\b");
    }

    #[test]
    fn nom_affiche() {
        assert_eq!(display_name(Some("Issouf"), "Tapsoba"), "Issouf Tapsoba");
        assert_eq!(
            display_name(None, "Boutique Wend-Panga SARL"),
            "Boutique Wend-Panga SARL"
        );
    }
}
