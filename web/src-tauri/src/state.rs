#![allow(dead_code)]
// Application State for Flaps ERP
// INV-01 對齊（2026-11-09）：金額一律 i64 minor（整數分），禁用浮點；
// 資料鏡射自 V2.8／domain 模型，供作者面工具使用（UI SSOT 為 platform 伺服器）。 Web
// Tauri 2.0 State Management

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub environment: String,
    pub mock_mode: bool,
    pub esun_base_url: String,
    pub esun_external_url: String,
    pub source_system_id: String,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub ca_cert_path: Option<String>,
    pub timeout_secs: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            environment: std::env::var("ESUN_ENV").unwrap_or_else(|_| "development".to_string()),
            mock_mode: std::env::var("ESUN_MOCK_MODE").unwrap_or_else(|_| "true".to_string())
                == "true",
            esun_base_url: std::env::var("ESUN_BASE_URL")
                .unwrap_or_else(|_| "https://127.0.0.1:8443".to_string()),
            esun_external_url: std::env::var("ESUN_EXTERNAL_URL")
                .unwrap_or_else(|_| "https://eospt.testesunbank.com.tw:9443".to_string()),
            source_system_id: std::env::var("ESUN_SOURCE_SYSTEM_ID").unwrap_or_default(),
            client_cert_path: std::env::var("ESUN_CLIENT_CERT").ok(),
            client_key_path: std::env::var("ESUN_CLIENT_KEY").ok(),
            ca_cert_path: std::env::var("ESUN_CA_CERT").ok(),
            timeout_secs: std::env::var("ESUN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_transactions: i32,
    pub total_amount_minor: i64,
    pub active_accounts: i32,
    pub pending_transactions: i32,
    pub avg_processing_time_ms: u64,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall: String,
    pub api_status: Vec<ApiStatus>,
    pub connection_status: ConnectionStatus,
    pub security_status: SecurityStatus,
    pub last_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStatus {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub purpose: String,
    pub environment: String,
    pub last_check: Option<String>,
    pub result: String,
    pub response_time_ms: Option<u64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub tls_version: String,
    pub tls_ok: bool,
    pub two_way_ssl: bool,
    pub cert_status: String,
    pub cert_expiry: Option<String>,
    pub port_8443: bool,
    pub port_9443: bool,
    pub security_component: bool,
    pub device_binding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub cert_pinning: bool,
    pub source_system_id: bool,
    pub wdac_enabled: bool,
    pub mock_mode: bool,
    pub sensitive_data_masked: bool,
    pub csrf_protection: bool,
    pub xss_protection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance_minor: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub date: String,
    pub status: String,
    pub created_by: String,
    pub bank_ref: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCode {
    pub code: String,
    pub description: String,
    pub category: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRecord {
    pub id: String,
    pub api_name: String,
    pub test_type: String,
    pub result: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseChecklistItem {
    pub id: String,
    pub category: String,
    pub item: String,
    pub description: String,
    pub responsible: String,
    pub verification: String,
    pub status: String,
    pub evidence: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentRequest {
    pub payer_account_no: String,
    pub payer_id: String,
    pub payee_bank_id: String,
    pub payee_branch_id: String,
    pub payee_account_no: String,
    pub payee_name: String,
    pub payee_id: Option<String>,
    pub tx_date: String,
    pub tx_amt: String,
    pub charge_fee_type: String,
    pub memo: Option<String>,
    pub notify_method: Option<String>,
    pub notify_email: Option<String>,
    pub notify_fax: Option<String>,
    pub write_off_no: Option<String>,
    pub remark: Option<String>,
    pub tx_seq: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentResponse {
    pub msg_id: String,
    pub log_id: String,
    pub response_time: String,
    pub code: String,
    pub message: String,
    pub more_info: Option<String>,
    pub data_count: Option<String>,
    pub tx_record_list: Vec<TwdPaymentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentRecord {
    pub tx_seq: String,
    pub auto_delay: String,
    pub payer_account_no: String,
    pub payer_id: String,
    pub payee_bank_id: String,
    pub payee_branch_id: String,
    pub payee_account_no: String,
    pub payee_name: String,
    pub payee_id: String,
    pub tx_date: String,
    pub tx_amt: String,
    pub charge_fee_type: String,
    pub memo: String,
    pub notify_method: String,
    pub notify_email: String,
    pub notify_fax: String,
    pub write_off_no: String,
    pub remark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdStatusResponse {
    pub msg_id: String,
    pub log_id: String,
    pub response_time: String,
    pub code: String,
    pub message: String,
    pub tx_status: Option<String>,
    pub bank_ref: Option<String>,
    pub tx_amt: Option<String>,
    pub tx_date: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceResponse {
    pub msg_id: String,
    pub log_id: String,
    pub response_time: String,
    pub code: String,
    pub message: String,
    pub account_no: Option<String>,
    pub currency: Option<String>,
    pub available_balance: Option<String>,
    pub ledger_balance: Option<String>,
    pub pending_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResultResponse {
    pub msg_id: String,
    pub log_id: String,
    pub response_time: String,
    pub code: String,
    pub message: String,
    pub pdf_base64: Option<String>,
    pub file_name: Option<String>,
}
