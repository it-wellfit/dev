# 工作區治理準則 (Workspace Governance Standard)

**版本**: v1.0.0
**生效日期**: 2026-11-02
**狀態**: Active
**負責人**: 工作區擁有者（stephanie.huang）
**審查週期**: 每季；或重大事件（架構變更、重複違規、新子專案加入）後 7 日內
**上位文件**: [WORKSPACE_CHARTER.md](../../WORKSPACE_CHARTER.md)（憲政與導航）
**同位文件**: [SHARED_STANDARDS.md](SHARED_STANDARDS.md)（編碼／建置／測試／安全基線，本準則不重複其內容）

---

## 0. 目的與適用範圍

本準則是工作區**管理體系**的唯一 canonical 標準，管轄四個系統：

| 系統 | 管轄對象 | 下位 owner 文件 |
|------|----------|----------------|
| 檔案管理 (File) | session 產物、測試／執行輸出、匯入匯出檔案、建置目標 | `.kilo/plans/1793585665396-session-file-management.md`（實作規格，歷史提案） |
| 文件管理 (Document) | 文件類型、生命週期、註冊表、取代協定 | 本準則 §3；各專案 `docs/README.md` 為該專案文件地圖 |
| 知識管理 (Knowledge) | 認識標籤、來源治理、ADR、詞彙表、接手程序 | `docs/README.md`（導航）、`../Knowledge/README.md`（跨專案知識庫） |
| 工程管理 (Engineering) | 工作分解、風險分級、證據規則、發布與備份 | `SHARED_STANDARDS.md` §5–§8、§10；`docs/PLAN_REGISTER.md`（計畫登錄） |

**效力順位**：憲章 (WORKSPACE_CHARTER) > 本準則 > 共享標準 (SHARED_STANDARDS) > 子專案 AGENTS.md > 個別計畫／規格。
子專案文件與本準則衝突時，以本準則為準，除非子專案經 ADR 明示豁免並登錄於 §8 關係地圖。

---

## 1. 核心原則（十項）

| # | 原則 | 內容 | 違反時的後果 |
|---|------|------|--------------|
| P1 | 單一事實來源 (SSOT) | 每類資訊只有一份 owner 文件；其他文件僅可連結並陳述本地後果，不得複製規則 | 複製件視為遺棄，標記 Superseded |
| P2 | 修訂綁定證據 | 一切狀態宣稱（編譯、測試、發布、完成）必須綁定：具名 git 修訂版 + 精確指令 + 工作目錄 + 退出碼 | 無修訂版的宣稱一律視為 UNKNOWN，不得寫為 PASS |
| P3 | 具名所有權 | 每份文件、每個計畫、每個決策、每項阻塞必須有具名負責人；無負責人 = Blocked | 登錄表標記 Blocked，不得執行 |
| P4 | 顯式生命週期 | 文件與計畫只能處於 Draft / Reviewed / Approved / Active / Deprecated / Superseded 之一；狀態必須寫在文件頭 | 無狀態標記的文件視為 Draft（提案） |
| P5 | 可追溯鏈 | 需求 → 規格 → 設計/ADR → 契約 → ticket → commit → 測試 → 發布證據 → retro，每環可回指上一環 | 斷鏈的能力宣稱不得出現在現行文件 |
| P6 | 最小複製 | 禁止建立第二份規則；整合靠連結與註冊表，不靠搬運 | 同 P1 |
| P7 | 反模擬 | 合成測試只驗證一致性，不驗證真實價值；`UNKNOWN != PASS` 且 `UNKNOWN != FAIL` | 違反者視為缺陷，立即修正文件 |
| P8 | 演進式治理 | 本準則自身依 §6 版本化與修訂程序演進；規則可增修，但須留下修訂軌跡 | 未依程序變更的版本無效 |
| P9 | 最小摩擦 | 自動化優於人工檢查；規則數量受控（不變量總數 ≤ 20，見 SHARED_STANDARDS §10） | 新規則須先證明無法以自動化取代人工 |
| P10 | 失敗可見 | 驗證失敗必須中止流程，不得靜默降級、跳過或改寫門檻 | 靜默降級視為最高等級違規 |

---

## 2. 檔案管理體系 (File Management)

### 2.1 空間分層

| 層 | 路徑 | 內容 | 版控 |
|----|------|------|------|
| Registry | `.kilo/sessions/<session-id>/` | manifest、conversation 摘要、測試報告（小型 md/json） | 追蹤（runs/ 與 *.log 除外） |
| 專案產物 | `<project>/artifacts/<session-id>/` | tests/、runs/、io/{input,output} | io manifest 追蹤；重產物忽略 |
| 建置目標 | `%LOCALAPPDATA%\Temp\wellfit-target` 等庫外路徑 | 二進位產物（桌面路徑受應用程式控制原則限制） | 永不入版控 |
| 禁區 | 桌面、工作區根目錄 | **禁止**任何 session 產出落地 | `session-validate.ps1` 偵測 |

### 2.2 Session 生命週期

1. **開始**：`scripts/session-start.ps1 -SessionId <id> -Projects <p1,p2> -Goal "..."` → 產生目錄、manifest 骨架、記錄起始修訂版與 dirty 旗標。
2. **進行中**：所有產出寫入 `$env:SESSION_ROOT` 或 `$env:ARTIFACT_ROOT`；指令與退出碼即時記入 manifest「Commands run」。
3. **結束**：`scripts/session-end.ps1 -SessionId <id>` → 回填結束修訂版、更新 `.kilo/sessions/index.md`、跑 `scripts/session-validate.ps1`。
4. **提交**：僅在使用者明確要求時 commit；訊息引用 session ID 與修訂版。

### 2.3 命名規則

| 物件 | 模式 | 範例 |
|------|------|------|
| Session ID | `YYYYMMDD-<project>-<slug>`（小寫、連字號、slug ≤ 32 字元；跨專案用 `multi`） | `20261102-wellfit-bankpay-import` |
| 對話摘要 | `<nn>-<topic>.md` | `01-scope-and-decisions.md` |
| 測試日誌 | `<YYYYMMDD-HHMM>-<tool>-<short>.log` | `20261102-0930-cargo-test.log` |
| 測試報告 | `test-report-<suite>-<YYYYMMDD>.md` | `test-report-wellfit-workspace-20261102.md` |
| 執行日誌 | `<YYYYMMDD-HHMM>-<command-slug>.log` | `20261102-1015-wellfit-mtrix-run.log` |
| IO 快照 | 沿用原始檔名；碰撞時加 `<session-id>__` 前綴 | `shopee_orders.csv` |

通用限制：小寫 ASCII、連字號、無空格、無中文檔名（腳本相容性）；單檔名 ≤ 80 字元。

### 2.4 匯入／匯出流程

- 匯入：原始檔 → 複製快照至 `io/input/` → 於 `io/manifest.md` 記錄 SHA-256 與來源；重跑一律使用快照，不碰原始檔。
- 匯出：產出寫至 `io/output/` → 記錄 SHA-256 與產生指令；指令與修訂版留在 `runs/`。
- 機敏資料（客戶代碼、價格、帳戶）：`io/input|output/` 已 gitignore；提交前必跑 `session-validate.ps1 -Strict`。

### 2.5 保留與清理

| 物件 | 保留政策 |
|------|----------|
| manifest、index、conversation 摘要、測試報告、io manifest | 永久（隨版控） |
| runs/ 日誌、tests/*.log、io/input|output/ | 30 天後由 `session-cleanup.ps1` 清除（可調） |
| 建置目標 | 手動或專案腳本管理，不入版控 |

### 2.6 遷移原則

既有散落檔案採「分類 → 搬移至 `archive/legacy-root/` 或所屬專案 → 更新入站連結」；**不刪除**。無法判定歸屬者留在 archive 並標記待審。

---

## 3. 文件管理體系 (Document Management)

### 3.1 文件類型與責任

| 類型 | Owner 位置 | 註冊表 |
|------|-----------|--------|
| 憲政文件 | `WORKSPACE_CHARTER.md` | 自身 |
| 管理準則 | 本文件 | 本文件頭 + 憲章 §4 |
| 工程基線 | `SHARED_STANDARDS.md` | 憲章 §4 |
| 計畫／提案 | 各專案計畫文件 | `docs/PLAN_REGISTER.md` |
| 架構決策 | 各專案 `docs/adr/` | 各專案 `ADR_REGISTER.md`；跨專案決策在 `../Knowledge/adr/` |
| 文件地圖 | 各專案 `docs/README.md` | 自身 |
| 追溯證據 | 各專案 `TRACEABILITY.md` / `docs/traceability/` | 自身 |
| Session 記錄 | `.kilo/sessions/` | `index.md` |

### 3.2 文件頭必備欄位

每份正式文件必須有：版本、日期、狀態、負責人、變更摘要；規格類文件另須：驗證標記（✔ 逐字查證 / △ 推導 / 未驗證）。缺任一欄位者視為 Draft。

### 3.3 取代協定 (Supersession)

- 被取代文件**原地**標記 `Superseded (YYYY-MM-DD)` 並給出**唯一**替代文件連結。
- 禁止刪除或批次搬移有入站連結的文件；搬移前先盤點並更新連結。
- 歷史／備份目錄（`archive/`、`skills.backup-*`）不納入現行索引。

### 3.4 衝突仲裁

1. 先查 §0 效力順位；上位文件勝。
2. 同位文件衝突 → 兩者皆標 Blocked，取得負責人裁定前不得以合併文字掩蓋。
3. 仲裁結果寫入 ADR（若為持久工程決策）或直接修訂下位文件（若為文書更正）。

### 3.5 文件結構參考

專案 docs 目錄可採 Diátaxis 四象限（learning / how-to / reference / explanation）組織；以該專案 `docs/README.md` 為地圖 owner，本準則不強制統一目錄名。

---

## 4. 知識管理體系 (Knowledge Management)

### 4.1 認識標籤 (Epistemic Tags)

所有關鍵陳述須帶標籤，定義以 [docs/README.md §3](../README.md) 為 owner：
`[FACT:CODE]`（附原始碼路徑）、`[FACT:DOC]`、`[INFER]`、`[PROP]`、`[LIMIT]`、`[GAP]`。
文件間引用標籤定義時必須連結，不得改寫定義。

### 4.2 來源治理

- 外部來源必須登錄於來源登錄（`../Knowledge/sources/source-register.md` 或專案等效文件），含取得日期與 SHA-256。
- 無法取得的來源記為 `UNKNOWN`，**不得**轉寫為已驗證。
- 網路研究遵循研究規約（`../Knowledge/sources/research-protocol.md`）：可查什麼、不可查什麼、如何引用。

### 4.3 決策治理

- 持久工程決策 → ADR（格式與規則見 [SHARED_STANDARDS.md §9](SHARED_STANDARDS.md)）。
- ADR 唯有在「決策負責人 + 理由 + 日期 + 實作證據」四者齊備時才算 Accepted；索引列不等於核准。
- 產品／業務決策 → 各專案 governance/decisions 文件；跨專案裁決 → `../Knowledge/adr/`。

### 4.4 詞彙表

領域術語以單一 glossary 為 SSOT（如 `docs/03-reference/domain-glossary.md`）；新術語先進詞彙表再使用於文件。

### 4.5 接手程序（新 session／新代理人標準動作）

1. 讀憲章與本準則（§0 效力順位）。
2. 讀目標專案 `AGENTS.md` 與 `docs/README.md`。
3. 查計畫登錄（`docs/PLAN_REGISTER.md`）確認目標計畫狀態與阻塞。
4. 查 ADR 登錄與議題登錄（如存在）。
5. 跑專案基線檢查確認當下修訂版狀態（結果綁修訂版記錄）。
6. 之後才開始讀程式碼。

---

## 5. 工程管理體系 (Software Engineering Management)

### 5.1 工作分解

- 工作以 ticket（GitHub Issue）為單位；相依構成 DAG，禁循環。
- 1 ticket = 1 context window：單一 session 只承載一個 ticket 的實作與驗證。
- 契約先行：先定契約與測試，後寫實作；契約變更須先更新知識庫與規格，再改程式。

### 5.2 風險分級與門檻

以 [SHARED_STANDARDS.md §6](SHARED_STANDARDS.md) 為 owner（Low/Medium/High 三級 + 基線檢查六項）。本準則僅補充一條程序規則：**門檻不得為使現狀看起來合格而調降**；調降視同本準則 MAJOR 修訂，須走 §6 程序。

### 5.3 證據規則

- 檢查結果只有兩種合法狀態：`PASS`（精確指令 + 具名修訂版 + exit 0 三者齊備）或 `Not run / Blocked / UNKNOWN`。
- 先前的綠燈不通則當前 dirty 或更新後的樹；跨修訂版引用測試數字視為無效證據。
- 釋出／備份／審查發現記錄於專案 `docs/reviews/` 與 `docs/operations/backup-log.md`（如存在）。

### 5.4 變更流程

```
Issue → 規格/驗收條件 → (持久決策? ADR) → 分支 →
基線檢查 (fmt/clippy/test/selftest/docs/secret) →
commit (<type>: <subject> + Issue ref) → PR → 審查 → 合併 → (發布? SemVer tag + Release)
```

提交與分支命名規則以 [SHARED_STANDARDS.md §5](SHARED_STANDARDS.md) 為 owner。

### 5.5 發布與備份

- 發布：SemVer + annotated tag + GitHub Release（release notes、變更清單、建置產物）。
- 備份：`git bundle --branches --tags`、離線保存、每月還原演練並記錄（owner：SHARED_STANDARDS §8.1）。

### 5.6 營運節奏 (Operating Rhythm)

| 頻率 | 動作 |
|------|------|
| 每 session | §4.5 接手程序；結束時 session-end + validate |
| 每週 | 掃描 `.kilo/sessions/index.md` 與 PLAN_REGISTER 狀態漂移；清理逾保留期產物 |
| 每月 | 備份還原演練；`session-validate.ps1 -Strict` 全域審計 |
| 每季 | 本準則審查（§6.3）；register 完整性盤點 |
| 事件驅動 | 重複違規（同類 ≥ 3 次）→ 升級為程序修訂或不變量候選 |

---

## 6. 演進機制 (Evolution Mechanism)

### 6.1 版本語義

| 層級 | 意義 | 範例 |
|------|------|------|
| PATCH (v1.0.x) | 澄清措辭、修正連結、補範例；不改變義務 | 錯字、連結修復 |
| MINOR (v1.x.0) | 新增程序、新增下位文件、放寬現有程序；不觸及核心原則與效力順位 | 新增 register、調整保留期 |
| MAJOR (vX.0.0) | 修改核心原則 (§1)、效力順位、不變量集合、或使既有證據失效的變更 | 更換 SSOT 位置、調整 ≤20 不變量 |

### 6.2 修訂程序

1. **提案**：GitHub Issue 或 manifest 記錄（觸發原因、影響範圍、替代方案）。
2. **影響評估**：列出受影響的下位文件與 register；標出需同步修訂者。
3. **修訂稿**：更新本文件 + 文件頭版本/日期/變更摘要。
4. **生效**：負責人核准；同步修訂所有受影響下位文件（同一提交）。
5. **追溯**：修訂記錄留於本文件尾的「修訂歷史」表。

### 6.3 審查

每季固定審查一次；審查輸出：確認/修訂/廢止決議，記入修訂歷史。無變更亦須留「複審通過」記錄（日期 + 版本不變）。

### 6.4 廢止

本文件被取代時原地標記 Superseded 並指向唯一替代文件；不刪除。

### 6.5 度量（治理健康指標）

| 指標 | 來源 | 目標 |
|------|------|------|
| session-validate 錯誤數 | 腳本輸出 | 0 |
| 散落檔案警告數 | 腳本輸出 | 單調遞減；新增 = 0 |
| register 完整性 | index/PLAN_REGISTER 盤點 | 無孤兒（未登錄）文件 |
| 還原演練 | backup-log | 每月 ≥ 1 次且 PASS |

---

## 7. 合規與執行 (Compliance & Enforcement)

### 7.1 自動檢查鏈

| 檢查 | 腳本／指令 | 時機 | 失敗處置 |
|------|-----------|------|----------|
| Session 佈局 | `scripts/session-validate.ps1` | session 結束；每月 -Strict | 修正後重跑 |
| 機密掃描 | `scripts/check-secrets`（專案內）或等效 | 每次提交前 | 移除機密，輪換金鑰 |
| 文件連結 | `scripts/check-docs`（專案內）或等效 | 每次提交前 | 修復連結 |
| 工程基線 | cargo fmt/clippy/test（見 SHARED_STANDARDS §6.2） | 每次提交前 | 不得提交 |

### 7.2 檢查清單（複製即用）

**Session 開始**
```
[ ] session-start.ps1 已執行，SESSION_ROOT/ARTIFACT_ROOT 已設
[ ] 接手程序 §4.5 六步完成
[ ] manifest 記錄起始修訂版與 dirty 旗標
```

**Session 結束**
```
[ ] 所有產出都在 session/artifact 目錄內
[ ] 測試報告含指令、修訂版、退出碼
[ ] io/manifest.md 已更新（SHA-256）
[ ] session-end.ps1 + session-validate.ps1 通過
[ ] index.md 已更新
```

### 7.3 違規分級

| 級別 | 情形 | 處置 |
|------|------|------|
| L1 | 命名不符、manifest 欄位缺漏 | 當場修正 |
| L2 | 產出落於禁區（桌面／根目錄） | 搬移至正確位置 + 更新連結 + 記入 manifest |
| L3 | 無修訂版的 PASS 宣稱、門檻靜默降級、機密入版控 | 立即更正文件／輪換金鑰；記入修訂歷史；同類重複 ≥ 3 次觸發 §6 程序修訂 |

---

## 8. 關係地圖 (Document Relationship Map)

```
WORKSPACE_CHARTER.md（憲政）
├── 本文件 GOVERNANCE_STANDARD.md（管理體系：檔案/文件/知識/工程）
│   ├── SHARED_STANDARDS.md（工程基線 owner：§1-§8、§10 不變量）
│   ├── docs/PLAN_REGISTER.md（計畫登錄 owner）
│   ├── .kilo/sessions/index.md（session 登錄 owner）
│   ├── docs/README.md（知識導航與認識標籤 owner）
│   └── ../Knowledge/（跨專案知識庫：ADR/來源/追溯 owner）
└── 各子專案 AGENTS.md（執行入口）→ 各專案 docs/README.md（專案文件地圖）
```

---

## 9. 修訂歷史

| 版本 | 日期 | 變更摘要 | 負責人 |
|------|------|----------|--------|
| v1.0.0 | 2026-11-02 | 初版：整合 session 檔案管理實作成果，建立四系統治理、十項原則、演進機制與合規鏈 | 工作區擁有者 |

---

*本準則與 WORKSPACE_CHARTER.md、SHARED_STANDARDS.md 互補：憲章定效力順位，本準則定管理程序，共享標準定工程基線。子專案 AGENTS.md 為執行入口，不得與本準則衝突。*
