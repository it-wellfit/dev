//! E.Sun BankPay 整合模組（Slice 13）。
//!
//! 此模組將玉山銀行 BankPay 整合進平台主工作台，提供匯出、匯入、查詢
//! 三大 API 端點，並自動將每筆銀行作業寫入稽核軌跡 (audit_log)。
//!
//! 設計原則：
//! - 零 JavaScript：所有互動透過伺服器端渲染表單 + POST/Redirect/Get。
//! - 雙模式：模擬模式 (Mock) 與生產模式並存，透過 `ESUN_MOCK_MODE` 環境變數切換。
//! - 權限控管：匯出/匯入需要 `LEDGER_WRITE`，查詢需要 `LEDGER_READ`。
//! - 稽核完整：每筆作業自動寫入 `audit_log` 表，可追溯可驗證。

pub mod api;
pub mod client;
pub mod model;
pub mod state;
