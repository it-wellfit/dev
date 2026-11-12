//! AuxBridge server binary (ADR-0002).
//!
//! Zero-JS web tier over Axum 0.7:
//! - GET  /                        server-rendered UI (Askama, auto-escaped)
//! - GET  /assets/design-system.css  compile-time embedded stylesheet
//! - GET  /api/health              JSON health probe
//! - POST /transactions            form → validate → RBAC → ledger → 303 PRG
//!
//! Configuration (environment):
//! - `AUXBRIDGE_DB_URL`  SQLite URL, default `sqlite://data/auxbridge.db?mode=rwc`
//! - `AUXBRIDGE_PORT`    TCP port, default 8090 (binds 127.0.0.1 only)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 可觀測性：日誌初始化（原 tracing 無 subscriber → 日誌蒸發）。
    platform::obs::init();

    // 備份子命令（2026-11-12 規格補洞）：`--backup <db> <out.db>` 以 SQLite
    // `VACUUM INTO` 產生**一致性快照**（Online Backup 語義：不需停機、含
    // WAL 內容），供離線備份與災難復原演練（restore drill）。
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--backup") {
        if args.len() != 4 {
            eprintln!("用法：auxbridge-server --backup <來源.db> <目標快照.db>");
            std::process::exit(2);
        }
        let src = &args[2];
        let dst = &args[3];
        // VACUUM INTO 要求目標檔不存在（否則 SQLite 報錯）；先清除殘留。
        if std::path::Path::new(dst).exists() {
            std::fs::remove_file(dst)?;
        }
        let src_url = format!("sqlite://{src}?mode=ro");
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(platform::db::connect_options(&src_url)?)
            .await?;
        let dst_escaped = dst.replace('\'', "''");
        sqlx::query(sqlx::AssertSqlSafe(format!("VACUUM INTO '{dst_escaped}'")))
            .execute(&pool)
            .await?;
        pool.close().await;
        platform::obs_info!("backup: snapshot written: {dst}");
        println!("備份完成：{dst}");
        return Ok(());
    }

    let db_url = std::env::var("AUXBRIDGE_DB_URL")
        .unwrap_or_else(|_| "sqlite://data/auxbridge.db?mode=rwc".to_string());

    // Ensure the SQLite parent directory exists before sqlx opens the file.
    if let Some(path) = db_url
        .strip_prefix("sqlite://")
        .and_then(|rest| rest.split('?').next())
    {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(platform::db::connect_options(&db_url)?)
        .await?;

    // Schema SSOT: apply versioned migrations before any domain access.
    platform::db::DatabaseManager::new(pool.clone())
        .initialize()
        .await?;
    // G-1: bootstrap the administrator account (first run only).
    platform::auth::seed_admin_if_empty(&pool).await?;
    // Housekeeping: purge expired sessions on startup so the table does not
    // grow without bound across restarts.
    let _ = platform::auth::purge_expired_sessions(&pool).await;
    let state = platform::api::AppState::new(pool).await?;

    let app = platform::api::router(state);
    let port: u16 = std::env::var("AUXBRIDGE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);
    // 綁定位址：預設 127.0.0.1（僅本機，安全）；跨裝置/跨平台存取時設
    // AUXBRIDGE_BIND=0.0.0.0（區網開放；生產環境建議反向代理 + HTTPS）。
    let bind = std::env::var("AUXBRIDGE_BIND")
        .ok()
        .filter(|b| !b.trim().is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let addr = format!("{bind}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("AuxBridge server listening on http://{addr} (zero-JS UI)");
    // ConnectInfo：讓稽核取得真實對端 IP（2026-11-12 規格補洞：原僅能讀
    // X-Forwarded-For，直連情境下 ip_address 恆為 NULL，無法溯源）。
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
