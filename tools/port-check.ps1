# 埠號衝突檢測（計畫 1793782615513 §4.2）
# 比對 docs/ops/PORTS.toml 登記埠與實際監聽埠；任一登記埠已被佔用 → exit 1。
# 用法：
#   powershell -ExecutionPolicy Bypass -File tools/port-check.ps1              # 全表檢查
#   powershell -ExecutionPolicy Bypass -File tools/port-check.ps1 -Port 8090   # 單埠檢查
# 參數化：-Registry 指定登記表路徑；-Port 指定單一埠（覆蓋全表模式）。
param(
    [string]$Registry = (Join-Path $PSScriptRoot "..\docs\ops\PORTS.toml"),
    [int]$Port = 0
)
$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Registry)) {
    Write-Error "Registry not found: $Registry"
    exit 2
}

# 1. 解析登記埠（單埠 port = N 與範圍 port_range = "A-B" 皆展開）。
$registered = @()
foreach ($line in (Get-Content -LiteralPath $Registry)) {
    if ($line -match '^\s*port\s*=\s*(\d+)') {
        $registered += [int]$Matches[1]
    }
    elseif ($line -match '^\s*port_range\s*=\s*"(\d+)-(\d+)"') {
        $registered += [int]$Matches[1]..[int]$Matches[2]
    }
}
if ($Port -gt 0) { $registered = @($Port) }

if ($registered.Count -eq 0) {
    Write-Error "No registered ports parsed from $Registry"
    exit 2
}

# 2. 實際監聽埠（全系統去重）。
$listening = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty LocalPort -Unique)

# 3. 衝突判定：登記埠 ∈ 監聽埠 → 失敗（註冊即閒置的預期是「登記埠不該有人佔」；
#    ephemeral 例外由呼叫端以 -Port 指定當下想申請的埠）。
$conflict = @($registered | Where-Object { $_ -in $listening })
if ($conflict.Count -gt 0) {
    Write-Error ("Port conflict: registered ports already in use: " + ($conflict -join ', '))
    exit 1
}

Write-Host ("OK: {0} registered ports, no conflict" -f $registered.Count)
exit 0
