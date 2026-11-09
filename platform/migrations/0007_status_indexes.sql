-- 0007: 狀態查詢索引（計畫 1793782615513 §7.2 / Slice 13）。
--
-- audit_log 已有 performed_at DESC 索引（0005）；此處補 event_type 維度，
-- 使「依事件類型篩選近期稽核」（如 BANKPAY_*）為 index-covered 查詢，
-- 不需全表掃描（O(log n + k)）。

CREATE INDEX IF NOT EXISTS idx_audit_log_event_type
    ON audit_log (event_type, performed_at DESC);
