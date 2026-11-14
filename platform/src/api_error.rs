#![forbid(unsafe_code)]
//! G4.11 (D4-01/D4-04) — JSON error envelope and request-id propagation.

use axum::Json;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header::HeaderName};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request id inserted by [`request_id_middleware`] (extension) so handlers and
/// the auth middleware can echo it in error envelopes.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    /// 問題＋可能原因＋處理方式（HTML 頁同款三段式）。
    pub hint: String,
    pub retryable: bool,
    pub request_id: String,
}

/// Build the standard JSON error envelope.
pub fn api_error(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
    hint: impl Into<String>,
    retryable: bool,
    request_id: &str,
) -> Response {
    let body = ApiErrorBody {
        error: ApiErrorDetail {
            code: code.to_string(),
            message: message.into(),
            hint: hint.into(),
            retryable,
            request_id: request_id.to_string(),
        },
    };
    (status, Json(body)).into_response()
}

/// Read the request id from the request headers (middleware inserts the
/// extension before handlers run; this is the header fallback).
pub fn request_id_from_headers(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("unknown")
        .to_string()
}

/// Echo an incoming `X-Request-Id` (or generate one) and set it on the
/// response. Outermost middleware, so the extension is available to auth and
/// handlers.
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_shape() {
        let response = api_error(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "找不到文件",
            "問題：文件不存在；可能原因：已刪除；處理方式：重新索引。",
            false,
            "req-1",
        );
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = serde_json::to_value(ApiErrorBody {
            error: ApiErrorDetail {
                code: "NOT_FOUND".into(),
                message: "找不到文件".into(),
                hint: "問題：文件不存在；可能原因：已刪除；處理方式：重新索引。".into(),
                retryable: false,
                request_id: "req-1".into(),
            },
        })
        .expect("serialize");
        assert_eq!(body["error"]["code"], "NOT_FOUND");
        assert_eq!(body["error"]["request_id"], "req-1");
    }
}
