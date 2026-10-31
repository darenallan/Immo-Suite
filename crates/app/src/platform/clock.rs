//! Service d'horloge injectable (R-002, DP-50, CODING_RULES.md §4.3).
//!
//! Aucune autre partie du code ne lit l'heure : « aujourd'hui » détermine le retard, l'antidatage et la date future.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

/// Fuseau des organisations (R-002) : UTC+0 sans changement d'heure, mais le code ne repose pas sur cette coïncidence.
pub const OUAGADOUGOU: Tz = chrono_tz::Africa::Ouagadougou;

/// Source de l'heure et de la date locale d'une organisation.
pub trait Clock: Send + Sync {
    /// L'instant courant, en UTC.
    fn now(&self) -> DateTime<Utc>;

    /// La date du jour au fuseau de l'organisation (jamais la date du serveur).
    fn local_date(&self, tz: Tz) -> NaiveDate {
        self.now().with_timezone(&tz).date_naive()
    }
}

/// Horloge du système, utilisée en développement, en essai et en production.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        // Seul appel autorisé à l'horloge du système : `platform::clock` (CODING_RULES.md §3.1, décision n° 75).
        #[allow(clippy::disallowed_methods)]
        Utc::now()
    }
}

/// Horloge figée, pour les tests (le jeu de démonstration est au 13/11/2026 09:15).
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(DateTime<Utc>);

impl FixedClock {
    /// Horloge figée à un instant UTC.
    pub fn at(instant: DateTime<Utc>) -> Self {
        Self(instant)
    }

    /// Horloge figée à une date et une heure UTC ; `None` si elles n'existent pas.
    pub fn at_ymd_hm(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<Self> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .map(Self)
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horloge_figee_au_13_novembre_2026() {
        let clock = FixedClock::at_ymd_hm(2026, 11, 13, 9, 15).unwrap();
        assert_eq!(
            clock.local_date(OUAGADOUGOU),
            NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()
        );
    }

    #[test]
    fn la_date_locale_suit_le_fuseau_de_l_organisation() {
        // 23 h 30 UTC le 13 : déjà le 14 à Auckland, encore le 13 à Ouagadougou
        let clock = FixedClock::at_ymd_hm(2026, 11, 13, 23, 30).unwrap();
        assert_eq!(
            clock.local_date(OUAGADOUGOU),
            NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()
        );
        assert_eq!(
            clock.local_date(chrono_tz::Pacific::Auckland),
            NaiveDate::from_ymd_opt(2026, 11, 14).unwrap()
        );
    }

    #[test]
    fn passage_d_annee_e27() {
        let avant = FixedClock::at_ymd_hm(2026, 12, 31, 23, 59).unwrap();
        let apres = FixedClock::at_ymd_hm(2027, 1, 1, 0, 0).unwrap();
        assert_eq!(
            avant.local_date(OUAGADOUGOU),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        );
        assert_eq!(
            apres.local_date(OUAGADOUGOU),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
    }

    #[test]
    fn l_horloge_du_systeme_donne_un_instant_recent() {
        let now = SystemClock.now();
        assert!(now.timestamp() > 1_700_000_000);
    }
}
