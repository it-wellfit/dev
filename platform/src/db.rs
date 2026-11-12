use sqlx::SqlitePool;
use std::str::FromStr;
use std::time::Duration;

/// 連線選項 SSOT（2026-11-12 SOTA 精實化）：所有 SQLite 連線（bin、測試、
/// 未來工具）統一經此建構，PRAGMA 一次到位。
///
/// 為何需要：sqlx 預設**不開** `foreign_keys`（FK 約束靜默失效——重複帳戶
/// 類缺陷的溫床），且 `busy_timeout` 預設 0（並發寫入立即 SQLITE_BUSY）。
/// 併租 WAL + synchronous=NORMAL（WAL 下安全且寫入更快）+ 256MiB mmap。
/// 記憶體庫不套 WAL/synchronous（SQLite 對 :memory: 不支援 WAL）。
pub fn connect_options(db_url: &str) -> Result<sqlx::sqlite::SqliteConnectOptions, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
    let mut opts = SqliteConnectOptions::from_str(db_url)?
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    // 唯讀連線（備份/快照檢視、稽核工具）不得套用需寫入的 pragma：
    // journal_mode/synchronous 會要求寫入 DB 檔（實證：mode=ro 下
    // "attempt to write a readonly database"）。記憶體庫亦不支援 WAL。
    let needs_write = !db_url.contains(":memory:") && !db_url.contains("mode=ro");
    if needs_write {
        opts = opts
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .pragma("mmap_size", 268_435_456i64.to_string());
    }
    Ok(opts)
}

/// Database manager for AuxBridge platform
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

    // 遷移版本防呆（2026-11-12 規格補洞）：版本號唯一且從 1 連續。
    // 為何需要：並行工作線各自新增 `0008_*.sql` 導致 sqlx 以
    // "UNIQUE constraint failed: _sqlx_migrations.version" 炸掉全部測試
    // （實證 2026-11-12）——建置期即攔下，不必等執行期。
    #[test]
    fn migration_versions_are_unique_and_sequential() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let mut versions: Vec<u32> = Vec::new();
        for entry in std::fs::read_dir(&dir).expect("migrations dir readable") {
            let path = entry.expect("dir entry").path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.ends_with(".sql") {
                continue;
            }
            let prefix: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
            assert!(
                !prefix.is_empty(),
                "migration file must start with a numeric version: {name}"
            );
            let version: u32 = prefix
                .parse()
                .unwrap_or_else(|_| panic!("bad migration version in {name}"));
            assert!(
                !versions.contains(&version),
                "duplicate migration version {version} ({name}) — sqlx would fail with a UNIQUE constraint"
            );
            versions.push(version);
        }
        versions.sort_unstable();
        assert!(!versions.is_empty(), "no migrations found");
        for (index, version) in versions.iter().enumerate() {
            assert_eq!(
                *version,
                (index + 1) as u32,
                "migration versions must be sequential from 1 (gap or duplicate)"
            );
        }
    }

    // 備份一致性（2026-11-12 規格補洞）：VACUUM INTO 產生可用快照，
    // 且含來源資料（災難復原演練的機制驗證）。
    #[tokio::test]
    async fn vacuum_into_produces_consistent_snapshot() {
        let dir = std::env::temp_dir().join(format!("auxbridge-vac-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let src_path = dir.join("src.db");
        let snap_path = dir.join("snap.db");
        let src_url = format!("sqlite://{}?mode=rwc", src_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(super::connect_options(&src_url).expect("opts"))
            .await
            .expect("src pool");
        DatabaseManager::new(pool.clone())
            .initialize()
            .await
            .expect("migrations");
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, \
             description, status) VALUES ('t1', 'TX-1', '2026-11-01', 'snapshot me', 'POSTED')",
        )
        .execute(&pool)
        .await
        .expect("insert");
        let escaped = snap_path.display().to_string().replace('\'', "''");
        sqlx::query(sqlx::AssertSqlSafe(format!("VACUUM INTO '{escaped}'")))
            .execute(&pool)
            .await
            .expect("vacuum into");
        pool.close().await;

        let snap_url = format!("sqlite://{}?mode=ro", snap_path.display());
        let snap = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(super::connect_options(&snap_url).expect("opts"))
            .await
            .expect("snapshot pool");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&snap)
            .await
            .expect("count");
        assert_eq!(count, 1, "snapshot must carry source rows");
        snap.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    // PRAGMA 硬化（2026-11-12 SOTA 精實化）：foreign_keys 必須為 ON（sqlx
    // 預設關閉 → FK 約束靜默失效）、檔案庫必須 WAL；FK 違規必須真的被拒。
    #[tokio::test]
    async fn connect_options_enforces_foreign_keys_and_wal() {
        let dir = std::env::temp_dir().join(format!("auxbridge-fk-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let db_path = dir.join("fk.db");
        let url = format!("sqlite://{}?mode=rwc", db_path.display());
        let opts = super::connect_options(&url).expect("connect options");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .expect("file pool");

        let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .expect("pragma read");
        assert_eq!(fk, 1, "foreign_keys must be ON");
        let jm: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("journal mode");
        assert_eq!(jm.to_lowercase(), "wal", "file DBs must use WAL");

        DatabaseManager::new(pool.clone())
            .initialize()
            .await
            .expect("migrations");
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, \
             description, status) VALUES ('t1', 'TX-1', '2026-11-01', 'x', 'POSTED')",
        )
        .execute(&pool)
        .await
        .expect("parent row");
        let bad = sqlx::query(
            "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, \
             amount_minor) VALUES ('i1', 't1', 'NO-SUCH-ACCOUNT', 'DEBIT', 100)",
        )
        .execute(&pool)
        .await;
        assert!(
            bad.is_err(),
            "FK must reject an item referencing a nonexistent account"
        );
        let _ = std::fs::remove_dir_all(&dir);
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
            "users",
            "sessions",
            "import_batches",
            "import_rows",
            "drafts",
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
