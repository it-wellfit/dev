# AGENTS.md — Dev 根工作區代理人執行入口

> 版本：v1.1｜日期：2026-11-10｜狀態：Active
> 本檔為新代理人／開發者的快速執行入口。**規則 SSOT 不在本檔**：憲政與效力順位見 `WORKSPACE_CHARTER.md`，管理體系見 `docs/standards/GOVERNANCE_STANDARD.md`，工程基線見 `docs/standards/SHARED_STANDARDS.md`。本檔僅陳述本地後果與入口連結（P1 單一事實來源）。

## 1. 專案概覽與產品裁定

本工作區包含多個子專案（導航見 `WORKSPACE_CHARTER.md` §1）。2026-11-03 裁定（ADR-0001）：

- **主產品**：`ekpi-platform/`（OEKAP——企業知識與流程智慧平台）。
- **根 workspace**（`core/`、`platform/`、`web/`、`crates/flaps_bridge`）：FlapsErp RPA 支援線。
- **獨立子專案**：`Wellfit/`、`SyncMesh/` 各依其 AGENTS.md 操作。

## 2. 根 workspace 快速命令

```powershell
# 於工作區根目錄執行（PowerShell；串接用分號，禁止 &&）
cargo fmt -- --check                                  ; # 格式（提交前必過）
cargo clippy --all-targets -- -D warnings             ; # 零警告（提交前必過；不得加 allow）
cargo test --locked                                   ; # 全部測試（0 failed / 0 ignored）
cargo audit                                           ; # 依賴安全（.cargo/audit.toml 記錄已接受的風險）
```

- 建置產物若受桌面應用程式控制原則阻擋，改用 `$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\Temp\dev-target"`。
- 技術棧基線與決策見 `docs/adr/ADR-0001-tech-stack-baseline.md`（edition 2021 / axum 0.7 / SQLite，升級需獨立 POC）。

## 3. 硬性規則速查（完整定義見 owner 文件）

| 規則 | Owner |
|------|-------|
| 每個 `.rs` 第一行 `#![forbid(unsafe_code)]`；clippy 零警告 | 憲章 §2.2、SHARED_STANDARDS §1.1 |
| 表現層 100% JavaScript-Free：`<script>` 數 = 0、無內聯事件 | 憲章 §2.3；現行基準 `web/index.html` |
| 金額／數量禁用浮點（現有 f64 技術債見 IMPROVEMENT_BACKLOG P0-1） | SHARED_STANDARDS §1.1、ARCHITECTURE INV-01 |
| 讀取 GET；建立／修改／刪除 POST；成功後 PRG | 憲章 §3 之前的專案約束、`web/index.html` 表單 |
| PowerShell 串接用 `;`；`UNKNOWN ≠ PASS ≠ FAIL`；不編造結果 | 憲章 §2.4–§2.5、GOVERNANCE P7 |
| 不將 Secrets 提交版控；新增依賴前安全審查 | SHARED_STANDARDS §7.1、§2.2 |
| 文件與代碼同一提交更新 | SHARED_STANDARDS §4.3 |

## 4. UI/UX 標準（本工作區現行基準）

- 樣式一律使用 Design Token（`web/assets/design-system.css`；文件快照 `docs/design-system.css`），禁止內聯硬編碼色彩／間距／字體。
- 分區切換用 `:target`、導航高亮用 `:has()`；互動回饋只用 CSS（`:hover/:focus-visible/:user-invalid`）。
- 每個互動元件至少具備 Default／Hover／Focus／Disabled／Loading／Empty／Error 狀態；錯誤訊息需含「問題＋可能原因＋處理方式」。
- 無障礙方向 WCAG 2.2 AA：語義 HTML、可見焦點、鍵盤導航、`prefers-reduced-motion`、不以顏色為唯一傳達。
- 介面文字預設繁體中文（台灣情境）。

## 5. 技能與工具使用政策

- 啟動時先盤點可用 Skills／MCP／代理（清單與使用記錄：`docs/engineering/SKILL_INVENTORY.md`）。
- 可用代理：architect（規劃）、code-reviewer（審查）、code-skeptic（批判驗證）、test-engineer（測試）。
- MCP：context7（官方文件）、playwright（介面驗證）、crates-io（依賴）、rust-analyzer（診斷）等；技能未載入不得假裝已執行。

## 6. Git 工作流（現狀說明）

根工作區**已初始化 git**（2026-11-10 實測：toplevel＝`Dev/`、remote＝`origin`（`github.com/it-wellfit/dev.git`）；ADR-0001 §5.3 之「非 git」記載已成歷史）。子專案（Wellfit 等）各自有 git 規範。提交依 SHARED_STANDARDS §5 執行（`<type>: <subject>`、關聯 Issue、`main` 保護）。

## 7. 接手程序（新 session 標準動作）

1. 讀 `WORKSPACE_CHARTER.md` → `docs/standards/GOVERNANCE_STANDARD.md` §4.5 六步。
2. 查 `docs/PLAN_REGISTER.md` 與相關 ADR（`docs/adr/`）。
3. 執行 §2 快速命令確認當前基線（結果綁定 session 記錄，見 GOVERNANCE §2）。
4. 讀 `.kilo/memories/corrections.md`（強制規則）→ `.kilo/memories/project.md`（事實索引）→ `.kilo/memories/decisions.md`（決策記錄）。
5. 之後才讀程式碼。
