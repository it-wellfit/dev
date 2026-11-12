-- 稽核請求脈絡（2026-11-12 規格補洞）
--
-- 為何需要：audit_log 原本只記錄「誰、做了什麼、何時」，沒有「從哪裡」。
-- 認證事件（登入成功/失敗、登出）與高風險操作在安全調查時必須可溯源
-- （IP、User-Agent），否則無法回答「這次登入來自何處」這類基本問題。
--
-- 欄位皆 nullable：歷史列不回填（不可捏造脈絡）；新事件由 write_event_ctx
-- 寫入，認證路徑必填（X-Forwarded-For 或 user-agent 可得時）。

ALTER TABLE audit_log ADD COLUMN ip_address TEXT;
ALTER TABLE audit_log ADD COLUMN user_agent TEXT;

-- 依來源 IP 查詢（安全事件溯源常用面）。
CREATE INDEX IF NOT EXISTS idx_audit_log_ip ON audit_log (ip_address);