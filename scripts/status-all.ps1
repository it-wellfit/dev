# status-all.ps1 — 顯示所有 repo 與分支狀態
# 用法: .\scripts\status-all.ps1
param()
$root = Split-Path -Parent $PSScriptRoot
$repos = @{
    "." = @("main", "develop", "test", "release")
    "ekpi-platform" = @("main", "audit/export-hardening")
    "Wellfit" = @("main")
    "SyncMesh" = @("master")
    "agentre-bench" = @("main")
}
foreach ($repoPath in $repos.Keys) {
    $fullPath = Join-Path $root $repoPath
    if (-not (Test-Path -LiteralPath "$fullPath\.git")) { continue }
    Write-Host "`n=== $repoPath ===" -ForegroundColor Cyan
    Push-Location -LiteralPath $fullPath
    try {
        $currentBranch = git branch --show-current 2>&1
        Write-Host "  Current branch: $currentBranch"
        $status = git status --porcelain 2>&1
        if ($status) {
            Write-Host "  [DIRTY] Uncommitted changes:" -ForegroundColor Yellow
            $status | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
        } else {
            Write-Host "  [CLEAN] Working tree clean" -ForegroundColor Green
        }
        foreach ($branch in $repos[$repoPath]) {
            $localExists = git branch --list $branch 2>&1
            if (-not $localExists) {
                Write-Host "  $branch : not found locally" -ForegroundColor Red
                continue
            }
            $unpushed = git log --oneline "origin/$branch..$branch" 2>&1
            $unpulled = git log --oneline "$branch..origin/$branch" 2>&1
            $syncStatus = "synced"
            if ($unpushed) { $syncStatus = "ahead (unpushed)" }
            if ($unpulled) { $syncStatus = "behind (unpulled)" }
            if ($unpushed -and $unpulled) { $syncStatus = "diverged" }
            $color = if ($syncStatus -eq "synced") { "Green" } else { "Yellow" }
            Write-Host "  $branch : $syncStatus" -ForegroundColor $color
        }
    } finally {
        Pop-Location
    }
}
Write-Host "`n=== Status-all complete ===" -ForegroundColor Cyan
