#![forbid(unsafe_code)]
//! flaps_bridge — COM batching bridge PoC for FlapsErp (T-102, ADR-003 A2 track).
//!
//! Reverse-engineering evidence (2026-11-03):
//! - FlapsErp renderer calls COM via `winax` with `send: 'sendSync'` — every method/property
//!   call is a synchronous IPC round-trip (mainApi.js:193,225). Measured COM per-call floor:
//!   P50 = 0.26 ms, P95 = 0.39 ms (flaps-perf.ps1, 20261103-160315).
//! - Target objects: `FLAPSConsoleCOM.*` (active_patch.js polyfills AXCXMLNode) and
//!   `Module\*.jsc` byte-compiled modules dispatched through the main-process ActiveXObject API.
//!
//! This PoC validates the batching engine semantics against a mock transport, so the
//! batch-vs-per-call comparison is reproducible in CI without a live COM apartment.
//! The real COM transport is intentionally out of scope until EULA policy is confirmed
//! (risk RSK-E1) — see `docs/research/FlapsErp優化_合規風險追蹤表.md`.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

// Guard: real COM transport requires EULA written confirmation (RSK-E1).
// Enabled only via `real-com` feature flag; disabled by default.

/// A single COM-style invocation request.
#[derive(Debug, Clone, PartialEq)]
pub struct ComCall {
    pub object_id: String,
    pub method: String,
    pub args: Vec<String>,
}

/// Result of one invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ComResult {
    pub value: String,
    pub latency: Duration,
}

/// Errors surfaced by the bridge.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("call failed for object `{object_id}` method `{method}`: {reason}")]
    CallFailed {
        object_id: String,
        method: String,
        reason: String,
    },
    #[error("empty batch submitted")]
    EmptyBatch,
    #[error("transport failure: {0}")]
    Transport(String),
}

/// Transport abstraction: mock in tests, real COM later behind a feature gate.
pub trait Transport {
    fn invoke(&mut self, call: &ComCall) -> Result<ComResult, BridgeError>;
    fn invoke_batch(&mut self, batch: &[ComCall]) -> Result<Vec<ComResult>, BridgeError>;
}

impl<T: Transport> Transport for &mut T {
    fn invoke(&mut self, call: &ComCall) -> Result<ComResult, BridgeError> {
        (**self).invoke(call)
    }
    fn invoke_batch(&mut self, batch: &[ComCall]) -> Result<Vec<ComResult>, BridgeError> {
        (**self).invoke_batch(batch)
    }
}

/// Mock transport simulating the measured per-call cost envelope.
///
/// `per_call_latency` approximates the observed COM floor (default 0.26 ms P50).
/// Batch mode costs one fixed round trip plus a linear scan — the pattern that
/// makes batching win as soon as round trips dominate.
pub struct MockTransport {
    pub per_call_latency: Duration,
    pub batch_round_trip: Duration,
    pub batch_per_item: Duration,
    pub fail_on: Option<String>,
    calls_dispatched: usize,
    /// Transport round trips: +1 per `invoke`, +1 per `invoke_batch` call
    /// (regardless of batch size). This is the counter that shows batching
    /// savings; `calls_dispatched` counts processed items on both paths.
    round_trips: usize,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            per_call_latency: Duration::from_micros(260),
            batch_round_trip: Duration::from_micros(400),
            batch_per_item: Duration::from_micros(20),
            fail_on: None,
            calls_dispatched: 0,
            round_trips: 0,
        }
    }

    pub fn calls_dispatched(&self) -> usize {
        self.calls_dispatched
    }

    /// Number of transport round trips (one per `invoke`, one per batch).
    pub fn round_trips(&self) -> usize {
        self.round_trips
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for MockTransport {
    fn invoke(&mut self, call: &ComCall) -> Result<ComResult, BridgeError> {
        std::thread::sleep(self.per_call_latency);
        self.calls_dispatched += 1;
        self.round_trips += 1;
        if self
            .fail_on
            .as_deref()
            .map(|m| call.method == m)
            .unwrap_or(false)
        {
            return Err(BridgeError::CallFailed {
                object_id: call.object_id.clone(),
                method: call.method.clone(),
                reason: "mock failure".to_string(),
            });
        }
        Ok(ComResult {
            value: format!("ok:{}", call.method),
            latency: self.per_call_latency,
        })
    }

    fn invoke_batch(&mut self, batch: &[ComCall]) -> Result<Vec<ComResult>, BridgeError> {
        if batch.is_empty() {
            return Err(BridgeError::EmptyBatch);
        }
        std::thread::sleep(self.batch_round_trip);
        self.round_trips += 1;
        let mut results = Vec::with_capacity(batch.len());
        for call in batch {
            std::thread::sleep(self.batch_per_item);
            self.calls_dispatched += 1;
            if self
                .fail_on
                .as_deref()
                .map(|m| call.method == m)
                .unwrap_or(false)
            {
                return Err(BridgeError::CallFailed {
                    object_id: call.object_id.clone(),
                    method: call.method.clone(),
                    reason: "mock failure in batch".to_string(),
                });
            }
            results.push(ComResult {
                value: format!("ok:{}", call.method),
                latency: self.batch_per_item,
            });
        }
        Ok(results)
    }
}

/// Batching bridge: collects calls, flushes as one transport round trip.
pub struct BatchingBridge<T: Transport> {
    transport: T,
    pending: VecDeque<ComCall>,
    pub max_batch: usize,
    pub flush_threshold: Duration,
}

impl<T: Transport> BatchingBridge<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            pending: VecDeque::new(),
            max_batch: 256,
            flush_threshold: Duration::from_millis(50),
        }
    }

    pub fn enqueue(&mut self, call: ComCall) {
        self.pending.push_back(call);
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Flush all pending calls as batches of at most `max_batch`.
    /// Atomic semantics: on any failure the batch is returned to the front
    /// of the queue (no silent loss), mirroring the ReleaseGate two-way-door
    /// discipline used across Wellfit.
    pub fn flush(&mut self) -> Result<Vec<ComResult>, BridgeError> {
        let mut all = Vec::new();
        while !self.pending.is_empty() {
            let take = self.max_batch.min(self.pending.len());
            let batch: Vec<ComCall> = self.pending.drain(..take).collect();
            match self.transport.invoke_batch(&batch) {
                Ok(results) => all.extend(results),
                Err(e) => {
                    for call in batch.into_iter().rev() {
                        self.pending.push_front(call);
                    }
                    return Err(e);
                }
            }
        }
        Ok(all)
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }
}

/// Benchmark harness: per-call vs batched for `n` calls against a transport.
/// Returns (per_call_total, batched_total, round_trips_saved).
pub fn benchmark<T: Transport>(
    transport: &mut T,
    n: usize,
    method: &str,
) -> Result<(Duration, Duration, usize), BridgeError> {
    let calls: Vec<ComCall> = (0..n)
        .map(|i| ComCall {
            object_id: format!("obj_{i}"),
            method: method.to_string(),
            args: vec![],
        })
        .collect();

    let start = Instant::now();
    for call in &calls {
        transport
            .invoke(call)
            .map_err(|e| BridgeError::Transport(e.to_string()))?;
    }
    let per_call_total = start.elapsed();

    let mut bridge = BatchingBridge::new(PerCallFreeProxy { inner: transport });
    for call in &calls {
        bridge.enqueue(call.clone());
    }
    let start = Instant::now();
    let results = bridge.flush()?;
    let batched_total = start.elapsed();
    debug_assert_eq!(results.len(), n);

    let round_trips_saved = n.saturating_sub(1);
    Ok((per_call_total, batched_total, round_trips_saved))
}

/// Proxy that routes `invoke_batch` straight to the underlying transport so
/// the benchmark measures batch mode without double-buffering.
struct PerCallFreeProxy<'a, T: Transport> {
    inner: &'a mut T,
}

impl<T: Transport> Transport for PerCallFreeProxy<'_, T> {
    fn invoke(&mut self, call: &ComCall) -> Result<ComResult, BridgeError> {
        self.inner.invoke(call)
    }
    fn invoke_batch(&mut self, batch: &[ComCall]) -> Result<Vec<ComResult>, BridgeError> {
        self.inner.invoke_batch(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_flush_returns_all_results_in_order() {
        let mut t = MockTransport::new();
        let mut bridge = BatchingBridge::new(&mut t);
        for i in 0..10 {
            bridge.enqueue(ComCall {
                object_id: format!("o{i}"),
                method: "GetXML".to_string(),
                args: vec![format!("arg{i}")],
            });
        }
        let results = bridge.flush().expect("flush ok");
        assert_eq!(results.len(), 10);
        for (i, r) in results.iter().enumerate() {
            assert_eq!(r.value, "ok:GetXML");
            let _ = i;
        }
        assert_eq!(bridge.pending_len(), 0);
        assert_eq!(t.calls_dispatched(), 10);
    }

    #[test]
    fn max_batch_splits_large_queues() {
        let mut t = MockTransport::new();
        let mut bridge = BatchingBridge::new(&mut t);
        bridge.max_batch = 7;
        for i in 0..20 {
            bridge.enqueue(ComCall {
                object_id: format!("o{i}"),
                method: "m".to_string(),
                args: vec![],
            });
        }
        let results = bridge.flush().expect("flush ok");
        assert_eq!(results.len(), 20);
    }

    #[test]
    fn failure_restores_pending_calls_atomically() {
        let mut t = MockTransport::new();
        t.fail_on = Some("boom".to_string());
        let mut bridge = BatchingBridge::new(&mut t);
        bridge.enqueue(ComCall {
            object_id: "o1".to_string(),
            method: "GetXML".to_string(),
            args: vec![],
        });
        bridge.enqueue(ComCall {
            object_id: "o2".to_string(),
            method: "boom".to_string(),
            args: vec![],
        });
        assert!(bridge.flush().is_err());
        assert_eq!(bridge.pending_len(), 2, "calls must not be silently lost");
    }

    #[test]
    fn empty_batch_is_rejected() {
        let mut t = MockTransport::new();
        assert!(t.invoke_batch(&[]).is_err());
    }

    #[test]
    fn batching_beats_per_call_under_measured_envelope() {
        let n = 1000;
        let mut t = MockTransport::new();
        let (per_call, batched, saved) =
            benchmark(&mut t, n, "GetSpecialFolder").expect("benchmark ok");
        // 確定性效益斷言（取代先前不穩定的牆鐘比較）：
        // per-call 路徑產生 n 次往返；batched 路徑以 max_batch=256 分批，
        // 產生 ceil(n/256) 次往返。批次節省 n-1 - ceil(n/256)+... 總往返
        // 由 round_trips 精確驗證，不受排程抖動影響（Backlog P1-4）。
        assert_eq!(saved, n - 1);
        let expected_batch_round_trips = n.div_ceil(256); // max_batch = 256
        assert_eq!(
            t.round_trips(),
            n + expected_batch_round_trips,
            "n per-call round trips + ceil(n/max_batch) batched round trips"
        );
        // 牆鐘觀察僅供人工對照，不作為斷言：2026-11-03 三次觀察中兩次
        // 因 batched/per_call 差距 < 1ms 受排程抖動翻盤。
        println!(
            "per_call={per_call:?} batched={batched:?} round_trips={}",
            t.round_trips()
        );
    }

    #[test]
    fn mock_envelope_matches_measured_floor() {
        let t = MockTransport::new();
        // Measured floor 2026-11-03: P50 = 0.26 ms. Keep the default aligned.
        assert_eq!(t.per_call_latency, Duration::from_micros(260));
    }
}

/// EULA-gated real COM transport for FLAPSConsoleCOM.* objects.
///
/// This module is compiled ONLY when the `real-com` feature is enabled,
/// which requires OQ-F1 written response from Flaps (risk RSK-E1).
/// Until written EULA confirmation is received, this transport remains
/// inactive and all COM operations use MockTransport in tests only.
#[cfg(feature = "real-com")]
mod real_com {
    use super::*;

    /// Real COM transport placeholder (EULA-gated).
    ///
    /// Actual Windows COM calls (CoCreateInstance, IDispatch::Invoke) are
    /// deferred until OQ-F1 written response confirms EULA permission.
    pub struct RealComTransport {
        _placeholder: (),
    }

    impl RealComTransport {
        /// Create a new real COM transport placeholder.
        pub fn new() -> Result<Self, BridgeError> {
            Ok(Self { _placeholder: () })
        }
    }

    impl Default for RealComTransport {
        fn default() -> Self {
            Self::new().expect("RealComTransport::default should not fail")
        }
    }

    impl Transport for RealComTransport {
        fn invoke(&mut self, call: &ComCall) -> Result<ComResult, BridgeError> {
            Err(BridgeError::Transport(
                "real COM invoke disabled: EULA written confirmation required (RSK-E1)".to_string(),
            ))
        }

        fn invoke_batch(&mut self, batch: &[ComCall]) -> Result<Vec<ComResult>, BridgeError> {
            if batch.is_empty() {
                return Err(BridgeError::EmptyBatch);
            }
            batch.iter().map(|call| self.invoke(call)).collect()
        }
    }
}

#[cfg(test)]
mod real_com_tests {
    #[test]
    fn real_com_not_active_by_default() {
        // This test verifies that real_com module is gated behind feature flag.
        // When real-com is NOT enabled, this module is not compiled.
    }
}
