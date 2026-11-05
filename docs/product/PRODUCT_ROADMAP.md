# PRODUCT ROADMAP — 產品路線圖（六階段）

> 現況：Phase 1（MVP Demo）**Demo Ready**（87 tests 全綠）。本路線圖對映六階段交付系統，每階段以 Go/No-Go 閘門把關。

## Phase 0 — Product Discovery & Blueprint ✅（本輪完成）

| 交付 | 狀態 |
|------|------|
| Product Vision／PRD／User Stories／User Journeys | ✅ docs/product/ |
| Design Token／IA（路由即 sitemap） | ✅ web/assets/design-system.css＋docs/design/ |
| Tech Stack／Technical Decisions | ✅ docs/engineering/＋docs/adr/ |
| 資料庫/API 設計 | ✅ docs/architecture/（migrations＋路由表為 SSOT） |

## Phase 1 — POC → MVP Demo ✅ Demo Ready

| 項目 | 狀態 |
|------|------|
| 可啟動、可登入、核心流程完整 | ✅ |
| 87 自動化測試全綠 | ✅ |
| Demo 文件＋seed/reset 腳本 | ✅（本輪） |
| 已知限制 | 見 docs/demo/DEMO_LIMITATIONS.md |

## Phase 2 — Demo → Production（**下一階段**，需授權）

| 工作 | 前置 | 狀態 |
|------|------|------|
| PRODUCTION_READINESS（Go/No-Go 矩陣） | 本輪 | ✅ 已建 |
| 正式部署環境（TLS＋網域） | **需授權＋採購/環境** | ⬜ |
| OpenTelemetry Monitoring | 部署環境 | ⬜ |
| Backup/Restore 演練 | 部署環境 | ⬜ |
| DAST／外部安全掃描 | 部署環境 | ⬜ |
| Login rate limiting 多實例遷移 | 多實例需求 | ⬜ |

**No-Go 條件**：未取得部署授權、無 TLS 環境。

## Phase 3 — Production → Scale-up（等 Phase 2 + 流量數據）

| 工作 | 前置 |
|------|------|
| 負載測試（k6） | 真實流量模型 |
| SLI/SLO | 監控數據 |
| 成本模型 | 基礎設施 |

## Phase 4 — Product Line（等商業模式決策）

| 工作 | 前置 |
|------|------|
| Entitlement 架構 | Plan Matrix 決策 |
| Subscription/Billing | 支付服務授權 |
| Multi-tenant | 資料隔離設計＋測試 |

## Phase 5 — Continuous Evolution（持續）

| 制度 | 狀態 |
|------|------|
| BACKLOG／CHANGELOG／PLAN_REGISTER | ✅ 運作中 |
| Product Analytics | ⬜ 需隱私評估 |
| 定期 Product Audit | ✅ 稽核收尾制度已建立 |
