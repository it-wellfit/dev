use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Account management - chart of accounts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String, // ASSETS, LIABILITIES, EQUITY, REVENUE, EXPENSE
    pub parent_account_id: Option<String>,
    pub currency: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Account {
    /// Create a new account
    pub fn new(
        account_id: String,
        account_code: String,
        account_name: String,
        account_type: String,
        parent_account_id: Option<String>,
        currency: String,
        description: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            account_id,
            account_code,
            account_name,
            account_type,
            parent_account_id,
            currency,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if account is a leaf node (has no children)
    pub async fn is_leaf_node(&self, pool: &sqlx::SqlitePool) -> Result<bool, sqlx::Error> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE parent_account_id = ?")
                .bind(&self.account_id)
                .fetch_one(pool)
                .await?;

        Ok(count.0 == 0)
    }

    /// Get account balance (in minor units, i64)
    pub async fn get_balance(&self, pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
        // Query all transaction items for this account
        let rows = sqlx::query_as::<_, (i64, String)>(
            "SELECT amount_minor, item_type FROM transaction_items WHERE account_id = ?",
        )
        .bind(&self.account_id)
        .fetch_all(pool)
        .await?;

        let mut balance: i64 = 0;
        for (amount, item_type) in rows {
            // Determine sign based on item type and account type
            let is_debit_account = matches!(self.account_type.as_str(), "ASSETS" | "EXPENSE");
            let delta: i64 = match item_type.as_str() {
                "DEBIT" => amount,
                "CREDIT" => -amount,
                _ => 0,
            };
            // For liability/equity/revenue accounts, debit decreases balance
            let signed = if is_debit_account { delta } else { -delta };
            balance += signed;
        }

        Ok(balance)
    }
}
