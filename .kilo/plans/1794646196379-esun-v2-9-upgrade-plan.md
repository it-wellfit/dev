# 玉山銀企直連 V2.8→V2.9 規格差異與 wellfit_bankpay 升級計畫（可追溯 v4）
> 2026-11-17 Reviewed（整合完成計畫 1794810000000 I-4）：掃描無未勾選任務（\- [ ]\=0）；無未關閉程式任務。證據：PLAN_REGISTER addendum #52。

版本：v4｜2026-11-15｜狀態：T1-T12 已實作並驗證；剩餘工作見 §9
- v4：補 §9 完成狀態（T1-T12＋審查修正 8 項＋build.ps1 實測證據）與剩餘工作清單（W1-W5：T13 冒煙腳本、執行手冊、追溯登錄、文件漂移修正、重驗證）。
- v3：全部差異列完成精確行號定位；§6.1「V2.9 為標準但不亂抄」六項邏輯裁決；§6.2 自我勘誤。
- v2：逐條勘誤參考分析（§2）。
- v1：初版比對結論。

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
- `grep "2-Way|SSL|雙向|憑證檔" V2.9.txt` → **0 命中**（V2.9 全文無雙向認證/憑證檔交換概念；對照 V2.8:113-114、:126-128）。
- `grep "9443" V2.8.txt` → 命中 `:85`（防火牆 9443 兩版皆有，見 §2-R5）。
- 「至少」斷詞注意：pypdf 萃取有跨行斷詞（如 V2.8「至少⏎10 碼」位於 :256-258；V2.9「至少⏎8 碼」位於 :246-247），grep 單行 pattern「至少 10 碼」會漏命中——重驗時須以跨行 context 或「至少」單詞檢索。

---

## 1. 規格差異總表（逐列附證據）

| # | 項目 | V2.8 | V2.9 | 證據 V28 | 證據 V29 |
|---|---|---|---|---|---|
| 1 | API 數量 | 8 支 | **9 支**（新增外幣憑證下載） | http.rs:8-15 對照 | V29.txt:32、:3475-3556 |
| 2 | 新 API `/payment/downloadFxRemittanceVoucher` | 無 | Req：remittanceNumber(14~16)、companyId(8~10)、applyDate(8)；Resp：fileInfo[]（fileType/fileName/fileBody=Base64 PDF） | — | V29.txt:3478、:3498-3500、:3526-3529、:3553-3556 |
| 3 | Http Header | Content-Type＋**Source-System（Y）** | 僅 Content-Type | V28.txt:126-128 | 0 命中（§0.3） |
| 4 | `rqHeader:sourceSystem` | 所有 API 必填 | 全部移除（僅 msgId/requestTime/customerIxd） | V28.txt:115、:128 | 0 命中；V29.txt:703-709、:1861-1864 範例 |
| 5 | 2-Way SSL | 「以 2-Way SSL 方式進行檢核」＋憑證檔互傳 | 移除該句與憑證交換前置；保留 TLS1.2+ HTTPS | V28.txt:113-114、:126-128 | grep 0 命中（§0.3）；910067（解密失敗）保留 V29.txt:3704 |
| 6 | 元件 URL | (暫)DEV:8080、測試:8080、營運:8443 | 測試:**8443**（註 IP=元件安裝機器 ex.127.0.0.1:8443）、營運:**8080**；台幣放行 URL 不變 | V28.txt:121-124、:82-83（內部 8443）、:192 對照 | V29.txt:115-116、:82-83、:192 |
| 7 | 環境與防火牆 | 三環境網域（含 eospt DEV）＋防火牆 9443 | 移除 eospt；UAT eospu 219.87.16.166、營運 eosp 61.220.45.177/219.87.57.177/61.222.116.195；防火牆 **9443** | V28.txt:85-87 | V29.txt:85-89 |
| 8 | 台幣 `operatorName` | 32 | **12** | V28.txt:233 | V29.txt:220 |
| 9 | 台幣 `flowName` | Y、64 | 條件必填、**10** | V28.txt:242 | V29.txt:229-232 |
| 10 | 外幣 `operatorName`/`flowName` | 32/64 | **仍 32/64**（未改；台幣外幣雙軌並存，非筆誤） | V28.txt:2468/2479 | V29.txt:2453/2464 |
| 11 | 台幣 `twdTxList:payerId` | 至少 10 碼 | **至少 8 碼** | V28.txt:256-258（跨行「至少⏎10 碼」） | V29.txt:246-247（跨行「至少⏎8 碼」） |
| 12 | 台幣 `payeeName`/`memo` | 80/80 | **78/78** | V28.txt:262-263、:284 | V29.txt:251-252、:271 |
| 13 | 台幣 `remark` | 38 | **36**（18 全形×2） | V28.txt:300-301 | V29.txt:285-286 |
| 14 | 台幣 detailList 金額 | 放行 req 18.2（:311-314）；明細查詢 resp 10（:389-395） | **兩處皆 15.2**（:295-298、:373-379） | V28.txt:311-314、:389-395 | V29.txt:295-298、:373-379 |
| 15 | 外幣 `payerAmount`/`payeeAmount` | 16（BigDecimal） | 16（不變；:1334/:1357 之 15.2 為單據欄） | V28.txt:1729/1734 | V29.txt:1702/1707 |
| 16 | 外幣 `bargainNo` | 15 | **18**（req+resp） | V28.txt:1837 | V29.txt:1602-1603、:1811 |
| 17 | `paymentType` 語意 | 1:借一貸一／2:借一貸多 | **1:單筆付款交易／2:整批付款交易**（台幣+外幣同改） | V28.txt:230/2464-2465 | V29.txt:215-217、:1288、:2447-2450 |
| 18 | `formatType` | 台幣 1:檔案（僅限借一貸一）/2:json；外幣僅 2:json | 台幣 1:檔案（**僅限單筆付款交易**）/2:json；外幣仍僅 2:json | V28.txt 台幣放行節 | V29.txt:212-213（台幣）、:1287/:2446（外幣僅 2） |
| 19 | 台幣 `content` 分流 | 「台幣付款檔案轉為 Base64 格式（台幣付款資料格式參台幣付款資料(json))」— **已提 json 參照但無分流條件** | **明確分流：paymentType=1 且 verificationType=2 → 台幣付款檔案；其餘 → 台幣付款資料(json)**（消除歧義） | V28.txt:248-250、json 節 :252 | V29.txt:238-242（json 節 :242 起） |
| 20 | 整批費用一致性 | paymentType=2 時費用付款**帳號**與**幣別**同批皆須相同 | 同（未新增） | **V28.txt:1639、:1656** | V29.txt:1615-1617、:1632-1633 |
| 21 | 外幣 `feeDebitAccountNo` | 17 | 17（補最小 12／最大 17） | V28.txt:1633 | V29.txt:1610-1614 |
| 22 | 查詢上限錯誤碼 | 範例誤用 **910070** 共 2 處（台幣查詢:1008、外幣查詢:3153），附錄定義=約定資料有誤（:3587）→ 自相矛盾 | 兩處範例皆改 **910073**（與附錄一致） | V28.txt:1008、:3153 vs :3587 | V29.txt:987-988、:3135-3136 vs :3710 |
| 23 | 新增錯誤碼 | 無 | **910093**{電文欄位}為必填、**910094**{電文欄位}不符 | V28.txt:3575-3597 無此二碼 | V29.txt:3713-3714 |
| 24 | 範例筆誤修正 | `"feeDebitAccount"`、CITITWTP/TW | `"feeDebitAccountNo"`、CITIUS33/US | V28.txt:1976、:1961/1966 | V29.txt:1960、:1945/1950 |
| 25 | Response 範例鍵名 | camelCase | 外幣檢核成功範例改 PascalCase（TxSeq/Status…）；表格欄位名仍 camelCase | V28.txt:2015-2093 | V29.txt:2002-2075（表 :1688-1852） |
| 26 | 媒體檔規格（BIG-5/422/欄位位置/附言 80/備註 38/付款統編≥10 碼/單據金額 10 位） | — | **完全不變** | V28.txt:3497-3567（統編 :3508-3509、金額 :3560-3562） | V29.txt:3622-3687（統編 :3633-3634、金額 :3680-3682） |
| 27 | 台幣交易明細查詢 `txRecordList`（resp） | payeeName 80、remark 38、detailList 金額 10 | payeeName 80、remark 38 **不變**；detailList 10→15.2 | V28.txt:360、:379、:389-395 | V29.txt:344、:363、:373-379 |
| 28 | 狀態表/幣別表/msgId 規則 | — | 不變 | V28.txt:3604-3682 | V29.txt:3726-3801 |

### 1.1 推導（規格數字的由來）
- 78＝39 全形字×2（V29.txt:252「收款戶名」說明＋媒體檔 :3641-3642 同構）、36＝18 全形字×2（V29.txt:285-286 明載）→ V2.8 的 80/38 與全形上限換算不一致，V2.9 統一為 byte 長度對齊全形字上限。
- 15.2＝13 位整數＋2 位小數，對齊媒體檔付款金額欄 13 位（V29.txt:3645）；消除 V2.8 放行 req 18.2（V28:311-314）與明細查詢 resp 10（V28:389-395）不一致。
- 8 碼＝台灣營利事業統一編號長度；V2.8「至少 10 碼」（V28:256-258）與媒體檔付款統編「至少 10 碼」（兩版皆 :3508-3509/:3633-3634）並存 → json 與 BIG-5 檔兩通道規格獨立，驗證分通道（§6.1-1）。
- 12＝網銀使用者名稱上限；V2.8 範例 "ESB17794"（8 碼，V28.txt:725）相容。
- Port 架構：9443＝銀行端防火牆外部（兩版同，V28.txt:85/V29.txt:85）；8443/8080＝安控元件服務 port（V2.9 對調）；ERP→元件 8443→放行主管電腦（V28.txt:83、V29.txt:83 同）。
- sourceSystem 移除推導：V2.9 連線對象＝本地/內網安控元件（V29.txt:115「IP 位址指向元件安裝機器」），元件端已綁定約定身分，故應用層免再帶；且 V2.9 全文無 sourceSystem/2-Way/憑證檔（§0.3 三重否定證據）。

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
| R7 | 「長度驗證由 32 修正為 12…」「付款統編說明誤植為至少 10 碼」等 | ⚠️ 性質誤述 | crate **無任何欄位級 runtime 驗證**（僅 media.rs:38-121 之 422 檔整行檢查；api.rs:170-181 長度僅存在於註解）→ 任務是「**新增**驗證層」，非「修改」；「至少 10 碼」確為 V2.8 原文（V28.txt:256-258 跨行），V2.9 改 8 碼（V29.txt:246-247）✅ |
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
| A7 | 欄位長度僅存在註解（operator 32/flow 64 等） | api.rs:170-181、:178-181 | 需新增驗證層（§4-T6） |
| A8 | 錯誤碼常數僅 SUCCESS/FIELD_ERROR（910000/910071） | api.rs:381-383 | 需擴全表＋分類 |
| A9 | 媒體檔 422 固定長度驗證已有（BIG-5 padding：附言 80/備註 38） | media.rs:38-121、:87-105 | **不動**（V2.9 未改媒體檔，§1-26） |
| A10 | 憑證下載未實作 | 全 crate 無 downloadFxRemittanceVoucher | 新增（§4-T9） |
| A11 | 整批費用一致性驗證未實作（規則兩版皆有） | crate 無對應檢查 | 新增（§4-T8） |
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
  - 台幣放行（req）：payeeName≤78（:251-252）、memo≤78（:271）、remark≤36（:285-286）、payerId≥8 碼（:246-247）、operatorName≤12（:220）、flowName≤10（:229-232）、detailList 三金額 15.2（:295-298）。
  - 台幣明細查詢（resp 解析）：payeeName 80、remark 38、detailList 15.2（:344/:363/:373-379，§1-27）。
  - 外幣：operatorName≤32（:2453）、flowName≤64（:2464）、bargainNo≤18（:1602）、feeDebitAccountNo 12~17（:1613-1614）。
  - 違規回傳結構化 Err（欄位名＋原因），不入電文。
- **T7 content 分流檢核**：台幣放行——paymentType=1 且 verificationType=2 → 檔案格式（媒體檔產生器輸出）；其餘 → json（V29.txt:238-242）；formatType=1 僅允許單筆付款（V29.txt:212-213）；外幣 formatType 僅 2（:1287/:2446）。
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

## 6. 風險與 V2.9 文件模糊處

### 6.1 「V2.9 為標準但不亂抄」六項邏輯裁決
V2.9 為實作基準，但其文件自身有模糊/看似矛盾處，逐條以證據裁決（不可全盤照抄）：

| # | V2.9 表面現象 | 證據 | 裁決與推導 |
|---|---|---|---|
| 1 | 媒體檔付款統編「至少 10 碼」vs json `payerId`「至少 8 碼」 | V29.txt:3633-3634 vs :246-247 | **非矛盾**：BIG-5 媒體檔與 json 為兩通道，媒體檔規格兩版未變（V28:3508-3509）。裁決：驗證分通道——媒體檔 ≥10 碼、json ≥8 碼（統一編號本體 8 碼；媒體檔屬歸戶登記規則，保留原文）。 |
| 2 | 台幣 operatorName 12/flowName 10 vs 外幣 32/64 | V29.txt:220/229 vs :2453/2464 | **非筆誤**：兩節並存於同一 V2.9 文件，且兩版外幣皆 32/64（V28:2468/2479）。裁決：驗證依 API 分流（台幣規則集/外幣規則集），禁止統一取值。 |
| 3 | 範例用 `formetType`，表格用 `formatType` | V29.txt:2522 vs :212/:1287/:2446（V28:1906 同錯） | **範例拼字錯誤**：表格為規格本體（兩版一致）。裁決：送出一律 `formatType`；解析端容錯兩種拼字（P2-T12）。 |
| 4 | 外幣檢核 Response 範例鍵名 PascalCase，表格欄位名 camelCase | V29.txt:2005-2074 vs :1688-1852 | **範例呈現差異**：表格為權威。裁決：實作以 camelCase 為主，反序列化加大小寫容差（P2-T11），防真實元件不一致。 |
| 5 | 憑證下載失敗範例回 999999＋moreInfo「applyDate 為必填」 | V29.txt:3612-3615 vs 910093（:3713） | **兩種回報路徑並存**：910093 為元件欄位檢核、999999 為電文層失敗兜底。裁決：錯誤處理同時支援（moreInfo 帶欄位名時解析顯示），不可只映射其一。 |
| 6 | 2-Way SSL/憑證檔全數移除，但 910067（解密失敗）、910074（放行憑證查詢異常）仍在 | V29.txt:3704、:3711；§0.3 0 命中 | **訊息層安控仍在、傳輸層雙向認證移除**：910067 指電文解密（元件內）、910074 指放行憑證查詢（非 TLS 憑證）。裁決：客戶端 TLS 材料（client cert/CA/pinning）維持 opt-in 能力不刪除（http.rs 現況 A5），預設不啟用；待 UAT 實證元件行為。 |

### 6.2 v2→v3 自我勘誤（可追溯原則的示範）
| # | v2 原記載 | 判定 | v3 修正 |
|---|---|---|---|
| E1 | 「台幣 Response 長度仍寫 80/38 vs Request 78/36」為 V2.9 自相矛盾 | ❌ 誤判 | `twdTxList`（付款放行 req）與 `txRecordList`（**交易明細查詢 API** resp）是不同 API 的不同欄位集；txRecordList 的 payeeName 80/remark 38 兩版皆未變（V28:360/379＝V29:344/363），與放行 req 78/36（V29:251/285）無衝突。§1 新增第 27 列記載。 |
| E2 | §1-19 稱 V2.8 content「僅說檔案轉 Base64」 | ⚠️ 不精確 | V2.8:248-250 原文已含「台幣付款資料格式參台幣付款資料(json)」；V2.9 的實質變更是**加入明確分流條件**（paymentType=1 且 verificationType=2），消除歧義，而非從「檔案」改為「分流」。 |
| E3 | §1-22 僅記外幣查詢範例 910070（:3153） | ⚠️ 不完整 | 台幣查詢範例同誤用 910070（V28.txt:1008）；V2.9 兩處皆改 910073（:987-988、:3135-3136）。 |

### 6.3 殘餘風險
- 2-Way SSL 移除為文件層面結論；元件實際行為未經 UAT 實證（UNKNOWN ≠ PASS，C-10）→ TLS 材料維持 opt-in（§6.1-6）。
- UAT 實測（T13）前，所有「元件將接受/拒絕 X」的推導均屬待驗證假設，不得宣稱通過。

## 7. 決策紀錄（已定案）
- sourceSystem：**可選開關、預設關閉**（非硬刪）——向後相容舊元件回測＋避免 V2.9 拒收。
- mTLS/pinning：**程式碼不動**（本就 opt-in），僅文件對齊。
- 驗證層：**新增模組**，規則集依台幣/外幣分流。
- 文件：api-spec 升級 V2.9（舊檔歸檔標記）；歷史計畫 A13 維持 Superseded。

## 8. 範圍外
- 約定書條款法律審閱（僅技術比對；`玉山全球智匯網—ERP系統直連服務約定書_已簊.txt` 已萃取可供日後引用）。
- eToken/放行主管電腦端流程（屬安控元件側，非本 crate 範圍）。
- UAT 實際連線執行（需約定帳號與環境，屬 T13/W1 腳本的執行期事項）。

## 9. 完成狀態與剩餘工作（2026-11-15）

### 9.1 已完成並驗證（T1-T12＋審查修正）
- T1-T12 全部落碼：`api.rs`（sourceSystem Option、bargainNo 18、錯誤碼全表＋classify、FxVoucherDownload 型別、PascalCase alias、`foreignTxList` 鍵、formetType 容差）、`validation.rs`（新模組，台幣/外幣規則集＋content 分流＋整批費用一致性＋憑證參數驗證）、`http.rs`（EsunEnvironment 8443/8080、`for_environment` host 驗證、`download_fx_remittance_voucher`、Source-System 條件式＋控制字元防護）、`bankpay.rs`（config Option/verification/flow/payment_type＋`with_bank_verification`/`with_batch_payments`、build→Result 驗證閘門、`download_fx_voucher`/`validate_fx_payment_request`、批次時間戳上提、`format_findings`）、`wellfit_cli` 接線、T5 文件五件套。
- code review 8 項全修（1 CRITICAL：`file_body` 補 `rename="fileBody"`＋round-trip 斷言；2 WARNING：export 驗證閘門移出 client 判斷、invoiceReturn 允許 0；5 SUGGESTION 全做）。
- **驗證（build.ps1 實測 2026-11-15）**：`clippy --workspace --all-targets -- -D warnings` 零警告；`test --workspace --no-fail-fast` **484 passed / 0 failed / 10 ignored**（`wellfit_bankpay` 101 單元＋18 整合全數通過）。
- 既有測試同步：2 處 RqHeader＋11 處 Source-System 測試建構、3 個 builder 測試改 config 形式＋新增 file-channel 拒絕測試；`wellfit_cli` 呼叫點接線。

### 9.2 剩餘工作（待實作清單）
- **W1＝T13 冒煙測試腳本（Rust-first，C-11）**：新增 `Wellfit/crates/wellfit_bankpay/tests/uat_smoke.rs`——`#[ignore = "requires UAT component and credentials"]`；環境變數驅動：`ESUN_UAT_BASE_URL`（預設 `http://219.87.16.166:8443`，V29.txt:87-89）、`ESUN_CUSTOMER_ID`、`ESUN_PAYER_ACCOUNT`、`ESUN_PAYER_ID`；9 支 API 逐支 happy path＋欄位錯誤 path（預期 910093/910094/910071 類回應）；一律走既有 `EsunHttpClient`（禁自創 HTTP）；證據輸出 JSON（含 SHA-256）至 `artifacts/<session>/`。**付款類 API（createTwdPayment/createFxPaymentInstruction）預設跳過，需 `ESUN_ALLOW_PAYMENT=1` 才執行**（避免測試環境誤觸真實放行）。
- **W2＝T13 執行手冊＋報告範本**：新增 `Wellfit/docs/bankpay/10-uat-smoke-runbook.md`（沿襲 `09-sandbox-acceptance-runbook.md` 系列編號）——前置條件（網路/帳號/元件版次）、build.ps1 執行指令（`test -p wellfit_bankpay --test uat_smoke -- --ignored --nocapture`）、Evidence-Driven **A–L 節**報告範本與五級判定（C-13：PASS/CONDITIONAL PASS/BLOCKED/REJECTED/NOT VERIFIED）。
- **W3＝追溯登錄（本計畫 §5 收尾）**：新測試 ID ↔ V2.9 規格行號（`V29.txt:nnn`）對照表，登錄 `Knowledge/traceability/claims-vs-evidence.md`（附加 esun-v2.9 區段，含 484/0/10 驗證證據行與指令）；同步 `Knowledge/specs/bankpay-contract.md`（KNB-SPC-006：V2.8→V2.9 契約事實）與 `Knowledge/adr/ADR-0013-bankpay-integration-and-contract-boundary.md` 錨定註記。
- **W4＝文件漂移修正（已實證）**：
  - `Wellfit/docs/TRACEABILITY_MATRIX.md`：26 處 bankpay 行號漂移（`bankpay.rs:327/392/404/510/559/784` 等重構後位移）＋ `build_twd_payment_request_format1/2` 描述過時（現為 config 形式、回傳 Result、format1 需 verification=2）＋新增 V2.9 能力列（validation.rs、API #9、EsunEnvironment、910093/094、foreignTxList 鍵）。
  - `Wellfit/docs/CODE_REFERENCE.md`：§173 `build_twd_payment_request` 描述過時（新簽名/Result）；**§379-383「CLI Export 參數被 `_` 忽略」已過時**（`main.rs:2164-2181` 已全部接線）——改為已接線並移除「⚠ 待確認」；補 `validation`/`download_fx_remittance_voucher`/`EsunEnvironment` 條目。
  - `Wellfit/docs/bankpay/04-interface-specification.md`：錨定 V2.8，加 V2.9 對照註記（指向 `esun-bank-flaps-erp-integration/api-spec-v29.md`）。
- **W5＝重驗證**：W1 新測試檔納入後重跑 clippy＋test（ignored 預設不執行，計數將 +1 ignored）；W2-W4 純文件；若 `specs/bankpay-contract.md` 變更，跑 `scripts/spec-drift-check.ps1` 確認無漂移。

### 9.3 範圍外（不變）
- UAT 實際連線執行（W1 腳本的手冊執行期事項，需帳號/憑證/環境）。
