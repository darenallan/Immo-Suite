//! Téléphone du Burkina Faso (R-018, `API_CONTRACTS.md` §2.4) : forme canonique `+226` suivie de 8 chiffres.
//!
//! Une seule normalisation pour tout le produit : identifiant de connexion (F01), locataires (F06), mandants (F05).

/// Caractères de présentation ignorés : espaces (dont l'insécable), tirets, points, parenthèses.
fn is_separator(c: char) -> bool {
    matches!(c, ' ' | '-' | '.' | '(' | ')' | '\u{a0}')
}

/// Partie locale (après l'indicatif) d'une saisie : `None` si elle contient autre chose que des chiffres et des séparateurs,
/// ou un `+` suivi d'un autre indicatif que 226.
fn local_part(text: &str) -> Option<String> {
    let compact: String = text.chars().filter(|c| !is_separator(*c)).collect();
    let (had_plus, digits) = match compact.strip_prefix('+') {
        Some(rest) => (true, rest),
        None => (false, compact.as_str()),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let local = if had_plus {
        digits.strip_prefix("226")?
    } else if let Some(rest) = digits.strip_prefix("00226") {
        rest
    } else if digits.len() == 11 {
        digits.strip_prefix("226")?
    } else {
        digits
    };
    Some(local.to_string())
}

/// Normalise une saisie tolérante (`70123456`, `70 12 34 56`, `+226 70 12 34 56`, `0022670123456`, `226 70123456`) en
/// `+22670123456`. Tout autre format, dont 7 ou 9 chiffres : `None` (le serveur répond `validation.phone_format`).
pub fn normalize(text: &str) -> Option<String> {
    let local = local_part(text)?;
    (local.len() == 8).then(|| format!("+226{local}"))
}

/// Chiffres à chercher dans les téléphones pour la recherche unique : la saisie sans séparateurs ni indicatif, si elle ne
/// contient que des chiffres et en compte de 4 à 8. Sinon `None` : la saisie n'est pas un numéro.
pub fn search_digits(text: &str) -> Option<String> {
    let local = local_part(text)?;
    (4..=8).contains(&local.len()).then_some(local)
}
