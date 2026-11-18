# GitHub main 分支保護 — 操作指引（Repo Admin 執行）

> 依據：計畫 1793782615513 §2/§17.1 Phase 1。本操作需 GitHub repo 管理員權限（網頁 UI，無法由本地代理代行）。
> Repo：github.com/it-wellfit/dev.git

## 操作步驟（約 5 分鐘）

1. 開啟 repo → **Settings** → 左欄 **Branches**
2. **Branch protection rules** → **Add branch protection rule**（或 Add rule）
3. **Branch name pattern** 欄位填入：`main`
4. 勾選以下規則：
   - ✅ **Require a pull request before merging**
     - Required approvals: `1`
     - ✅ Dismiss stale pull request approvals when new commits are pushed
   - ✅ **Require status checks to pass before merging**
     - ✅ Require branches to be up to date before merging
     - 搜尋並加入 status checks（CI 首次運行後會出現）：`lint`、`unit`、`contract`、`build`、`security`、`integration`
   - ✅ **Require conversation resolution before merging**
   - ❌ 不要勾選：Restrict pushes that create matching branches（無需）
5. **Create** / **Save changes**

## 補充

- 若 CI status checks 尚未出現：先由任意 PR 觸發一次 `.github/workflows/ci.yml`（已就位），GitHub 記錄 job 名稱後即可選取。
- 緊急豁免：保護規則設定頁可暫時加入例外或由 Admin 使用「Merge without waiting」；依 §10.1 錯誤預算政策，豁免須記錄於 ADR。
- 驗證：設定完成後，直接 push 到 main 應被拒絕（`remote: error: GH006`）。