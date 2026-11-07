//! Règles pures de l'échéancier (étape 06 ; F07, F08, DP-94) : date d'échéance (R-098, R-031), périodes à créer (R-027 à R-030,
//! R-099, R-100), conditions en vigueur (R-023, R-104), statut d'une échéance (R-032, R-033, R-095, DP-13, DP-27), dépôt (R-024).
//! Tests d'or E-01, E-02, E-04, E-22, E-35, E-37, E-39, E-40, E-44 ; propriétés P9 et P15.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // tests : les lints de production ne s'y appliquent pas (CODING_RULES.md §3.1)

use chrono::{Datelike, NaiveDate};
use immo_domain::schedule::{
    Amendment, InstallmentStatus, LeaseWindow, days_late, deposit_above_two_months, due_date_for, installment_status,
    periods_to_create, terms_at,
};
use proptest::prelude::*;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

// ───────── date d'échéance (R-098, E-39) ─────────

#[test]
fn e39_jour_31_en_fevrier_tombe_le_dernier_jour() {
    assert_eq!(due_date_for(d(2027, 2, 1), 31), Some(d(2027, 2, 28)));
    assert_eq!(
        due_date_for(d(2028, 2, 1), 31),
        Some(d(2028, 2, 29)),
        "année bissextile"
    );
    assert_eq!(due_date_for(d(2027, 4, 1), 31), Some(d(2027, 4, 30)));
    assert_eq!(due_date_for(d(2027, 3, 1), 5), Some(d(2027, 3, 5)));
    assert_eq!(due_date_for(d(2027, 3, 1), 0), None);
    assert_eq!(due_date_for(d(2027, 3, 1), 32), None);
    assert_eq!(due_date_for(d(2027, 3, 2), 5), None, "une période est le 1er du mois");
}

// ───────── périodes à créer (R-027 à R-030, R-099, R-100) ─────────

fn window(start: NaiveDate, activated: NaiveDate, end: Option<NaiveDate>, due_day: u8) -> LeaseWindow {
    LeaseWindow {
        start_date: start,
        activated_on: activated,
        effective_end: end,
        due_day,
    }
}

#[test]
fn e01_bail_active_le_1er_mars_echeance_du_5_mars() {
    let w = window(d(2027, 3, 1), d(2027, 3, 1), None, 5);
    let p = periods_to_create(&w, d(2027, 3, 1), 30);
    assert_eq!(
        p,
        vec![(d(2027, 3, 1), d(2027, 3, 5))],
        "le 05/04 est au-delà de 30 jours"
    );
    // un mois plus tard, la fenêtre atteint avril
    let p = periods_to_create(&w, d(2027, 3, 10), 30);
    assert_eq!(p, vec![(d(2027, 3, 1), d(2027, 3, 5)), (d(2027, 4, 1), d(2027, 4, 5))]);
}

#[test]
fn e40_premier_mois_du_en_entier_et_r100_bail_ancien_depuis_le_mois_d_activation() {
    // activé le 20 mars : mars est une période (dû en entier, aucun prorata : le montant ne dépend pas du jour)
    let w = window(d(2027, 3, 20), d(2027, 3, 20), None, 5);
    assert_eq!(
        periods_to_create(&w, d(2027, 3, 20), 30)[0],
        (d(2027, 3, 1), d(2027, 3, 5))
    );
    // commencé le 01/01, activé le 10/03 : rien avant mars
    let w = window(d(2027, 1, 1), d(2027, 3, 10), None, 5);
    let first = periods_to_create(&w, d(2027, 3, 10), 30)[0];
    assert_eq!(first.0, d(2027, 3, 1));
    // signé pour avril, activé en mars : première période en avril
    let w = window(d(2027, 4, 1), d(2027, 3, 10), None, 5);
    assert_eq!(
        periods_to_create(&w, d(2027, 3, 10), 30),
        vec![(d(2027, 4, 1), d(2027, 4, 5))]
    );
}

#[test]
fn r030_rien_apres_la_fin_et_le_mois_de_fin_est_du() {
    // fin le 15/04 : avril est dû (DP-94), mai non
    let w = window(d(2027, 3, 1), d(2027, 3, 1), Some(d(2027, 4, 15)), 20);
    let p = periods_to_create(&w, d(2027, 5, 1), 30);
    assert_eq!(
        p.iter().map(|x| x.0).collect::<Vec<_>>(),
        vec![d(2027, 3, 1), d(2027, 4, 1)]
    );
}

#[test]
fn e35_prolongation_les_echeances_suivent_la_nouvelle_fin() {
    let before = window(d(2026, 1, 1), d(2026, 1, 1), Some(d(2026, 12, 31)), 5);
    assert!(
        periods_to_create(&before, d(2027, 1, 10), 30)
            .iter()
            .all(|p| p.0 <= d(2026, 12, 1))
    );
    let after = window(d(2026, 1, 1), d(2026, 1, 1), Some(d(2027, 6, 30)), 5);
    let p = periods_to_create(&after, d(2027, 1, 10), 30);
    assert!(p.contains(&(d(2027, 1, 1), d(2027, 1, 5))));
    assert!(p.contains(&(d(2027, 2, 1), d(2027, 2, 5))));
}

// ───────── conditions en vigueur (R-023, R-104, E-02, E-44) ─────────

#[test]
fn e02_avenant_a_80000_a_partir_d_avril() {
    let a = [Amendment {
        id: 7,
        effective_from: d(2027, 4, 1),
        entered_on: d(2027, 3, 15),
        rent: Some(80_000),
        charges: None,
    }];
    assert_eq!(terms_at(75_000, 0, &a, d(2027, 3, 5)).rent, (75_000, None));
    assert_eq!(terms_at(75_000, 0, &a, d(2027, 4, 5)).rent, (80_000, Some(7)));
    // jour d'échéance 1 : l'avenant « à partir du 01/04 » vaut pour l'échéance du 01/04
    assert_eq!(terms_at(75_000, 0, &a, d(2027, 4, 1)).rent, (80_000, Some(7)));
}

#[test]
fn e44_avenant_saisi_apres_une_echeance_echue() {
    let a = [Amendment {
        id: 9,
        effective_from: d(2026, 12, 1),
        entered_on: d(2026, 12, 15),
        rent: Some(80_000),
        charges: None,
    }];
    assert_eq!(
        terms_at(75_000, 0, &a, d(2026, 12, 5)).rent,
        (75_000, None),
        "décembre reste à 75 000"
    );
    assert_eq!(
        terms_at(75_000, 0, &a, d(2027, 1, 5)).rent,
        (80_000, Some(9)),
        "janvier passe à 80 000"
    );
}

#[test]
fn avenants_successifs_loyer_et_charges_independants() {
    let a = [
        Amendment {
            id: 1,
            effective_from: d(2027, 2, 1),
            entered_on: d(2027, 1, 20),
            rent: None,
            charges: Some(5_000),
        },
        Amendment {
            id: 2,
            effective_from: d(2027, 4, 1),
            entered_on: d(2027, 3, 1),
            rent: Some(90_000),
            charges: None,
        },
    ];
    let t = terms_at(75_000, 0, &a, d(2027, 5, 5));
    assert_eq!(t.rent, (90_000, Some(2)));
    assert_eq!(t.charges, (5_000, Some(1)));
    let t = terms_at(75_000, 0, &a, d(2027, 1, 5));
    assert_eq!((t.rent, t.charges), ((75_000, None), (0, None)));
}

// ───────── statut d'une échéance (R-032, R-033, R-095, E-04, E-37) ─────────

#[test]
fn e37_le_4_a_venir_le_5_due_le_6_en_retard() {
    let due = d(2027, 3, 5);
    assert_eq!(
        installment_status(75_000, 0, due, d(2027, 3, 4)),
        InstallmentStatus::Upcoming
    );
    assert_eq!(
        installment_status(75_000, 0, due, d(2027, 3, 5)),
        InstallmentStatus::Due
    );
    assert_eq!(
        installment_status(75_000, 0, due, d(2027, 3, 6)),
        InstallmentStatus::Late
    );
    assert_eq!(days_late(due, d(2027, 3, 6)), 1);
    assert_eq!(days_late(due, d(2027, 3, 12)), 7);
    assert_eq!(days_late(due, d(2027, 3, 5)), 0);
}

#[test]
fn e04_partielle_avant_j1_en_retard_ensuite_et_payee() {
    let due = d(2027, 3, 5);
    assert_eq!(
        installment_status(75_000, 50_000, due, d(2027, 3, 5)),
        InstallmentStatus::Partial
    );
    assert_eq!(
        installment_status(75_000, 50_000, due, d(2027, 3, 1)),
        InstallmentStatus::Partial
    );
    assert_eq!(
        installment_status(75_000, 50_000, due, d(2027, 3, 6)),
        InstallmentStatus::Late,
        "« en retard » prime"
    );
    assert_eq!(
        installment_status(75_000, 75_000, due, d(2027, 4, 1)),
        InstallmentStatus::Paid
    );
    assert_eq!(
        installment_status(0, 0, due, d(2027, 4, 1)),
        InstallmentStatus::Paid,
        "dû nul (DP-94)"
    );
    assert_eq!(InstallmentStatus::Late.as_str(), "late");
}

// ───────── dépôt de garantie (R-024, E-22, DP-94) ─────────

#[test]
fn e22_depot_superieur_a_deux_mois_en_habitation() {
    assert!(deposit_above_two_months(true, 200_000, 75_000));
    assert!(
        !deposit_above_two_months(true, 150_000, 75_000),
        "exactement deux mois : pas d'avertissement"
    );
    assert!(
        !deposit_above_two_months(false, 200_000, 75_000),
        "bail commercial : R-024 ne s'applique pas"
    );
}

// ───────── propriétés ─────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3_000))]

    /// P15 : la date d'échéance tombe toujours dans le mois de sa période ; les périodes sont uniques, croissantes, au 1er du mois,
    /// dans la fenêtre, et rejouer le calcul donne le même résultat.
    #[test]
    fn p15_periodes(sy in 2024_i32..2030, sm in 1_u32..13, sd in 1_u32..29, act in 0_i64..400, today in 0_i64..800,
                    dd in 1_u8..32, end in proptest::option::of(0_i64..900)) {
        let start = d(sy, sm, sd);
        let activated = start.checked_add_days(chrono::Days::new(u64::try_from(act).unwrap())).unwrap();
        let today = activated.checked_add_days(chrono::Days::new(u64::try_from(today).unwrap())).unwrap();
        let end = end.map(|e| start.checked_add_days(chrono::Days::new(u64::try_from(e).unwrap())).unwrap());
        let w = window(start, activated, end, dd);
        let p = periods_to_create(&w, today, 30);
        prop_assert_eq!(&p, &periods_to_create(&w, today, 30));
        for pair in p.windows(2) { prop_assert!(pair[0].0 < pair[1].0); }
        for (period, due) in &p {
            prop_assert_eq!(period.day(), 1);
            prop_assert_eq!((due.year(), due.month()), (period.year(), period.month()));
            prop_assert!(*due <= today.checked_add_days(chrono::Days::new(30)).unwrap());
            if let Some(e) = end { prop_assert!(*period <= e); }
        }
    }

    /// P9 : le statut ne dépend que du triplet (dû, affecté, aujourd'hui) par rapport à la date d'échéance.
    #[test]
    fn p9_statut(due in 0_i64..1_000_000, alloc in 0_i64..1_000_000, offset in -40_i64..40) {
        let due_date = d(2027, 3, 5);
        let today = if offset >= 0 {
            due_date.checked_add_days(chrono::Days::new(u64::try_from(offset).unwrap())).unwrap()
        } else {
            due_date.checked_sub_days(chrono::Days::new(u64::try_from(-offset).unwrap())).unwrap()
        };
        let s = installment_status(due, alloc, due_date, today);
        let expected = if alloc >= due { InstallmentStatus::Paid }
            else if offset > 0 { InstallmentStatus::Late }
            else if alloc > 0 { InstallmentStatus::Partial }
            else if offset == 0 { InstallmentStatus::Due }
            else { InstallmentStatus::Upcoming };
        prop_assert_eq!(s, expected);
    }
}
