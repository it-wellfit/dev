# 統一語言詞彙表 — Flaps ERP 工作區（DDD Ubiquitous Language）

> 領域知識 100% 正確性要求：每個詞彙對映到程式模組的具體型別/函式。新詞彙必須先入本表再入碼。

## 核心領域（Ledger）

| 詞彙 | 定義 | 程式對映 | 不變量 |
|------|------|----------|--------|
| 交易（Transaction） | 一次已登帳的會計事件，含編號、日期、摘要、狀態 | `ledger::Transaction`（聚合根標頭） | 日期不可未來；冪等 UUID 主鍵 |
| 分錄（Transaction Item） | 交易的一條借或貸記錄 | `ledger::TransactionItem` | 金額 >0 且 ≤DECIMAL(15,2)；借貸別 ∈ {DEBIT, CREDIT}；匯率 >0 |
| 借貸平衡（Balance） | Σ借方（整數分）= Σ貸方 且 >0 | `engine::validate_journal_entry` | INV-02；任何呼叫端不可繞過 |
| 整數分（Minor Units） | 金額的唯一儲存形態：i64，10^-2 貨幣單位 | `money::parse_decimal_to_minor`／`Money` 值物件 | INV-01：嚴禁浮點 |
| 匯率縮放 | 匯率以 10^4 縮放的整數（1.0000 → 10,000） | `exchange_rate_scaled` | >0 |
| 稽核事件（Audit Event） | 不可否認的操作軌跡 | `audit_log`（TRANSACTION_POSTED／IMPORT_CONFIRMED／PASSWORD_RESET） | 登帳/匯入/重設必寫 |

## 身分與存取（Identity & Access）

| 詞彙 | 定義 | 程式對映 | 規則 |
|------|------|----------|------|
| 使用者（User） | 可登入的操作者 | `users` 表（Argon2id 雜湊） | 停用即撤銷全部會話（P1-9） |
| 工作階段（Session） | 伺服器端可撤銷的登入狀態 | `sessions` 表＋`flaps_session` cookie（HttpOnly/SameSite） | 24h 過期；密碼重設/停用即撤銷 |
| 權限（Permission） | 以 permission_code 判定的能力 | `permissions` 表／`has_permission_db` | **即時 DB 查詢**（啟動後指派即時生效） |
| 速率鎖定（Rate Lockout） | 連續 5 次登入失敗 → 15 分鐘鎖定 | `auth::LoginRateLimiter` | 成功登入重置；鎖定訊息含剩餘秒數 |

## 資料獲取（Slice 8/10）

| 詞彙 | 定義 | 程式對映 | 規則 |
|------|------|----------|------|
| 預檢（Preview） | 匯入的乾跑：逐列驗證、零帳務副作用 | `/import/preview`＋`import_batches(status='draft')` | 上限 2MB/5,000 列 |
| 確認（Confirm） | 批次登帳：僅合格列、機械重放防護 | `/import/confirm` | 二次確認 → 409 |
| 依據卡（Evidence Card） | 解析器每欄位的規則命中紀錄 | `parser::Evidence`（rule＋matched） | 100% 可解釋 |
| 草稿（Draft） | 未經人工確認的分錄提案 | `drafts` 表（source: text/ai-adopted） | **永不自動落地**；確認走精靈 |
| AI 建議（AI Suggestion） | 模型產生的提案（需人工覆核） | `parser::AiSuggestion`／`parse_ai_suggestion` schema 閘門 | 旗標預設關閉；schema 驗證拒絕畸形 |
| 樞紐（Pivot） | 白名單維度的多維矩陣 | `/reports/pivot`＋`dim_expr`/`measure_expr` | R-PIV-WHITELIST；rows≠cols |
| 平行座標（Parallel Coordinates） | 每科目一折線跨越月份軸的多維投影 | `parallel::render_parallel_coordinates` | 整數 min-max 正規化 |

## 狀態機

```
匯入批次：draft ──confirm──> confirmed（不可逆；重放 409）
草稿：draft ──人工確認（精靈步驟 2→3→/transactions）──> POSTED
會話：active ──logout/密碼重設/帳號停用──> revoked（立即）
```
