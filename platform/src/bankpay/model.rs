//! E.Sun BankPay 資料模型。
//!
//! 定義銀行支付批次記錄與稽核記錄的結構，所有金額使用 i64 整數分 (INV-01)。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 銀行支付批次狀態機。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BankPayStatus {
    /// 初始狀態，尚未提交至銀行。
    Pending,
    /// 已提交至 E.Sun，等待處理結果。
    Submitted,
    /// 銀行處理成功。
    Completed,
    /// 銀行處理失敗。
    Failed,
    /// 銀行退件，需要人工複核。
    Rejected,
}

impl std::fmt::Display for BankPayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BankPayStatus::Pending => write!(f, "PENDING"),
            BankPayStatus::Submitted => write!(f, "SUBMITTED"),
            BankPayStatus::Completed => write!(f, "COMPLETED"),
            BankPayStatus::Failed => write!(f, "FAILED"),
            BankPayStatus::Rejected => write!(f, "REJECTED"),
        }
    }
}

/// 銀行支付批次記錄。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayBatch {
    pub batch_id: String,
    pub esun_ref: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub status: BankPayStatus,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 銀行支付稽核記錄。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayAudit {
    pub log_id: String,
    pub batch_id: String,
    pub event_type: String,
    pub performed_by: String,
    pub performed_at: DateTime<Utc>,
    pub request_payload: Option<String>,
    pub response_payload: Option<String>,
    pub status_code: u16,
}

/// 匯出請求。
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub batch_id: String,
    pub amount_minor: i64,
    pub currency: String,
}

/// 匯入請求。
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportRequest {
    pub file_path: String,
}

/// 查詢請求。
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub esun_ref: String,
}

/// 批次 ID／esun_ref 的格式約束：1–64 個英數、底線或連字號。
///
/// 防止路徑分隔字元、SQL 片段或控制字元經由識別碼欄位進入系統。
pub fn validate_identifier(value: &str, field: &str) -> Result<(), String> {
    let ok = !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok {
        Ok(())
    } else {
        Err(format!(
            "驗證失敗：{field}「{value}」格式無效。問題：僅接受 1–64 個英數字、底線或連字號；可能原因：含空白、路徑分隔字元或控制字元；處理方式：請使用如 BATCH-001 的識別碼後重新提交。"
        ))
    }
}

/// 幣別約束：三個大寫英文字母（ISO 4217）。
pub fn validate_currency(currency: &str) -> Result<(), String> {
    let ok = currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase());
    if ok {
        Ok(())
    } else {
        Err(format!(
            "驗證失敗：幣別「{currency}」無效。問題：幣別須為 3 個大寫英文字母（ISO 4217，如 TWD、USD）；可能原因：輸入小寫或長度錯誤；處理方式：修正後重新提交。"
        ))
    }
}

/// 金額約束：正整數分（INV-01，禁用浮點）。
pub fn validate_amount(amount_minor: i64) -> Result<(), String> {
    if amount_minor > 0 {
        Ok(())
    } else {
        Err(format!(
            "驗證失敗：金額 {amount_minor} 無效。問題：匯出金額必須為正整數（單位：分）；可能原因：金額為零、負值或溢位；處理方式：確認分錄金額後重新提交。"
        ))
    }
}

/// 匯出回應。
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub status: String,
    pub batch_id: String,
    pub esun_ref: Option<String>,
}

/// 匯入回應。
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub status: String,
    pub matched: usize,
    pub mismatched: usize,
}

/// 查詢回應。
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub status: String,
    pub amount_minor: Option<i64>,
    pub currency: Option<String>,
}

/// 匯出結果摘要。
#[derive(Debug, Serialize)]
pub struct ExportSummary {
    pub total_batches: usize,
    pub completed_batches: usize,
    pub pending_batches: usize,
    pub failed_batches: usize,
    pub rejected_batches: usize,
    pub total_amount_minor: i64,
}

/// E.Sun 銀行連線設定。
///
/// 欄位涵蓋 V2.8 連線要件（萃取自 Tauri 桌面版 `web/src-tauri/src/state.rs`
/// 的連線設定模型，兩作品同步演進——見 ADR-0012 附錄）：
/// - 內部/外部端點 URL（內部 dev/test 為 `http://<IP>:8080`、正式 `:8443`；
///   外部沙盒為 `https://eospt.testesunbank.com.tw:9443`）
/// - mTLS 三件套（client cert / client key / CA）
/// - 逾時秒數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsunConfig {
    pub esun_mock_mode: bool,
    pub esun_url: String,
    pub source_system_id: Option<String>,
    pub customer_id: Option<String>,
    pub payer_account: Option<String>,
    /// E.Sun 外部（沙盒/正式）端點，例 `https://eospt.testesunbank.com.tw:9443`。
    pub esun_external_url: String,
    /// mTLS 用戶端憑證路徑。
    pub client_cert_path: Option<String>,
    /// mTLS 用戶端私鑰路徑。
    pub client_key_path: Option<String>,
    /// E.Sun CA 憑證路徑。
    pub ca_cert_path: Option<String>,
    /// HTTP 逾時秒數（Okapi 預設 30）。
    pub timeout_secs: u64,
}

impl Default for EsunConfig {
    fn default() -> Self {
        Self {
            esun_mock_mode: true,
            // 誠實預設：未取得經驗證的端點規格與憑證前不猜測 URL（治理 P7；
            // 權威 V2.8 契約實作位於 Wellfit/crates/wellfit_bankpay）。
            esun_url: String::new(),
            source_system_id: None,
            customer_id: None,
            payer_account: None,
            esun_external_url: String::new(),
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            timeout_secs: 30,
        }
    }
}

/// 銀行支付儀表板卡片資料。
#[derive(Debug, Serialize)]
pub struct BankPayDashboardCard {
    pub total_batches: usize,
    pub completed_batches: usize,
    pub pending_batches: usize,
    pub failed_batches: usize,
    pub rejected_batches: usize,
    pub total_amount_minor: i64,
    pub recent_batches: Vec<BankPayBatchSummary>,
}

/// 銀行支付批次摘要（用於儀表板顯示）。
#[derive(Debug, Serialize)]
pub struct BankPayBatchSummary {
    pub batch_id: String,
    pub esun_ref: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}
