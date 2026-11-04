use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Audit log entry for compliance and tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub log_id: String,
    pub transaction_id: Option<String>,
    pub event_type: String, // TRANSACTION_POSTED, TRANSACTION_UPDATED, ACCOUNT_CREATED, etc.
    pub event_data: Option<String>, // JSON serialized event data
    pub performed_by: String,
    pub performed_at: chrono::NaiveDateTime,
}

impl AuditLog {
    /// Create a new audit log entry
    pub fn new(
        log_id: String,
        transaction_id: Option<String>,
        event_type: String,
        event_data: Option<String>,
        performed_by: String,
    ) -> Self {
        Self {
            log_id,
            transaction_id,
            event_type,
            event_data,
            performed_by,
            performed_at: chrono::Utc::now().naive_utc(),
        }
    }

    /// Log transaction posting
    pub fn log_transaction_posted(
        transaction_id: String,
        transaction_data: String,
        performed_by: String,
    ) -> Self {
        Self::new(
            uuid::Uuid::new_v4().to_string(),
            Some(transaction_id),
            "TRANSACTION_POSTED".to_string(),
            Some(transaction_data),
            performed_by,
        )
    }
}
