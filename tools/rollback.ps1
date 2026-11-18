# 一鍵回滾（計畫 1793782615513 §9.2）
# 從 $env:LOCALAPPDATA\Flaps ERP\backup\<TargetTag> 還原二進位與 DB 並重啟。
# 用法：powershell -ExecutionPolicy Bypass -File tools/rollback.ps1 -TargetTag pre-deploy -Port 8090
# 參數化：-TargetTag 備份目錄名；-Port 環境埠（prod=8090）；-RepoRoot 資料檔根目錄。
param(
    [Parameter(Mandatory = $true)][string]$TargetTag,
    [int]$Port = 8090,
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)
$ErrorActionPreference = "Stop"

$binDir = Join-Path $env:LOCALAPPDATA "Flaps ERP"
$backup = Join-Path $binDir "backup\$TargetTag"
if (-not (Test-Path -LiteralPath $backup)) {
    Write-Error "backup not found: $backup"
    exit 2
}

# 1. 停止服務（該埠的監聽 process）。
Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 800

# 2. 還原二進位。
Copy-Item (Join-Path $backup "auxbridge-server.exe") (Join-Path $binDir "auxbridge-server.exe") -Force -ErrorAction SilentlyContinue
# 相容舊命名（flaps-server.exe，11/10 之前建置）。
Copy-Item (Join-Path $backup "flaps-server.exe") (Join-Path $binDir "flaps-server.exe") -Force -ErrorAction SilentlyContinue

# 3. 還原資料庫（覆蓋前先快照現況供事後鑑識）。
$forensic = Join-Path $binDir ("backup\pre-rollback-" + (Get-Date -Format yyyyMMddHHmmss))
New-Item -ItemType Directory -Force -Path $forensic | Out-Null
Get-ChildItem (Join-Path $RepoRoot "data") -Filter "*.db" -ErrorAction SilentlyContinue |
    Copy-Item -Destination $forensic -Force -ErrorAction SilentlyContinue
Copy-Item (Join-Path $backup "*.db") (Join-Path $RepoRoot "data") -Force -ErrorAction SilentlyContinue

# 4. 重啟（prod 埠；DB URL 依 PORTS.toml 登記——此處以參數傳遞保持單檔可執行）。
$env:AUXBRIDGE_PORT = "$Port"
$env:AUXBRIDGE_DB_URL = "sqlite://$RepoRoot/data/prod.db?mode=rwc"
$exe = Get-ChildItem $binDir -Filter "auxbridge-server.exe" -ErrorAction SilentlyContinue |
    Select-Object -First 1
if (-not $exe) { $exe = Get-ChildItem $binDir -Filter "flaps-server.exe" | Select-Object -First 1 }
if (-not $exe) { Write-Error "server binary not found in $binDir"; exit 2 }
Start-Process -FilePath $exe.FullName -WindowStyle Hidden

Write-Host "Rolled back to $TargetTag on port $Port (forensic snapshot: $forensic)"
