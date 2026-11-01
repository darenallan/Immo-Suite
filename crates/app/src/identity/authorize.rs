//! Matrice des droits (F02, `API_CONTRACTS.md` §4.6, `CONTEXTE.md`, DP-66) : codée UNE fois, sous forme de données.
//!
//! Refus par défaut : `row` est une correspondance exhaustive, une action sans ligne ne compile pas. Le test
//! `tests/authorize_matrix.rs` compare ces 175 couples au tableau du document d'API, lu dans le document lui-même.

/// Plan d'une organisation (DP-65).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plan {
    Agency,
    Owner,
}

impl Plan {
    pub fn as_str(self) -> &'static str {
        match self {
            Plan::Agency => "agency",
            Plan::Owner => "owner",
        }
    }

    pub fn parse(text: &str) -> Option<Plan> {
        match text {
            "agency" => Some(Plan::Agency),
            "owner" => Some(Plan::Owner),
            _ => None,
        }
    }
}

/// Rôle d'une adhésion (décision n° 45, DP-85) : cinq rôles du plan Agence, deux du plan Propriétaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Manager,
    PropertyManager,
    Cashier,
    FieldAgent,
    Accountant,
    Owner,
    Collaborator,
}

impl Role {
    /// Ordre des colonnes de la matrice : G, Ge, C, A, Co, P, Col.
    pub const ALL: [Role; 7] = [
        Role::Manager,
        Role::PropertyManager,
        Role::Cashier,
        Role::FieldAgent,
        Role::Accountant,
        Role::Owner,
        Role::Collaborator,
    ];

    /// Valeur stockée dans `memberships.role`.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Manager => "manager",
            Role::PropertyManager => "property_manager",
            Role::Cashier => "cashier",
            Role::FieldAgent => "field_agent",
            Role::Accountant => "accountant",
            Role::Owner => "owner",
            Role::Collaborator => "collaborator",
        }
    }

    pub fn parse(text: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.as_str() == text)
    }

    /// Le plan auquel ce rôle appartient (F01 règle 6).
    pub fn plan(self) -> Plan {
        match self {
            Role::Owner | Role::Collaborator => Plan::Owner,
            _ => Plan::Agency,
        }
    }

    /// Durée d'inactivité avant expiration de la session, en minutes (SECURITY.md §3.2, DP-85).
    pub fn idle_minutes(self) -> i64 {
        match self {
            Role::Manager | Role::Owner => 60,
            _ => 30,
        }
    }
}

/// Les 25 actions de la matrice (`API_CONTRACTS.md` §4.6). Énumération fermée : un code inconnu n'existe pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    CatalogRead,
    CatalogWrite,
    PaymentRecord,
    PaymentRead,
    PaymentRequestCorrection,
    PaymentReverse,
    ReceiptIssue,
    ArrearsRead,
    ReminderWrite,
    StatementRead,
    StatementValidate,
    PayoutRecord,
    MandateWrite,
    ExpenseWrite,
    ExpenseImpute,
    ExportRun,
    ImportRun,
    UserManage,
    AuditRead,
    SettingsRead,
    SettingsWrite,
    SupportGrant,
    SearchUse,
    FeedbackCreate,
    EventsCreate,
}

impl Action {
    pub const ALL: [Action; 25] = [
        Action::CatalogRead,
        Action::CatalogWrite,
        Action::PaymentRecord,
        Action::PaymentRead,
        Action::PaymentRequestCorrection,
        Action::PaymentReverse,
        Action::ReceiptIssue,
        Action::ArrearsRead,
        Action::ReminderWrite,
        Action::StatementRead,
        Action::StatementValidate,
        Action::PayoutRecord,
        Action::MandateWrite,
        Action::ExpenseWrite,
        Action::ExpenseImpute,
        Action::ExportRun,
        Action::ImportRun,
        Action::UserManage,
        Action::AuditRead,
        Action::SettingsRead,
        Action::SettingsWrite,
        Action::SupportGrant,
        Action::SearchUse,
        Action::FeedbackCreate,
        Action::EventsCreate,
    ];

    /// Code de droit de `API_CONTRACTS.md` §4.6 (aussi renvoyé dans `permissions[]` de `GET /me`).
    pub fn code(self) -> &'static str {
        match self {
            Action::CatalogRead => "catalog.read",
            Action::CatalogWrite => "catalog.write",
            Action::PaymentRecord => "payment.record",
            Action::PaymentRead => "payment.read",
            Action::PaymentRequestCorrection => "payment.request_correction",
            Action::PaymentReverse => "payment.reverse",
            Action::ReceiptIssue => "receipt.issue",
            Action::ArrearsRead => "arrears.read",
            Action::ReminderWrite => "reminder.write",
            Action::StatementRead => "statement.read",
            Action::StatementValidate => "statement.validate",
            Action::PayoutRecord => "payout.record",
            Action::MandateWrite => "mandate.write",
            Action::ExpenseWrite => "expense.write",
            Action::ExpenseImpute => "expense.impute",
            Action::ExportRun => "export.run",
            Action::ImportRun => "import.run",
            Action::UserManage => "user.manage",
            Action::AuditRead => "audit.read",
            Action::SettingsRead => "settings.read",
            Action::SettingsWrite => "settings.write",
            Action::SupportGrant => "support.grant",
            Action::SearchUse => "search.use",
            Action::FeedbackCreate => "feedback.create",
            Action::EventsCreate => "events.create",
        }
    }
}

/// Pourquoi une action est refusée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenyReason {
    /// `403 auth.forbidden`.
    Forbidden,
    /// `403 plan.feature_unavailable` : action réservée au plan Agence (R-082).
    PlanUnavailable,
}

/// Décision de `authorize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// Limité à ses lots assignés (agent terrain) : le filtre est appliqué par les requêtes (F02 règle 8).
    AllowScoped,
    /// Lecture des seuls éléments utiles au rôle (`settings.read`).
    AllowPartial,
    /// Droit réglable : dépend du réglage nommé (`role_settings`, étape 03). Refusé tant que le réglage n'existe pas.
    NeedsSetting(&'static str),
    Deny(DenyReason),
}

impl Decision {
    /// Vrai si l'action est permise sans consulter aucun réglage. Un droit réglable ne l'est pas (F02 règle 4).
    pub fn is_allowed_without_settings(self) -> bool {
        matches!(self, Decision::Allow | Decision::AllowScoped | Decision::AllowPartial)
    }
}

/// Cellule de la matrice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    /// Oui.
    O,
    /// Non.
    N,
    /// Oui, pour ses lots.
    S,
    /// Droit réglable, avec la clé de réglage.
    Setting(&'static str),
    /// Partiel.
    P,
    /// Action du plan Agence.
    AgencyOnly,
}

use Cell::{AgencyOnly, N, O, P, S};

const RECORD: Cell = Cell::Setting("record_payment");
const EXPORT: Cell = Cell::Setting("export_data");
const IMPORT: Cell = Cell::Setting("import_data");

/// La matrice : une ligne par action, colonnes G, Ge, C, A, Co, P, Col (ordre de `Role::ALL`).
fn row(action: Action) -> [Cell; 7] {
    match action {
        Action::CatalogRead => [O, O, O, S, O, O, O],
        Action::CatalogWrite => [O, O, N, N, N, O, N],
        Action::PaymentRecord => [O, RECORD, O, N, N, O, RECORD],
        Action::PaymentRead => [O, O, O, N, O, O, O],
        Action::PaymentRequestCorrection => [O, O, O, N, N, O, O],
        Action::PaymentReverse => [O, N, N, N, N, O, N],
        Action::ReceiptIssue => [O, O, O, N, N, O, O],
        Action::ArrearsRead => [O, O, O, S, O, O, O],
        Action::ReminderWrite => [O, O, N, S, N, O, O],
        Action::StatementRead => [O, O, N, N, O, AgencyOnly, AgencyOnly],
        Action::StatementValidate => [O, N, N, N, N, AgencyOnly, AgencyOnly],
        Action::PayoutRecord => [O, N, N, N, N, AgencyOnly, AgencyOnly],
        Action::MandateWrite => [O, N, N, N, N, AgencyOnly, AgencyOnly],
        Action::ExpenseWrite => [O, O, N, N, N, O, N],
        Action::ExpenseImpute => [O, N, N, N, N, O, N],
        Action::ExportRun => [O, EXPORT, N, N, O, O, N],
        Action::ImportRun => [O, IMPORT, N, N, N, O, N],
        Action::UserManage => [O, N, N, N, N, O, N],
        Action::AuditRead => [O, N, N, N, O, O, N],
        Action::SettingsRead => [O, P, P, P, P, O, P],
        Action::SettingsWrite => [O, N, N, N, N, O, N],
        Action::SupportGrant => [O, N, N, N, N, O, N],
        Action::SearchUse => [O, O, O, S, O, O, O],
        Action::FeedbackCreate | Action::EventsCreate => [O; 7],
    }
}

/// Décide si `role`, dans une organisation du plan `org_plan`, peut faire `action`. Refus par défaut.
pub fn authorize(role: Role, org_plan: Plan, action: Action) -> Decision {
    // Un rôle hors de son plan ne peut pas exister (F01 règle 6) ; s'il apparaît en base, tout lui est refusé.
    if role.plan() != org_plan {
        return Decision::Deny(DenyReason::Forbidden);
    }
    let Some(index) = Role::ALL.iter().position(|r| *r == role) else {
        return Decision::Deny(DenyReason::Forbidden);
    };
    match row(action).get(index).copied() {
        Some(Cell::O) => Decision::Allow,
        Some(Cell::S) => Decision::AllowScoped,
        Some(Cell::P) => Decision::AllowPartial,
        Some(Cell::Setting(key)) => Decision::NeedsSetting(key),
        Some(Cell::AgencyOnly) => Decision::Deny(DenyReason::PlanUnavailable),
        Some(Cell::N) | None => Decision::Deny(DenyReason::Forbidden),
    }
}

/// Les rôles qui ont l'action sans condition (message « Rôle requis : … » de `auth.forbidden`).
pub fn required_roles(action: Action) -> Vec<Role> {
    Role::ALL
        .into_iter()
        .filter(|r| authorize(*r, r.plan(), action).is_allowed_without_settings())
        .collect()
}

/// Actions qui exigent aussi la confirmation du mot de passe, valable 5 minutes (mécanisme à l'étape 03).
pub fn requires_reauth(action: Action) -> bool {
    matches!(
        action,
        Action::PaymentReverse
            | Action::StatementValidate
            | Action::PayoutRecord
            | Action::SupportGrant
            | Action::UserManage
            | Action::ExportRun
    )
}
