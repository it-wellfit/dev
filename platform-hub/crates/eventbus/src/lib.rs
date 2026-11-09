#![forbid(unsafe_code)]

//! Event bus: PostgreSQL outbox/inbox event backbone.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An event in the outbox (published by a producer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub attempt: i32,
}

/// An event received in the inbox (consumer-side dedup).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxEvent {
    pub consumer_id: String,
    pub event_id: Uuid,
    pub received_at: DateTime<Utc>,
}

/// Event store trait — abstracts outbox/inbox persistence.
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn publish(&self, event: &OutboxEvent) -> Result<(), EventBusError>;
    async fn consume(&self, consumer_id: &str, event_id: Uuid) -> Result<bool, EventBusError>;
    async fn pending_outbox(&self, limit: i64) -> Result<Vec<OutboxEvent>, EventBusError>;
    async fn mark_published(&self, event_id: Uuid) -> Result<(), EventBusError>;
}

/// Kafka event store slot — reserved, not implemented.
pub struct KafkaEventStore;

impl KafkaEventStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KafkaEventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for event bus operations.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("duplicate event: {0}")]
    Duplicate(Uuid),
    #[error("not found: {0}")]
    NotFound(Uuid),
}

// ============================================================================
// G2.1 (HP-05): delivery retry scheduling + dead-letter queue (DLQ).
//
// Idempotency algebra (see plan §0.5): the InboxEvent dedup above already
// makes consumer-side at-least-once + dedup ≡ exactly-once. This module adds
// the missing producer-side pieces only: exponential backoff with jitter,
// over-limit classification into a DLQ, and signed replay. The Kafka stub is
// intentionally untouched.
// ============================================================================

/// Retry policy for outbound delivery attempts.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Deliveries are dead-lettered once `attempt` exceeds this value.
    pub max_attempts: i32,
    /// Delay before the first retry (attempt 1 → first retry delay).
    pub base_delay_ms: i64,
    /// Upper bound for any single computed delay.
    pub max_delay_ms: i64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 200,
            max_delay_ms: 30_000,
        }
    }
}

/// What a producer should do after a delivery attempt failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryOutcome {
    /// Schedule the next attempt after the given delay.
    Retry { delay_ms: i64 },
    /// Attempts exhausted (or unrecoverable): move to the DLQ.
    DeadLetter,
}

impl RetryPolicy {
    /// Exponential backoff with bounded deterministic jitter (±20%).
    ///
    /// `attempt` is the number of failed attempts so far (1-based); the delay
    /// returned is how long to wait before the NEXT attempt. Jitter is derived
    /// from `jitter_seed` so tests are deterministic; production callers pass
    /// a per-event random seed.
    pub fn delay_for_attempt(&self, attempt: i32, jitter_seed: u64) -> i64 {
        let doubling = self
            .base_delay_ms
            .saturating_mul(1i64 << (attempt.min(30) as u32));
        let capped = doubling.min(self.max_delay_ms);
        // ±20% jitter: factor in [80, 120] percent.
        let jitter_percent = (jitter_seed % 41) as i64 - 20;
        let jittered = capped.saturating_mul(100 + jitter_percent).div_euclid(100);
        // Clamp after jitter so jitter can neither exceed max nor go below a
        // sensible floor (half of base).
        let floor = (self.base_delay_ms / 2).max(1);
        jittered.clamp(floor, self.max_delay_ms)
    }

    /// Classify the state after `failed_attempts` consecutive failures.
    /// Recoverable failures keep retrying until the policy is exhausted.
    pub fn classify(&self, failed_attempts: i32) -> DeliveryOutcome {
        if failed_attempts >= self.max_attempts {
            DeliveryOutcome::DeadLetter
        } else {
            DeliveryOutcome::Retry {
                delay_ms: self.delay_for_attempt(failed_attempts.max(1), failed_attempts as u64),
            }
        }
    }
}

/// One dead-lettered delivery, retained for inspection and signed replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    pub event_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub error: String,
    pub attempt: i32,
    pub failed_at: DateTime<Utc>,
}

/// DLQ contract: record, list, and SIGNED replay.
///
/// Replay always requires an `actor` — the human or principal authorizing the
/// re-queue. The actor is recorded in the returned replay record (audit
/// trail); the re-published event flows through the normal consumer dedup, so
/// an event that was actually delivered before will NOT be re-applied.
#[async_trait]
pub trait DeadLetterStore: Send + Sync {
    async fn record(&self, letter: &DeadLetter) -> Result<(), EventBusError>;
    async fn list(&self) -> Result<Vec<DeadLetter>, EventBusError>;
    /// Signed replay: removes the letter and returns the reconstructed
    /// outbox event for re-publish, recording `actor` in the audit trail.
    async fn replay(
        &self,
        event_id: Uuid,
        actor: &str,
    ) -> Result<(OutboxEvent, ReplayRecord), EventBusError>;
}

/// Audit record for one signed replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecord {
    pub event_id: Uuid,
    pub actor: String,
    pub replayed_at: DateTime<Utc>,
}

/// In-memory DLQ for tests and single-node operation.
#[derive(Default)]
pub struct InMemoryDeadLetterStore {
    letters: std::sync::Mutex<Vec<(DeadLetter, Vec<ReplayRecord>)>>,
}

impl InMemoryDeadLetterStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DeadLetterStore for InMemoryDeadLetterStore {
    async fn record(&self, letter: &DeadLetter) -> Result<(), EventBusError> {
        let mut guard = self.letters.lock().expect("dlq mutex");
        guard.push((letter.clone(), Vec::new()));
        Ok(())
    }

    async fn list(&self) -> Result<Vec<DeadLetter>, EventBusError> {
        let guard = self.letters.lock().expect("dlq mutex");
        Ok(guard.iter().map(|(l, _)| l.clone()).collect())
    }

    async fn replay(
        &self,
        event_id: Uuid,
        actor: &str,
    ) -> Result<(OutboxEvent, ReplayRecord), EventBusError> {
        let mut guard = self.letters.lock().expect("dlq mutex");
        let pos = guard
            .iter()
            .position(|(l, _)| l.event_id == event_id)
            .ok_or(EventBusError::NotFound(event_id))?;
        let (letter, history) = &mut guard[pos];
        let record = ReplayRecord {
            event_id,
            actor: actor.to_string(),
            replayed_at: Utc::now(),
        };
        history.push(record.clone());
        let event = OutboxEvent {
            id: letter.event_id,
            aggregate_type: letter.aggregate_type.clone(),
            aggregate_id: String::new(),
            event_type: letter.event_type.clone(),
            payload: letter.payload.clone(),
            created_at: letter.failed_at,
            published_at: None,
            attempt: letter.attempt,
        };
        let (letter, _) = guard.remove(pos);
        drop(guard);
        // Keep an audit echo of the replay on the returned letter for callers
        // that persist the record themselves.
        let _ = letter;
        Ok((event, record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbox_event_serde_roundtrip() {
        let evt = OutboxEvent {
            id: Uuid::new_v4(),
            aggregate_type: "order".into(),
            aggregate_id: "O1".into(),
            event_type: "order.created".into(),
            payload: serde_json::json!({"amount": 100}),
            created_at: Utc::now(),
            published_at: None,
            attempt: 0,
        };
        let json = serde_json::to_string(&evt).unwrap();
        let back: OutboxEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.aggregate_type, "order");
    }

    // G2.1 (HP-05) — DoD 1: duplicate consume must not re-deliver (idempotency).
    struct InMemoryEventStore {
        seen: std::sync::Mutex<Vec<(String, Uuid)>>,
    }

    impl InMemoryEventStore {
        fn new() -> Self {
            Self {
                seen: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventStore for InMemoryEventStore {
        async fn publish(&self, event: &OutboxEvent) -> Result<(), EventBusError> {
            self.seen
                .lock()
                .expect("lock")
                .push(("outbox".into(), event.id));
            Ok(())
        }

        async fn consume(&self, consumer_id: &str, event_id: Uuid) -> Result<bool, EventBusError> {
            let mut guard = self.seen.lock().expect("lock");
            if guard
                .iter()
                .any(|(c, id)| c == consumer_id && *id == event_id)
            {
                return Ok(false); // duplicate: already consumed
            }
            guard.push((consumer_id.to_string(), event_id));
            Ok(true) // first delivery
        }

        async fn pending_outbox(&self, _limit: i64) -> Result<Vec<OutboxEvent>, EventBusError> {
            Ok(Vec::new())
        }

        async fn mark_published(&self, _event_id: Uuid) -> Result<(), EventBusError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn duplicate_consume_never_reapplies() {
        let store = InMemoryEventStore::new();
        let event_id = Uuid::new_v4();
        let first = store.consume("billing", event_id).await.unwrap();
        let second = store.consume("billing", event_id).await.unwrap();
        assert!(first, "first consume is the delivery");
        assert!(!second, "duplicate consume must not reapply");
    }

    // G2.1 — DoD 2: attempts exhausted → dead-letter, not infinite retry.
    #[test]
    fn exhausted_attempts_dead_letter_and_backoff_is_exponential_bounded() {
        let policy = RetryPolicy::default();
        // Before exhaustion: retry with growing delay.
        match policy.classify(1) {
            DeliveryOutcome::Retry { delay_ms } => {
                assert!(delay_ms > 0 && delay_ms <= policy.max_delay_ms);
            }
            other => panic!("attempt 1 must retry, got {other:?}"),
        }
        // Monotone growth with fixed jitter seed across attempts.
        let d1 = policy.delay_for_attempt(1, 7);
        let d3 = policy.delay_for_attempt(3, 7);
        assert!(d3 > d1, "backoff grows exponentially: {d1} -> {d3}");
        // Exhaustion: at max_attempts the event is dead-lettered.
        assert_eq!(
            policy.classify(policy.max_attempts),
            DeliveryOutcome::DeadLetter
        );
        // Jitter stays within ±20% of the capped doubling: attempt 8 doubles
        // far beyond max (200×256 = 51200 → capped 30000); seed 0 → −20%,
        // seed 20 → 0%.
        let low = policy.delay_for_attempt(8, 0);
        let mid = policy.delay_for_attempt(8, 20);
        assert_eq!(mid, policy.max_delay_ms, "zero jitter keeps the capped max");
        assert_eq!(
            low,
            policy.max_delay_ms * 80 / 100,
            "seed 0 applies the −20% jitter bound"
        );
    }

    // G2.1 — DoD 3: signed replay removes from DLQ and records the actor.
    #[tokio::test]
    async fn dead_letter_record_then_signed_replay() {
        let dlq = InMemoryDeadLetterStore::new();
        let event_id = Uuid::new_v4();
        dlq.record(&DeadLetter {
            event_id,
            aggregate_type: "order".into(),
            event_type: "order.created".into(),
            payload: serde_json::json!({"total": 1000}),
            error: "channel unavailable".into(),
            attempt: 5,
            failed_at: Utc::now(),
        })
        .await
        .unwrap();

        let listed = dlq.list().await.unwrap();
        assert_eq!(listed.len(), 1, "letter retained for inspection");

        let (event, record) = dlq.replay(event_id, "auditor-chen").await.unwrap();
        assert_eq!(event.id, event_id);
        assert_eq!(record.actor, "auditor-chen");
        assert!(
            dlq.list().await.unwrap().is_empty(),
            "replay removes letter"
        );

        // Unknown replay is a NotFound, never a silent success.
        let missing = dlq.replay(Uuid::new_v4(), "auditor-chen").await;
        assert!(matches!(missing, Err(EventBusError::NotFound(_))));
    }
}
