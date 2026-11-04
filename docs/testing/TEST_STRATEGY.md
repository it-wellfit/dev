# Test Strategy — Dev 根工作區

> 版本：v1.0｜日期：2026-11-03｜狀態：Active
> Owner：SHARED_STANDARDS §3／§6（測試標準與品質門檻）；本檔補充本 workspace 的測試矩陣與證據記錄方式。

## 1. 基線命令（提交前必過；PowerShell）

```powershell
cargo fmt -- --check;                        # EXIT 必須 = 0
cargo clippy --all-targets -- -D warnings;   # EXIT 必須 = 0（不得加 allow）
cargo test --locked;                         # 0 failed / 0 ignored
cargo audit;                                 # EXIT = 0（.cargo/audit.toml 記錄接受項）
```

證據記錄：指令＋工作目錄＋退出碼＋session log（`.kilo/sessions/<id>/tests/`）。

## 2. 2026-11-03 基線證據

| 檢查 | 結果 | 證據 |
|------|------|------|
| fmt --check | EXIT=0 | session runs log |
| clippy -D warnings | EXIT=0（修復 19 錯誤後） | `runs/20261103-1245-clippy-before.log`（before）／ADR-0001 §4 |
| test --locked | EXIT=0：flaps_bridge 7、flaps_core 2、platform 3（migration）、web 0 = 12 passed / 0 failed | `tests/20261103-1305-cargo-test.log`、`tests/20261103-1320-cargo-test-final.log` |
| audit | EXIT=0（h2 升級修復；rsa 文件化接受） | `runs/20261103-1306-cargo-audit-after.log` |

## 3. 測試矩陣（依風險選擇）

| 類型 | 範圍 | 工具 | 現狀 | 目標 |
|------|------|------|------|------|
| Static / Lint / Type | 全 workspace | fmt＋clippy -D warnings | ✅ 門檻運作 | 持續 |
| 單元測試 | core 領域規則 | cargo test --lib | flaps_core 6（permission 正反路徑＋money 邊界與 roundtrip） | 補 validate_journal_entry 負向案例（未來日期／負數／非法借貸別／空 items） |
| 遷移測試 | platform schema | cargo test（in-memory SQLite） | ✅ 4 passed（建表/重放冪等/舊庫相容/整數 roundtrip） | 新遷移一律附重放測試 |
| API 整合測試 | 路由→驗證→PRG→渲染 | axum oneshot（tower util） | ✅ 8 passed（303 PRG＋Location、403 RBAC、400 ×3、零 JS 渲染、health、CSS） | 補 500 路徑與冪等重放案例 |
| 橋接測試 | flaps_bridge 批次語義 | cargo test | ✅ 7 passed | flaky 效能測試改確定性斷言（Backlog P1-4） |
| E2E / UI | 零 JS 介面 | curl.exe 煙霧＋Integrated Browser | ✅ 端到端（POST 303→渲染真實資料）＋三分區實機截圖 | 1280/768/375px 三斷點自動化（Node 缺失，暫以手動走查） |
| 無障礙 | WCAG 2.2 AA | 鍵盤走查＋對比檢查 | ✅ skip-link／焦點可見／語義結構已驗 | 表單錯誤識別、螢幕閱讀器走查 |
| 安全測試 | T-02 SQL 注入、T-06 越權 | 負向測試 | permission code 判定回歸＋RBAC 403 整合測試 | TC-SQL-01：注入字串走 bind 不生效 |
| 回歸 | bug 修正 | — | permission.rs 語法修復已納入既有測試 | 每個 bug 先寫失敗測試（TDD） |

## 4. 測試規則（摘錄自 SHARED_STANDARDS §3.2，本地後果）

1. 新增功能必須有對應測試；bug 修正先寫回歸測試。
2. 禁止 `assert!(true)`；斷言綁具名變數。
3. 合成測試不通過不冒充驗收（AT-）案例；`UNKNOWN ≠ PASS`。
4. flaky 測試不得靜默重跑掩飾——記錄 backlog 並修復設計（見 P1-4）。
5. 測試不得依賴網路／時鐘精度；時間敏感斷言需裕度設計。

## 5. 覆蓋優先序

Critical User Journeys（USER_JOURNEYS.md 1–3）→ 帳務驗證規則 → RBAC 判定 → 遷移重放 → API 契約（G-1 後）→ UI 狀態樣式。
