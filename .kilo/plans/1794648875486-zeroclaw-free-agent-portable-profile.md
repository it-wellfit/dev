# 計畫：ZeroClaw 免費個人助理（無 Node、Windows 優先、可攜可擴充）

日期：2026-11-14｜狀態：待審核（refine r2）｜範圍：本工作區（Dev）＋可轉移至 Windows 10/11、Linux、macOS

## 1. 目標與已裁定決策

目標：在**不安裝 Node.js** 的前提下，用**零 token 成本**建置聊天／排程型個人助理 agent；設定可攜（Windows 先行、可轉移）、安全預設、可驗收、可回滾。

| # | 決策 | 依據 |
|---|---|---|
| D1 | 不採用 OpenClaw（TypeScript/Node 24.16+；此機無 Node） | 使用者指示；`docs.openclaw.ai/zh-CN/install` |
| D2 | 採用 **ZeroClaw v0.9.0**（Rust 單一執行檔、MIT OR Apache-2.0） | Windows 有 prebuilt zip，**免 Rust 工具鏈、免管理員** |
| D3 | 用途＝聊天／排程助理；第一階段**只開 CLI＋本機 dashboard** | 使用者裁定 |
| D4 | 模型＝**Ollama 本機為主、Gemini 免費層為輔**（敏感內容僅走本機） | 使用者裁定；免費為硬底 |
| D5 | 隔離＝**Windows 原生＋嚴格應用層政策** | 使用者裁定；Windows 無 OS 級沙箱（§6） |
| D6 | 攜帶式 profile 放 **`Dev/agents/zeroclaw/`**；**密鑰與 instance 資料不進版控** | 使用者裁定 |
| D7 | 第一階段**不裝常駐服務**（不註冊排程器）；需要時手動執行 | 由 D3 推導；常駐列入 Phase 2 |
| D8 | 版本**固定 v0.9.0**，不使用 `latest`；允許 `ZEROCLAW_VERSION` 覆寫 | 使用者裁定 |

現地盤點（唯讀實測，2026-11-14）：無 `C:\Program Files\nodejs`；無 Docker；有 `wsl.exe`；`ai.opencode.desktop`、VS Code＋Copilot、Kilo 已存在；Ollama 已裝（`%LOCALAPPDATA%\Programs\Ollama`），既有模型 `qwen3.5:4b`、`llama3.1:8b`、`gemma3:4b`、`qwen2.5:7b-instruct`；有 PostgreSQL 18、McAfee、ESUN EES Service（企業機）→ 不採用需要 PostgreSQL 的 IronClaw。

## 2. 為什麼不是其他「更好用的 agent」

- 聊天／排程型助理：OpenClaw 生態最大（22+ 通道），但**移除不掉 Node** → 排除。
- 非 Node 同級：**ZeroClaw**（Rust、<5 MB RAM 級、30+ 通道、~20 provider、預設 `supervised` 審核、工具結果附 receipt）＝選定；IronClaw（需 PostgreSQL，與主 DB 重疊）、PicoClaw（Go，功能極簡）、NanoClaw（需 Docker）→ 不採用。
- coding agent（OpenCode／Kilo／Copilot）不在範圍：你已有。

## 3. 交付物（`Dev/agents/zeroclaw/`）

```
agents/zeroclaw/
  README.md                     # 跨平台 runbook（不得寫死 CPU/GPU/RAM 規格）
  .gitignore                    # instance/、*.env、*.log、config.toml（實際）
  config/zeroclaw.toml.tmpl     # 攜帶式模板（無密鑰、無機器專屬絕對路徑）
  scripts/install.ps1           # Windows：抓版本→驗 SHA256→解→設 PATH→渲染 config
  scripts/install.sh            # Linux/macOS/WSL2：tar.gz + SHA256SUMS 驗證
  scripts/verify.ps1            # Windows 驗收（§6）
  scripts/verify.sh             # Unix/WSL2 驗收
  scripts/uninstall.ps1         # 回滾（§7）
```

規則：
- 版本與雜湊寫進腳本常數：`ZEROCLAW_VERSION=0.9.0`；Windows zip 預期 SHA256 `f81fd10b8663c22b3f7c66e622f5f7249a2404a6973c32a5bd7f4af86a2afd6d`（來源：release v0.9.0 asset digest）。Unix 一律以官方 `SHA256SUMS` 驗證，不寫死。
- 機器相關值只從環境變數注入：`ZEROCLAW_CONFIG_DIR`（預設 `%USERPROFILE%\.zeroclaw`）、`ZEROCLAW_DATA_DIR`、`ZEROCLAW_providers__models__gemini__free__api_key`（schema-mirror 覆寫，**不落地到 config**）。
- 密鑰：`zeroclaw config set <path>`（遮罩輸入）或 secrets store（`~/.zeroclaw/secrets`）；**永不進版控、永不寫進文件**。
- 不新增 Python 工件（Rust-first）；PowerShell/bash 僅作為安裝與驗收腳本。
- 每項效能數字附量測方法與日期；不能測者標 `UNKNOWN ≠ PASS`。

## 4. 設定檔內容（模板要長這樣）

```toml
schema_version = 4

[runtime]
shell = "powershell"          # 此機無 pwsh 7；固定 5.1 以得確定行為
# kind = "native"

[providers.models.ollama.local]
model = "<由實測選定>"          # 見 §5 步驟 6
uri = "http://127.0.0.1:11434"  # 不可加 /v1：會破壞工具呼叫
num_ctx = 8192                   # 依記憶體／VRAM 實測結果調整後記
num_predict = 1024
fallback = ["gemini.free"]       # 本機失敗才外送；fallback 是 provider alias 的欄位

[providers.models.gemini.free]        # gemini = 正式 slot；google 只是別名
model = "<以 API 列出後選定>"
fallback_models = ["gemini-3.5-flash-lite"]
timeout_secs = 60
# api_key 由 env override 注入

[agents.assistant]
model_provider = "ollama.local"
risk_profile = "local"
runtime_profile = "bounded"

[agents.assistant.workspace]
unrestricted_filesystem = false
read_memory_from = []

[agents.assistant.memory]
# backend 預設 sqlite；一旦寫入即鎖定，之後不可換
# 若需要 embedding 供應商 → 見 §10 Q1

[risk_profiles.local]
level = "supervised"
workspace_only = true
allowed_roots = ["<WORKSPACE>"]
forbidden_paths = ["<DEV_REPO_ROOT>", "<USER_DOTFILES_DIR>", "<CORP_DATA_DIR>"]
allowed_commands = []               # 空＝沿用預設政策；非空即「未列即封鎖」，實測後逐項填入
sandbox_enabled = false             # Windows 無 OS 後端，如實關閉並記錄殘餘風險
sandbox_backend = "none"
shell_env_passthrough = []
excluded_tools = []                 # 至少排除瀏覽器／硬體類工具
```

要點：
- `[agents.<alias>]` 是「join」：指向 provider／risk profile／runtime profile，並擁有 workspace 與 memory。
- `[runtime] shell="powershell"` 會走 `-NoProfile -NonInteractive -Command`；**PowerShell 政策只接受有限語法**：`;` 串接、`$(...)`、管線中的括號、重導向、call operator `&`、script block、`$env:X`、內嵌解譯器一律被歸類高風險（=阻擋或需逐次核准）。→ 對你的習慣（用 `;` 串接）**是硬限制**，必須在 runbook 寫清楚可用命令形式，並用 §6 V8b 實測。
- `allowed_commands` 非空即「未列即封鎖」；逐項加入實測可用且無破壞性的 cmdlet。
- `runtime_profile` 用來**限制成本與延遲**：`max_tool_iterations`、`max_history_messages`、`max_context_tokens` 設保守值（例如 8／40／32000），實測後再調。
- 通電前先確認 gateway 綁定位址與配對：以 `zeroclaw config schema`／`config list` 查 `gateway.bind`（須為回環）與 `gateway.require_pairing`（存在則設 `true`）；無法確認時在 README 標 `UNKNOWN` 並列為 Phase 2 必辦。

## 5. 執行步驟（依序，每步有產出）

1. 建立 `agents/zeroclaw/` 骨架與 `.gitignore`；確認 Dev repo 不會誤收 `config.toml`、`*.log`、任何金鑰。
2. 寫 `install.ps1`：下載 v0.9.0 `zeroclaw-x86_64-pc-windows-msvc.zip`＋`SHA256SUMS` → 驗雜湊（不符即 `exit 1`）→ 解到 `%USERPROFILE%\.zeroclaw\bin` → 加入**使用者** PATH（`[Environment]::SetEnvironmentVariable('Path',...,'User')`，免管理員）→ 印出下一步（**不**自動跑 quickstart）。冪等：已安裝同版且已在 PATH 即跳過下載。
3. 寫 `install.sh`：`zeroclaw-<target>.tar.gz`＋`SHA256SUMS`，`sha256sum -c` 通過才安裝到 `~/.zeroclaw/bin`。
4. 寫 `uninstall.ps1`：`zeroclaw service stop` → `zeroclaw service uninstall`（若曾安裝）→ 刪 `%USERPROFILE%\.zeroclaw\bin` → 可選刪 `%USERPROFILE%\.zeroclaw`（會連歷史一起刪，需明確 `-Purge` 旗標）→ 還原使用者 PATH。
5. 渲染 config：模板 → `%USERPROFILE%\.zeroclaw\config.toml`；僅替換 `<WORKSPACE>` 等佔位符；金鑰不寫入。
6. **模型選型（實測，不憑印象）**：對 `qwen3.5:4b`、`llama3.1:8b`、`qwen2.5:7b-instruct` 各跑同一組 3 個工具任務（讀檔、寫檔、列目錄），記錄「工具呼叫成功率／首次回應時間／`ollama ps` 實際 CPU-GPU 配置」。預設序位 `qwen3.5:4b` → `llama3.1:8b` → `qwen2.5:7b-instruct`；結果與日期寫進 README。
7. **Gemini 免費層接線**：以 `GEMINI_API_KEY` 走 `generativelanguage.googleapis.com/v1beta` 列出可用模型 → 選免費層可用者（預設 `gemini-3.8-flash`，備援 `gemini-3.5-flash-lite`）→ 用 `zeroclaw agent -a assistant -m "只回 Ready，不要使用工具"` 實測一次（**必須真的 completions 成功**，設定檔存在不算通過）。
8. 跑 `zeroclaw quickstart` 補完 memory／channel（第一階段不選任何聊天通道）→ 再跑一次 §6 全部驗收。
9. 寫 README：指令、預期輸出、PowerShell 政策限制、已知問題（服務 console 視窗、SmartScreen、重裝排程器需先 Disable＋重開機）、回滾步驟、量測數據。
10. 提交：文件與腳本同一提交，`<type>: <subject>`；確認 `git status` 無金鑰、無 instance 檔。

## 6. 驗收基準（全 PASS 才可交接；不能測者標 UNKNOWN 並註明補測條件）

| # | 檢查 | 方法 | 預期 |
|---|---|---|---|
| V1 | 版本 | `zeroclaw --version` | `0.9.0` |
| V2 | 完整性 | `Get-FileHash` vs `SHA256SUMS` | 相符 |
| V3 | 設定合法 | `zeroclaw config validate`（若無此子命令，用 `zeroclaw config migrate --json` 回報 `valid: true`） | 無錯誤、無 dangling fallback |
| V4 | 本機模型 completions | `zeroclaw agent -a assistant -m "一句話說明你能做什麼"` | 有回覆、`ollama ps` 顯示模型已載入 |
| V5 | 工作區內寫入需審核 | 要求寫 `<WORKSPACE>/probe.txt` | 跳審核；核准後檔案存在 |
| V6 | 工作區外寫入被拒 | 要求寫 `C:\Windows\probe.txt` | 阻擋 |
| V7 | forbidden_paths | 要求讀 `Dev/` 與使用者 dotfiles | 阻擋 |
| V8 | 命令白名單 | `allowed_commands` 填入實測可用命令後：(a) 未列命令 (b) `Get-ChildItem a; Get-ChildItem b` | (a) 阻擋 (b) 高風險（需核准或阻擋）→ 記錄實際行為 |
| V9 | Gateway 僅回環 | 本機 `curl http://127.0.0.1:42617/health`；由另一台機器連同埠 | 本機通、外部不通；無第二台機器 → **UNKNOWN** |
| V10 | 免費層回退 | 停掉 Ollama 後再問一句 | 走 `gemini.free` 並回覆；未設金鑰 → **UNKNOWN** |
| V11 | 常駐未安裝 | `schtasks /Query /TN "ZeroClaw Daemon"` | 不存在（Phase 1 預期） |
| V12 | 效能量測 | 冷/熱首次回應、`ollama ps`、成本帳 | 記錄數值＋方法＋日期（不設主觀門檻） |
| V13 | 回滾演練 | 執行 `uninstall.ps1` | PATH 還原、bin/`~/.zeroclaw` 依旗標處理、無殘留排程器 |

## 7. 風險與殘餘風險

- **Windows 無 OS 級沙箱**（Landlock/Firejail/Seatbelt 皆不適用；`auto` → `none`）：模型若被 prompt injection，理論上可以**你的使用者權限**執行命令。緩解：`workspace_only`＋`forbidden_paths`＋命令白名單＋`supervised` 審核＋不啟用瀏覽器／硬體工具＋**不**使用 `full`／YOLO。
- **PowerShell 政策限制**：`;`、`$()`、重導向、`$env:`、`&` 等被歸為高風險 → 會降低可用性；需在 runbook 記錄「可用命令寫法」，必要時才逐項加白名單。
- **McAfee／企業政策**可能攔未簽章執行檔或下載：先驗雜湊；被攔時改用官方 MSI 或請 IT 放行，**不得**關閉防毒。SmartScreen 以「解除封鎖」處理。
- **Gemini 免費層**：內容會被 Google 用於產品改進、額度與速率限制不透明且可能變動 → 敏感內容僅走本機；Dev repo／公司資料不上雲。
- **常駐（Phase 2）**：Windows 只有使用者層級排程器，且已知會附一個空 console 視窗；重裝前必須先 `schtasks /Change /TN "ZeroClaw Daemon" /Disable` 並重開機。
- **本機模型較弱**：多步工具鏈易誤判 → 由 V4/V12 實測把關；必要時才評估較大模型（需核准下載）。

## 8. 可攜性／可擴充性設計

- 可攜：同一份模板＋`install.sh` 可在 Linux/macOS/WSL2 重現；機器差異只進環境變數；文件不得寫死 CPU/GPU/RAM。
- 可擴充：新增模型＝加 `[providers.models.<type>.<alias>]`（不動 agent）；新增頻道＝`[channels.<type>.<alias>]`（Phase 2）；需要更嚴格表面時，另開第二個 agent 綁更嚴的 risk profile（不要為此放寬既有 profile）。
- 可靠：fallback 鏈（ollama → gemini）、`timeout_secs`、審核逾時視為拒絕、`/health` 監控、`data/state/costs.jsonl` 成本帳、`service logs`。
- 效率：`num_ctx`／`num_predict` 限制、Ollama keep-alive、runtime profile 迭代與上下文上限。

## 9. 不在範圍

OpenClaw（Node）安裝；Telegram／Slack／Discord 等通道；常駐服務與開機自啟（Phase 2）；雲端主機；coding agent 整合；任何付費模型或方案；修改 FlapsErp／ekpi-platform／Wellfit 等子專案原始碼。

## 10. 待確認（實作者遇到再問；不阻塞）

- Q1：是否允許 `ollama pull nomic-embed-text`（約 274 MB）以啟用記憶檢索？預設**不 pull**，改用 `markdown` memory。
- Q2：是否允許 agent 寫入 workspace 以外的暫存路徑？預設**僅 workspace**。
- Q3：Phase 2（Telegram＋常駐）是否要另外開立計畫？預設**是**，本計畫不涵蓋。