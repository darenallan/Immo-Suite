//! Tests d'or des règles d'argent de l'étape 01 : R-001 (entiers et bornes), R-015 (taux), R-068 (commission).
//! Écrits AVANT le code (procédure ROADMAP.md §8, point 3). Cas chiffrés : E-16, E-26, E-28 de DOMAINE.md.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // tests : les lints de production ne s'y appliquent pas (CODING_RULES.md §3.1)

use immo_domain::{AmountError, Fcfa, RateBp};

fn fcfa(v: i64) -> Fcfa {
    Fcfa::new(v).unwrap()
}

// ───────── R-001 : bornes ─────────

#[test]
fn paiement_loyer_reversement_acceptent_1_a_999_999_999() {
    assert_eq!(Fcfa::positive(1).unwrap().get(), 1);
    assert_eq!(Fcfa::positive(999_999_999).unwrap().get(), 999_999_999);
}

#[test]
fn paiement_loyer_reversement_refusent_zero_negatif_et_trop_grand() {
    assert_eq!(Fcfa::positive(0), Err(AmountError::OutOfRange(0)));
    assert_eq!(Fcfa::positive(-1), Err(AmountError::OutOfRange(-1)));
    assert_eq!(
        Fcfa::positive(1_000_000_000),
        Err(AmountError::OutOfRange(1_000_000_000))
    );
}

#[test]
fn charges_depot_depense_acceptent_0_a_999_999_999() {
    assert_eq!(Fcfa::non_negative(0).unwrap().get(), 0);
    assert_eq!(Fcfa::non_negative(999_999_999).unwrap().get(), 999_999_999);
    assert_eq!(Fcfa::non_negative(-1), Err(AmountError::OutOfRange(-1)));
    assert_eq!(
        Fcfa::non_negative(1_000_000_000),
        Err(AmountError::OutOfRange(1_000_000_000))
    );
}

#[test]
fn une_valeur_signee_reste_dans_les_bornes_en_valeur_absolue() {
    // les contre-passations sont des montants négatifs (R-047) : |v| <= 999 999 999
    assert_eq!(fcfa(-999_999_999).get(), -999_999_999);
    assert_eq!(Fcfa::new(-1_000_000_000), Err(AmountError::OutOfRange(-1_000_000_000)));
    assert_eq!(Fcfa::new(i64::MIN), Err(AmountError::OutOfRange(i64::MIN)));
    assert_eq!(Fcfa::new(i64::MAX), Err(AmountError::OutOfRange(i64::MAX)));
}

// ───────── débordement vérifié ─────────

#[test]
fn addition_verifiee() {
    assert_eq!(fcfa(75_000).checked_add(fcfa(25_000)).unwrap().get(), 100_000);
    assert_eq!(
        fcfa(500_000_000).checked_add(fcfa(499_999_999)).unwrap().get(),
        999_999_999
    );
    assert_eq!(
        fcfa(500_000_000).checked_add(fcfa(500_000_000)),
        Err(AmountError::OutOfRange(1_000_000_000))
    );
}

#[test]
fn soustraction_verifiee_et_resultat_negatif_permis() {
    assert_eq!(fcfa(100_000).checked_sub(fcfa(75_000)).unwrap().get(), 25_000);
    assert_eq!(fcfa(75_000).checked_sub(fcfa(100_000)).unwrap().get(), -25_000);
    assert_eq!(
        fcfa(-999_999_999).checked_sub(fcfa(1)),
        Err(AmountError::OutOfRange(-1_000_000_000))
    );
}

#[test]
fn negation_exacte() {
    assert_eq!((-fcfa(75_000)).get(), -75_000);
    assert_eq!((-fcfa(-75_000)).get(), 75_000);
    assert_eq!((-fcfa(0)).get(), 0);
}

#[test]
fn somme_verifiee() {
    assert_eq!(
        Fcfa::sum([fcfa(10_000), fcfa(20_000), fcfa(30_000)]).unwrap().get(),
        60_000
    );
    assert_eq!(Fcfa::sum([]).unwrap().get(), 0);
    assert!(Fcfa::sum([fcfa(600_000_000), fcfa(600_000_000)]).is_err());
}

#[test]
fn comparaison_et_tri() {
    assert!(fcfa(1) < fcfa(2));
    assert!(fcfa(-5) < fcfa(0));
    let mut v = [fcfa(30), fcfa(10), fcfa(20)];
    v.sort();
    assert_eq!(v.iter().map(|x| x.get()).collect::<Vec<_>>(), vec![10, 20, 30]);
}

// ───────── R-015 : taux ─────────

#[test]
fn taux_de_0_a_100_pour_cent() {
    assert_eq!(RateBp::new(0).unwrap().get(), 0);
    assert_eq!(RateBp::new(10_000).unwrap().get(), 10_000);
    assert_eq!(RateBp::new(10_001), Err(AmountError::RateOutOfRange(10_001)));
}

// ───────── R-068 : commission, un seul arrondi (DP-14) ─────────

#[test]
fn e16_commission_de_8_pour_cent_sur_100_000() {
    assert_eq!(
        RateBp::new(800).unwrap().commission(fcfa(100_000)).unwrap().get(),
        8_000
    );
}

#[test]
fn e26_arrondi_au_plus_proche_0_5_vers_le_haut() {
    // 7,5 % de 333 333 = 24 999,975 -> 25 000
    assert_eq!(
        RateBp::new(750).unwrap().commission(fcfa(333_333)).unwrap().get(),
        25_000
    );
}

#[test]
fn e28_base_par_defaut_puis_base_loyer_plus_charges() {
    let taux = RateBp::new(800).unwrap();
    assert_eq!(taux.commission(fcfa(100_000)).unwrap().get(), 8_000);
    assert_eq!(taux.commission(fcfa(110_000)).unwrap().get(), 8_800);
}

#[test]
fn arrondi_exactement_a_la_moitie_va_vers_le_haut() {
    // 50 % de 1 = 0,5 -> 1 ; 49,99 % de 1 -> 0
    assert_eq!(RateBp::new(5_000).unwrap().commission(fcfa(1)).unwrap().get(), 1);
    assert_eq!(RateBp::new(4_999).unwrap().commission(fcfa(1)).unwrap().get(), 0);
}

#[test]
fn taux_nul_et_taux_plein() {
    assert_eq!(RateBp::new(0).unwrap().commission(fcfa(123_456)).unwrap().get(), 0);
    assert_eq!(
        RateBp::new(10_000).unwrap().commission(fcfa(123_456)).unwrap().get(),
        123_456
    );
}

#[test]
fn base_negative_refusee() {
    assert_eq!(
        RateBp::new(800).unwrap().commission(fcfa(-1)),
        Err(AmountError::NegativeBase)
    );
}

#[test]
fn commission_maximale_sans_debordement() {
    // 100 % de la base maximale : le calcul intermédiaire dépasse i64 si l'on n'élargit pas
    assert_eq!(
        RateBp::new(10_000)
            .unwrap()
            .commission(fcfa(999_999_999))
            .unwrap()
            .get(),
        999_999_999
    );
}
