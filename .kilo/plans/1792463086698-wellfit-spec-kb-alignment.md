# Wellfit spec 對齊與知識庫 evolve 計畫

> 計畫編號：1792463086698｜日期：2026-11-12｜狀態：**Active（規劃完成，待執行）**
> 範圍：Wellfit MTRIX spec（`spec.rs`／CLI 匯出）與跨專案知識庫 `2026/Knowledge/` 之**對齊**與**演進**。
> 治理：C-13 Evidence-Driven（A–L＋五級判定）；KB 維護規則（`Knowledge/README.md` §6）；Wellfit `AGENTS.md` §8（契約變更先改 KB、再改程式）。
> 雙 repo：Wellfit 變更 commit 於 **Dev** repo；KB 變更 commit 於 **Knowledge** repo（各自獨立 commit，不混提）。
> 執行邊界：以唯讀核對＋產生器重跑為主；**禁止手改衍生檔**；任何刪除以 SUPERSEDED 歸檔（C-01）。

---

## 0. 完成定義（DoD）

1. `spec.rs` ↔ KB 衍生檔**零漂移**：重跑產生器後 `git diff` 為空。
2. VR／ANOM 目錄與程式一致：`VrRule` = **15**、`REPAIR_RULES` = **12**。
3. `traceability/` 三表與現行程式／測試一致。
4. `issues/mtx-issue-register.md` 狀態有證據更新（含 `resolved`／`accepted-as-is` 標註）。
5. 新增 `specs/bankpay-contract.md`；必要 ADR 補齊（編號續 0013+）。
6. `kb-verify.ps1` 全 PASS、`verify-compliance.ps1` FAIL=0、`rag-index` 重建。
7. 新增 drift gate 並接入 `Wellfit/scripts/check.ps1`（可重跑、失敗即 exit 1）。

---

## 1. 現況證據（2026-11-12 核實）

| ID | 事實 | 證據 |
|---|---|---|
| F1 | `Wellfit/spec/mtrix-schema.json` 與權威 `spec.rs` **嚴重不符**：ALTHCI 宣稱 11 欄（`CUSTOMER_CODE/VENDOR_PREFIX/COUNTRY…`，頁 3），實際 16 欄（`VENDOR_ID/DIST_ID/CUSTOMER_ID/CUSTOMER_NAME/ADRESS…`，p.19/[p23]）；ALTHES/ALTHPD/ALTHVD/ALTHFV 欄名與頁碼亦不符；各檔欄數總和 **36 ≠ 宣稱 56**；`issue_id` 出現非制式 `G-07`/`G-11`（KB 登錄用 `MTRIX-xxx`/`ISS-xxx`/`LOCAL-xxx`） | `spec/mtrix-schema.json` vs `crates/wellfit_mtrix/src/spec.rs`；`Knowledge/specs/mtrix-field-dictionary.md` |
| F2 | 無任何程式／測試／KB 引用 `mtrix-schema.json`（僅舊計畫 `Wellfit/plans/1792486147134-*` 提及） | grep `mtrix-schema`（Dev 與 Knowledge 全掃） |
| F3 | KB `specs/mtrix-vr-rules.md` 標 **VR-01..VR-14（12+2）**、`README.md` §2 亦寫「VR-01..VR-14」；但 `spec.rs` 現為 `VrRule::Vr01..Vr15`（**15**） | `Knowledge/specs/mtrix-vr-rules.md:1-27`；`validators.rs`（Vr01..Vr15）；`Wellfit/AGENTS.md` §2 |
| F4 | KB 衍生檔 `specs/mtrix-field-dictionary.md`（產生時間 2026-10-20T07:52:10Z）與 `reconciliation/field-diff.md`（v1.0 2026-10-20）可能已過期 | 兩檔檔頭 |
| F5 | 工具鏈就緒：`tools/spec-extract/generate-field-dictionary.ps1`、`tools/reconciliation/build-field-diff.ps1`、`tools/kb-verify.ps1`、`tools/rag/build-kb-index.ps1`、`tools/compliance/verify-compliance.ps1`；CLI `wellfit spec --format json\|markdown\|html` | `Knowledge/tools/*`；`wellfit_cli/src/main.rs:171,573,1853` |
| F6 | `Knowledge/` 為**獨立 git repo**（`Knowledge/.git`） | 目錄 |
| F7 |  newer crates 未入 KB：`wellfit_bankpay`（匯出/匯入/HTTP/查詢）、`wellfit_assist`（OEKAP 協助層） | `Wellfit/AGENTS.md` §2；`Knowledge/specs/` 無 bankpay 契約 |

---

## 2. 決策

| ID | 決策 | 理由 |
|---|---|---|
| D-01 | `spec/mtrix-schema.json`：以 `wellfit spec --format json` **重新產生**為 repo 內機讀匯出，並納入 drift gate；若 CLI JSON 形狀不適合，則**刪除＋SUPERSEDED 歸檔**（舊內容先存 `spec/archive/`） | 現檔為死檔且漂移；C-01 禁無證據刪除 |
| D-02 | KB 衍生檔一律**重跑產生器**（`generate-field-dictionary.ps1`、`build-field-diff.ps1`），**禁手改** | README §6 規則 1 |
| D-03 | VR／ANOM 目錄**以程式為準**：VR 補齊至 15（含 VR-15 之範圍／層級／行為／實作／測試）、ANOM 維持 12 並逐條核對 | README §6 規則 1（ID/範圍/層級須與生成檔一致） |
| D-04 | KB 變更 commit 於 Knowledge repo；Wellfit 變更 commit 於 Dev repo | F6 |
| D-05 | 新增 `specs/bankpay-contract.md`，內容以 `bankpay.rs`／`http.rs`／`validators.rs` 為準（22 必填欄、編碼、媒體檔格式、VR-BANKPAY-01..03、查詢規則） | F7；KB 為契約 SSOT |
| D-06 | 新 ADR 編號續 **ADR-0013+**（bankpay 整合治理、spec 匯出治理） | `adr/` 現至 0012 |
| D-07 | drift gate 以 PowerShell 腳本實作，接入 `scripts/check.ps1`（失敗 exit 1） | 與既有 `.ps1` 工具鏈一致（C-11） |
| D-08 | 全程 C-13：每軌道收尾附 artifact，最終 A–L＋五級判定 | 制度 |

---

## 3. 任務清單（有序；無時間估計）

### Track A — spec ↔ KB 對齊

| ID | 任務 | 驗收 |
|---|---|---|
| A1 | 處理 `spec/mtrix-schema.json`：先 `wellfit spec --format json` 比對形狀 → 重生成（或刪除＋歸檔），更新任何引用 | 檔案與 `spec.rs` 一致；`git diff` 可審；無懸空引用 |
| A2 | 重跑 KB 衍生檔：`generate-field-dictionary.ps1`、`build-field-diff.ps1`；diff 對照 committed 版 | 兩檔與 `spec.rs` 一致；差異逐項審閱後 commit（Knowledge repo） |
| A3 | 修 `specs/mtrix-vr-rules.md`：VR-01..VR-14 → **VR-01..VR-15**（補 VR-15 之範圍/層級/結果語意/實作/測試）；同步 `README.md` §2 與任何「14 條」字樣 | 與 `VrRule::all()`（15）一致；`L2_DECLARED`/`MANUAL_DERIVED` 數字正確 |
| A4 | 核對 `specs/mtrix-anomaly-catalogue.md`（ANOM-01..12）與 `repairs.rs::REPAIR_RULES` | 逐條 ID／授權來源／實作位置／測試一致；不一致即修 |
| A5 | 同步 `traceability/field-to-code.md`、`vr-to-test.md`：新增 VR-15、bankpay 規則、近期測試 | 每規則有對應測試 ID；無懸空 |
| A6 | 新增 drift gate：`Wellfit/scripts/spec-drift-check.ps1`（重跑 `spec --format json/markdown`＋兩支 KB 產生器，與 committed 版 diff；不一致 exit 1），接入 `scripts/check.ps1` | 故意竄改 spec.rs 後 gate 失敗；還原後通過 |

### Track B — KB evolve

| ID | 任務 | 驗收 |
|---|---|---|
| B1 | 更新 `issues/mtx-issue-register.md` 狀態：逐項以證據標 `resolved`／`accepted-as-is`／維持 `pending-counterparty`／`open` | 狀態變更可回溯到程式/測試/來源 |
| B2 | 新增 `specs/bankpay-contract.md`（D-05） | 欄位/編碼/媒體格式/規則與程式一致 |
| B3 | 新增 ADR-0013（bankpay 整合與契約邊界）、ADR-0014（spec 匯出與 KB 衍生檔治理，含 `mtrix-schema.json` 裁決） | 格式依 `adr/` 慣例；含背景/選項/理由/後果 |
| B4 | 更新 `traceability/claims-vs-evidence.md`：新增/撤回清單（COM P50、LAA=True、bankpay standalone 等） | 每宣稱附證據/測試 ID |
| B5 | 更新 `README.md`（版本/變更摘要/目錄圖）、Wellfit `docs/knowledge-index.md` | 版本 +1；索引無死鏈 |
| B6 | 重建 `rag-index`（`tools/rag/build-kb-index.ps1`） | 索引涵蓋新檔；`query-kb.ps1` 可檢索 bankpay 契約 |
| B7 | 跑 `kb-verify.ps1`（全 PASS）＋`verify-compliance.ps1`（FAIL=0） | 兩者 exit 0；報告存 `verification/` |

### Track C — 驗證與交付

| ID | 任務 | 驗收 |
|---|---|---|
| C1 | 全鏈重跑：`build.ps1 check/clippy/test`（Wellfit）＋ Track A/B 產生器＋驗證器 | 0 錯誤／0 警告／測試綠；產生器無 diff |
| C2 | A–L 報告（`docs/` 或 KB）＋五級判定 | 每數字附命令/artifact；無 UNKNOWN 冒充 |
| C3 | 更新 Wellfit `CHANGELOG.md` 與 KB 變更紀錄 | 兩 repo 各自記錄 |

---

## 4. 驗證指令集

```powershell
# 0) 建置 CLI（Wellfit 入口；WDAC 需經 build.ps1）
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "build -p wellfit_cli" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

# 1) spec 匯出（比對形狀、重生成）
& "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe" spec --format json
& "$env:LOCALAPPDATA\Temp\wellfit-target\debug\wellfit.exe" spec --format markdown

# 2) KB 衍生檔重跑（於 2026/ 或 Dev/ 依腳本相對路徑）
powershell -ExecutionPolicy Bypass -File "..\Knowledge\tools\spec-extract\generate-field-dictionary.ps1"
powershell -ExecutionPolicy Bypass -File "..\Knowledge\tools\reconciliation\build-field-diff.ps1"

# 3) drift gate
powershell -ExecutionPolicy Bypass -File "Wellfit\scripts\spec-drift-check.ps1"

# 4) KB 驗證
powershell -ExecutionPolicy Bypass -File "..\Knowledge\tools\kb-verify.ps1"
powershell -ExecutionPolicy Bypass -File "..\Knowledge\tools\compliance\verify-compliance.ps1"
powershell -ExecutionPolicy Bypass -File "..\Knowledge\tools\rag\build-kb-index.ps1"

# 5) Wellfit 全驗證
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "clippy --workspace --all-targets -- -D warnings" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" -Command "test --workspace --no-fail-fast" -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

---

## 5. 風險與回滾

- **重跑產生大 diff**（格式/排序差異）：逐項審閱後 commit；不得以「diff 太大」為由跳過。
- **雙 repo 混淆**：Wellfit 改於 Dev repo、KB 改於 Knowledge repo；不得交叉 commit。
- **CLI JSON 形狀不符** `mtrix-schema.json` 舊形狀：改走刪除＋`spec/archive/` 歸檔（C-01）。
- **衍生檔被手改**：gate（A6）會抓到；還原以產生器重跑為準。
- **build/WDAC**：CLI 執行一律經 `build.ps1`（桌面路徑執行檔可能被應用程式控制原則阻擋）。
- ** rollback**：各_track 變更以 git 還原；KB 衍生檔以重跑產生器還原。

---

## 6. 範圍外（明確）

FlapsErp config 優化（另計畫 `1792463086698-flaps-erp-completion-plan.md`）；ekpi-platform；原始 PDF/DOCX 重抽取（除非 A2 顯示來源需更新）；ISO 27001 合規內容變更（除非 `verify-compliance` 要求）；Wellfit 生產批次/報表外移（另計畫）。

---

## 7. 未決（實作者依證據裁定；附建議）

| 項目 | 建議 |
|---|---|
| `mtrix-schema.json` 保留或刪除 | 優先以 `spec --format json` 重生成；形狀不符則刪除＋歸檔 |
| VR-15 正式名稱/範圍 | 由 `spec.rs` 的 `VrRule::Vr15` 判別後補入目錄（勿臆測） |
| bankpay 契約是否納入 VR 編號體系 | 建議沿用 `VR-BANKPAY-xx` 前綴，與 MTRIX `VR-xx` 分域 |

---

*本計畫僅規劃；衍生檔一律重跑產生器，禁手改；未量測標 UNKNOWN；UNKNOWN ≠ PASS ≠ FAIL。*