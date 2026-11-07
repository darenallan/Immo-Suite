//! Règles pures de l'étape 05 (F04, F06, DP-92) : téléphone `+226` (R-018), référence d'un lot (R-011), champs selon l'usage
//! (R-012, DP-41), loyer de référence entier et borné (R-001, P16), surface décimale sans flottant.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // tests : les lints de production ne s'y appliquent pas (CODING_RULES.md §3.1)

use immo_domain::catalog::{
    CatalogError, UnitUsage, parse_surface, permitted_activity_applies, reference_key, reference_rent, unit_reference,
};
use immo_domain::phone::{normalize, search_digits};
use proptest::prelude::*;

// ───────── téléphone (R-018, cas chiffrés de l'étape 05) ─────────

#[test]
fn r018_les_trois_formes_de_l_etape_donnent_la_meme_valeur() {
    for form in ["70123456", "+226 70 12 34 56", "0022670123456"] {
        assert_eq!(normalize(form).as_deref(), Some("+22670123456"), "forme : {form}");
    }
    for form in [
        "70 12 34 56",
        "70.12.34.56",
        "70-12-34-56",
        "226 70123456",
        "+22670123456",
        "  +226 70 12 34 56  ",
    ] {
        assert_eq!(normalize(form).as_deref(), Some("+22670123456"), "forme : {form}");
    }
    // bloc fictif des essais et de la démonstration
    assert_eq!(normalize("+226 00 00 01 01").as_deref(), Some("+22600000101"));
    assert_eq!(normalize("00000101").as_deref(), Some("+22600000101"));
}

#[test]
fn r018_sept_chiffres_neuf_chiffres_et_lettres_sont_refuses() {
    for form in [
        "7012345",
        "701234567",
        "+2267012345",
        "+226701234567",
        "+2267012345678",
        "7O123456",
        "++22670123456",
        "",
        "+33612345678",
    ] {
        assert_eq!(normalize(form), None, "forme : {form:?}");
    }
}

#[test]
fn recherche_par_chiffres_avec_ou_sans_indicatif() {
    assert_eq!(search_digits("00000101").as_deref(), Some("00000101"));
    assert_eq!(search_digits("+226 00 00 01 01").as_deref(), Some("00000101"));
    assert_eq!(search_digits("00226 70 12").as_deref(), Some("7012"));
    assert_eq!(search_digits("70 12").as_deref(), Some("7012"));
    // moins de 4 chiffres, ou des lettres : ce n'est pas une recherche de téléphone
    assert_eq!(search_digits("701"), None);
    assert_eq!(search_digits("tapsoba"), None);
    assert_eq!(search_digits("A1 Zogona"), None);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5_000))]

    /// Toute sortie de la normalisation respecte `^\+226[0-9]{8}$` ; huit chiffres quelconques sont toujours acceptés.
    #[test]
    fn r018_sortie_toujours_canonique(text in "\\PC{0,24}") {
        if let Some(p) = normalize(&text) {
            prop_assert_eq!(p.len(), 12);
            prop_assert!(p.starts_with("+226"));
            prop_assert!(p[4..].chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn r018_huit_chiffres_acceptes(n in 0_u32..100_000_000) {
        let local = format!("{n:08}");
        prop_assert_eq!(normalize(&local), Some(format!("+226{local}")));
        prop_assert_eq!(normalize(&format!("+226 {local}")), Some(format!("+226{local}")));
    }
}

// ───────── référence d'un lot (R-011, DP-92) ─────────

#[test]
fn r011_reference_sans_espaces_de_bord_et_bornee() {
    assert_eq!(unit_reference("  A1 ").unwrap(), "A1");
    assert_eq!(unit_reference("Boutique 3").unwrap(), "Boutique 3");
    assert_eq!(unit_reference("   "), Err(CatalogError::Required));
    assert_eq!(unit_reference(&"x".repeat(41)), Err(CatalogError::TooLong));
    assert!(unit_reference(&"x".repeat(40)).is_ok());
}

#[test]
fn r011_a1_et_a1_minuscule_sont_la_meme_reference() {
    assert_eq!(reference_key("A1"), reference_key("a1"));
    assert_eq!(reference_key(" A1 "), reference_key("a1"));
    assert_ne!(reference_key("A1"), reference_key("A2"));
}

// ───────── champs selon l'usage (R-012, R-094, DP-41) ─────────

#[test]
fn r012_activite_autorisee_seulement_pour_un_lot_commercial_ou_mixte() {
    assert!(!permitted_activity_applies(UnitUsage::Residential));
    assert!(permitted_activity_applies(UnitUsage::Commercial));
    assert!(permitted_activity_applies(UnitUsage::Mixed));
    assert_eq!(UnitUsage::parse("residential"), Some(UnitUsage::Residential));
    assert_eq!(UnitUsage::parse("mixte"), None);
}

// ───────── loyer de référence (R-001, P16) et surface ─────────

#[test]
fn p16_loyer_de_reference_cas_limites() {
    assert_eq!(reference_rent(75_000).unwrap().get(), 75_000);
    assert_eq!(reference_rent(1).unwrap().get(), 1);
    assert_eq!(reference_rent(999_999_999).unwrap().get(), 999_999_999);
    assert_eq!(reference_rent(0), Err(CatalogError::AmountNotPositive));
    assert_eq!(reference_rent(-5), Err(CatalogError::AmountNotPositive));
    assert_eq!(reference_rent(1_000_000_000), Err(CatalogError::AmountOutOfRange));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// P16 : tout loyer de référence accepté est un entier de 1 à 999 999 999 ; tout autre est refusé.
    #[test]
    fn p16_loyer_de_reference(v in any::<i64>()) {
        match reference_rent(v) {
            Ok(x) => { prop_assert_eq!(x.get(), v); prop_assert!((1..=999_999_999).contains(&v)); }
            Err(_) => prop_assert!(!(1..=999_999_999).contains(&v)),
        }
    }
}

#[test]
fn surface_decimale_sans_flottant() {
    assert_eq!(parse_surface("45").unwrap(), "45");
    assert_eq!(parse_surface("45.5").unwrap(), "45.5");
    assert_eq!(parse_surface("45,50").unwrap(), "45.50");
    assert_eq!(parse_surface(" 120.25 ").unwrap(), "120.25");
    for bad in ["0", "0.00", "-3", "45.555", "1e3", "abc", "", "1234567.5", ".5"] {
        assert!(parse_surface(bad).is_err(), "surface : {bad:?}");
    }
}
