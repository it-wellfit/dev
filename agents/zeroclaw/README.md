# ZeroClaw 免費個人助理（無 Node・Windows 優先・可攜）

本目錄是 ZeroClaw 的**攜帶式 profile**：可版控的設定模板、安裝／驗證／回滾腳本與操作手冊。
**執行期檔（config、workspace、logs、金鑰）一律不進版控**。

為什麼是 ZeroClaw 而不是 OpenClaw：OpenClaw 需要 Node.js 24.16+（本機無 Node），
ZeroClaw 是 Rust 單一執行檔，Windows 有官方 prebuilt zip，**免 Rust 工具鏈、免管理員**。

- 固定版本：**v0.9.0**（不使用 `latest`；以 `install.ps1 -Version` 或 `install.sh --version` 覆寫）
- 模型：**Ollama 本機為主**，Gemini API 免費層為輔（敏感內容僅走本機）
- 通訊：第一階段只有 CLI ＋本機 dashboard（`127.0.0.1:42617`），不接任何聊天軟體
- 常駐：第一階段**不註冊**排程器（`ZeroClaw Daemon`）

---

## 1. 前置條件

| 項目 | Windows | Linux/macOS/WSL2 |
|---|---|---|
| 執行檔 | `zeroclaw-x86_64-pc-windows-msvc.zip`（34.3 MB） | `zeroclaw-<target>.tar.gz` |
| 權限 | 一般使用者即可（寫入 `%USERPROFILE%`） | 一般使用者 |
| Node / Rust | **不需要** | **不需要** |
| 模型 | Ollama 本機服務（`127.0.0.1:11434`） | 同左（或遠端 `uri`） |
| 網路 | GitHub Releases（下載）＋ Google Gemini API（免費層，選用） | 同左 |

本機已有模型（實測）：`qwen3.5:4b`、`llama3.1:8b`、`gemma3:4b`、`qwen2.5:7b-instruct`。
**實際採用哪一個必須實測決定**（見 §4），不得憑印象挑。

---

## 2. 安裝

### Windows（PowerShell 5.1）

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

腳本會：下載 pinned 版本 → 驗 SHA-256（不符即中止）→ 解到 `%USERPROFILE%\.zeroclaw\bin`
→ 加入**使用者** PATH → 由模板渲染 `%USERPROFILE%\.zeroclaw\config.toml`
→ 把 `identity/*.md` 複製到 agent 工作區（護欄檔，見 §5.1）。
冪等：同版本已存在且不帶 `-Force` 時跳過下載；已存在的 config 與身分檔不覆寫。

常用旗標：

```
-Force            重新下載／重新渲染（會覆寫同版本執行檔與 config）
-SkipConfig       只裝執行檔，不產生 config
-RenderOnly       只由模板產生 config（不下載、不改 PATH），供換機或修正設定後重用
-PrimaryModel     本機模型標籤（預設 qwen3.5:4b，實測後再改）
-FallbackModel    免費層模型 ID（預設 gemini-3.8-flash）
```

### Linux / macOS / WSL2

```bash
./scripts/install.sh
```

以 release 的 `SHA256SUMS` 驗證後安裝到 `~/.zeroclaw/bin`，並把該目錄寫進 `~/.profile`。

> macOS 目前沒有 arm64（Apple Silicon）官方資產；若你是 Apple Silicon，WSL2/實體 Linux 或改用
> Docker 映像 `ghcr.io/zeroclaw-labs/zeroclaw:v0.9.0`（不在本計畫範圍，需另開計畫）。

---

## 3. 第一次設定

```powershell
zeroclaw --version                      # 期望 0.9.0
zeroclaw quickstart                     # provider / risk profile / memory / channels
zeroclaw agent -a assistant -m "只回 Ready，不要使用工具"
zeroclaw daemon                         # gateway + dashboard（终生執行，Ctrl+C 結束）
```

`quickstart` 時：**不選任何聊天通道**（第一階段只有 CLI＋dashboard）。
記憶體（memory）若要求 embedding 供應商，先不要下載新模型 → 見 §9 Q1。

### 金鑰（不得寫進檔案）

```powershell
$env:ZEROCLAW_providers__models__gemini__free__api_key = '<key>'
```

或

```powershell
zeroclaw config set providers.models.gemini.free.api_key      # 遮罩輸入
```

金鑰位置：`GEMINI_API_KEY`／`GOOGLE_API_KEY`（免費層，內容會被 Google 用於產品改進）。

---

## 4. 模型選型（實測，不憑印象）

工具：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\measure-models.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\measure-models.ps1 -NoTools   # 只測延遲
```

每個模型以**獨立子行程**跑（同一個長連線下，某些 Ollama build 會對第二個工具請求回 400）。

**實測紀錄（2026-11-15，本機 Ollama，同一 prompt：以 `list_directory` 工具列出 `C:\Windows\Temp`）**

| 模型 | 工具呼叫 | 首次回應（冷） | 暖啟動 | 生成 tokens |
|---|---|---|---|---|
| `qwen3.5:4b` | 是 | 10.4 s | ~2.1 s | 87 |
| `llama3.1:8b` | 是 | 11.9 s | ~11.7 s | 21 |
| `qwen2.5:7b-instruct` | 是 | 11.3 s | ~11.3 s | 24 |

判定：目前 config 採 **`qwen3.5:4b`**（唯一暖啟動後仍在 2 秒內、且工具呼叫穩定）。
配額或行為改變時，重跑上表並更新此段（含量測日期）。

另案：`ollama ps` 可確認模型實際落在 CPU 或 GPU；若全部落在 CPU，先別換大模型。

### 被拒絕後的行為（實測，2026-11-15，重要）

同一 prompt（叫它跑 `Get-Date`，屬中風險 → 需審核；非互動環境無審核者 → runtime 直接拒絕）：

| 模型 | 工具是否執行 | 拒絕後的回應 | 判定 |
|---|---|---|---|
| `qwen3.5:4b` | **否**（runtime 拒絕） | 空回應 → 重試 3 次後回合失敗（`invalid semantic completion`） | 失敗可見，不會騙人 |
| `llama3.1:8b`（**未加身分檔前**） | **否**（runtime 拒絕） | **編造** `Get-Date` 的成功輸出（年份還寫 2023） | 危險：宣稱已執行 |
| `llama3.1:8b`（**加身分檔後**） | **否** | 「No results found.」 | 明顯改善，不再編造 |

證據：`data/state/runtime-trace.jsonl` 的 `approval_gate` 事件 `denied_by_runtime: true`、
`result: "Tool call not executed: ..."`；`turn_final_response` 內容可逐字比對。

結論：**先加身分檔再啟用**（見 §5），且不要用本機小模型無人監督地執行關鍵步驟。

---

## 5. 設定檔重點（`config/zeroclaw.toml.tmpl`）

- `[providers.models.ollama.local]`：`uri` 必須是原生 `http://127.0.0.1:11434`，**不可寫 `/v1`**，
  否則工具呼叫會不可靠（模型會把工具 JSON 當普通文字吐回）。
- `[providers.models.gemini.free]`：`gemini` 是正式 slot（`google` 只是別名）。
- `fallback` 是 **provider alias 上的欄位**：本機失敗才外送到免費層。
- `[agents.<alias>]` 是「join」：指向 provider／risk profile／runtime profile，並擁有 workspace 與 memory。
- `[agents.<alias>.workspace] unrestricted_filesystem = false`：**不可**在本機開啟（Windows 無 OS 沙箱）。
- `[risk_profiles.local]`：`supervised`（中風險需核准、高風險阻擋）、`workspace_only`、
  `forbidden_paths`、`allowed_commands`（**非空即「未列即封鎖」**）。
- `[runtime_profiles.bounded]`：限制工具迭代、歷史訊息與上下文，避免成本與延遲失控。
- `[gateway] require_pairing = true`：`/api/*` 不可對本機任意行程開放。
- `[memory] search_mode = "bm25"`：本方案不裝 embedding 模型，用關鍵字檢索；改回 `hybrid` 前必須先有
  embedding provider（否則只會一直跳警告）。
- `[tools] optional = []`：不開選配工具（縮小曝露面，尤其 Windows 沒有 OS 沙箱）。

### 身分／護欄檔（identity/ → 工作區）

執行期會把工作區根目錄的 `AGENTS.md`、`TOOLS.md`、`SOUL.md`、`IDENTITY.md`、`USER.md`
注入每一次的 system prompt。本 profile 在 `identity/` 收納這五份，安裝時複製到
`%USERPROFILE%\.zeroclaw\agents\<alias>\workspace\`（已存在不覆寫，除非 `-Force`）。

它們的作用不是美化人格，而是**安全護欄**：禁止編造工具輸出、被拒絕要據實回報、
只准動工作區、敏感內容不送雲端。實測把 `llama3.1:8b` 的「編造成功輸出」壓成
「No results found.」（見 §4）。

- 想改語氣或規則：改 `identity/*.md`，重跑 `install.ps1 -Force`（或直接改工作區檔案）。
- 檔案上限每份 20,000 字元；保持簡短，因為它們佔用每次對話的 context。

---

## 6. 驗收（每項只能是 PASS／FAIL／UNKNOWN）

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\verify.ps1 -Full   # 加跑 completions
```

```bash
./scripts/verify.sh
./scripts/verify.sh --full
```

**實測紀錄（2026-11-15，Windows，daemon 啟動中）**

```
[PASS] V1  zeroclaw 0.9.0 (6c78ba5)
[PASS] V2  config at %USERPROFILE%\.zeroclaw\config.toml
[PASS] V3  config validates (valid: true)
[PASS] V4  no inline api_key in config.toml
[PASS] V5  bin on user PATH
[PASS] V6  ollama reachable, 4 model(s)
[PASS] V7  primary model present: qwen3.5:4b
[PASS] V8  gateway health OK (/health → 200; / → 200)
[PASS] V9  no 'ZeroClaw Daemon' scheduled task
[PASS] V10 completion via local model → "Ready"
```

額外人工檢查：`zeroclaw security status --agent assistant` →
`public bind: false`、`pairing required: true`、`workspace-only: true`、`rw roots: 0`、
`Sandbox: requested none, active none`（Windows 無 OS 沙箱，如預期）。

**政策行為實測（2026-11-15，非互動 CLI）**

| 情境 | 要求 | 實測結果 | 判定 |
|---|---|---|---|
| 讀工作區外（`Documents`） | 列目錄 | 模型說自己在 workspace 內、拒絕執行 | 通過 |
| 寫工作區外（`Desktop\zc-probe.txt`） | 寫檔 | 跳審核 → 無審核者 → **runtime 拒絕**；檔案不存在 | 通過 |
| 執行 `shell`（`Get-Date`） | 跑命令 | 跳審核 → **runtime 拒絕**（`denied_by_runtime: true`） | 通過 |
| 讀工作區內 | 讀 `hello.txt` | 低風險自動執行，回覆內容正確 | 通過 |
| 寫工作區內（暫時 `auto_approve=["file_write"]`） | 寫 `probe-approved.txt` | 成功寫入、內容逐字相符 | 通過 |
| 寫工作區內（還原 `auto_approve=[]`） | 寫檔 | 回到「需審核 → 非互動則拒絕」 | 通過 |

→ 未經核准的動作**一律不會執行**；需要無人監督執行特定工具時，才在 risk profile 加
`auto_approve = ["<tool>"]`（測完務必還原成 `[]`）。

| ID | 檢查 | 通過條件 |
|---|---|---|
| V1 | 版本 | `0.9.0` |
| V2 | config 存在 | 檔案在 `%USERPROFILE%\.zeroclaw\config.toml` |
| V3 | config 合法 | `config migrate --json` 回報 `valid: true`，或 `migrated: true` 且 `error: null` |
| V4 | 無內嵌金鑰 | config 內沒有 `api_key = "..."` |
| V5 | PATH | bin 在使用者 PATH |
| V6 | Ollama | `/api/tags` 可回 |
| V7 | 主模型存在 | 該模型在本機 Ollama 內 |
| V8 | Gateway | `127.0.0.1:42617/health` 回 200（未啟動時 → UNKNOWN） |
| V9 | 未裝常駐 | 無 `ZeroClaw Daemon` 排程器 |
| V10 | completions（`-Full`） | 模型真的回 `Ready` |

規則：**`UNKNOWN` 不是 PASS**。凡是 UNKNOWN，必須在交接說明寫清楚「缺什麼才能補測」。

人工補測（腳本無法安全代勞的部分）：

1. 工作區內寫入要跳審核、工作區外寫入被拒、讀 `forbidden_paths` 被拒。
2. 未列於 `allowed_commands` 的命令被阻擋；`;` 串接被歸為高風險。
3. Gateway 只綁回環：本機可連、由另一台機器連不上（無第二台機器時記 UNKNOWN）。

---

## 7. PowerShell 政策限制（會直接影響可用性）

`[runtime] shell = "powershell"` 會以 `-NoProfile -NonInteractive -Command` 執行，且 ZeroClaw 的
PowerShell 政策只接受**有限語法**。以下都被歸類為高風險（＝阻擋或需逐次核准）：

- 陳述式分隔 `;`、子運算式 `$(...)`、script block `{ ... }`
- 重導向 `>`、call operator `&`、內嵌其他解譯器
- 環境變數限定 `$env:NAME`、PowerShell provider 路徑
- 未知或 `Verb-Noun` 形式的命令（預設高風險）

實務上有用的寫法：單一命令、具名參數、簡單-or-引號字串、有限度管線。
需要哪些命令就**逐項實測後**加入 `allowed_commands`（不要憑猜測填）。

---

## 8. 已知問題

| 問題 | 因應 |
|---|---|
| Windows **沒有 OS 級沙箱**（Landlock/Firejail/Seatbelt 皆不適用，`auto` → `none`） | 以 `workspace_only`＋`forbidden_paths`＋命令白名單＋`supervised` 補強；殘餘風險見 §10 |
| 未簽章執行檔可能觸發 SmartScreen | 由 PowerShell 啟動通常不觸發；若觸發，用「解除封鎖」，**不要**關閉防毒 |
| McAfee／企業政策可能攔下載 | 先驗雜湊；被攔時改用官方 MSI 或請 IT 放行 |
| 常駐（Phase 2）會附一個空 console 視窗 | 已知問題；若要用常駐，保留該視窗，且重裝前必須先 Disable 排程器並重開機 |
| `ollama` 誤寫 `/v1` | 工具呼叫不可靠；config 內只准原生路徑 |
| 模型較弱 | 多步工具鏈易誤判；由 V10 與 §4 實測把關，必要時才評估較大模型 |
| `fallback` 指向沒有金鑰的 provider 時**啟動即失敗** | fail-closed 設計；金鑰備妥後再取消註解 `fallback`（實測訊息：`no profile-resolved credential exists`） |
| 同一長連線連發多個工具請求，Ollama 可能回 400 | 每個模型用獨立子行程量測（`measure-models.ps1` 已內建） |
| 非互動環境遇到需要審核的工具 → 直接拒絕（fail-closed） | 這是設計；要互動核准請在真終端機跑 `zeroclaw agent -a assistant`，或用 `auto_approve` 明確放行單一工具 |
| `qwen3.5:4b` 在被拒絕後會吐空回應，重試 3 次後回合以 `invalid semantic completion` 失敗 | 已知模型缺陷；失敗是「可見的失敗」，不會謊報成功。需要連續工具鏈時改用互動模式或較大模型 |
| 小模型可能**編造**工具輸出（llama3.1 未加身分檔前實測） | 一律加 `identity/` 護欄檔（已內建）；關鍵步驟不要無人監督執行 |
| `config patch` 成功訊息會把陣列印成字串 | 顯示問題而已；用 `config list`／直接看 `config.toml` 確認實際值 |

---

## 9. 可攜性與擴充

**可攜（換機器／換 OS）**

1. 把本目錄一起帶走（模板＋腳本＋`identity/` 護欄檔，不含任何密鑰或機器路徑）。
2. 目標機執行對應的 `install.ps1`／`install.sh`（同一 pinned 版本）。
3. 機器差異只透過環境變數：`ZEROCLAW_CONFIG_DIR`、`ZEROCLAW_DATA_DIR`、`ZEROCLAW_*` schema-mirror 覆寫。
4. 文件不得寫死 CPU/GPU/RAM；效能數字一律附量測方法與日期。

**擴充**

- 新增模型：加 `[providers.models.<type>.<alias>]`，並在需要時掛進 `fallback`（不動 agent）。
- 新增通道（Phase 2）：`[channels.<type>.<alias>]`；需要更嚴格表面時，**另開第二個 agent** 綁更嚴的
  risk profile，不要為此放寬既有 profile。
- 成本/延遲：`runtime_profile` 的迭代與上下限、Ollama 的 `num_ctx`／`num_predict`。

**待確認**

- Q1：是否允許 `ollama pull nomic-embed-text`（約 274 MB）以啟用記憶檢索？預設**不 pull**。
- Q2：是否允許 agent 寫入 workspace 以外的暫存路徑？預設**僅 workspace**。
- Q3：Phase 2（Telegram＋常駐）另開計畫？預設**是**。

---

## 10. 安全態樣與殘餘風險

- 模型若被 prompt injection，理論上可以**你的使用者權限**執行命令（Windows 無 OS 沙箱）。
  緩解：workspace 限制、forbidden paths、命令白名單、審核、不啟用瀏覽器／硬體類工具、**不使用**
  `full`／YOLO。
- 免費層會把你的內容用於 Google 產品改進，且額度與速率限制不透明 → 敏感內容僅走本機；
  Dev repo／公司資料不上雲。
- 金鑰只進環境變數或 secrets store；`config.toml`、`*.log` 不得進版控。

---

## 11. 回滾

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1           # 保留設定
powershell -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1 -Purge -Yes   # 連記憶一起刪
```

會：移除 `ZeroClaw Daemon` 排程器（若有）→ 刪 `bin` → 還原使用者 PATH →（`-Purge` 時）刪
`%USERPROFILE%\.zeroclaw`（**會連對話歷史一起刪除**）。

---

## 12. 不在範圍

OpenClaw（Node）安裝；Telegram／Slack／Discord 等通道；常駐服務與開機自啟（Phase 2）；雲端主機；
coding agent 整合；任何付費模型或方案；修改 FlapsErp／ekpi-platform／Wellfit 等子專案原始碼。