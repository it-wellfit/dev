# ADR-002：對接工具選型決策（OpenRPA／FlaUI／Rust 原生）

> 專案：FLAPS-RPA-ECOM-2026｜文件狀態：草案（Draft for Decision）｜決策期限：M2（2026-11-30）
> 對應：Gate 1 缺陷 B-05｜決策者：ARCH（主責）；會簽：DEV、SEC、PMT
> 關聯：v2.1 §3.5、§11；Wellfit docs\ENTERPRISE_AI_AUTOMATION_RUNTIME.md；三大專案藍圖 L3

## 1. 背景（Context）

v2.0 §3.5 直接選定 OpenRPA，但未與組織既有文件對齊：ENTERPRISE_AI_AUTOMATION_RUNTIME.md 列 FlaUI（隔離 Windows Worker）為桌面自動化選項；Wellfit 本體走 Rust 原生（無 RPA 平台）。Gate 1 要求補做比較與決策（缺陷 B-05）。

**不變前提**（v2.1 §11 已定）：對接模式優先序 A（API／服務層）> B（UI 自動化）> C（DB 唯讀對帧）。工具選型僅在**模式 B 被啟動**時有實質影響。

## 2. 選項與比較（模式 B 情境）

| 準則 | OpenRPA（v2.0 現選） | FlaUI（自建 Worker） | Rust 原生（windows-rs UIA） |
|---|---|---|---|
| 授權 | MPL-2.0 | MIT | MIT／Apache |
| 形態 | 完整 RPA 平台（設計器＋機器人＋OpenCore 協調） | .NET UIA 程式庫（薄層） | 程式庫（需自建大量基礎設施） |
| 與 Windmill 關係 | **協調職能重疊**（兩套排程／重試） | 無重疊（Windmill 單一協調點） | 無重疊 |
| winax／COM 親和 | 高（.NET COM interop） | 高 | 中（需包裝） |
| 部署 | 需安裝 OpenRPA 執行環境 | 單一 worker 程序 | 與 wellfit.exe 同型（原生單檔） |
| 維護／技能 | 現成設計器、學習曲線低 | 需自建元素庫與日誌 | 開發成本最高 |
| 組織一致性 | 引入新平台 | 與既有文件方向一致 | 與 Wellfit 技術棧一致 |

## 3. 建議決策（Recommendation，待核准）

1. **主線：模式 A（API）**。OQ-01~03 於第 2 週回覆後，若 API 可用 → 本 ADR 以「模式 B 不啟動」結案，OpenRPA 僅列備選。
2. **若必須模式 B**：第 2 週進行雙候選 PoC（各約 0.5 人日）——「OpenRPA 設計器」vs「FlaUI Worker」對同一電商畫面完成登入＋抓取一筆訂單；評分項目：元素辨識穩定性、腳本可簽章性（AllSigned）、與 Windmill 整合、錯誤恢復、維運成本。
3. **不建議 Rust 原生 UIA**：開發成本與時程風險最高，除非前兩者 PoC 皆失敗。
4. **不論選項**：OpenRPA／Worker 皆**不得承擔排程職能**（維持 Windmill 單一協調點），僅執行 UI 動作；重試與斷路器語意與 v2.1 §3.2 一致。

## 4. 影響與驗收

- 選型結果回填 v2.2 §3.5 與 DEL-D05（RPA 工作流規格）。
- 本 ADR 於 M2（2026-11-30）前完成簽核 → Gate 1 缺陷 B-05 關閉。
