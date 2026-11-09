#![forbid(unsafe_code)]

//! Channel adapter trait and format derivation.

use async_trait::async_trait;
use platform_hub_udm::{InventoryRecord, UnifiedOrder};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Capabilities a channel adapter supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub fetch_orders: bool,
    pub push_inventory: bool,
    pub webhook: bool,
}

/// Channel adapter trait — implemented per platform (momo, PChome, Cyberbiz, ...).
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn platform_id(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError>;
    async fn push_inventory(&self, records: &[InventoryRecord]) -> Result<(), ChannelError>;
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
    async fn health(&self) -> Result<(), ChannelError>;
}

/// Bidirectional format derivation.
pub trait FormatDeriver: Send + Sync {
    fn platform_to_udm(&self, raw: &[u8]) -> Result<Vec<UnifiedOrder>, ChannelError>;
    fn udm_to_platform(&self, records: &[InventoryRecord]) -> Result<Vec<u8>, ChannelError>;
}

/// Column contract — name-based resolution, hard failure on missing required columns.
#[derive(Debug, Clone)]
pub struct ColumnContract {
    pub name: &'static str,
    pub required: Vec<&'static str>,
    pub optional: Vec<&'static str>,
}

impl ColumnContract {
    pub fn validate_header(&self, header: &[String]) -> Result<(), ChannelError> {
        let header_set: std::collections::HashSet<&str> =
            header.iter().map(|s| s.as_str()).collect();
        let missing: Vec<&str> = self
            .required
            .iter()
            .copied()
            .filter(|col| !header_set.contains(col))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(ChannelError::MissingColumns {
                contract: self.name,
                missing,
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("missing required columns for contract {contract}: {missing:?}")]
    MissingColumns {
        contract: &'static str,
        missing: Vec<&'static str>,
    },
    #[error("http error: {0}")]
    Http(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("rate limited")]
    RateLimited,
    #[error("circuit open for {0}: failing fast until half-open probe")]
    CircuitOpen(&'static str),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

// ============================================================================
// G2.2 (HP-05): circuit breaker decorator.
//
// Wraps any ChannelAdapter without touching the trait or the
// PendingContractAdapter semantics: consecutive failures open the circuit,
// the circuit fast-fails for `open_duration`, then one probe path runs in
// half-open; enough probe successes close it again. Fault-injection testable
// through an injectable clock (FAULT-07).
// ============================================================================

/// Tuning for the breaker. Defaults: open after 3 consecutive failures,
/// re-close after 2 probe successes, stay open 30s.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub half_open_successes: u32,
    pub open_duration: std::time::Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            half_open_successes: 2,
            open_duration: std::time::Duration::from_secs(30),
        }
    }
}

type ClockFn = std::sync::Arc<dyn Fn() -> std::time::Instant + Send + Sync>;

#[derive(Debug)]
enum BreakerState {
    Closed { consecutive_failures: u32 },
    Open { opened_at: std::time::Instant },
    HalfOpen { consecutive_successes: u32 },
}

/// Short-circuit counters, exposed for dashboards (HP-06 observability).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CircuitStats {
    pub total_calls: u32,
    pub short_circuited: u32,
    pub transitions_to_open: u32,
}

/// Decorator implementing [`ChannelAdapter`] around an inner adapter.
pub struct CircuitBreakerAdapter<A: ChannelAdapter> {
    inner: A,
    platform: &'static str,
    config: CircuitBreakerConfig,
    state: std::sync::Mutex<BreakerState>,
    stats: std::sync::Mutex<CircuitStats>,
    clock: ClockFn,
}

impl<A: ChannelAdapter> CircuitBreakerAdapter<A> {
    pub fn new(inner: A, platform: &'static str) -> Self {
        Self::with_config(inner, platform, CircuitBreakerConfig::default())
    }

    pub fn with_config(inner: A, platform: &'static str, config: CircuitBreakerConfig) -> Self {
        Self {
            inner,
            platform,
            config,
            state: std::sync::Mutex::new(BreakerState::Closed {
                consecutive_failures: 0,
            }),
            stats: std::sync::Mutex::new(CircuitStats::default()),
            clock: std::sync::Arc::new(std::time::Instant::now),
        }
    }

    /// Test seam: inject a controllable clock (FAULT-07).
    pub fn with_clock(
        inner: A,
        platform: &'static str,
        config: CircuitBreakerConfig,
        clock: ClockFn,
    ) -> Self {
        Self {
            inner,
            platform,
            config,
            state: std::sync::Mutex::new(BreakerState::Closed {
                consecutive_failures: 0,
            }),
            stats: std::sync::Mutex::new(CircuitStats::default()),
            clock,
        }
    }

    pub fn stats(&self) -> CircuitStats {
        *self.stats.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Admission: fast-fail while Open; promote to HalfOpen when the window
    /// elapses. Returns Err(CircuitOpen) when the call must not run.
    fn admit(&self) -> Result<(), ChannelError> {
        let now = (self.clock)();
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if let BreakerState::Open { opened_at } = *state {
            if now.duration_since(opened_at) < self.config.open_duration {
                let mut stats = self.stats.lock().unwrap_or_else(|p| p.into_inner());
                stats.total_calls += 1;
                stats.short_circuited += 1;
                return Err(ChannelError::CircuitOpen(self.platform));
            }
            *state = BreakerState::HalfOpen {
                consecutive_successes: 0,
            };
        }
        let mut stats = self.stats.lock().unwrap_or_else(|p| p.into_inner());
        stats.total_calls += 1;
        Ok(())
    }

    /// Outcome recording after the awaited call completes.
    fn record(&self, ok: bool) {
        let now = (self.clock)();
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        let mut stats = self.stats.lock().unwrap_or_else(|p| p.into_inner());
        match (&*state, ok) {
            (_, true) => {
                *state = match *state {
                    BreakerState::HalfOpen {
                        consecutive_successes,
                    } => {
                        let successes = consecutive_successes + 1;
                        if successes >= self.config.half_open_successes {
                            BreakerState::Closed {
                                consecutive_failures: 0,
                            }
                        } else {
                            BreakerState::HalfOpen {
                                consecutive_successes: successes,
                            }
                        }
                    }
                    _ => BreakerState::Closed {
                        consecutive_failures: 0,
                    },
                };
            }
            (BreakerState::HalfOpen { .. }, false) => {
                // Probe failed: reopen immediately (no retry storm).
                stats.transitions_to_open += 1;
                *state = BreakerState::Open { opened_at: now };
            }
            (_, false) => {
                if let BreakerState::Closed {
                    consecutive_failures,
                } = *state
                {
                    let failures = consecutive_failures + 1;
                    if failures >= self.config.failure_threshold {
                        stats.transitions_to_open += 1;
                        *state = BreakerState::Open { opened_at: now };
                    } else {
                        *state = BreakerState::Closed {
                            consecutive_failures: failures,
                        };
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<A: ChannelAdapter> ChannelAdapter for CircuitBreakerAdapter<A> {
    fn platform_id(&self) -> &'static str {
        self.platform
    }

    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError> {
        self.admit()?;
        let result = self.inner.fetch_orders().await;
        self.record(result.is_ok());
        result
    }

    async fn push_inventory(&self, records: &[InventoryRecord]) -> Result<(), ChannelError> {
        self.admit()?;
        let result = self.inner.push_inventory(records).await;
        self.record(result.is_ok());
        result
    }

    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool {
        self.inner.verify_webhook_signature(payload, signature)
    }

    async fn health(&self) -> Result<(), ChannelError> {
        self.admit()?;
        let result = self.inner.health().await;
        self.record(result.is_ok());
        result
    }
}

/// Adapter stub for platforms whose official API contract has not been
/// obtained yet (Phase E1 / E7). Every operation returns
/// `ChannelError::NotImplemented` referencing the pending contract, and
/// `contract_status()` reports `pending_contract`. No contract fact is ever
/// guessed (Knowledge research-protocol §1.1).
pub struct PendingContractAdapter {
    platform_id: &'static str,
    spec_request_ref: &'static str,
}

impl PendingContractAdapter {
    pub fn new(platform_id: &'static str, spec_request_ref: &'static str) -> Self {
        Self {
            platform_id,
            spec_request_ref,
        }
    }

    pub fn contract_status(&self) -> &'static str {
        "pending_contract"
    }

    pub fn spec_request_ref(&self) -> &'static str {
        self.spec_request_ref
    }
}

#[async_trait]
impl ChannelAdapter for PendingContractAdapter {
    fn platform_id(&self) -> &'static str {
        self.platform_id
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fetch_orders: false,
            push_inventory: false,
            webhook: false,
        }
    }

    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }

    async fn push_inventory(&self, _records: &[InventoryRecord]) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }

    fn verify_webhook_signature(&self, _payload: &[u8], _signature: &str) -> bool {
        false
    }

    async fn health(&self) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fetch_orders={} push_inventory={} webhook={}",
            self.fetch_orders, self.push_inventory, self.webhook
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_contract_rejects_missing() {
        let c = ColumnContract {
            name: "test",
            required: vec!["A", "B"],
            optional: vec![],
        };
        assert!(c.validate_header(&["A".into()]).is_err());
        assert!(c.validate_header(&["A".into(), "B".into()]).is_ok());
    }

    #[tokio::test]
    async fn pending_contract_adapter_refuses_all_operations() {
        let pchome = PendingContractAdapter::new("pchome", "OQ-F2-5 (PChome spec request)");
        let cyberbiz = PendingContractAdapter::new("cyberbiz", "OQ-F2-5 (Cyberbiz spec request)");

        for adapter in [&pchome, &cyberbiz] {
            assert_eq!(adapter.contract_status(), "pending_contract");
            let caps = adapter.capabilities();
            assert!(!caps.fetch_orders && !caps.push_inventory && !caps.webhook);
            assert!(adapter.fetch_orders().await.is_err());
            assert!(adapter.health().await.is_err());
            assert!(!adapter.verify_webhook_signature(b"{}", "sig"));
        }
    }

    // ------------------------------------------------------------------
    // G2.2 (HP-05, FAULT-07): circuit breaker — open on threshold,
    // fast-fail, half-open probe, re-close, reopen on probe failure.
    // ------------------------------------------------------------------

    /// Scripted adapter: each call consumes the next outcome from the plan.
    struct ScriptedAdapter {
        plan: std::sync::Mutex<std::collections::VecDeque<bool>>, // true = Ok
        calls: std::sync::atomic::AtomicU32,
    }

    impl ScriptedAdapter {
        fn new(plan: &[bool]) -> Self {
            Self {
                plan: std::sync::Mutex::new(plan.iter().copied().collect()),
                calls: std::sync::atomic::AtomicU32::new(0),
            }
        }

        fn calls(&self) -> u32 {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ChannelAdapter for ScriptedAdapter {
        fn platform_id(&self) -> &'static str {
            "scripted"
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities {
                fetch_orders: true,
                push_inventory: false,
                webhook: false,
            }
        }

        async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let next = self
                .plan
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .pop_front();
            match next {
                Some(true) => Ok(vec![]),
                _ => Err(ChannelError::Http("injected failure".into())),
            }
        }

        async fn push_inventory(&self, _records: &[InventoryRecord]) -> Result<(), ChannelError> {
            Err(ChannelError::NotImplemented("scripted".into()))
        }

        fn verify_webhook_signature(&self, _payload: &[u8], _signature: &str) -> bool {
            false
        }

        async fn health(&self) -> Result<(), ChannelError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let next = self
                .plan
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .pop_front();
            match next {
                Some(true) => Ok(()),
                _ => Err(ChannelError::Http("injected failure".into())),
            }
        }
    }

    /// Controllable clock for deterministic open-window tests.
    struct FakeClock(std::sync::Mutex<std::time::Instant>);

    impl FakeClock {
        fn start() -> (std::sync::Arc<FakeClock>, ClockFn) {
            let clock =
                std::sync::Arc::new(FakeClock(std::sync::Mutex::new(std::time::Instant::now())));
            let getter: ClockFn = {
                let handle = clock.clone();
                std::sync::Arc::new(move || *handle.0.lock().unwrap_or_else(|p| p.into_inner()))
            };
            (clock, getter)
        }

        fn advance(&self, by: std::time::Duration) {
            *self.0.lock().unwrap_or_else(|p| p.into_inner()) += by;
        }
    }

    fn breaker_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            half_open_successes: 2,
            open_duration: std::time::Duration::from_secs(30),
        }
    }

    #[tokio::test]
    async fn threshold_failures_open_and_fast_fail() {
        let adapter = ScriptedAdapter::new(&[false, false, false]);
        let breaker = CircuitBreakerAdapter::with_config(adapter, "scripted", breaker_config());

        // Three consecutive failures reach the threshold.
        for _ in 0..3 {
            assert!(breaker.health().await.is_err());
        }
        // The circuit is open: the next call short-circuits without
        // reaching the inner adapter (call count stays at 3).
        let err = breaker.health().await.unwrap_err();
        assert!(matches!(err, ChannelError::CircuitOpen("scripted")));
        assert_eq!(breaker.stats().short_circuited, 1);
        assert_eq!(
            breaker.inner.calls(),
            3,
            "open circuit must not call the adapter"
        );
        assert_eq!(breaker.stats().transitions_to_open, 1);
    }

    #[tokio::test]
    async fn open_recloses_via_half_open_probes() {
        let scripted = ScriptedAdapter::new(&[false, false, false, true, true, true]);
        let (fake_clock, clock_fn) = FakeClock::start();
        let breaker =
            CircuitBreakerAdapter::with_clock(scripted, "scripted", breaker_config(), clock_fn);

        // Trip the breaker.
        for _ in 0..3 {
            let _ = breaker.health().await;
        }
        assert!(matches!(
            breaker.health().await,
            Err(ChannelError::CircuitOpen(_))
        ));

        // Advance past the open window: next call is a half-open probe.
        fake_clock.advance(std::time::Duration::from_secs(31));
        assert!(breaker.health().await.is_ok(), "first probe succeeds");
        assert!(breaker.health().await.is_ok(), "second probe closes it");

        // Fully closed again: a normal call flows through.
        assert!(breaker.health().await.is_ok());
        assert_eq!(breaker.stats().transitions_to_open, 1);
    }

    #[tokio::test]
    async fn half_open_failure_reopens_immediately() {
        let scripted = ScriptedAdapter::new(&[false, false, false, false, false]);
        let (fake_clock, clock_fn) = FakeClock::start();
        let breaker =
            CircuitBreakerAdapter::with_clock(scripted, "scripted", breaker_config(), clock_fn);

        for _ in 0..3 {
            let _ = breaker.health().await;
        }
        fake_clock.advance(std::time::Duration::from_secs(31));

        // Probe fails: circuit reopens without waiting for the window again.
        assert!(breaker.health().await.is_err());
        assert!(matches!(
            breaker.health().await,
            Err(ChannelError::CircuitOpen(_))
        ));
        assert_eq!(breaker.stats().transitions_to_open, 2);
    }
}
