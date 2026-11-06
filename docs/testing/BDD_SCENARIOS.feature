# BDD_SCENARIOS.feature — 核心旅程 Given/When/Then 場景
#
# 語言：Gherkin 關鍵字（Feature/Scenario/Given/When/Then），以既有 Rust 整合
# 測試為可執行綁定（step 綁定＝axum oneshot 整合測試；見對照表）。
# 選擇不引入 cucumber crate 的理由（精選原則）：既有 oneshot 測試已是真實
# router 端到端，cucumber 的步驟解耦層在本規模屬純開銷；本檔為 SSOT 場景註冊表。
# 規則：每個 Scenario 必須綁定一個存在且通過的測試名稱；UNKNOWN 不等於 PASS。

Feature: Flaps ERP 工作台核心旅程（零 JavaScript，伺服器端渲染）
  作為倉儲/帳務操作員與管理員，
  我需要在瀏覽器完成登入、登帳、查詢、覆核與管理，
  以確保帳務正確、安全且可稽核。

  Background:
    Given 已套用全部 migrations 的 SQLite 資料庫
    And 已播種 admin 與 store1 帳號

  Scenario: 操作員以錯誤密碼登入被速率限制
    Given admin 已連續失敗登入 5 次
    When admin 以正確密碼登入
    Then 回應為鎖定訊息且含剩餘秒數
    綁定測試: rate_limiter_locks_after_threshold（auth.rs，P1-7；正確密碼被鎖另有整合案例）

  Scenario: 操作員登入後登帳並立即見於清單
    Given admin 已登入並取得 CSRF 權杖
    When 提交借貸平衡的雙欄交易（1,234.50 TWD）
    Then 回應 303 且 Location /#transactions
    And 重新 GET 首頁渲染 1,234.50 與摘要
    And 資料庫中該交易的借貸分錄平衡
    綁定測試: post_then_get_shows_posted_transaction_end_to_end

  Scenario: 超過 20 筆交易後清單分頁
    Given 已存在 25 筆交易
    When 操作員開啟首頁與 ?page=2
    Then 首頁顯示 20 列與「第 1 / 2 頁・共 25 筆」
    And page=2 顯示其餘 5 列
    And ?page=999 導回末頁；?page=abc 回第 1 頁
    綁定測試: pagination_shows_twenty_rows_per_page_with_page_two
    綁定測試: pagination_clamps_out_of_range_page_to_last_page
    綁定測試: pagination_falls_back_to_page_one_on_illegal_page

  Scenario: 儀表板待覆核 KPI 顯示真實計數
    Given 已存在 2 筆 status=PENDING_REVIEW 的交易
    When 操作員開啟首頁
    Then KPI 顯示 2 且不再顯示佔位破折號
    綁定測試: pending_review_kpi_shows_real_count_instead_of_placeholder

  Scenario: 資料庫不可達時健康探測降級
    Given 連線池已關閉
    When 探測 GET /api/health
    Then 回應 503 與 status=degraded
    綁定測試: health_reports_degraded_when_database_unreachable

  Scenario: 無 ROLE_MANAGE 者不得進入使用者管理
    Given store1 僅具 STORE 角色
    When store1 開啟 /admin/users
    Then 回應 403
    綁定測試: admin_users_forbidden_without_role_manage

  Scenario: 停用帳號的活躍會話立即撤銷
    Given store1 已登入
    When admin 停用 store1
    Then store1 的舊會話在下次請求被導向 /login
    綁定測試: deactivated_user_cannot_login；logout_revokes_the_session

  Scenario: 全部 POST 受 CSRF 保護
    Given 任一未攜帶正確權杖的 POST
    Then 回應 403
    綁定測試: csrf 護欄 10/10（THREAT_MODEL T-08）