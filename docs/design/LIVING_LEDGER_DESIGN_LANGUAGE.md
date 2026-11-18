# Living Ledger 設計語言 — Decision OS 產品 UI

> 文件 ID: DOC-DL-001｜狀態：Active v1.0｜日期：2026-11-17｜owner：設計線
> 父系：`docs/design/UI_DESIGN_DECONSTRUCTION_v1.0.md`（五層解剖）＋`web/assets/ui-v2.css`（token 系統）
> 本文為 **Decision OS 產品面**（dashboard／console 44 頁／未來桌面殼）的設計語言；**不取代**平台 ui-v2，而是延伸其 token 語彙與零 JS 互動規格。

## 1. 設計方向探索（三案，每案具體規格）

### 方向 A — Ledger Classic（帳冊古典）
- **哲學**：知識如帳冊——紙感、橫線、鋼印、活字數字；印刷級排版紀律。
- **色彩**：暖紙 #f7f4ec、墨 #2a2620、朱紅印章點綴（僅狀態用）。
- **字體**：標題與數字 Noto Serif TC（鉛字感）、內文系統黑體、等寬用於 ID。
- **風險**：暗色模式需重繪（紙感在暗色失效）；資訊密度低於操作需求。
- **最適**：閱讀／稽核情境（M11 審計演練）。

### 方向 B — Instrument Dark（暗色儀器）
- **哲學**：決策如飛行——座艙式資訊密度、狀態燈、鍵盤優先。
- **色彩**：深板岩 #10161d 底、面板 #182029、玉山綠狀態燈、琥珀警告。
- **字體**：等寬主導（Cascadia）、小號高對比、tabular-nums 對齊。
- **風險**：「冷」感、新使用者學習曲線；與「精品」敘事衝突。
- **最適**：操作／維運情境（H2 人在迴圈上）。

### 方向 C — Living Ledger（活帳本）★採用
- **哲學**：帳本不是死物而是**活的有機體**——明暗雙生（紙與板岩）、帳冊秩序×儀器密度；玉山綠唯一品牌色；序號、時間戳、取代鏈是視覺主角。
- **採納理由**：直接具象化「可審計」定位（差異化交叉點）；覆蓋 A 的閱讀優雅＋B 的操作密度；與 ui-v2 雙主題 token 結構同構——**零衝突延伸**。
- **三案取捨**：A 襯線數字紀律→KPI／序號；B 等寬狀態燈→ID／時間戳／佇列；C 時間軸→決策鏈主視覺。

## 2. 色彩系統（對比度依 WCAG 公式實算）

| Token | Light | Dark | 用途 | 對比（vs 表面） |
|-------|-------|------|------|------------------|
| `--bg` | `#f4f6f2` | `#0e141b` | 頁面底 | — |
| `--surface` | `#ffffff` | `#182029` | 面板／卡片 | — |
| `--ink` | `#1b2733` | `#e8edf3` | 主文字 | 12.9:1 / 14.5:1 AAA |
| `--muted` | `#5c6b7a` | `#9db0c2` | 次文字 | 5.3:1 / 7.3:1 AA |
| `--accent` | `#0a7a4f` | `#35c78a` | 品牌／焦點 | 5.2:1 / 8.0:1 AA |
| `--ok` | `#177245` | `#5cd6a2` | published | 5.6:1 AA |
| `--warn` | `#9a5b00` | `#f0b45c` | superseded／佇列 | 5.4:1 AA |
| `--err` | `#b3261e` | `#ff8f86` | retired／錯誤 | 6.4:1 AA |
| `--border` | `#d8dee2` | `#2c3947` | 髮絲邊框 | — |
| `--chip` | `#eef2ee` | `#212c2a` | 徽章底 | — |

規則：狀態色永不單獨傳達（必配文字標籤，WCAG 1.4.1）；alpha 疊層降飽和（沿用 ui-v2 badge 手法）。
## 3. 字體與排版

| Token | 值 | 用途 |
|-------|-----|------|
| `--font-display` | `"Noto Serif TC","Songti TC",serif` | 大數字、KPI、wordmark（鉛字感） |
| `--font-body` | `"Microsoft JhengHei","PingFang TC",system-ui,sans-serif` | 內文 |
| `--font-mono` | `Consolas,"Cascadia Mono",monospace` | ID／時間戳／指令（tabular-nums） |

- 數字一律 `font-variant-numeric: tabular-nums`（帳冊對齊）。
- 標題層級：h1 1.35rem／h2 1.05rem／body .875rem／meta .78rem；行高 1.65；中文段落避免 justify（峽谷）。

## 4. 間距與栅格

- 間距階：4/8/12/16/24/32；卡片內距 16，區塊間 24；內容寬 max 1100px；KPI grid auto-fit minmax(140px,1fr)。
- 圓角 10px（面板）／999px（徽章）；髮絲邊框 1px；淺影 `0 1px 3px rgba(16,32,48,.08)`（暗色改亮度階）。

## 5. 簽名元件（Living Ledger 專屬）

| 元件 | 規格 | 狀態 |
|------|------|------|
| **KPI 卡** | 襯線大數字＋灰標籤；published 綠、佇列琥珀、警示紅 | Default／Empty（0 顯示 0 不隱藏） |
| **信心計量條** | 8px 軌道＋玉山綠填滿；≥0.8 綠（hi）、<0.45 琥珀（lo） | 數值恆伴（非顏色單傳） |
| **決策時間軸** | 左側 2px 玉山綠縱線（li::before）＋seq 徽章（等寬）＋動作＋主張＋時間 | 最後 12 條；hover 列高亮 |
| **狀態徽章** | published 綠／superseded·deprecated 琥珀／draft·pending 灰／retired 紅（alpha 底＋文字） | 全生命週期 |
| **取代鏈** | supersedes／superseded_by 雙向渲染「v1 → v2」（mono） | 空則不渲染 |
| **新鮮度戳** | 「最後同步：時間戳」＋資料來源（claims SSOT＋帳本） | 恆顯示於頁尾 |
| **警示橫幅** | 左 4px 紅線＋問題＋可能原因＋處理方式（三句式） | 僅 findings/queue>0 |
| **導航** | 膠囊連結；:hover 底色、:focus-visible 3px 玉山綠 outline | Zero-JS |

## 6. 互動與無障礙（零 JS 剛性）

- 導航：頁面連結＋:hover／:focus-visible（console 同頁錨點可用 :target）。
- 表格列 hover：accent 5% 混色；prefers-reduced-motion 全關過渡；深色 prefers-color-scheme 切換（token 重定義）；列印 @media print 隱藏 header/nav。
- 對比：§2 實算全 AA；鍵盤：全部連結可 focus；語義：header/nav/main/section/table/footer；lang=zh-Hant。
- **誠實標註**：靜態頁無 Loading 態（以生成時間戳替代）；Disabled 不適用（唯讀面）；Empty/Error 皆有專屬元件（.empty／.banner-err）。

## 7. 資訊架構（承接 INFORMATION ARCHITECTURE 方法）

- dashboard.html（單頁總覽）：KPI（生命週期）→ 主張（信心排序）→ 治理（聯合裁決＋佇列）→ 決策時間軸。
- console/（44 頁）：index（角色入口）→ roles/（5）→ workflows/（13）→ claims/（20＋索引）→ ledger-N（分頁）→ queue／vectors／maintenance。
- 導航原則沿用 UI_DESIGN_DECONSTRUCTION L1 判準：分類軸＝使用者的下一個動作（總覽→主張→治理→帳本），非技術堆疊。

## 8. 決策記錄（Design Decision Log 摘錄）

| 決策 | 選擇 | 原因 | 證據 |
|------|------|------|------|
| 設計方向 | Living Ledger（C） | 差異化具象化＋雙主題相容 | §1 三案對照 |
| 主品牌色 | 玉山綠 #0a7a4f | 既有品牌一致（ui-v2）＋對比 5.2:1 | §2 實算 |
| 字體策略 | 襯線數字＋黑體內文＋等寬 ID | 帳冊感×可讀性×對齊 | §3 |
| 互動 | 零 JS（:hover/:focus-visible/:target） | 憲章 §2.3 剛性 | §6 |
| CSS 單一來源 | docs/design/living-ledger.css（include_str! 嵌入） | 衍生物 SSOT 原則（禁雙份 CSS） | §9 |

## 9. 實作綁定

- `crates/kem/src/dashboard.rs` 與 `console.rs` 以 include_str! 嵌入同一份 CSS——單一事實來源，改動集中一處。
- 驗證：cargo test -p kem（script=0 護欄＋dashboard 測試）、四關、生成後逐頁掃描 script 計數。
- 變更記錄：v1.0（2026-11-17）首版；后续依 §8 追加。