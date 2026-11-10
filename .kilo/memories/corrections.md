> 強制規則（C-01..C-12；ID 前綴 `C-` 為 correction，與 plan C-01..23 不同命名空間）。
> 每條以單行六欄表（ID＋五內容欄）固定格式；編號一經賦予不得改序；作廢者標 SUPERSEDED 不刪號。

| 規則 ID | 規則 | 為什麼 | 證據 | 驗證 | 狀態 |
|---|---|---|---|---|---|
| C-01 | 不亂覆蓋亂刪（修訂先保留被取代內容，標 SUPERSEDED 就地歸檔） | plan 檔修訂曾可能遺失既有條文 | `1792865422970-* §0 變更管制（第 4 條）`；`§25.6-archived` 範例 | `§24` 修訂紀錄完整且舊文仍存 | ACTIVE |
| C-02 | 修訂記於 §24（歷史編號保留於軌跡，掃描器排除 §24 區塊） | 修訂軌跡與現行主張需分離 | `1792865422970-* §24`（SLICE-G18、QG-19 處於歷史列） | `§24` 外故有 ID 引用零懸空（掃描器排除 §24） | ACTIVE |
| C-03 | 先量測後動刀（編輯前先機器掃描全檔 REGISTER，未量測之聲明不得提出） | r2 覆蓋聲明未經機器驗證即提出，事後發現五項不實 | `1792865422970-* §25.7` 方法論；`§25.8-C` 實測對照表 | 引用圖＋語義覆蓋重算結果與聲明一致 | ACTIVE |
| C-04 | 斷言式補丁（每次檔改先斷言目標字串存在且唯一，失敗即中止；禁正則大範圍替換） | 避免誤命中與靜默覆蓋 | `1792865422970-* §25.8-E`；r3 全程斷言式組裝 | 每處補丁有存在性與唯一性斷言紀錄 | ACTIVE |
| C-05 | 並行編輯先定基準（動手前重測行數/大小/mtime 與已知狀態對帳；實證宣稱 grep 回驗） | 並行 session 編輯同一檔案（§22）期間發生覆蓋風險 | `platform/src/api.rs` 測試名 grep 10/11 命中（`states_page_covers_eight_data_states` 等） | 行數/mtime 與已知狀態對帳一致，實證宣稱有命中紀錄 | ACTIVE |
| C-06 | 引用完整性稽核（每個 `§N(.M)` 與 ID 引用須解析到定義；懸空先查出處） | v3.1 重構後子節引用（12.1-12.3、6.3 等舊子節號，已修）語義錯位 | `1792865422970-* §25.8-B`；`§5`「憲章 §2.2」跨文件甄別 | 引用圖掃描：現行引用零懸空（跨文件引用已白名單） | ACTIVE |
| C-07 | 語義覆蓋≠ID 存在（斜線/範圍簡寫展開後計數） | 誤報 21/24 實為 23/24 | `1792865422970-* §25.8-C/E`（`RIG-06/07/11/12/13`、`PSN-01..08` 展開） | 展開計數結果與聲明一致 | ACTIVE |
| C-08 | 偽影甄別（掃描器失敗先區分假陽性與真錯誤，再決定修文件或修掃描器） | 字串多一空格／歷史記錄區塊誤判為文件錯誤 | deep3 三項失敗經 deep4 甄別全為偽影（`§24` 歷史列） | 每項失敗有偽影或真錯誤之判定紀錄 | ACTIVE |
| C-09 | 寫入通道（PowerShell 5.1：heredoc 不支援；中文佔位；寫檔先 here-string 寫 temp，再由 PowerShell 原生 `[IO.File]::AppendAllText` 或 Rust 工具併入）〔原「Python append」SUPERSEDED 2026-11-10（Rust-first，見 C-11）；原句保留於軌跡〕 | heredoc／BOM 節入事件將檔案截斷至 41 行 | `WriteAllText` 需 `[Text.UTF8Encoding]::new($false)` 防 BOM | 寫入後檔案可讀、行數與預期一致 | ACTIVE |
| C-10 | 誠實註記（未量測標 UNKNOWN，`UNKNOWN ≠ PASS ≠ FAIL`；數字聲明附方法與日期） | `PLAN_REGISTER.md:174` 登載「plan stub」負面紀錄（聲明與實況不符） | `1792865422970-* §0 證據紀律（第 2 條）`；`§25.8` 全部數字附實測日期 | 數字聲明皆有量測方法與日期可追溯 | ACTIVE |
| C-11 | Rust-first 工件規範（新增工具與開發一律 Rust；不得新增 Python 工件；既存 `.ps1` 工具鏈為例外） | 2026-11-10 使用者指令「必須嚴格遵守 Rust-first」；原 Python 版 lint 已刪、改以 Rust 實作（D-05） | `tools/memory_lint/`（零依賴 Rust crate）；`.kilo/plans/1792461065184-*` 2026-11-10 更新段 | 新增檔案清單（`git status`）掃描零 `*.py`；本列修訂時更新 | ACTIVE |
| C-12 | 同名 ID 前綴多檔防呆（以 ID 前綴 glob 挑檔時，須以「含結構標題＋最大檔」剔除舊 stub；挑選邏輯須有測試） | v2.0 首跑誤選 stub（`quality-restoration-plan`／`unified-approval-channel`）致引用全數誤判 | `tools/memory_lint/src/lib.rs`（`_pick_plan`）；`.kilo/sessions/20261110-dev-memory-v2/manifest.md` Notes | `tests/negative.rs` fixture 含 stub 檔仍正確挑選（6/6 綠）；或手動印出被選檔核對 | ACTIVE |
