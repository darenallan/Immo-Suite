//! Propriété P16 (TESTING_QA.md) : aucune valeur de montant produite n'est autre qu'un entier dans les bornes (R-001).
//! Les flottants sont exclus par construction : aucune API de `Fcfa` n'accepte ni ne rend un `f32` ou un `f64`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // tests : les lints de production ne s'y appliquent pas (CODING_RULES.md §3.1)

use immo_domain::{Fcfa, RateBp};
use proptest::prelude::*;

const MAX: i64 = 999_999_999;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// Toute valeur construite avec succès est dans les bornes ; toute valeur hors bornes est refusée.
    #[test]
    fn p16_construction(v in any::<i64>()) {
        match Fcfa::new(v) {
            Ok(x) => { prop_assert_eq!(x.get(), v); prop_assert!(v.unsigned_abs() <= 999_999_999); }
            Err(_) => prop_assert!(v.unsigned_abs() > 999_999_999),
        }
    }

    /// Une addition réussie vaut exactement la somme entière et reste dans les bornes ; sinon elle est refusée.
    #[test]
    fn p16_addition(a in -MAX..=MAX, b in -MAX..=MAX) {
        let (fa, fb) = (Fcfa::new(a).unwrap(), Fcfa::new(b).unwrap());
        let exact = a + b;
        match fa.checked_add(fb) {
            Ok(x) => { prop_assert_eq!(x.get(), exact); prop_assert!(exact.abs() <= MAX); }
            Err(_) => prop_assert!(exact.abs() > MAX),
        }
    }

    #[test]
    fn p16_soustraction(a in -MAX..=MAX, b in -MAX..=MAX) {
        let (fa, fb) = (Fcfa::new(a).unwrap(), Fcfa::new(b).unwrap());
        let exact = a - b;
        match fa.checked_sub(fb) {
            Ok(x) => { prop_assert_eq!(x.get(), exact); prop_assert!(exact.abs() <= MAX); }
            Err(_) => prop_assert!(exact.abs() > MAX),
        }
    }

    /// La négation est exacte et involutive.
    #[test]
    fn p16_negation(a in -MAX..=MAX) {
        let f = Fcfa::new(a).unwrap();
        prop_assert_eq!((-f).get(), -a);
        prop_assert_eq!(-(-f), f);
    }

    /// La commission est un entier dans les bornes, jamais supérieure à la base, et égale à l'arrondi exact (0,5 vers le haut).
    #[test]
    fn p16_commission(base in 0..=MAX, bp in 0u16..=10_000) {
        let c = RateBp::new(bp).unwrap().commission(Fcfa::new(base).unwrap()).unwrap().get();
        prop_assert!((0..=MAX).contains(&c));
        prop_assert!(c <= base);
        // référence indépendante : arrondi à l'entier le plus proche du produit exact base × taux / 10 000
        let produit = i128::from(base) * i128::from(bp);
        let attendu = (produit * 2 + 10_000) / 20_000; // (2p + d) / 2d : 0,5 vers le haut, sans réutiliser la formule testée
        prop_assert_eq!(i128::from(c), attendu);
    }

    /// Un taux plus élevé ne donne jamais une commission plus faible (monotonie).
    #[test]
    fn commission_monotone_en_taux(base in 0..=MAX, a in 0u16..=10_000, b in 0u16..=10_000) {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let f = Fcfa::new(base).unwrap();
        let cl = RateBp::new(lo).unwrap().commission(f).unwrap().get();
        let ch = RateBp::new(hi).unwrap().commission(f).unwrap().get();
        prop_assert!(cl <= ch);
    }
}
