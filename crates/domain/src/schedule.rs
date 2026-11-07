//! Règles pures de l'échéancier (étape 06 ; F07, F08, DP-94). Aucune horloge ici : « aujourd'hui » est un paramètre, calculé par
//! l'appelant au fuseau de l'organisation (R-002, DP-50). Une seule fonction de statut pour toutes les lectures (ARCHITECTURE §4.9).

use chrono::{Datelike, Days, Months, NaiveDate};

/// Premier jour du mois d'une date.
pub fn first_of_month(d: NaiveDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(d.year(), d.month(), 1)
}

/// Dernier jour du mois d'une période (1er du mois).
pub fn last_day_of_month(period: NaiveDate) -> Option<NaiveDate> {
    first_of_month(period)?
        .checked_add_months(Months::new(1))?
        .checked_sub_days(Days::new(1))
}

/// R-098 : date d'échéance d'une période (1er du mois) pour un jour d'échéance de 1 à 31 ; un jour absent du mois (29, 30, 31)
/// donne le dernier jour. `None` si la période n'est pas un 1er du mois ou si le jour est hors bornes.
pub fn due_date_for(period: NaiveDate, due_day: u8) -> Option<NaiveDate> {
    if period.day() != 1 || !(1..=31).contains(&due_day) {
        return None;
    }
    let last = last_day_of_month(period)?;
    NaiveDate::from_ymd_opt(period.year(), period.month(), u32::from(due_day).min(last.day()))
}

/// Ce qu'il faut savoir d'un bail pour lister ses périodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseWindow {
    pub start_date: NaiveDate,
    /// Jour de l'activation (fuseau de l'organisation).
    pub activated_on: NaiveDate,
    /// Fin effective : résiliation, sinon départ, sinon date de fin ; `None` pour un bail sans fin.
    pub effective_end: Option<NaiveDate>,
    pub due_day: u8,
}

/// Garde-fou : jamais plus de 100 ans de périodes d'un coup.
const MAX_PERIODS: usize = 1_200;

/// Périodes (1er du mois, date d'échéance) qu'un bail en cours doit avoir à la date `today` :
/// - à partir du mois de la date de début, ou du mois de l'activation si la date de début est passée (R-100) ; le premier mois est
///   une période entière, sans prorata (R-099) ;
/// - jusqu'au mois qui contient la fin effective, inclus (R-030, DP-94) ;
/// - seulement celles dont la date d'échéance tombe d'ici `horizon_days` jours (R-029 : 30), les échues comprises.
///
/// Le résultat ne dépend que de ses paramètres : le rejouer donne la même liste (l'unicité en base fait le reste, R-027, E-03).
pub fn periods_to_create(w: &LeaseWindow, today: NaiveDate, horizon_days: u32) -> Vec<(NaiveDate, NaiveDate)> {
    let Some(limit) = today.checked_add_days(Days::new(u64::from(horizon_days))) else {
        return vec![];
    };
    let Some(mut period) = first_of_month(w.start_date.max(w.activated_on)) else {
        return vec![];
    };
    let last_period = w.effective_end.and_then(first_of_month);
    let mut out = vec![];
    while out.len() < MAX_PERIODS {
        if last_period.is_some_and(|last| period > last) {
            break;
        }
        let Some(due) = due_date_for(period, w.due_day) else {
            break;
        };
        if due > limit {
            break;
        }
        out.push((period, due));
        match period.checked_add_months(Months::new(1)) {
            Some(next) => period = next,
            None => break,
        }
    }
    out
}

/// Avenant daté (R-023) : nouveau loyer et/ou nouvelles charges. `I` est l'identifiant de l'avenant chez l'appelant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amendment<I> {
    pub id: I,
    pub effective_from: NaiveDate,
    /// Jour de la saisie (fuseau de l'organisation).
    pub entered_on: NaiveDate,
    pub rent: Option<i64>,
    pub charges: Option<i64>,
}

impl<I> Amendment<I> {
    /// R-104 : un avenant vaut pour une échéance dont la date est atteinte par sa date d'effet (« à partir du ») **et** postérieure
    /// à sa date de saisie ; une échéance échue garde l'ancien montant.
    pub fn applies_to(&self, due_date: NaiveDate) -> bool {
        due_date >= self.effective_from && due_date > self.entered_on
    }
}

/// Loyer et charges en vigueur pour une échéance, avec l'avenant d'où vient chaque montant (`None` : conditions du bail).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terms<I> {
    pub rent: (i64, Option<I>),
    pub charges: (i64, Option<I>),
}

/// Conditions en vigueur à une date d'échéance : pour le loyer comme pour les charges, le dernier avenant applicable qui les fixe.
pub fn terms_at<I: Copy>(
    base_rent: i64,
    base_charges: i64,
    amendments: &[Amendment<I>],
    due_date: NaiveDate,
) -> Terms<I> {
    let mut applicable: Vec<&Amendment<I>> = amendments.iter().filter(|a| a.applies_to(due_date)).collect();
    applicable.sort_by_key(|a| (a.effective_from, a.entered_on));
    let rent = applicable
        .iter()
        .rev()
        .find_map(|a| a.rent.map(|r| (r, Some(a.id))))
        .unwrap_or((base_rent, None));
    let charges = applicable
        .iter()
        .rev()
        .find_map(|a| a.charges.map(|c| (c, Some(a.id))))
        .unwrap_or((base_charges, None));
    Terms { rent, charges }
}

/// Statut d'une échéance (R-032). Jamais stocké : il dépend du jour (R-034, DP-51).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallmentStatus {
    Upcoming,
    Due,
    Partial,
    Paid,
    Late,
}

impl InstallmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Due => "due",
            Self::Partial => "partial",
            Self::Paid => "paid",
            Self::Late => "late",
        }
    }
}

/// R-033, R-095, DP-13, DP-27 : payée si l'affecté couvre le dû (un dû nul est réglé, DP-94) ; sinon en retard dès J+1, même
/// partiellement payée ; sinon partielle si quelque chose est affecté ; sinon due le jour même ; sinon à venir.
pub fn installment_status(due: i64, allocated: i64, due_date: NaiveDate, today: NaiveDate) -> InstallmentStatus {
    if allocated >= due {
        InstallmentStatus::Paid
    } else if today > due_date {
        InstallmentStatus::Late
    } else if allocated > 0 {
        InstallmentStatus::Partial
    } else if today == due_date {
        InstallmentStatus::Due
    } else {
        InstallmentStatus::Upcoming
    }
}

/// Jours de retard : 0 jusqu'à la date d'échéance, puis le nombre de jours depuis.
pub fn days_late(due_date: NaiveDate, today: NaiveDate) -> i64 {
    if today > due_date {
        today.signed_duration_since(due_date).num_days()
    } else {
        0
    }
}

/// R-024, E-22, DP-94 : dépôt strictement supérieur à deux mois de loyer (charges exclues), bail d'habitation seulement.
pub fn deposit_above_two_months(residential: bool, deposit: i64, rent: i64) -> bool {
    residential && rent.checked_mul(2).is_some_and(|two_months| deposit > two_months)
}
