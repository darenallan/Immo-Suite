//! Numéro de quittance (F10 ; R-050, DP-37, DP-61, DP-96) : `{préfixe}-{année}-{séquence sur 6 chiffres}`,
//! précédé de `ESSAI-` en série d'essai (organisation en phase `setup`).

/// Numéro impossible à former.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NumberError {
    #[error("préfixe invalide (2 à 8 lettres majuscules ou chiffres)")]
    Prefix,
    #[error("année hors de 2020 à 2100")]
    Year,
    #[error("séquence hors de 1 à 999 999")]
    Sequence,
}

/// Préfixe valide : 2 à 8 caractères `A-Z` ou `0-9` (contrainte de `org_settings.receipt_prefix`).
pub fn valid_prefix(prefix: &str) -> bool {
    (2..=8).contains(&prefix.len()) && prefix.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

/// Numéro de la quittance. `trial` : série d'essai (`ESSAI-` devant, DP-61).
pub fn number(prefix: &str, year: i32, sequence: u32, trial: bool) -> Result<String, NumberError> {
    if !valid_prefix(prefix) {
        return Err(NumberError::Prefix);
    }
    if !(2020..=2100).contains(&year) {
        return Err(NumberError::Year);
    }
    if !(1..=999_999).contains(&sequence) {
        return Err(NumberError::Sequence);
    }
    let lead = if trial { "ESSAI-" } else { "" };
    Ok(format!("{lead}{prefix}-{year:04}-{sequence:06}"))
}

/// Lecture d'un numéro : `(préfixe, année, séquence, essai)`.
pub fn parse(text: &str) -> Option<(String, i32, u32, bool)> {
    let (trial, rest) = match text.strip_prefix("ESSAI-") {
        Some(r) => (true, r),
        None => (false, text),
    };
    let mut it = rest.rsplitn(3, '-');
    let seq = it.next()?;
    let year = it.next()?;
    let prefix = it.next()?;
    if seq.len() != 6 || year.len() != 4 || !valid_prefix(prefix) {
        return None;
    }
    if !seq.bytes().all(|b| b.is_ascii_digit()) || !year.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((prefix.to_string(), year.parse().ok()?, seq.parse().ok()?, trial))
}
