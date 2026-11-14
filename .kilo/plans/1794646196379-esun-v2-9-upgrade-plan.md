# 玉山銀企直連 V2.8→V2.9 規格差異與 wellfit_bankpay 升級計畫（可追溯 v2）

版本：v2｜2026-11-14｜狀態：待實作
上版：v1（同路徑）。v2 依「可追溯、可查證、可解釋、可推導」要求全面重寫：每項宣稱附來源行號與重驗方法；對使用者提供的參考分析逐條勘誤。

---

## 0. 證據基礎與查證方法

### 0.1 來源
| 編號 | 來源 | 位置 |
|---|---|---|
| S1 | 【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf | `C:\Users\stephanie.huang\Desktop\新人訓練\Projects\玉山串接專案\` |
| S2 | 【玉山銀行】企業ERP銀企直連串接_技術文件V2.9.pdf | 同上 |
| T1 | S1 之 pypdf 全文萃取 | `C:\Users\stephanie.huang\AppData\Local\Temp\kilo\【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.txt`（3,682 行） |
| T2 | S2 之 pypdf 全文萃取 | 同目錄 `..._技術文件V2.9.txt`（3,801 行） |
| C1 | `Wellfit/crates/wellfit_bankpay/src/*.rs` + `tests/` | 行內引用格式 `file:line` |
| D1 | `Wellfit/docs/esun-bank-flaps-erp-integration/api-spec-v28.md` 等 | 行內引用 |
| D2 | `Wellfit/AGENTS.md`（建置入口、不變量） | §3、§4 |

引用慣例：`V28.txt:nnn`＝T1 行號、`V29.txt:nnn`＝T2 行號。

### 0.2 萃取方法（可重現）
```powershell
& "C:\Users\stephanie.huang\.local\bin\uv.exe" run --with pypdf python -c "import pathlib;from pypdf import PdfReader;src=pathlib.Path(r'C:\Users\stephanie.huang\Desktop\新人訓練\Projects\玉山串接專案');out=pathlib.Path(r'C:\Users\stephanie.huang\AppData\Local\Temp\kilo');[(out/(f.stem+'.txt')).write_text('\n\n'.join((p.extract_text() or '') for p in PdfReader(str(f)).pages),encoding='utf-8') for f in sorted(src.glob('*.pdf'))]"
```
- 2026-11-14 實測成功產出 4 個 txt；fontTools 警告僅涉拉丁子集字型寬度（`VHNRKD+Helvetica` 等為 WinAnsiEncoding 標準字型），不影響 CJK 內文與表格數字。
- 行號可重現：pypdf 逐頁輸出＋固定換行，同指令重跑行號不變。

### 0.3 關鍵否定證據（負面查證）
- `grep "sourceSystem|Source-System" V2.9.txt` → **0 命中**（V2.9 全文已無此概念；非僅範例移除）。
- `grep "9443" V2.8.txt` → 命中 `:85`（防火牆 9443 兩版皆有，見 §2-E3）。

---

## 1. 規格差異總表（逐列附證據）

| # | 項目 | V2.8 | V2.9 | 證據 V28 | 證據 V29 |
|---|---|---|---|---|---|
| 1 | API 數量 | 8 支 | **9 支**（新增外幣憑證下載） | http.rs:8-15 對照 | V29.txt:32、:3475-3556 |
| 2 | 新 API `/payment/downloadFxRemittanceVoucher` | 無 | Req：remittanceNumber(14~16)、companyId(8~10)、applyDate(8)；Resp：fileInfo[]（fileType/fileName/fileBody=Base64 PDF） | — | V29.txt:3478、:3498-3500、:3526-3529、:3553-3556 |
| 3 | Http Header | Content-Type＋**Source-System（Y）** | 僅 Content-Type | V28.txt:126-128 | 0 命中（§0.3） |
| 4 | `rqHeader:sourceSystem` | 所有 API 必填 | 全部移除（僅 msgId/requestTime/customerIxd） | V28.txt:115、:128 | 0 命中；V29.txt:703-709、:1861-1864 範例 |
| 5 | 2-Way SSL | 「以 2-Way SSL 方式進行檢核」＋憑證檔互傳 | 移除該句與憑證交換前置；保留 TLS1.2+ HTTPS | V28.txt:113-114 | 連線節無 2-Way 字樣；910067（解密失敗）保留 V29.txt:3704 |
| 6 | 元件 URL | (暫)DEV:8080、測試:8080、營運:8443 | 測試:**8443**、營運:**8080**；IP=元件安裝機器（例 127.0.0.1:8443） | V28.txt:54-56 同 D1:54-56 | V29.txt URL 節（連線章節） |
| 7 | 環境與防火牆 | 三環境網域（含 eospt DEV）＋防火牆 9443 | 移除 eospt；UAT eospu 219.87.16.166、營運 eosp 61.220.45.177/219.87.57.177/61.222.116.195；防火牆 **9443** | V28.txt:85-87 | V29.txt:85-89 |
| 8 | 台幣 `operatorName` | 32 | **12** | V28.txt:233 | V29.txt:220 |
| 9 | 台幣 `flowName` | Y、64 | 條件必填、**10** | V28.txt:242 | V29.txt:229-232 |
| 10 | 外幣 `operatorName`/`flowName` | 32/64 | **仍 32/64**（未改） | V28.txt:2468/2479 | V29.txt:2453/2464 |
| 11 | 台幣 `twdTxList:payerId` | 至少 10 碼 | **至少 8 碼** | V28.txt（台幣付款資料節） | V29.txt:244 起節內 |
| 12 | 台幣 `payeeName`/`memo` | 80/80 | **78/78** | V28.txt 台幣付款資料節 | V29.txt:268±、:271 |
| 13 | 台幣 `remark` | 38 | **36** | 同上 | V29.txt:285 |
| 14 | 台幣 detailList 金額 | req 18.2 / resp 10 | **req=resp=15.2** | V28.txt 台幣 detailList 節 | V29.txt:295-298、:373-379 |
| 15 | 外幣 `payerAmount`/`payeeAmount` | 16（BigDecimal） | 16（不變；:1334/:1357 之 15.2 為單據欄） | V28.txt:1729/1734 | V29.txt:1702/1707 |
| 16 | 外幣 `bargainNo` | 15 | **18**（req+resp） | V28.txt:1837 | V29.txt:1602-1603、:1811 |
| 17 | `paymentType` 語意 | 1:借一貸一／2:借一貸多 | **1:單筆付款交易／2:整批付款交易**（台幣+外幣同改） | V28.txt:230/2464-2465 | V29.txt:215-217、:2447-2450 |
| 18 | `formatType=1` 說明 | 僅限借一貸一交易 | 僅限**單筆付款交易** | V28.txt 台幣放行節 | V29.txt 台幣放行 formatType 列 |
| 19 | 台幣 `content` 分流 | 「台幣付款檔案轉 Base64」 | **paymentType=1 且 verificationType=2 → 台幣付款檔案；其餘 → 台幣付款資料(json)** | V28.txt 台幣 content 列 | V29.txt:238-242（§台幣付款資料 json :242 起） |
| 20 | 整批費用一致性 | paymentType=2 時費用付款**帳號**與**幣別**同批皆須相同 | 同（未新增） | **V28.txt:1639、:1656** | V29.txt:1615-1617、:1632-1633 |
| 21 | 外幣 `feeDebitAccountNo` | 17 | 17（補最小 12／最大 17） | V28.txt:1633 | V29.txt:1610-1614 |
| 22 | 查詢上限錯誤碼 | 範例誤用 **910070**（附錄定義=約定資料有誤，自相矛盾） | 範例統一 **910073** | V28.txt:3153 vs :3587 | V29.txt:3135-3136 vs :3710 |
| 23 | 新增錯誤碼 | 無 | **910093**{電文欄位}為必填、**910094**{電文欄位}不符 | V28.txt:3575-3597 無此二碼 | V29.txt:3713-3714 |
| 24 | 範例筆誤修正 | `"feeDebitAccount"`、CITITWTP/TW | `"feeDebitAccountNo"`、CITIUS33/US | V28.txt:1976、:1961/1966 | V29.txt:1960、:1945/1950 |
| 25 | Response 範例鍵名 | camelCase | 外幣檢核成功範例改 PascalCase（TxSeq/Status…） | V28.txt:2015-2093 | V29.txt:2002-2075 |
| 26 | 媒體檔規格（BIG-5/422/欄位位置/附言 80/備註 38） | — | **完全不變** | V28.txt:3497-3567 | V29.txt:3622-3687 |
| 27 | 狀態表/幣別表/msgId 規則 | — | 不變 | V28.txt:3604-3682 | V29.txt:3726-3801 |

### 1.1 推導（規格數字的由來）
- 78＝39 全形字×2、36＝18 全形字×2（V29.txt:271-272、:285-286 明載全形換算法）→ V2.8 的 80/38 是「半形位數上限」與「全形計數」混用，V2.9 統一為 byte 長度對齊全形上限。
- 15.2＝13 位整數＋2 位小數，對齊媒體檔付款金額欄 13 位（V29.txt:3645）；消除 V2.8 req 18.2/resp 10 矛盾。
- 12＝網銀使用者名稱上限；V2.8 範例 "ESB17794"（8 碼，V28.txt:725）相容。
- Port 架構：9443＝銀行端防火牆外部（兩版同，V28.txt:85/V29.txt:85）；8443/8080＝安控元件服務 port（V2.9 對調）；ERP→元件 8443→放行主管電腦（V28.txt:83，D1:23 同構）。
- sourceSystem 移除推導：V2.9 連線對象＝本地/內網安控元件（V29.txt:6「IP 指向元件安裝機器」），元件端已綁定約定身分，故應用層免再帶。

---

## 2. 參考分析勘誤表（使用者提供之參考資料 vs 實證）

| Ref # | 參考宣稱 | 判定 | 實證 |
|---|---|---|---|
| R1 | 9 支 API、新增憑證下載 | ✅ 正確 | §1-1/2 |
| R2 | Header 廢除 Source-System；rqHeader 全面移除 | ✅ 正確 | §1-3/4（0 命中） |
| R3 | Port 反轉（測試 8443/營運 8080） | ✅ 正確 | §1-6 |
| R4 | 「V2.9 明文增列 paymentType=2 費用帳號/幣別同批規則」 | ❌ **錯誤** | V2.8 已有：V28.txt:1639（帳號）、:1656（幣別）；非 V2.9 新增，僅為「實作未涵蓋之既有規則」 |
| R5 | 「V2.9 移除開發網域、僅列 IP 白名單矩陣（9443）」 | ⚠️ 部分錯誤 | V2.9 仍列 eospu/eosp 網域（V29.txt:87-88），僅移除 eospt；9443 防火牆 port V2.8:85 已存在；V2.9 新增的是**具體 IP** |
| R6 | 「將 client_cert_path/client_key_path 降為可選」 | ⚠️ 不需做 | 程式碼本就是 `Option` 且預設 None（http.rs:62-72、:91），V2.8 模式即為 opt-in；殘留點僅在文件 D1:48 |
| R7 | 「長度驗證由 32 修正為 12…」等 | ⚠️ 性質誤述 | crate **無任何欄位級 runtime 驗證**（僅 media.rs:38-121 之 422 檔整行檢查；api.rs:170-181 長度僅存在於註解）→ 任務是「**新增**驗證層」，非「修改」 |
| R8 | api-spec-v28.md 端點筆誤（retrieveBalance/retrieveTwdPaymentResult） | ✅ 正確 | D1:209、:214、:248-249；程式碼端點正確（http.rs:551、:558-566），僅文件錯 |
| R9 | 錯誤碼 910070→910073 | ✅ 正確 | §1-22 |
| R10 | 範例 feeDebitAccountNo/CITI 修正 | ✅ 正確 | §1-24 |
| R11 | 憑證下載 struct 範例 | ✅ 可用 | 欄位與 §1-2 一致（fileName 50、fileType 10 長度須納入驗證） |

參考資料未涵蓋、本計畫補上：§1-10（外幣 32/64 不變→驗證須分流）、§1-18/19（formatType=1 限制與 content 分流）、§1-21（feeDebitAccountNo 最小 12）、§1-23（910093/094）、§1-25（PascalCase 容差）、`formetType` 兩版皆存的拼字陷阱（V28.txt:1906、V29.txt:2522 範例用 `formetType`，表格為 `formatType`）、Wellfit 建置必經 build.ps1（D2 §3）。

---

## 3. wellfit_bankpay 現況稽核（code 證據）

| # | 現況 | 證據 | V2.9 判定 |
|---|---|---|---|
| A1 | 8 支 V2.8 API 全部已實作 | http.rs:450-566（端點常數與方法） | 保留；缺第 9 支 |
| A2 | `RqHeader.sourceSystem` 欄位（必填語意） | api.rs:39-40 | 移除/改可選 |
| A3 | `Source-System` HTTP header | http.rs:390 | 移除（或可選開關） |
| A4 | config `source_system` 必填 | bankpay.rs:97-102 | 改非必填 |
| A5 | TLS：rustls；client cert/CA/pinning 皆 opt-in、預設 None；設定載入失敗即拒發（不靜默降級） | http.rs:36-39、:61-95、:124-131、:329-331 | 程式碼不動；文件對齊 |
| A6 | 預設 base_url `https://127.0.0.1:8443` | http.rs:61-65、:90 | 需環境化（UAT 8443/營運 8080） |
| A7 | 欄位長度僅存在註解（operator 32/flow 64 等） | api.rs:170-181、:178-181 | 需新增驗證層（§4-T4） |
| A8 | 錯誤碼常數僅 SUCCESS/FIELD_ERROR（910000/910071） | api.rs:381-383 | 需擴全表＋分類 |
| A9 | 媒體檔 422 固定長度驗證已有（BIG-5 padding：附言 80/備註 38） | media.rs:38-121、:87-105 | **不動**（V2.9 未改媒體檔，§1-26） |
| A10 | 憑證下載未實作 | 全 crate 無 downloadFxRemittanceVoucher | 新增（§4-T5） |
| A11 | 整批費用一致性驗證未實作（規則兩版皆有） | crate 無對應檢查 | 新增（§4-T6） |
| A12 | 建置須經知識庫 build.ps1（應用程式控制原則）；clippy -D warnings；forbid(unsafe_code)；金額零浮點 | D2 §3、§4 | 驗證計畫依此執行 |
| A13 | 歷史計畫 `Wellfit/plans/1793034142477-esun-flaps-ap-voucher-integration.md` 已標 Superseded（API-first 為準） | 該檔 Disposition 2026-11-02 | 僅參照，不執行 |

---

## 4. 升級任務清單

### P0（V2.9 合規必要；不修無法正確對接）
- **T1 環境與 Port 對調**：config 新增 `environment: uat|production`（來源：A-004 部署模型）；UAT 預設 `…:8443`、營運 `…:8080`；移除 DEV 網域支援；文件登錄防火牆 9443 與營運 IP 表（V29.txt:85-89）。檔案：http.rs:61-95、bankpay.rs config、api-spec 文件。
- **T2 sourceSystem 下線（可選開關）**：`RqHeader.source_system` 改 `Option<String>`＋`#[serde(skip_serializing_if = "Option::is_none")]`；`Source-System` header 僅在設定時送出；config 欄位改非必填，預設 None（V2.9 模式）。保留開關供舊元件回測。檔案：api.rs:39-40、http.rs:390、bankpay.rs:97-102。理由：向 V2.9 元件發送未知欄位有被 910094 拒收風險（V29.txt:3714）。
- **T3 bargainNo 18**：外幣欄位 schema/驗證 15→18（V29.txt:1602-1603、:1811）。
- **T4 錯誤碼映射更新**：新增 910093/910094（欄位級錯誤，含 moreInfo 電文欄位名解析）；「查詢上限」歸類 910073；910070 回歸其原義（約定資料有誤）。檔案：api.rs:381-383 擴充。
- **T5 文件同步**：`api-spec-v28.md` 升級為 `api-spec-v29.md`（保留舊檔＋SUPERSEDED 標記）：修正端點筆誤（D1:209→`/account/retrieveAccountBalance`、:214→`/payment/retrieveTwdPaymentInstructionPrintResult`）、Header 表刪 Source-System、URL 表改 V2.9、新增第 9 支 API、補 910093/094；`implementation-guide.md`/`testing-environment.md` 的 8443 語意改為「UAT 元件 port；營運 8080」。ADR-0012（Dev/docs/adr）A-004 錨定版本註記 V2.8→V2.9。

### P1（規格合規＋新功能）
- **T6 新增欄位驗證層**（性質＝新增，見 R7）：於 api.rs（或新 validation.rs）實作兩套規則集：
  - 台幣：payeeName≤78、memo≤78、remark≤36、payerId≥8 碼、operatorName≤12、flowName≤10、detailList 三金額 15.2（V29.txt:220/229/268-285/295-298）。
  - 外幣：operatorName≤32、flowName≤64、bargainNo≤18、feeDebitAccountNo 12~17（V29.txt:2453/2464/1602/1613-1614）。
  - 違規回傳結構化 Err（欄位名＋原因），不入電文。
- **T7 content 分流檢核**：台幣放行——paymentType=1 且 verificationType=2 → 檔案格式（媒體檔產生器輸出）；其餘 → json（V29.txt:238-242）；formatType=1 僅允許單筆付款（V29.txt 台幣 formatType 列）。
- **T8 整批費用一致性驗證**（既有規則補實作，見 R4）：paymentType=2 時同批 foreignTxList 的 feeDebitAccountNo 與 feeDebitCurrency 必須一致（V28.txt:1639/1656＝V29.txt:1615-1617/1632-1633）。
- **T9 實作外幣憑證下載 API**：型別（remittanceNumber 14~16、companyId 8~10、applyDate 8）、`POST /payment/downloadFxRemittanceVoucher`、fileInfo[] 解析、Base64→PDF 落地（附 SHA-256 記錄，符合 D2 不變量 #8）；mock HTTP 測試仿 http.rs 既有模式（:1312-1635）。檔案：api.rs、http.rs、（整合點）query.rs/release.rs。

### P2（穩健度）
- **T10 錯誤碼全表常數化＋分類**：可重試（910068）；不可重試（910063/064/065/066/067/070/071/072/073/074/075）；欄位級（910069/093/094/097/098/099）；電文異常 910144；未知 999999（V29.txt:3696-3719）。
- **T11 鍵名大小寫容差**：V2.9 外幣檢核 Response 範例為 PascalCase（V29.txt:2005-2074）——serde 反序列化加 alias 或收到後正規化，防真實元件大小寫不一致。
- **T12 `formetType` 防呆**：送出一律 `formatType`；解析端容錯兩版拼字（V28.txt:1906、V29.txt:2522 範例即拼錯，文件表格正確）。
- **T13 UAT 冒煙腳本**：以 V29.txt:87-89 IP/網域對 219.87.16.166:8443 逐 API happy path＋欄位錯誤 path；產出 Evidence-Driven A–L 節報告（C-13）。

---

## 5. 驗證計畫
1. 建置/測試一律經 `Knowledge/tools/dev-env/build.ps1`（D2 §3；`-TargetDir` 用預設或各自不同值）：
   `clippy --workspace --all-targets -- -D warnings`；`test --workspace --no-fail-fast`。
2. 新增測試對應每項任務：T1 環境表、T2 sourceSystem 開關（送出/省略兩態）、T4 三錯誤碼、T6 邊界值（78/36/12/10/8/18/12-17/15.2）、T7 分流、T8 同批不一致拒絕、T9 voucher mock（成功/999999 applyDate 缺漏）。
3. 追溯：每測試 ID 對應規格行號（本檔 §1 表），登錄 `Knowledge/traceability/claims-vs-evidence.md`（D2 不變量 #8）。
4. UAT 實測（T13）前不宣稱通過；UNKNOWN ≠ PASS（C-10）。

## 6. 風險與 V2.9 文件自身不一致（送出端取嚴、接收端取寬，註解登記待 UAT 釐清）
- 台幣 Response 長度仍寫 80/38 vs Request 78/36（V29.txt resp 節）。
- 媒體檔付款統編「至少 10 碼」（V29.txt:3633-3634）vs json payerId「至少 8 碼」→ 兩路徑分別驗證。
- 外幣 operatorName 32 vs 台幣 12（V29.txt:2453 vs :220）→ 依 API 分流，禁止統一。
- 憑證下載失敗範例以 999999＋moreInfo 呈現欄位缺漏（V29.txt:3612-3615），與 910093 語意重疊 → 兩者皆須處理。
- 2-Way SSL 文件移除但元件實際行為未經 UAT 實證（V2.9 僅文字層面）→ T5 保留 opt-in TLS 材料能力，不刪除。

## 7. 決策紀錄（已定案）
- sourceSystem：**可選開關、預設關閉**（非硬刪）——向後相容舊元件回測＋避免 V2.9 拒收。
- mTLS/pinning：**程式碼不動**（本就 opt-in），僅文件對齊。
- 驗證層：**新增模組**，規則集依台幣/外幣分流。
- 文件：api-spec 升級 V2.9（舊檔歸檔標記）；歷史計畫 A13 維持 Superseded。

## 8. 範圍外
- 約定書條款法律審閱（僅技術比對；`玉山全球智匯網—ERP系統直連服務約定書_已簊.txt` 已萃取可供日後引用）。
- eToken/放行主管電腦端流程（屬安控元件側，非本 crate 範圍）。
- UAT 實際連線（需約定帳號與環境，屬 T13 執行期事項）。
