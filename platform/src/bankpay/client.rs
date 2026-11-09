//! E.Sun BankPay 客戶端（port 層）。
//!
//! 雙模式設計（治理 P7：不將不可驗證的模擬呈現為真實）：
//!
//! - **模擬模式（Mock，預設）**：確定性模擬器——無網路、無隨機、無合成金額。
//!   `esun_ref` 由 `batch_id` 決定性推導（可重放、可測試）；匯入仍真實解析
//!   檔案內容並回報實際筆數，不虛構匹配數。
//! - **生產模式**：明確 fail-closed。E.Sun V2.8 契約（`rqHeader`/`rqData`
//!   信封、mTLS、CP950/Big5 媒體編碼、`rsStatus` 代碼語意）之權威實作位於
//!   `Wellfit/crates/wellfit_bankpay`（8 端點、17 測試、冪等帳本、ReleaseGate）。
//!   本平台在取得經驗證的端點規格與憑證前，不得自行猜測端點或信封——
//!   因此生產模式回傳明確錯誤，而非向虛構端點發送請求。

use crate::bankpay::model::{
    EsunConfig, ExportRequest, ExportResponse, ImportRequest, ImportResponse, QueryRequest,
    QueryResponse,
};

/// 生產模式未啟用的統一說明（問題＋原因＋處理方式）。
const PRODUCTION_UNAVAILABLE: &str = "生產模式尚未啟用。問題：平台內建 E.Sun 客戶端僅含模擬器，未串接真實銀行通道；可能原因：E.Sun V2.8 契約需 mTLS 憑證與經驗證端點規格，權威實作位於 Wellfit/crates/wellfit_bankpay；處理方式：請保持模擬模式作業，或改用 wellfit bankpay CLI（export/import/query）執行真實支付，並由系統管理員完成憑證設定後再切換。";

/// E.Sun BankPay 客戶端。
///
/// 模擬模式下不持有任何網路資源（零 TLS 連線池、零每請求重建開銷）。
pub struct EsunClient {
    config: EsunConfig,
}

impl EsunClient {
    /// 建立客戶端（不進行任何 I/O，不會 panic）。
    pub fn new(config: EsunConfig) -> Self {
        Self { config }
    }

    /// 檢查是否為模擬模式。
    pub fn is_mock_mode(&self) -> bool {
        self.config.esun_mock_mode
    }

    /// 獲取 E.Sun 連線配置。
    pub fn config(&self) -> &EsunConfig {
        &self.config
    }

    /// 匯出付款批次至 E.Sun。
    pub async fn export(&self, req: &ExportRequest) -> Result<ExportResponse, String> {
        if !self.config.esun_mock_mode {
            return Err(PRODUCTION_UNAVAILABLE.to_string());
        }
        // 確定性模擬：esun_ref 由 batch_id 推導；同 batch_id 重放得到同參照，
        // 與資料庫唯一索引共同構成端到端冪等。
        Ok(ExportResponse {
            status: "SUBMITTED".to_string(),
            batch_id: req.batch_id.clone(),
            esun_ref: Some(format!("ESUN-MOCK-{}", req.batch_id)),
        })
    }

    /// 匯入銀行回應檔。
    ///
    /// 模擬模式仍**真實讀取並解析**檔案：以 CSV 資料列計算 matched/mismatched，
    /// 絕不回傳捏造的固定筆數。檔案讀取失敗會如實回報錯誤。
    pub async fn import(&self, req: &ImportRequest) -> Result<ImportResponse, String> {
        if !self.config.esun_mock_mode {
            return Err(PRODUCTION_UNAVAILABLE.to_string());
        }
        let content = tokio::fs::read_to_string(&req.file_path)
            .await
            .map_err(|e| {
                format!(
                    "檔案讀取失敗：{e}。問題：無法讀取「{}」；可能原因：檔案不存在、編碼非 UTF-8 或權限不足；處理方式：確認檔案已放置於匯入目錄且為 UTF-8 CSV 後重試。",
                    req.file_path
                )
            })?;

        let mut matched = 0usize;
        let mut mismatched = 0usize;
        // 格式：首行表頭；資料列至少需 4 欄且首欄（參照編號）非空。
        for (idx, line) in content.lines().enumerate() {
            if idx == 0 || line.trim().is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() >= 4 && !cols[0].trim().is_empty() {
                matched += 1;
            } else {
                mismatched += 1;
            }
        }

        Ok(ImportResponse {
            status: "processed".to_string(),
            matched,
            mismatched,
        })
    }

    /// 查詢銀行交易狀態。
    ///
    /// 模擬模式回傳 `COMPLETED` 與 `None` 金額——金額/幣別由 API 層自
    /// `bankpay_batches`（本地批次帳）補入，客戶端不虛構任何財務數值。
    pub async fn query(&self, _req: &QueryRequest) -> Result<QueryResponse, String> {
        if !self.config.esun_mock_mode {
            return Err(PRODUCTION_UNAVAILABLE.to_string());
        }
        Ok(QueryResponse {
            status: "COMPLETED".to_string(),
            amount_minor: None,
            currency: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_config() -> EsunConfig {
        EsunConfig::default()
    }

    #[test]
    fn test_mock_config_default() {
        let config = mock_config();
        assert!(config.esun_mock_mode);
        // 誠實預設：未驗證前不猜測銀行端點 URL。
        assert!(config.esun_url.is_empty());
    }

    #[test]
    fn test_client_creation_no_panic() {
        let client = EsunClient::new(mock_config());
        assert!(client.is_mock_mode());
    }

    #[test]
    fn test_production_mode_fails_closed() {
        let mut config = mock_config();
        config.esun_mock_mode = false;
        config.esun_url = "https://example.invalid".to_string();
        let client = EsunClient::new(config);
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let err = rt
            .block_on(client.export(&ExportRequest {
                batch_id: "B1".to_string(),
                amount_minor: 100,
                currency: "TWD".to_string(),
            }))
            .expect_err("production mode must fail closed");
        assert!(err.contains("生產模式尚未啟用"));
    }

    #[tokio::test]
    async fn test_mock_export_is_deterministic() {
        let client = EsunClient::new(mock_config());
        let req = ExportRequest {
            batch_id: "BATCH-001".to_string(),
            amount_minor: 50000,
            currency: "TWD".to_string(),
        };
        let r1 = client.export(&req).await.expect("export 1");
        let r2 = client.export(&req).await.expect("export 2");
        assert_eq!(r1.esun_ref, r2.esun_ref, "same batch must derive same ref");
        assert_eq!(r1.esun_ref.as_deref(), Some("ESUN-MOCK-BATCH-001"));
    }

    #[tokio::test]
    async fn test_mock_import_parses_real_file() {
        let dir = std::env::temp_dir().join("bankpay-client-test");
        tokio::fs::create_dir_all(&dir).await.expect("mkdir");
        let path = dir.join("responses.csv");
        tokio::fs::write(
            &path,
            "ref,status,amount,currency\nR1,OK,100,TWD\nR2,OK,200,TWD\nBAD\n",
        )
        .await
        .expect("write csv");
        let client = EsunClient::new(mock_config());
        let resp = client
            .import(&ImportRequest {
                file_path: path.to_string_lossy().to_string(),
            })
            .await
            .expect("import");
        assert_eq!(resp.matched, 2, "two valid data rows");
        assert_eq!(resp.mismatched, 1, "one malformed row");
    }

    #[tokio::test]
    async fn test_mock_import_missing_file_is_error() {
        let client = EsunClient::new(mock_config());
        let err = client
            .import(&ImportRequest {
                file_path: "Z:/definitely/not/here.csv".to_string(),
            })
            .await
            .expect_err("missing file must error");
        assert!(err.contains("檔案讀取失敗"));
    }
}
