//! Limitation générale du débit de l'API (API_CONTRACTS.md §4.7) : fenêtre glissante en mémoire, par session et par adresse.
//!
//! Écart assumé (DP-85) : les compteurs de **connexion** sont dans PostgreSQL (`login_attempts`, partagés entre instances) ;
//! ce plafond général est **en mémoire**, valable pour une instance (le pilote en a une). À déplacer dans PostgreSQL avant d'en
//! lancer plusieurs.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const SWEEP_ABOVE: usize = 10_000;

#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compte un appel. `Err(secondes)` : plafond atteint, attente avant le prochain appel accepté (arrondie par excès).
    pub fn hit(&self, key: &str, limit: usize, window: Duration) -> Result<(), i64> {
        self.hit_at(key, limit, window, Instant::now())
    }

    fn hit_at(&self, key: &str, limit: usize, window: Duration, now: Instant) -> Result<(), i64> {
        let mut map = self.windows.lock().unwrap_or_else(PoisonError::into_inner);
        if map.len() > SWEEP_ABOVE {
            map.retain(|_, q| q.back().is_some_and(|t| now.duration_since(*t) < window));
        }
        let queue = map.entry(key.to_string()).or_default();
        while let Some(front) = queue.front() {
            if now.duration_since(*front) >= window {
                queue.pop_front();
            } else {
                break;
            }
        }
        if queue.len() >= limit {
            let oldest = queue.front().copied().unwrap_or(now);
            let wait = window.saturating_sub(now.duration_since(oldest));
            let seconds = i64::try_from(wait.as_millis().div_ceil(1000)).unwrap_or(i64::MAX);
            return Err(seconds.max(1));
        }
        queue.push_back(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_plafond_est_atteint_puis_la_fenetre_glisse() {
        let limiter = RateLimiter::new();
        let t0 = Instant::now();
        let window = Duration::from_secs(60);
        for i in 0..3 {
            assert_eq!(limiter.hit_at("k", 3, window, t0 + Duration::from_secs(i)), Ok(()));
        }
        // 4ᵉ appel à t0+10 s : refusé, il faut attendre que le plus ancien (t0) sorte de la fenêtre : 50 s
        assert_eq!(limiter.hit_at("k", 3, window, t0 + Duration::from_secs(10)), Err(50));
        // un refus ne compte pas
        assert_eq!(limiter.hit_at("k", 3, window, t0 + Duration::from_secs(10)), Err(50));
        // à t0+60 s le premier appel est sorti : un appel passe
        assert_eq!(limiter.hit_at("k", 3, window, t0 + Duration::from_secs(60)), Ok(()));
        // les clés sont indépendantes
        assert_eq!(limiter.hit_at("autre", 3, window, t0), Ok(()));
    }
}
