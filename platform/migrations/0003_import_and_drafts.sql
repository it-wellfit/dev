-- Migration 0003: import pipeline and draft storage (Slice 8, Layers A/C)
--
-- import_batches — one uploaded file (dry-run first; A2 confirm promotes it)
-- import_rows    — per-row parse/validate result, preserving the raw line
-- drafts         — unstructured-text parse output (Layer C), always confirmed
--                  by a human before posting (H3 trust model)

CREATE TABLE IF NOT EXISTS import_batches (
    batch_id TEXT PRIMARY KEY,
    file_sha256 TEXT NOT NULL,
    row_total INTEGER NOT NULL,
    row_ok INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    created_by TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS import_rows (
    batch_id TEXT NOT NULL,
    row_no INTEGER NOT NULL,
    raw_line TEXT NOT NULL,
    status TEXT NOT NULL,
    error_msg TEXT,
    draft_json TEXT NOT NULL,
    PRIMARY KEY (batch_id, row_no),
    FOREIGN KEY (batch_id) REFERENCES import_batches(batch_id)
);

CREATE TABLE IF NOT EXISTS drafts (
    draft_id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    draft_json TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_import_rows_batch ON import_rows (batch_id, row_no);
CREATE INDEX IF NOT EXISTS idx_drafts_created ON drafts (created_at DESC);
