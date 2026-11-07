/// Maximum absolute amount in minor units: DECIMAL(15,2) lineage
/// (15 integral digits + 2 fractional digits = 17 digits total).
pub const MAX_AMOUNT_MINOR: i64 = 99_999_999_999_999_999;

/// Main ledger engine - core accounting processing
#[derive(Clone)]
pub struct LedgerEngine {
    pool: sqlx::SqlitePool,
}

impl LedgerEngine {
    /// Initialize the engine with database connection (legacy path: also
    /// creates tables). Prefer `with_pool` after applying
    /// `platform/migrations/` via `DatabaseManager::initialize` (schema SSOT).
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;

        Self::apply_pragmas(&pool).await?;

        // Legacy path: create tables if they don't exist.
        Self::ensure_tables(&pool).await?;

        Ok(Self { pool })
    }

    /// Build the engine over an existing pool WITHOUT creating tables.
    ///
    /// Schema SSOT is `platform/migrations/` — apply those first via
    /// `DatabaseManager::initialize`. This is the preferred constructor for
    /// servers and tests.
    pub async fn with_pool(pool: sqlx::SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        Self::apply_pragmas(&pool).await?;
        Ok(Self { pool })
    }

    /// Performance optimizations (INV-03: WAL, busy_timeout, bounded cache;
    /// write transactions must stay ≤ 50 ms).
    async fn apply_pragmas(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(pool)
            .await?;
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(pool)
            .await?;
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(pool)
            .await?;
        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Ensure all required tables exist (legacy bootstrap path).
    ///
    /// Mirrors `platform/migrations/0001_baseline_schema.sql` (schema SSOT).
    /// Keep in sync until this path is retired (IMPROVEMENT_BACKLOG P1-2).
    async fn ensure_tables(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        // Accounts table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                account_id TEXT PRIMARY KEY,
                account_code TEXT NOT NULL,
                account_name TEXT NOT NULL,
                account_type TEXT NOT NULL,
                parent_account_id TEXT,
                currency TEXT DEFAULT 'TWD',
                description TEXT,
                is_active BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await?;

        // Transactions table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                transaction_id TEXT PRIMARY KEY,
                transaction_code TEXT NOT NULL,
                transaction_date DATE NOT NULL,
                description TEXT NOT NULL,
                reference_number TEXT,
                status TEXT NOT NULL,
                source_type TEXT,
                created_by TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await?;

        // Transaction items table (integer money: INV-01)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transaction_items (
                item_id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                amount_minor INTEGER NOT NULL,
                currency TEXT DEFAULT 'TWD',
                exchange_rate INTEGER DEFAULT 10000,
                reference TEXT,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id),
                FOREIGN KEY (account_id) REFERENCES accounts(account_id)
            )",
        )
        .execute(pool)
        .await?;

        // Audit log table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_log (
                log_id TEXT PRIMARY KEY,
                transaction_id TEXT,
                event_type TEXT NOT NULL,
                event_data TEXT,
                performed_by TEXT,
                performed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id)
            )",
        )
        .execute(pool)
        .await?;

        // Hot-path indexes (FR-007)
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions (transaction_date DESC)")
            .execute(pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions (status)")
            .execute(pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transaction_items_transaction ON transaction_items (transaction_id)")
            .execute(pool)
            .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_log_transaction ON audit_log (transaction_id)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Post a journal entry: validate, ensure accounts exist, then write the
    /// header, items, and audit event in ONE atomic transaction (INV-04).
    ///
    /// Idempotency: callers generate the `transaction_id` (UUID) — replaying
    /// an identical id fails on the primary key instead of double-posting.
    pub async fn post_journal_entry(
        &self,
        transaction: crate::ledger::transaction::Transaction,
        items: Vec<crate::ledger::transaction::TransactionItem>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = &self.pool;

        // 1) Domain validation (balance, bounds, dates)
        self.validate_journal_entry(&transaction, &items).await?;

        // 2) Ensure referenced accounts exist (auto-create on first use)
        for item in &items {
            self.ensure_account_exists(&item.account_id).await?;
        }

        // 3) Atomic write
        let mut tx = pool.begin().await?;

        // Insert transaction header
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, reference_number, status, source_type, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&transaction.transaction_id)
        .bind(&transaction.transaction_code)
        .bind(transaction.transaction_date)
        .bind(&transaction.description)
        .bind(&transaction.reference_number)
        .bind(&transaction.status)
        .bind(&transaction.source_type)
        .bind(&transaction.created_by)
        .execute(&mut *tx)
        .await?;

        // Insert transaction items
        for item in &items {
            sqlx::query(
                "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate, reference, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.item_id)
            .bind(&item.transaction_id)
            .bind(&item.account_id)
            .bind(&item.item_type)
            .bind(item.amount_minor)
            .bind(&item.currency)
            .bind(item.exchange_rate_scaled)
            .bind(&item.reference)
            .bind(&item.description)
            .execute(&mut *tx)
            .await?;
        }

        // Log the transaction
        let event_data = serde_json::to_string(&transaction).unwrap_or_default();
        sqlx::query(
            "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&transaction.transaction_id)
        .bind("TRANSACTION_POSTED")
        .bind(event_data)
        .bind(&transaction.created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Validate journal entry for accounting rules.
    ///
    /// Double-entry core (INV-02 / P1-5): debits must equal credits and the
    /// total must be positive. Enforced here so NO caller — form, API, or
    /// future batch importer — can persist a one-sided or zero entry.
    async fn validate_journal_entry(
        &self,
        transaction: &crate::ledger::transaction::Transaction,
        items: &[crate::ledger::transaction::TransactionItem],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check required fields
        if transaction.transaction_id.is_empty() {
            return Err("Transaction ID is required".into());
        }

        if transaction.transaction_date > chrono::Utc::now().date_naive() {
            return Err("Transaction date cannot be in the future".into());
        }

        if items.is_empty() {
            return Err("At least one transaction item is required".into());
        }

        let mut total_debit: i128 = 0;
        let mut total_credit: i128 = 0;

        // Validate each item
        for item in items {
            if item.account_id.is_empty() {
                return Err("Account ID is required for transaction items".into());
            }

            if item.amount_minor <= 0 {
                return Err("Transaction item amount must be positive".into());
            }

            if item.amount_minor > MAX_AMOUNT_MINOR {
                return Err(format!(
                    "Transaction item amount exceeds the schema cap DECIMAL(15,2) (max minor units {MAX_AMOUNT_MINOR})"
                )
                .into());
            }

            if item.exchange_rate_scaled <= 0 {
                return Err("Exchange rate must be positive (scaled by 10^4)".into());
            }

            if !["DEBIT", "CREDIT"].contains(&item.item_type.as_str()) {
                return Err("Invalid item type. Must be DEBIT or CREDIT".into());
            }

            // i128 accumulation: MAX_AMOUNT_MINOR is 17 digits, i64 could
            // overflow on many items; i128 makes the sum total-order safe.
            match item.item_type.as_str() {
                "DEBIT" => total_debit += i128::from(item.amount_minor),
                _ => total_credit += i128::from(item.amount_minor),
            }
        }

        if total_debit != total_credit {
            return Err(format!(
                "Journal entry is not balanced: debits ({total_debit}) must equal credits ({total_credit})"
            )
            .into());
        }

        if total_debit == 0 {
            return Err("Journal entry total must be greater than zero".into());
        }

        Ok(())
    }

    /// Ensure account exists, create if not
    async fn ensure_account_exists(&self, account_id: &str) -> Result<(), sqlx::Error> {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&self.pool)
            .await?;

        if exists.0 == 0 {
            // Create default account if it doesn't exist
            let default_account = crate::ledger::account::Account::new(
                account_id.to_string(),
                account_id.to_string(),
                format!("Account {}", account_id),
                "ASSETS".to_string(), // Default to asset account
                None,
                "TWD".to_string(),
                Some("Auto-created account".to_string()),
            );

            sqlx::query(
                "INSERT INTO accounts (account_id, account_code, account_name, account_type, parent_account_id, currency, description, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&default_account.account_id)
            .bind(&default_account.account_code)
            .bind(&default_account.account_name)
            .bind(&default_account.account_type)
            .bind(&default_account.parent_account_id)
            .bind(&default_account.currency)
            .bind(&default_account.description)
            .bind(default_account.is_active)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Query transactions with filters
    pub async fn query_transactions(
        &self,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        status: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::ledger::transaction::Transaction>, Box<dyn std::error::Error>> {
        use sqlx::QueryBuilder;
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            "SELECT transaction_id, transaction_code, transaction_date, description, reference_number, status, source_type, created_by, created_at, updated_at FROM transactions WHERE 1=1"
        );

        if let Some(start) = start_date {
            qb.push(" AND transaction_date >= ");
            qb.push_bind(start);
        }

        if let Some(end) = end_date {
            qb.push(" AND transaction_date <= ");
            qb.push_bind(end);
        }

        if let Some(s) = status {
            qb.push(" AND status = ");
            qb.push_bind(s);
        }

        qb.push(" ORDER BY transaction_date DESC, created_at DESC LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb
            .build_query_as::<crate::ledger::transaction::Transaction>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}

/// A transaction header joined with its posting amount (for list views).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransactionSummary {
    #[sqlx(flatten)]
    pub transaction: crate::ledger::transaction::Transaction,
    /// Debit-side total in minor units (INV-01 integer money). With balanced
    /// entries this equals the entry amount — NOT the debit+credit sum.
    pub total_amount_minor: i64,
    /// Representative currency of the items (MIN across items).
    pub currency: String,
}

impl LedgerEngine {
    /// Most recent transactions with their summed minor amounts, newest first.
    /// Read-only (GET path); parameterized limit.
    pub async fn query_recent_transaction_summaries(
        &self,
        limit: i64,
    ) -> Result<Vec<TransactionSummary>, Box<dyn std::error::Error>> {
        let rows = sqlx::query_as::<_, TransactionSummary>(
            "SELECT t.transaction_id, t.transaction_code, t.transaction_date, t.description, \
             t.reference_number, t.status, t.source_type, t.created_by, t.created_at, t.updated_at, \
             COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS total_amount_minor, \
             COALESCE(MIN(i.currency), 'TWD') AS currency \
             FROM transactions t LEFT JOIN transaction_items i ON i.transaction_id = t.transaction_id \
             GROUP BY t.transaction_id \
             ORDER BY t.transaction_date DESC, t.created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Paged variant of `query_recent_transaction_summaries`. `limit` and
    /// `offset` are bound parameters — the page number never reaches SQL as
    /// concatenated text. Callers validate/clamp the page number.
    pub async fn query_transaction_summaries_paged(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TransactionSummary>, Box<dyn std::error::Error>> {
        let rows = sqlx::query_as::<_, TransactionSummary>(
            "SELECT t.transaction_id, t.transaction_code, t.transaction_date, t.description, \
             t.reference_number, t.status, t.source_type, t.created_by, t.created_at, t.updated_at, \
             COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS total_amount_minor, \
             COALESCE(MIN(i.currency), 'TWD') AS currency \
             FROM transactions t LEFT JOIN transaction_items i ON i.transaction_id = t.transaction_id \
             GROUP BY t.transaction_id \
             ORDER BY t.transaction_date DESC, t.created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

/// One month's debit/credit totals (dashboard chart, Slice 8 Layer D).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MonthlyTotal {
    /// Year-month, e.g. `2026-11`.
    pub ym: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

/// Per-account totals for one month (monthly report, Slice 8 Layer D).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountMonthTotal {
    pub account_id: String,
    pub debit_minor: i64,
    pub credit_minor: i64,
}

impl LedgerEngine {
    /// Last `months` months of debit/credit totals, oldest first.
    pub async fn monthly_totals(
        &self,
        months: i64,
    ) -> Result<Vec<MonthlyTotal>, Box<dyn std::error::Error>> {
        let rows = sqlx::query_as::<_, MonthlyTotal>(
            "SELECT strftime('%Y-%m', t.transaction_date) AS ym, \
             COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS debit_minor, \
             COALESCE(SUM(CASE WHEN i.item_type = 'CREDIT' THEN i.amount_minor ELSE 0 END), 0) AS credit_minor \
             FROM transactions t JOIN transaction_items i ON i.transaction_id = t.transaction_id \
             GROUP BY ym ORDER BY ym DESC LIMIT ?",
        )
        .bind(months)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().rev().collect())
    }

    /// Per-account debit/credit totals for one `YYYY-MM` month.
    pub async fn monthly_by_account(
        &self,
        ym: &str,
    ) -> Result<Vec<AccountMonthTotal>, Box<dyn std::error::Error>> {
        // Convert "YYYY-MM" to date range for index-friendly query.
        let start = format!("{}-01", ym);
        let end = if ym.ends_with("-12") {
            format!("{}-01-01", &ym[..4])
        } else {
            // Next month
            let parts: Vec<&str> = ym.split('-').collect();
            let year: i32 = parts[0].parse().unwrap_or(2026);
            let month: u32 = parts[1].parse().unwrap_or(1);
            if month == 12 {
                format!("{}-01-01", year + 1)
            } else {
                format!("{:04}-{:02}-01", year, month + 1)
            }
        };
        let rows = sqlx::query_as::<_, AccountMonthTotal>(
            "SELECT i.account_id, \
             COALESCE(SUM(CASE WHEN i.item_type = 'DEBIT' THEN i.amount_minor ELSE 0 END), 0) AS debit_minor, \
             COALESCE(SUM(CASE WHEN i.item_type = 'CREDIT' THEN i.amount_minor ELSE 0 END), 0) AS credit_minor \
             FROM transaction_items i JOIN transactions t ON i.transaction_id = t.transaction_id \
             WHERE t.transaction_date >= ? AND t.transaction_date < ? \
             GROUP BY i.account_id ORDER BY i.account_id",
        )
        .bind(&start)
        .bind(&end)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::transaction::TransactionItem;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;

    async fn test_engine() -> LedgerEngine {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory pool");
        let engine = LedgerEngine::with_pool(pool.clone()).await.expect("engine");
        // Tests bootstrap via the legacy path (schema mirrors migrations 0001).
        LedgerEngine::ensure_tables(&pool)
            .await
            .expect("bootstrap tables");
        engine
    }

    fn header(id: &str, date: chrono::NaiveDate) -> crate::ledger::transaction::Transaction {
        crate::ledger::transaction::Transaction::builder()
            .transaction_id(id.to_string())
            .transaction_code(format!("TX-{}", &id[..8.min(id.len())]))
            .transaction_date(date)
            .description("engine test entry")
            .reference_number(None)
            .status("POSTED")
            .source_type(Some("test".to_string()))
            .created_by("tester")
            .build()
            .expect("valid test header")
    }

    fn item(id: &str, tx: &str, account: &str, kind: &str, amount: i64) -> TransactionItem {
        TransactionItem::builder()
            .item_id(id.to_string())
            .transaction_id(tx.to_string())
            .account_id(account.to_string())
            .item_type(kind.to_string())
            .amount_minor(amount)
            .currency("TWD")
            .exchange_rate_scaled(10_000)
            .build()
            .expect("valid test item")
    }

    fn balanced_pair(tx: &str, amount: i64) -> Vec<TransactionItem> {
        vec![
            item(&format!("{tx}-d"), tx, "ACC-DEBIT", "DEBIT", amount),
            item(&format!("{tx}-c"), tx, "ACC-CREDIT", "CREDIT", amount),
        ]
    }

    #[tokio::test]
    async fn future_date_is_rejected() {
        let engine = test_engine().await;
        let future = chrono::Utc::now().date_naive() + chrono::Duration::days(3);
        let tx = header("tx-future", future);
        let result = engine
            .post_journal_entry(tx, balanced_pair("tx-future", 500))
            .await;
        assert!(result.is_err(), "future dates must be rejected");
        assert!(format!("{}", result.unwrap_err()).contains("future"));
    }

    #[tokio::test]
    async fn negative_amount_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-neg", today);
        let items = vec![
            item("tx-neg-d", "tx-neg", "A", "DEBIT", -100),
            item("tx-neg-c", "tx-neg", "B", "CREDIT", -100),
        ];
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "negative amounts must be rejected");
    }

    #[tokio::test]
    async fn amount_over_cap_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-cap", today);
        let items = vec![
            item("tx-cap-d", "tx-cap", "A", "DEBIT", MAX_AMOUNT_MINOR + 1),
            item("tx-cap-c", "tx-cap", "B", "CREDIT", MAX_AMOUNT_MINOR + 1),
        ];
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "over-cap amounts must be rejected");
    }

    #[tokio::test]
    async fn nonpositive_exchange_rate_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-rate", today);
        let mut items = balanced_pair("tx-rate", 100);
        items[0].exchange_rate_scaled = 0;
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "non-positive rates must be rejected");
    }

    #[tokio::test]
    async fn invalid_item_type_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-kind", today);
        let items = vec![
            item("tx-kind-d", "tx-kind", "A", "DEBIT", 100),
            item("tx-kind-c", "tx-kind", "B", "POST", 100),
        ];
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "invalid item types must be rejected");
    }

    #[tokio::test]
    async fn empty_items_are_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-empty", today);
        let result = engine.post_journal_entry(tx, Vec::new()).await;
        assert!(result.is_err(), "empty entries must be rejected");
        assert!(format!("{}", result.unwrap_err()).contains("At least one"));
    }

    #[tokio::test]
    async fn unbalanced_entry_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-unbal", today);
        let items = vec![
            item("tx-unbal-d", "tx-unbal", "A", "DEBIT", 1_000),
            item("tx-unbal-c", "tx-unbal", "B", "CREDIT", 900),
        ];
        let result = engine.post_journal_entry(tx, items).await;
        let err = format!("{}", result.expect_err("unbalanced must fail"));
        assert!(err.contains("not balanced"), "must name the balance rule");
        assert!(
            err.contains("1000") && err.contains("900"),
            "must show both totals"
        );
    }

    #[tokio::test]
    async fn one_sided_entry_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-side", today);
        let items = vec![item("tx-side-d", "tx-side", "A", "DEBIT", 1_000)];
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "one-sided entries must be rejected");
    }

    #[tokio::test]
    async fn zero_amount_item_is_rejected() {
        let engine = test_engine().await;
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-zero", today);
        let items = vec![
            item("tx-zero-d", "tx-zero", "A", "DEBIT", 0),
            item("tx-zero-c", "tx-zero", "B", "CREDIT", 0),
        ];
        let result = engine.post_journal_entry(tx, items).await;
        assert!(result.is_err(), "zero-amount items must be rejected");
    }

    #[tokio::test]
    async fn balanced_entry_posts_and_roundtrips() {
        let engine = Arc::new(test_engine().await);
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-ok0000", today);
        let tx_id = tx.transaction_id.clone();
        engine
            .post_journal_entry(tx, balanced_pair("tx-ok0000", 1_234_567))
            .await
            .expect("balanced entry must post");

        // Header + summed items round-trip (FR-007 read path).
        let summaries = engine
            .query_recent_transaction_summaries(10)
            .await
            .expect("summaries");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].transaction.transaction_id, tx_id);
        assert_eq!(summaries[0].total_amount_minor, 1_234_567);

        // Audit event written (traceability).
        let audit_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE transaction_id = ? AND event_type = 'TRANSACTION_POSTED'",
        )
        .bind(&tx_id)
        .fetch_one(&engine.pool)
        .await
        .expect("audit query");
        assert_eq!(audit_count.0, 1, "posting must be audited");
    }

    #[test]
    fn status_transition_posted_to_pending_review() {
        // DDD 聚合狀態機：POSTED（建立時預設）→ PENDING_REVIEW（覆核中）。
        let today = chrono::Utc::now().date_naive();
        let mut tx = header("tx-status-review", today);
        assert_eq!(tx.status, "POSTED", "header 預設狀態為 POSTED");
        tx.update_status("PENDING_REVIEW".to_string());
        assert_eq!(tx.status, "PENDING_REVIEW", "狀態轉移至 PENDING_REVIEW");
    }

    #[tokio::test]
    async fn auto_creates_missing_accounts() {
        let engine = Arc::new(test_engine().await);
        let today = chrono::Utc::now().date_naive();
        let tx = header("tx-auto0", today);
        engine
            .post_journal_entry(
                tx,
                vec![
                    item("tx-auto0-d", "tx-auto0", "NEW-BANK", "DEBIT", 250),
                    item("tx-auto0-c", "tx-auto0", "NEW-CASH", "CREDIT", 250),
                ],
            )
            .await
            .expect("posting with new accounts must succeed");

        let accounts: Vec<(String,)> =
            sqlx::query_as("SELECT account_id FROM accounts WHERE account_id IN ('NEW-BANK','NEW-CASH') ORDER BY account_id")
                .fetch_all(&engine.pool)
                .await
                .expect("accounts");
        assert_eq!(accounts.len(), 2, "both accounts auto-created");
    }
}
