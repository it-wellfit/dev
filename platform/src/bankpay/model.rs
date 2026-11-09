//! E.Sun BankPay 資料模型。
//!
//! 定義銀行支付批次記錄與稽核記錄的結構，所有金額使用 i64 整數分 (INV-01)。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

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
    pub created_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}

/// 銀行支付稽核記錄。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankPayAudit {
    pub log_id: String,
    pub batch_id: String,
    pub event_type: String,
    pub performed_by: String,
    pub performed_at: OffsetDateTime,
    pub request_payload: Option<String>,
    pub response_payload: Option<String>,
    pub status_code: u16,
}

/// 匯出請求。
#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub batch_id: String,
    pub amount_minor: i64,
    pub currency: String,
}

/// 匯入請求。
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub file_path: String,
}

/// 查詢請求。
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub esun_ref: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsunConfig {
    pub esun_mock_mode: bool,
    pub esun_url: String,
    pub source_system_id: Option<String>,
    pub customer_id: Option<String>,
    pub payer_account: Option<String>,
}

impl Default for EsunConfig {
    fn default() -> Self {
        Self {
            esun_mock_mode: true,
            esun_url: "https://esunbank.com.tw/bankpay".to_string(),
            source_system_id: None,
            customer_id: None,
            payer_account: None,
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
