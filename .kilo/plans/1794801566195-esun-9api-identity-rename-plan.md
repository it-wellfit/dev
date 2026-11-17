# E.Sun UAT 9 支 API 全數串接成功計畫（裝置身份對齊：改名 WELLFITCOMPANY）
> 2026-11-17 Reviewed（整合完成計畫 1794810000000 I-4）：掃描無未勾選任務（\- [ ]\=0）；無未關閉程式任務。證據：PLAN_REGISTER addendum #52。

## 1. 目標與成功定義（兩級制，均已裁定納入）
- **Stage 1（安控層冒煙）**：9/9 支 API `outcome=envelope_received` 且結果碼形狀合法（`910ddd`/`999999`），付款探測被元件正確拒絕（`10-uat-smoke-runbook.md` §5 PASS）。
- **Stage 2（業務級，2026-11-17 裁定納入）**：每支 API 以真實測試資料取得 `910000`——真實放行鏈（uat_01→02/08、uat_05→06/09）、uat_03/uat_07 真實帳號資料、經辦兩層核可、負責人書面核可。
- 證據：JSON + SHA-256 sidecar + A–L 報告（C-13 五級判定）。

## 2. 背景與根因（證據綁定）
- 11/16 冒煙 **BLOCKED**（`Wellfit/artifacts/uat-smoke/20261116/UAT_SMOKE_REPORT_20261116.md`）：7/7 打遠端 `219.87.16.166:8443` os error 10054＋缺啟用碼。
- 綁定請求已送達玉山 UAT（RS 200），元件實際送出：`hostName=stephanie`（**原樣小寫**，證明元件不自動轉換大小寫）、`MAC=A8-7E-EA-E8-A5-03`、`osUuid=D77D5E8A-9B82-A947-952A-954511835E02`、`customerId=68001672`、啟用碼 `72442416`。
- **核心根因**：銀行已登記 `hostName=WELLFITCOMPANY`（全大寫），元件只送本機電腦名稱 `stephanie` → 不一致即綁定/串接失敗。
- **大小寫裁定（2026-11-17）**：Windows 電腦名稱不區分大小寫（`WELLFITCOMPANY`≡`WellfitCompany`，同一邏輯名），顯示大小寫＝改名時輸入值；元件原樣送出（stephanie 實證）。銀行端比對是否區分大小寫＝UNKNOWN → **改名輸入與銀行登記逐字元一致的 `WELLFITCOMPANY`**，並在重綁後核對送出值的 case。
- 次要根因：11/16 測試目標用遠端 IP；V2.9 規則＝「元件安裝機器」→ 應打 `http://127.0.0.1:8443`（EES Service 已在本機 Listen）。
- MAC 風險：使用者曾以 `00-FF-31-5C-3C-26` 送銀行申請，與元件實際送出的 `A8-7E-EA-E8-A5-03` 不符；須以元件實際值更正銀行登記。

## 3. 前置條件（銀行端，P0；未閉合不得執行 M-5）
| # | 動作 | 判準 |
|---|------|------|
| P-1 | 向銀行確認/更正裝置登記：`hostName=WELLFITCOMPANY`、`MAC=A8-7E-EA-E8-A5-03`、`osUuid=D77D5E8A-9B82-A947-952A-954511835E02`、統編 `68001672` | 銀行書面回覆四值逐字元一致（含大小寫） |
| P-2 | 詢問改名後重新綁定是否需**重新發啟用碼**（舊碼 `72442416` 綁 stephanie、7 天效期） | 取得明確答案；若需新碼，記錄效期 |
| P-3 | （Stage 2 才需要）取得負責人書面核可（C-05）與 UAT 測試受益人帳號 | 書面紀錄 + 帳號清單 |

## 4. 本機執行步驟（P0）
| # | 動作 | 指令/方式 | 判準 |
|---|------|-----------|------|
| M-1 | 改名前現況存證 | 記錄 `hostname`、`getmac /v`（辨識 `A8-7E-EA-E8-A5-03` 屬哪張網卡、`00-FF-31-5C-3C-26` 是否為虛擬網卡）、osUuid 來源、EES Service 狀態 | 綁定窗口內不得切換 Wi-Fi/乙太網路（MAC 來源必須穩定） |
| M-2 | 改名 | 管理員 PowerShell：`Rename-Computer -NewName WELLFITCOMPANY -Restart`（需重開機，輸入**全大寫**） | 重開機後 `hostname` 顯示 `WELLFITCOMPANY` |
| M-3 | 授權回歸（風險閘） | 實測啟動 FlapsErp（`KEEP_MACHINE_ID=1`；machineId 是否受主機名影響＝UNKNOWN，未量測不得視為 PASS） | FlapsErp 可正常登入；失效則聯繫原廠（禁改內容，僅 config 層） |
| M-4 | 元件與網路就緒 | EES Service 啟動（若殘留 stephanie 時期的綁定狀態，先清除/重裝元件再首次啟動）、`Get-NetTCPConnection -LocalPort 8443` Listen、`w32tm /resync`、`Test-NetConnection` 對銀行 9443 | 8443 LISTEN；時間誤差 <1s；9443 通 |
| M-5 | 重新綁定 | 元件首啟輸入 統編 `68001672` + ClientId `3c37a6988dd8dfaf9af5d8fb7f03b4df` +（新/舊）啟用碼 | 綁定請求 log 顯示 `hostName=WELLFITCOMPANY`（**核對大小寫與銀行登記逐字元一致**）且 RS 200 |

## 5. 測試矩陣
### Stage 1：安控層冒煙（P1，既有 harness）
| # | 動作 | 判準 |
|---|------|------|
| T-1 | 修 `uat_smoke.rs::finish()`（:160-173）：`write_evidence` 移到 `assert_envelope_hygiene` **之前**（transport_error 時證據仍落盤）；同步修正文件註解 :14-15 與 `10-uat-smoke-runbook.md` 步驟 1 的建議 URL 為 `http://127.0.0.1:8443` | clippy 零警告；單測驗證證據寫入 |
| T-2 | 基線：`build.ps1` → `clippy --workspace --all-targets -- -D warnings` + `test --workspace --no-fail-fast` | 零警告；0 failed |
| T-3 | 非付款 7 支（uat_02/03/04/06/07/08/09）：`ESUN_UAT_BASE_URL=http://127.0.0.1:8443` + 環境變數，`--ignored --nocapture` | 每支 `envelope_received`、碼形合法（查詢類 `910004`/`910072` 族＝安控+傳輸通，註記） |
| T-4 | 付款 2 支（uat_01/05）：書面核可 → `ESUN_ALLOW_PAYMENT=1` → 補測（空 content 探測，元件必須拒絕） | `envelope_received` + 拒絕碼；若元件接受空 content＝REJECTED，立即通報銀行 |
| T-5 | A–L 報告（新增日期版）+ 全量 SHA-256 sidecars | 五級判定 |

### Stage 2：業務級真實放行鏈（P2，已裁定納入）
| # | 動作 | 依賴 |
|---|------|------|
| S2-1 | 新增有效資料版探測（uat_01 真實 TWD 放行 content、uat_04 完整 payee 欄位、uat_05 真實 FX 放行）；金額最小化（如 1 元），txSeq 唯一，仿 `09-sandbox-acceptance-runbook.md` TC-P01 | P-3 書面核可 + UAT 受益人帳號（帳戶經理提供） |
| S2-2 | 鏈式驗證：uat_01 →（uat_02 查詢、uat_08 列印，**需經辦 test01 在銀行端完成兩層核可**——人工步驟）；uat_05 →（uat_06 查詢、uat_09 憑證下載，需 remittanceNumber） | S2-1 回傳的 paymentNo/txSeq/remittanceNumber；經辦核可完成 |
| S2-3 | uat_03/uat_07 以真實帳號 `9915940107636` 驗證回傳資料 | 帳號有實際餘額/明細；無資料時以 `910004` 註記（安控通、業務待資料） |

## 6. 風險與對策
| 風險 | 對策 |
|------|------|
| FlapsErp 授權可能綁主機名（UNKNOWN） | M-3 第一時間實測；失效則聯繫原廠 |
| 啟用碼 7 天效期 | M-2..M-5 與 T-3 安排在同一效期窗口內完成 |
| 元件殘留 stephanie 綁定狀態 | M-4 先清除/重裝；M-5 log 核對送出值 |
| 銀行端大小寫比對未知 | M-2 輸入全大寫 + M-5 逐字元核對；不符即回銀行 |
| MAC 來源網卡切換 | M-1 辨識網卡；綁定窗口內固定網路 |
| 付款類誤觸真實資金移動 | Stage 1 維持空 content 探測 + 雙重閘門；Stage 2 僅 UAT 環境 |

## 7. 驗證與證據紀律
- 每步綁定：指令 + 工作目錄 + 退出碼 + 日期（GOVERNANCE P2）；`UNKNOWN ≠ PASS ≠ FAIL`（C-10）。
- 斷言式變更：改 `uat_smoke.rs` 前先確認目標字串存在且唯一（C-04）。
- 提交前：`cargo fmt -- --check`、`clippy -D warnings`、`cargo test --locked`；文件與代碼同一提交。

## 8. 範圍外
- 生產環境（eosp）部署與驗收；`09-sandbox-acceptance-runbook.md` 的 OQ-E3 沙盒驗收（獨立軌道）。
- FlapsErp 內容修改（命名禁令）。
- 遠端 `219.87.16.166:8443` 元件側問題。

## 9. 待決議題（剩外部依賴，無設計未決）
1. P-1/P-2 銀行回覆：裝置登記四值確認（含 MAC 更正）與啟用碼是否重發——閉合前 M-5 不得執行。
2. P-3（Stage 2 前置）：負責人書面核可 + 帳戶經理提供 UAT 測試受益人帳號。
3. 經辦兩層核可（uat_08 列印鏈）為銀行端人工步驟，排程需配合營業時間。
