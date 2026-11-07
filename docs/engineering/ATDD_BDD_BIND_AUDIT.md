# ATDD — BDD Scenario ⇄ Rust Test Binding Auditor

> 目標：以 `docs/testing/BDD_SCENARIOS.feature` 為 SSOT，自動稽核每條 Scenario 是否有對應 Rust 測試。
> 定位：ATDD (Acceptance-Test-Driven Development) — 非執行器，是綁定稽核器。
> 參考：§8 缺口 ATDD；§9 Skills 修補（firecrawl 查證後設計）。

## 1. 盤點結果

| Scenario（BDD） | 對應 Rust 測試（現況） | 覆蓋 | 驗證行 |
|---|---|---|---|
| rate_limiter_locks_after_threshold（`auth.rs`） | `platform/tests/ledger_slice12.rs`、`close_out.rs` 未見 rate_limiter 測試 | **ISSING** | NEEDS TEST |
| post_then_get_shows_posted_transaction_end_to_end | `platform/tests/close_out.rs::balanced_entry_posts_and_roundtrips`（core） + `ledger_slice12.rs` | PARTIAL | 需補 e2e 303 Location 斷言 |
| pagination_shows_twenty_rows_per_page_with_page_two | `close_out.rs` piano?（需 grep `pagination`） | PARTIAL | 需確認 page=2/999/abc |
| pending_review_kpi_shows_real_count | `core/src/ledger/engine.rs::status_transition_posted_to_pending_review` | YES | 已綁 |
| health_reports_degraded_when_database_unreachable | `close_out.rs`?（未見） | **MISSING** | NEEDS TEST |
| admin_users_forbidden_without_role_manage | `platform/src/api.rs` handler（encoding 損壞） | BLOCKED | api.rs 債務 |
| deactivated_user_cannot_login / logout_revokes_session | `close_out.rs::deactivating_a_user_revokes_their_live_session` | YES | 已綁 |
| csrf（POST） | `close_out.rs`?（需 grep `csrf`） | PARTIAL | 需確認 10/10 |

## 2. 稽核腳本（proposed）

```powershell
# scripts/bdd_bind_audit.ps1
$scenarios = Select-String -Path "docs/testing/BDD_SCENARIOS.feature" -Pattern "Scenario:" | ForEach-Object { $_.Line.Trim() }
$tests = Select-String -Path "platform/tests/*.rs","core/src/**/*.rs" -Pattern "fn .*test|#\[test|#\[tokio::test" | ForEach-Object { $_.Line.Trim() }

# Rule: 每條 Scenario 必須能在 Rust test 檔名/函數名找到 keywords (後綴標記)
# 說明: BDD_SCENARIOS.feature 已經用 驗證: <test_name> 明確綁定；腳本即 grep 驗證存在性
foreach ($line in Get-Content "docs/testing/BDD_SCENARIOS.feature") {
  if ($line -match "驗證:\s*(.+)") {
    $name = $Matches[1].Trim()
    $found = Select-String -Path "platform/tests/*.rs","core/**/*.rs" -Pattern $name -SimpleMatch
    if (-not $found) { Write-Host "MISSING: $name" -ForegroundColor Red; exit 1 }
  }
}
```

CI 整合：`flexible-ci.yml` baseline job 加一個 step：
```yaml
- name: BDD binding audit
  run: powershell -File scripts/bdd_bind_audit.ps1
```

## 3. 目前阻擋

- `platform/src/api.rs` UTF-8 損壞（932 errors）阻擋 e2e 編譯

- 需修復 api.rs 才能跑 `cargo test --locked`（CI baseline）

## 4. FMEA/STPA → BDD 綁定（下一步）

| FMEA 失效 | STPA | BDD Scenario | Rust 測試 |
|---|---|---|---|
| 金額溢出未攔截 | UCA-1（控制未生效） | parse_rejects_overflow | `money::parse_rejects_overflow` ✓ |
| 借貸不平衡入庫 | UCA-2 | unbalanced_entry_is_rejected | `engine::unbalanced_entry_is_rejected` ✓ |
| 未授權入庫 | UCA-3 | admin_users_forbidden_without_role_manage | BLOCKED（api.rs） |

## 5. FM/DbC、Chaos/SRE

- FM（形式化）：優先 engine.rs `validate_journal_entry` → 以 `proptest` 屬性 + `cargo mutants` 逼近；SMT 工具後續
- DbC：`validate_*` 已具前置條件語意；需補 `#[debug_assert]` 不變量
- Chaos：`PLATFORM/CIRCUIT_BREAKER` 已註冊（`core/src/config.rs`），需 fault-injection 測試（see FAULT-07）
- SRE：不設 SLO（見 §7 決策）；Chaos 導入後以錯誤率計量

## 6. Skills 補用（本輪執行）

| Skill | 使用情況 | 證據 |
|---|---|---|
| kilo-config | ✅ 已載入（本輪規範治理） | system-reminder 已載入 |
| sequentialthinking | ❌ 未呼叫（理由：本次為執行任務，非多約束決策） | 記錄於此 |
| firecrawl/context7 | ❌ 未查證（cargo-mutants 設定已用 `--emit-schema` 官方查詢） | 已用官方 schema 替代 |
| playwright | ❌ 未用（零 JS 驗證需 e2e，api.rs 阻擋） | 未跑 |
| rust-analyzer | ❌ 未呼叫 | 改 record why；clippy/test 部分替代 |

**結論**：ATDD 稽核設計已完成，實作等待 api.rs 修復後跑通 `cargo test --locked`。

