# WEB3 Project Workspace — 最高憲政工程總綱與生命週期導航中樞
## (Workspace Supreme Constitutional Codex & Multi-Project Routing Hub)

**文件路徑**: `GEMINI.md` (Workspace Root Rule & Primary Execution Context)  
**所屬體系**: Antigravity 2.0 Workspace Rules ⨁ Tri-Stratum Quality Science (v36.0)  
**最高指示者**: 創辦人暨首席架構師 陳彥邦 先生  
**核心哲學**: **漸進式揭露 (Progressive Disclosure) ⨁ 剛性邊界 (Rigid Invariants) ⨁ 控制反轉 (IoC) ⨁ 精準導引 (Surgical Routing) ⨁ 邊界內充分自主 (Bounded Autonomy)**  
**生效範圍**: 本工作區根目錄 (`WEB3/project`)、所有子專案與所有代理人 (Agent)

---

## 🏛️ 1. 工作區子專案權威導航矩陣 (Multi-Project Navigation Matrix)

本工作區目前可直接辨識的產品／工程目錄如下。此表只描述可在本機找到的範圍，不代表功能、法規符合性或發布狀態已驗收：

跨專案文件權威、計畫重複與未決決策的索引見 [`docs/PLAN_REGISTER.md`](docs/PLAN_REGISTER.md)；該登錄不取代各專案的規格或驗收證據。

| 目錄 | 專案定位與主要入口 | 文件記載狀態／驗收邊界 |
| :--- | :--- | :--- |
| 工作區根目錄 | ARES Rust workspace：`Cargo.toml`、`core/`、`platform/`、`web/`；規格與計畫見 `docs/ARES_MASTER_SPEC.md`、`docs/ARES_IMPLEMENTATION_PLAN.md` | 文件標示 Planning；實際完成度須以當前程式碼與測試證據判定 |
| **`ekpi-platform/`** | EKPI Platform；入口 `PROJECT_CHARTER.md` 與 `docs/README.md` | 憲章記載 Phase A 未核准、Phase B 受阻；不得宣稱 production-ready |
| **`Wellfit/`** | 七 crate workspace；入口 `README.md`、`docs/project/CURRENT_STATE.md`、`docs/DOCUMENTATION_MAP.md` | Current State 是 2026-10-25 的歷史驗證快照；銀行介接模式仍有文件衝突 |
| **`SyncMesh/`** | SyncMesh Rust workspace；入口 `PROJECT_CHARTER.md`、`docs/` | 憲章中的完成／效能宣稱尚未由本次審核獨立驗證 |

支援／研究／上游目錄包括 `agentre-bench/`、`oekap/`、`re-toolchain/` 與 `rea-rust-upstream/`，須按各自入口文件辨識用途；`archive/`、`kilo-settings-skills-archive/` 與 `skills.backup-*/` 是歷史／封存內容，不是現行產品計畫來源。不要由目錄名稱推定產品狀態或跨專案依賴。

---

## ⚖️ 2. 全工作區剛性不可變守則 (Workspace Rigid Invariants)

以下守則適用於工作區內所有子專案、所有程式語言與所有代理人操作，絕對不可逾越：

### 2.1 創辦人最高公理：反過早固化與可決定性最小固化 (Anti-Premature Ossification Axiom)
- **警惕偽固化之危**：嚴禁在缺乏數學形式化證明或物理剛性界限下，將啟發式門檻（如排隊長度、重試退避、排程偏好）硬編碼寫死為固化不變量。
- **合法固化準繩**：僅有宇宙大定理（李雅普諾夫收斂、熱力學矽結溫極限、密碼學單向雜湊）與局部封閉問題的可決定性最佳解（法定稅務整數元四捨五入、盲索引雙射）始得固化。
- **核硬殼軟 (Hard Core, Adaptive Shell)**：剛性大定理為核，自適應動態調諧為殼。

### 2.2 記憶體安全與編譯零容忍 (Rust Memory Sovereignty)
- 所有 Rust 微內核原始碼 (`crates/**/src/**/*.rs` 及測試代碼) 第一行強制宣告 `#![forbid(unsafe_code)]`，零 `unsafe` 區塊容忍。
- `cargo clippy --workspace --all-targets -- -D warnings` 編譯期絕對零警告。禁止 `assert!(true)` 偽斷言，必須綁定具名變數。

### 2.3 表現層 100% JavaScript-Free (Pure CSS Zero-JS Surface)
- 客戶端展示層優先採用語義化 HTML5 與現代 CSS（CSS `:has()`、Popover API、Anchor Positioning），全站 `<script>` 數量恆等於 0。
- 物理層面杜絕 XSS 向量，消滅 V8 引擎 JIT 暖機負擔與 GC 記憶體毛刺。

### 2.4 微軟 Windows PowerShell 執行剛性邊界 (Windows Execution Invariants)
- **指令串接分號律**：PowerShell 多指令串接必須使用分號 (`;`)，嚴禁使用 Unix 風格之 `&&`。
- **二進位檔案鎖定防護 (OS Error 5)**：Windows 會鎖定執行中的 `.exe` 檔案。在執行 `cargo build --release` 前，必須先終止殘留之後台守護程序 (`manage_task kill` 或 `Stop-Process`)。
- **無介面 HTTP 探針律**：避免使用未加 `-UseBasicParsing` 的 `Invoke-WebRequest`，優先調用 Windows 原生 `curl.exe`。
- **複雜腳本檔案封裝**：跨多行之斷言與遙測檢測，一律寫入獨立 `.ps1` 檔案調用，禁止在命令列使用脆弱的內聯字串插值。

### 2.5 絕對反模擬與實證真理源 (Absolute Anti-Simulation Ban)
- 嚴禁偽造或硬編碼掃描結果、漏洞證明或固定金額損失（如 "$38,400 USD"）假冒即時情報。
- 探針失敗（如 HTTP 403, 429, Timeout）必須如實回報技術失敗，絕對不可回退為偽造之成功資料。`UNKNOWN != PASS` 且 `UNKNOWN != FAIL`。

### 2.6 規則包容壓縮協議 (Subsumption Compression Protocol ≤ 20)
- 本工作區規章條款總數嚴格限制 $\le 20$ 條。新增規則時必須強制執行包容性合併，向上抽象為更高階的正向模式，杜絕零碎負向補丁。

---

## 🧭 3. 複合指令與自適應工作流導航 (Adaptive Workflow Navigation)

任何工程任務啟動時，代理人應透過以下標準工作流推進：
1. **任務探索與評估**：使用 `/workflow` 評估任務複雜度與單雙向門影響半徑。
2. **需求拷問與消歧**：使用 `/grill-me` 或 `/grill-with-docs` 消除規格歧義，形成決策共識。
3. **規格合成與工單切分**：使用 `/to-spec` 合成 RFC 規格，使用 `/to-tickets` 切分 DAG 工單（1 Ticket = 1 Context Window）。
4. **實作與紅綠重構**：使用 `/implement` 與 `/tdd` 循序落實代碼，保持 100% 測試通過與零警告。
5. **雙軸代碼審查與驗證 PR**：使用 `/code-review` 與 `/pr` 進行行為與架構雙盲審查，附帶形式化回滾證明。
6. **大一統全生命週期**：重大里程碑啟用 `/diamond-sdlc-apex`，貫徹 8 大主門禁綠燈閉環。

---

## 🌐 4. 跨專案整合狀態

舊版聯邦路線圖、驗收腳本、Schema 與 orchestrator 宣稱未在目前工作區證實，且其列出的產品目錄與 §1 實際盤點不符；已移除此失效細節。各專案目前按自身入口與契約獨立治理。未來若要整合，先取得各專案負責人核准並建立可驗證的介面契約與驗收證據，不得把舊路線圖視為已核准或已完成。
