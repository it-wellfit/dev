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

use sqlx::SqlitePool;

use crate::bankpay::model::EsunConfig;

/// `erp_config` 中銀行支付設定的分類名稱（單一事實來源）。
const CONFIG_CATEGORY: &str = "bankpay";

/// 從 `erp_config` 載入銀行支付設定（啟動時呼叫）。
///
/// 缺鍵一律回退 `EsunConfig::default()` 的對應值（fail-closed：
/// 空值不偽裝成已設定，None 欄位以空字串儲存、載入時還原為 None）。
pub async fn load_config(pool: &SqlitePool) -> EsunConfig {
    let mut config = EsunConfig::default();
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT config_key, config_value FROM erp_config WHERE config_category = ?")
            .bind(CONFIG_CATEGORY)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    for (key, value) in rows {
        match key.as_str() {
            "esun_mock_mode" => config.esun_mock_mode = value == "1",
            "esun_url" => config.esun_url = value,
            "source_system_id" if !value.is_empty() => config.source_system_id = Some(value),
            "customer_id" if !value.is_empty() => config.customer_id = Some(value),
            "payer_account" if !value.is_empty() => config.payer_account = Some(value),
            "esun_external_url" if !value.is_empty() => config.esun_external_url = value,
            "client_cert_path" if !value.is_empty() => config.client_cert_path = Some(value),
            "client_key_path" if !value.is_empty() => config.client_key_path = Some(value),
            "ca_cert_path" if !value.is_empty() => config.ca_cert_path = Some(value),
            "timeout_secs" => {
                if let Ok(v) = value.parse::<u64>() {
                    config.timeout_secs = v;
                }
            }
            _ => {}
        }
    }
    config
}

/// 將銀行支付設定持久化至 `erp_config`（管理員 POST 設定時呼叫）。
///
/// 冪等性：`config_id` 以「分類:鍵」決定性建構，同鍵重存為 UPSERT
/// （不新增列）。schema 為 SSOT，本函式不變更 schema。
pub async fn save_config(pool: &SqlitePool, config: &EsunConfig) -> Result<(), sqlx::Error> {
    let pairs: [(&str, String); 10] = [
        (
            "esun_mock_mode",
            if config.esun_mock_mode { "1" } else { "0" }.to_string(),
        ),
        ("esun_url", config.esun_url.clone()),
        (
            "source_system_id",
            config.source_system_id.clone().unwrap_or_default(),
        ),
        (
            "customer_id",
            config.customer_id.clone().unwrap_or_default(),
        ),
        (
            "payer_account",
            config.payer_account.clone().unwrap_or_default(),
        ),
        ("esun_external_url", config.esun_external_url.clone()),
        (
            "client_cert_path",
            config.client_cert_path.clone().unwrap_or_default(),
        ),
        (
            "client_key_path",
            config.client_key_path.clone().unwrap_or_default(),
        ),
        (
            "ca_cert_path",
            config.ca_cert_path.clone().unwrap_or_default(),
        ),
        ("timeout_secs", config.timeout_secs.to_string()),
    ];
    for (key, value) in pairs {
        let config_id = format!("bankpay:{key}");
        sqlx::query(
            "INSERT INTO erp_config (config_id, config_category, config_key, config_value, config_type) \
             VALUES (?, ?, ?, ?, 'string') \
             ON CONFLICT(config_id) DO UPDATE SET \
             config_value = excluded.config_value, \
             config_category = excluded.config_category, \
             config_key = excluded.config_key, \
             updated_at = CURRENT_TIMESTAMP",
        )
        .bind(&config_id)
        .bind(CONFIG_CATEGORY)
        .bind(key)
        .bind(&value)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn migrated_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory pool");
        crate::db::DatabaseManager::new(pool.clone())
            .initialize()
            .await
            .expect("migrations");
        pool
    }

    fn sample_config() -> EsunConfig {
        EsunConfig {
            esun_mock_mode: false,
            esun_url: "https://esun.example.test/bankpay".to_string(),
            source_system_id: Some("SRC-ERP".to_string()),
            customer_id: Some("CUST-001".to_string()),
            payer_account: Some("012345678901".to_string()),
            esun_external_url: "https://eospt.testesunbank.com.tw:9443".to_string(),
            client_cert_path: Some("certs/client.crt".to_string()),
            client_key_path: Some("certs/client.key".to_string()),
            ca_cert_path: Some("certs/esun-ca.crt".to_string()),
            timeout_secs: 30,
        }
    }

    #[tokio::test]
    async fn config_roundtrips_through_erp_config() {
        let pool = migrated_pool().await;
        save_config(&pool, &sample_config()).await.expect("save");
        let loaded = load_config(&pool).await;
        assert!(!loaded.esun_mock_mode);
        assert_eq!(loaded.esun_url, "https://esun.example.test/bankpay");
        assert_eq!(loaded.source_system_id.as_deref(), Some("SRC-ERP"));
        assert_eq!(loaded.customer_id.as_deref(), Some("CUST-001"));
        assert_eq!(loaded.payer_account.as_deref(), Some("012345678901"));
    }

    #[tokio::test]
    async fn save_config_is_idempotent_upsert() {
        let pool = migrated_pool().await;
        save_config(&pool, &sample_config()).await.expect("first");
        let mut updated = sample_config();
        updated.esun_mock_mode = true;
        updated.esun_url = "https://esun.example.test/v2".to_string();
        save_config(&pool, &updated).await.expect("second");
        let loaded = load_config(&pool).await;
        assert!(loaded.esun_mock_mode);
        assert_eq!(loaded.esun_url, "https://esun.example.test/v2");
        let rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM erp_config WHERE config_category = 'bankpay'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(rows, 10, "UPSERT must not duplicate rows (10 config keys)");
    }

    #[tokio::test]
    async fn app_state_loads_persisted_config_at_startup() {
        let pool = migrated_pool().await;
        save_config(&pool, &sample_config()).await.expect("save");
        let state = crate::api::AppState::new(pool.clone())
            .await
            .expect("app state");
        let loaded = state.bankpay_config.lock().await.clone();
        assert_eq!(loaded.esun_url, "https://esun.example.test/bankpay");
        assert_eq!(loaded.source_system_id.as_deref(), Some("SRC-ERP"));
    }
}
