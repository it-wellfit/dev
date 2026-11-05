# TECH STACK — 技術堆疊

> 版本以 `Cargo.lock` 為 SSOT（--locked 建置）。決策理由見 docs/adr/。

## 核心堆疊

| 層 | 技術 | 版本 | 選擇理由 |
|----|------|------|----------|
| 語言 | Rust | 1.99（rust-toolchain.toml） | 記憶體安全＋效能＋#![forbid(unsafe_code)] |
| Web | Axum | 0.7 | 零 JS 伺服器渲染＋middleware 生態 |
| 模板 | Askama | 0.16 | 編譯期型別檢查＋自動 HTML 跳脫 |
| DB | SQLite（SQLx） | 0.8（WAL） | 檔案式、零維運、單機交付 |
| 密碼 | argon2 | 0.5 | OWASP 首選 |
| 匯入 | csv | 1.3 | RFC 4180 解析 |
| 指紋 | sha2 | 0.10 | 匯入檔追溯 |

## 明確排除（理由）

| 技術 | 理由 |
|------|------|
| JavaScript／HTMX／WASM（表現層） | Strict Zero-JS 憲章 §2.3（HTMX/WASM 亦禁） |
| Node/npm 建置鏈 | 零前端依賴 |
| 浮點金額 | INV-01 |
| PostgreSQL（現階段） | 單機檔案式 DB 足夠；Stage 3 評估 |

## 品質工具鏈

| 工具 | 用途 | 門檻 |
|------|------|------|
| rustfmt | 格式 | --check EXIT=0 |
| clippy | lint | -D warnings EXIT=0 |
| cargo test | 測試 | 0 failed |
| cargo audit | 依賴漏洞 | EXIT=0 |
