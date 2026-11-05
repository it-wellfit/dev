# Git Worktree 多環境協作工作流 — Dev 根工作區

> 版本：v1.0｜日期：2026-11-05｜狀態：Active  
> 適用：it-wellfit/dev (GitHub)，涵蓋本地開發、本地測試、本地分支、本地正式、遠端 worktree、遠端測試、遠端分支、遠端正式八環境。

---

## 1. 環境對應表

| 環境名稱 | 本地路徑 | Git 分支 | 遠端分支 | 用途 | 保護 |
|----------|---------|---------|---------|------|------|
| 本地開發 | `worktrees/local-develop/` | `develop` | `origin/develop` | 功能整合與日常開發 | 無（開放推送） |
| 本地測試 | `worktrees/local-test/` | `test` | `origin/test` | QA 驗證與冒煙測試 | 無 |
| 本地分支 | `worktrees/local-<feature>/` | `feature/<issue>-<desc>` | `origin/feature/...` | 個人功能分支開發 | 無 |
| 本地正式 | 根目錄 `Dev/` | `main` | `origin/main` | 生產發行候選 | ✅ 受保護，禁止直接推送 |
| 遠端 worktree | GitHub Codespaces / `gh` worktree | 對應本地分支 | `origin/*` | 遠端開發機協作 | 視分支 |
| 遠端測試 | GitHub Actions `deploy-test` | `test` | `origin/test` | 自動化 QA 部署 | 無 |
| 遠端分支 | GitHub PR 分支 | `feature/...` | `origin/feature/...` | PR 評審與合併前驗證 | ✅ 需 PR 核准 |
| 遠端正式 | GitHub Actions `deploy-main` | `main` | `origin/main` | 生產環境部署 | ✅ 受保護，需 PR + 核准 |

---

## 2. 分支策略 (Git Flow 變體)

```
main        ──●────●────●  (生產，受保護，僅透過 release PR 合併)
             \    /    /
release      ──●────●  (預發布，RC 標記)
             \
develop      ──●────●────●  (整合，功能匯入)
             /    |    \
feature/*   ──●   │   ●  (功能開發，並行)
fix/*        ───●   │  (熱修復)
hotfix/*     ────●───●  (生產熱修復，直接合併 main + develop)
```

### 分支命名規範
- `feature/<issue-id>-<short-desc>` — 新功能
- `fix/<issue-id>-<short-desc>` — Bug 修復
- `release/v<major>.<minor>.<patch>` — 發布候選
- `hotfix/<issue-id>-<short-desc>` — 生產熱修復

### 提交訊息格式
```
<type>: <subject>

本文：問題描述 / 變更摘要 / 測試證據 / 回滾方式
Fixes #<issue-id>
```

類型：`feat`｜`fix`｜`docs`｜`test`｜`refactor`｜`perf`｜`security`｜`chore`

---

## 3. Worktree 建立與操作命令

### 3.1 建立新工作環境 (本地)

```powershell
# 於 Dev 根目錄執行
git worktree add worktrees/local-develop develop
git worktree add worktrees/local-test test
git worktree add worktrees/local-release release
git worktree add worktrees/local-feature-123 feature/123-example
```

### 3.2 切換與同步

```powershell
# 根目錄 (main)
git checkout main; git pull origin main

# 各 worktree 獨立操作
cd worktrees/local-develop; git pull origin develop
cd worktrees/local-test; git pull origin test
```

### 3.3 移除 worktree

```powershell
git worktree remove worktrees/local-feature-123 --force
git branch -D feature/123-example
git push origin --delete feature/123-example
```

### 3.4 查看所有 worktree

```powershell
git worktree list
```

---

## 4. AI-CI/CD 流程 (.github/workflows/ci.yml)

### 4.1 pipeline 結構

| Job | 觸發分支 | 內容 | 環境 |
|-----|---------|------|------|
| `baseline` | 所有 | `fmt` / `clippy -D warnings` / `test --locked` / `cargo audit` | ubuntu + windows |
| `deploy-develop` | `develop` | 建置 + 冒煙測試 | 開發環境 |
| `deploy-test` | `test` | 建置 + 完整測試 + 部署預備 | 測試環境 (QA) |
| `deploy-release` | `release` | 建置 + RC 驗證 + 打 tag | 預發布環境 |
| `deploy-main` | `main` | 建置 + 生產驗證 + 部署 | 生產環境 |

### 4.2 基線檢查 (每個 PR / push 必須通過)

1. `cargo fmt -- --check` — 程式碼格式
2. `cargo clippy --all-targets -- -D warnings` — 零警告
3. `cargo test --locked` — 所有測試通過 (0 failed / 0 ignored)
4. `cargo audit` — 依賴安全

### 4.3 部署門檻

- **develop**：push 即觸發 `deploy-develop`，自動建置與冒煙測試。
- **test**：push 即觸發 `deploy-test`，完整測試後部署至測試環境。
- **release**：push 觸發 `deploy-release`，RC 驗證並自動打 `rc-<timestamp>` tag。
- **main**：僅透過 PR 合併，需經保護分支核准，觸發 `deploy-main` 部署至生產。

### 4.4 遠端 GitHub Actions 設定說明

因 GitHub OAuth token 目前未授予 `workflow` scope，`.github/workflows/ci.yml` **不納入 git 版本控制**（避免推送被拒），但檔案保留於本地所有 worktree。

**手動在 GitHub 啟用 CI/CD：**
1. 登入 `https://github.com/it-wellfit/dev`
2. 進入 `Settings` → `Actions` → `General` → 確認 Workflow 權限開啟
3. 在 `.github/workflows/` 建立 `ci.yml`，內容參考本地 `worktrees/*/.github/workflows/ci.yml`
4. 為 `main` 分支啟用分支保護規則 (Branch Protection)：
   - Require a pull request before merging
   - Require status checks: `baseline`
   - Restrict pushes to main

---

## 5. 多 Agent 與多人協同機制

### 5.1 CodeOWNERS

建立 `.github/CODEOWNERS`：

```
*                                           @it-wellfit/maintainers
.github/workflows/                          @it-wellfit/devops
docs/                                       @it-wellfit/architect
core/                                       @it-wellfit/backend
platform/                                   @it-wellfit/backend
web/                                        @it-wellfit/frontend
```

### 5.2 PR 評審流程

1. 開發者在 `feature/<issue>-<desc>` 分支開發，完成後 push 到遠端。
2. 開啟 Pull Request 從 `feature/...` → `develop`。
3. CI 自動執行 `baseline` (fmt / clippy / test / audit)。
4. 至少一位 CodeOWNERS 成員必須 Approve。
5. 合併後自動刪除 feature 分支。

### 5.3 發布流程

1. 從 `develop` 建立 `release/v<version>` 分支。
2. 開啟 PR 從 `release/...` → `main`。
3. CI 執行 `deploy-release` (RC 驗證 + 打 tag)。
4. 經維護人員核准後合併到 `main`，觸發 `deploy-main` 部署生產。
5. 釋出 GitHub Release (annotated tag `v<version>`)。

### 5.4 冲突解決與同步

- 每日於本地開發 worktree 執行 `git pull origin develop` 同步最新。
- 如遇衝突，先於 feature 分支上 `git merge develop` 解決，再 push 並重新發起 PR。
- 釋放後，所有 worktree 執行 `git pull origin main` 與 `git pull origin develop`。

---

## 6. 驗證清單 (Definition of Done)

每個環境變更必須滿足：

- [ ] `git worktree list` 顯示對應 worktree 存在且分支正確
- [ ] `cargo fmt -- --check` EXIT=0
- [ ] `cargo clippy --all-targets -- -D warnings` EXIT=0
- [ ] `cargo test --locked` 0 failed / 0 ignored
- [ ] `cargo audit` EXIT=0
- [ ] PR 已經至少一位 CodeOWNERS Approve (跨分支合併)
- [ ] 遠端分支 `origin/<branch>` 與本地同步
- [ ] 文件與代碼同一提交更新 (SHARED_STANDARDS §4.3)

---

## 7. 故障排除

| 問題 | 原因 | 解決方式 |
|------|------|---------|
| `git push` 被拒 (workflow scope) | OAuth token 無 `workflow` scope | ci.yml 不追蹤；於 GitHub Web 手動建立 workflow |
| worktree 分支已被佔用 | 分支已於另一 worktree checkout | 改用根目錄或新 worktree 路徑，或先 `git worktree remove` |
| `cargo test` 超時 | 編譯冷啟動 | 增加 bash timeout，或重複執行利用 cargo cache |
| 分支推送衝突 | 遠端已有新提交 | `git pull --rebase` 後重新推送 |

---

## 8. 參考文件

- `docs/standards/SHARED_STANDARDS.md` — 版本控制 §5、品質門檻 §6
- `docs/testing/TEST_STRATEGY.md` — 測試矩陣與規則
- `WORKSPACE_CHARTER.md` — 工作區憲章
- `AGENTS.md` — 代理人執行入口

---

*本文件與代碼同一提交更新 (SHARED_STANDARDS §4.3)。*
