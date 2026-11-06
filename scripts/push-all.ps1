# push-all.ps1 — 一鍵推送所有 repo 與分支
# 用法: .\scripts\push-all.ps1 [-DryRun] [-Force]
param(
    [switch]$DryRun,
    [switch]$Force
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$repos = @{
    "." = @("main", "develop", "test", "release")
    "ekpi-platform" = @("main", "audit/export-hardening")
    "Wellfit" = @("main")
    "SyncMesh" = @("master")
    "agentre-bench" = @("main")
}
$pushArgs = @()
if ($Force) { $pushArgs += "--force-with-lease" }
foreach ($repoPath in $repos.Keys) {
    $fullPath = Join-Path $root $repoPath
    if (-not (Test-Path -LiteralPath "$fullPath\.git")) {
        Write-Host "[SKIP] $repoPath — not a git repo" -ForegroundColor Yellow
        continue
    }
    Write-Host "`n=== $repoPath ===" -ForegroundColor Cyan
    Push-Location -LiteralPath $fullPath
    try {
        $status = git status --porcelain 2>&1
        if ($status) {
            Write-Host "  [INFO] Uncommitted changes detected" -ForegroundColor Yellow
            if (-not $DryRun) {
                git add -A 2>&1 | Out-Null
                git reset HEAD -- .github/workflows/ 2>&1 | Out-Null
                git reset HEAD -- worktrees/ 2>&1 | Out-Null
                $hasStaged = git diff --cached --stat 2>&1
                if ($hasStaged -and $hasStaged -notmatch "fatal") {
                    $msg = "chore: auto-commit pending changes $(Get-Date -Format 'yyyy-MM-dd HH:mm')"
                    git commit -m $msg 2>&1 | Out-Null
                    Write-Host "  [COMMIT] $msg" -ForegroundColor Green
                }
            }
        } else {
            Write-Host "  [OK] Working tree clean" -ForegroundColor Green
        }
        foreach ($branch in $repos[$repoPath]) {
            $localBranch = git branch --list $branch 2>&1
            if (-not $localBranch) {
                Write-Host "  [SKIP] Branch '$branch' does not exist locally" -ForegroundColor Yellow
                continue
            }
            git checkout $branch 2>&1 | Out-Null
            if (-not $DryRun) {
                git pull --rebase origin $branch 2>&1 | Out-Null
            }
            if ($DryRun) {
                Write-Host "  [DRY-RUN] Would push $branch" -ForegroundColor Cyan
            } else {
                $result = git push @pushArgs origin $branch 2>&1
                if ($LASTEXITCODE -eq 0) {
                    Write-Host "  [PUSH] $branch -> origin/$branch OK" -ForegroundColor Green
                } else {
                    Write-Host "  [FAIL] $branch push failed: $result" -ForegroundColor Red
                }
            }
        }
    } finally {
        Pop-Location
    }
}
Write-Host "`n=== Push-all complete ===" -ForegroundColor Cyan
