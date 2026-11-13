# EXPERIENCE_SCENARIO_MATRIX — 角色×任務×情境（v1.0；2026-11-13）

> 覆蓋規則：① 關鍵路徑（P0 組合）全測；② 其餘 pairwise 抽樣（每對維度值至少一次）；③ **禁止留空**——未覆蓋標 UNKNOWN 並附解鎖條件。
> 組合空間是級數，本矩陣承諾「可驗證子集＋擴展律」，不承諾全枚舉（與體驗室 `/design/matrices` 同步）。

## 維度

| 維度 | 取值 |
|---|---|
| 角色 | V 訪客／C 經辦／R 覆核／M 主管／A 管理員／T 技術／U 稽核 |
| 任務 | 每角色 5–8 高頻任務（見關鍵路徑） |
| 狀態 | empty／zero／loading／partial／stale／denied／error／conflict |
| 裝置 | desktop ≥1280／laptop 1024／tablet 768／phone 390 |
| 網路 | LAN／WAN 高延遲／離線（殼） |
| 資料量 | 0／20／2k／100k |
| 併發 | 1／10／50 |
| 輸入 | 滑鼠／鍵盤／觸控 |
| 輔助 | 預設／reduced-motion／200%／高對比／螢幕閱讀器 |

## 關鍵路徑（CP；全測）

| CP | 角色 | 任務 | 預期 | 驗證（測試／證據） | 判定 |
|---|---|---|---|---|---|
| CP-01 | V | 未登入存取受保護頁（全頁） | 303 → login（Public 三頁除外） | web_crawl::auth_gate_covers_every_page；page_matrix visitor 樣本 | PASS |
| CP-02 | V | 登入失敗與鎖定 | 錯誤三要素文案；連續失敗鎖定 | 手動實證（#44 實證輪）＋auth 機制測試（platform --lib 146/0 範圍） | PASS |
| CP-03 | C | 匯入（CSV/XLSX/PDF/DOCX/EML）→預覽→確認 | 端到端入帳；冪等重跑拒絕 | import_xlsx／import_pdf／import_docx／import_eml 端到端測試 | PASS |
| CP-04 | C | 草稿→登帳（金額/科目帶入） | 金額不失真、科目正確、帶解析證據 | #44 修復實證（瀏覽器）＋2 迴歸測試 | PASS |
| CP-05 | R | 批量裁決（具名筆數二段確認） | 裁決成立＋稽核事件 | 批量裁決測試（platform 整合套件）＋審計狀態測試 | PASS |
| CP-06 | R | 併發衝突（他人已處理） | 409 語義；不重複入帳 | audit_states::double_claim_fails_cleanly_and_audits_partial_outcome | PASS |
| CP-07 | A | 忘記密碼核發→重設全鏈 | 一次性 token（30 分、用後即焚） | password_reset 2/2（TT-6） | PASS |
| CP-08 | A | AI 旗標治理（開/關＋稽核） | toggle 生效且留稽核 | #44 修復實證（curl） | PASS |
| CP-09 | T | 體驗室閘矩陣（十頁） | admin 200／store、finance 403／訪客 303 | design_lab 2/2 | PASS |
| CP-10 | V/T | 未知路徑 404 品牌頁 | 識別＋問題/原因/處理＋返回；零 JS | web_crawl 404 測試（TT-8） | PASS |
| CP-11 | 全角色 | 全站內部連結 | 0 死鏈（BFS ≥30 連結） | web_crawl::every_internal_link_resolves（TT-7） | PASS |
| CP-12 | M | 月報（2k 交易） | p95 < 500ms | close_out::monthly_report_stays_under_500ms_with_2000_transactions | PASS |

## Pairwise 抽樣計畫（非關鍵組合）

- 樣本軸：裝置×輔助×狀態（如 phone×reduced-motion×error；tablet×200%×empty）。
- 抽樣責任：E-5 全端一致性輪執行；結果回填本檔「抽樣紀錄」小節（現況：未執行，屬 UNKNOWN XP-007）。
- 每輪至少覆蓋：4 裝置各 ≥1；輔助 5 值各 ≥1；八態每態 ≥2 角色情境。

## UNKNOWN 登錄

| UNKNOWN | 解鎖條件 |
|---|---|
| R6 端到端全走查（E2E 全鏈含失敗注入） | 排定 R6 驗證窗口（XP-007） |
| R7 跨裝置/列印（tablet/phone/列印走查） | 排定 R7 窗口（XP-002/XP-007） |
| R8 效能資源（頁面級 bytes/位移；S2 桌面效能） | 排定 R8 窗口；E-2 預算表已訂 |

## 抽樣紀錄

（尚未執行；E-5 起逐輪回填日期、樣本清單、結果。）
