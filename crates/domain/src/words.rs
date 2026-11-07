//! Montant en lettres de la quittance (F10 ; R-053, DP-64, DP-96) [À VÉRIFIER AVEC UN JURISTE].
//!
//! Orthographe traditionnelle : trait d'union sous cent seulement entre dizaine et unité, « et » pour 21 à 71 (sauf 81, 91),
//! « vingt » et « cent » au pluriel quand ils sont multipliés et terminent le nombre ou précèdent « millions » (nom), jamais
//! devant « mille » (invariable). Unité : « franc(s) CFA », « de francs CFA » après un nombre exact de millions.

/// Plus grand montant écrit en lettres (aligné sur `MAX_ABS_FCFA`).
pub const MAX_WORDS: u32 = 999_999_999;

/// Montant hors de 1 à 999 999 999.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("montant hors bornes pour l'écriture en lettres : {0}")]
pub struct WordsError(pub u32);

fn unit(n: u32) -> &'static str {
    match n {
        1 => "un",
        2 => "deux",
        3 => "trois",
        4 => "quatre",
        5 => "cinq",
        6 => "six",
        7 => "sept",
        8 => "huit",
        9 => "neuf",
        10 => "dix",
        11 => "onze",
        12 => "douze",
        13 => "treize",
        14 => "quatorze",
        15 => "quinze",
        16 => "seize",
        _ => "",
    }
}

fn tens(n: u32) -> &'static str {
    match n {
        2 => "vingt",
        3 => "trente",
        4 => "quarante",
        5 => "cinquante",
        _ => "soixante",
    }
}

/// 1 à 99. `last` : rien de numérique ne suit (fin du nombre ou « millions ») : « quatre-vingts » prend alors un s.
fn below_100(n: u32, last: bool) -> String {
    match n {
        0 => String::new(),
        1..=16 => unit(n).to_string(),
        17..=19 => format!("dix-{}", unit(n % 10)),
        20..=69 => {
            let t = tens(n / 10);
            match n % 10 {
                0 => t.to_string(),
                1 => format!("{t} et un"),
                u => format!("{t}-{}", unit(u)),
            }
        }
        71 => "soixante et onze".to_string(),
        70..=79 => format!("soixante-{}", below_100(n.saturating_sub(60), false)),
        80 => {
            if last {
                "quatre-vingts".to_string()
            } else {
                "quatre-vingt".to_string()
            }
        }
        _ => format!("quatre-vingt-{}", below_100(n % 80, false)),
    }
}

/// 1 à 999. Voir `below_100` pour `last` (« deux cents », « deux cent mille »).
fn below_1000(n: u32, last: bool) -> String {
    let h = n / 100;
    let r = n % 100;
    let hundreds = match h {
        0 => String::new(),
        1 => "cent".to_string(),
        _ if r == 0 && last => format!("{} cents", unit(h)),
        _ => format!("{} cent", unit(h)),
    };
    let rest = below_100(r, last);
    match (hundreds.is_empty(), rest.is_empty()) {
        (true, _) => rest,
        (false, true) => hundreds,
        (false, false) => format!("{hundreds} {rest}"),
    }
}

/// Nombre en lettres, sans unité : 1 à 999 999 999.
pub fn number_in_words(n: u32) -> Result<String, WordsError> {
    if !(1..=MAX_WORDS).contains(&n) {
        return Err(WordsError(n));
    }
    let millions = n / 1_000_000;
    let thousands = (n / 1_000) % 1_000;
    let rest = n % 1_000;
    let mut out: Vec<String> = Vec::new();
    match millions {
        0 => {}
        1 => out.push("un million".to_string()),
        m => out.push(format!("{} millions", below_1000(m, true))),
    }
    match thousands {
        0 => {}
        1 => out.push("mille".to_string()),
        t => out.push(format!("{} mille", below_1000(t, false))),
    }
    if rest > 0 {
        out.push(below_1000(rest, true));
    }
    Ok(out.join(" "))
}

/// Montant de la quittance en lettres avec l'unité (E-11 : 75 000 → « soixante-quinze mille francs CFA »).
pub fn amount_in_words(n: u32) -> Result<String, WordsError> {
    let words = number_in_words(n)?;
    let unit = if n == 1 {
        "franc CFA"
    } else if n.is_multiple_of(1_000_000) {
        "de francs CFA"
    } else {
        "francs CFA"
    };
    Ok(format!("{words} {unit}"))
}
