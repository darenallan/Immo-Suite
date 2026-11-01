//! Test A3 généré (F02, critère 7 de l'étape 02) : 25 actions × 7 rôles = 175 couples, comparés au tableau de
//! `API_CONTRACTS.md` §4.6 lu dans le document lui-même (pas recopié). Refus par défaut ; plan Agence ; marque « réauth ».

// Fichier de test : les échecs de lecture du document d'API doivent faire échouer le test avec un message clair.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use immo_app::identity::authorize::{Action, Decision, DenyReason, Plan, Role, authorize, requires_reauth};

const API_CONTRACTS: &str = include_str!("../../../API_CONTRACTS.md");

/// Cellule attendue, lue dans le document.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Expected {
    Yes,
    No,
    Scoped,
    Setting,
    Partial,
    PlanOnly,
}

fn is_read_action(code: &str) -> bool {
    code.ends_with(".read") || code == "search.use"
}

fn strip_notes(cell: &str) -> String {
    cell.chars()
        .filter(|c| !matches!(c, '¹' | '²' | '³' | '⁴'))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Les sept cellules (G, Ge, C, A, Co, P, Col) d'une ligne du tableau.
fn parse_cells(code: &str, cells: &[&str]) -> Vec<Expected> {
    cells
        .iter()
        .enumerate()
        .map(|(i, raw)| {
            let cell = strip_notes(raw);
            match cell.as_str() {
                "oui" => Expected::Yes,
                "non" | "proposer" => Expected::No,
                "*" => Expected::Setting,
                "(s)" => Expected::Scoped,
                "—" => Expected::PlanOnly,
                "partiel" => Expected::Partial,
                // « (L) » : autorisé pour une action de lecture, refusé pour une action d'écriture (F02 règle 5)
                "(L)" => {
                    if is_read_action(code) {
                        Expected::Yes
                    } else {
                        Expected::No
                    }
                }
                // cellule réduite à une note ¹ : le collaborateur demande une correction (DP-66, F02 §3)
                "" if code == "payment.request_correction" && i == 6 => Expected::Yes,
                other => panic!("cellule illisible pour {code} : « {other} » (brut : « {raw} »)"),
            }
        })
        .collect()
}

/// (action, 7 cellules) pour chaque ligne du tableau « Droits par action ».
fn expected_table() -> Vec<(String, Vec<Expected>)> {
    let start = API_CONTRACTS.find("### 4.6 Droits par action").expect("section 4.6");
    let end = API_CONTRACTS[start..].find("### 4.7").expect("section 4.7") + start;
    let mut rows = vec![];
    for line in API_CONTRACTS[start..end].lines().filter(|l| l.starts_with("| `")) {
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        // cols : ["", « `code` : description », G, Ge, C, A, Co, P, Col, source, ""]
        if cols[1].contains("feedback.create") {
            for code in ["feedback.create", "events.create"] {
                rows.push((code.to_string(), vec![Expected::Yes; 7]));
            }
            continue;
        }
        let code = cols[1]
            .trim_start_matches('`')
            .split('`')
            .next()
            .expect("code")
            .to_string();
        assert!(cols.len() >= 10, "ligne mal formée : {line}");
        rows.push((code.clone(), parse_cells(&code, &cols[2..9])));
    }
    rows
}

fn action_by_code(code: &str) -> Action {
    *Action::ALL
        .iter()
        .find(|a| a.code() == code)
        .unwrap_or_else(|| panic!("le document cite l'action {code}, absente de l'énumération"))
}

#[test]
fn le_document_contient_25_actions() {
    let table = expected_table();
    assert_eq!(
        table.len(),
        25,
        "lignes lues : {:?}",
        table.iter().map(|r| &r.0).collect::<Vec<_>>()
    );
    assert_eq!(Action::ALL.len(), 25);
    assert_eq!(Role::ALL.len(), 7);
}

#[test]
fn a3_les_175_couples_role_action_sont_conformes_a_la_matrice() {
    let table = expected_table();
    // complétude : toute action de l'énumération est dans le document, et inversement
    for action in Action::ALL {
        assert!(
            table.iter().any(|(c, _)| c == action.code()),
            "{} absente du document",
            action.code()
        );
    }
    let mut checked = 0;
    let mut mismatches = vec![];
    for (code, cells) in &table {
        let action = action_by_code(code);
        for (role, expected) in Role::ALL.iter().zip(cells) {
            let decision = authorize(*role, role.plan(), action);
            let ok = match expected {
                Expected::Yes => decision == Decision::Allow,
                Expected::No => decision == Decision::Deny(DenyReason::Forbidden),
                Expected::Scoped => decision == Decision::AllowScoped,
                Expected::Setting => matches!(decision, Decision::NeedsSetting(_)),
                Expected::Partial => decision == Decision::AllowPartial,
                Expected::PlanOnly => decision == Decision::Deny(DenyReason::PlanUnavailable),
            };
            if !ok {
                mismatches.push(format!("{code} × {role:?} : attendu {expected:?}, obtenu {decision:?}"));
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 175);
    assert!(
        mismatches.is_empty(),
        "{} écart(s) :\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn les_droits_reglables_sont_refuses_tant_que_role_settings_n_existe_pas() {
    // clés de réglage de API_CONTRACTS.md §7.3 : record_payment, import_data, export_data
    assert_eq!(
        authorize(Role::PropertyManager, Plan::Agency, Action::PaymentRecord),
        Decision::NeedsSetting("record_payment")
    );
    assert_eq!(
        authorize(Role::Collaborator, Plan::Owner, Action::PaymentRecord),
        Decision::NeedsSetting("record_payment")
    );
    assert_eq!(
        authorize(Role::PropertyManager, Plan::Agency, Action::ExportRun),
        Decision::NeedsSetting("export_data")
    );
    assert_eq!(
        authorize(Role::PropertyManager, Plan::Agency, Action::ImportRun),
        Decision::NeedsSetting("import_data")
    );
    // sans réglage connu, un droit réglable est refusé
    for (role, plan, action) in [
        (Role::PropertyManager, Plan::Agency, Action::PaymentRecord),
        (Role::Collaborator, Plan::Owner, Action::PaymentRecord),
        (Role::PropertyManager, Plan::Agency, Action::ExportRun),
        (Role::PropertyManager, Plan::Agency, Action::ImportRun),
    ] {
        assert!(
            !authorize(role, plan, action).is_allowed_without_settings(),
            "{role:?} {action:?}"
        );
    }
}

#[test]
fn les_actions_du_plan_agence_sont_indisponibles_pour_le_plan_proprietaire() {
    let agency_only = [
        Action::StatementRead,
        Action::StatementValidate,
        Action::PayoutRecord,
        Action::MandateWrite,
    ];
    let mut n = 0;
    for action in agency_only {
        for role in [Role::Owner, Role::Collaborator] {
            assert_eq!(
                authorize(role, Plan::Owner, action),
                Decision::Deny(DenyReason::PlanUnavailable),
                "{role:?} {action:?}"
            );
            n += 1;
        }
    }
    assert_eq!(n, 8);
}

#[test]
fn un_role_hors_de_son_plan_est_refuse_partout() {
    // rôle de l'agence dans une organisation du plan Propriétaire, et inversement : impossible à créer, refusé si présent
    for action in Action::ALL {
        assert_eq!(
            authorize(Role::Manager, Plan::Owner, action),
            Decision::Deny(DenyReason::Forbidden),
            "{action:?}"
        );
        assert_eq!(
            authorize(Role::Owner, Plan::Agency, action),
            Decision::Deny(DenyReason::Forbidden),
            "{action:?}"
        );
    }
}

#[test]
fn les_actions_a_confirmation_par_mot_de_passe_sont_celles_de_l_api() {
    let marked: Vec<&str> = Action::ALL
        .iter()
        .filter(|a| requires_reauth(**a))
        .map(|a| a.code())
        .collect();
    let mut expected = vec![
        "payment.reverse",
        "statement.validate",
        "payout.record",
        "support.grant",
        "user.manage",
        "export.run",
    ];
    expected.sort_unstable();
    let mut got = marked;
    got.sort_unstable();
    assert_eq!(got, expected);
}

#[test]
fn les_codes_d_action_sont_uniques_et_au_format_objet_verbe() {
    let mut seen = std::collections::HashSet::new();
    for a in Action::ALL {
        assert!(seen.insert(a.code()), "code en double : {}", a.code());
        assert!(
            a.code().split('.').count() == 2
                && a.code().chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
            "{}",
            a.code()
        );
    }
}

#[test]
fn les_noms_de_role_correspondent_a_la_base() {
    let names: Vec<&str> = Role::ALL.iter().map(|r| r.as_str()).collect();
    assert_eq!(
        names,
        [
            "manager",
            "property_manager",
            "cashier",
            "field_agent",
            "accountant",
            "owner",
            "collaborator"
        ]
    );
    for r in Role::ALL {
        assert_eq!(Role::parse(r.as_str()), Some(r));
    }
    assert_eq!(Role::parse("admin"), None);
}
