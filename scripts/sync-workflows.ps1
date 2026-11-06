# sync-workflows.ps1 — 同步 workflow 檔案到 GitHub（需 workflow scope）
# 用法: .\scripts\sync-workflows.ps1 [-Repo "it-wellfit/dev"] [-Branch "main"]
# 前置: gh auth refresh -h github.com -s workflow
param(
    [string]$Repo = "it-wellfit/dev",
    [string]$Branch = "main"
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$workflowDir = Join-Path $root ".github\workflows"
if (-not (Test-Path -LiteralPath $workflowDir)) {
    Write-Host "No .github/workflows directory found" -ForegroundColor Red
    exit 1
}
$files = Get-ChildItem -LiteralPath $workflowDir -Filter "*.yml"
foreach ($file in $files) {
    $content = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($file.FullName))
    $body = @{ message = "ci: sync $($file.Name)"; content = $content; branch = $Branch } | ConvertTo-Json -Depth 3
    $body | Set-Content -LiteralPath "$env:TEMP\sync_payload.json" -Encoding UTF8
    Write-Host "Syncing $($file.Name) to $Repo..." -ForegroundColor Cyan
    $result = gh api "repos/$Repo/contents/.github/workflows/$($file.Name)" --method PUT --input "$env:TEMP\sync_payload.json" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] $($file.Name) synced" -ForegroundColor Green
    } else {
        Write-Host "  [FAIL] $($file.Name): $result" -ForegroundColor Red
    }
}
Write-Host "`nSync complete. If failed, ensure 'gh auth refresh -h github.com -s workflow' was run." -ForegroundColor Yellow
