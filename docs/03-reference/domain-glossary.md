# 領域專業術語與實體別名對照表 (SSOT)
## Domain Glossary & Entity Mapping Specification

> **文件路徑**: `docs/03-reference/domain-glossary.md`  
> **單一真理源**: 本文件為專案所有業務名詞、資料庫欄位與程式型別唯一權威對照表。  
> **認知身分**: `[FACT:DOC]` ⨁ `[FACT:CODE]`

---

## 📖 1. 核心業務名詞定義

| 中文標準名 | 英文標準名 | 縮寫 | 白話直覺定義 | 本系統特定用途與代碼對應 | 易混淆概念區分 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **輔翼 ERP** | Flaps ERP | `Flaps` | 企業內部核心會計、進銷存與總帳系統。 | RPA 自動化目標，透過 Win32/COM 對接。 | 與本平台的 Axum Web 服務區分，此為外部既有 Legacy 系統。 |
| **哈瓦士 MTRIX** | Havaianas MTRIX | `MTRIX` | 國際品牌供應鏈與經銷對帳報表格式。 | 56 欄複雜資料清洗與跨檔勾稽引擎。 | 非單純的訂單表，而是含多階通路拆帳之矩陣報表。 |
| **蝦皮訂單流** | Shopee Order Stream | `Shopee` | 電商平台交易與撥款報表。 | 58 欄電商資料清洗、代收代付拆解。 | 撥款日報與訂單出貨日不同，需進行時序對照。 |
| **冪等印記** | Idempotency Key | `IdemKey` | 標記一次獨立業務操作的唯一指紋。 | 寫入 SQLite 與外部系統前檢核，防止重送。 | 與一般的資料庫主鍵（PK）不同，它代表客戶端的「操作意圖」。 |
| **單執行緒套間** | Single-Threaded Apartment | `STA` | 微軟 COM 物件執行模型，綁定特定線程。 | 封裝於 Dedicated Thread 中執行輔翼 RPA 操作。 | 嚴禁與多執行緒套間 (MTA) 或 Tokio 非同步池混用。 |

---

## 🗄️ 2. 資料欄位跨系統映射表 (Cross-System Mapping)

| 領域模型標準名稱 | 輔翼 ERP 欄位名 | 蝦皮報表欄位名 | MTRIX 報表欄位名 | 內部型別定義 | 業務限制與約束 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `order_id` | `TB_ORDER.ORD_NO` | 訂單編號 | OrderReference | `String (UUID/Varchar)` | 必填，全域唯一 |
| `settled_amount`| `TB_ORDER.AMT_NTD` | 買家支付總金額 | SettleAmt | `Money { cents: i64 }` | 剛性整數分，禁止浮點數 |
| `platform_fee` | `TB_ORDER.FEE_EXP` | 交易手續費+佣金 | CommissionAmt | `Money { cents: i64 }` | 必須大於或等於 0 |
| `payout_date` | `TB_ORDER.PAY_DATE` | 撥款完成日期 | ClearingDate | `chrono::NaiveDate` | ISO 8601 格式 YYYY-MM-DD |
| `audit_status` | `TB_ORDER.CHK_FLAG` | 撥款狀態 | ReconciliationFlag | `Enum (Audited, Pending, Diff)` | 狀態機控制項 |

