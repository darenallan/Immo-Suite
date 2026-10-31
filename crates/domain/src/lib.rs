//! Règles pures d'Immo Suite (CODING_RULES.md §1.1, §4.2).
//!
//! Ce crate ne dépend ni de SQLx, ni de Tokio, ni de l'horloge : il se teste sans base, vite, par propriétés.
//! Les lints ci-dessous interdisent toute arithmétique flottante et tout débordement silencieux (R-001, DP-45).

#![deny(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation
)]
#![deny(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::as_conversions
)]

use std::cmp::Ordering;
use std::fmt;

/// Valeur absolue maximale d'un montant (DP-56) : alignée sur le montant en lettres testé (R-053).
pub const MAX_ABS_FCFA: i64 = 999_999_999;
const MAX_ABS_FCFA_U64: u64 = 999_999_999;

/// Taux maximal : 100 % en centièmes de point (R-015).
pub const MAX_RATE_BP: u16 = 10_000;

/// Erreurs des règles d'argent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AmountError {
    /// Montant hors des bornes de R-001 (la valeur refusée est conservée pour le message d'erreur).
    #[error("montant hors bornes : {0}")]
    OutOfRange(i64),
    /// Taux de commission hors de 0 à 10 000 centièmes de point (R-015).
    #[error("taux hors plage : {0}")]
    RateOutOfRange(u16),
    /// La base d'une commission ne peut pas être négative (règle non définie, API_CONTRACTS.md §6.3).
    #[error("base de commission négative")]
    NegativeBase,
    /// Débordement du calcul intermédiaire.
    #[error("débordement de calcul")]
    Overflow,
}

/// Montant en francs CFA : un entier signé, jamais un flottant (R-001).
///
/// La valeur absolue est bornée à 999 999 999. Le signe sert aux contre-passations (R-047).
/// Aucune conversion depuis ou vers un nombre à virgule n'existe, et aucune opération ne déborde en silence.
///
/// Les **totaux** (somme de nombreux paiements) peuvent dépasser la borne d'un montant unitaire : `Fcfa::sum`
/// les refuse aujourd'hui. Les bornes des totaux sont une question ouverte, à trancher avant l'étape 07.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fcfa(i64);

impl Fcfa {
    /// Montant signé dans les bornes de valeur absolue.
    pub fn new(value: i64) -> Result<Self, AmountError> {
        if value.unsigned_abs() > MAX_ABS_FCFA_U64 {
            Err(AmountError::OutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }

    /// 1 à 999 999 999 : un paiement, un loyer, un reversement (R-001).
    pub fn positive(value: i64) -> Result<Self, AmountError> {
        if (1..=MAX_ABS_FCFA).contains(&value) {
            Ok(Self(value))
        } else {
            Err(AmountError::OutOfRange(value))
        }
    }

    /// 0 à 999 999 999 : des charges, un dépôt de garantie, une dépense (R-001).
    pub fn non_negative(value: i64) -> Result<Self, AmountError> {
        if (0..=MAX_ABS_FCFA).contains(&value) {
            Ok(Self(value))
        } else {
            Err(AmountError::OutOfRange(value))
        }
    }

    /// La valeur entière.
    pub fn get(self) -> i64 {
        self.0
    }

    /// Addition vérifiée : refus si le résultat sort des bornes.
    pub fn checked_add(self, other: Self) -> Result<Self, AmountError> {
        Self::new(self.0.checked_add(other.0).ok_or(AmountError::Overflow)?)
    }

    /// Soustraction vérifiée : un résultat négatif est permis, hors bornes il est refusé.
    pub fn checked_sub(self, other: Self) -> Result<Self, AmountError> {
        Self::new(self.0.checked_sub(other.0).ok_or(AmountError::Overflow)?)
    }

    /// Somme vérifiée de montants.
    pub fn sum<I: IntoIterator<Item = Self>>(items: I) -> Result<Self, AmountError> {
        items.into_iter().try_fold(Self(0), Self::checked_add)
    }
}

/// Négation exacte : la valeur absolue reste dans les bornes, le résultat est donc toujours un montant valide.
impl std::ops::Neg for Fcfa {
    type Output = Fcfa;

    fn neg(self) -> Fcfa {
        // `saturating_sub` n'est jamais saturant ici (|valeur| <= 999 999 999) ; évite un `-` non vérifié
        Fcfa(0_i64.saturating_sub(self.0))
    }
}

impl PartialOrd for Fcfa {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fcfa {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl fmt::Debug for Fcfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fcfa({})", self.0)
    }
}

/// Taux en centièmes de point : 750 = 7,5 % (R-015, R-068).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RateBp(u16);

impl RateBp {
    /// De 0 à 10 000 (0 % à 100 %).
    pub fn new(basis_points: u16) -> Result<Self, AmountError> {
        if basis_points > MAX_RATE_BP {
            Err(AmountError::RateOutOfRange(basis_points))
        } else {
            Ok(Self(basis_points))
        }
    }

    /// Le taux en centièmes de point.
    pub fn get(self) -> u16 {
        self.0
    }

    /// Commission sur une base : `(base × taux + 5 000) / 10 000` en division entière, sur un entier intermédiaire
    /// plus large. C'est la **seule** opération d'arrondi du code (R-068, DP-14) : 0,5 vers le haut.
    pub fn commission(self, base: Fcfa) -> Result<Fcfa, AmountError> {
        if base.get() < 0 {
            return Err(AmountError::NegativeBase);
        }
        let product = i128::from(base.get())
            .checked_mul(i128::from(self.0))
            .ok_or(AmountError::Overflow)?;
        let rounded = product
            .checked_add(5_000)
            .ok_or(AmountError::Overflow)?
            .checked_div(10_000)
            .ok_or(AmountError::Overflow)?;
        Fcfa::new(i64::try_from(rounded).map_err(|_| AmountError::Overflow)?)
    }
}
