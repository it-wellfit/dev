-- 0006: E.Sun BankPay 批次帳與稽核明細（計畫 1793782615513 §5.1.4）。
--
-- 冪等 SSOT：bankpay_batches.batch_id 主鍵唯一——重複匯出（雙擊、重試、
-- CSRF 重放）於資料庫層即被約束，handler 先 INSERT OR CONFLICT DO NOTHING
-- 再讀回現況，已 SUBMITTED/COMPLETED 的批次不重複呼叫銀行。
-- 金額一律整數分 i64（INV-01，禁用浮點）；狀態受 CHECK 約束（白名單）。

CREATE TABLE IF NOT EXISTS bankpay_batches (
    batch_id TEXT PRIMARY KEY,
    esun_ref TEXT UNIQUE,
    amount_minor INTEGER NOT NULL CHECK (amount_minor >= 0),
    currency TEXT NOT NULL DEFAULT 'TWD'
        CHECK (length(currency) = 3 AND currency = upper(currency)),
    status TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING','SUBMITTED','COMPLETED','FAILED','REJECTED')),
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    CHECK (created_by <> '')
);

CREATE INDEX IF NOT EXISTS idx_bankpay_batches_status
    ON bankpay_batches (status);
CREATE INDEX IF NOT EXISTS idx_bankpay_batches_created_at
    ON bankpay_batches (created_at DESC);

-- BankPay 專屬稽核明細（audit_log 為全域稽核 SSOT；本表保存批次維度的
-- 請求/回應原始負載與 HTTP 狀態碼，供對帳與爭議處理）。
CREATE TABLE IF NOT EXISTS bankpay_audits (
    log_id TEXT PRIMARY KEY,
    batch_id TEXT,
    event_type TEXT NOT NULL
        CHECK (event_type LIKE 'BANKPAY_%'),
    performed_by TEXT NOT NULL,
    performed_at TEXT NOT NULL,
    request_payload TEXT,
    response_payload TEXT,
    status_code INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (batch_id) REFERENCES bankpay_batches (batch_id)
);

CREATE INDEX IF NOT EXISTS idx_bankpay_audits_batch
    ON bankpay_audits (batch_id, performed_at DESC);
CREATE INDEX IF NOT EXISTS idx_bankpay_audits_event
    ON bankpay_audits (event_type, performed_at DESC);
