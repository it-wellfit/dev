# REDESIGN TEST PLAN v2.0 — 全面測試驗證計畫

> 編號：TEST-RD-2.0｜日期：2026-11-06｜對應：REDESIGN_BLUEPRINT_v2.0.md
> 證據規則：每項測試記錄「指令＋退出碼」；UNKNOWN 不等於 PASS。flaky 測試視同缺陷（P1-4 先例）。

## 1. 單元測試（Unit）

| 目標 | 案例 | 斷言 |
|---|---|---|
| 分頁參數解析 | parse_page（0/負數/非數字/巨大值/正常） | 全部回合法頁碼；非數字→1；越界 clamp |
| 金額格式 | format_minor 邊界（0/負/最大 i64） | 字串精確比對（既有 6 案延續） |
| 換算（M4） | i128 中間值換算、四捨五入 | 100 USD @31250 → 312,500 分 |
| 健康檢查組裝 | degraded/ok payload 組裝 | JSON 欄位齊備 |

## 2. 整合測試（Integration，router oneshot）

| 目標 | 案例 | 斷言 |
|---|---|---|
| 分頁 | 建立 25 筆後 GET / 與 GET /?page=2 | 第 1 頁含 20 列、第 2 頁 5 列；分頁列文字「第 1 / 2 頁・共 25 筆」 |
| 分頁越界 | GET /?page=999 | 回末頁（顯示第 2 頁） |
| 分頁非法 | GET /?page=abc | 回第 1 頁（不 500） |
| 健康檢查 | GET /api/health | 200 ok＋migrations 欄位；關閉 DB 後 503 degraded |
| Secure cookie | POST /login 成功 | Set-Cookie 含 HttpOnly、SameSite=Lax、Secure（HTTPS 綁定時） |
| 回歸 | 既有 96+ 測試全數 | 0 failed（每步 R1 之後重跑） |

## 3. 端到端（E2E，真實伺服器）

- seed-demo → 瀏覽 3 頁（總覽/報表/管理）→ 建立交易 → 分頁導覽 → reset-demo；全程 200/303、無 5xx。
- 腳本：scripts/e2e-smoke.ps1（既有 demo 腳本延伸分頁步驟）。

## 4. 效能（Performance）

| 案例 | 預算 | 方法 |
|---|---|---|
| 月報表 2,000 交易 | < 500ms | 既有 close_out 測試（RPT-001） |
| 首頁 2,000 交易含分頁 | < 500ms（20 筆/頁應顯著低於全量） | 新增 close_out 案例：seed 2,000 筆後 GET /?page=1 計時 |
| Lighthouse 首頁 | Accessibility ≥ 95 | Playwright+Lighthouse 基線量測一次，記錄 JSON 於 artifacts |

## 5. 安全（Security）

| 案例 | 斷言 |
|---|---|
| 分頁參數注入 | page=1%27OR 等惡意字串 → 當作非法回第 1 頁（bind 綁定，無 SQL 拼接） |
| CSRF 回歸 | 10/10 既有案例全過；新增 POST（若有）同數補齊 |
| 安全標頭 | CSP/nosniff/frame-deny/no-store 全回應（既有 middleware 回歸） |
| 帳號邊界 | STORE 403 管理/規則頁；停用會話即撤銷（既有） |

## 6. 可用性（Usability）

| 案例 | 方法 | 通過準則 |
|---|---|---|
| 鍵盤導覽 | Playwright Tab 序 | skip-link→nav→main 順序可達；focus ring 可見（--focus-ring） |
| 螢幕閱讀器語意 | 標題層級/landmark 審查 | h1→h2→h3 無跳級；nav/main/footer landmarks |
| 分頁可用性 | 手動＋截圖 | 上一頁/下一頁在首/末頁正確禁用；頁碼文字正確 |
| 減少動態 | prefers-reduced-motion 模擬 | spinner 停止動畫（既有 CSS） |

## 7. 執行記錄（2026-11-06 本輪）

- 分頁 3 條整合測試＋健康檢查強化＋Secure cookie：編寫後執行。
- 全域關卡：fmt/clippy/test/audit（結果見 REDESIGN 執行報告）。
- 尚未執行（Pending）：Lighthouse 基線、E2E 腳本延伸、soak（既有 ignored 標記不變）。