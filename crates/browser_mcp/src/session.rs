#![forbid(unsafe_code)]
//! 瀏覽器工作階段：惰性啟動 Chrome、序列化呼叫、生命週期與 profile 管理。
//!
//! - D-13：一律使用專用暫存 `user_data_dir`，結束時清除。
//! - D-14：Chrome 選擇序 `--chrome` > `CHROME` 環境變數 > chromiumoxide 自動偵測。
//! - D-15：單一工具呼叫在途（本結構以 `Mutex` 包裹整個工作階段）。
//! - D-11：工具路徑不得 panic。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::accessibility::{
    EnableParams as AxEnableParams, GetFullAxTreeParams,
};
use chromiumoxide::cdp::browser_protocol::dom::{BackendNodeId, ResolveNodeParams};
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams, DispatchMouseEventType,
    InsertTextParams, MouseButton,
};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotParams, GetNavigationHistoryParams, NavigateParams,
    NavigateToHistoryEntryParams,
};
use chromiumoxide::cdp::js_protocol::runtime::{
    CallArgument, CallFunctionOnParams, EvaluateParams, RemoteObjectId,
};
use chromiumoxide::page::Page;
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::error::{ToolError, ToolResult};
use crate::snapshot::{AxNodeFlat, Limits, Snapshot, render};

/// 工作階段設定。
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// headless 模式（預設 headed）。
    pub headless: bool,
    /// Chrome 執行檔（優先於 `CHROME`）。
    pub chrome: Option<PathBuf>,
    /// 單次工具呼叫逾時。
    pub timeout: Duration,
    /// 快照上限。
    pub limits: Limits,
}

impl SessionConfig {
    /// 依 D-14 解析 Chrome 執行檔：`--chrome` > `CHROME` 環境變數 > 自動偵測。
    pub fn resolved_chrome(&self) -> Option<PathBuf> {
        if let Some(path) = &self.chrome {
            return Some(path.clone());
        }
        std::env::var_os("CHROME").map(PathBuf::from)
    }
}

/// 進行中的瀏覽器狀態。
struct Active {
    browser: Browser,
    page: Page,
    handler: JoinHandle<()>,
    profile_dir: PathBuf,
    refs: BTreeMap<String, i64>,
}

/// 瀏覽器工作階段（序列化所有工具呼叫）。
pub struct BrowserSession {
    inner: Mutex<Option<Active>>,
    config: SessionConfig,
}

impl BrowserSession {
    /// 建立工作階段（不啟動瀏覽器；首次使用時惰性啟動）。
    pub fn new(config: SessionConfig) -> Self {
        Self {
            inner: Mutex::new(None),
            config,
        }
    }

    /// 對外可見的逾時設定（毫秒）。
    pub fn timeout_ms(&self) -> u128 {
        self.config.timeout.as_millis()
    }

    /// 導覽至 URL 並等待頁面載入完成。
    pub async fn navigate(&self, url: &str) -> ToolResult<String> {
        validate_url(url)?;
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        active.refs.clear();

        let params = NavigateParams::builder().url(url).build().map_err(|e| {
            ToolError::new(format!(
                "導覽參數建構失敗：{e}。可能原因：URL 格式不合法。處理方式：確認 URL 後重試。"
            ))
        })?;
        let result = self.exec(&active.page, params, "導覽").await?;
        if let Some(error_text) = &result.error_text {
            return Err(ToolError::new(format!(
                "導覽 `{url}` 失敗：{error_text}。可能原因：目標無法連線或網址錯誤。處理方式：確認本機服務已啟動後重試。"
            )));
        }
        self.wait_until_ready(&active.page).await?;
        Ok(format!(
            "已導覽至 {url}（載入完成；元素 ref 已重建，請重新呼叫 snapshot）。"
        ))
    }

    /// 取得無障礙樹快照（含元素 ref）。
    pub async fn snapshot(&self) -> ToolResult<Snapshot> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;

        self.exec(&active.page, AxEnableParams::default(), "啟用無障礙域")
            .await?;

        let params = GetFullAxTreeParams::builder().build();
        let result = self.exec(&active.page, params, "取得無障礙樹").await?;

        let flat: Vec<AxNodeFlat> = result.nodes.iter().map(to_flat_node).collect();
        let snap = render(&flat, &self.config.limits);
        active.refs = snap.refs.clone();
        Ok(snap)
    }

    /// 點擊指定 ref 元素（捲入視野後以滑鼠事件點擊中心點）。
    pub async fn click(&self, ref_id: &str) -> ToolResult<String> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let (x, y) = self.element_center(active, ref_id).await?;
        self.dispatch_mouse(&active.page, DispatchMouseEventType::MousePressed, x, y)
            .await?;
        self.dispatch_mouse(&active.page, DispatchMouseEventType::MouseReleased, x, y)
            .await?;
        Ok(format!("已點擊 {ref_id}（座標 {x:.1}, {y:.1}）。"))
    }

    /// 於指定 ref 輸入元素輸入文字（不清空既有內容）。
    pub async fn type_text(&self, ref_id: &str, text: &str) -> ToolResult<String> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let object_id = self.resolve_object(active, ref_id).await?;
        let focus = CallFunctionOnParams::builder()
            .object_id(object_id.clone())
            .function_declaration("function(){ this.focus(); }")
            .build()
            .map_err(|e| param_error("聚焦", &e))?;
        self.exec(&active.page, focus, "聚焦元素").await?;

        let insert = InsertTextParams::builder()
            .text(text)
            .build()
            .map_err(|e| param_error("輸入", &e))?;
        self.exec(&active.page, insert, "輸入文字").await?;
        Ok(format!(
            "已於 {ref_id} 輸入 {} 個字元（既有內容保留）。",
            text.chars().count()
        ))
    }

    /// 發送按鍵（keyDown＋keyUp）。
    pub async fn press_key(&self, key: &str) -> ToolResult<String> {
        let spec = key_spec(key)?;
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;

        for event_type in [DispatchKeyEventType::KeyDown, DispatchKeyEventType::KeyUp] {
            let params = DispatchKeyEventParams::builder()
                .r#type(event_type)
                .key(spec.key)
                .code(spec.code)
                .windows_virtual_key_code(spec.vk)
                .build()
                .map_err(|e| param_error("按鍵", &e))?;
            self.exec(&active.page, params, "發送按鍵").await?;
        }
        Ok(format!("已發送按鍵 `{key}`。"))
    }

    /// 設定下拉選項值（派發 change 事件）。
    pub async fn select_option(&self, ref_id: &str, value: &str) -> ToolResult<String> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let object_id = self.resolve_object(active, ref_id).await?;

        let argument = CallArgument::builder()
            .value(serde_json::Value::String(value.to_string()))
            .build();
        let call = CallFunctionOnParams::builder()
            .object_id(object_id)
            .function_declaration(
                "function(v){ this.value = v; this.dispatchEvent(new Event('change', { bubbles: true })); return this.value; }",
            )
            .arguments(vec![argument])
            .return_by_value(true)
            .build()
            .map_err(|e| param_error("選項設定", &e))?;
        let result = self.exec(&active.page, call, "設定下拉選項").await?;
        let applied = result
            .result
            .value
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if applied != value {
            return Err(ToolError::new(format!(
                "設定 {ref_id} 選項 `{value}` 未生效（目前值 `{applied}`）。可能原因：選項不存在或元素非 select。處理方式：先 snapshot 確認可選項後重試。"
            )));
        }
        Ok(format!("已將 {ref_id} 設為 `{value}`。"))
    }

    /// 將滑鼠移動到指定 ref 元素中心。
    pub async fn hover(&self, ref_id: &str) -> ToolResult<String> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let (x, y) = self.element_center(active, ref_id).await?;
        self.dispatch_mouse(&active.page, DispatchMouseEventType::MouseMoved, x, y)
            .await?;
        Ok(format!("已將滑鼠移至 {ref_id}（座標 {x:.1}, {y:.1}）。"))
    }

    /// 截取目前畫面（PNG bytes）。
    pub async fn screenshot(&self) -> ToolResult<Vec<u8>> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let params = CaptureScreenshotParams::builder().build();
        let result = self.exec(&active.page, params, "截圖").await?;
        let encoded = String::from(result.data);
        BASE64.decode(encoded.as_bytes()).map_err(|e| {
            ToolError::new(format!(
                "截圖資料解碼失敗：{e}。可能原因：CDP 回傳非預期格式。處理方式：重試或回報問題。"
            ))
        })
    }

    /// 回到上一頁（瀏覽歷史）。
    pub async fn go_back(&self) -> ToolResult<String> {
        let mut guard = self.inner.lock().await;
        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;
        let params = GetNavigationHistoryParams::default();
        let history = self.exec(&active.page, params, "查詢瀏覽歷史").await?;
        if history.current_index <= 0 {
            return Err(ToolError::new(
                "已在最舊的瀏覽紀錄，無法返回。可能原因：尚未導覽過其他頁面。處理方式：先 navigate 至另一頁後重試。",
            ));
        }
        let index = (history.current_index - 1) as usize;
        let Some(entry) = history.entries.get(index) else {
            return Err(ToolError::new(
                "瀏覽歷史索引不一致，無法返回。處理方式：以 navigate 直接前往目標頁面。",
            ));
        };
        let go = NavigateToHistoryEntryParams::builder()
            .entry_id(entry.id)
            .build()
            .map_err(|e| param_error("返回", &e))?;
        self.exec(&active.page, go, "返回上一頁").await?;
        self.wait_until_ready(&active.page).await?;
        active.refs.clear();
        Ok("已返回上一頁（元素 ref 已重建，請重新呼叫 snapshot）。".to_string())
    }

    /// 等待指定條件（三者恰一）。
    pub async fn wait_for(
        &self,
        time_ms: Option<u64>,
        text: Option<&str>,
        ref_id: Option<&str>,
    ) -> ToolResult<String> {
        let specified = [time_ms.is_some(), text.is_some(), ref_id.is_some()]
            .iter()
            .filter(|x| **x)
            .count();
        if specified != 1 {
            return Err(ToolError::new(
                "wait_for 需要且僅需要一種條件（time_ms、text、ref 三者恰一）。處理方式：只提供其中一個參數。",
            ));
        }

        let mut guard = self.inner.lock().await;

        if let Some(ms) = time_ms {
            let bounded = ms.min(self.config.timeout.as_millis() as u64);
            tokio::time::sleep(Duration::from_millis(bounded)).await;
            return Ok(format!("已等待 {bounded} ms。"));
        }

        self.ensure(&mut guard).await?;
        let active = active_mut(&mut guard)?;

        if let Some(target) = ref_id {
            if active.refs.contains_key(target) {
                return Ok(format!("{target} 存在於目前快照中。"));
            }
            return Err(ToolError::new(format!(
                "{target} 不存在於目前快照中。可能原因：尚未 snapshot 或頁面已變更。處理方式：先呼叫 snapshot 取得最新 ref。"
            )));
        }

        let needle = text.unwrap_or_default();
        let expression = format!(
            "document.body ? document.body.innerText.includes({}) : false",
            serde_json::Value::String(needle.to_string())
        );
        let deadline = tokio::time::Instant::now() + self.config.timeout;
        loop {
            let params = EvaluateParams::builder()
                .expression(expression.clone())
                .return_by_value(true)
                .build()
                .map_err(|e| param_error("等待文字", &e))?;
            let result = self.exec(&active.page, params, "等待文字出現").await?;
            if result.result.value.as_ref().and_then(|v| v.as_bool()) == Some(true) {
                return Ok(format!("頁面已出現文字 `{needle}`。"));
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(ToolError::new(format!(
                    "等待文字 `{needle}` 逾時（{} ms）。可能原因：文字未出現或為動態載入。處理方式：確認文案無誤，或提高 --timeout-ms 後重試。",
                    self.timeout_ms()
                )));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// 關閉瀏覽器並清除暫存 profile（可重複呼叫）。
    pub async fn close(&self) -> ToolResult<()> {
        let mut guard = self.inner.lock().await;
        let Some(mut active) = guard.take() else {
            return Ok(());
        };
        let _ = tokio::time::timeout(Duration::from_secs(10), active.browser.close()).await;
        let _ = tokio::time::timeout(Duration::from_secs(10), active.browser.wait()).await;
        active.handler.abort();
        let _ = std::fs::remove_dir_all(&active.profile_dir);
        Ok(())
    }

    /// 伺服器結束時的清理（與 `close` 相同語義）。
    pub async fn shutdown(&self) {
        let _ = self.close().await;
    }

    /// 確保瀏覽器已啟動（惰性）。
    async fn ensure(&self, slot: &mut Option<Active>) -> ToolResult<()> {
        if slot.is_some() {
            return Ok(());
        }

        let profile_dir =
            std::env::temp_dir().join(format!("browser-mcp-profile-{}", std::process::id()));
        std::fs::create_dir_all(&profile_dir).map_err(|e| {
            ToolError::new(format!(
                "暫存 profile 目錄建立失敗（{e}）：可能原因：權限不足或磁碟問題。處理方式：檢查 %TEMP% 權限後重試。"
            ))
        })?;

        let mut builder = BrowserConfig::builder()
            .user_data_dir(&profile_dir)
            .window_size(1280, 800)
            .launch_timeout(Duration::from_secs(30));
        if let Some(chrome) = self.config.resolved_chrome() {
            builder = builder.chrome_executable(chrome);
        }
        builder = if self.config.headless {
            builder.new_headless_mode()
        } else {
            builder.with_head()
        };
        let config = builder.build().map_err(|e| {
            ToolError::new(format!(
                "Chrome 啟動設定建構失敗：{e}。可能原因：參數不合法。處理方式：確認 --chrome 路徑後重試。"
            ))
        })?;

        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            let detail = e.to_string();
            let lower = detail.to_lowercase();
            if lower.contains("detect")
                || lower.contains("not found")
                || lower.contains("no such file")
            {
                ToolError::new(format!(
                    "找不到可用的 Chrome（{detail}）。可能原因：未安裝 Chrome 或路徑未設定。處理方式：安裝 Chrome，或設定 CHROME 環境變數／以 --chrome <path> 指定執行檔。"
                ))
            } else {
                ToolError::new(format!(
                    "Chrome 啟動失敗：{detail}。可能原因：執行檔損毀、被安全政策阻擋或埠衝突。處理方式：確認可手動開啟 Chrome，或改用 --headless 後重試。"
                ))
            }
        })?;

        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let page = browser.new_page("about:blank").await.map_err(|e| {
            ToolError::new(format!(
                "建立分頁失敗：{e}。可能原因：瀏覽器初始化異常。處理方式：重試或回報問題。"
            ))
        })?;

        *slot = Some(Active {
            browser,
            page,
            handler: handle,
            profile_dir,
            refs: BTreeMap::new(),
        });
        Ok(())
    }

    /// 等待 `document.readyState === "complete"`。
    async fn wait_until_ready(&self, page: &Page) -> ToolResult<()> {
        let deadline = tokio::time::Instant::now() + self.config.timeout;
        loop {
            let params = EvaluateParams::builder()
                .expression("document.readyState")
                .return_by_value(true)
                .build()
                .map_err(|e| param_error("載入偵測", &e))?;
            let result = self.exec(page, params, "偵測載入狀態").await?;
            if result.result.value.as_ref().and_then(|v| v.as_str()) == Some("complete") {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(ToolError::new(format!(
                    "頁面載入逾時（{} ms）。可能原因：目標服務無回應或資源載入緩慢。處理方式：確認服務狀態，或提高 --timeout-ms 後重試。",
                    self.timeout_ms()
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// 將 ref 解析為遠端物件 id。
    async fn resolve_object(
        &self,
        active: &mut Active,
        ref_id: &str,
    ) -> ToolResult<RemoteObjectId> {
        let Some(backend_id) = active.refs.get(ref_id).copied() else {
            return Err(ToolError::new(format!(
                "{ref_id} 不存在於目前快照中。可能原因：尚未 snapshot、頁面已變更或曾導覽。處理方式：先呼叫 snapshot 取得最新 ref。"
            )));
        };
        let params = ResolveNodeParams::builder()
            .backend_node_id(BackendNodeId::new(backend_id))
            .build();
        let resolved = self.exec(&active.page, params, "解析元素").await?;
        resolved.object.object_id.clone().ok_or_else(|| {
            ToolError::new(format!(
                "{ref_id} 無法解析為可操作元素。可能原因：節點已失效。處理方式：重新 snapshot 後重試。"
            ))
        })
    }

    /// 取得元素中心座標（先捲入視野）。
    async fn element_center(&self, active: &mut Active, ref_id: &str) -> ToolResult<(f64, f64)> {
        let object_id = self.resolve_object(active, ref_id).await?;
        let call = CallFunctionOnParams::builder()
            .object_id(object_id)
            .function_declaration(
                "function(){ this.scrollIntoView({ block: 'center', inline: 'center' }); const r = this.getBoundingClientRect(); const x = r.width > 0 ? r.left + r.width / 2 : r.left; const y = r.height > 0 ? r.top + r.height / 2 : r.top; return { x: x, y: y }; }",
            )
            .return_by_value(true)
            .build()
            .map_err(|e| param_error("座標計算", &e))?;
        let result = self.exec(&active.page, call, "計算元素座標").await?;
        let point = result.result.value.as_ref().ok_or_else(|| {
            ToolError::new(format!(
                "{ref_id} 座標計算失敗。可能原因：元素已消失。處理方式：重新 snapshot 後重試。"
            ))
        })?;
        let x = point.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = point.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        Ok((x, y))
    }

    /// 發送滑鼠事件。
    async fn dispatch_mouse(
        &self,
        page: &Page,
        event_type: DispatchMouseEventType,
        x: f64,
        y: f64,
    ) -> ToolResult<()> {
        let params = DispatchMouseEventParams::builder()
            .r#type(event_type)
            .x(x)
            .y(y)
            .button(MouseButton::Left)
            .click_count(1)
            .build()
            .map_err(|e| param_error("滑鼠事件", &e))?;
        self.exec(page, params, "發送滑鼠事件").await?;
        Ok(())
    }

    /// 執行 CDP 命令並套用逾時（D-11：永不 panic）。
    async fn exec<T>(&self, page: &Page, command: T, what: &str) -> ToolResult<T::Response>
    where
        T: chromiumoxide::Command,
    {
        match tokio::time::timeout(self.config.timeout, page.execute(command)).await {
            Ok(Ok(response)) => Ok(response.result),
            Ok(Err(error)) => Err(ToolError::new(format!(
                "{what} 失敗：{error}。可能原因：頁面已關閉、元素失效或 CDP 連線中斷。處理方式：關閉後重試或回報問題。"
            ))),
            Err(_) => Err(ToolError::new(format!(
                "{what} 逾時（{} ms）。可能原因：瀏覽器忙碌或頁面無回應。處理方式：提高 --timeout-ms 或重試。",
                self.timeout_ms()
            ))),
        }
    }
}

/// 取得可變的進行中狀態。
fn active_mut(slot: &mut Option<Active>) -> ToolResult<&mut Active> {
    slot.as_mut().ok_or_else(|| {
        ToolError::new(
            "內部狀態異常：瀏覽器未初始化。可能原因：啟動流程中斷。處理方式：重試或回報問題。",
        )
    })
}

/// 組合參數錯誤訊息。
fn param_error(what: &str, detail: &str) -> ToolError {
    ToolError::new(format!(
        "{what} 參數建構失敗：{detail}。可能原因：輸入不合法。處理方式：檢查參數後重試。"
    ))
}

/// 僅允許 http／https／about／data 協定。
fn validate_url(url: &str) -> ToolResult<()> {
    let lower = url.trim().to_lowercase();
    let allowed = ["http://", "https://", "about:", "data:"];
    if allowed.iter().any(|prefix| lower.starts_with(prefix)) {
        Ok(())
    } else {
        Err(ToolError::new(format!(
            "不支援的 URL `{url}`。可能原因：協定不在允許清單（http／https／about／data）。處理方式：改用完整 http(s) URL。"
        )))
    }
}

/// 按鍵規格。
struct KeySpec {
    key: &'static str,
    code: &'static str,
    vk: i64,
}

/// 常見按鍵對照（其餘單一可列印字元直接送 keyDown／keyUp）。
fn key_spec(key: &str) -> ToolResult<KeySpec> {
    let spec = match key {
        "Enter" => KeySpec {
            key: "Enter",
            code: "Enter",
            vk: 13,
        },
        "Tab" => KeySpec {
            key: "Tab",
            code: "Tab",
            vk: 9,
        },
        "Escape" => KeySpec {
            key: "Escape",
            code: "Escape",
            vk: 27,
        },
        "Backspace" => KeySpec {
            key: "Backspace",
            code: "Backspace",
            vk: 8,
        },
        "ArrowUp" => KeySpec {
            key: "ArrowUp",
            code: "ArrowUp",
            vk: 38,
        },
        "ArrowDown" => KeySpec {
            key: "ArrowDown",
            code: "ArrowDown",
            vk: 40,
        },
        "ArrowLeft" => KeySpec {
            key: "ArrowLeft",
            code: "ArrowLeft",
            vk: 37,
        },
        "ArrowRight" => KeySpec {
            key: "ArrowRight",
            code: "ArrowRight",
            vk: 39,
        },
        "Home" => KeySpec {
            key: "Home",
            code: "Home",
            vk: 36,
        },
        "End" => KeySpec {
            key: "End",
            code: "End",
            vk: 35,
        },
        "PageUp" => KeySpec {
            key: "PageUp",
            code: "PageUp",
            vk: 33,
        },
        "PageDown" => KeySpec {
            key: "PageDown",
            code: "PageDown",
            vk: 34,
        },
        "Space" | " " => KeySpec {
            key: " ",
            code: "Space",
            vk: 32,
        },
        single if single.chars().count() == 1 => {
            return Err(ToolError::new(format!(
                "不支援的按鍵 `{single}`。可能原因：單一字元請改用 type 工具。處理方式：type 輸入文字，或以支援的具名按鍵（Enter／Tab／Escape／方向鍵等）呼叫 press_key。"
            )));
        }
        other => {
            return Err(ToolError::new(format!(
                "不支援的按鍵 `{other}`。可能原因：未在支援清單。處理方式：使用 Enter／Tab／Escape／Backspace／ArrowUp／ArrowDown／ArrowLeft／ArrowRight／Home／End／PageUp／PageDown／Space。"
            )));
        }
    };
    Ok(spec)
}

/// 將 CDP AX 節點扁平化為本模組輸入型別。
fn to_flat_node(node: &chromiumoxide::cdp::browser_protocol::accessibility::AxNode) -> AxNodeFlat {
    let role = axis_value(node.role.as_ref());
    let name = axis_value(node.name.as_ref());
    let value = node
        .value
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let flags = node
        .properties
        .as_deref()
        .map(collect_flags)
        .unwrap_or_default();
    AxNodeFlat {
        id: node.node_id.inner().clone(),
        parent_id: node.parent_id.as_ref().map(|id| id.inner().clone()),
        role,
        name,
        value,
        flags,
        ignored: node.ignored,
        backend_node_id: node.backend_dom_node_id.as_ref().map(|id| *id.inner()),
    }
}

/// 取出 AX 值的字串內容。
fn axis_value(
    value: Option<&chromiumoxide::cdp::browser_protocol::accessibility::AxValue>,
) -> String {
    value
        .and_then(|v| v.value.as_ref())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 收集關注的狀態旗標（checked／disabled／expanded／focused／selected／required／readonly／invalid／level／valuetext）。
fn collect_flags(
    properties: &[chromiumoxide::cdp::browser_protocol::accessibility::AxProperty],
) -> Vec<String> {
    const INTERESTING: [&str; 11] = [
        "checked",
        "disabled",
        "expanded",
        "focused",
        "selected",
        "required",
        "readonly",
        "invalid",
        "level",
        "valuetext",
        "multiselectable",
    ];
    let mut out = Vec::new();
    for property in properties {
        let name = serde_json::to_value(&property.name)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string));
        let Some(name) = name else { continue };
        if !INTERESTING.contains(&name.as_str()) {
            continue;
        }
        let value = property.value.value.as_ref();
        let rendered = match value {
            Some(serde_json::Value::Bool(true)) => name.clone(),
            Some(serde_json::Value::Bool(false)) => format!("{name}=false"),
            Some(serde_json::Value::String(text)) if !text.is_empty() => {
                format!("{name}=\"{text}\"")
            }
            Some(serde_json::Value::Number(number)) => format!("{name}={number}"),
            _ => continue,
        };
        out.push(rendered);
    }
    out
}
