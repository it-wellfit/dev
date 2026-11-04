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
    /// Builder entry point (S10-FR-4): replaces the 8-argument constructor so
    /// call sites read by field name and clippy's `too_many_arguments`
    /// exemption is no longer needed.
    pub fn builder() -> TransactionBuilder {
        TransactionBuilder::default()
    }

    /// Update transaction status
    pub fn update_status(&mut self, new_status: String) {
        self.status = new_status;
        self.updated_at = chrono::Utc::now().naive_utc();
    }
}

/// Builder for [`Transaction`] (DDD tactical pattern: construction separated
/// from the aggregate; required fields enforced at `build`).
#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder {
    transaction_id: Option<String>,
    transaction_code: Option<String>,
    transaction_date: Option<chrono::NaiveDate>,
    description: Option<String>,
    reference_number: Option<String>,
    status: Option<String>,
    source_type: Option<String>,
    created_by: Option<String>,
}

impl TransactionBuilder {
    pub fn transaction_id(mut self, v: impl Into<String>) -> Self {
        self.transaction_id = Some(v.into());
        self
    }

    pub fn transaction_code(mut self, v: impl Into<String>) -> Self {
        self.transaction_code = Some(v.into());
        self
    }

    pub fn transaction_date(mut self, v: chrono::NaiveDate) -> Self {
        self.transaction_date = Some(v);
        self
    }

    pub fn description(mut self, v: impl Into<String>) -> Self {
        self.description = Some(v.into());
        self
    }

    pub fn reference_number(mut self, v: Option<String>) -> Self {
        self.reference_number = v;
        self
    }

    pub fn status(mut self, v: impl Into<String>) -> Self {
        self.status = Some(v.into());
        self
    }

    pub fn source_type(mut self, v: Option<String>) -> Self {
        self.source_type = v;
        self
    }

    pub fn created_by(mut self, v: impl Into<String>) -> Self {
        self.created_by = Some(v.into());
        self
    }

    /// Assemble the aggregate; fails closed when a required field is absent.
    pub fn build(self) -> Result<Transaction, String> {
        Ok(Transaction {
            transaction_id: self
                .transaction_id
                .ok_or_else(|| "transaction_id is required".to_string())?,
            transaction_code: self
                .transaction_code
                .ok_or_else(|| "transaction_code is required".to_string())?,
            transaction_date: self
                .transaction_date
                .ok_or_else(|| "transaction_date is required".to_string())?,
            description: self
                .description
                .ok_or_else(|| "description is required".to_string())?,
            reference_number: self.reference_number,
            status: self.status.unwrap_or_else(|| "POSTED".to_string()),
            source_type: self.source_type,
            created_by: self
                .created_by
                .ok_or_else(|| "created_by is required".to_string())?,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        })
    }
}

impl TransactionItem {
    /// Builder entry point (S10-FR-4).
    pub fn builder() -> TransactionItemBuilder {
        TransactionItemBuilder::default()
    }
}

/// Builder for [`TransactionItem`].
#[derive(Debug, Clone, Default)]
pub struct TransactionItemBuilder {
    item_id: Option<String>,
    transaction_id: Option<String>,
    account_id: Option<String>,
    item_type: Option<String>,
    amount_minor: Option<i64>,
    currency: Option<String>,
    exchange_rate_scaled: Option<i64>,
    reference: Option<String>,
    description: Option<String>,
}

impl TransactionItemBuilder {
    pub fn item_id(mut self, v: impl Into<String>) -> Self {
        self.item_id = Some(v.into());
        self
    }

    pub fn transaction_id(mut self, v: impl Into<String>) -> Self {
        self.transaction_id = Some(v.into());
        self
    }

    pub fn account_id(mut self, v: impl Into<String>) -> Self {
        self.account_id = Some(v.into());
        self
    }

    pub fn item_type(mut self, v: impl Into<String>) -> Self {
        self.item_type = Some(v.into());
        self
    }

    pub fn amount_minor(mut self, v: i64) -> Self {
        self.amount_minor = Some(v);
        self
    }

    pub fn currency(mut self, v: impl Into<String>) -> Self {
        self.currency = Some(v.into());
        self
    }

    pub fn exchange_rate_scaled(mut self, v: i64) -> Self {
        self.exchange_rate_scaled = Some(v);
        self
    }

    pub fn reference(mut self, v: Option<String>) -> Self {
        self.reference = v;
        self
    }

    pub fn description(mut self, v: Option<String>) -> Self {
        self.description = v;
        self
    }

    /// Assemble the line item; fails closed on required fields.
    pub fn build(self) -> Result<TransactionItem, String> {
        Ok(TransactionItem {
            item_id: self
                .item_id
                .ok_or_else(|| "item_id is required".to_string())?,
            transaction_id: self
                .transaction_id
                .ok_or_else(|| "transaction_id is required".to_string())?,
            account_id: self
                .account_id
                .ok_or_else(|| "account_id is required".to_string())?,
            item_type: self
                .item_type
                .ok_or_else(|| "item_type is required".to_string())?,
            amount_minor: self
                .amount_minor
                .ok_or_else(|| "amount_minor is required".to_string())?,
            currency: self.currency.unwrap_or_else(|| "TWD".to_string()),
            exchange_rate_scaled: self.exchange_rate_scaled.unwrap_or(10_000),
            reference: self.reference,
            description: self.description,
            created_at: chrono::Utc::now().naive_utc(),
        })
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
