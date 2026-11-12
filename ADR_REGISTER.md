# 架構決策登記（ADR Register）— Dev 根工作區

- 版本：v1.0｜生效日期：2026-11-12｜狀態：Active
- 格式與規則 SSOT：`docs/standards/SHARED_STANDARDS.md` §9

## 決策清單

| ADR | 標題 | 狀態 | 日期 | 決策者 |
|---|---|---|---|---|
| [0001](docs/adr/ADR-0001-tech-stack-baseline.md) | 技術棧基線（edition 2021／axum／SQLite） | Accepted | 2026-11-09 | 專案架構 |
| [0002](docs/adr/ADR-0002-api-layer.md) | API 分層（apihub 契約目錄） | Accepted | 2026-11-09 | 專案架構 |
| [0003](docs/adr/ADR-0003-double-entry-auth.md) | 雙一分錄與認證模型 | Accepted | 2026-11-09 | 專案架構 |
| [0011](docs/adr/ADR-0011-语言分工与多语言整合.md) | 語言分工與多語言整合 | Accepted | 2026-11-09 | 專案架構 |
| [0012](docs/adr/ADR-0012-bankpay-integration-boundary.md) | 銀行支付整合邊界 | Accepted | 2026-11-09 | 專案架構 |
| [0013](docs/adr/ADR-0013-cross-platform-strategy.md) | 跨平台策略（Tauri 殼＋零 JS Web） | Accepted | 2026-11-09 | 專案架構 |
| [0014](docs/adr/ADR-0014-unsafe-boundary-and-supply-chain-policy.md) | unsafe 邊界、供應鏈政策與 SBOM 閘 | Accepted | 2026-11-12 | 專案架構 |

## 使用規則

1. 每個持久工程決策必須有 ADR（§9.2）。
2. 狀態僅限 Proposed／Accepted／Deprecated／Superseded；修改決策須以新 ADR 取代並標 Superseded。
3. ADR 變更屬 High 風險驗證範圍；需附實作證據。

## 變更紀錄

| 日期 | 版本 | 說明 |
|---|---|---|
| 2026-11-12 | v1.0 | 建立登記表；補登 ADR-0001..0014 |