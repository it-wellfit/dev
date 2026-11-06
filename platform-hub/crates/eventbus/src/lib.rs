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
}