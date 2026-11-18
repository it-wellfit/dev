# 啟用瀏覽器自動化 — 純 Rust MCP 伺服器（路徑 C）v3
> 2026-11-17 Reviewed（整合完成計畫 1794810000000 I-4）：掃描無未勾選任務（\- [ ]\=0）；無未關閉程式任務。證據：PLAN_REGISTER addendum #52。

## 狀態：已實作並通過閘門（2026-11-15）

## 目標

以**純 Rust** 自建 MCP 伺服器 `browser-mcp`，取代 Kilo 內建（硬編碼 `npx`）的 Browser Automation，提供瀏覽器工具給 Kilo CLI 後端，**完全不使用 Node.js／npm／bun**；並消除一開啟即出現的 `MCP error -32000: Connection closed`。

## 根因摘要

錯誤來自 VS Code 擴充功能內建 `BrowserAutomationService`（`packages/kilo-vscode/src/services/browser-automation/browser-automation-service.ts`），硬編碼 `["npx", "@playwright/mcp@latest"]` 註冊 `kilo-playwright` MCP 伺服器，不可由設定改寫指令；與「不用 Node.js/npm」約束互斥。→ 改走獨立自建 MCP 伺服器，並關閉內建開關。

## 已鎖定決策

| 代號 | 決策 | 值 |
|---|---|---|
| D-1 | 路徑 | C：純 Rust MCP 伺服器 |
| D-2 | 工具範圍 | 精簡子集（本機 web app UI 驗證） |
| D-3 | CDP 引擎 | `chromiumoxide` 0.9.x（pin 精確版；MSRV 1.85、edition 2024，與工作區相容） |
| D-4 | 瀏覽器來源 | 系統 Chrome；**不**啟用 fetcher、不自動下載 |
| D-5 | 傳輸 | local stdio（JSON-RPC） |
| D-6 | MCP SDK | `rmcp` v3.x（features：`server`,`macros`,`schemars`） |
| D-7 | 內建開關 | 使用者於 Settings → Web Tools → Browser Automation **關閉**（互動式） |
| D-8 | 現有 `playwright`（bunx）條目 | `.kilo/kilo.jsonc` 設 `enabled: false`（不刪除，利回滾） |
| D-9 | 部署 | `cargo install --path crates/browser_mcp --locked` → `%USERPROFILE%\.cargo\bin\browser-mcp.exe`（沿用既有 MCP 工具慣例；避開 `CARGO_TARGET_DIR` 漂移） |
| D-10 | 日誌紀律 | **stdout 僅 JSON-RPC**；`tracing` 全部導向 **stderr** |
| D-11 | 恐慌紀律 | 工具路徑**零 panic**；錯誤映射為 MCP **tool error**（release `panic="abort"`，panic 即殺死伺服器） |
| D-12 | 參數 | `--headless`、`--chrome <path>`、`--timeout-ms`；預設 **headed + 系統 Chrome**（對齊 Kilo 內建預設） |
| D-13 | Profile 隔離 | **一律使用專用暫存 `user_data_dir`**（`BrowserConfig.user_data_dir`；結束時清除）。避免污染使用者 Chrome profile 與「Chrome 已在執行」啟動衝突 |
| D-14 | Chrome 選擇序 | `--chrome` > `CHROME` 環境變數 > chromiumoxide 內建偵測（`which`／Windows registry） |
| D-15 | 呼叫序列化 | **單一工具呼叫在途**（頁面操作循序執行；併發呼叫排隊），避免 CDP 交錯 |
| D-16 | 錯誤語義 | 工具執行失敗 → 回 `isError: true` 的 tool result（含可行動訊息）；JSON-RPC error **僅**用於協議層錯誤 |

## 範圍（MCP 伺服器名 `browser`；Kilo 以 `{server}_{tool}` 命名）

| 工具 | 參數 | 語義（已精確定義） |
|---|---|---|
| `navigate` | `url` | 導覽並等待 load；完成後**重建** ref 對照 |
| `snapshot` | — | 無障礙樹縮排文字（`role "name" [ref=eN]`＋值/勾選等狀態）；受截斷上限保護（見下） |
| `click` | `ref` | 解析 ref → 座標（`getBoundingClientRect` 中心）→ `Input.dispatchMouseEvent`（pressed/released） |
| `type` | `ref`, `text` | focus 後 `Input.insertText`（**不**清空既有內容；清空需求以全選＋輸入處理，之後可加 `fill`） |
| `press_key` | `key` | `Input.dispatchKeyEvent`（`Enter`／`Tab` 等） |
| `select_option` | `ref`, `value` | 設值並派發 `change` |
| `hover` | `ref` | 移動滑鼠至元素中心 |
| `screenshot` | — | PNG（MCP image content，**不落地**） |
| `go_back` | — | 上一頁；**重建** ref |
| `wait_for` | `time_ms?` \| `text?` \| `ref?` | **三者恰一**（多給即錯誤）；總長受 `--timeout-ms` 上限 |
| `close` | — | 關閉瀏覽器並清除暫存 profile |

**Snapshot 截斷**：預設上限（例如 2000 節點或 60,000 字元，實作可調）＋明確截斷註記，避免灌爆模型上下文；忽略 `ignored` AX 節點的規則須固定且有測試。

**不在範圍**：network mock、storage/cookies 編輯、多分頁、tracing、video、frames、file upload、跨啟動 profile 持久化。

## 架構與資料流（已對 chromiumoxide API 查證）

```
Kilo CLI 後端 (MCP client)
  └─ spawn: %USERPROFILE%\.cargo\bin\browser-mcp.exe  (stdio JSON-RPC; 日誌→stderr)
        └─ rmcp ServerHandler（initialize / tools/list / tools/call）
              └─ BrowserSession actor（單一 async 任務；工具呼叫序列化）
                    ├─ Browser（BrowserConfig：user_data_dir=暫存、headless、with_executable）
                    ├─ Handler（**保持驅動**，JoinHandle 必須存活）
                    └─ Page（單頁）
```

- **runtime**：`#[tokio::main(flavor="multi_thread")]`；rmcp 服務與 chromiumoxide 共用。
- **啟動**：`Browser::launch(config)`；查證事實——**無可偵測 Chrome 即失敗**、**20 秒內未從子進程 stderr 取得 websocket URL 即失敗**（`launch_timeout` 預設 20s）。
- **關閉（防 zombie）**：`Browser::close()` → `Browser::wait()`／`try_wait()`；逾時則 `Browser::kill()`（自動等待）。**`Browser::drop` 會自動等待**，不會遺留 zombie Chrome。
- **Snapshot**：`Accessibility.enable`（保證 `AXNodeId` 跨呼叫一致）→ `Accessibility.getFullAXTree` → 格式化；**ref＝`AXNodeId`**；`AxNode.backendDOMNodeId` 供解析。
- **元素解析**：ref → `backendDOMNodeId` → `DOM.resolveNode` → `objectId` → `Runtime.callFunctionOn` 取座標。
- **優雅關閉**：stdin EOF（客戶端斷線）→ 關閉瀏覽器 → 行程退出。

## 關鍵實作紀律（易錯點）

1. **stdout 純淨**：任何 `println!`／第三方輸出污染 JSON-RPC → 連線中斷。第三方（chromiumoxide/rmcp）若寫 stdout 亦需驗證。
2. **Handler 必須被持續驅動**：丟棄其任務 → 頁面呼叫永久掛起。
3. **零 panic**：release `panic="abort"`，panic 直接殺進程（等效 `-32000`）。
4. **無 auto-wait**：時序需 `wait_for`；工具描述需明確引導。
5. **Chrome 缺失**：啟動即回報可行動錯誤（問題＋原因＋處理）。
6. **ref 失效**：`navigate`／`go_back` 後一律重建；用陳舊 ref 回報明確錯誤。

## 有序任務清單

1. **建立 crate**：`crates/browser_mcp`（`bin="browser-mcp"`）；`Cargo.toml`：`edition="2024"`、`rust-version="1.99"`、`license="LicenseRef-Proprietary"`、`publish=false`、**不設 optional features**（縮小 deny all-features 掃描面）；`src/main.rs`／`src/lib.rs` 首行 `#![forbid(unsafe_code)]`；根 `Cargo.toml` `members` 加入。
2. **依賴與治理**：`rmcp`、`chromiumoxide`（pin）、`tokio`（`rt-multi-thread`,`macros`,`time`,`sync`）、`serde`、`serde_json`、`schemars`、`thiserror`、`tracing`、`tracing-subscriber`、`base64`。查核 rmcp MSRV 相容。執行 `cargo deny check`＋`cargo audit`；如需 ignore，先於 `.cargo/audit.toml`（處置 SSOT）留書面處置再同步 `deny.toml`。
3. **MCP 骨架**：rmcp `ServerHandler`（`initialize`／`tools/list`／`tools/call`）；stdio；stderr 日誌；優雅關閉。
4. **Session actor**：`BrowserConfig`（暫存 `user_data_dir`、headless、`with_executable`）；handler 保活；單頁；序列化；`close`→`wait`→（逾時）`kill`；暫存清理；Chrome 缺失明確錯誤。
5. **實作工具**：§範圍 全部（含 D-15／D-16 語義）；每工具 `schemars` Schema＋描述（含 `wait_for` 引導）。
6. **Snapshot 與 ref**：AX 樹抓取、格式化、ref 指派、解析、導覽後重建、**截斷上限**。
7. **錯誤／逾時語義**：訊息「問題＋可能原因＋處理方式」；`--timeout-ms` 貫穿。
8. **CLI 參數**：D-12＋D-14 選擇序。
9. **測試**：單元（格式化／schema／ref 對照／截斷）；MCP 交握（rmcp `client`）；**stdout 純淨冒煙**（管入 `initialize`，斷言 stdout 為純 JSON）；整合（見驗證計畫之 Chrome 缺席雙路徑設計——**不使用 `#[ignore]`**，以符合 0 ignored）。
10. **建置與部署**：`cargo install --path crates/browser_mcp --locked`；記錄路徑與大小。
11. **註冊**：備份 `.kilo/kilo.jsonc`（C-01／C-04 斷言式補丁）：新增 `browser` 條目（`command`=絕對路徑、`enabled:true`、`timeout:60000`）；`playwright` 設 `enabled:false`。
12. **使用者動作**：關閉 Settings → Web Tools → Browser Automation；重啟 Kilo 客戶端。
13. **文件同步**（SHARED_STANDARDS §4.3 同提交）：`AGENTS.md` §5、`docs/engineering/SKILL_INVENTORY.md`、`docs/management/營運治理機制.md`、`docs/testing/MVP_TEST_PLAN.md`／`MVP_TEST_REPORT.md`。
14. **閘門**：`cargo fmt -- --check`；`cargo clippy --all-targets -- -D warnings`；`cargo test --locked`（0 failed／0 ignored）；`cargo deny check`；`cargo audit`。

## 依賴與治理（實測規則，2026-11-15）

- `deny.toml`：`unmaintained="all"`、`yanked="deny"`、`wildcards="deny"`、`multiple-versions="warn"`、`all-features=true`（掃描最大圖含 optional）、`allow-git=[]`、`licenses` 白名單（`chromiumoxide` MIT OR Apache-2.0 ✓）。
- 已知可接受：`reqwest` 0.12（platform）與 0.13（chromiumoxide）並存 → 僅 warn。
- 圖變更觸發複審；新增 ignore 需書面處置。
- `cargo test --locked` 前提：`Cargo.lock` 已更新並提交。
- **工作區「0 ignored」慣例**與整合測試衝突之解法：整合測試**不使用 `#[ignore]`**；改為**雙路徑斷言**——有 Chrome 時跑完整流程；無 Chrome 時斷言「回傳文件所述可行動錯誤且不 panic」（流程結果標 UNKNOWN，C-10）。

## 前置條件

系統已安裝 **Chrome**（或 `--chrome`／`CHROME`）；使用者關閉內建開關；重啟 Kilo 客戶端。

## 驗證計畫

| 層級 | 驗證 | 通過條件 |
|---|---|---|
| 單元 | snapshot 格式化／截斷、schema、ref 對照 | 測試綠 |
| MCP 交握 | rmcp client 經 stdio `initialize`／`tools/list` | 回應正確、11 工具齊全 |
| stdout 純淨 | 管入 `initialize` | stdout 為純 JSON、無雜訊 |
| 整合（雙路徑） | 有 Chrome：`localhost` navigate→snapshot→click→screenshot；無 Chrome：斷言可行動錯誤 | 對應路徑斷言皆綠 |
| zombie 檢查 | `close` 後確認無殘留 Chrome 子進程 | 無殘留 |
| 端對端 | Kilo 對話呼叫 `browser_*` | 工具可用、無 `-32000` |
| 閘門 | fmt／clippy／test --locked／deny／audit | 全綠、零警告 |

## 失敗模式

| 症狀 | 原因 | 處置 |
|---|---|---|
| 啟動即 `-32000` | 路徑錯／未安裝／crash | 獨立冒煙測試；檢查 stdout 純淨性 |
| 呼叫全掛 | handler 未驅動 | 保活設計＋審查 |
| 隨機崩潰 | panic（abort 殺進程） | 移除 `unwrap`／`expect` |
| 啟動失敗訊息 | 無 Chrome／20s 內無 websocket URL | 明確指引 `--chrome`／`CHROME` |
| 與使用者 Chrome 衝突 | 共用 profile | D-13 專用暫存 profile |
| zombie Chrome | 未收割子進程 | `close`→`wait`／`kill`；`drop` 自動等待 |
| 時序不穩 | 無 auto-wait | `wait_for` 引導 |
| 命名混淆（實驗性 `browser_open`） | 名稱相近 | 驗證；必要時改伺服器名 `browser-rust` |

## 回滾計畫

- `.kilo/kilo.jsonc`：`browser` 設 `enabled:false`（或移除）；必要時由備份還原；`playwright` 復為 `enabled:true`。
- 恢復 Kilo 內建瀏覽器：Settings 重開 Browser Automation（需 Node/npx）。
- 移除 crate 與 workspace member；`cargo uninstall browser-mcp`。

## 風險與待確認

- **治理閘門（高）**：`unmaintained="all"`＋`all-features=true` 可能使 chromiumoxide 樹失敗 → 需處置或改引擎（替代：`headless_chrome`）。
- **引擎成熟度**：pre-1.0、維護趨緩；pin＋lockfile。
- **快照保真度**：與 Playwright 有落差，可能迭代（`AXNodeId` 穩定性已由 `Accessibility.enable` 保證）。
- **MCP 協定版本**：確認 Kilo CLI 後端協商版本（rmcp 支援 `2025-11-25`／`2026-07-28`），以相容模式實作。
- **`cargo install` profile**：確認是否沿用工作區 `[profile.release]`。
- **`bunx` 條目去留**：採 `enabled:false`；是否改刪除待確認。

## 強制規則（遵守）

C-01 不亂覆蓋亂刪（編輯前備份）；C-03 先量測後動刀；C-04 斷言式補丁（先確認目標字串存在且唯一）；C-05 並行編輯先定基準；C-10 UNKNOWN ≠ PASS ≠ FAIL；C-11 Rust-first（不新增 Python）；C-13 Evidence-Driven 五級判定。每個 `.rs` 首行 `#![forbid(unsafe_code)]`；clippy 零警告；不得加 allow。

## 證據註記（C-10）

- chromiumoxide `BrowserConfig`／`Browser` API 事實：docs.rs `chromiumoxide 0.9.1`（2026-11-15 查證，含 `user_data_dir`、`with_executable`、`close`/`wait`/`kill`、`drop` 自動等待、`launch_timeout` 20s 預設）。
- Accessibility 域與 `AXNodeId` 一致性保證：docs.rs `chromiumoxide_cdp 0.9.1`（2026-11-15 查證）。
- deny/audit 政策：工作區 `deny.toml`／`.cargo/audit.toml`（2026-11-15 讀取）。
- 內建服務硬編碼 npx：`Kilo-Org/kilocode` `browser-automation-service.ts`（2026-11-15 查證）。

## 實作完成紀錄（2026-11-15；C-13 證據）

| 項目 | 證據 |
|---|---|
| Crate | `crates/browser_mcp`（bin `browser-mcp`；edition 2024／rust-version 1.99／`#![forbid(unsafe_code)]`；已加入根 `Cargo.toml` members） |
| 依賴治理 | `cargo deny check`：**advisories ok, bans ok, licenses ok, sources ok**；`cargo audit`：EXIT=0（掃描 500 依賴） |
| 測試 | 單元 13（cli／snapshot／server）＋整合 3（MCP 交握、stdout 純淨、Chrome 全流程）**16 passed / 0 failed / 0 ignored** |
| Chrome 全流程 | `chrome_roundtrip`：navigate→snapshot（含 "Hello"）→click→screenshot（非空 PNG）→close，實機通過 |
| stdout 純淨（D-10） | `stdout_purity`：實際二進位 initialize 首行即純 JSON-RPC |
| 部署（D-9） | `cargo install --path crates/browser_mcp --locked` → `C:\Users\stephanie.huang\.cargo\bin\browser-mcp.exe`（6,482,432 bytes） |
| 註冊 | `.kilo/kilo.jsonc` 新增 `browser` 條目（絕對路徑、`timeout: 60000`）；`playwright`（bunx）設 `enabled: false`；設定驗證通過；備份 `.kilo/kilo.jsonc.bak` |
| 全工作區閘門 | `cargo fmt -- --check`=0；`cargo clippy --all-targets -- -D warnings`=0；`cargo test --locked` EXIT=0（全成員 0 failed／0 ignored） |
| 文件同步 | `AGENTS.md` §5、`docs/engineering/SKILL_INVENTORY.md`、`docs/management/營運治理機制.md`、`docs/testing/MVP_TEST_PLAN.md`／`MVP_TEST_REPORT.md`（原 playwright 記載以 SUPERSEDED 就地保留） |

## 待使用者完成的互動步驟（D-7）

1. VS Code → Settings → Web Tools → **關閉**「Browser Automation」（消除內建 `kilo-playwright` 的 npx 連線錯誤）。
2. 重啟 Kilo 客戶端，使 `.kilo/kilo.jsonc` 的 `browser` 條目生效（工具將以 `browser_*` 命名出現）。

## 已知偏離計畫事項

- 工具名以方法名生成：`browser_type_text`（計畫表原列 `type`；避免依賴巨集未驗證的 `name` 覆寫參數）。
- `cat`/`git` 等閘門未含提交：依規則**未執行 `git commit`**（僅於使用者明確要求時提交）。

## 交接

實作已完成；剩餘為上節使用者互動步驟。規劃代理於實作階段後不再變更原始碼。
