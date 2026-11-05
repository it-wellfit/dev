# ============================================================================
# seed-demo.ps1 - Build a repeatable DEMO database for the Flaps workspace.
#
# Demo-data rules (docs/demo/DEMO_DATA.md):
#   * Every demo row is prefixed [DEMO] so it can never be confused with
#     production data.
#   * The demo database is a SEPARATE file (data\demo.db) - the production
#     database (data\flaps.db) is never touched.
#   * Seeding goes through the REAL HTTP API (server started temporarily),
#     so demo data exercises exactly the same validation as production.
#   * Re-running this script rebuilds the demo database from scratch
#     (repeatable demo).
#
# Usage:  powershell -File scripts\seed-demo.ps1 [-Port 8093] [-SkipBuild]
# ============================================================================

param(
    [int]$Port = 8093,
    [string]$AdminPassword = "demo-admin-2026",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$dataDir = Join-Path $root "data"
$dbPath = Join-Path $dataDir "demo.db"
$exe = Join-Path $root "target\debug\flaps-server.exe"
$cookieJar = Join-Path $env:TEMP "flaps-seed-cookies.txt"

Write-Host "== Flaps DEMO seeder =="

# --- 1) Fresh demo database (reset semantics: seed = rebuild) ---------------
if (Test-Path $dataDir) {
    Get-ChildItem $dataDir -Filter "demo.db*" -ErrorAction SilentlyContinue |
        Remove-Item -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
Write-Host "[1/6] Demo database reset: $dbPath"

# --- 2) Build the server binary if missing ----------------------------------
if (-not (Test-Path $exe) -and -not $SkipBuild) {
    Write-Host "[2/6] Building flaps-server (first run)..."
    Push-Location $root
    cargo build --locked -p platform --bin flaps-server
    Pop-Location
    if ($LASTEXITCODE -ne 0) { throw "build failed" }
} else {
    Write-Host "[2/6] Server binary present, skipping build"
}

# --- 3) Start a temporary server bound to the demo database -----------------
# Child processes inherit the current environment (PS 5.1 compatible).
$env:FLAPS_DB_URL = "sqlite://$($dbPath -replace '\\','/')?mode=rwc"
$env:FLAPS_PORT = "$Port"
$env:FLAPS_ADMIN_PASSWORD = $AdminPassword
$proc = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
Write-Host "[3/6] Temporary server started (pid $($proc.Id), port $Port)"

# --- 4) Wait for health ------------------------------------------------------
$ready = $false
for ($i = 0; $i -lt 30; $i++) {
    Start-Sleep -Milliseconds 500
    $code = curl.exe -s -o NUL -w "%{http_code}" "http://127.0.0.1:$Port/api/health" 2>$null
    if ($code -eq "200") { $ready = $true; break }
}
if (-not $ready) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    throw "server did not become healthy"
}
Write-Host "[4/6] Server healthy"

# --- 5) Login as admin and grab the CSRF token ------------------------------
curl.exe -s -c $cookieJar -o NUL -X POST `
    -d "username=admin&password=$AdminPassword" `
    "http://127.0.0.1:$Port/login" | Out-Null
$page = curl.exe -s -b $cookieJar "http://127.0.0.1:$Port/" 
$csrf = [regex]::Match(($page -join "`n"), 'name="csrf_token" value="([^"]+)"').Groups[1].Value
if (-not $csrf) { Stop-Process -Id $proc.Id -Force; throw "csrf token not found" }
Write-Host "[5/6] Logged in as admin (csrf acquired)"

# --- 6) Seed demo users + 10 [DEMO] transactions through the REAL API -------
# (debit = receiving/increasing side, credit = paying/decreasing side)
$demoTx = @(
    @("2026-09-05", "OFFICE-EXP",    "BANK-001",       "25000",  "[DEMO] 9月辦公室租金"),
    @("2026-09-15", "SUPPLIES-EXP",  "CASH-001",       "3200",   "[DEMO] 文具採購"),
    @("2026-10-05", "OFFICE-EXP",    "BANK-001",       "25000",  "[DEMO] 10月辦公室租金"),
    @("2026-10-12", "BANK-001",      "REVENUE-001",    "88000",  "[DEMO] 專案收入（收款）"),
    @("2026-10-20", "PAYROLL-EXP",   "BANK-001",       "65000",  "[DEMO] 10月薪資"),
    @("2026-11-01", "BANK-001",      "OPENING-EQUITY", "500000", "[DEMO] 期初資本（開帳）"),
    @("2026-11-05", "SUPPLIES-EXP",  "CASH-001",       "1850",   "[DEMO] 雜項採購"),
    @("2026-11-08", "CASH-001",      "REVENUE-001",    "42000",  "[DEMO] 零售收入（收款）"),
    @("2026-11-10", "UTILITIES-EXP", "BANK-001",       "6800",   "[DEMO] 水電費"),
    @("2026-11-12", "TRAVEL-EXP",    "CASH-001",       "5600",   "[DEMO] 出差差旅")
)

$okCount = 0
foreach ($tx in $demoTx) {
    curl.exe -s -b $cookieJar -o NUL -X POST `
        -d "csrf_token=$csrf" `
        --data-urlencode "debit_account=$($tx[1])" `
        --data-urlencode "credit_account=$($tx[2])" `
        --data-urlencode "amount=$($tx[3])" `
        --data-urlencode "currency=TWD" `
        --data-urlencode "description=$($tx[4])" `
        --data-urlencode "transaction_date=$($tx[0])" `
        "http://127.0.0.1:$Port/transactions" | Out-Null
    if ($LASTEXITCODE -eq 0) { $okCount++ }
}
Write-Host "      Seeded $okCount / $($demoTx.Count) demo transactions"

# Demo users (finance can post, viewer is read-only)
$demoUsers = @(
    @("demo-finance", "demo-finance-2026", "FINANCE"),
    @("demo-viewer",  "demo-viewer-2026",  "STORE")
)
foreach ($u in $demoUsers) {
    curl.exe -s -b $cookieJar -o NUL -X POST `
        -d "csrf_token=$csrf" `
        --data-urlencode "username=$($u[0])" `
        --data-urlencode "password=$($u[1])" `
        --data-urlencode "role_code=$($u[2])" `
        "http://127.0.0.1:$Port/admin/users" | Out-Null
}
Write-Host "      Demo users created: demo-finance / demo-viewer"

# --- Cleanup: stop the temporary server --------------------------------------
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Remove-Item $cookieJar -Force -ErrorAction SilentlyContinue

Write-Host "[6/6] Done."
Write-Host ""
Write-Host "DEMO READY"
Write-Host "  Database : $dbPath"
Write-Host "  Accounts : admin / demo-finance / demo-viewer"
Write-Host "  Start    : `$env:FLAPS_DB_URL='sqlite://$($dbPath -replace '\\','/')?mode=rwc'; `$env:FLAPS_PORT='8090'; target\debug\flaps-server.exe"
Write-Host "  Reset    : powershell -File scripts\reset-demo.ps1"
