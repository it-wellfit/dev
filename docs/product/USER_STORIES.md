# USER STORIES — 使用者故事（含驗收測試對映）

> 格式：As a \<role\>, I want \<capability\>, so that \<benefit\>。每條故事對映整合測試（單一事實來源）。

## Epic：登入與安全

| ID | Story | 測試 |
|----|-------|------|
| US-1 | As an admin, I want to log in with Argon2id-verified credentials so that only I can access the ledger. | login/logout 整合測試 |
| US-2 | As an admin, I want brute-force attempts locked after 5 failures so that probing is expensive. | login_lockout_survives_correct_password |
| US-3 | As an admin, I want to reset a user's password so that lost credentials can be recovered. | admin_password_reset_revokes_sessions... |
| US-4 | As an admin, I want deactivating a user to revoke their sessions immediately so that departed staff cannot act. | deactivating_a_user_revokes_their_live_session |

## Epic：登帳（Ledger）

| ID | Story | 測試 |
|----|-------|------|
| US-5 | As a finance user, I want a guided wizard that pre-fills and names each rule so that I can post a balanced entry in under 30 seconds. | wizard_step2_rejects_same_account... |
| US-6 | As a finance user, I want the engine to reject unbalanced entries so that my books always balance. | 引擎負向測試 8 條 |
| US-7 | As a finance user, I want amounts stored as integer minor units so that no floating-point rounding corrupts my books. | money 測試 6 條 |

## Epic：資料獲取

| ID | Story | 測試 |
|----|-------|------|
| US-8 | As a finance user, I want to import a CSV with a dry-run preview so that I can verify before committing. | IMP-001/002/003 |
| US-9 | As a finance user, I want to paste unstructured text and get a draft with evidence cards so that I can see why each field was extracted. | DRAFT-001＋parser 測試 |
| US-10 | As a finance user, I want optional AI suggestions that require my review so that AI never silently changes my books. | AIS-001..003 |

## Epic：報表與洞察

| ID | Story | 測試 |
|----|-------|------|
| US-11 | As an auditor, I want a monthly report with balanced totals so that I can verify the books. | RPT-001＋效能預算 |
| US-12 | As an auditor, I want a pivot across account × month so that I can analyze multidimensionally. | PIV-001..003 |
| US-13 | As an auditor, I want a parallel-coordinates projection so that I can compare accounts across months without 3D gimmicks. | PAR-001/002 |
| US-14 | As an auditor, I want CSV export in integer minor units so that downstream tools get exact values. | monthly_report_page_and_csv... |

## Epic：管理

| ID | Story | 測試 |
|----|-------|------|
| US-15 | As an admin, I want to create/deactivate users and reset passwords with session revocation so that I control access. | admin 測試 5 條＋P1-9 |
| US-16 | As an admin, I want parse coverage stats so that I can decide when AI integration adds value. | parse_coverage_computes_hit_ratio |
