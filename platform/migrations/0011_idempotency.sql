-- G4.11 (D4-02): idempotency ledger for POST /api/* with an Idempotency-Key.
-- Keyed by (key, user_id): the same key from a different user never collides.
-- Stores the first response so replays return it verbatim (no re-execution).

CREATE TABLE IF NOT EXISTS idempotency_records (
    key         TEXT NOT NULL,
    user_id     TEXT NOT NULL,
    method      TEXT NOT NULL,
    path        TEXT NOT NULL,
    status      INTEGER NOT NULL,
    body        TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (key, user_id)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_created_at
    ON idempotency_records (created_at);