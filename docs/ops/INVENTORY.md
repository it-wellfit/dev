# 現況盤點（INVENTORY）— Phase 0（計畫 1793782615513 §17.1）

> 量測時間：2026-11-17 11:45（UTC+8）｜量測者：AI 代理（code）｜git HEAD：`ec3a380`
> 本檔為機器量測結果存檔（C-10：數字皆附量測方法）；埠/process 事後會變動，以 `tools/port-check.ps1` 為準。

## 1. 埠與 Process（量測方法：`Get-NetTCPConnection -State Listen`）

| 埠 | 狀態 | 量測時 process |
|---|---|---|
| 8090 | **無監聽** | — |
| 8091 | 無監聽（本日驗證後停止） | — |
| 8092 | 無監聽 | — |
| 8093-8099 | 無監聽 | — |
| 8095 | 已退役（無監聽） | 歷史：平行流空 DB 測試實例 |
| 8096 | 已退役（無監聽） | 歷史：Tauri 殼 release 舊版（11-15 啟動、11-17 09:11 重啟後停止） |

結論：**8090-8099 全靜默＝乾淨基態**；prod（8090）下次啟動由 `deploy.ps1 -Env prod` 或 17.0 止血命令接管。

## 2. 資料庫檔案（量測方法：`Get-ChildItem data`）

| 檔案 | 大小 | 最後寫入 | 角色 |
|---|---|---|---|
| `data/prod.db` | 430,080 B | 2026-11-17 14:5x | **prod 正式資料**（canonical；遷自 demo-auxbridge.db，原檔保留為來源備份） |
| `data/staging.db` | 430,080 B | 2026-11-17 14:5x | **staging**（prod 副本＋`auxbridge-anonymize` 遮罩：39 交易描述、78 分錄備註、195 稽核、33 sessions 清除） |
| `data/demo-auxbridge.db` | 430,080 B | 2026-11-15 21:00 | 遷移來源備份（prod 前身） |
| `data/auxbridge.db` | 438,272 B | 2026-11-17 10:31 | 平行流使用中（歸屬待其 owner 確認） |
| `data/dev.db` | +WAL | 2026-11-17 14:5x | **dev 環境**（seed-dev.sql 已套用：5 筆樣本交易、借貸平衡） |
| `data/demo.db` | 204,800 B | 2026-11-09 19:22 | 舊示範檔（待清理決策） |
| `platform/data/*` | 各異 | 2026-11-10~13 | 舊測試 DB（測試用，不影響運行） |

## 3. 二進位（量測方法：`Get-Item`）

| 二進位 | 大小 | 建置時間 | 狀態 |
|---|---|---|---|
| `%TEMP%\dev-build-target\debug\auxbridge-server.exe` | — | 2026-11-17（本日 W12b+CI/CD 實作） | **最新**（含 /health /ready BUILD_INFO obs-JSON） |
| `%LOCALAPPDATA%\Flaps ERP\flaps-server.exe` | 6,194,688 B | 2026-11-10 12:50 | **過時**（舊名、無新端點）→ 待重建（Pending P0-NSIS） |
| `web\src-tauri\target\release\auxbridge-server.exe` | — | 2026-11-15（W12，無 W12b） | Tauri 殼內嵌版；殼退役後待重打包 |

## 4. 品質閘門基線（2026-11-17 實測，方法：cargo fmt/clippy/test）

- `cargo fmt -- --check`：**PASS**（exit 0）
- `cargo clippy -p platform -p visual-diff --all-targets -- -D warnings`：**PASS**（exit 0）
- `cargo test -p platform -p visual-diff`：**0 failed**（lib 168 passed；integration 全綠含 page_matrix 4、contract_nav 3、visual-diff 4）

## 5. 本日新增工件

| 檔案 | 用途 |
|---|---|
| `docs/ops/PORTS.toml` | 埠號登記表 SSOT（含退役埠歷史；canonical DB 命名已落地） |
| `tools/port-check.ps1` | 埠衝突檢測（CI + 本地） |
| `platform/build.rs` | BUILD_COMMIT/BUILD_TIME 注入 |
| `/health` `/ready` 路由 + `/api/health` 擴充 | 部署探針（commit/build_time/uptime_s） |
| `platform/src/obs.rs` JSON 格式 | `AUXBRIDGE_LOG_FORMAT=json`（預設不變） |
| `.github/workflows/ci.yml` | CI 骨架（budget-guard + 快路徑降級 + hub-check） |
| `.github/PULL_REQUEST_TEMPLATE.md` | PR 門禁自查 + 設計審核清單 |
| `tools/deploy.ps1` / `tools/rollback.ps1` | 部署/一鍵回滾（/health 驗證 + 失敗自動回滾） |
| `tools/capture-page.ps1` / `tools/capture-baseline.ps1` | CDP 截圖工具（自有 headless Chrome；Text frame + 分幀重組 + 每幀逾時） |
| `tools/visual-diff` crate | 視覺回歸比對（4 tests 綠；sanity：同圖 PASS、異頁 FAIL 實證） |
| `platform/src/bin/auxbridge-anonymize.rs` | staging 匿名化（含 prod 防呆：路徑含 prod 拒絕執行） |
| `platform/tests/contract_nav.rs` + 3 snapshots | 導覽 IA 契約測試（insta） |
| `scripts/seed-dev.sql` | dev 種子（5 筆樣本交易、INSERT OR IGNORE 冪等、雙分錄平衡） |
| `docs/benchmarks/baseline.json` | 效能基線（p95=504ms / SLA 650ms / 42 samples / ec3a380） |
| `tests/visual/baseline/staging-*.png` | 視覺回歸基線 8 頁（1440x900，staging 匿名化資料） |

## 6. Pending（未完成項與原因）

| 項目 | 計畫 Phase | 為何延後 | 重啟條件 |
|---|---|---|---|
| ~~NSIS 安裝版重建＋updater 簽名~~ | ~~P0~~ | ✅ 完成（2026-11-17：金鑰輪換後完整簽名產出——setup.exe 12.9MiB＋.sig（minisign RWS2C2vRYEVC...）＋config pubkey 同步輪換；私鑰 %TEMP%\tauri-signing\tauri.key，密碼 AuxBridge2026，內部部署可接受並已記錄） | — |
| ~~視覺回歸基線 PNG~~ | ~~P3~~ | ✅ 完成（staging 8 頁基線，CDP 自持 headless Chrome） | — |
| ~~criterion 效能基線~~ | ~~P3~~ | ✅ 完成（docs/benchmarks/baseline.json，stress 實測 p95=504ms） | 效能門禁正式化時 |
| ~~seed-dev.sql / auxbridge-anonymize~~ | ~~P2/P3~~ | ✅ 完成（dev.db 已套用；staging.db 已匿名化＋prod 防呆實證） | — |
| ~~data/env.db canonical 命名~~ | ~~P2~~ | ✅ 完成（prod.db/staging.db 落地，demo-auxbridge.db 保留為備份） | — |
| ~~回滾演練 / SLO~~ | ~~P5/P6~~ | ✅ 完成（DRILL_LOG.md 演練 #1：RTO 1 分鐘/RPO 0，PASS；SLO 99.5% 裁定於 docs/standards/SLO_SERVICE_LEVEL_OBJECTIVES.md） | 營運 3 個月後複核 |
| main 分支保護 | P1 | GitHub repo 設定（UI 操作，非檔案） | Repo Admin 依 docs/ops/DRILL_LOG.md 同目錄之操作指引執行 |
