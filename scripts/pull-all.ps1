# pull-all.ps1 — 一鍵拉取所有 repo 與分支
# 用法: .\scripts\pull-all.ps1
param()
$ErrorActionPreference = "Stop"
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
        foreach ($branch in $repos[$repoPath]) {
            $localBranch = git branch --list $branch 2>&1
            if (-not $localBranch) { continue }
            git checkout $branch 2>&1 | Out-Null
            $result = git pull --rebase origin $branch 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  [PULL] $branch <- origin/$branch OK" -ForegroundColor Green
            } else {
                Write-Host "  [FAIL] $branch pull failed: $result" -ForegroundColor Red
            }
        }
    } finally {
        Pop-Location
    }
}
Write-Host "`n=== Pull-all complete ===" -ForegroundColor Cyan
