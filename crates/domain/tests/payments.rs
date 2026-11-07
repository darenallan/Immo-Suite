//! Étape 07 : règles d'argent de l'encaissement (F09, F10, DP-96), écrites avant le code.
//! Cas d'or E-05, E-06, E-11, E-31, E-32 ; ordre R-043, R-087 ; propriétés P1, P2, P4, P7, P14 ; numéro DP-37, DP-61.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use chrono::NaiveDate;
use immo_domain::allocation::{self, AllocationError, LineKind, OpenLine, Part};
use immo_domain::receipt;
use immo_domain::words;
use proptest::prelude::*;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn line(id: u32, due: NaiveDate, kind: LineKind, remaining: i64) -> OpenLine<u32> {
    OpenLine {
        id,
        due_date: due,
        kind,
        remaining,
    }
}

fn today() -> NaiveDate {
    d(2026, 11, 13)
}

// ───────── E-05, E-32 : partiel et ordre loyer, charges ─────────

#[test]
fn e05_partiel_50000_sur_un_loyer_de_75000() {
    let lines = [line(1, d(2026, 11, 5), LineKind::Rent, 75_000)];
    let s = allocation::allocate(50_000, &lines, today()).unwrap();
    assert_eq!(
        s.parts,
        vec![Part {
            line: 1,
            amount: 50_000
        }]
    );
    assert_eq!((s.advance, s.credit), (0, 0));
    assert_eq!(s.remaining_due_after, 25_000);
}

#[test]
fn e32_loyer_puis_charges_a_date_egale() {
    let lines = [
        line(2, d(2026, 11, 5), LineKind::Charges, 10_000),
        line(1, d(2026, 11, 5), LineKind::Rent, 75_000),
    ];
    let s = allocation::allocate(50_000, &lines, today()).unwrap();
    assert_eq!(
        s.parts,
        vec![Part {
            line: 1,
            amount: 50_000
        }]
    );
    assert_eq!(s.remaining_due_after, 35_000, "25 000 de loyer et 10 000 de charges");
}

// ───────── E-06, E-31 : avance et crédit du bail ─────────

#[test]
fn e06_avance_sur_l_echeance_suivante_deja_generee() {
    let lines = [
        line(1, d(2026, 11, 13), LineKind::Rent, 75_000),
        line(2, d(2026, 12, 13), LineKind::Rent, 75_000),
    ];
    let s = allocation::allocate(100_000, &lines, today()).unwrap();
    assert_eq!(
        s.parts,
        vec![
            Part {
                line: 1,
                amount: 75_000
            },
            Part {
                line: 2,
                amount: 25_000
            }
        ]
    );
    assert_eq!((s.advance, s.credit, s.remaining_due_after), (25_000, 0, 0));
    assert!(s.requires_surplus_confirmation());
}

#[test]
fn e31_sans_echeance_suivante_le_reste_est_credit_puis_s_affecte_a_la_generation() {
    let lines = [line(1, d(2026, 11, 13), LineKind::Rent, 75_000)];
    let s = allocation::allocate(100_000, &lines, today()).unwrap();
    assert_eq!(
        s.parts,
        vec![Part {
            line: 1,
            amount: 75_000
        }]
    );
    assert_eq!((s.advance, s.credit), (25_000, 25_000));
    // génération de l'échéance suivante : le crédit du paiement 7 s'y affecte, reste dû 50 000
    let new_lines = [line(9, d(2026, 12, 13), LineKind::Rent, 75_000)];
    let parts = allocation::apply_credit(&[(7_u32, 25_000)], &new_lines);
    assert_eq!(
        parts,
        vec![(
            7,
            Part {
                line: 9,
                amount: 25_000
            }
        )]
    );
}

#[test]
fn le_credit_consomme_le_paiement_le_plus_ancien_d_abord_et_ne_depasse_pas_les_lignes() {
    let new_lines = [
        line(9, d(2026, 12, 5), LineKind::Rent, 30_000),
        line(10, d(2026, 12, 5), LineKind::Charges, 5_000),
    ];
    let parts = allocation::apply_credit(&[(1_u32, 20_000), (2_u32, 40_000)], &new_lines);
    assert_eq!(
        parts,
        vec![
            (
                1,
                Part {
                    line: 9,
                    amount: 20_000
                }
            ),
            (
                2,
                Part {
                    line: 9,
                    amount: 10_000
                }
            ),
            (
                2,
                Part {
                    line: 10,
                    amount: 5_000
                }
            ),
        ]
    );
}

#[test]
fn montant_exactement_egal_au_reste_du_aucune_confirmation() {
    let lines = [
        line(1, d(2026, 10, 5), LineKind::Rent, 75_000),
        line(2, d(2026, 11, 5), LineKind::Rent, 75_000),
    ];
    let s = allocation::allocate(150_000, &lines, today()).unwrap();
    assert_eq!((s.advance, s.credit, s.remaining_due_after), (0, 0, 0));
    assert!(!s.requires_surplus_confirmation());
}

#[test]
fn rien_d_ouvert_tout_le_montant_est_avance_en_credit() {
    let s = allocation::allocate::<u32>(30_000, &[], today()).unwrap();
    assert!(s.parts.is_empty());
    assert_eq!((s.advance, s.credit), (30_000, 30_000));
}

// ───────── R-087, DP-96 : ordre ─────────

#[test]
fn ordre_date_puis_loyer_charges_exceptionnelle_arriere_puis_identifiant() {
    let lines = [
        line(6, d(2026, 11, 5), LineKind::Arrears, 1),
        line(5, d(2026, 11, 5), LineKind::Exceptional, 1),
        line(4, d(2026, 11, 5), LineKind::Charges, 1),
        line(8, d(2026, 11, 5), LineKind::Rent, 1),
        line(3, d(2026, 11, 5), LineKind::Rent, 1),
        line(2, d(2026, 9, 30), LineKind::Arrears, 1),
    ];
    let ids: Vec<u32> = allocation::ordered(&lines).iter().map(|l| l.id).collect();
    assert_eq!(ids, vec![2, 3, 8, 4, 5, 6]);
}

// ───────── affectation ajustée (F09 règle 11) ─────────

#[test]
fn affectation_ajustee_valide_et_refus() {
    let lines = [
        line(1, d(2026, 10, 5), LineKind::Rent, 75_000),
        line(2, d(2026, 11, 5), LineKind::Rent, 75_000),
    ];
    // payer novembre plutôt qu'octobre : accepté, rien à confirmer
    let s = allocation::check_manual(
        75_000,
        &lines,
        &[Part {
            line: 2,
            amount: 75_000,
        }],
        today(),
    )
    .unwrap();
    assert_eq!((s.advance, s.credit, s.remaining_due_after), (0, 0, 75_000));
    // ligne absente (autre bail, annulée ou soldée)
    assert_eq!(
        allocation::check_manual(10, &lines, &[Part { line: 3, amount: 10 }], today()),
        Err(AllocationError::UnknownLine)
    );
    // au-delà du reste de la ligne
    assert_eq!(
        allocation::check_manual(
            80_000,
            &lines,
            &[Part {
                line: 1,
                amount: 80_000
            }],
            today()
        ),
        Err(AllocationError::AboveLineRemaining)
    );
    // somme supérieure au paiement (R-044)
    assert_eq!(
        allocation::check_manual(
            50_000,
            &lines,
            &[
                Part {
                    line: 1,
                    amount: 40_000
                },
                Part {
                    line: 2,
                    amount: 20_000
                }
            ],
            today()
        ),
        Err(AllocationError::AboveAmount)
    );
    // part nulle, ou deux parts sur la même ligne
    assert_eq!(
        allocation::check_manual(10, &lines, &[Part { line: 1, amount: 0 }], today()),
        Err(AllocationError::InvalidPart)
    );
    assert_eq!(
        allocation::check_manual(
            20,
            &lines,
            &[Part { line: 1, amount: 10 }, Part { line: 1, amount: 10 }],
            today()
        ),
        Err(AllocationError::InvalidPart)
    );
    // reste non affecté : crédit, avec confirmation d'avance
    let s = allocation::check_manual(
        100_000,
        &lines,
        &[Part {
            line: 1,
            amount: 75_000,
        }],
        today(),
    )
    .unwrap();
    assert_eq!((s.advance, s.credit), (25_000, 25_000));
}

#[test]
fn montant_hors_bornes_refuse() {
    assert_eq!(
        allocation::allocate::<u32>(0, &[], today()),
        Err(AllocationError::Amount)
    );
    assert_eq!(
        allocation::allocate::<u32>(1_000_000_000, &[], today()),
        Err(AllocationError::Amount)
    );
    assert_eq!(
        allocation::allocate::<u32>(-5, &[], today()),
        Err(AllocationError::Amount)
    );
}

// ───────── propriétés P1, P2, P4, P7 ─────────

fn kind_of(n: u8) -> LineKind {
    match n % 4 {
        0 => LineKind::Rent,
        1 => LineKind::Charges,
        2 => LineKind::Exceptional,
        _ => LineKind::Arrears,
    }
}

fn arb_lines() -> impl Strategy<Value = Vec<OpenLine<u32>>> {
    prop::collection::vec((0_u64..120, any::<u8>(), 1_i64..200_000), 0..12).prop_map(|v| {
        v.into_iter()
            .enumerate()
            .map(|(i, (offset, k, rem))| OpenLine {
                id: u32::try_from(i).unwrap(),
                due_date: d(2026, 9, 1) + chrono::Days::new(offset),
                kind: kind_of(k),
                remaining: rem,
            })
            .collect()
    })
}

proptest! {
    #[test]
    fn p1_p2_p4_somme_bornee_credit_positif_lignes_jamais_depassees(
        amount in 1_i64..3_000_000,
        lines in arb_lines(),
    ) {
        let s = allocation::allocate(amount, &lines, today()).unwrap();
        let total: i64 = s.parts.iter().map(|p| p.amount).sum();
        prop_assert!(total <= amount, "P1");
        prop_assert_eq!(total + s.credit, amount, "P2");
        prop_assert!(s.credit >= 0);
        for p in &s.parts {
            let l = lines.iter().find(|l| l.id == p.line).unwrap();
            prop_assert!(p.amount > 0 && p.amount <= l.remaining, "P4");
        }
        // chaque ligne n'est servie qu'une fois et le crédit n'existe que si tout est soldé
        let mut ids: Vec<u32> = s.parts.iter().map(|p| p.line).collect();
        ids.sort_unstable();
        ids.dedup();
        prop_assert_eq!(ids.len(), s.parts.len());
        if s.credit > 0 {
            prop_assert_eq!(total, lines.iter().map(|l| l.remaining).sum::<i64>());
        }
    }

    #[test]
    fn p7_ordre_deterministe_quel_que_soit_l_ordre_d_entree(
        amount in 1_i64..3_000_000,
        lines in arb_lines(),
        seed in any::<u64>(),
    ) {
        let mut shuffled = lines.clone();
        // permutation déterministe tirée de la graine
        let n = shuffled.len();
        if n > 1 {
            let mut x = seed;
            for i in (1..n).rev() {
                x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let j = usize::try_from(x % u64::try_from(i + 1).unwrap()).unwrap();
                shuffled.swap(i, j);
            }
        }
        prop_assert_eq!(
            allocation::allocate(amount, &lines, today()).unwrap(),
            allocation::allocate(amount, &shuffled, today()).unwrap()
        );
    }
}

// ───────── E-11, R-053, DP-64, DP-96 : montant en lettres ─────────

#[test]
fn e11_et_tests_d_or_du_montant_en_lettres() {
    let cases: &[(u32, &str)] = &[
        (1, "un franc CFA"),
        (2, "deux francs CFA"),
        (16, "seize francs CFA"),
        (17, "dix-sept francs CFA"),
        (21, "vingt et un francs CFA"),
        (22, "vingt-deux francs CFA"),
        (71, "soixante et onze francs CFA"),
        (72, "soixante-douze francs CFA"),
        (80, "quatre-vingts francs CFA"),
        (81, "quatre-vingt-un francs CFA"),
        (91, "quatre-vingt-onze francs CFA"),
        (97, "quatre-vingt-dix-sept francs CFA"),
        (100, "cent francs CFA"),
        (101, "cent un francs CFA"),
        (200, "deux cents francs CFA"),
        (201, "deux cent un francs CFA"),
        (280, "deux cent quatre-vingts francs CFA"),
        (1_000, "mille francs CFA"),
        (1_001, "mille un francs CFA"),
        (2_000, "deux mille francs CFA"),
        (75_000, "soixante-quinze mille francs CFA"),
        (80_000, "quatre-vingt mille francs CFA"),
        (100_000, "cent mille francs CFA"),
        (200_000, "deux cent mille francs CFA"),
        (1_000_000, "un million de francs CFA"),
        (2_000_000, "deux millions de francs CFA"),
        (1_250_000, "un million deux cent cinquante mille francs CFA"),
        (2_500_000, "deux millions cinq cent mille francs CFA"),
        (80_000_000, "quatre-vingts millions de francs CFA"),
        (200_000_000, "deux cents millions de francs CFA"),
        (
            999_999_999,
            "neuf cent quatre-vingt-dix-neuf millions neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf francs CFA",
        ),
    ];
    for (n, expected) in cases {
        assert_eq!(words::amount_in_words(*n).unwrap(), *expected, "{n}");
    }
    assert!(words::amount_in_words(0).is_err());
    assert!(words::amount_in_words(1_000_000_000).is_err());
}

/// Analyseur de référence indépendant (P14) : il ne partage aucune table avec `words`.
fn parse_french(text: &str) -> u64 {
    let body = text
        .strip_suffix(" de francs CFA")
        .or_else(|| text.strip_suffix(" francs CFA"))
        .or_else(|| text.strip_suffix(" franc CFA"))
        .expect("unité");
    let mut total: u64 = 0;
    let mut group: u64 = 0;
    for token in body.split([' ', '-']) {
        match token {
            "et" => {}
            "un" => group += 1,
            "deux" => group += 2,
            "trois" => group += 3,
            "quatre" => group += 4,
            "cinq" => group += 5,
            "six" => group += 6,
            "sept" => group += 7,
            "huit" => group += 8,
            "neuf" => group += 9,
            "dix" => group += 10,
            "onze" => group += 11,
            "douze" => group += 12,
            "treize" => group += 13,
            "quatorze" => group += 14,
            "quinze" => group += 15,
            "seize" => group += 16,
            "vingt" | "vingts" => {
                // « quatre-vingt(s) » : le quatre déjà compté multiplie vingt
                if group % 100 == 4 {
                    group = group - 4 + 80;
                } else {
                    group += 20;
                }
            }
            "trente" => group += 30,
            "quarante" => group += 40,
            "cinquante" => group += 50,
            "soixante" => group += 60,
            "cent" | "cents" => group = if group == 0 { 100 } else { group * 100 },
            "mille" => {
                total += if group == 0 { 1_000 } else { group * 1_000 };
                group = 0;
            }
            "million" | "millions" => {
                total += group * 1_000_000;
                group = 0;
            }
            other => panic!("mot inconnu : {other}"),
        }
    }
    total + group
}

#[test]
fn l_analyseur_de_reference_relit_les_cas_d_or() {
    assert_eq!(parse_french("soixante-quinze mille francs CFA"), 75_000);
    assert_eq!(parse_french("quatre-vingt-dix-sept francs CFA"), 97);
    assert_eq!(parse_french("deux cent quatre-vingts francs CFA"), 280);
}

proptest! {
    #[test]
    fn p14_le_texte_relu_redonne_le_nombre(n in 1_u32..=999_999_999) {
        let text = words::amount_in_words(n).unwrap();
        prop_assert_eq!(parse_french(&text), u64::from(n), "{}", text);
        prop_assert!(!text.contains("  ") && !text.starts_with(' '));
    }

    #[test]
    fn p14_petits_nombres(n in 1_u32..=2_000) {
        prop_assert_eq!(parse_french(&words::amount_in_words(n).unwrap()), u64::from(n));
    }
}

// ───────── DP-37, DP-61, DP-96 : numéro de quittance ─────────

#[test]
fn numero_officiel_et_d_essai() {
    assert_eq!(receipt::number("HOR", 2026, 94, false).unwrap(), "HOR-2026-000094");
    assert_eq!(receipt::number("HOR", 2027, 1, false).unwrap(), "HOR-2027-000001");
    assert_eq!(receipt::number("HOR", 2026, 1, true).unwrap(), "ESSAI-HOR-2026-000001");
    assert_eq!(receipt::number("REC", 2026, 999_999, false).unwrap(), "REC-2026-999999");
    assert!(receipt::number("HOR", 2026, 0, false).is_err());
    assert!(receipt::number("HOR", 2026, 1_000_000, false).is_err());
    assert!(receipt::number("hor", 2026, 1, false).is_err(), "préfixe en majuscules");
    assert!(receipt::number("H", 2026, 1, false).is_err(), "2 à 8 caractères");
    assert!(receipt::number("HOR", 1999, 1, false).is_err());
}

#[test]
fn numero_relu() {
    assert_eq!(
        receipt::parse("HOR-2026-000094"),
        Some(("HOR".to_string(), 2026, 94, false))
    );
    assert_eq!(
        receipt::parse("ESSAI-HOR-2026-000001"),
        Some(("HOR".to_string(), 2026, 1, true))
    );
    assert_eq!(receipt::parse("HOR-2026-94"), None);
}
