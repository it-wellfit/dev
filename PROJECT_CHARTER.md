# 專案章程（Project Charter）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜適用範圍：`Dev/` 根 workspace（core／platform／web／crates/*）｜狀態：Active
- 上層規則：`WORKSPACE_CHARTER.md`（憲政）、`AGENTS.md`（執行入口）、`docs/standards/*.md`（管理與工程標準）
- 本檔依 `docs/standards/SHARED_STANDARDS.md` §4.1 第 1 項建立；內容不重複規則條文，只連結 SSOT。

## 1. 目的

提供「輔翼 ERP（原廠 FlapsErp）周邊」之自建整合工作台：**不修改原廠產品**，僅以 RPA／設定／資料整合方式
串接，並提供企業知識與流程智慧平台（OEKAP，位於 `ekpi-platform/`）。

## 2. 產品身份邊界（不可混淆）

| 項目 | 原廠 | 自建 |
|---|---|---|
| 名稱 | FlapsErp（輔翼 ERP） | **AuxBridge／整合工作台** |
| 可改性 | 僅 config／深度串接，禁改內容 | 全權（本 repo） |
| 二進位／env | — | `auxbridge-server`、`AUXBRIDGE_*` |

裁定 SSOT：`docs/standards/NAMING_AND_BOUNDARY_DECREE.md`；防呆測試 `platform/tests/naming_guard.rs`。

## 3. 目標（可驗收）

1. 零 JavaScript 之伺服器渲染工作台（`platform/`）＋桌面殼（`web/src-tauri/`）＋靜態知識台（`crates/kem`）。
2. 單一 Console Manifest 為頁面/導覽 SSOT（`crates/console_manifest`）。
3. 可稽核之交付鏈：SBOM、簽章、更新 feed、弱點與授權政策（`docs/release/UPDATER_RUNBOOK.md`）。
4. 所有變更具證據：測試、量測、文件同步（`docs/standards/GOVERNANCE_STANDARD.md`）。

## 4. 非目標

- 不重新實作原廠 ERP 會計／報表邏輯（僅對接其資料）。
- 不引入 Node/npm 前端工具鏈（見憲章 §2.3）。
- 不以付費 AI 為必要依賴（免費优先原則；付費僅經授權層觸發）。

## 5. 主要利害關係人與入口

| 角色 | 入口 |
|---|---|
| 新代理/開發者 | `AGENTS.md`（§7 接手程序） |
| 變更計畫 | `docs/PLAN_REGISTER.md` |
| 架構決策 | `docs/adr/` |
| 發行 | `docs/release/UPDATER_RUNBOOK.md` |
| 風險與缺口 | `docs/GAPS_AND_RISKS.md` |

## 6. 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 依 SHARED_STANDARDS §4.1 建立；內容連結既有 SSOT，不重複條文 |