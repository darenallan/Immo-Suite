//! Montant en lettres (R-053, DP-64 : orthographe traditionnelle, « francs CFA »).
//!
//! Fonction pure, sans dépendance. Règles d'accord appliquées, **à faire valider** (TESTING_QA.md §6) :
//! - trait d'union entre dizaines et unités seulement (« vingt-deux », « quatre-vingt-dix-sept ») ;
//! - « et » sans trait d'union pour 21, 31, 41, 51, 61 et 71 (« vingt et un », « soixante et onze »),
//!   mais pas pour 81 et 91 (« quatre-vingt-un », « quatre-vingt-onze ») ;
//! - « vingt » et « cent » prennent un « s » quand rien ne les suit et qu'ils multiplient
//!   (« quatre-vingts », « deux cents ») ; invariables devant « mille » (« quatre-vingt mille »,
//!   « deux cent mille ») ; avec « s » devant « million » (« deux cents millions ») ;
//! - « mille » invariable, jamais « un mille » ; « million » prend un « s » au pluriel ;
//! - « de » devant la devise quand le montant finit par des millions entiers
//!   (« un million de francs CFA ») ; « franc » au singulier pour 0 et 1.

const UNITS: [&str; 17] = [
    "zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze", "douze", "treize", "quatorze",
    "quinze", "seize",
];

/// 0..=99. `last` : rien ne suit, donc « quatre-vingts » prend son « s ».
fn below_100(n: u64, last: bool) -> String {
    match n {
        0..=16 => UNITS[n as usize].to_string(),
        17..=19 => format!("dix-{}", UNITS[(n - 10) as usize]),
        20..=69 => {
            let tens = ["vingt", "trente", "quarante", "cinquante", "soixante"][(n / 10 - 2) as usize];
            match n % 10 {
                0 => tens.to_string(),
                1 => format!("{tens} et un"),
                u => format!("{tens}-{}", UNITS[u as usize]),
            }
        }
        70..=79 => match n - 60 {
            11 => "soixante et onze".to_string(),
            m => format!("soixante-{}", below_100(m, false)),
        },
        80 => if last { "quatre-vingts" } else { "quatre-vingt" }.to_string(),
        81..=99 => format!("quatre-vingt-{}", below_100(n - 80, false)),
        _ => unreachable!(),
    }
}

/// 0..=999. `last` : rien ne suit (le « s » de « cents » et de « quatre-vingts » est permis).
fn below_1000(n: u64, last: bool) -> String {
    let (h, r) = (n / 100, n % 100);
    match (h, r) {
        (0, _) => below_100(r, last),
        (1, 0) => "cent".to_string(),
        (1, _) => format!("cent {}", below_100(r, last)),
        (_, 0) => format!("{} cent{}", UNITS[h as usize], if last { "s" } else { "" }),
        (_, _) => format!("{} cent {}", UNITS[h as usize], below_100(r, last)),
    }
}

/// Le nombre en lettres, en minuscules, sans devise. Valable de 0 à 999 999 999.
pub fn number_in_words(n: u64) -> String {
    assert!(n <= 999_999_999, "hors borne (DP-56) : {n}");
    if n == 0 {
        return UNITS[0].to_string();
    }
    let (millions, thousands, rest) = (n / 1_000_000, (n / 1000) % 1000, n % 1000);
    let mut parts: Vec<String> = vec![];
    if millions > 0 {
        let unit = if millions == 1 { "million" } else { "millions" };
        parts.push(format!("{} {unit}", below_1000(millions, true)));
    }
    if thousands > 0 {
        parts.push(if thousands == 1 { "mille".to_string() } else { format!("{} mille", below_1000(thousands, false)) });
    }
    if rest > 0 {
        parts.push(below_1000(rest, true));
    }
    parts.join(" ")
}

/// Le montant en francs CFA, en minuscules : « soixante-quinze mille francs CFA ».
pub fn amount_in_words(n: u64) -> String {
    let words = number_in_words(n);
    let unit = if n <= 1 { "franc CFA" } else { "francs CFA" };
    // « un million de francs », « douze millions de francs » ; « mille francs », « deux millions cinq cent mille francs »
    let de = if n >= 1_000_000 && n % 1_000_000 == 0 { " de" } else { "" };
    format!("{words}{de} {unit}")
}

/// Même texte avec une majuscule initiale, tel qu'imprimé sur la quittance.
pub fn amount_in_words_capitalized(n: u64) -> String {
    let s = amount_in_words(n);
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tableau écrit à la main pour les frontières de TESTING_QA.md §6 (indépendant de l'algorithme).
    const FRONTIERS: [(u64, &str); 26] = [
        (0, "zéro franc CFA"),
        (1, "un franc CFA"),
        (20, "vingt francs CFA"),
        (21, "vingt et un francs CFA"),
        (70, "soixante-dix francs CFA"),
        (71, "soixante et onze francs CFA"),
        (80, "quatre-vingts francs CFA"),
        (81, "quatre-vingt-un francs CFA"),
        (90, "quatre-vingt-dix francs CFA"),
        (91, "quatre-vingt-onze francs CFA"),
        (99, "quatre-vingt-dix-neuf francs CFA"),
        (100, "cent francs CFA"),
        (101, "cent un francs CFA"),
        (200, "deux cents francs CFA"),
        (201, "deux cent un francs CFA"),
        (1_000, "mille francs CFA"),
        (1_100, "mille cent francs CFA"),
        (75_000, "soixante-quinze mille francs CFA"),
        (80_000, "quatre-vingt mille francs CFA"),
        (100_000, "cent mille francs CFA"),
        (200_000, "deux cent mille francs CFA"),
        (1_000_000, "un million de francs CFA"),
        (1_250_000, "un million deux cent cinquante mille francs CFA"),
        (2_000_000, "deux millions de francs CFA"),
        (21_000_000, "vingt et un millions de francs CFA"),
        (999_999_999, "neuf cent quatre-vingt-dix-neuf millions neuf cent quatre-vingt-dix-neuf mille neuf cent quatre-vingt-dix-neuf francs CFA"),
    ];

    #[test]
    fn frontiers_match_hand_written_table() {
        for (n, expected) in FRONTIERS {
            assert_eq!(amount_in_words(n), expected, "n = {n}");
        }
    }

    #[test]
    fn every_integer_up_to_2000_is_non_empty_and_distinct() {
        let mut seen = std::collections::HashSet::new();
        for n in 0..=2000 {
            let s = amount_in_words(n);
            assert!(!s.is_empty());
            assert!(seen.insert(s), "doublon pour {n}");
        }
    }

    #[test]
    fn capitalization_keeps_accents() {
        assert_eq!(amount_in_words_capitalized(0), "Zéro franc CFA");
        assert_eq!(amount_in_words_capitalized(75_000), "Soixante-quinze mille francs CFA");
    }
}
