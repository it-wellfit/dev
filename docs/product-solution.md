# 產品解決方案：Open Enterprise Knowledge & Action Platform (OEKAP)

## 1. 產品概述

### 1.1 願景
建立一個**證據導向的企業知識與流程智慧平台**，讓企業員工能在 30 秒內找到正確答案，並自動執行相關動作，同時保持完整的稽核軌跡。

### 1.2 核心價值主張
- **H1 資訊取得成本**：減少搜尋時間 80%（從 15 分鐘 → 3 分鐘）
- **H2 工作流程斷裂**：整合分散系統，減少切換成本
- **H3 AI 信任不足**：每項答案附帶來源證據與信心分數
- **H4 整合維護成本**：模組化架構，降低維護負擔

### 1.3 目標使用者
| 角色 | 痛點 | 解決方案 |
|------|------|----------|
| 知識工作者 | 資訊散落多系統 | 統一搜尋 + 證據鏈 |
| 流程管理員 | 流程不透明 | 流程視覺化 + 異常偵測 |
| 合規/稽核 | 無法追溯決策 | 完整稽核軌跡 |
| IT 管理員 | 整合成本高 | 模組化 + 標準介面 |

---

## 2. 產品規格

### 2.1 功能模組

```
┌─────────────────────────────────────────────────┐
│                  OEKAP Platform                  │
├─────────────┬─────────────┬─────────────────────┤
│  知識搜尋    │  流程智慧    │  動作執行           │
│  (Search)   │  (Process)  │  (Action)           │
├─────────────┼─────────────┼─────────────────────┤
│ • 全文搜尋   │ • 流程挖掘   │ • 自動化腳本        │
│ • 語意搜尋   │ • 異常偵測   │ • 審批工作流        │
│ • 證據鏈     │ • 瓶頸分析   │ • 通知/提醒         │
│ • 來源追溯   │ • 合規檢查   │ • 報表生成          │
└─────────────┴─────────────┴─────────────────────┘
```

### 2.2 非功能需求

| 需求 | 目標 |
|------|------|
| 回應時間 | < 200ms (P95) |
| 可用性 | 99.9% |
| 安全性 | 零信任架構、RBAC、稽核日誌 |
| 可擴展性 | 水平擴展、無狀態服務 |
| 可維護性 | 模組化、測試覆蓋率 > 80% |

---

## 3. 技術架構

### 3.1 技術棧（避免 Node.js）

| 層級 | 技術 | 選擇理由 |
|------|------|----------|
| **前端** | Rust (Maud) + CSS-first | 零 JS、語意 HTML、高效能 |
| **後端核心** | Rust (Axum) | 記憶體安全、高效能、並發 |
| **搜尋引擎** | Tantivy (Rust) | 全文 + 語意搜尋、嵌入式 |
| **資料庫** | SQLite (rusqlite) | 零配置、嵌入式、ACID |
| **自動化** | PowerShell + Bun | 跨平台腳本、現代 runtime |
| **MCP 工具** | Bun (bunx) + Rust (cargo) | 避免 Node.js、高效能 |

### 3.2 模組化架構

```
oe-kap/
├── core/                    # 核心領域模型 (Rust)
│   ├── knowledge/           # 知識搜尋模組
│   ├── process/             # 流程智慧模組
│   └── action/              # 動作執行模組
├── platform/                # 平台服務 (Rust)
│   ├── api/                 # REST API (Axum)
│   ├── auth/                # 認證/授權
│   ├── audit/               # 稽核日誌
│   └── storage/             # 資料持久化
├── tools/                   # 工具鏈 (Bun + PowerShell)
│   ├── mcp/                 # MCP 伺服器配置
│   ├── skills/              # Agent Skills
│   └── scripts/             # 自動化腳本
├── web/                     # 前端 (Maud + CSS)
│   ├── components/          # UI 元件
│   ├── layouts/             # 版面配置
│   └── styles/              # CSS-first 樣式
└── tests/                   # 測試
    ├── unit/                # 單元測試
    ├── integration/         # 整合測試
    └── e2e/                 # 端對端測試
```

### 3.3 MCP 模組化整合

```
┌──────────────────────────────────────────────┐
│              Kilo Agent (Orchestrator)        │
├──────────┬──────────┬──────────┬─────────────┤
│ sequential│ playwright│ crates-io│ rust-anal.  │  Bun/Cargo
│ thinking │ (browser) │ (deps)   │ (code intel)│
├──────────┴──────────┴──────────┴─────────────┤
│  firecrawl │ jina │ context7 │ grep-app     │  Remote HTTP
│  (search)  │ (read)│ (docs)   │ (code)      │
├──────────────────────────────────────────────┤
│  websearch │ webfetch │ generate_image     │  Kilo Built-in
└──────────────────────────────────────────────┘
```

---

## 4. 開發流程

### 4.1 工作流程（基於 ekpi-platform 實踐）

```
需求 → 規格 → 設計 → 實作 → 測試 → 審查 → 部署 → 監控
  ↑___________________________________________↓
                    (回饋循環)
```

### 4.2 Git 工作流

- **分支策略**：`main` (穩定) → `feature/*` → `develop` → `release/*`
- **提交規範**：Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`)
- **PR 要求**：至少 1 人審查 + CI 通過 + 測試覆蓋率不下降

### 4.3 品質閘門（scripts/check.ps1）

```powershell
# 固定進入點
powershell -ExecutionPolicy Bypass -File scripts/check.ps1

# 檢查內容：
# 1. cargo fmt --check
# 2. cargo clippy -- -D warnings
# 3. cargo test
# 4. cargo run -- --selftest
# 5. Schema 驗證
# 6. 文件連結檢查
# 7. 高信號秘密掃描
```

### 4.4 CI/CD (GitHub Actions)

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: [ubuntu-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo run -- --selftest
```

---

## 5. 商業模型

### 5.1 定價策略

| 層級 | 價格 | 功能 |
|------|------|------|
| **Community** | 免費 | 基礎搜尋、100 文件、單使用者 |
| **Professional** | $29/月/使用者 | 完整搜尋、5000 文件、團隊協作、API |
| **Enterprise** | 客製化 | 無限制、SSO、SLA、專屬支援 |

### 5.2 收入來源

1. **SaaS 訂閱**（主要）
2. **企業授權**（本地部署）
3. **專業服務**（客製化、培訓）
4. **Marketplace**（第三方整合）

### 5.3 成本結構

| 項目 | 佔比 | 優化策略 |
|------|------|----------|
| 基礎設施 | 30% | Rust 高效能降低伺服器成本 |
| 開發 | 40% | 模組化 + 自動化測試 |
| 運營 | 20% | 自動化部署 + 監控 |
| 行銷 | 10% | 社群 + 內容行銷 |

---

## 6. 使用者痛點與解決方案

### 6.1 痛點矩陣

| 痛點 | 嚴重度 | 頻率 | 解決方案 | 驗證方式 |
|------|--------|------|----------|----------|
| 找不到資料 | 高 | 每日 | 證據導向搜尋 | 搜尋時間 < 3min |
| 流程不透明 | 中 | 每週 | 流程視覺化 | 流程完成率 +20% |
| AI 不可信 | 高 | 每日 | 來源證據 + 信心分數 | 使用者信任度 > 80% |
| 整合困難 | 中 | 每月 | 標準 API + MCP | 整合時間 < 1 週 |
| 效能差 | 高 | 每日 | Rust + 快取 | 回應 < 200ms |

### 6.2 使用者旅程

```
發現問題 → 搜尋知識 → 找到答案 → 驗證證據 → 執行動作 → 記錄結果
   ↓           ↓          ↓          ↓          ↓          ↓
  30s         10s        5s         5s         10s        5s
  └─────────── 總計 < 65 秒 ──────────────────────────────┘
```

---

## 7. UI/UX 設計原則

### 7.1 設計系統

- **色彩**：深色/淺色主題，WCAG AA 對比度
- **字體**：系統字體優先，避免 FOUT
- **間距**：8px 基礎單位
- **圓角**：4px (小) / 8px (中) / 16px (大)
- **陰影**：3 層深度系統

### 7.2 CSS-First 方法

```css
/* 零 JavaScript，純 CSS 互動 */
.search-results {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1rem;
}

.result-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: var(--space-md);
  transition: transform 0.2s ease;
}

.result-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

/* 無障礙 */
:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {
  * { animation: none !important; transition: none !important; }
}
```

### 7.3 響應式設計

| 斷點 | 寬度 | 佈局 |
|------|------|------|
| Mobile | < 640px | 單欄、底部導航 |
| Tablet | 640-1024px | 雙欄、側邊欄可收合 |
| Desktop | > 1024px | 三欄、固定側邊欄 |

---

## 8. 圖片與美術規範

### 8.1 圖片格式策略

| 用途 | 格式 | 品質 | 最大尺寸 |
|------|------|------|----------|
| Hero Banner | AVIF/WebP | 85% | 200KB |
| 內容圖片 | WebP | 80% | 100KB |
| 縮圖 | WebP | 75% | 30KB |
| 圖示 | SVG | 無損 | 5KB |
| 截圖 | PNG/WebP | 90% | 150KB |

### 8.2 圖片處理管線

```
原始圖片 → 格式轉換 → 壓縮 → 響應式版本 → CDN 部署
   ↓           ↓          ↓          ↓            ↓
  PNG/JPG    WebP/AVIF  Quality   srcset     CloudFront
```

### 8.3 視覺設計原則

- **一致性**：使用設計 Token（色彩、字體、間距）
- **層次**：視覺權重引導注意力
- **留白**：呼吸空間，避免擁擠
- **對比**：WCAG AA 最低標準
- **動效**：有意義的過渡，尊重 prefers-reduced-motion

---

## 9. 測試策略

### 9.1 測試金字塔

```
        ┌─────────┐
        │  E2E    │  10%  (Playwright)
        ├─────────┤
        │  整合   │  20%  (Rust integration tests)
        ├─────────┤
        │  單元   │  70%  (Rust unit tests)
        └─────────┘
```

### 9.2 測試命令

```powershell
# Rust 單元 + 整合測試
cargo test --locked

# 自測
cargo run --locked -- --selftest

# E2E (Playwright MCP)
# 透過 Kilo Agent + Playwright MCP 執行

# 效能測試
cargo bench

# 安全掃描
cargo audit
cargo deny check
```

### 9.3 覆蓋率目標

| 模組 | 目標 |
|------|------|
| 核心領域 | > 90% |
| API 層 | > 85% |
| 前端 | > 70% |
| 整體 | > 80% |

---

## 10. 部署策略

### 10.1 部署架構

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   CDN       │────▶│  Load       │────▶│  App        │
│  (靜態資源)  │     │  Balancer   │     │  Servers    │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                                │
                    ┌─────────────┐     ┌───────┴───────┐
                    │  Database   │◀────│  Rust Backend │
                    │  (SQLite)   │     │  (Axum)       │
                    └─────────────┘     └───────────────┘
```

### 10.2 部署方式

| 環境 | 方式 | 說明 |
|------|------|------|
| 開發 | `cargo run` | 本地開發 |
| 測試 | Docker + CI | 自動化測試 |
| 預覽 | 靜態二進位 | 單一檔案部署 |
| 生產 | systemd/Windows Service | 服務管理 |

### 10.3 部署檢查清單

- [ ] `cargo build --release` 成功
- [ ] `cargo test` 全部通過
- [ ] `cargo clippy` 無警告
- [ ] Schema 驗證通過
- [ ] 文件連結有效
- [ ] 安全掃描無高危
- [ ] 效能基準達標
- [ ] 備份/還原測試通過

---

## 11. 失敗修正策略

### 11.1 常見失敗與修正

| 失敗類型 | 診斷方法 | 修正策略 |
|----------|----------|----------|
| 編譯錯誤 | `cargo build` 輸出 | 修復型別/借用錯誤 |
| 測試失敗 | `cargo test` 輸出 | 修復邏輯/邊界條件 |
| Clippy 警告 | `cargo clippy` | 重構程式碼 |
| 效能退化 | `cargo bench` | 優化熱點路徑 |
| 安全漏洞 | `cargo audit` | 更新依賴/修補 |
| WDAC 封鎖 | 事件日誌 | 使用 Bun/Rust 替代 |

### 11.2 修正原則

1. **最小化變更**：只修復問題，不重構無關程式碼
2. **測試先行**：先寫失敗測試，再修復
3. **根因分析**：找到根本原因，而非症狀
4. **文件更新**：修復後更新相關文件
5. **回歸測試**：確保修復不引入新問題

---

## 12. 監控與維運

### 12.1 關鍵指標

| 指標 | 目標 | 監控方式 |
|------|------|----------|
| 回應時間 (P95) | < 200ms | Prometheus + Grafana |
| 錯誤率 | < 0.1% | Sentry |
| 可用性 | > 99.9% | Uptime monitoring |
| 記憶體使用 | < 512MB | 系統監控 |
| CPU 使用 | < 50% | 系統監控 |

### 12.2 日誌策略

- **結構化日誌**：JSON 格式，含 trace_id
- **日誌級別**：ERROR/WARN/INFO/DEBUG
- **保留策略**：30 天熱存、90 天冷存
- **敏感資料**：自動遮罩 PII

---

## 13. 實作路線圖

### Phase 1: 基礎 (Week 1-4)
- [ ] 專案初始化（Rust + Bun）
- [ ] 核心資料模型
- [ ] 基礎搜尋功能
- [ ] CI/CD 管線
- [ ] 基本 UI

### Phase 2: 增強 (Week 5-8)
- [ ] 語意搜尋
- [ ] 流程視覺化
- [ ] 證據鏈
- [ ] API 開放
- [ ] 效能優化

### Phase 3: 生產 (Week 9-12)
- [ ] 安全加固
- [ ] 監控告警
- [ ] 文件完善
- [ ] 使用者測試
- [ ] 正式部署

---

## 14. 成功指標

| 指標 | 目標 | 測量方式 |
|------|------|----------|
| 搜尋時間 | < 3 分鐘 | 使用者測試 |
| 使用者滿意度 | > 4.5/5 | NPS 調查 |
| 系統可用性 | > 99.9% | 監控 |
| 測試覆蓋率 | > 80% | CI 報告 |
| 部署頻率 | 每週 | CI/CD |

---

*本文件為產品解決方案藍圖，所有技術選擇均基於現有專案實踐與驗證。*
