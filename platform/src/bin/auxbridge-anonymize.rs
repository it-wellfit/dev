//! auxbridge-anonymize — staging 匿名化工具（計畫 1793782615513 §3.5）。
//!
//! 用途：將 staging.db（prod 快照副本）中的敏感內容遮罩，供整合測試與
//! 視覺回歸基線使用。
//!
//! 遮罩範圍（2026-11-17 決策記錄）：
//! - transactions.description / reference_number → 「匿名交易 <n>」/ NULL
//! - transaction_items.reference / description → NULL
//! - audit_log.event_data → NULL；performed_by → 'masked'
//! - erp_config.config_value（is_sensitive=1）→ '***masked***'
//! - sessions → 全刪（防跨環境 session 重放）
//!
//! 刻意保留（決策記錄）：
//! - users.username / password_hash：staging 需可登入（內部工具，非 PII 載體；
//!   現況資料為合成 demo 資料）。
//! - amounts（amount_minor）：遮罩將破壞借貸平衡顯示；現況為合成 demo 金額。
//!
//! 防呆：路徑含 "prod" → 拒絕執行（絕不匿名化正式庫）。
//!
//! 用法：`auxbridge-anonymize <staging.db>`

use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("用法：auxbridge-anonymize <staging.db>");
        std::process::exit(2);
    }
    let db_path = args[1].clone();
    // 防呆：絕不匿名化正式庫（fail-closed）。
    if db_path.to_lowercase().contains("prod") {
        eprintln!("拒絕執行：目標路徑含 \"prod\"（正式庫不可匿名化）: {db_path}");
        std::process::exit(2);
    }
    if !std::path::Path::new(&db_path).exists() {
        eprintln!("目標資料庫不存在: {db_path}");
        std::process::exit(2);
    }

    let url = format!("sqlite://{db_path}?mode=rw");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;

    // 1. 交易描述與參考號。
    let n = sqlx::query(
        "UPDATE transactions SET description = '匿名交易 #' || rowid, \
         reference_number = NULL",
    )
    .execute(&pool)
    .await?
    .rows_affected();
    println!("transactions masked: {n}");

    // 2. 分錄備註。
    let n = sqlx::query("UPDATE transaction_items SET description = NULL, reference = NULL")
        .execute(&pool)
        .await?
        .rows_affected();
    println!("transaction_items masked: {n}");

    // 3. 稽核軌跡（事件內容與操作者）。
    let n = sqlx::query("UPDATE audit_log SET event_data = NULL, performed_by = 'masked'")
        .execute(&pool)
        .await?
        .rows_affected();
    println!("audit_log masked: {n}");

    // 4. 敏感配置值。
    let n =
        sqlx::query("UPDATE erp_config SET config_value = '***masked***' WHERE is_sensitive = 1")
            .execute(&pool)
            .await?
            .rows_affected();
    println!("erp_config sensitive masked: {n}");

    // 5. 清除跨環境 session（防重放）。
    let n = sqlx::query("DELETE FROM sessions")
        .execute(&pool)
        .await?
        .rows_affected();
    println!("sessions deleted: {n}");

    pool.close().await;
    println!("anonymization complete: {db_path}");
    Ok(())
}
