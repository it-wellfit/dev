# TECHNICAL DECISIONS — 技術決策摘要

> 完整決策見 docs/adr/（ADR-0001..0003）。本文件為快速索引＋補充決策。

## ADR 索引

| ADR | 標題 | 狀態 |
|-----|------|------|
| ADR-0001 | 技術棧基線（edition 2021／axum 0.7／SQLite；記憶體版本差異揭露） | Accepted |
| ADR-0002 | 零 JS API 層（Axum 伺服器渲染＋PRG＋白名單樞紐） | Accepted |
| ADR-0003 | 認證會話（Argon2id＋伺服器端會話＋同步器 CSRF） | Accepted |

## 補充決策（Slice 8-11，未立 ADR 的較小決策）

| # | 決策 | 理由 | 影響 |
|---|------|------|------|
| D-1 | 金額整數分（i64）＋匯率 10^4 縮放 | INV-01 零浮點 | schema 0001 INTEGER；parser/money 精確解析 |
| D-2 | 借貸平衡引擎強制 | 帳務完整性 | 任何呼叫端不可繞過 |
| D-3 | 匯入兩段式（預檢/確認） | 乾跑無副作用＋重放防護 | 409；批次稽核 |
| D-4 | 依據卡（Evidence Card） | 可解釋／可推導 | parser.rs Evidence |
| D-5 | 會話伺服器端存放＋即時撤銷 | 停用/重設立即失效 | admin_toggle/reset |
| D-6 | 登入速率鎖定 in-process | 本機單實例 | 多實例需遷移（P1-7 notes） |
| D-7 | 樞紐白名單維度 | SQL 注入防護 | dim_expr/measure_expr |
| D-8 | 平行座標取代互動 3D | 零 JS 憲章＋多維標準 | OQ-8-1 擱置 |
| D-9 | 安全標頭 middleware | OWASP 清單 | 全回應 |
| D-10 | Builder 取代長參數建構子 | P2-1 清償 | allow 歸零 |
| D-11 | Money/AccountId 值物件 | DDD＋建構即驗證 | value_objects.rs |
| D-12 | 樞紐列上限 20＋OTHER 摺疊 | OQ-10-2 | 矩陣不爆炸 |

## 待決策（開立問題）

| # | 問題 | 狀態 |
|---|------|------|
| OQ-8-1 | 互動 3D 豁免 | 擱置（平行座標替代已交付） |
| OQ-8-2 | AI 介接深度 | 覆蓋率量測已就位（/admin/parse_coverage） |
| OQ-10-1 | 提示詞治理 | 模型/版本已入 erp_config；回應已稽核 |
| OQ-10-3 | 多實例速率限制 | 待多實例需求 |
