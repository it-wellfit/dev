-- Migration 0010: self-service password reset (2026-11-12 瀏覽器實證補洞)
--
-- 本地部署無郵件基礎設施；流程為「請求 → 管理員核發一次性連結 → 使用者自設」：
--   password_reset_requests ?鑑身分請求（使用者送出帳號即建請求＋稽核）
--   password_reset_tokens   ?砸效一次性 token（僅存 SHA-256 雜湊；30 分鐘時效；
--                           用後即焚；重設成功撤銷該使用者全部 session）
--
-- Token 明文只出現一次（管理員核發頁面），永不入庫——與 session/CSRF 同一
-- 威脅模型（THREAT_MODEL T-01）。

CREATE TABLE IF NOT EXISTS password_reset_requests (
    request_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_prr_status ON password_reset_requests (status, created_at);

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    token_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    request_id TEXT,
    expires_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP,
    created_by TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE INDEX IF NOT EXISTS idx_prt_user ON password_reset_tokens (user_id);
