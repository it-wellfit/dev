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

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use askama::Template;
use axum::extract::{ConnectInfo, Form, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::api::AppState;

/// Session lifetime: 24 hours (fixed; re-login renews).
pub const SESSION_TTL_HOURS: i64 = 24;
pub const SESSION_COOKIE: &str = "auxbridge_session";

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
    /// 顯示名（users.username；2026-11-13 補）：頂欄過去直接顯示 user_id
    /// （"user-admin"）而非帳號名（"admin"）——對使用者是無意義的內部識別碼。
    pub username: String,
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
        "SELECT s.session_id, s.user_id, COALESCE(u.username, s.user_id) AS username, \
                s.csrf_token, s.expires_at \
         FROM sessions s \
         LEFT JOIN users u ON u.user_id = s.user_id \
         WHERE s.session_id = ? AND s.expires_at > datetime('now')",
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

/// Read a security toggle from erp_config (whitelisted keys only).
pub async fn security_flag(state: &AppState, key: &str) -> bool {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'security' AND config_key = ?",
    )
    .bind(key)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);
    row.map(|(v,)| v == "1").unwrap_or(false)
}

/// Build the Set-Cookie header value for a session id. The `Secure`
/// attribute is config-driven (R-07): enable `security/cookie_secure` once
/// TLS terminates in front of this server; default OFF for local HTTP.
pub fn session_cookie_value(session_id: &str, secure: bool) -> String {
    let secure_attr = if secure { "; Secure" } else { "" };
    format!(
        "{SESSION_COOKIE}={session_id}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}{}",
        SESSION_TTL_HOURS * 3600,
        secure_attr
    )
}

/// Build the cookie-clearing header value.
pub fn clear_cookie_value() -> String {
    format!("{SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0")
}

/// Housekeeping at the auth boundary: purge expired session rows so the
/// table does not grow without bound (expired rows are already
/// non-authentic; this only reclaims storage).
pub async fn purge_expired_sessions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE expires_at <= datetime('now')")
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Security headers middleware (OWASP secure headers checklist, Slice 11)
// ---------------------------------------------------------------------------

/// Apply the secure-header set to every response. The CSP allows only
/// self-hosted styles/images plus `data:` favicons - the UI is zero-JS, so
/// no script sources are needed at all.
pub async fn security_headers(mut response: axum::response::Response) -> axum::response::Response {
    use axum::http::header::{
        CACHE_CONTROL, CONTENT_SECURITY_POLICY, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
        X_FRAME_OPTIONS,
    };
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_SECURITY_POLICY,
        axum::http::HeaderValue::from_static(
            "default-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; \
             form-action 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    headers.insert(
        X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        REFERRER_POLICY,
        axum::http::HeaderValue::from_static("no-referrer"),
    );
    // HSTS（2026-11-12 規格補洞）：cookie_secure 啟用（HTTPS 部署）時強制
    // 瀏覽器未來只走 HTTPS。於 http:// 部署時瀏覽器會忽略此標頭（無害），
    // 故可無條件送出；max-age 一年、includeSubDomains。
    headers.insert(
        axum::http::header::STRICT_TRANSPORT_SECURITY,
        axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

// ---------------------------------------------------------------------------
// Administrator bootstrap
// ---------------------------------------------------------------------------

/// Bootstrap the administrator account at startup.
///
/// Password source: `AUXBRIDGE_ADMIN_PASSWORD`; when absent, a random password is
/// generated and printed ONCE to stdout for the operator to store. Default
/// credentials are never silently installed.
///
/// 2026-11-15（登入可及性補洞，TT-1）：舊實作只要 `users` 表非空就直接 return，
/// 導致環境變數設定的密碼在第一次啟動後完全失效——資料庫一旦被其他流程改過密碼，
/// 維運端就再也無法用 `AUXBRIDGE_ADMIN_PASSWORD` 進入自己的部署（實測：登入頁
/// 持續回「帳號或密碼不正確」）。現行為：環境變數視為部署端密碼的權威來源，
/// 已存在但雜湊不符時旋轉密碼並撤銷該帳號所有 session。
///
/// 環境變數讀取與核心邏輯分離（`seed_admin`），使測試可在不動用行程全域環境
/// 變數的前提下驗證同步行為（edition 2024 的 `std::env::set_var` 為 unsafe，
/// 與 crate 頂端 `#![forbid(unsafe_code)]` 衝突，不得於測試中使用）。
pub async fn seed_admin_if_empty(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let configured = std::env::var("AUXBRIDGE_ADMIN_PASSWORD")
        .ok()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());
    seed_admin(pool, configured).await
}

/// Core bootstrap logic (testable): `configured` is the operator-provided
/// password; `None` means "first run, generate a random password once".
///
/// - `users` 表為空：以 `configured`（或隨機密碼）建立 `admin`。
/// - `users` 表非空且 `configured = None`：保持既有密碼不動（原行為）。
/// - `users` 表非空且 `configured = Some(p)`：若 `admin` 的雜湊無法通過 `p`
///   驗證，則旋轉密碼並撤銷該帳號所有 session（密碼變更必須使既有登入失效）。
pub async fn seed_admin(
    pool: &SqlitePool,
    configured: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        if let Some(password) = configured.as_deref() {
            sync_admin_password(pool, "admin", password).await?;
        }
        return Ok(());
    }

    let password = configured.unwrap_or_else(|| {
        let random = uuid::Uuid::new_v4().to_string().replace('-', "");
        let random = random[..16].to_string();
        println!("=== 首次啟動：已建立管理員帳號 admin，密碼（僅顯示這一次）：{random} ===");
        random
    });

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

/// Align the stored `admin` credential with the operator-configured password.
///
/// Returns `true` when the hash was rotated. Sessions of that account are
/// revoked on rotation: a password change must invalidate existing sessions
/// (same rule as `POST /admin/users/reset_password`).
async fn sync_admin_password(
    pool: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT user_id, password_hash FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await?;
    let Some((user_id, stored_hash)) = row else {
        return Ok(false);
    };
    if verify_password(password, &stored_hash) {
        return Ok(false);
    }
    let hash = hash_password(password)?;
    sqlx::query("UPDATE users SET password_hash = ? WHERE user_id = ?")
        .bind(&hash)
        .bind(&user_id)
        .execute(pool)
        .await?;
    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(&user_id)
        .execute(pool)
        .await;
    println!("管理員密碼已依環境變數同步（user_id={user_id}）；既有登入階段已撤銷。");
    Ok(true)
}

/// Single CSRF verdict for POST forms (PRE-R1b, 2026-11-17).
///
/// Returns `None` when the submitted token matches the acting session
/// (the request may proceed); otherwise the canonical 403 response. This is
/// the ONE place the canonical verdict lives — handlers must not hand-roll
/// the comparison. 13 former copies in `api.rs` were consolidated here;
/// heterogeneous flows (dashboard index re-render, decision fail-macro)
/// intentionally keep their own rendering paths and do NOT use this helper.
pub fn csrf_mismatch(submitted: &str, session_csrf: &str) -> Option<Response> {
    if submitted == session_csrf {
        None
    } else {
        Some(
            (
                StatusCode::FORBIDDEN,
                "CSRF 驗證失敗；請重新整理頁面後再操作。".to_string(),
            )
                .into_response(),
        )
    }
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

/// 使用者的主要角色碼（roles 表第一筆；無角色回 None）。
async fn primary_role_code(pool: &SqlitePool, user_id: &str) -> Option<String> {
    sqlx::query_scalar(
        "SELECT r.role_code FROM user_roles ur JOIN roles r ON ur.role_id = r.role_id \
         WHERE ur.user_id = ? ORDER BY r.role_code LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// POST /login — rate-limit check, authenticate, create a session (303 PRG).
/// 用戶端位址（可選）。
///
/// 為何需要：axum 0.8 移除 OptionalFromRequestParts，Option<ConnectInfo<..>>
/// 不再是合法 extractor；此型別自 extensions 提取，未經 connect-info 的測試
/// 環境（oneshot）仍可回 None，行為與 0.7 相同。
#[derive(Clone, Copy, Debug)]
pub struct ClientPeer(pub Option<std::net::SocketAddr>);

impl<S> axum::extract::FromRequestParts<S> for ClientPeer
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(ClientPeer(
            parts
                .extensions
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|info| info.0),
        ))
    }
}
pub async fn post_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    peer: ClientPeer,
    Form(form): Form<LoginForm>,
) -> Response {
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
            // Slice 11: housekeeping at the auth boundary - expired session
            // rows are purged whenever a new session is created.
            let _ = purge_expired_sessions(&state.pool).await;
            // 稽核：登入成功必可溯源（2026-11-12 規格補洞）。
            let (ip, ua) = request_context_with_peer(&headers, peer.0);
            crate::pages::write_event_ctx(
                &state.pool,
                "LOGIN_SUCCEEDED",
                &user_id,
                serde_json::json!({ "username": form.username }),
                ip.as_deref(),
                ua.as_deref(),
            )
            .await;
            let secure = security_flag(&state, "cookie_secure").await;
            match create_session(&state.pool, &user_id).await {
                Ok((session_id, _)) => {
                    // 角色落地頁（不同角色登入後看到不同首頁）：
                    // ADMIN → 平台總覽；FINANCE → 試算表（帳務焦點）；
                    // STORE/POS → 交易清單（作業焦點）。
                    let landing = match primary_role_code(&state.pool, &user_id).await {
                        Some(rc) if rc == "FINANCE" => "/reports/trial-balance",
                        Some(rc) if rc == "STORE" || rc == "POS" => "/transactions",
                        _ => "/",
                    };
                    let mut response = Redirect::to(landing).into_response();
                    if let Ok(value) =
                        header::HeaderValue::from_str(&session_cookie_value(&session_id, secure))
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
            // 稽核：登入失敗（含嘗試帳號）必可溯源——暴力破解調查的基本資料。
            let (ip, ua) = request_context_with_peer(&headers, peer.0);
            crate::pages::write_event_ctx(
                &state.pool,
                "LOGIN_FAILED",
                "unknown",
                serde_json::json!({ "attempted_username": form.username }),
                ip.as_deref(),
                ua.as_deref(),
            )
            .await;
            login_page(Some(LOGIN_FAILURE_MSG.to_string()), &form.username)
        }
    }
}

/// 從請求標頭擷取稽核脈絡（2026-11-12 規格補洞）。
///
/// 反向代理情境用 `X-Forwarded-For` 第一跳；無則 None（誠實標示未知，
/// 不猜測、不捏造）。User-Agent 逾 512 字元截斷（避免巨大 header 膨脹稽核表）。
/// 帶對端位址的稽核脈絡（2026-11-12 規格補洞）：優先序
/// `X-Forwarded-For`（反向代理）→ `ConnectInfo` 對端 IP → None（不捏造）。
/// User-Agent 逾 512 字元截斷（避免巨大 header 膨脹稽核表）。
fn request_context_with_peer(
    headers: &HeaderMap,
    peer: Option<std::net::SocketAddr>,
) -> (Option<String>, Option<String>) {
    let forwarded = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let ip = forwarded.or_else(|| peer.map(|p| p.ip().to_string()));
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.chars().take(512).collect::<String>());
    (ip, ua)
}

#[derive(Debug, Deserialize)]
pub struct LogoutForm {
    pub csrf_token: String,
}

/// POST /logout — CSRF-checked session revocation + cookie clearing.
pub async fn post_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    peer: ClientPeer,
    Form(form): Form<LogoutForm>,
) -> Response {
    if let Some(session) = current_session(&state, &headers).await
        && session.csrf_token == form.csrf_token
    {
        let _ = delete_session(&state.pool, &session.session_id).await;
        // 稽核：登出（工作階段撤銷）可溯源（2026-11-12 規格補洞）。
        let (ip, ua) = request_context_with_peer(&headers, peer.0);
        crate::pages::write_event_ctx(
            &state.pool,
            "LOGOUT",
            &session.user_id,
            serde_json::json!({ "session_id": session.session_id }),
            ip.as_deref(),
            ua.as_deref(),
        )
        .await;
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
            // W2（grill-B：middleware 注入）：NavFlags 一次查詢 → NavModel
            // 純記憶體過濾 → Extension；handler 一行複製。API/JSON 路徑亦付
            // 一次查詢（本機 SQLite <1ms；page_matrix 效能斷言驗證預算）。
            let flags = crate::api::nav_flags(&state, &session.user_id).await;
            let current_path = request.uri().path().to_string();
            request
                .extensions_mut()
                .insert(crate::nav::NavModel::build(&flags, &current_path));
            request.extensions_mut().insert(session);
            next.run(request).await
        }
        None => Redirect::to("/login").into_response(),
    }
}

/// API 版 session 中間件：未登入回 401 JSON（而非 HTML 轉址）。
///
/// 用於 `/api/*` 路由群組——API 用戶端（curl/前端 fetch）需要機器可判讀的
/// 狀態碼，且計畫驗收標準明定「未登入用戶存取 API 返回 401」。
pub async fn require_session_api(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    match current_session(&state, request.headers()).await {
        Some(session) => {
            request.extensions_mut().insert(session);
            next.run(request).await
        }
        None => {
            let request_id = request
                .extensions()
                .get::<crate::api_error::RequestId>()
                .map(|r| r.0.clone())
                .unwrap_or_else(|| crate::api_error::request_id_from_headers(request.headers()));
            crate::api_error::api_error(
                axum::http::StatusCode::UNAUTHORIZED,
                "UNAUTHENTICATED",
                "未授權：請先登入（缺少有效 auxbridge_session Cookie）。",
                "問題：未登入；可能原因：工作階段過期或 Cookie 被阻擋；處理方式：先以 /login 取得 session 後重試。",
                false,
                &request_id,
            )
        }
    }
}

// F5 死碼清除（計畫 1793033568961 W3；2026-11-10）：原 `pub fn login_router()`
// 為便利建構器，但零呼叫者（/login、/logout 由 api.rs inline 註冊）。路由註冊
// 單一來源＝api.rs。

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
            post_login(
                State(state.clone()),
                HeaderMap::new(),
                ClientPeer(None),
                Form(form("wrong")),
            )
            .await;
        }

        // Even the CORRECT password is now rejected with a lockout message.
        let response = post_login(
            State(state.clone()),
            HeaderMap::new(),
            ClientPeer(None),
            Form(form("correct-horse")),
        )
        .await;
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

    #[test]
    fn csrf_mismatch_is_single_verdict_point() {
        assert!(
            csrf_mismatch("tok", "tok").is_none(),
            "matching token must let the request proceed"
        );
        let response = csrf_mismatch("wrong", "tok").expect("mismatch must yield response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    /// In-memory pool with all migrations applied (test helper).
    async fn migrated_pool() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("pool");
        crate::db::DatabaseManager::new(pool.clone())
            .initialize()
            .await
            .expect("migrations");
        pool
    }

    #[tokio::test]
    async fn seed_admin_creates_admin_with_configured_password() {
        let pool = migrated_pool().await;
        seed_admin(&pool, Some("configured-pass-123".to_string()))
            .await
            .expect("seed");
        assert_eq!(
            authenticate(&pool, "admin", "configured-pass-123").await,
            Some("user-admin".to_string()),
            "the configured password must authenticate right after the first bootstrap"
        );
        assert!(
            authenticate(&pool, "admin", "wrong-password")
                .await
                .is_none(),
            "a wrong password must still be rejected"
        );
    }

    #[tokio::test]
    async fn seed_admin_rotates_password_when_configured_password_changed() {
        let pool = migrated_pool().await;
        seed_admin(&pool, Some("old-pass-123".to_string()))
            .await
            .expect("seed");
        seed_admin(&pool, Some("new-pass-456".to_string()))
            .await
            .expect("reseed");
        assert_eq!(
            authenticate(&pool, "admin", "new-pass-456").await,
            Some("user-admin".to_string()),
            "the newly configured password must work after rotation"
        );
        assert!(
            authenticate(&pool, "admin", "old-pass-123").await.is_none(),
            "the previous password must stop working after rotation"
        );
    }

    #[tokio::test]
    async fn seed_admin_keeps_existing_password_when_none_configured() {
        let pool = migrated_pool().await;
        seed_admin(&pool, Some("kept-pass-123".to_string()))
            .await
            .expect("seed");
        seed_admin(&pool, None).await.expect("reseed");
        assert_eq!(
            authenticate(&pool, "admin", "kept-pass-123").await,
            Some("user-admin".to_string()),
            "without a configured password the stored hash must be preserved"
        );
    }

    #[tokio::test]
    async fn seed_admin_is_idempotent_when_password_already_matches() {
        let pool = migrated_pool().await;
        seed_admin(&pool, Some("same-pass-123".to_string()))
            .await
            .expect("seed");
        let (before,): (String,) =
            sqlx::query_as("SELECT password_hash FROM users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .expect("hash");
        seed_admin(&pool, Some("same-pass-123".to_string()))
            .await
            .expect("reseed");
        let (after,): (String,) =
            sqlx::query_as("SELECT password_hash FROM users WHERE username = 'admin'")
                .fetch_one(&pool)
                .await
                .expect("hash");
        assert_eq!(
            before, after,
            "a matching configured password must not rewrite the stored hash"
        );
    }

    #[tokio::test]
    async fn seed_admin_revokes_sessions_when_password_rotated() {
        let pool = migrated_pool().await;
        seed_admin(&pool, Some("old-pass-123".to_string()))
            .await
            .expect("seed");
        create_session(&pool, "user-admin").await.expect("session");
        let (before,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = 'user-admin'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(before, 1, "a session row must exist before the rotation");
        seed_admin(&pool, Some("new-pass-456".to_string()))
            .await
            .expect("reseed");
        let (after,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = 'user-admin'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(
            after, 0,
            "rotating the password must revoke every session of that account"
        );
    }
}
