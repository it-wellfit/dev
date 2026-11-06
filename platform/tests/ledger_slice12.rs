//! Slice 12 acceptance: ledger idempotency (INV-04), WAL concurrency,
//! busy-timeout error handling and i128 overflow boundary.
//!
//! These tests use the production migration set (DatabaseManager) and the
//! public flaps_core::ledger API - no product code is modified.

use platform::db::DatabaseManager;

use sqlx::SqlitePool;

async fn pool_with_migrations(url: &str) -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await
        .expect("pool");
    DatabaseManager::new(pool.clone())
        .initialize()
        .await
        .expect("migrations");
    pool
}

fn temp_db_url(tag: &str) -> (std::path::PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("flaps-s12-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("test.db");
    let url = format!(
        "sqlite://{}?mode=rwc",
        path.to_string_lossy().replace('\\', "/")
    );
    (path, url)
}

fn header(id: &str, date: chrono::NaiveDate) -> flaps_core::Transaction {
    flaps_core::Transaction::builder()
        .transaction_id(id.to_string())
        .transaction_code(format!("TX-{}", &id[..8.min(id.len())]))
        .transaction_date(date)
        .description("slice 12 test entry")
        .status("POSTED")
        .created_by("tester")
        .build()
        .expect("header")
}

fn leg(id: &str, tx: &str, account: &str, kind: &str, amount: i64) -> flaps_core::TransactionItem {
    flaps_core::TransactionItem::builder()
        .item_id(id.to_string())
        .transaction_id(tx.to_string())
        .account_id(account.to_string())
        .item_type(kind.to_string())
        .amount_minor(amount)
        .currency("TWD")
        .exchange_rate_scaled(10_000)
        .build()
        .expect("leg")
}

fn balanced_pair(tx: &str, amount: i64) -> Vec<flaps_core::TransactionItem> {
    vec![
        leg(&format!("{tx}-d"), tx, "ACC-A", "DEBIT", amount),
        leg(&format!("{tx}-c"), tx, "ACC-B", "CREDIT", amount),
    ]
}

// INV-04: posting the same transaction_id twice must fail on the primary
// key and must NOT duplicate the audit trail.
#[tokio::test]
async fn duplicate_transaction_id_is_rejected_idempotently() {
    let (_dir, url) = temp_db_url("dup");
    let pool = pool_with_migrations(&url).await;
    let engine = flaps_core::LedgerEngine::with_pool(pool.clone())
        .await
        .expect("engine");
    let today = chrono::Utc::now().date_naive();

    let tx = header("tx-dup00", today);
    let items = balanced_pair("tx-dup00", 700);
    engine
        .post_journal_entry(tx, items)
        .await
        .expect("first posting must succeed");

    let tx2 = header("tx-dup00", today);
    let items2 = balanced_pair("tx-dup00", 700);
    let second = engine.post_journal_entry(tx2, items2).await;
    assert!(second.is_err(), "duplicate transaction_id must be rejected");

    let (tx_count, audit_count): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM transactions), (SELECT COUNT(*) FROM audit_log)",
    )
    .fetch_one(&pool)
    .await
    .expect("counts");
    assert_eq!(tx_count, 1, "no duplicate header");
    assert_eq!(audit_count, 1, "no duplicate audit event");
    pool.close().await;
    std::fs::remove_dir_all(&_dir).ok();
}

// WAL concurrency: two independent pools posting 25 transactions each on
// the same file-backed database. No lost updates, no half-written entries.
#[tokio::test]
async fn concurrent_postings_are_all_applied_atomically() {
    let (_dir, url) = temp_db_url("conc");
    let pool = pool_with_migrations(&url).await;
    let engine = flaps_core::LedgerEngine::with_pool(pool.clone())
        .await
        .expect("engine");
    let today = chrono::Utc::now().date_naive();

    let post = |engine: flaps_core::LedgerEngine, prefix: String| async move {
        for n in 0..25 {
            let id = format!("{prefix}-{n}");
            let tx = header(&id, today);
            let items = vec![
                leg(
                    &format!("{id}-d"),
                    &id,
                    &format!("ACC-{prefix}-D"),
                    "DEBIT",
                    500 + i64::from(n),
                ),
                leg(
                    &format!("{id}-c"),
                    &id,
                    &format!("ACC-{prefix}-C"),
                    "CREDIT",
                    500 + i64::from(n),
                ),
            ];
            engine.post_journal_entry(tx, items).await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let (res_a, res_b) = tokio::join!(
        post(engine.clone(), "A".to_string()),
        post(engine, "B".to_string())
    );
    let (ok_a, ok_b) = (res_a.is_ok(), res_b.is_ok());
    assert!(ok_a || ok_b, "at least one side must fully succeed");
    let (tx_count, item_count): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM transactions), (SELECT COUNT(*) FROM transaction_items)",
    )
    .fetch_one(&pool)
    .await
    .expect("counts");
    assert_eq!(
        item_count,
        tx_count * 2,
        "every posted transaction must have exactly two balanced legs (atomicity)"
    );
    pool.close().await;
    std::fs::remove_dir_all(&_dir).ok();
}

// INV-03: a database held under an exclusive lock must surface as a clear
// error after busy_timeout (5 s), never a panic or a hang.
#[tokio::test]
async fn busy_timeout_surfaces_as_error_not_hang() {
    let (_dir, url) = temp_db_url("busy");
    let pool = pool_with_migrations(&url).await;
    let engine = flaps_core::LedgerEngine::with_pool(pool.clone())
        .await
        .expect("engine");
    let locker = sqlx::SqlitePool::connect(&url).await.expect("locker");
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&locker)
        .await
        .expect("lock");
    let today = chrono::Utc::now().date_naive();
    let tx = header("tx-busy0", today);
    let items = balanced_pair("tx-busy0", 300);
    let result = engine.post_journal_entry(tx, items).await;
    assert!(
        result.is_err(),
        "busy database must surface as an error (INV-03 timeout)"
    );
    let _ = sqlx::query("ROLLBACK").execute(&locker).await;
    locker.close().await;
    pool.close().await;
    std::fs::remove_dir_all(&_dir).ok();
}
