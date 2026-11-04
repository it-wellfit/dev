# session-cleanup.ps1 - purge ignored heavy artifacts older than retention days
# Usage: powershell -ExecutionPolicy Bypass -File scripts\session-cleanup.ps1 [-Days 30] [-DryRun]

param(
    [int]$Days = 30,
    [switch]$DryRun
)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
$cutoff = (Get-Date).AddDays(-$Days)

$patterns = @(
    "runs",
    "tests\*.log"
)
$targets = @()
# Collect under .kilo/sessions and */artifacts
$baseDirs = @()
$baseDirs += (Join-Path $root ".kilo\sessions")
Get-ChildItem -LiteralPath $root -Directory | ForEach-Object {
    $art = Join-Path $_.FullName "artifacts"
    if (Test-Path $art) { $baseDirs += $art }
}

foreach ($bd in $baseDirs) {
    if (-not (Test-Path $bd)) { continue }
    Get-ChildItem -LiteralPath $bd -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
        $f = $_
        # only purge files under runs/ or *.log under tests/ or io/input|output
        $rel = $f.FullName.Substring($bd.Length)
        $isPurge = $false
        if ($rel -match '\\runs\\') { $isPurge = $true }
        if ($rel -match '\\tests\\[^\\]*\.log$') { $isPurge = $true }
        if ($rel -match '\\io\\(input|output)\\') { $isPurge = $true }
        if ($isPurge -and $f.LastWriteTime -lt $cutoff) {
            if ($DryRun) {
                Write-Host "DRYRUN would remove: $($f.FullName)"
            } else {
                Remove-Item -LiteralPath $f.FullName -Force
                Write-Host "Removed: $($f.FullName)"
            }
        }
    }
}
Write-Host "Cleanup done (retention=$Days days, dryrun=$DryRun)."
