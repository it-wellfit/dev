//! E.Sun BankPay API 端點處理函數。
//!
//! 提供匯出、匯入、查詢三大 RESTful 端點，並整合稽核軌跡。
//! 此模組僅包含處理函數，路由註冊在 api.rs 中統一管理。

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::auth::{self, Session};
use crate::bankpay::client::EsunClient;
use crate::bankpay::model::{
    ExportRequest, ExportResponse, ImportRequest, ImportResponse, QueryRequest, QueryResponse,
};
use crate::bankpay::state::{status_from_event, validate_transition, BankPayStatus};

/// 匯出付款批次。
pub async fn export_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, Response> {
    let session = match auth::current_session(&state, &headers).await {
        Some(s) => s,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "未授權：請先登入".to_string()).into_response())
        }
    };

    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_WRITE")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "無權限執行銀行支付匯出（需要 LEDGER_WRITE 權限）".to_string(),
        )
            .into_response());
    }

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.export(&req).await {
        Ok(resp) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_EXPORT_SUCCESS', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Ok(Json(resp))
        }
        Err(e) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_EXPORT_FAILURE', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(Some(format!("{}", e)))
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("銀行支付匯出失敗：{}", e),
            )
                .into_response())
        }
    }
}

/// 匯入銀行回應檔。
pub async fn import_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, Response> {
    let session = match auth::current_session(&state, &headers).await {
        Some(s) => s,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "未授權：請先登入".to_string()).into_response())
        }
    };

    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_WRITE")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "無權限執行銀行支付匯入（需要 LEDGER_WRITE 權限）".to_string(),
        )
            .into_response());
    }

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.import(&req).await {
        Ok(resp) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_IMPORT_SUCCESS', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(serde_json::to_string(&resp).unwrap_or_default())
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Ok(Json(resp))
        }
        Err(e) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_IMPORT_FAILURE', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(Some(format!("{}", e)))
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("銀行支付匯入失敗：{}", e),
            )
                .into_response())
        }
    }
}

/// 查詢銀行交易狀態。
pub async fn query_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(req): Query<QueryRequest>,
) -> Result<Json<QueryResponse>, Response> {
    let session = match auth::current_session(&state, &headers).await {
        Some(s) => s,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "未授權：請先登入".to_string()).into_response())
        }
    };

    if !state
        .permissions
        .has_permission_db(&session.user_id, "LEDGER_READ")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "無權限執行銀行支付查詢（需要 LEDGER_READ 權限）".to_string(),
        )
            .into_response());
    }

    let client = EsunClient::new(state.bankpay_config.lock().await.clone());
    match client.query(&req).await {
        Ok(resp) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_QUERY_SUCCESS', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(serde_json::to_string(&resp).unwrap_or_default())
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Ok(Json(resp))
        }
        Err(e) => {
            let _ = sqlx::query(
                "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
                 VALUES (?, NULL, 'BANKPAY_QUERY_FAILURE', ?, ?)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(serde_json::to_string(&req).unwrap_or_default())
            .bind(Some(format!("{}", e)))
            .bind(&session.user_id)
            .execute(&state.pool)
            .await;

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("銀行支付查詢失敗：{}", e),
            )
                .into_response())
        }
    }
}

/// 獲取銀行支付設定。
pub async fn config_get_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::bankpay::model::EsunConfig>, Response> {
    let session = match auth::current_session(&state, &headers).await {
        Some(s) => s,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "未授權：請先登入".to_string()).into_response())
        }
    };

    if !state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "無權限讀取銀行支付設定（需要 ROLE_MANAGE 權限）".to_string(),
        )
            .into_response());
    }

    Ok(Json(state.bankpay_config.lock().await.clone()))
}

/// 更新銀行支付設定。
#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    pub esun_mock_mode: Option<bool>,
    pub esun_url: Option<String>,
    pub source_system_id: Option<String>,
    pub customer_id: Option<String>,
    pub payer_account: Option<String>,
}

pub async fn config_post_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ConfigUpdateRequest>,
) -> Result<Json<crate::bankpay::model::EsunConfig>, Response> {
    let session = match auth::current_session(&state, &headers).await {
        Some(s) => s,
        None => {
            return Err((StatusCode::UNAUTHORIZED, "未授權：請先登入".to_string()).into_response())
        }
    };

    if !state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await
        .unwrap_or(false)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "無權限修改銀行支付設定（需要 ROLE_MANAGE 權限）".to_string(),
        )
            .into_response());
    }

    let mut config = state.bankpay_config.lock().await.clone();
    if let Some(v) = req.esun_mock_mode {
        config.esun_mock_mode = v;
    }
    if let Some(v) = req.esun_url {
        config.esun_url = v;
    }
    if let Some(v) = req.source_system_id {
        config.source_system_id = Some(v);
    }
    if let Some(v) = req.customer_id {
        config.customer_id = Some(v);
    }
    if let Some(v) = req.payer_account {
        config.payer_account = Some(v);
    }

    *state.bankpay_config.lock().await = config.clone();

    Ok(Json(config))
}

/// 銀行支付健康檢查。
#[derive(Debug, Serialize)]
pub struct BankPayHealthResponse {
    pub status: String,
    pub mock_mode: bool,
    pub configured: bool,
}

pub async fn bankpay_health_handler(
    State(state): State<AppState>,
) -> Result<Json<BankPayHealthResponse>, Response> {
    let config = state.bankpay_config.lock().await;
    Ok(Json(BankPayHealthResponse {
        status: "ok".to_string(),
        mock_mode: config.esun_mock_mode,
        configured: config.source_system_id.is_some() && config.customer_id.is_some(),
    }))
}

/// 狀態機轉換驗證端點（用於開發除錯）。
#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct TransitionResponse {
    pub valid: bool,
    pub message: Option<String>,
}

pub async fn transition_handler(
    Json(req): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, Response> {
    let from: BankPayStatus = match req.from.as_str() {
        "Pending" => BankPayStatus::Pending,
        "Submitted" => BankPayStatus::Submitted,
        "Completed" => BankPayStatus::Completed,
        "Failed" => BankPayStatus::Failed,
        "Rejected" => BankPayStatus::Rejected,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("無效的來源狀態：{}", req.from),
            )
                .into_response())
        }
    };

    let to: BankPayStatus = match req.to.as_str() {
        "Pending" => BankPayStatus::Pending,
        "Submitted" => BankPayStatus::Submitted,
        "Completed" => BankPayStatus::Completed,
        "Failed" => BankPayStatus::Failed,
        "Rejected" => BankPayStatus::Rejected,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("無效的目標狀態：{}", req.to),
            )
                .into_response())
        }
    };

    match validate_transition(&from, &to) {
        Ok(()) => Ok(Json(TransitionResponse {
            valid: true,
            message: Some("狀態轉換合法".to_string()),
        })),
        Err(msg) => Ok(Json(TransitionResponse {
            valid: false,
            message: Some(msg),
        })),
    }
}
