# 部署腳本（計畫 1793782615513 §9.2.1）
# 流程：port-check → 備份現版（exe＋DB）→ 停舊 → 換新 → 起 → /health 驗證 → 失敗自動回滾。
# 前置：新二進位已放置於 $binDir\incoming\auxbridge-server.exe（由 CI artifact 或本機建置複製）。
# 用法：powershell -ExecutionPolicy Bypass -File tools/deploy.ps1 -Env prod -Tag v0.2.0
# 參數化：-Env 對應埠與 DB（dev=8091/staging=8092/prod=8090）；-Tag 僅供記錄。
param(
    [ValidateSet("dev", "staging", "prod")][string]$Env = "dev",
    [string]$Tag = "",
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)
$ErrorActionPreference = "Stop"

$port = @{ dev = 8091; staging = 8092; prod = 8090 }[$Env]
$binDir = Join-Path $env:LOCALAPPDATA "Flaps ERP"
$exe = Join-Path $binDir "auxbridge-server.exe"
$incoming = Join-Path $binDir "incoming\auxbridge-server.exe"

# 0. 前置檢查：新二進位存在。
if (-not (Test-Path -LiteralPath $incoming)) {
    Write-Error "incoming binary not found: $incoming（請先放置新版本）"
    exit 2
}

# 1. 埠衝突檢查。
& (Join-Path $PSScriptRoot "port-check.ps1") -Port $port

# 2. 備份現版（exe + DB）。
$backupDir = Join-Path $binDir ("backup\pre-deploy-" + (Get-Date -Format yyyyMMddHHmmss))
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
Copy-Item $exe "$backupDir\" -Force -ErrorAction SilentlyContinue
Get-ChildItem (Join-Path $RepoRoot "data") -Filter "*.db" -ErrorAction SilentlyContinue |
    Copy-Item -Destination $backupDir -Force -ErrorAction SilentlyContinue

# 3. 停止舊 process。
Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue |
    ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }
Start-Sleep -Milliseconds 800

# 4. 換上新二進位。
Copy-Item $incoming $exe -Force

# 5. 啟動（DB 依環境對應；見 docs/ops/PORTS.toml）。
$env:AUXBRIDGE_PORT = "$port"
$env:AUXBRIDGE_DB_URL = "sqlite://$RepoRoot/data/$Env.db?mode=rwc"
$env:AUXBRIDGE_BIND = "127.0.0.1"
Start-Process -FilePath $exe -WindowStyle Hidden

# 6. /health 驗證（30 秒逾時；觸發條件見 §9.1）。
$ok = $false
for ($i = 0; $i -lt 30 -and -not $ok; $i++) {
    Start-Sleep -Seconds 1
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$port/health" -UseBasicParsing -TimeoutSec 2
        if ($r.StatusCode -eq 200) { $ok = $true }
    } catch {}
}
if (-not $ok) {
    Write-Error "Health check failed after deploy; rolling back"
    & (Join-Path $PSScriptRoot "rollback.ps1") -TargetTag (Split-Path $backupDir -Leaf) -Port $port -RepoRoot $RepoRoot
    exit 1
}

Write-Host "Deployed to $Env on port $port (tag=$Tag)"
