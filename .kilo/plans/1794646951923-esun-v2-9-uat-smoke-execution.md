# E.Sun V2.9 UAT 冒煙測試執行計畫（uat_smoke）

**文件編號：** 1794646951923-esun-v2-9-uat-smoke-execution  
**版本：** 1.0  
**日期：** 2026-11-16  
**狀態：** 準備就緒，等待實作代理人執行  
**相依計畫：** `.kilo/plans/1794646196379-esun-v2-9-upgrade-plan.md`（W1-W5 已完成，腳本已備）

---

## 1. 目的

在取得 E.Sun UAT 環境與約定帳號後，執行 `crates/wellfit_bankpay/tests/uat_smoke.rs`（9 支 V2.9 API），產出 Evidence-Driven A–L 報告，判定 PASS／CONDITIONAL PASS／BLOCKED／REJECTED／NOT VERIFIED。

## 2. 環境與帳號（自 `玉山串接專案/郵件紀錄.txt`）

| 項目 | 值 | 備註 |
|------|-----|------|
| UAT 元件位址 | `http://219.87.16.166:8443` | V29.txt:87-89，連外 port 8443 |
| 防火牆外部 port | 9443 | V29.txt:85 |
| ClientID | `3c37a6988dd8dfaf9af5d8fb7f03b4df` | 元件啟用用 |
| 統一編號 | `68001672` | 企業集團統編 |
| 台幣帳號 | `9915940107636` | 查詢用 |
| 查詢區間 | 2026/01/01–2026/06/30 | 台幣查詢 |
| EES Service 執行檔 | `C:\Users\stephanie.huang\AppData\Local\Programs\ESUN_EES_Service\ESUN_EES_Service.exe` | 本機安控元件 |
| 啟用碼 | 7 天效期，由銀行依電腦提供 | **缺，需確認** |

## 3. 前置條件（全部 ✔ 才可開始）

| # | 檢查項 | 驗證方式 | 狀態 |
|---|--------|----------|------|
| C-01 | UAT 元件位址與連線埠書面確認 | 帳戶經理書面回覆（郵件紀錄已有） | ✔ 已有（待啟用碼） |
| C-02 | 防火牆出向規則開通（port 9443） | `Test-NetConnection 219.87.16.166 -Port 9443` | 待執行 |
| C-03 | 約定帳號／統編資料（ESUN_CUSTOMER_ID 等） | 郵件紀錄：統編 68001672、台幣帳號 9915940107636 | ✔ 已有 |
| C-04 | 系統時間同步（NTP，誤差 < 1s） | `w32tm /query /status` | 待執行 |
| C-05 | 執行授權：負責人書面核可本次 UAT 連線 | 授權紀錄 | 待核可 |
| C-06 | 本 crate 建置／測試基線綠（clippy 零警告＋484/0/19） | `build.ps1` 執行 clippy + test | ✔ 2026-11-15 已實測 |
| C-07 | EES Service 元件啟動與埠監聽 | 確認 `ESUN_EES_Service.exe` 運行且 8443 可連 | 待執行 |
| C-08 | 啟用碼取得並設定至元件 | 銀行提供，效期 7 天 | **缺，必須先取得** |

## 4. 執行步驟（實作代理人）

### 步驟 0：基線確認（不連網）

```powershell
cd "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev\Wellfit"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "clippy --workspace --all-targets -- -D warnings" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"

powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test --workspace --no-fail-fast" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

任一失敗 → 修正後重跑；不得帶著紅基線連線。

### 步驟 1：防火牆與時間確認

```powershell
Test-NetConnection 219.87.16.166 -Port 9443
w32tm /query /status
```

### 步驟 2：啟動 EES Service 元件

確認 `C:\Users\stephanie.huang\AppData\Local\Programs\ESUN_EES_Service\ESUN_EES_Service.exe` 已啟動，且本機 8443 埠可監聽（UAT 元件 port，V2.9 對調）。

### 步驟 3：設定環境變數（Master gate）

```powershell
$env:ESUN_UAT_BASE_URL = "http://219.87.16.166:8443"
$env:ESUN_CUSTOMER_ID  = "68001672"
$env:ESUN_PAYER_ACCOUNT = "9915940107636"
$env:ESUN_PAYER_ID     = "68001672"
$env:ESUN_EVIDENCE_DIR = "artifacts/uat-smoke/20261116"
# 負責人書面核可後才開啟：
$env:ESUN_ALLOW_PAYMENT = "1"
```

### 步驟 4：執行冒煙測試

```powershell
powershell -ExecutionPolicy Bypass -File "..\..\Knowledge\tools\dev-env\build.ps1" `
  -Command "test -p wellfit_bankpay --test uat_smoke -- --ignored --nocapture" `
  -TargetDir "$env:LOCALAPPDATA\Temp\wellfit-target"
```

### 步驟 5：產出 Evidence-Driven A–L 報告

依 `10-uat-smoke-runbook.md` §5 範本，撰寫：

```
# UAT 冒煙報告 — <日期>
## A. 範圍
## B. 環境與版本
## C. 方法（指令逐字）
## D. 結果矩陣（9 支 API）
## E. 證據清單（檔案＋SHA-256）
## F. 偏差與歸因
## G. 風險
## H. 後續議題
## I. 判定（五級擇一）
## J. 簽核（Dev×QA×PMT）
## K. 引用（V29.txt 行號＋測試 ID）
## L. 附錄（原始 JSON）
```

證據檔寫至 `artifacts/uat-smoke/20261116/`（不進版控），帳號／統編不得寫入證據（遮蔽規則）。

## 5. 判定標準（Evidence-Driven，C-13）

| 判定 | 條件 |
|------|------|
| **PASS** | 9/9 entries `outcome=envelope_received` 且 code 形狀合法；付款探測皆被元件拒絕 |
| **CONDITIONAL PASS** | ≥7 entries 通過；失敗項已歸因並登錄議題 |
| **BLOCKED** | 傳輸不可達（防火牆/VPN）或 envelope 解析失敗 |
| **REJECTED** | 付款探測未被拒絕（元件接受了空 content）→ 立即通報銀行並中止 |
| **NOT VERIFIED** | 未取得環境，本手冊僅完成腳本與演練準備 |

## 6. 完成判準與結案

- PASS／CONDITIONAL PASS → 更新 `Wellfit/docs/bankpay/08-open-questions-and-decisions.md`（OQ-E3 狀態）與 `CHANGELOG.md`。
- REJECTED/BLOCKED → 登錄議題、保留證據、中止 UAT。
- 簽核：執行者（Dev）× 見證者（QA）× 負責人（PMT）。

## 7. 風險與約束

- `ESUN_ALLOW_PAYMENT=1` 才會觸發付款類 API（uat_01、uat_05），預設跳過，避免誤觸真實放行。
- 啟用碼效期 7 天，取得後應盡快執行。
- 不得將帳號／統編提交版控；證據檔需遮蔽。
- 所有證據標記 `uat-smoke/v1`，絕不呈現為真實銀行接受證據（不變量 #5）。
- UNKNOWN ≠ PASS ≠ FAIL；未執行案例不得以 PASS 代之。
