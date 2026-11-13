#![forbid(unsafe_code)]
//! G4.11 (D4-03) — per-session sliding-window rate limit for `/api/search`.
//!
//! 30 requests per 60 seconds per session. Exceeding returns 429 with
//! `Retry-After` (seconds). In-memory (process-local) — adequate for the
//! single-node deployment; a distributed limiter is out of scope.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const SEARCH_LIMIT: usize = 30;
pub const SEARCH_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    RetryAfter(u64),
}

#[derive(Default)]
pub struct SearchRateLimiter {
    inner: Mutex<HashMap<String, Vec<Instant>>>,
}

impl SearchRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a hit and decide. Keys are session ids.
    pub fn check(&self, key: &str) -> Decision {
        let now = Instant::now();
        let mut map = self.inner.lock().expect("search rate limiter lock");
        let hits = map.entry(key.to_string()).or_default();
        hits.retain(|t| now.duration_since(*t) < SEARCH_WINDOW);
        if hits.len() >= SEARCH_LIMIT {
            let oldest = *hits.first().expect("non-empty bucket");
            let wait = SEARCH_WINDOW.saturating_sub(now.duration_since(oldest));
            return Decision::RetryAfter(wait.as_secs().max(1));
        }
        hits.push(now);
        Decision::Allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_limit_then_retries() {
        let limiter = SearchRateLimiter::new();
        for _ in 0..SEARCH_LIMIT {
            assert_eq!(limiter.check("s1"), Decision::Allowed);
        }
        match limiter.check("s1") {
            Decision::RetryAfter(secs) => assert!((1..=60).contains(&secs)),
            Decision::Allowed => panic!("limit must reject the 31st request"),
        }
    }

    #[test]
    fn buckets_are_per_session() {
        let limiter = SearchRateLimiter::new();
        for _ in 0..SEARCH_LIMIT {
            assert_eq!(limiter.check("a"), Decision::Allowed);
        }
        assert_eq!(
            limiter.check("b"),
            Decision::Allowed,
            "other session unaffected"
        );
    }
}
