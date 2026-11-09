//! 系統狀態監控模組（Slice 13）。
//!
//! 提供 `/api/status` 端點與可重用的即時探測（`probe_modules`），供全域
//! 狀態列（index 頁 server-render）與 JSON API 共用同一事實來源。
//!
//! 設計原則（第一性原理：derive，勿快取）：
//! - 狀態為**每次請求即時探測**的衍生值，不保存可漂移的快取——狀態
//!   永不過期、永不殘留 `Unknown`、無需鎖與共享可變狀態。
//! - **誠實標示**：未接入真實後端的模組（sync/com/ai）標為 `Idle` 並
//!   說明原因，絕不以合成數值偽裝可用（治理 P7）。
//! - 整體狀態：任一 `Error` → `error`；任一 `Warning` → `degraded`；
//!   其餘（含 `Idle`）→ `healthy`。

use std::collections::HashMap;

use axum::extract::State;
use axum::response::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api::AppState;

/// 模組狀態。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    /// 正常運作。
    Ok,
    /// 警告（可用但需要注意：模擬模式、設定不完整等）。
    Warning,
    /// 錯誤（不可用）。
    Error,
    /// 閒置（模組存在但後端未接入／無內容）。
    Idle,
}

impl std::fmt::Display for ModuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleStatus::Ok => write!(f, "ok"),
            ModuleStatus::Warning => write!(f, "warning"),
            ModuleStatus::Error => write!(f, "error"),
            ModuleStatus::Idle => write!(f, "idle"),
        }
    }
}

/// 狀態的繁體中文標籤（UI 顯示用；`Display` 保留為機器碼/JSON 值）。
pub fn zh_label(status: &ModuleStatus) -> &'static str {
    match status {
        ModuleStatus::Ok => "正常",
        ModuleStatus::Warning => "注意",
        ModuleStatus::Error => "異常",
        ModuleStatus::Idle => "閒置",
    }
}

/// 單一模組狀態資訊。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatusInfo {
    pub status: ModuleStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
    pub last_checked: String,
}

/// 系統狀態回應。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    /// "healthy" | "degraded" | "error"
    pub status: String,
    pub modules: HashMap<String, ModuleStatusInfo>,
    pub timestamp: String,
    pub version: String,
}

fn now_stamp() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn info(status: ModuleStatus, message: Option<String>, ms: Option<u64>) -> ModuleStatusInfo {
    ModuleStatusInfo {
        status,
        message,
        response_time_ms: ms,
        last_checked: now_stamp(),
    }
}

/// 即時探測全部模組（`/api/status` 與 index 狀態列共用）。
pub async fn probe_modules(state: &AppState) -> HashMap<String, ModuleStatusInfo> {
    let mut modules = HashMap::new();

    // database：真實 I/O 探測（SELECT 1）＋回應時間。
    let start = std::time::Instant::now();
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.pool).await.is_ok();
    let db_ms = start.elapsed().as_millis() as u64;
    modules.insert(
        "database".to_string(),
        if db_ok {
            info(ModuleStatus::Ok, None, Some(db_ms))
        } else {
            info(
                ModuleStatus::Error,
                Some(
                    "資料庫連線失敗。處理建議：確認 SQLite 資料庫檔案可寫入，或聯絡系統管理員。"
                        .to_string(),
                ),
                Some(db_ms),
            )
        },
    );

    // search：in-process 索引計數（無 I/O 失敗模式）。
    let count = state.search.document_count().await;
    modules.insert(
        "search".to_string(),
        if count > 0 {
            info(ModuleStatus::Ok, None, None)
        } else {
            info(
                ModuleStatus::Idle,
                Some("全文索引尚無文件（可於匯入或建立交易後建立索引）。".to_string()),
                None,
            )
        },
    );

    // sync：任務佇列引擎未於 Web 層組裝（誠實標示，不虛構 pending 數）。
    modules.insert(
        "sync".to_string(),
        info(
            ModuleStatus::Idle,
            Some(
                "同步引擎未於 Web 層啟用（sync::SyncEngine 待接入）；目前無排程中之同步任務。"
                    .to_string(),
            ),
            None,
        ),
    );

    // com：ComBridge::invoke 為佔位實作；真實通道位於 flaps_bridge（real-com 門控）。
    modules.insert(
        "com".to_string(),
        info(
            ModuleStatus::Idle,
            Some("COM 橋接為佔位實作，未連接真實 COM 物件；真實通道請使用 flaps_bridge（real-com feature）。".to_string()),
            None,
        ),
    );

    // ai：推論端點未設定（ai 模組客戶端待接設定來源）。
    modules.insert(
        "ai".to_string(),
        info(
            ModuleStatus::Idle,
            Some(
                "AI 推論端點未設定（ai 模組待接入 ollama/chutes 設定）；AI 建議功能停用中。"
                    .to_string(),
            ),
            None,
        ),
    );

    // bankpay：模擬模式為 Warning（資料非真實銀行交易，必須讓使用者看見）。
    let config = state.bankpay_config.lock().await;
    let bankpay = if config.esun_mock_mode {
        info(
            ModuleStatus::Warning,
            Some("模擬模式（Mock）：回應為確定性模擬，非真實銀行交易。".to_string()),
            None,
        )
    } else if config.source_system_id.is_some() && config.customer_id.is_some() {
        info(ModuleStatus::Ok, None, None)
    } else {
        info(
            ModuleStatus::Warning,
            Some(
                "生產模式但設定不完整（缺 source_system_id 或 customer_id）；請於設定頁補齊。"
                    .to_string(),
            ),
            None,
        )
    };
    modules.insert("bankpay".to_string(), bankpay);

    modules
}

/// 依模組狀態計算整體狀態。
pub fn overall_status(modules: &HashMap<String, ModuleStatusInfo>) -> String {
    let mut has_error = false;
    let mut has_warning = false;
    for m in modules.values() {
        match m.status {
            ModuleStatus::Error => has_error = true,
            ModuleStatus::Warning => has_warning = true,
            ModuleStatus::Ok | ModuleStatus::Idle => {}
        }
    }
    if has_error {
        "error".to_string()
    } else if has_warning {
        "degraded".to_string()
    } else {
        "healthy".to_string()
    }
}

/// GET /api/status — 系統狀態檢查端點（永遠 200；狀態本身在 body 中表達）。
pub async fn status_handler(State(state): State<AppState>) -> Json<SystemStatusResponse> {
    let modules = probe_modules(&state).await;
    Json(SystemStatusResponse {
        status: overall_status(&modules),
        timestamp: now_stamp(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        modules,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(status: ModuleStatus) -> ModuleStatusInfo {
        info(status, None, None)
    }

    #[test]
    fn test_overall_status_rules() {
        let mut m = HashMap::new();
        m.insert("database".to_string(), module(ModuleStatus::Ok));
        m.insert("search".to_string(), module(ModuleStatus::Idle));
        m.insert("bankpay".to_string(), module(ModuleStatus::Idle));
        assert_eq!(overall_status(&m), "healthy", "Idle 不應降級整體狀態");

        m.insert("bankpay".to_string(), module(ModuleStatus::Warning));
        assert_eq!(overall_status(&m), "degraded");

        m.insert("bankpay".to_string(), module(ModuleStatus::Error));
        assert_eq!(overall_status(&m), "error");
    }

    #[test]
    fn test_module_status_display() {
        assert_eq!(ModuleStatus::Ok.to_string(), "ok");
        assert_eq!(ModuleStatus::Error.to_string(), "error");
        assert_eq!(ModuleStatus::Warning.to_string(), "warning");
        assert_eq!(ModuleStatus::Idle.to_string(), "idle");
    }

    #[test]
    fn test_probe_marks_every_module_never_unknown() {
        // probe_modules 需要 AppState；此處僅驗證 info() 一致性。
        let i = info(ModuleStatus::Idle, Some("x".to_string()), None);
        assert_eq!(i.status, ModuleStatus::Idle);
        assert!(!i.last_checked.is_empty());
    }
}
