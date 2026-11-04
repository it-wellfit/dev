//! Flaps ERP server binary (ADR-0002).
//!
//! Zero-JS web tier over Axum 0.7:
//! - GET  /                        server-rendered UI (Askama, auto-escaped)
//! - GET  /assets/design-system.css  compile-time embedded stylesheet
//! - GET  /api/health              JSON health probe
//! - POST /transactions            form → validate → RBAC → ledger → 303 PRG
//!
//! Configuration (environment):
//! - `FLAPS_DB_URL`  SQLite URL, default `sqlite://data/flaps.db?mode=rwc`
//! - `FLAPS_PORT`    TCP port, default 8090 (binds 127.0.0.1 only)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = std::env::var("FLAPS_DB_URL")
        .unwrap_or_else(|_| "sqlite://data/flaps.db?mode=rwc".to_string());

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
        .connect(&db_url)
        .await?;

    // Schema SSOT: apply versioned migrations before any domain access.
    platform::db::DatabaseManager::new(pool.clone())
        .initialize()
        .await?;
    // G-1: bootstrap the administrator account (first run only).
    platform::auth::seed_admin_if_empty(&pool).await?;
    let state = platform::api::AppState::new(pool).await?;

    let app = platform::api::router(state);
    let port: u16 = std::env::var("FLAPS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Flaps ERP server listening on http://{addr} (zero-JS UI)");
    axum::serve(listener, app).await?;

    Ok(())
}
