//! Limitation des tentatives de connexion (SECURITY.md §3.3, F01 §4, DP-85). Fonctions pures : les lectures de `login_attempts`
//! sont faites ailleurs et passées ici, ce qui permet de tester les cas chiffrés sans base ni horloge.

use chrono::{DateTime, Duration, Utc};

/// Échecs consécutifs (depuis la dernière réussite) à partir desquels une attente est imposée.
pub const FAILURES_BEFORE_WAIT: usize = 5;
/// Échecs depuis une même adresse, dans la fenêtre, qui déclenchent un refus.
pub const IP_FAILURES: usize = 20;
pub const IP_WINDOW_MINUTES: i64 = 10;
pub const IP_BLOCK_MINUTES: i64 = 15;

/// Attente restante pour un identifiant. `failures_desc` : heures des échecs consécutifs depuis la dernière réussite, du plus récent
/// au plus ancien. 5ᵉ échec → 1 minute, 6ᵉ → 5 minutes, 7ᵉ et suivants → 15 minutes, comptées depuis le dernier échec.
pub fn identifier_wait(failures_desc: &[DateTime<Utc>], now: DateTime<Utc>) -> Option<Duration> {
    let last = failures_desc.first()?;
    let wait = match failures_desc.len() {
        0..FAILURES_BEFORE_WAIT => return None,
        FAILURES_BEFORE_WAIT => Duration::minutes(1),
        6 => Duration::minutes(5),
        _ => Duration::minutes(15),
    };
    let remaining = *last + wait - now;
    (remaining > Duration::zero()).then_some(remaining)
}

/// Attente restante pour une adresse. `failures` : heures des échecs récents de cette adresse (dans l'ordre quelconque, au moins les
/// 25 dernières minutes). Une série de 20 échecs en 10 minutes bloque l'adresse 15 minutes à partir du dernier échec de la série.
pub fn ip_wait(failures: &[DateTime<Utc>], now: DateTime<Utc>) -> Option<Duration> {
    let mut times = failures.to_vec();
    times.sort_unstable();
    let window = Duration::minutes(IP_WINDOW_MINUTES);
    let mut blocked_until: Option<DateTime<Utc>> = None;
    for i in (IP_FAILURES - 1)..times.len() {
        let (Some(first), Some(last)) = (times.get(i + 1 - IP_FAILURES), times.get(i)) else {
            continue;
        };
        if *last - *first <= window {
            let until = *last + Duration::minutes(IP_BLOCK_MINUTES);
            blocked_until = Some(blocked_until.map_or(until, |b| b.max(until)));
        }
    }
    let remaining = blocked_until? - now;
    (remaining > Duration::zero()).then_some(remaining)
}

/// La plus grande des deux attentes, en secondes entières arrondies par excès (valeur de `Retry-After`).
pub fn retry_after_seconds(a: Option<Duration>, b: Option<Duration>) -> Option<i64> {
    let longest = match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, None) => x,
        (None, y) => y,
    }?;
    let ms = longest.num_milliseconds();
    Some((ms + 999) / 1000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 13, 9, 0, 0).unwrap()
    }

    /// `n` échecs espacés de 10 secondes, le dernier à `t0`, du plus récent au plus ancien.
    fn failures_ending_at_t0(n: usize) -> Vec<DateTime<Utc>> {
        (0..n)
            .map(|i| t0() - Duration::seconds(10 * i64::try_from(i).unwrap()))
            .collect()
    }

    #[test]
    fn avant_le_cinquieme_echec_aucune_attente() {
        for n in 0..5 {
            assert_eq!(identifier_wait(&failures_ending_at_t0(n), t0()), None, "{n} échecs");
        }
    }

    #[test]
    fn cinq_echecs_une_minute_puis_cinq_puis_quinze() {
        assert_eq!(
            identifier_wait(&failures_ending_at_t0(5), t0()),
            Some(Duration::minutes(1))
        );
        assert_eq!(
            identifier_wait(&failures_ending_at_t0(6), t0()),
            Some(Duration::minutes(5))
        );
        assert_eq!(
            identifier_wait(&failures_ending_at_t0(7), t0()),
            Some(Duration::minutes(15))
        );
        assert_eq!(
            identifier_wait(&failures_ending_at_t0(8), t0()),
            Some(Duration::minutes(15))
        );
        assert_eq!(
            identifier_wait(&failures_ending_at_t0(30), t0()),
            Some(Duration::minutes(15))
        );
    }

    #[test]
    fn l_attente_decroit_et_se_termine() {
        let f = failures_ending_at_t0(5);
        assert_eq!(
            identifier_wait(&f, t0() + Duration::seconds(30)),
            Some(Duration::seconds(30))
        );
        assert_eq!(
            identifier_wait(&f, t0() + Duration::minutes(1)),
            None,
            "à la minute pile, la tentative est permise"
        );
        assert_eq!(identifier_wait(&f, t0() + Duration::minutes(10)), None);
    }

    #[test]
    fn vingt_echecs_en_dix_minutes_bloquent_l_adresse_quinze_minutes() {
        let nineteen: Vec<_> = (0..19).map(|i| t0() - Duration::seconds(20 * i)).collect();
        assert_eq!(ip_wait(&nineteen, t0()), None, "19 échecs : pas de blocage");
        let twenty: Vec<_> = (0..20).map(|i| t0() - Duration::seconds(20 * i)).collect();
        assert_eq!(ip_wait(&twenty, t0()), Some(Duration::minutes(15)));
        // 12 minutes plus tard : la fenêtre de 10 minutes est vide mais le blocage de 15 minutes dure encore
        assert_eq!(
            ip_wait(&twenty, t0() + Duration::minutes(12)),
            Some(Duration::minutes(3))
        );
        assert_eq!(ip_wait(&twenty, t0() + Duration::minutes(15)), None);
    }

    #[test]
    fn vingt_echecs_etales_sur_plus_de_dix_minutes_ne_bloquent_pas() {
        let spread: Vec<_> = (0..20).map(|i| t0() - Duration::seconds(40 * i)).collect(); // 760 s au total
        assert_eq!(ip_wait(&spread, t0()), None);
    }

    #[test]
    fn retry_after_prend_la_plus_longue_attente_arrondie_par_exces() {
        assert_eq!(retry_after_seconds(None, None), None);
        assert_eq!(retry_after_seconds(Some(Duration::seconds(60)), None), Some(60));
        assert_eq!(
            retry_after_seconds(Some(Duration::milliseconds(59_001)), Some(Duration::seconds(10))),
            Some(60)
        );
        assert_eq!(
            retry_after_seconds(Some(Duration::minutes(1)), Some(Duration::minutes(15))),
            Some(900)
        );
    }
}
