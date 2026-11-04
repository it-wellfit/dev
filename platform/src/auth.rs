//! Authentication and session management (G-1, ADR-0003).
//!
//! Design, kept minimal and honest:
//! - Passwords: **Argon2id** hashing via the `argon2` crate (PHC string
//!   format); plaintext never touches storage or logs.
//! - Sessions: server-side rows in `sessions` (revocable, expiry-checked).
//!   The cookie carries only an opaque random id (uuid v4, 122 bits) and is
//!   `HttpOnly; SameSite=Lax; Path=/`. `Secure` is added once TLS terminates
//!   in front of this server (documented in THREAT_MODEL R-07).
//! - CSRF (T-08): a per-session token rendered into every write form and
//!   verified on every mutating POST (synchronizer token pattern).
//! - Authorization remains RBAC (`user_roles` → `LEDGER_WRITE`); this module
//!   only establishes *who* is acting (T-01), not what they may do.

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use askama::Template;
use axum::extract::{Form, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::api::AppState;

/// Session lifetime: 24 hours (fixed; re-login renews).
pub const SESSION_TTL_HOURS: i64 = 24;
pub const SESSION_COOKIE: &str = "flaps_session";

/// Errors surfaced to the login form (deliberately generic — never reveal
/// whether the username or the password was wrong).
pub const LOGIN_FAILURE_MSG: &str =
    "登入失敗（可能原因：帳號或密碼不正確、帳號已停用）；請重新輸入，或聯絡系統管理員。";

// ---------------------------------------------------------------------------
// Login rate limiting (P1-7): consecutive-failure lockout per username
// ---------------------------------------------------------------------------

/// Default policy: 5 consecutive failures lock the username for 15 minutes.
pub const MAX_LOGIN_ATTEMPTS: u32 = 5;
pub const LOGIN_LOCKOUT_SECS: u64 = 900;

/// In-process failure tracker keyed by username (trimmed, lowercased).
///
/// Local-first single-instance deployment: the state lives in the server
/// process. A multi-instance deployment would move this to the database —
/// recorded in IMPROVEMENT_BACKLOG (P1-7 notes).
#[derive(Default)]
pub struct LoginRateLimiter {
    attempts: Mutex<HashMap<String, (u32, Option<Instant>)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateDecision {
    Allowed,
    Locked { remaining_secs: u64 },
}

impl LoginRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether `username` is currently locked out.
    pub fn check(&self, username: &str) -> RateDecision {
        let key = username.trim().to_lowercase();
        let mut map = self.attempts.lock().expect("rate limiter lock");
        let lock_state = map.get(&key).and_then(|(_, locked_until)| *locked_until);
        match lock_state {
            Some(until) if until > Instant::now() => RateDecision::Locked {
                remaining_secs: (until - Instant::now()).as_secs() + 1,
            },
            Some(_) => {
                // Lockout elapsed: give the account a fresh counter.
                map.remove(&key);
                RateDecision::Allowed
            }
            None => RateDecision::Allowed,
        }
    }

    /// Record a failed attempt; locks the username when the threshold is hit.
    pub fn record_failure(&self, username: &str, max_attempts: u32, lockout: Duration) {
        let key = username.trim().to_lowercase();
        let mut map = self.attempts.lock().expect("rate limiter lock");
        let entry = map.entry(key).or_insert((0, None));
        entry.0 += 1;
        if entry.0 >= max_attempts {
            *entry = (entry.0, Some(Instant::now() + lockout));
        }
    }

    /// Clear the failure counter after a successful login.
    pub fn record_success(&self, username: &str) {
        self.attempts
            .lock()
            .expect("rate limiter lock")
            .remove(&username.trim().to_lowercase());
    }
}

// ---------------------------------------------------------------------------
// Password hashing (Argon2id)
// ---------------------------------------------------------------------------

/// Hash a password with Argon2id (crate default OWASP parameters).
pub fn hash_password(plaintext: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|h| h.to_string())
}

/// Verify a password against a stored PHC hash.
pub fn verify_password(plaintext: &str, stored: &str) -> bool {
    PasswordHash::new(stored)
        .map(|parsed| {
            Argon2::default()
                .verify_password(plaintext.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Session store (server-side, revocable)
// ---------------------------------------------------------------------------

/// One authenticated session as loaded from the store.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub csrf_token: String,
    pub expires_at: String,
}

/// Create a new session row for `user_id`; returns (session_id, csrf_token).
pub async fn create_session(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<(String, String), sqlx::Error> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let csrf_token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (session_id, user_id, csrf_token, expires_at) \
         VALUES (?, ?, ?, datetime('now', '+' || ? || ' hours'))",
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(&csrf_token)
    .bind(SESSION_TTL_HOURS)
    .execute(pool)
    .await?;
    Ok((session_id, csrf_token))
}

/// Load a session if it exists and has not expired.
pub async fn get_valid_session(pool: &SqlitePool, session_id: &str) -> Option<Session> {
    sqlx::query_as::<_, Session>(
        "SELECT session_id, user_id, csrf_token, expires_at FROM sessions \
         WHERE session_id = ? AND expires_at > datetime('now')",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Delete a session (logout; server-side revocation).
pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE session_id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Extract the session cookie value from request headers.
fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for pair in raw.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(SESSION_COOKIE) {
            let value = value.strip_prefix('=')?;
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Resolve the acting session from request headers.
pub async fn current_session(state: &AppState, headers: &HeaderMap) -> Option<Session> {
    let session_id = session_cookie(headers)?;
    get_valid_session(&state.pool, &session_id).await
}

/// Build the Set-Cookie header value for a session id.
pub fn session_cookie_value(session_id: &str) -> String {
    format!(
        "{SESSION_COOKIE}={session_id}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        SESSION_TTL_HOURS * 3600
    )
}

/// Build the cookie-clearing header value.
pub fn clear_cookie_value() -> String {
    format!("{SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}

// ---------------------------------------------------------------------------
// Administrator bootstrap
// ---------------------------------------------------------------------------

/// Seed the initial administrator account exactly once (when no users exist).
///
/// Password source: `FLAPS_ADMIN_PASSWORD`; when absent, a random password is
/// generated and printed ONCE to stdout for the operator to store. Default
/// credentials are never silently installed.
pub async fn seed_admin_if_empty(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }

    let password = match std::env::var("FLAPS_ADMIN_PASSWORD") {
        Ok(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => {
            let random = uuid::Uuid::new_v4().to_string().replace('-', "");
            let random = random[..16].to_string();
            println!("=== 首次啟動：已建立管理員帳號 admin，密碼（僅顯示這一次）：{random} ===");
            random
        }
    };

    let hash = hash_password(&password)?;
    sqlx::query(
        "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
         VALUES ('user-admin', 'admin', ?, 1)",
    )
    .bind(&hash)
    .execute(pool)
    .await?;
    println!("管理員帳號就緒：admin（user_id=user-admin，RBAC 種子角色 role-admin）");
    Ok(())
}

/// Validate credentials; returns the user_id when valid and active.
pub async fn authenticate(pool: &SqlitePool, username: &str, password: &str) -> Option<String> {
    let row: Option<(String, String, i64)> =
        sqlx::query_as("SELECT user_id, password_hash, is_active FROM users WHERE username = ?")
            .bind(username.trim())
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    let (user_id, stored_hash, is_active) = row?;
    if is_active == 0 {
        return None;
    }
    if !verify_password(password, &stored_hash) {
        return None;
    }
    Some(user_id)
}

// ---------------------------------------------------------------------------
// Routes: login / logout / session middleware
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub username: String,
}

fn render_login(error: Option<String>, username: &str) -> Response {
    let template = LoginTemplate {
        error,
        username: username.to_string(),
    };
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("登入頁渲染失敗：{e}"),
        )
            .into_response(),
    }
}

fn login_page(error: Option<String>, username: &str) -> Response {
    render_login(error, username)
}

/// GET /login — already-authenticated visitors go back to the workspace.
pub async fn get_login(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if current_session(&state, &headers).await.is_some() {
        return Redirect::to("/").into_response();
    }
    login_page(None, "")
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// POST /login — rate-limit check, authenticate, create a session (303 PRG).
pub async fn post_login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    // P1-7: consecutive-failure lockout per username (checked BEFORE any
    // credential work, so a locked account costs nothing to probe).
    if let RateDecision::Locked { remaining_secs } = state.rate_limiter.check(&form.username) {
        return login_page(
            Some(format!(
                "登入嘗試次數過多（問題：該帳號因連續失敗已暫時鎖定；處理方式：請於約 {remaining_secs} 秒後再試，或聯絡系統管理員）。"
            )),
            &form.username,
        );
    }

    match authenticate(&state.pool, &form.username, &form.password).await {
        Some(user_id) => {
            state.rate_limiter.record_success(&form.username);
            match create_session(&state.pool, &user_id).await {
                Ok((session_id, _)) => {
                    let mut response = Redirect::to("/").into_response();
                    if let Ok(value) =
                        header::HeaderValue::from_str(&session_cookie_value(&session_id))
                    {
                        response.headers_mut().append(header::SET_COOKIE, value);
                    }
                    response
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("無法建立工作階段（可能原因：資料庫忙碌）；請重試。詳情：{e}"),
                )
                    .into_response(),
            }
        }
        // Generic failure: same page, same message for wrong username OR
        // wrong password (no account enumeration). The attempt is recorded
        // BEFORE rendering so the lockout is reported on the next attempt.
        None => {
            state.rate_limiter.record_failure(
                &form.username,
                MAX_LOGIN_ATTEMPTS,
                Duration::from_secs(LOGIN_LOCKOUT_SECS),
            );
            login_page(Some(LOGIN_FAILURE_MSG.to_string()), &form.username)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LogoutForm {
    pub csrf_token: String,
}

/// POST /logout — CSRF-checked session revocation + cookie clearing.
pub async fn post_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LogoutForm>,
) -> Response {
    if let Some(session) = current_session(&state, &headers).await {
        if session.csrf_token == form.csrf_token {
            let _ = delete_session(&state.pool, &session.session_id).await;
        }
    }
    let mut response = Redirect::to("/login").into_response();
    if let Ok(value) = header::HeaderValue::from_str(&clear_cookie_value()) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

/// Middleware: every protected route requires a valid session; the resolved
/// session is injected into request extensions for handlers.
pub async fn require_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    match current_session(&state, request.headers()).await {
        Some(session) => {
            request.extensions_mut().insert(session);
            next.run(request).await
        }
        None => Redirect::to("/login").into_response(),
    }
}

/// Convenience constructor shared with the API router.
pub fn login_router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/login", get(get_login).post(post_login))
        .route("/logout", post(post_logout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rate_limiter_locks_after_threshold() {
        let limiter = LoginRateLimiter::new();
        assert_eq!(limiter.check("admin"), RateDecision::Allowed);
        limiter.record_failure("admin", 3, Duration::from_secs(60));
        limiter.record_failure("admin", 3, Duration::from_secs(60));
        assert_eq!(
            limiter.check("admin"),
            RateDecision::Allowed,
            "below threshold"
        );
        limiter.record_failure("admin", 3, Duration::from_secs(60));
        match limiter.check("admin") {
            RateDecision::Locked { remaining_secs } => {
                assert!(remaining_secs > 0 && remaining_secs <= 60);
            }
            RateDecision::Allowed => panic!("account must be locked at threshold"),
        }
    }

    #[test]
    fn rate_limiter_is_per_username_and_case_insensitive() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            limiter.record_failure("Admin", MAX_LOGIN_ATTEMPTS, Duration::from_secs(60));
        }
        assert!(
            matches!(limiter.check("ADMIN"), RateDecision::Locked { .. }),
            "lockout must be case-insensitive per username"
        );
        assert_eq!(
            limiter.check("other"),
            RateDecision::Allowed,
            "other usernames must be unaffected"
        );
    }

    #[test]
    fn rate_limiter_unlocks_after_lockout_window() {
        let limiter = LoginRateLimiter::new();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            limiter.record_failure("admin", MAX_LOGIN_ATTEMPTS, Duration::from_millis(50));
        }
        assert!(matches!(
            limiter.check("admin"),
            RateDecision::Locked { .. }
        ));
        std::thread::sleep(Duration::from_millis(80));
        assert_eq!(
            limiter.check("admin"),
            RateDecision::Allowed,
            "lockout must expire"
        );
    }

    #[test]
    fn rate_limiter_resets_on_success() {
        let limiter = LoginRateLimiter::new();
        limiter.record_failure("admin", MAX_LOGIN_ATTEMPTS, Duration::from_secs(60));
        limiter.record_failure("admin", MAX_LOGIN_ATTEMPTS, Duration::from_secs(60));
        limiter.record_success("admin");
        assert_eq!(
            limiter.check("admin"),
            RateDecision::Allowed,
            "successful login must clear the counter"
        );
    }

    #[tokio::test]
    async fn login_lockout_survives_correct_password() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("pool");
        crate::db::DatabaseManager::new(pool.clone())
            .initialize()
            .await
            .expect("migrations");
        seed_admin_if_empty_with_password(&pool, "correct-horse").await;
        let state = AppState::new(pool).await.expect("state");

        let form = |password: &str| LoginForm {
            username: "admin".to_string(),
            password: password.to_string(),
        };

        // Burn MAX_LOGIN_ATTEMPTS with wrong passwords.
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            post_login(State(state.clone()), Form(form("wrong"))).await;
        }

        // Even the CORRECT password is now rejected with a lockout message.
        let response = post_login(State(state.clone()), Form(form("correct-horse"))).await;
        let body = axum::body::to_bytes(response.into_body(), 1_048_576)
            .await
            .expect("body");
        let body = String::from_utf8_lossy(&body).to_string();
        assert!(
            body.contains("鎖定"),
            "locked account must be told about the lockout"
        );
    }

    /// Test-friendly seed: fixed password instead of random print.
    async fn seed_admin_if_empty_with_password(pool: &SqlitePool, password: &str) {
        let hash = hash_password(password).expect("hash");
        sqlx::query(
            "INSERT OR IGNORE INTO users (user_id, username, password_hash, is_active) \
             VALUES ('user-admin', 'admin', ?, 1)",
        )
        .bind(&hash)
        .execute(pool)
        .await
        .expect("seed");
    }
}
