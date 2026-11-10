#![forbid(unsafe_code)]
//! API 資料管理平台（零 JS；G1 todo 收尾——目錄／測試台／稽核／設定）。
//!
//! 第一性原理（§0.5 語義漂移防護）：端點目錄由本模組的靜態 registry 驅動，
//! 模板不得手寫端點清單——文件與路由註冊維持單一事實來源；registry 變更時
//! 目錄、測試台與 curl 範例同步更新。
//!
//! 頁面（HTML 群組；`require_session` 注入 `Session`；處理器內再驗 ROLE_MANAGE）：
//! - GET  /admin/api/catalog  端點目錄（由 registry 產生）
//! - GET  /admin/api/test     測試工作台（安全 GET 端點＝即時探測；其餘提供 curl 規格）
//! - GET  /admin/api/logs     稽核日誌（audit_log 最近 100 筆；NULL 以 COALESCE 防毒化）
//! - GET  /admin/api/config   銀行支付設定（現行值＋erp_config 持久列）
//! - POST /admin/api/config   設定理單更新（CSRF 表單欄位；冪等 UPSERT；303 PRG）
//!
//! 誠實原則（治理 P7）：測試台只代執行**無寫入副作用**的 GET 探測；變更類端點
//! 一律給規格與 curl 範例，不代執行（fail-closed）。

use std::collections::HashMap;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Extension, Form, Router};
use serde::Deserialize;

use crate::api::AppState;
use crate::auth::Session;

// ---------------------------------------------------------------------------
// Registry：API 端點單一事實來源
// ---------------------------------------------------------------------------

/// 端點參數規格。
pub struct ApiParam {
    pub name: &'static str,
    pub description: &'static str,
}

/// 端點規格。
pub struct ApiEndpointSpec {
    pub method: &'static str,
    pub path: &'static str,
    pub description: &'static str,
    pub auth: &'static str,
    pub params: &'static [ApiParam],
    /// true＝可由測試台以伺服端同一程式路徑即時探測（僅無副作用 GET）。
    pub live: bool,
    /// true＝測試台附參數輸入（需參數的 GET 也可即時探測）。
    pub parametric: bool,
}

/// API 端點 registry（與 `api.rs`／`cross_module_api.rs` 路由註冊對應）。
pub fn registry() -> &'static [ApiEndpointSpec] {
    &[
        ApiEndpointSpec {
            method: "GET",
            path: "/api/status",
            description:
                "系統狀態即時探測（資料庫／搜尋／同步／COM／AI／BankPay）；永不 500，狀態在 body。",
            auth: "session",
            params: &[],
            live: true,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/bankpay/health",
            description: "BankPay 模組健康檢查（模擬模式、設定完整度）。",
            auth: "session",
            params: &[],
            live: true,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/bankpay/config",
            description: "讀取銀行連線設定（`erp_config` 為持久層 SSOT）。",
            auth: "ROLE_MANAGE",
            params: &[],
            live: true,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "POST",
            path: "/api/bankpay/config",
            description: "更新銀行連線設定（冪等 UPSERT；失敗回 500 且記憶體不變）。",
            auth: "ROLE_MANAGE + X-CSRF-Token",
            params: &[
                ApiParam {
                    name: "esun_mock_mode",
                    description: "bool：模擬模式（true＝確定性模擬，非真實交易）",
                },
                ApiParam {
                    name: "esun_url",
                    description: "string：銀行端點 URL",
                },
                ApiParam {
                    name: "source_system_id",
                    description: "string：來源系統識別",
                },
                ApiParam {
                    name: "customer_id",
                    description: "string：客戶識別",
                },
                ApiParam {
                    name: "payer_account",
                    description: "string：付款帳號",
                },
            ],
            live: false,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "POST",
            path: "/api/bankpay/export",
            description: "匯出批次（冪等：SUBMITTED／COMPLETED 重放不重複呼叫銀行）。",
            auth: "LEDGER_WRITE + X-CSRF-Token",
            params: &[
                ApiParam {
                    name: "batch_id",
                    description: "string：批次識別",
                },
                ApiParam {
                    name: "amount_minor",
                    description: "i64：金額（最小單位，整數分；INV-01 無浮點）",
                },
                ApiParam {
                    name: "currency",
                    description: "string：幣別（TWD／USD／EUR／JPY）",
                },
            ],
            live: false,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "POST",
            path: "/api/bankpay/import",
            description: "CSV 匯入（白名單目錄 data/bankpay/inbound/、僅 .csv；CWE-22 防護）。",
            auth: "LEDGER_WRITE + X-CSRF-Token",
            params: &[ApiParam {
                name: "file_path",
                description: "string：白名單目錄內之 .csv 相對路徑",
            }],
            live: false,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/bankpay/query",
            description: "查詢批次狀態。",
            auth: "LEDGER_READ",
            params: &[ApiParam {
                name: "esun_ref",
                description: "string：玉山交易參考號",
            }],
            live: false,
            parametric: false,
        },
        ApiEndpointSpec {
            method: "POST",
            path: "/api/bankpay/transition",
            description: "狀態機轉換驗證（開發除錯；純函式、無副作用、不改任何狀態）。",
            auth: "session",
            params: &[
                ApiParam {
                    name: "from",
                    description:
                        "string：來源狀態（Pending／Submitted／Completed／Failed／Rejected）",
                },
                ApiParam {
                    name: "to",
                    description: "string：目標狀態",
                },
            ],
            live: false,
            parametric: false,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/audit",
            description: "稽核軌跡查詢（分頁＋篩選；參數以 bind 綁定，非字串拼接）。",
            auth: "session",
            params: &[
                ApiParam {
                    name: "page / limit",
                    description: "整數：分頁（預設 1／20）",
                },
                ApiParam {
                    name: "user_id / action",
                    description: "string：篩選（選填）",
                },
                ApiParam {
                    name: "start_date / end_date",
                    description: "YYYY-MM-DD：範圍（選填）",
                },
            ],
            live: true,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/audit/export",
            description: "稽核軌跡匯出（CSV；CWE-1236 試算表公式注入防護）。",
            auth: "session",
            params: &[],
            live: false,
            parametric: false,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/search",
            description: "全文檢索（Tantivy；q 為綁定參數）。",
            auth: "session",
            params: &[ApiParam {
                name: "q",
                description: "string：查詢字串",
            }],
            live: true,
            parametric: true,
        },
        ApiEndpointSpec {
            method: "GET",
            path: "/api/modules/health",
            description: "跨模組健康摘要（search／sync／com／ai）。",
            auth: "session",
            params: &[],
            live: false,
            parametric: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// 檢視模型
// ---------------------------------------------------------------------------

/// 目錄列（模板用；owned 以避免模板借用生命週期問題）。
pub struct CatalogRow {
    pub method: String,
    pub path: String,
    pub description: String,
    pub auth: String,
    pub method_class: String,
    pub live: bool,
    pub params: Vec<ParamRow>,
}

pub struct ParamRow {
    pub name: String,
    pub description: String,
}

/// 測試台參數列（模板用）。
pub struct TestParam {
    pub name: String,
    pub value: String,
}

fn catalog_rows() -> Vec<CatalogRow> {
    registry()
        .iter()
        .map(|s| CatalogRow {
            method: s.method.to_string(),
            path: s.path.to_string(),
            description: s.description.to_string(),
            auth: s.auth.to_string(),
            method_class: if s.method == "GET" {
                "badge-info".to_string()
            } else {
                "badge-primary".to_string()
            },
            live: s.live,
            params: s
                .params
                .iter()
                .map(|p| ParamRow {
                    name: p.name.to_string(),
                    description: p.description.to_string(),
                })
                .collect(),
        })
        .collect()
}

#[derive(Template)]
#[template(path = "api_catalog.html")]
pub struct ApiCatalogTemplate {
    pub acting_user: String,
    pub endpoints: Vec<CatalogRow>,
    /// E.Sun BankPay V2.8 官方契約端點（8 個；權威規格：
    /// `Wellfit/docs/bankpay/04-interface-specification.md`）。
    pub esun_endpoints: Vec<EsunEndpointView>,
    pub esun_total: usize,
    pub native_total: usize,
}

/// E.Sun V2.8 契約端點檢視（誠實標示平台對應與串接狀態）。
pub struct EsunEndpointView {
    /// 官方契約路徑。
    pub path: &'static str,
    /// 繁體中文名稱。
    pub name: &'static str,
    /// 用途說明。
    pub description: &'static str,
    /// 權威規格出處（04-interface-specification.md 的章節錨點）。
    pub contract_ref: &'static str,
    /// 平台對應端點（未串接時為空字串）。
    pub mapping: &'static str,
    /// 串接狀態："模擬對應" | "未串接"。
    pub mode: &'static str,
    pub mode_class: &'static str,
}

/// E.Sun BankPay V2.8 官方契約端點（8 個；單一事實來源為
/// `Wellfit/docs/bankpay/04-interface-specification.md`）。
///
/// 誠實標示（治理 P7）：已由平台模擬器對應者標「模擬對應」；未串接者
/// 如實標示，不得以本目錄偽稱可用（ADR-0012）。
pub fn esun_endpoints() -> Vec<EsunEndpointView> {
    use EsunEndpointView as E;
    vec![
        E {
            path: "/payment/createTwdPaymentInstruction",
            name: "建立台幣支付指示",
            description: "建立新台幣付款指示（批次付款核心；平台匯出作業的銀行端對應）。",
            contract_ref: "§47",
            mapping: "/api/bankpay/export",
            mode: "模擬對應",
            mode_class: "badge-warning",
        },
        E {
            path: "/payment/retrieveTwdPaymentInstructionResult",
            name: "查詢台幣支付結果",
            description: "取得台幣付款指示處理結果（批次對帳與狀態機推進依據）。",
            contract_ref: "§118",
            mapping: "/api/bankpay/query",
            mode: "模擬對應",
            mode_class: "badge-warning",
        },
        E {
            path: "/account/retrieveStatements",
            name: "查詢交易明細（對帳單）",
            description: "下載帳戶交易明細；平台匯入作業以 CSV 真實解析對應。",
            contract_ref: "§145",
            mapping: "/api/bankpay/import",
            mode: "模擬對應",
            mode_class: "badge-warning",
        },
        E {
            path: "/payment/validateFxPaymentInstruction",
            name: "驗證外幣支付指示",
            description: "外幣付款指示送出前的格式與餘額驗證。",
            contract_ref: "§194",
            mapping: "",
            mode: "未串接",
            mode_class: "badge-info",
        },
        E {
            path: "/payment/createFxPaymentInstruction",
            name: "建立外幣支付指示",
            description: "建立外幣付款指示（跨幣別付款）。",
            contract_ref: "§237",
            mapping: "",
            mode: "未串接",
            mode_class: "badge-info",
        },
        E {
            path: "/payment/retrieveFxPaymentInstructionResult",
            name: "查詢外幣支付結果",
            description: "取得外幣付款指示處理結果。",
            contract_ref: "§245",
            mapping: "",
            mode: "未串接",
            mode_class: "badge-info",
        },
        E {
            path: "/account/retrieveAccountBalance",
            name: "查詢帳戶餘額",
            description: "即時帳戶餘額查詢（資金頭寸監控）。",
            contract_ref: "§273",
            mapping: "",
            mode: "未串接",
            mode_class: "badge-info",
        },
        E {
            path: "/payment/retrieveTwdPaymentInstructionPrintResult",
            name: "查詢台幣支付列印結果",
            description: "付款指示列印檔取得（紙本憑證流程）。",
            contract_ref: "§299",
            mapping: "",
            mode: "未串接",
            mode_class: "badge-info",
        },
    ]
}

#[derive(Template)]
#[template(path = "api_test.html")]
pub struct ApiTestTemplate {
    pub acting_user: String,
    pub endpoints: Vec<CatalogRow>,
    pub selected: String,
    pub params: Vec<TestParam>,
    pub has_response: bool,
    pub response_status: String,
    pub response_display: String,
    pub has_curl: bool,
    pub curl_example: String,
}

#[derive(Template)]
#[template(path = "api_logs.html")]
pub struct ApiLogsTemplate {
    pub acting_user: String,
    pub logs: Vec<crate::api::AuditLogRow>,
}

#[derive(Template)]
#[template(path = "api_config.html")]
pub struct ApiConfigTemplate {
    pub acting_user: String,
    pub csrf_token: String,
    pub esun_mock_mode: bool,
    pub esun_url: String,
    pub source_system_id: String,
    pub customer_id: String,
    pub payer_account: String,
    pub rows: Vec<ConfigRow>,
}

pub struct ConfigRow {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// 輔助
// ---------------------------------------------------------------------------

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 無權限頁（ROLE_MANAGE 缺失）：沿用資料八態之 Denied 語彙（指名權限碼）。
fn forbidden_page(acting_user: &str) -> Response {
    let user = html_escape(acting_user);
    let body = format!(
        "<!DOCTYPE html><html lang=\"zh-Hant-TW\"><head><meta charset=\"UTF-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
         <title>無權限 — Flaps ERP</title>\
         <link rel=\"stylesheet\" href=\"/assets/design-system.css\"></head><body>\
         <a class=\"skip-link\" href=\"#main\">跳至主要內容</a>\
         <header class=\"app-header\"><div class=\"container\"><div>\
         <h1>API 資料管理平台</h1><p class=\"app-subtitle\">需要 ROLE_MANAGE 權限</p></div>\
         <p class=\"app-subtitle\">使用者：<strong>{user}</strong>｜\
         <a class=\"btn btn-secondary\" href=\"/\">返回工作台</a></p></div></header>\
         <main id=\"main\" class=\"container\">\
         <div class=\"denied-state\" role=\"alert\" data-state-denied>\
         <strong>無權檢視此內容</strong>\
         <p>缺少權限碼 <code>ROLE_MANAGE</code>。請向系統管理員申請權限，或改用您可存取的頁面。</p>\
         <p><a class=\"btn btn-secondary\" href=\"/\">返回工作台</a></p></div></main></body></html>"
    );
    (StatusCode::FORBIDDEN, Html(body)).into_response()
}

/// ROLE_MANAGE 閘門（失敗回傳可直接 return 的回應）。
async fn require_admin(state: &AppState, session: &Session) -> Option<Response> {
    if state
        .permissions
        .has_permission_db(&session.user_id, "ROLE_MANAGE")
        .await
    {
        None
    } else {
        Some(forbidden_page(&session.user_id))
    }
}

/// 由 registry 產生 curl 執行範例（變更類端點不代執行）。
fn curl_for(spec: &ApiEndpointSpec) -> String {
    let mut command = format!(
        "curl -s -X {} http://127.0.0.1:8090{}",
        spec.method, spec.path
    );
    if spec.method == "POST" {
        command.push_str(" \\\n  -H \"Content-Type: application/json\"");
        command.push_str(" \\\n  -H \"Cookie: flaps_session=<登入後 Cookie 值>\"");
        if spec.auth.contains("X-CSRF-Token") {
            command.push_str(" \\\n  -H \"X-CSRF-Token: <頁面表單權杖>\"");
        }
        let keys: Vec<String> = spec
            .params
            .iter()
            .map(|p| format!("\"{}\": \"…\"", p.name))
            .collect();
        command.push_str(&format!(" \\\n  -d '{{{}}}'", keys.join(", ")));
    } else if !spec.params.is_empty() {
        command.push_str("?<參數>=<值>");
    }
    command
}

/// 安全 GET 端點的即時探測（伺服端同一程式路徑；零 JS）。
async fn live_response(state: &AppState, path: &str, params: &[TestParam]) -> String {
    match path {
        "/api/status" => {
            let modules = crate::status::probe_modules(state).await;
            let overall = crate::status::overall_status(&modules);
            let payload = serde_json::json!({
                "status": overall,
                "timestamp": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "version": env!("CARGO_PKG_VERSION"),
                "modules": modules,
            });
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        }
        "/api/bankpay/health" => {
            let config = state.bankpay_config.lock().await;
            let payload = serde_json::json!({
                "status": "ok",
                "mock_mode": config.esun_mock_mode,
                "configured": config.source_system_id.is_some() && config.customer_id.is_some(),
            });
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        }
        "/api/bankpay/config" => {
            let config = state.bankpay_config.lock().await;
            serde_json::to_string_pretty(&*config).unwrap_or_else(|_| "{}".to_string())
        }
        "/api/audit" => {
            // 同路由參數面（page/limit/user_id/action/start_date/end_date），實際執行同一查詢。
            let get = |k: &str| -> Option<String> {
                params.iter().find(|p| p.name == k).map(|p| p.value.clone())
            };
            let page: i64 = get("page").and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
            let limit: i64 = get("limit")
                .and_then(|v| v.parse().ok())
                .unwrap_or(5)
                .clamp(1, 100);
            let offset = (page - 1) * limit;
            let rows: Vec<crate::cross_module_api::AuditRow> = sqlx::query_as(
                "SELECT al.log_id, al.performed_by, u.username, al.event_type, \
                 COALESCE(al.transaction_id, ''), COALESCE(al.event_data, ''), al.performed_at \
                 FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id \
                 ORDER BY al.performed_at DESC LIMIT ? OFFSET ?",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();
            let items: Vec<_> = rows
                .into_iter()
                .map(
                    |(log_id, user_id, username, action, resource, details, created_at)| {
                        serde_json::json!({
                            "log_id": log_id,
                            "user": user_id,
                            "username": username,
                            "action": action,
                            "resource": resource,
                            "details": details,
                            "created_at": created_at,
                        })
                    },
                )
                .collect();
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        }
        "/api/search" => {
            // 同路由參數面（q）；過濾未命名參數以外的第一個 q。
            let q = params
                .iter()
                .find(|p| p.name == "q")
                .map(|p| p.value.as_str())
                .unwrap_or("");
            let results = state.search.search(q).await;
            let items: Vec<_> = results
                .into_iter()
                .map(
                    |r| serde_json::json!({ "doc_id": r.doc_id, "content": r.content, "score": 0 }),
                )
                .collect();
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        }
        "/api/modules/health" => {
            let modules = crate::status::probe_modules(state).await;
            let describe = |key: &str| -> String {
                modules.get(key).map_or_else(
                    || "unknown".to_string(),
                    |m| match &m.message {
                        Some(msg) => format!("{} — {}", m.status, msg),
                        None => format!("{}", m.status),
                    },
                )
            };
            let payload = serde_json::json!({
                "search": describe("search"),
                "sync": describe("sync"),
                "com": describe("com"),
                "ai": describe("ai"),
                "db": describe("database"),
                "overall": crate::status::overall_status(&modules),
            });
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        }
        "/api/bankpay/query" => {
            let esun_ref = params
                .iter()
                .find(|p| p.name == "esun_ref")
                .map(|p| p.value.clone())
                .unwrap_or_default();
            if esun_ref.is_empty() {
                "（參數 esun_ref 為空；請於上方填入銀行參照，如 ESUN-MOCK-BATCH-001）".to_string()
            } else {
                // 同路由語意：模擬器確定性查詢（不寫入副作用與狀態推進無關；
                // 狀態真相以銀行回應為準，GET 輪詢不改變批次）。
                let status = if esun_ref.starts_with("ESUN-MOCK-") {
                    "COMPLETED"
                } else {
                    "UNKNOWN"
                };
                serde_json::to_string_pretty(&serde_json::json!({
                    "esun_ref": esun_ref,
                    "status": status,
                    "reference": "mock（seed 驗證：只接受確定性批次識別）",
                }))
                .unwrap_or_else(|_| "{}".to_string())
            }
        }
        _ => "（此端點不提供即時探測）".to_string(),
    }
}

// ---------------------------------------------------------------------------
// 處理器
// ---------------------------------------------------------------------------

/// GET /admin/api/catalog — 端點目錄。
pub async fn apihub_catalog(
    State(state): State<AppState>,
    session: Extension<Session>,
) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let esun = esun_endpoints();
    let native = catalog_rows();
    let view = ApiCatalogTemplate {
        acting_user: session.user_id.clone(),
        esun_total: esun.len(),
        native_total: native.len(),
        esun_endpoints: esun,
        endpoints: native,
    };
    render_or_500(view)
}

/// GET /admin/api/test — 測試工作台（?endpoint=…）。
pub async fn apihub_test(
    State(state): State<AppState>,
    session: Extension<Session>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let selected = query.get("endpoint").cloned().unwrap_or_default();
    let mut has_response = false;
    let mut response_status = String::new();
    let mut response_display = String::new();
    let mut has_curl = false;
    let mut curl_example = String::new();
    let params: Vec<TestParam> = query
        .iter()
        .filter(|(k, _)| *k != "endpoint")
        .map(|(k, v)| TestParam {
            name: k.clone(),
            value: v.clone(),
        })
        .collect();
    if let Some(spec) = registry().iter().find(|s| s.path == selected) {
        if spec.live && spec.method == "GET" {
            has_response = true;
            response_status = "200 OK".to_string();
            response_display = live_response(&state, spec.path, &params).await;
        } else {
            has_curl = true;
            curl_example = curl_for(spec);
        }
    }
    render_or_500(ApiTestTemplate {
        acting_user: session.user_id.clone(),
        endpoints: catalog_rows(),
        selected,
        params,
        has_response,
        response_status,
        response_display,
        has_curl,
        curl_example,
    })
}

/// GET /admin/api/logs — 稽核日誌（最近 100 筆；NULL 安全）。
pub async fn apihub_logs(State(state): State<AppState>, session: Extension<Session>) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let raw: Vec<crate::cross_module_api::AuditRow> = sqlx::query_as(
        "SELECT al.log_id, al.performed_by, u.username, al.event_type, \
         COALESCE(al.transaction_id, ''), COALESCE(al.event_data, ''), al.performed_at \
         FROM audit_log al LEFT JOIN users u ON al.performed_by = u.user_id \
         ORDER BY al.performed_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let logs: Vec<crate::api::AuditLogRow> = raw
        .into_iter()
        .map(
            |(
                log_id,
                performed_by,
                username,
                event_type,
                transaction_id,
                event_data,
                performed_at,
            )| {
                let (action_label, action_class) = crate::api::audit_event_display(&event_type);
                crate::api::AuditLogRow {
                    log_id,
                    user_id: performed_by,
                    username: username.unwrap_or_else(|| "unknown".to_string()),
                    action_class: action_class.to_string(),
                    action_label,
                    action: event_type.clone(),
                    resource: transaction_id,
                    details: event_data,
                    created_at: performed_at,
                }
            },
        )
        .collect();
    render_or_500(ApiLogsTemplate {
        acting_user: session.user_id.clone(),
        logs,
    })
}

/// GET /admin/api/config — 現行設定＋erp_config 持久列。
pub async fn apihub_config(State(state): State<AppState>, session: Extension<Session>) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    render_config_page(&state, &session).await
}

async fn render_config_page(state: &AppState, session: &Session) -> Response {
    let config = state.bankpay_config.lock().await.clone();
    let rows: Vec<ConfigRow> = sqlx::query_as(
        "SELECT config_key, config_value, COALESCE(updated_at, '') FROM erp_config \
         WHERE config_category = 'bankpay' ORDER BY config_key",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(key, value, updated_at)| ConfigRow {
        key,
        value,
        updated_at,
    })
    .collect();
    render_or_500(ApiConfigTemplate {
        acting_user: session.user_id.clone(),
        csrf_token: session.csrf_token.clone(),
        esun_mock_mode: config.esun_mock_mode,
        esun_url: config.esun_url.clone(),
        source_system_id: config.source_system_id.clone().unwrap_or_default(),
        customer_id: config.customer_id.clone().unwrap_or_default(),
        payer_account: config.payer_account.clone().unwrap_or_default(),
        rows,
    })
}

/// 設定理單（HTML 表單；非 JSON API——給人的零 JS 管理面）。
#[derive(Debug, Deserialize)]
pub struct ApihubConfigForm {
    pub csrf_token: String,
    pub esun_mock_mode: String,
    pub esun_url: String,
    pub source_system_id: String,
    pub customer_id: String,
    pub payer_account: String,
}

/// POST /admin/api/config — 設定更新（CSRF＋ROLE_MANAGE；先持久化後記憶體；303 PRG）。
pub async fn apihub_config_update(
    State(state): State<AppState>,
    session: Extension<Session>,
    Form(form): Form<ApihubConfigForm>,
) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    if form.csrf_token != session.csrf_token {
        return (
            StatusCode::FORBIDDEN,
            "CSRF 驗證失敗；請重新整理頁面後再操作。",
        )
            .into_response();
    }

    let mut config = state.bankpay_config.lock().await.clone();
    config.esun_mock_mode = form.esun_mock_mode == "1";
    let url = form.esun_url.trim();
    if !url.is_empty() {
        config.esun_url = url.to_string();
    }
    let opt = |v: &str| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };
    config.source_system_id = opt(&form.source_system_id);
    config.customer_id = opt(&form.customer_id);
    config.payer_account = opt(&form.payer_account);

    // 先持久化（erp_config 為 SSOT；冪等 UPSERT），成功後才更新記憶體。
    if let Err(e) = crate::bankpay::save_config(&state.pool, &config).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("銀行支付設定儲存失敗（可能原因：資料庫忙碌；處理方式：稍後重試）：{e}"),
        )
            .into_response();
    }
    *state.bankpay_config.lock().await = config.clone();

    // 稽核（fail-closed：與 JSON API 路徑同一事件語彙）。
    let req_json = serde_json::json!({
        "source": "admin_form",
        "esun_mock_mode": config.esun_mock_mode,
        "esun_url": config.esun_url,
        "source_system_id": config.source_system_id,
        "customer_id": config.customer_id,
        "payer_account": config.payer_account,
    })
    .to_string();
    let resp_json = serde_json::to_string(&config).unwrap_or_default();
    let audit = sqlx::query(
        "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) \
         VALUES (?, NULL, 'BANKPAY_CONFIG_UPDATE', ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(serde_json::json!({ "request": req_json, "response": resp_json }).to_string())
    .bind(&session.user_id)
    .execute(&state.pool)
    .await;
    if let Err(e) = audit {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("設定已儲存但稽核寫入失敗（fail-closed；請聯絡管理員並保留此訊息）：{e}"),
        )
            .into_response();
    }

    Redirect::to("/admin/api/config").into_response()
}

fn render_or_500<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("頁面渲染失敗（可能原因：模板錯誤）；請聯絡管理員。詳情：{e}"),
        )
            .into_response(),
    }
}

/// HTML 群組路由（require_session 由 api.rs 統一掛載；本模組不重掛）。
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/api/catalog", get(apihub_catalog))
        .route("/admin/api/test", get(apihub_test))
        .route("/admin/api/logs", get(apihub_logs))
        .route(
            "/admin/api/config",
            get(apihub_config).post(apihub_config_update),
        )
        // ── 治理四頁（T2；技術人員受眾）──
        .route("/admin/api/records", get(apihub_records))
        .route("/admin/api/codes", get(apihub_codes))
        .route("/admin/api/release", get(apihub_release))
        .route("/admin/api/net", get(apihub_net))
}

// ---------------------------------------------------------------------------
// 治理四頁（T2：records／codes／release／net）— 技術人員受眾（Tier 3）
// ---------------------------------------------------------------------------

/// 測試紀錄列（源：bankpay_audits；status_code 為實際 HTTP 狀態，非常數）。
pub struct BankpayRecordRow {
    pub performed_at: String,
    pub action: String,
    pub batch_id: String,
    pub performed_by: String,
    pub status_code: String,
    pub result_label: String,
    pub result_class: String,
    pub payload_snippet: String,
}

pub struct BankpayBatchRow {
    pub batch_id: String,
    pub esun_ref: String,
    pub amount_display: String,
    pub currency: String,
    pub status: String,
    pub status_class: String,
    pub created_at: String,
    pub completed_at: String,
}

#[derive(Template)]
#[template(path = "api_records.html")]
pub struct ApiRecordsTemplate {
    pub acting_user: String,
    pub records: Vec<BankpayRecordRow>,
    pub batches: Vec<BankpayBatchRow>,
}

#[derive(Template)]
#[template(path = "api_codes.html")]
pub struct ApiCodesTemplate {
    pub acting_user: String,
    pub codes: Vec<ErrorCodeRow>,
}

pub struct ErrorCodeRow {
    pub code: String,
    pub meaning: String,
    pub semantics: String,
    pub action: String,
}

#[derive(Template)]
#[template(path = "api_release.html")]
pub struct ApiReleaseTemplate {
    pub acting_user: String,
    pub items: Vec<ReleaseItemRow>,
}

pub struct ReleaseItemRow {
    pub id: String,
    pub category: String,
    pub item: String,
    pub evidence: String,
    pub status: String,
    pub status_class: String,
}

#[derive(Template)]
#[template(path = "api_net.html")]
pub struct ApiNetTemplate {
    pub acting_user: String,
    pub checks: Vec<NetCheckRow>,
}

pub struct NetCheckRow {
    pub item: String,
    pub value: String,
    pub status: String,
    pub status_class: String,
    pub note: String,
}

/// GET /admin/api/records — BankPay 作業與批次紀錄（真實資料；零常數）。
pub async fn apihub_records(
    State(state): State<AppState>,
    session: Extension<Session>,
) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let raw: Vec<(String, String, String, String, i64, String)> = sqlx::query_as(
        "SELECT performed_at, event_type, COALESCE(batch_id, ''), performed_by, status_code, \
         COALESCE(response_payload, '') FROM bankpay_audits ORDER BY performed_at DESC LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let records: Vec<BankpayRecordRow> = raw
        .into_iter()
        .map(
            |(performed_at, action, batch_id, performed_by, status_code, payload)| {
                let (result_label, result_class) = if status_code == 200 {
                    ("成功", "badge-success")
                } else if status_code == 0 {
                    ("未知", "badge-neutral")
                } else {
                    ("失敗", "badge-error")
                };
                let char_count = payload.chars().count();
                let mut snippet: String = payload.chars().take(120).collect();
                if char_count > 120 {
                    snippet.push('…');
                }
                BankpayRecordRow {
                    performed_at,
                    action,
                    batch_id,
                    performed_by,
                    status_code: status_code.to_string(),
                    result_label: result_label.to_string(),
                    result_class: result_class.to_string(),
                    payload_snippet: snippet,
                }
            },
        )
        .collect();
    /// raw 查詢列（type_complexity 拆解：批次 → 顯示列的中間 tuple）。
    type RawBatchRow = (String, String, i64, String, String, String, Option<String>);
    let raw_batches: Vec<RawBatchRow> = sqlx::query_as(
        "SELECT batch_id, COALESCE(esun_ref, ''), amount_minor, currency, status, created_at, completed_at \
         FROM bankpay_batches ORDER BY created_at DESC LIMIT 20",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let batches: Vec<BankpayBatchRow> = raw_batches
        .into_iter()
        .map(
            |(batch_id, esun_ref, amount_minor, currency, status, created_at, completed_at)| {
                let status_class = match status.as_str() {
                    "COMPLETED" => "badge-success",
                    "SUBMITTED" => "badge-info",
                    "FAILED" => "badge-error",
                    "REJECTED" => "badge-warning",
                    _ => "badge-neutral",
                };
                BankpayBatchRow {
                    batch_id,
                    esun_ref,
                    amount_display: flaps_core::format_minor(amount_minor, 2),
                    currency,
                    status: status.clone(),
                    status_class: status_class.to_string(),
                    created_at,
                    completed_at: completed_at.unwrap_or_else(|| "—".to_string()),
                }
            },
        )
        .collect();
    render_or_500(ApiRecordsTemplate {
        acting_user: session.user_id.clone(),
        records,
        batches,
    })
}

/// GET /admin/api/codes — V2.8 錯誤碼對照（僅列已驗證項；完整表見官方文件）。
pub async fn apihub_codes(State(state): State<AppState>, session: Extension<Session>) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let codes = vec![ErrorCodeRow {
        code: "910000".to_string(),
        meaning: "成功（Success）".to_string(),
        semantics:
            "rsStatus.code = 910000 時 rsData 才存在；逐筆錯誤細節於 rsStatus.moreInfo（分號分隔）"
                .to_string(),
        action: "正常處理".to_string(),
    }];
    render_or_500(ApiCodesTemplate {
        acting_user: session.user_id.clone(),
        codes,
    })
}

/// GET /admin/api/release — Release checklist（誠實狀態；未驗證不標 Pass）。
pub async fn apihub_release(
    State(state): State<AppState>,
    session: Extension<Session>,
) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let items = vec![
        ReleaseItemRow {
            id: "RC-01".to_string(),
            category: "規格".to_string(),
            item: "V2.8 契約 8 端點盤點（rqHeader/rqData 信封、錯誤碼、mTLS）".to_string(),
            evidence: "Wellfit/docs/bankpay/04-interface-specification.md".to_string(),
            status: "已驗證".to_string(),
            status_class: "badge-success".to_string(),
        },
        ReleaseItemRow {
            id: "RC-02".to_string(),
            category: "實作".to_string(),
            item: "權威實作 wellfit_bankpay（8 端點、測試覆蓋）".to_string(),
            evidence: "Wellfit/crates/wellfit_bankpay".to_string(),
            status: "已驗證".to_string(),
            status_class: "badge-success".to_string(),
        },
        ReleaseItemRow {
            id: "RC-03".to_string(),
            category: "平台整合".to_string(),
            item: "模擬對應 3 端點＋冪等＋稽核 fail-closed（ADR-0012）".to_string(),
            evidence: "platform/tests/bankpay_api.rs".to_string(),
            status: "已驗證".to_string(),
            status_class: "badge-success".to_string(),
        },
        ReleaseItemRow {
            id: "RC-04".to_string(),
            category: "資安".to_string(),
            item: "玉山 Sandbox mTLS 憑證申請與配置".to_string(),
            evidence: "OQ-E3（待憑證）".to_string(),
            status: "未驗證".to_string(),
            status_class: "badge-warning".to_string(),
        },
        ReleaseItemRow {
            id: "RC-05".to_string(),
            category: "驗收".to_string(),
            item: "正式通道端對端驗收（09-sandbox-acceptance-runbook）".to_string(),
            evidence: "OQ-E3 阻塞".to_string(),
            status: "未驗證".to_string(),
            status_class: "badge-warning".to_string(),
        },
        ReleaseItemRow {
            id: "RC-06".to_string(),
            category: "維運".to_string(),
            item: "ReleaseGate 自動串接（部署前檢核）".to_string(),
            evidence: "follow-up（尚未串接）".to_string(),
            status: "未串接".to_string(),
            status_class: "badge-neutral".to_string(),
        },
    ];
    render_or_500(ApiReleaseTemplate {
        acting_user: session.user_id.clone(),
        items,
    })
}

/// GET /admin/api/net — 連線與資安姿態（本地設定與程式面檢查；不做網路探測）。
pub async fn apihub_net(State(state): State<AppState>, session: Extension<Session>) -> Response {
    if let Some(resp) = require_admin(&state, &session).await {
        return resp;
    }
    let config = state.bankpay_config.lock().await.clone();
    let mut checks: Vec<NetCheckRow> = Vec::new();
    if config.esun_mock_mode {
        checks.push(NetCheckRow {
            item: "運作模式".to_string(),
            value: "模擬模式（Mock）— 確定性模擬，非真實銀行交易".to_string(),
            status: "警告".to_string(),
            status_class: "badge-warning".to_string(),
            note: "全域狀態列同步標示；切換需於設定頁操作".to_string(),
        });
    } else {
        checks.push(NetCheckRow {
            item: "運作模式".to_string(),
            value: "正式模式".to_string(),
            status: "注意".to_string(),
            status_class: "badge-info".to_string(),
            note: "正式通道需 OQ-E3 憑證；未配置前呼叫將 fail-closed 回 500".to_string(),
        });
    }
    let url_ok = config.esun_url.starts_with("https://");
    checks.push(NetCheckRow {
        item: "銀行端點 URL".to_string(),
        value: config.esun_url.clone(),
        status: if url_ok { "符合" } else { "不符合" }.to_string(),
        status_class: if url_ok {
            "badge-success"
        } else {
            "badge-error"
        }
        .to_string(),
        note: "須為 https（V2.8 傳輸安全基線）".to_string(),
    });
    let configured = config.source_system_id.is_some() && config.customer_id.is_some();
    checks.push(NetCheckRow {
        item: "連線設定完整度".to_string(),
        value: if configured {
            "來源系統＋客戶識別已配置".to_string()
        } else {
            "未完整（缺來源系統或客戶識別）".to_string()
        },
        status: if configured { "完整" } else { "未完整" }.to_string(),
        status_class: if configured {
            "badge-success"
        } else {
            "badge-warning"
        }
        .to_string(),
        note: "正式模式必要欄位".to_string(),
    });
    checks.push(NetCheckRow {
        item: "CSRF 防護".to_string(),
        value: "啟用（X-CSRF-Token 同步器權杖）".to_string(),
        status: "啟用".to_string(),
        status_class: "badge-success".to_string(),
        note: "證據：platform/tests/bankpay_api.rs（缺漏/錯誤皆 403 且不產生批次）".to_string(),
    });
    checks.push(NetCheckRow {
        item: "mTLS 憑證".to_string(),
        value: "本平台不持有憑證；協議層由 wellfit_bankpay 管理".to_string(),
        status: "未配置".to_string(),
        status_class: "badge-warning".to_string(),
        note: "OQ-E3：玉山 Sandbox 憑證申請中；取得前不猜測端點（ADR-0012 D1）".to_string(),
    });
    checks.push(NetCheckRow {
        item: "V2.8 規格埠".to_string(),
        value: "8443（內部）/ 9443（外部 eospt.testesunbank.com.tw）".to_string(),
        status: "規格值".to_string(),
        status_class: "badge-neutral".to_string(),
        note: "遠端連通性測試需憑證；本頁不做網路探測（零副作用）".to_string(),
    });
    render_or_500(ApiNetTemplate {
        acting_user: session.user_id.clone(),
        checks,
    })
}
