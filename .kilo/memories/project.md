> 事實索引（workspace 標籤制；只連結 SSOT，不複寫條文）。
> 標籤：`[根]` 根 workspace（`Dev/`，edition 2021／Axum 0.7／SQLite）｜`[ekpi]` ekpi-platform（Clean Architecture／Rust 2024／PostgreSQL 18）｜`[KB]` `Knowledge/`（合規 SSOT）｜`[全]` 跨 workspace。
> 逾 30 日未覆核者標 `STALE`；`UNKNOWN` 不得視為 PASS。

| 事實（一句話） | 標籤 | 來源連結 | 覆核日期 |
|---|---|---|---|
| 主產品為 ekpi-platform/OEKAP；根 workspace 為 FlapsErp RPA 支援線 | [根] | `docs/adr/ADR-0001-tech-stack-baseline.md` §3-1 | 2026-11-09 |
| 根 workspace 技術棧基線：edition 2021／resolver 2／axum 0.7／SQLite（升級列為獨立 POC） | [根] | `docs/adr/ADR-0001-tech-stack-baseline.md` §3-2 | 2026-11-09 |
| 表現層 100% JavaScript-Free 為剛性不變量 | [根] | `docs/adr/ADR-0001-tech-stack-baseline.md` §3-3 | 2026-11-09 |
| 三大工作室計畫 SSOT 為 `1792865422970-*`（§0–§27，v3.1-r3） | [根] | `.kilo/plans/1792865422970-three-studio-governance-platform.md` | 2026-11-09 |
| 合規事實權威在 `Knowledge/`（SoA 93 控制、crosswalk、ADR-0001..0012） | [KB] | `Knowledge/README.md`（知識庫索引） | 2026-11-09 |
| 記憶與程式碼曾矛盾（project.md 宣稱 vs 實際棧）——新事實必標 workspace 來源 | [全] | `docs/adr/ADR-0001-tech-stack-baseline.md:16` | 2026-11-09 |
| `Dev/` 根 workspace 非 git repo（記憶庫無 commit 背書，以 session 記錄綁定） | [根] | `docs/adr/ADR-0001-tech-stack-baseline.md` §7；`AGENTS.md` §6 | 2026-11-09 |
