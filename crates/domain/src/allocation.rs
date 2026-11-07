//! Affectation d'un paiement aux lignes d'échéance (F09 ; R-043, R-044, R-046, R-086, R-087 ; DP-20, DP-21, DP-96).
//!
//! - Ordre : date d'échéance de la ligne, puis loyer, charges, exceptionnelles, arriérés, puis identifiant (ordre total, P7).
//! - L'**avance** est la part du paiement qui ne sert pas une ligne échue ou due à la date du jour : elle remplit les échéances
//!   futures déjà générées, et le reste est le **crédit du bail** (part non affectée). Une avance exige une confirmation (R-046).
//! - Aucune lecture de l'heure : « aujourd'hui » est un paramètre.

use chrono::NaiveDate;

use crate::MAX_ABS_FCFA;

/// Nature d'une ligne ; l'ordre des variantes est l'ordre d'affectation à date égale (DP-21, DP-96).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineKind {
    Rent,
    Charges,
    Exceptional,
    Arrears,
}

impl LineKind {
    /// Code de la base (`installment_lines.kind`).
    pub fn code(self) -> &'static str {
        match self {
            Self::Rent => "rent",
            Self::Charges => "charges",
            Self::Exceptional => "exceptional",
            Self::Arrears => "arrears",
        }
    }

    /// Lecture du code de la base.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "rent" => Some(Self::Rent),
            "charges" => Some(Self::Charges),
            "exceptional" => Some(Self::Exceptional),
            "arrears" => Some(Self::Arrears),
            _ => None,
        }
    }
}

/// Ligne d'échéance non annulée, avec son reste dû (`dû − affectations nettes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenLine<I> {
    pub id: I,
    pub due_date: NaiveDate,
    pub kind: LineKind,
    pub remaining: i64,
}

/// Part d'un paiement affectée à une ligne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part<I> {
    pub line: I,
    pub amount: i64,
}

/// Répartition d'un paiement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split<I> {
    /// Parts, dans l'ordre d'affectation (ou dans l'ordre saisi pour une répartition ajustée).
    pub parts: Vec<Part<I>>,
    /// Part qui ne sert pas une ligne échue ou due aujourd'hui : échéances futures plus crédit (R-046).
    pub advance: i64,
    /// Part non affectée : crédit du bail (R-086).
    pub credit: i64,
    /// Reste dû des lignes échues ou dues aujourd'hui, après ce paiement (« solde restant » de la quittance, DP-96).
    pub remaining_due_after: i64,
}

impl<I> Split<I> {
    /// Vrai si le paiement dépasse ce qui est dû aujourd'hui : `surplus: "advance"` est exigé (R-046).
    pub fn requires_surplus_confirmation(&self) -> bool {
        self.advance > 0
    }
}

/// Refus d'une affectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AllocationError {
    /// Montant hors de 1 à 999 999 999 (R-001, R-037).
    #[error("montant hors bornes")]
    Amount,
    /// Ligne inconnue : autre bail, annulée ou déjà soldée.
    #[error("ligne non ouverte")]
    UnknownLine,
    /// Part supérieure au reste de sa ligne.
    #[error("part supérieure au reste de la ligne")]
    AboveLineRemaining,
    /// Somme des parts supérieure au paiement (R-044).
    #[error("somme des parts supérieure au paiement")]
    AboveAmount,
    /// Part nulle ou négative, ou deux parts sur la même ligne.
    #[error("part invalide")]
    InvalidPart,
    /// Débordement d'une somme (impossible dans les bornes ; refusé plutôt que tronqué).
    #[error("débordement")]
    Overflow,
}

fn check_amount(amount: i64) -> Result<(), AllocationError> {
    if (1..=MAX_ABS_FCFA).contains(&amount) {
        Ok(())
    } else {
        Err(AllocationError::Amount)
    }
}

fn add(a: i64, b: i64) -> Result<i64, AllocationError> {
    a.checked_add(b).ok_or(AllocationError::Overflow)
}

fn sub(a: i64, b: i64) -> Result<i64, AllocationError> {
    a.checked_sub(b).ok_or(AllocationError::Overflow)
}

/// Lignes ouvertes (reste positif) dans l'ordre d'affectation.
pub fn ordered<I: Ord + Clone>(lines: &[OpenLine<I>]) -> Vec<OpenLine<I>> {
    let mut v: Vec<OpenLine<I>> = lines.iter().filter(|l| l.remaining > 0).cloned().collect();
    v.sort_by(|a, b| {
        a.due_date
            .cmp(&b.due_date)
            .then(a.kind.cmp(&b.kind))
            .then_with(|| a.id.cmp(&b.id))
    });
    v
}

/// Totaux d'une répartition : avance, crédit et reste dû à la date du jour.
fn summarize<I: Ord + Clone>(
    amount: i64,
    open: &[OpenLine<I>],
    parts: Vec<Part<I>>,
    today: NaiveDate,
) -> Result<Split<I>, AllocationError> {
    let mut due_now = 0_i64;
    for l in open.iter().filter(|l| l.due_date <= today) {
        due_now = add(due_now, l.remaining)?;
    }
    let mut allocated = 0_i64;
    let mut allocated_due_now = 0_i64;
    for p in &parts {
        allocated = add(allocated, p.amount)?;
        if open.iter().any(|l| l.id == p.line && l.due_date <= today) {
            allocated_due_now = add(allocated_due_now, p.amount)?;
        }
    }
    Ok(Split {
        parts,
        advance: sub(amount, allocated_due_now)?,
        credit: sub(amount, allocated)?,
        remaining_due_after: sub(due_now, allocated_due_now)?,
    })
}

/// Répartition par défaut : chaque ligne ouverte, dans l'ordre, jusqu'à épuisement du montant (R-043, R-087).
pub fn allocate<I: Ord + Clone>(
    amount: i64,
    lines: &[OpenLine<I>],
    today: NaiveDate,
) -> Result<Split<I>, AllocationError> {
    check_amount(amount)?;
    let open = ordered(lines);
    let mut left = amount;
    let mut parts = Vec::new();
    for l in &open {
        if left == 0 {
            break;
        }
        let take = left.min(l.remaining);
        parts.push(Part {
            line: l.id.clone(),
            amount: take,
        });
        left = sub(left, take)?;
    }
    summarize(amount, &open, parts, today)
}

/// Répartition ajustée par l'utilisateur (F09 règle 11) : chaque part vise une ligne ouverte, au plus son reste, une fois ;
/// la somme ne dépasse pas le montant (R-044). Le reste non affecté est un crédit.
pub fn check_manual<I: Ord + Clone>(
    amount: i64,
    lines: &[OpenLine<I>],
    parts: &[Part<I>],
    today: NaiveDate,
) -> Result<Split<I>, AllocationError> {
    check_amount(amount)?;
    let open = ordered(lines);
    let mut seen: Vec<&I> = Vec::new();
    let mut total = 0_i64;
    for p in parts {
        if p.amount <= 0 || seen.contains(&&p.line) {
            return Err(AllocationError::InvalidPart);
        }
        seen.push(&p.line);
        let line = open
            .iter()
            .find(|l| l.id == p.line)
            .ok_or(AllocationError::UnknownLine)?;
        if p.amount > line.remaining {
            return Err(AllocationError::AboveLineRemaining);
        }
        total = add(total, p.amount)?;
    }
    if total > amount {
        return Err(AllocationError::AboveAmount);
    }
    summarize(amount, &open, parts.to_vec(), today)
}

/// Crédit du bail appliqué à de nouvelles lignes (R-086, E-31) : `credits` est la part non affectée de chaque paiement,
/// du plus ancien au plus récent ; renvoie `(paiement, part)` dans l'ordre d'affectation des lignes.
pub fn apply_credit<P: Clone, I: Ord + Clone>(credits: &[(P, i64)], lines: &[OpenLine<I>]) -> Vec<(P, Part<I>)> {
    let mut out = Vec::new();
    let mut pool: Vec<(P, i64)> = credits.iter().filter(|(_, c)| *c > 0).cloned().collect();
    let mut k = 0_usize;
    for l in ordered(lines) {
        let mut need = l.remaining;
        while need > 0 {
            let Some((payment, left)) = pool.get_mut(k) else {
                return out;
            };
            let take = need.min(*left);
            out.push((
                payment.clone(),
                Part {
                    line: l.id.clone(),
                    amount: take,
                },
            ));
            // `take` ≤ `need` et ≤ `left` : les soustractions ne peuvent pas passer sous zéro
            need = need.saturating_sub(take);
            *left = left.saturating_sub(take);
            if *left == 0 {
                k = k.saturating_add(1);
            }
        }
    }
    out
}
