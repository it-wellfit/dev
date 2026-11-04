use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Transaction record - financial transaction header
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub transaction_id: String,
    pub transaction_code: String,
    pub transaction_date: chrono::NaiveDate,
    pub description: String,
    pub reference_number: Option<String>,
    pub status: String,
    pub source_type: Option<String>,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Transaction {
    /// Create a new transaction
    #[allow(clippy::too_many_arguments)] // constructor mirrors DB columns; builder refactor deferred
    pub fn new(
        transaction_id: String,
        transaction_code: String,
        transaction_date: chrono::NaiveDate,
        description: String,
        reference_number: Option<String>,
        status: String,
        source_type: Option<String>,
        created_by: String,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            transaction_id,
            transaction_code,
            transaction_date,
            description,
            reference_number,
            status,
            source_type,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update transaction status
    pub fn update_status(&mut self, new_status: String) {
        self.status = new_status;
        self.updated_at = chrono::Utc::now().naive_utc();
    }
}
/// Transaction item - individual line item in a transaction
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionItem {
    pub item_id: String,
    pub transaction_id: String,
    pub account_id: String,
    pub item_type: String, // DEBIT or CREDIT
    /// Amount in minor units (10^-2 of a currency unit, e.g. cents).
    /// INV-01: integers only — never floating point.
    #[sqlx(rename = "amount_minor")]
    pub amount_minor: i64,
    pub currency: String,
    /// Exchange rate scaled by 10^4 (1.0000 → 10_000). INV-01: integers only.
    #[sqlx(rename = "exchange_rate")]
    pub exchange_rate_scaled: i64,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

impl TransactionItem {
    /// Create a new transaction item
    #[allow(clippy::too_many_arguments)] // constructor mirrors DB columns; builder refactor deferred
    pub fn new(
        item_id: String,
        transaction_id: String,
        account_id: String,
        item_type: String,
        amount_minor: i64,
        currency: String,
        exchange_rate_scaled: i64,
        reference: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            item_id,
            transaction_id,
            account_id,
            item_type,
            amount_minor,
            currency,
            exchange_rate_scaled,
            reference,
            description,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
