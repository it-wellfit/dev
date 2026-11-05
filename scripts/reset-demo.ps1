# ============================================================================
# reset-demo.ps1 - Wipe the DEMO database and rebuild it from scratch.
#
# Reset semantics (docs/demo/DEMO_DATA.md): the demo database is a separate
# file (data\demo.db); the production database (data\flaps.db) is untouched.
#
# Usage:  powershell -File scripts\reset-demo.ps1 [-Port 8093]
# ============================================================================

param(
    [int]$Port = 8093,
    [string]$AdminPassword = "demo-admin-2026"
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$dataDir = Join-Path $root "data"

Write-Host "== Flaps DEMO reset =="
Get-ChildItem $dataDir -Filter "demo.db*" -ErrorAction SilentlyContinue |
    ForEach-Object { Write-Host "  removing $($_.Name)"; Remove-Item $_.FullName -Force }

Write-Host "Re-seeding demo data..."
& (Join-Path $PSScriptRoot "seed-demo.ps1") -Port $Port -AdminPassword $AdminPassword
