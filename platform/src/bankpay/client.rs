//! E.Sun BankPay API 客戶端。
//!
//! 負責與 E.Sun 銀行系統通訊，支援模擬模式 (Mock) 與生產模式。
//! 在模擬模式下不進行實際網路請求，適合開發與測試環境。

use crate::bankpay::model::{EsunConfig, ExportRequest, ImportRequest, QueryRequest};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// E.Sun 銀行付款請求（內部格式）。
#[derive(Debug, Serialize)]
struct EsunPaymentRequest {
    pub source_system_id: String,
    pub customer_id: String,
    pub payer_account: String,
    pub payee_account: String,
    pub amount_minor: i64,
    pub currency: String,
    pub batch_id: String,
    pub remark: Option<String>,
}

/// E.Sun 銀行付款回應（內部格式）。
#[derive(Debug, Deserialize)]
struct EsunPaymentResponse {
    pub esun_ref: String,
    pub status: String,
    pub message: Option<String>,
}

/// E.Sun 銀行查詢回應（內部格式）。
#[derive(Debug, Deserialize)]
struct EsunQueryResponse {
    pub esun_ref: String,
    pub status: String,
    pub amount_minor: Option<i64>,
    pub currency: Option<String>,
}

/// E.Sun BankPay 客戶端。
pub struct EsunClient {
    config: EsunConfig,
    http_client: reqwest::Client,
}

impl EsunClient {
    /// 建立新的 E.Sun 客戶端。
    pub fn new(config: EsunConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("HTTP client 建立失敗"),
        }
    }

    /// 匯出付款批次至 E.Sun。
    pub async fn export(&self, req: &ExportRequest) -> Result<ExportResponse, String> {
        if self.config.esun_mock_mode {
            return self.mock_export(req);
        }

        let payment_req = EsunPaymentRequest {
            source_system_id: self
                .config
                .source_system_id
                .clone()
                .unwrap_or_else(|| "DEFAULT".to_string()),
            customer_id: self
                .config
                .customer_id
                .clone()
                .unwrap_or_else(|| "DEFAULT".to_string()),
            payer_account: self
                .config
                .payer_account
                .clone()
                .unwrap_or_else(|| "DEFAULT".to_string()),
            payee_account: "DEFAULT".to_string(),
            amount_minor: req.amount_minor,
            currency: req.currency.clone(),
            batch_id: req.batch_id.clone(),
            remark: Some(format!("Batch {}", req.batch_id)),
        };

        let resp: EsunPaymentResponse = self
            .http_client
            .post(format!("{}/v1/payment/export", self.config.esun_url))
            .json(&payment_req)
            .send()
            .await
            .map_err(|e| format!("E.Sun 匯出請求失敗：{}", e))?
            .json()
            .await
            .map_err(|e| format!("E.Sun 匯出回應解析失敗：{}", e))?;

        Ok(ExportResponse {
            status: resp.status,
            batch_id: req.batch_id.clone(),
            esun_ref: Some(resp.esun_ref),
        })
    }

    /// 匯入銀行回應檔。
    pub async fn import(&self, req: &ImportRequest) -> Result<ImportResponse, String> {
        if self.config.esun_mock_mode {
            return self.mock_import(req);
        }

        // 讀取檔案並上傳
        let file_content =
            std::fs::read(&req.file_path).map_err(|e| format!("檔案讀取失敗：{}", e))?;

        let form = reqwest::multipart::Part::bytes(file_content).file_name(
            std::path::Path::new(&req.file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        let form = reqwest::multipart::Form::new().part("file", form);

        let _resp: serde_json::Value = self
            .http_client
            .post(format!("{}/v1/payment/import", self.config.esun_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("E.Sun 匯入請求失敗：{}", e))?
            .json()
            .await
            .map_err(|e| format!("E.Sun 匯入回應解析失敗：{}", e))?;

        // 簡化：假設匯入成功
        Ok(ImportResponse {
            status: "processed".to_string(),
            matched: 0,
            mismatched: 0,
        })
    }

    /// 查詢銀行交易狀態。
    pub async fn query(&self, req: &QueryRequest) -> Result<QueryResponse, String> {
        if self.config.esun_mock_mode {
            return self.mock_query(req);
        }

        let resp: EsunQueryResponse = self
            .http_client
            .get(format!(
                "{}/v1/payment/query?esun_ref={}",
                self.config.esun_url, req.esun_ref
            ))
            .send()
            .await
            .map_err(|e| format!("E.Sun 查詢請求失敗：{}", e))?
            .json()
            .await
            .map_err(|e| format!("E.Sun 查詢回應解析失敗：{}", e))?;

        Ok(QueryResponse {
            status: resp.status,
            amount_minor: resp.amount_minor,
            currency: resp.currency,
        })
    }

    /// 獲取 E.Sun 連線配置。
    pub fn config(&self) -> &EsunConfig {
        &self.config
    }

    /// 檢查是否為模擬模式。
    pub fn is_mock_mode(&self) -> bool {
        self.config.esun_mock_mode
    }

    // ----- 模擬模式實作 -----

    fn mock_export(&self, req: &ExportRequest) -> Result<ExportResponse, String> {
        // 模擬網路延遲
        std::thread::sleep(Duration::from_millis(100));

        let esun_ref = format!("ESUN-{}-{}", req.batch_id, chrono::Utc::now().timestamp());

        Ok(ExportResponse {
            status: "SUBMITTED".to_string(),
            batch_id: req.batch_id.clone(),
            esun_ref: Some(esun_ref),
        })
    }

    fn mock_import(&self, _req: &ImportRequest) -> Result<ImportResponse, String> {
        std::thread::sleep(Duration::from_millis(150));

        Ok(ImportResponse {
            status: "processed".to_string(),
            matched: 5,
            mismatched: 0,
        })
    }

    fn mock_query(&self, req: &QueryRequest) -> Result<QueryResponse, String> {
        std::thread::sleep(Duration::from_millis(80));

        Ok(QueryResponse {
            status: "completed".to_string(),
            amount_minor: Some(50000),
            currency: Some("TWD".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_config_default() {
        let config = EsunConfig::default();
        assert!(config.esun_mock_mode);
        assert!(!config.esun_url.is_empty());
    }

    #[test]
    fn test_client_creation() {
        let config = EsunConfig::default();
        let client = EsunClient::new(config);
        assert!(client.is_mock_mode());
    }
}
