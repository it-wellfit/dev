# 待覆核佇列種子流程（受控；測試/演示環境專用）

> 計畫 1793033761021 §16 T-P0.4｜D-7（Q10）短期方案 (c)
> 目的：在「匯入→PENDING_REVIEW」上游管線（方案 (a)，下一切片）落地前，
> 提供受控的待覆核種子，供 M-1/M-2 真人基線量測與 demo 使用。
> **警告**：本流程僅限測試/演示環境。生產環境禁用（種子交易無真實憑證，
> 且 A-4 保留政策下審計軌跡不可事後清理——生產種子將永久污染軌跡）。

## 1. 種子 SQL（10 筆 PENDING_REVIEW）

```sql
-- 受控種子：待覆核佇列 10 筆（計畫 1793033761021 T-P0.4）
-- 執行前檢查：SELECT COUNT(*) FROM transactions WHERE transaction_id LIKE 'seed-pr-%';
-- 預期 0（重複執行前先跑 §3 還原指令）。
INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, status, created_by) VALUES
  ('seed-pr-01', 'TX-SEEDPR01', '2026-11-12', '基線量測種子 1', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-02', 'TX-SEEDPR02', '2026-11-12', '基線量測種子 2', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-03', 'TX-SEEDPR03', '2026-11-12', '基線量測種子 3', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-04', 'TX-SEEDPR04', '2026-11-12', '基線量測種子 4', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-05', 'TX-SEEDPR05', '2026-11-12', '基線量測種子 5', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-06', 'TX-SEEDPR06', '2026-11-12', '基線量測種子 6', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-07', 'TX-SEEDPR07', '2026-11-12', '基線量測種子 7', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-08', 'TX-SEEDPR08', '2026-11-12', '基線量測種子 8', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-09', 'TX-SEEDPR09', '2026-11-12', '基線量測種子 9', 'PENDING_REVIEW', 'seed-operator'),
  ('seed-pr-10', 'TX-SEEDPR10', '2026-11-12', '基線量測種子 10', 'PENDING_REVIEW', 'seed-operator');
```

執行後檢查：`SELECT COUNT(*) FROM transactions WHERE transaction_id LIKE 'seed-pr-%' AND status='PENDING_REVIEW';` 預期 **10**。

> 已知限制：種子列不含 `transaction_items` 分錄（批量覆核僅讀
> transactions.status/created_by，不影響裁決流程）；點入詳情頁將無分錄可看
> ——屬種子語義，如需完整詳情請改走 (a) 管線後的真實匯入流。

## 2. 真人基線量測步驟（M-1）

1. 以覆核帳號登入工作台（首屏＝待覆核隊列，渲染即 t0）。
2. 於「交易編號」欄貼入上方 10 個 `seed-pr-XX`（逗號分隔）。
3. 選擇裁決方向（核准／駁回）。
4. 展開「送出批量裁決」→「確認送出」。
5. 事後查詢（固化用）：

```sql
SELECT event_data, performed_at FROM audit_log
WHERE event_type = 'REVIEW_BULK_SUMMARY'
ORDER BY performed_at DESC LIMIT 1;
-- elapsed_ms 即本次任務時間（伺服器渲染→提交送達）。
-- 聚合規則（E-1）：基線＝近 30 日 ok_count>0 之 elapsed_ms 的 P50/P95，
-- 樣本 ≥10；ok_count=0（重放）列不計。
```

## 3. 還原指令（種子清理）

```sql
DELETE FROM audit_log WHERE transaction_id LIKE 'seed-pr-%';
DELETE FROM transactions WHERE transaction_id LIKE 'seed-pr-%';
```

執行後檢查：`SELECT COUNT(*) FROM transactions WHERE transaction_id LIKE 'seed-pr-%';` 預期 **0**。

## 4. 檔案完整性

- 本檔 SHA-256：見檔尾「完整性」行（寫入後由工具計算並回填）。
- 修改本 SQL 須同步更新雜湊並於計畫 addendum 記錄修訂（C-01/C-10）。

## 5. 與 (a) 管線的關係

方案 (a)（import_confirm 建立 PENDING_REVIEW）落地後，本檔降級為 demo
輔助；真實資料流以匯入憑證為源（屆時 M-2 ground truth 的雙人複核抽樣
協議才有真實憑證可比對——見計畫 §15 E-2）。

完整性：SHA-256 = E5672A759E64083295A946D109214ECC554EC5D7551CB3BD25DD55587CB65301（2026-11-12 計算；不含本行）
