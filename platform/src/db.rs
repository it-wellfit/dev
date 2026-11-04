use sqlx::SqlitePool;

/// Database manager for Flaps ERP platform
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Apply versioned migrations from `platform/migrations/` (schema SSOT).
    ///
    /// Replaces the previous ad-hoc `CREATE TABLE IF NOT EXISTS` statements.
    /// Migrations are tracked in `_sqlx_migrations`; the baseline migration is
    /// idempotent (`IF NOT EXISTS`) so databases created by the legacy dynamic
    /// path can replay it safely.
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn in_memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite pool")
    }

    #[tokio::test]
    async fn migrations_create_all_baseline_tables() {
        let pool = in_memory_pool().await;
        let mgr = DatabaseManager::new(pool.clone());
        mgr.initialize().await.expect("migrations run");

        let expected = [
            "accounts",
            "transactions",
            "transaction_items",
            "audit_log",
            "erp_config",
            "config_history",
            "roles",
            "permissions",
            "role_permissions",
            "user_roles",
        ];
        for table in expected {
            let count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("query sqlite_master");
            assert_eq!(count.0, 1, "table {table} should exist after migration");
        }
    }

    #[tokio::test]
    async fn migrations_are_idempotent_on_replay() {
        let pool = in_memory_pool().await;
        let mgr = DatabaseManager::new(pool.clone());
        mgr.initialize().await.expect("first run");
        // 第二次執行不應失敗（sqlx 以 _sqlx_migrations 記錄；此處模擬既有庫重放）
        mgr.initialize().await.expect("second run is a no-op");

        let applied: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("query _sqlx_migrations");
        assert!(applied.0 >= 1, "migrations should be recorded");
    }

    #[tokio::test]
    async fn migration_is_safe_on_legacy_dynamic_schema() {
        // 模擬舊路徑（engine.rs::ensure_tables / 舊 initialize）先建好資料庫
        let pool = in_memory_pool().await;
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
        .execute(&pool)
        .await
        .expect("legacy accounts table");

        let mgr = DatabaseManager::new(pool);
        mgr.initialize()
            .await
            .expect("migration must not fail when tables pre-exist (IF NOT EXISTS)");
    }

    #[tokio::test]
    async fn integer_money_schema_roundtrip() {
        // INV-01（P0-1）：schema 全程整數——寫入 i64 minor 分、讀回無損，
        // 任何浮點形狀的值都無法進入金額欄位。
        let pool = in_memory_pool().await;
        let mgr = DatabaseManager::new(pool.clone());
        mgr.initialize().await.expect("migrations run");

        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by)
             VALUES ('tx-rt', 'TX-RT', '2026-01-01', 'roundtrip', 'POSTED', 'tester')",
        )
        .execute(&pool)
        .await
        .expect("seed header");

        sqlx::query(
            "INSERT INTO accounts (account_id, account_code, account_name, account_type, currency, is_active)
             VALUES ('acc-rt', 'ACC-RT', 'Roundtrip Account', 'ASSETS', 'TWD', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed account");

        sqlx::query(
            "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency, exchange_rate)
             VALUES ('it-rt', 'tx-rt', 'acc-rt', 'DEBIT', 1234567, 'TWD', 10000)",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        let row: (i64, i64) = sqlx::query_as(
            "SELECT amount_minor, exchange_rate FROM transaction_items WHERE item_id = 'it-rt'",
        )
        .fetch_one(&pool)
        .await
        .expect("read back");

        assert_eq!(
            row,
            (1_234_567, 10_000),
            "integer money must round-trip exactly"
        );

        // 整數以外的形狀（文字）被 SQLite INTEGER affinity 轉換為整數儲存，
        // 確認欄位型別約束存在（schema 不得回歸為 DECIMAL）。
        let decl: (String,) = sqlx::query_as(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'transaction_items'",
        )
        .fetch_one(&pool)
        .await
        .expect("ddl");
        assert!(
            decl.0.contains("amount_minor INTEGER NOT NULL"),
            "amount_minor must be INTEGER NOT NULL"
        );
        assert!(
            decl.0.contains("exchange_rate INTEGER"),
            "exchange_rate must be INTEGER"
        );
        assert!(!decl.0.contains("DECIMAL"), "DECIMAL must be gone (INV-01)");
    }
}
