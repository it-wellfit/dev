-- 0008: 稽核軌跡執行者索引（計畫 1793033761021 §15 C-1，P0 修復批次）。
--
-- /my-decisions（D-6 自我稽核視圖）以 WHERE performed_by = ? ORDER BY
-- performed_at DESC 查詢；audit_log 既有索引（0004 performed_at、0005
-- event_type、0007 event_type+performed_at）皆不覆蓋 performed_by 維度，
-- 查詢隨軌跡增長退化为全表掃描。本索引使自查為 index-covered
-- （O(log n + k)），與 p95 適應度函數的查詢複雜度精神一致。
--
-- 同時服務 /audit 的 user 篩選（audit_rows_page 動態 WHERE）。

CREATE INDEX IF NOT EXISTS idx_audit_log_performed_by
    ON audit_log (performed_by, performed_at DESC);
