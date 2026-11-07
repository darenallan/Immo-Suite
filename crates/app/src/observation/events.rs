//! Événements d'usage (F39-observation, DP-90, décision n° 28) : liste blanche fermée, propriétés typées, aucune donnée personnelle.
//!
//! Un événement n'existe que s'il est dans `SPECS` : nom, origine (`server` ou `client`), propriétés et leur nature. Une propriété
//! n'est jamais une chaîne libre : écran de la liste, énumération fermée, entier borné, motif étroit (champ, code de règle, modèle de
//! route). Toute valeur de chaîne est en plus refusée si elle contient `@`, une suite de 6 chiffres ou plus, ou `+226` (règle 5).
//!
//! `ProductEvent` ne se construit que par `parse_client` (ce que le front envoie) ou par les constructeurs typés des événements du
//! serveur : tous passent par la même validation. Un événement invalide est écarté entier, jamais à moitié.

// Insertion dans `product_events` : requête non vérifiée du module, comme celles de `audit` (clippy.toml, DP-77).
#![allow(clippy::disallowed_methods)]

use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::platform::db::{DbError, OrgTx};

/// Qui a le droit d'émettre l'événement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Server,
    Client,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Server => "server",
            Origin::Client => "client",
        }
    }
}

/// Nature d'une propriété. Aucune ne laisse passer du texte libre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `S00` (écran hors liste) à `S26`.
    Screen,
    /// Une valeur parmi une liste fermée.
    Choice(&'static [&'static str]),
    /// Entier borné (inclusif).
    Int(i64, i64),
    /// Identifiant de champ de formulaire : `^[a-z][a-z0-9_]{1,40}$`.
    FieldId,
    /// Code de règle `R-123` ou code `domaine.verbe`.
    Code,
    /// Modèle de route : `/api/users/{membership_id}`, jamais une adresse réelle.
    Route,
}

impl Kind {
    pub fn label(self) -> String {
        match self {
            Kind::Screen => "screen".to_string(),
            Kind::Choice(values) => format!("choice({})", values.join("|")),
            Kind::Int(min, max) => format!("int({min}..{max})"),
            Kind::FieldId => "field_id".to_string(),
            Kind::Code => "code".to_string(),
            Kind::Route => "route".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EventSpec {
    pub name: &'static str,
    pub origin: Origin,
    pub props: &'static [(&'static str, Kind)],
}

const WIDTHS: &[&str] = &["mobile", "tablet", "desktop"];
const ROLES: &[&str] = &[
    "manager",
    "property_manager",
    "cashier",
    "field_agent",
    "accountant",
    "owner",
    "collaborator",
];
const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE"];
const CLIENT_ERRORS: &[&str] = &[
    "TypeError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "unhandled_rejection",
    "other",
];
const LOGIN_REASONS: &[&str] = &["bad_credentials", "rate_limited", "totp_failed", "reauth_failed"];
const EXPIRY_REASONS: &[&str] = &["idle", "absolute"];
const RESET_CHANNELS: &[&str] = &["admin_link", "self_service"];
const CATEGORIES: &[&str] = &["bug", "request", "question"];
const DISCARD_REASONS: &[&str] = &["unknown_name", "bad_property", "server_origin", "batch_overflow"];

/// La liste blanche (versionnée dans `tests/contract/events-whitelist.json`). Chaque étape ajoute ses événements ici.
pub const SPECS: &[EventSpec] = &[
    EventSpec {
        name: "screen_viewed",
        origin: Origin::Client,
        props: &[("screen", Kind::Screen), ("width_class", Kind::Choice(WIDTHS))],
    },
    EventSpec {
        name: "validation_error",
        origin: Origin::Client,
        props: &[
            ("screen", Kind::Screen),
            ("field", Kind::FieldId),
            ("rule_code", Kind::Code),
        ],
    },
    EventSpec {
        name: "permission_denied",
        origin: Origin::Server,
        props: &[("action", Kind::Code), ("role", Kind::Choice(ROLES))],
    },
    EventSpec {
        name: "slow_response",
        origin: Origin::Server,
        props: &[
            ("route", Kind::Route),
            ("method", Kind::Choice(METHODS)),
            ("status", Kind::Int(100, 599)),
            ("duration_ms", Kind::Int(0, 3_600_000)),
        ],
    },
    EventSpec {
        name: "client_error",
        origin: Origin::Server,
        props: &[("screen", Kind::Screen), ("code", Kind::Choice(CLIENT_ERRORS))],
    },
    EventSpec {
        name: "login_failed",
        origin: Origin::Server,
        props: &[("reason", Kind::Choice(LOGIN_REASONS))],
    },
    EventSpec {
        name: "session_expired",
        origin: Origin::Server,
        props: &[("reason", Kind::Choice(EXPIRY_REASONS))],
    },
    EventSpec {
        name: "password_reset_requested",
        origin: Origin::Server,
        props: &[("channel", Kind::Choice(RESET_CHANNELS))],
    },
    EventSpec {
        name: "feedback_submitted",
        origin: Origin::Server,
        props: &[("screen", Kind::Screen), ("category", Kind::Choice(CATEGORIES))],
    },
    EventSpec {
        name: "event_discarded",
        origin: Origin::Server,
        props: &[("reason", Kind::Choice(DISCARD_REASONS)), ("count", Kind::Int(1, 1000))],
    },
    // étape 05 (F04 §4.1, DP-92) : la longueur et le groupe choisi, jamais le texte tapé
    EventSpec {
        name: "search_performed",
        origin: Origin::Client,
        props: &[
            ("length", Kind::Int(2, 80)),
            ("group", Kind::Choice(SEARCH_GROUPS)),
            ("empty", Kind::Choice(YES_NO)),
        ],
    },
    // étape 07 (F09 règle 21, DP-96) : le mode et des indicateurs, jamais un montant ni une référence
    EventSpec {
        name: "payment_recorded",
        origin: Origin::Server,
        props: &[
            ("method", Kind::Choice(PAYMENT_METHODS)),
            ("partial", Kind::Choice(YES_NO)),
            ("advance", Kind::Choice(YES_NO)),
            ("credit", Kind::Choice(YES_NO)),
            ("late_entry_days", Kind::Int(0, 3650)),
        ],
    },
    EventSpec {
        name: "payment_idempotent_replay",
        origin: Origin::Server,
        props: &[("route", Kind::Route)],
    },
    EventSpec {
        name: "payment_external_ref_duplicate",
        origin: Origin::Server,
        props: &[("method", Kind::Choice(PAYMENT_METHODS))],
    },
    EventSpec {
        name: "payment_form_opened",
        origin: Origin::Client,
        props: &[("screen", Kind::Screen)],
    },
    EventSpec {
        name: "payment_form_abandoned",
        origin: Origin::Client,
        props: &[("screen", Kind::Screen)],
    },
    // étape 08 (F10 règle 17, DP-98) : format, canal, durée ; jamais un numéro ni un jeton
    EventSpec {
        name: "receipt_pdf_rendered",
        origin: Origin::Server,
        props: &[("paper", Kind::Choice(PAPERS)), ("duration_ms", Kind::Int(0, 3_600_000))],
    },
    EventSpec {
        name: "receipt_pdf_failed",
        origin: Origin::Server,
        props: &[("paper", Kind::Choice(PAPERS))],
    },
    EventSpec {
        name: "receipt_shared",
        origin: Origin::Server,
        props: &[("channel", Kind::Choice(SEND_CHANNELS))],
    },
    EventSpec {
        name: "share_link_opened",
        origin: Origin::Server,
        props: &[("document", Kind::Choice(DOCUMENTS))],
    },
    EventSpec {
        name: "share_link_expired",
        origin: Origin::Server,
        props: &[("document", Kind::Choice(DOCUMENTS))],
    },
];

const PAYMENT_METHODS: &[&str] = &["cash", "mobile_money", "bank_transfer", "cheque"];
const PAPERS: &[&str] = &["a4", "a5"];
const SEND_CHANNELS: &[&str] = &["whatsapp", "print_a4", "print_a5", "link_copied"];
const DOCUMENTS: &[&str] = &["receipt"];

const SEARCH_GROUPS: &[&str] = &["tenants", "units", "receipts", "none"];
const YES_NO: &[&str] = &["yes", "no"];

/// Au plus 20 événements par lot (API_CONTRACTS.md §7.15).
pub const MAX_BATCH: usize = 20;
/// Taille maximale d'un lot, en octets.
pub const MAX_BATCH_BYTES: usize = 8 * 1024;
/// Taille maximale des propriétés sérialisées d'un événement : la moitié de la limite de la base (2 Kio), par prudence.
const MAX_PROPERTIES_BYTES: usize = 1024;

/// Pourquoi un événement est écarté. Jamais le nom reçu : il pourrait contenir une donnée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiscardReason {
    UnknownName,
    BadProperty,
    ServerOrigin,
    BatchOverflow,
}

impl DiscardReason {
    pub fn as_str(self) -> &'static str {
        match self {
            DiscardReason::UnknownName => "unknown_name",
            DiscardReason::BadProperty => "bad_property",
            DiscardReason::ServerOrigin => "server_origin",
            DiscardReason::BatchOverflow => "batch_overflow",
        }
    }
}

/// Événement validé. Les champs sont privés : la seule voie est une validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductEvent {
    spec: &'static EventSpec,
    properties: Map<String, Value>,
}

fn find_spec(name: &str) -> Option<&'static EventSpec> {
    SPECS.iter().find(|s| s.name == name)
}

/// Une valeur de chaîne qui ressemble à une donnée personnelle : `@`, 6 chiffres de suite ou plus, `+226`.
pub fn looks_personal(text: &str) -> bool {
    if text.contains('@') || text.contains("+226") {
        return true;
    }
    let mut run = 0_usize;
    for c in text.chars() {
        if c.is_ascii_digit() {
            run += 1;
            if run >= 6 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn is_field_id(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=41).contains(&b.len())
        && b.first().is_some_and(u8::is_ascii_lowercase)
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

fn is_code(s: &str) -> bool {
    if s.len() > 60 {
        return false;
    }
    // R-123
    if let Some(rest) = s.strip_prefix("R-") {
        return rest.len() == 3 && rest.bytes().all(|c| c.is_ascii_digit());
    }
    // domaine.verbe (ou plus de segments) : lettres minuscules, chiffres et tirets bas
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() >= 2
        && parts.iter().all(|p| {
            p.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && p.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
        })
}

fn is_route(s: &str) -> bool {
    s.len() <= 120
        && s.starts_with('/')
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'/' | b'_' | b'-' | b'{' | b'}'))
}

/// `S00` à `S26`.
pub fn is_screen(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 3
        && b.first() == Some(&b'S')
        && b.get(1..).is_some_and(|d| d.iter().all(u8::is_ascii_digit))
        && s.get(1..).and_then(|d| d.parse::<u8>().ok()).is_some_and(|n| n <= 26)
}

fn valid_value(kind: Kind, value: &Value) -> bool {
    match (kind, value) {
        (Kind::Int(min, max), Value::Number(n)) => n.as_i64().is_some_and(|v| v >= min && v <= max),
        (Kind::Screen, Value::String(s)) => is_screen(s),
        (Kind::Choice(values), Value::String(s)) => values.contains(&s.as_str()),
        (Kind::FieldId, Value::String(s)) => is_field_id(s),
        (Kind::Code, Value::String(s)) => is_code(s),
        (Kind::Route, Value::String(s)) => is_route(s),
        _ => false,
    }
}

fn validate(spec: &'static EventSpec, properties: Map<String, Value>) -> Result<ProductEvent, DiscardReason> {
    if properties.len() != spec.props.len() {
        return Err(DiscardReason::BadProperty);
    }
    if serde_json::to_string(&properties).map_or(true, |s| s.len() > MAX_PROPERTIES_BYTES) {
        return Err(DiscardReason::BadProperty);
    }
    for (name, kind) in spec.props {
        let Some(value) = properties.get(*name) else {
            return Err(DiscardReason::BadProperty);
        };
        if let Value::String(s) = value
            && looks_personal(s)
        {
            return Err(DiscardReason::BadProperty);
        }
        if !valid_value(*kind, value) {
            return Err(DiscardReason::BadProperty);
        }
    }
    Ok(ProductEvent { spec, properties })
}

/// Version du client : `1.2.3` ou `1.2.3-rc.1`, 20 caractères au plus.
pub fn valid_client_version(v: &str) -> bool {
    let (core, suffix) = match v.split_once('-') {
        Some((c, s)) => (c, Some(s)),
        None => (v, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    v.len() <= 20
        && parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 6 && p.bytes().all(|c| c.is_ascii_digit()))
        && suffix.is_none_or(|s| {
            (1..=12).contains(&s.len())
                && s.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'.')
        })
}

/// Un événement tel que le front l'envoie : `{ name, properties, client_version }`.
pub struct ParsedClientEvent {
    pub event: ProductEvent,
    pub client_version: Option<String>,
}

/// Valide un événement reçu du front. Écarté (et compté) si le nom est inconnu, d'origine serveur, ou si une propriété est hors liste.
pub fn parse_client(raw: &Value) -> Result<ParsedClientEvent, DiscardReason> {
    let Some(object) = raw.as_object() else {
        return Err(DiscardReason::BadProperty);
    };
    if object
        .keys()
        .any(|k| !matches!(k.as_str(), "name" | "properties" | "client_version"))
    {
        return Err(DiscardReason::BadProperty);
    }
    let Some(name) = object.get("name").and_then(Value::as_str) else {
        return Err(DiscardReason::UnknownName);
    };
    let Some(spec) = find_spec(name) else {
        return Err(DiscardReason::UnknownName);
    };
    if spec.origin != Origin::Client {
        return Err(DiscardReason::ServerOrigin);
    }
    let properties = match object.get("properties") {
        Some(Value::Object(m)) => m.clone(),
        None => Map::new(),
        Some(_) => return Err(DiscardReason::BadProperty),
    };
    let client_version = match object.get("client_version") {
        None | Some(Value::Null) => None,
        Some(Value::String(v)) if valid_client_version(v) => Some(v.clone()),
        Some(_) => return Err(DiscardReason::BadProperty),
    };
    let event = validate(spec, properties)?;
    Ok(ParsedClientEvent { event, client_version })
}

/// Construction typée d'un événement du serveur. Un échec ici est un défaut du code (test), jamais une donnée : l'appelant l'ignore
/// en meilleur effort.
fn server_event(name: &str, props: Value) -> Option<ProductEvent> {
    let spec = find_spec(name)?;
    let Value::Object(map) = props else { return None };
    validate(spec, map).ok()
}

impl ProductEvent {
    pub fn name(&self) -> &'static str {
        self.spec.name
    }

    pub fn origin(&self) -> Origin {
        self.spec.origin
    }

    pub fn properties(&self) -> &Map<String, Value> {
        &self.properties
    }

    pub fn permission_denied(action_code: &str, role: &str) -> Option<Self> {
        server_event("permission_denied", json!({ "action": action_code, "role": role }))
    }

    pub fn slow_response(route: &str, method: &str, status: u16, duration_ms: u64) -> Option<Self> {
        server_event(
            "slow_response",
            json!({ "route": route, "method": method, "status": status, "duration_ms": duration_ms.min(3_600_000) }),
        )
    }

    pub fn client_error(screen: &str, code: &str) -> Option<Self> {
        server_event("client_error", json!({ "screen": screen, "code": code }))
    }

    pub fn login_failed(reason: &str) -> Option<Self> {
        server_event("login_failed", json!({ "reason": reason }))
    }

    pub fn session_expired(reason: &str) -> Option<Self> {
        server_event("session_expired", json!({ "reason": reason }))
    }

    pub fn password_reset_requested(channel: &str) -> Option<Self> {
        server_event("password_reset_requested", json!({ "channel": channel }))
    }

    pub fn feedback_submitted(screen: &str, category: &str) -> Option<Self> {
        server_event("feedback_submitted", json!({ "screen": screen, "category": category }))
    }

    pub fn payment_recorded(
        method: &str,
        partial: bool,
        advance: bool,
        credit: bool,
        late_entry_days: i64,
    ) -> Option<Self> {
        let yn = |b: bool| if b { "yes" } else { "no" };
        server_event(
            "payment_recorded",
            json!({ "method": method, "partial": yn(partial), "advance": yn(advance), "credit": yn(credit),
                    "late_entry_days": late_entry_days.clamp(0, 3650) }),
        )
    }

    pub fn payment_idempotent_replay() -> Option<Self> {
        server_event("payment_idempotent_replay", json!({ "route": "/api/payments" }))
    }

    pub fn payment_external_ref_duplicate(method: &str) -> Option<Self> {
        server_event("payment_external_ref_duplicate", json!({ "method": method }))
    }

    pub fn receipt_pdf_rendered(paper: &str, duration_ms: u64) -> Option<Self> {
        server_event(
            "receipt_pdf_rendered",
            json!({ "paper": paper, "duration_ms": duration_ms.min(3_600_000) }),
        )
    }

    pub fn receipt_pdf_failed(paper: &str) -> Option<Self> {
        server_event("receipt_pdf_failed", json!({ "paper": paper }))
    }

    pub fn receipt_shared(channel: &str) -> Option<Self> {
        server_event("receipt_shared", json!({ "channel": channel }))
    }

    pub fn share_link_opened() -> Option<Self> {
        server_event("share_link_opened", json!({ "document": "receipt" }))
    }

    pub fn share_link_expired() -> Option<Self> {
        server_event("share_link_expired", json!({ "document": "receipt" }))
    }

    pub fn discarded(reason: DiscardReason, count: u32) -> Option<Self> {
        server_event(
            "event_discarded",
            json!({ "reason": reason.as_str(), "count": count.clamp(1, 1000) }),
        )
    }
}

/// Qui a produit l'événement : pris de la session, jamais du corps de la requête (règle 7).
#[derive(Debug, Clone, Default)]
pub struct EventContext {
    pub user_id: Option<Uuid>,
    pub role: Option<String>,
    /// 16 caractères hexadécimaux de l'empreinte de la session (pseudonyme).
    pub session_ref: Option<String>,
    pub client_version: Option<String>,
}

const INSERT_EVENT: &str = "INSERT INTO product_events (org_id, user_id, role, session_id, name, properties, client_version) \
     VALUES (current_org_id(), $1::uuid, $2, $3, $4, $5::jsonb, $6)";

/// Écrit l'événement dans la transaction de l'organisation. Utilisée par les gestionnaires (étapes suivantes comprises).
pub async fn emit(tx: &mut OrgTx, ctx: &EventContext, event: &ProductEvent) -> Result<(), DbError> {
    sqlx::query(INSERT_EVENT)
        .bind(ctx.user_id.map(|u| u.to_string()))
        .bind(ctx.role.as_deref())
        .bind(ctx.session_ref.as_deref())
        .bind(event.name())
        .bind(Value::Object(event.properties.clone()).to_string())
        .bind(ctx.client_version.as_deref())
        .execute(tx.conn())
        .await?;
    Ok(())
}

/// Résultat du traitement d'un lot reçu du front.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BatchReport {
    pub accepted: usize,
    /// Motif et nombre d'événements écartés, par motif, dans l'ordre des motifs.
    pub discarded: Vec<(DiscardReason, u32)>,
}

impl BatchReport {
    pub fn discarded_total(&self) -> u32 {
        self.discarded.iter().map(|(_, n)| *n).sum()
    }
}

/// Valide un lot : les `MAX_BATCH` premiers sont examinés, les suivants sont écartés (`batch_overflow`).
pub fn parse_batch(events: &[Value]) -> (Vec<ParsedClientEvent>, BatchReport) {
    let mut accepted = vec![];
    let mut counts: std::collections::BTreeMap<DiscardReason, u32> = std::collections::BTreeMap::new();
    for raw in events.iter().take(MAX_BATCH) {
        match parse_client(raw) {
            Ok(e) => accepted.push(e),
            Err(reason) => *counts.entry(reason).or_insert(0) += 1,
        }
    }
    if events.len() > MAX_BATCH {
        let extra = u32::try_from(events.len() - MAX_BATCH).unwrap_or(u32::MAX);
        *counts.entry(DiscardReason::BatchOverflow).or_insert(0) += extra;
    }
    let report = BatchReport {
        accepted: accepted.len(),
        discarded: counts.into_iter().collect(),
    };
    (accepted, report)
}

/// Écrit un lot validé et les lignes `event_discarded` (une par motif) dans une seule transaction.
pub async fn emit_batch(
    tx: &mut OrgTx,
    base: &EventContext,
    accepted: &[ParsedClientEvent],
    report: &BatchReport,
) -> Result<(), DbError> {
    for e in accepted {
        let ctx = EventContext {
            client_version: e.client_version.clone(),
            ..base.clone()
        };
        emit(tx, &ctx, &e.event).await?;
    }
    for (reason, count) in &report.discarded {
        if let Some(event) = ProductEvent::discarded(*reason, *count) {
            emit(tx, base, &event).await?;
        }
    }
    Ok(())
}

/// La liste blanche au format JSON versionné (`tests/contract/events-whitelist.json`) : un test la régénère et la compare.
pub fn whitelist_json() -> String {
    let events: Vec<Value> = SPECS
        .iter()
        .map(|s| {
            let props: Map<String, Value> = s
                .props
                .iter()
                .map(|(name, kind)| ((*name).to_string(), Value::String(kind.label())))
                .collect();
            json!({ "name": s.name, "origin": s.origin.as_str(), "properties": props })
        })
        .collect();
    let mut text = serde_json::to_string_pretty(&json!({
        "version": 1,
        "max_batch": MAX_BATCH,
        "max_batch_bytes": MAX_BATCH_BYTES,
        "forbidden_in_values": ["@", "+226", "6 chiffres de suite ou plus"],
        "events": events
    }))
    .unwrap_or_default();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(v: Value) -> ParsedClientEvent {
        parse_client(&v).unwrap()
    }

    #[test]
    fn un_evenement_du_front_valide_est_accepte() {
        let e = ok(
            json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "mobile" }, "client_version": "1.2.3" }),
        );
        assert_eq!(e.event.name(), "screen_viewed");
        assert_eq!(e.client_version.as_deref(), Some("1.2.3"));
        let e = ok(
            json!({ "name": "validation_error", "properties": { "screen": "S09", "field": "amount", "rule_code": "R-041" } }),
        );
        assert_eq!(e.event.name(), "validation_error");
    }

    #[test]
    fn nom_inconnu_origine_serveur_et_propriete_hors_liste_sont_ecartes() {
        let bad = |v: Value| parse_client(&v).err();
        assert_eq!(
            bad(json!({ "name": "payment_hacked", "properties": {} })),
            Some(DiscardReason::UnknownName)
        );
        assert_eq!(bad(json!({ "properties": {} })), Some(DiscardReason::UnknownName));
        assert_eq!(
            bad(json!({ "name": "login_failed", "properties": { "reason": "bad_credentials" } })),
            Some(DiscardReason::ServerOrigin),
            "le front ne fabrique pas un événement du serveur"
        );
        // propriété en trop, manquante, mal typée, hors liste fermée
        assert_eq!(
            bad(
                json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "mobile", "name": "x" } })
            ),
            Some(DiscardReason::BadProperty)
        );
        assert_eq!(
            bad(json!({ "name": "screen_viewed", "properties": { "screen": "S05" } })),
            Some(DiscardReason::BadProperty)
        );
        assert_eq!(
            bad(json!({ "name": "screen_viewed", "properties": { "screen": 5, "width_class": "mobile" } })),
            Some(DiscardReason::BadProperty)
        );
        assert_eq!(
            bad(json!({ "name": "screen_viewed", "properties": { "screen": "S99", "width_class": "mobile" } })),
            Some(DiscardReason::BadProperty)
        );
        assert_eq!(
            bad(json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "huge" } })),
            Some(DiscardReason::BadProperty)
        );
        // champ d'événement inconnu, version illisible
        assert_eq!(
            bad(
                json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "mobile" }, "user": "x" })
            ),
            Some(DiscardReason::BadProperty)
        );
        assert_eq!(
            bad(
                json!({ "name": "screen_viewed", "properties": { "screen": "S05", "width_class": "mobile" }, "client_version": "v1" })
            ),
            Some(DiscardReason::BadProperty)
        );
    }

    #[test]
    fn une_valeur_personnelle_est_refusee_meme_si_son_type_est_une_chaine() {
        for personal in ["Traoré", "+22670123456", "70123456", "a@b.bf", "ref 123456"] {
            let result = parse_client(&json!({
                "name": "validation_error",
                "properties": { "screen": "S09", "field": personal, "rule_code": "R-041" }
            }));
            assert_eq!(result.err(), Some(DiscardReason::BadProperty), "{personal}");
        }
        assert!(looks_personal("+226"));
        assert!(looks_personal("123456"));
        assert!(!looks_personal("12345 6"), "6 chiffres de suite seulement");
        assert!(!looks_personal("amount_2"));
        assert!(!looks_personal("12345"));
    }

    #[test]
    fn les_constructeurs_du_serveur_valident_comme_le_front() {
        assert!(ProductEvent::permission_denied("payment.reverse", "cashier").is_some());
        assert!(ProductEvent::permission_denied("payment.reverse", "pirate").is_none());
        assert!(ProductEvent::slow_response("/api/users/{membership_id}", "GET", 200, 2500).is_some());
        assert!(ProductEvent::slow_response("/api/users/70123456", "GET", 200, 2500).is_none());
        assert!(ProductEvent::client_error("S05", "TypeError").is_some());
        assert!(ProductEvent::login_failed("bad_credentials").is_some());
        assert!(ProductEvent::session_expired("idle").is_some());
        assert!(ProductEvent::password_reset_requested("admin_link").is_some());
        assert!(ProductEvent::feedback_submitted("S12", "bug").is_some());
        assert!(ProductEvent::discarded(DiscardReason::UnknownName, 3).is_some());
    }

    #[test]
    fn lot_de_vingt_et_un_le_vingt_et_unieme_est_ecarte_et_compte() {
        let one = json!({ "name": "screen_viewed", "properties": { "screen": "S01", "width_class": "mobile" } });
        let batch: Vec<Value> = std::iter::repeat_n(one, 21).collect();
        let (accepted, report) = parse_batch(&batch);
        assert_eq!(accepted.len(), 20);
        assert_eq!(report.discarded, vec![(DiscardReason::BatchOverflow, 1)]);
    }

    #[test]
    fn aucune_propriete_de_la_liste_blanche_n_est_une_chaine_libre() {
        for spec in SPECS {
            assert!(!spec.props.is_empty());
            for (name, kind) in spec.props {
                assert!(
                    matches!(
                        kind,
                        Kind::Screen | Kind::Choice(_) | Kind::Int(..) | Kind::FieldId | Kind::Code | Kind::Route
                    ),
                    "{} / {name}",
                    spec.name
                );
            }
        }
        assert!(
            whitelist_json().contains("\"screen_viewed\""),
            "la liste blanche s'exporte"
        );
    }

    #[test]
    fn versions_du_client() {
        for v in ["1.2.3", "0.0.1", "10.20.30-rc.1"] {
            assert!(valid_client_version(v), "{v}");
        }
        for v in [
            "",
            "1.2",
            "v1.2.3",
            "1.2.3.4",
            "1.2.3-RC",
            "1.2.3-aaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert!(!valid_client_version(v), "{v}");
        }
    }
}
