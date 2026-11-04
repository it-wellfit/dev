-- Migration 0002: users and server-side sessions (G-1, THREAT_MODEL T-01/R-06)
--
-- Minimal authentication baseline:
--   users    — credential store (Argon2id hashes only, never plaintext)
--   sessions — server-side, revocable sessions bound to a per-session CSRF token
--
-- The RBAC mapping (user_roles) continues to own authorization; users.user_id
-- joins it. Until a user-management UI exists, the operator seeds the first
-- account at server startup (see bin/flaps-server.rs).

CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    csrf_token TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions (expires_at);
