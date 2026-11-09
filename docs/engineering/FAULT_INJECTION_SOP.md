# Fault Injection SOP — 韌性注入標準作業程序

> 級別：ENG-FAULT-SOP-1.0 | 日期：2026-11-06 | 定位：Chaos / Fault Injection(SOP派生，自方法論 §8/§14)
> 參考：fault injection for TDD of robust SoC firmware (unit-level + static bit-level analysis for acceleration), Host mock/HAL/QEMU/ISS 氧化鏈路。

## 1. 目標與非目標

- 目標：以 unit-level fault 為主、系統級為輔，驗證 engine/money/web/app 韌性，不追求嚴重度分數。
- 非目標：不以故障注入證明生產力提升；TDD 定位不變（回饋迴圈+規範工具）。

## 2. 分類

| 類別 | 手法 | 對象 | 典型注入 |
|------|------|------|----------|
| F1 邏輯變異 | cargo-mutants | core/src/money.rs, ledger/engine.rs | 算術/比較/布林變異 |
| F2 邊界注入 | boundary checks | money parse/format, engine validate | MAX_AMOUNT, MAX_TOTAL_DIGITS, 日期/匯率極值 |
| F3  I/O 注入 | test-only mock | sqlx queue, api path | DB busy, 參數綁定失敗, 空庫 |
| F4 韌性注入 | stub/chaos flag | platform/api.rs, flaps_bridge | CB open, 逾時, 冪等重放 |
| F5 跨層/簽證 | X-cross | BDD Scenario ⇄ 故障 | 使用者場景 + CB/timeout/permisison 錯向 |

## 3. Unit-level 注入 SOP（先行）

1. **靜態 bit-level 分析**：以 mutants list 作為靜態候選，已在 `.cargo/mutants.toml` include_files 設定。
2. **變異集收斂**：
   - money: 73 mutants, 56 caught, 13 missed = 81.2% (viable 69, 已排除 Display 文字 mutants)
   - engine: 33 mutants, 21 caught, 6 missed, 6 unviable = 77.8% raw (viable 27)
   - **排除後** (exclude 5: apply_pragmas + 4× query Ok(vec![])，屬 DB schema/integration 域外)：21/22 = **95.5%**，餘 1 為 `>→>=` 上限邊界真未殺，需補 `amount == MAX_AMOUNT_MINOR` 通過案。
3. **真失敗補 test**：missed 之後只補 убийстве test，不改 prod code（除非 prod 語意錯誤）↠ TDD 紅綠：先補 failing mutation test（RED），再確認原碼 GREEN。
4. **Unviable 處理**：unviable 視為 UNKNOWN，不計入分子分母，需留 unviable.txt 並說明原因（型別約束/SQL 方言）。

## 4. 邊界/值注入 SOP

| 注入點 | 目標值 | 預期 | 備註 |
|--------|--------|------|------|
| parse MAX_TOTAL_DIGITS(17)±1 | 17/18 位 | 17 OK, 18 Overflow | money.rs:89 |
| parse i128 MAX(38)±1 | 38/39 位 | 同上 | money.rs:333 |
| amount == MAX_AMOUNT_MINOR | 上限等值 | 通過 | engine.rs:262, 待補 |
| amount == MAX+1 | 上限+1 | 拒絕 | 已有 amount_over_cap_is_rejected |
| future date = today+0/1 | 邊界日 | today OK, tomorrow Reject | engine.rs:241 |
| empty tx id, 空 items | 空輸入 | 拒絕且訊息含因 | engine.rs:237,245 |

## 5. I/O 注入 SOP（Sqlite/Queue）

- **DB busy/locked**：`PRAGMA busy_timeout=5000` 時序注入，驗 CB/queue 行為（F4）。
- **綁定失敗**：傳入過長字串、非法 UTF-8、負數 limit/offset，期望參數化綁定而非拼接 SQL（T-07）。

## 6. 頻率與閘門

| 頻率 | 內容 | 閘門 |
|------|------|------|
| PR | cargo mutant subset (money/engine unit only) | score ≥80% 或 UNKNOWN 理由 |
| Nightly | 全量 mutant (除 unviable) + F2/F3 注入 | ≥80%, 0 missed 無理由 |
| Release gate | PBT + mutation score 出具報告，附 mutants.out/* | 審核人簽名 |

## 7. 證據留存

- `mutants.out/{caught.txt, missed.txt, unviable.txt, outcomes.json}`（各次分開目錄 mutants.out/YYYY-MM-DD/）
- 每條 missed 必須有：位置、預期攔截 test、處理（補測/排除/接受風險）
- 失敗不得以「已接受」代替說明；UNKNOWN ≠ PASS。

## 8. 與 TDD/BDD/PBT/MT 關係

- TDD：提供快速回饋迴圈（kill mutant），不主張生產力。
- PBT：money proptest 8 案 + engine 不變量 PBT 擴張（§8 待辦）。
- BDD：F5 場景（登入限流、CSRF、CB）與注入對應。
- FMEA/STPA：注入結果回填失效模式表（§8 待辦）。

## 9. 已知未覆蓋（UNKNOWN，明示）

- 真實 E.Sun 沙箱、winax 真實 COM 通道、config.ini A/B、8h soak：未注入，屬環境不可用。
- HIL/ISS 脫鉤介面：尚未建立故障注入硬體通道，僅以 host mock + statical bit-level 為孿生替身。

## 10. 追蹤表

| ID | 事項 | 狀態 | 參照 |
|----|------|------|------|
| FAULT-01 | engine 上限等值補測殺死 `>→>=` | **DONE** (2026-11-07 `amount_at_cap_posts_successfully`) | money/engine mutant |
| FAULT-02 | money 剩餘 missed（Display/||→&&/+→-/>→>=） | **DONE** (2026-11-07 全數殺死，missed.txt 0 行) | MUTATION_SCORE_REPORT v2 |
| FAULT-03 | PBT 擴至 ledger/parallel/pivot | TODO (P1) — proptest strategies 設計已定 | §8 PBT |
| FAULT-04 | FBT 模糊：CSV/money parser | **IN PROGRESS** — `fuzz/fuzz_targets/money_parser.rs` build PASS；run 受阻 ASan DLL（見 FUZZING.md §3） | docs/engineering/FUZZING.md |
| FAULT-05 | ATDD feature→test 自動稽核 | **IN PROGRESS** — 8 情境綁定稽核完成（2 MISSING + 1 BLOCKED 已列管） | ATDD_BDD_BIND_AUDIT.md |
| FAULT-06 | FMEA/STPA→BDD 綁定 | TODO (P2) | §8 FMEA |
| FAULT-07 | Chaos/DevSecOps/SRE 導入 | TODO (P2) | §8 25-32 |

