//! Règles pures du patrimoine (F04, DP-92) : référence d'un lot (R-011), champs selon l'usage (R-012, R-094, DP-41),
//! loyer de référence (R-001, P16) et surface en texte décimal (aucun flottant, `numeric(8,2)` en base).

use crate::{Fcfa, MAX_ABS_FCFA};

/// Refus d'une valeur du patrimoine ; l'application le traduit en code `validation.*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CatalogError {
    #[error("valeur obligatoire")]
    Required,
    #[error("valeur trop longue")]
    TooLong,
    #[error("montant nul ou négatif")]
    AmountNotPositive,
    #[error("montant hors bornes")]
    AmountOutOfRange,
    #[error("format refusé")]
    Invalid,
}

/// Usage d'un lot (R-094 : `mixed` est accepté ; le type du bail choisira les règles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitUsage {
    Residential,
    Commercial,
    Mixed,
}

impl UnitUsage {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "residential" => Some(Self::Residential),
            "commercial" => Some(Self::Commercial),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Residential => "residential",
            Self::Commercial => "commercial",
            Self::Mixed => "mixed",
        }
    }
}

/// Types de lot du schéma (`units.kind`).
pub const UNIT_KINDS: [&str; 11] = [
    "house",
    "apartment",
    "villa",
    "studio",
    "room",
    "shop",
    "workshop",
    "warehouse",
    "office",
    "premises",
    "other",
];

/// R-012, DP-41 : « activité autorisée » n'existe que pour un lot commercial ou mixte.
pub fn permitted_activity_applies(usage: UnitUsage) -> bool {
    !matches!(usage, UnitUsage::Residential)
}

/// Longueur maximale d'une référence de lot (schéma).
pub const MAX_REFERENCE_CHARS: usize = 40;

/// Référence saisie, sans espaces de bord, de 1 à 40 caractères (R-011). La casse tapée est conservée.
pub fn unit_reference(text: &str) -> Result<String, CatalogError> {
    let t = text.trim();
    let n = t.chars().count();
    if n == 0 {
        Err(CatalogError::Required)
    } else if n > MAX_REFERENCE_CHARS {
        Err(CatalogError::TooLong)
    } else {
        Ok(t.to_string())
    }
}

/// Clé d'unicité d'une référence dans son bien : sans espaces de bord et sans tenir compte de la casse (DP-92). La base applique
/// la même règle (`lower(btrim(reference))`).
pub fn reference_key(text: &str) -> String {
    text.trim().to_lowercase()
}

/// Loyer de référence (R-001, P16) : entier FCFA de 1 à 999 999 999.
pub fn reference_rent(value: i64) -> Result<Fcfa, CatalogError> {
    if value < 1 {
        return Err(CatalogError::AmountNotPositive);
    }
    if value > MAX_ABS_FCFA {
        return Err(CatalogError::AmountOutOfRange);
    }
    Fcfa::new(value).map_err(|_| CatalogError::AmountOutOfRange)
}

/// Surface en m² : texte décimal strictement positif, 6 chiffres entiers et 2 décimales au plus, virgule ou point acceptés.
/// Rendue avec un point, prête pour `::numeric(8,2)`. Aucun flottant n'intervient.
pub fn parse_surface(text: &str) -> Result<String, CatalogError> {
    let t = text.trim().replace(',', ".");
    let (whole, fraction) = match t.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (t.as_str(), None),
    };
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    if !digits(whole) || whole.len() > 6 {
        return Err(CatalogError::Invalid);
    }
    if let Some(f) = fraction
        && (!digits(f) || f.len() > 2)
    {
        return Err(CatalogError::Invalid);
    }
    let is_zero = whole.chars().all(|c| c == '0') && fraction.is_none_or(|f| f.chars().all(|c| c == '0'));
    if is_zero {
        return Err(CatalogError::Invalid);
    }
    Ok(t)
}
