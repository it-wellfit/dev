# 視覺回歸基線擷取工具（計畫 1793782615513 §7.2）
# 自持 headless Chrome：獨立 profile + --remote-debugging-port → CDP 登入 → 逐頁截圖。
# 為何不共用 browser-mcp 的 Chrome：該實例的 target 被 browser-mcp CDP 獨佔（連線被踢）、
# 背景分頁受 Chrome 節流（loadEventFired 不觸發）——自持實例繞開兩者。
# 用法：
#   powershell -ExecutionPolicy Bypass -File tools/capture-baseline.ps1 -BaseUrl "http://127.0.0.1:8092" `
#     -Username admin -Password Admin1234 -OutDir tests/visual/baseline -Prefix staging
# 產物：<OutDir>/<Prefix>-<page>.png（1440x900）；已存在者**覆蓋**（更新基線）。
param(
    [Parameter(Mandatory = $true)][string]$BaseUrl,
    [Parameter(Mandatory = $true)][string]$Username,
    [Parameter(Mandatory = $true)][string]$Password,
    [string]$OutDir = "tests/visual/baseline",
    [string]$Prefix = "staging",
    [int]$CdpPort = 9224,
    [switch]$AttachOnly
)
$ErrorActionPreference = "Stop"
$chrome = "C:\Program Files\Google\Chrome\Application\chrome.exe"
if (-not (Test-Path $chrome)) { Write-Error "Chrome not found: $chrome"; exit 2 }

$proc = $null
if (-not $AttachOnly) {
    $profileDir = Join-Path $env:TEMP "visual-baseline-profile"
    # 強殺殘留的 SingletonLock 會讓新實例誤判已有實例而退出（URL 轉交即失敗），
    # 且 profile 無需保留（登入由本腳本 CDP 重做）→ 每次全新建立。
    if (Test-Path $profileDir) { Remove-Item $profileDir -Recurse -Force -ErrorAction SilentlyContinue }
    New-Item -ItemType Directory -Force -Path $OutDir, $profileDir | Out-Null
}

# ── 1. 啟動自持 headless Chrome（AttachOnly 模式跳過：外部已啟動）──
if (-not $AttachOnly) {
    # PS 5.1 坑：-ArgumentList 陣列會把含逗號的參數（--window-size=1440,900）拆壞，
    # 必須以**單一字串**原樣傳遞。
    $argString = "--headless=new --disable-gpu --hide-scrollbars --remote-debugging-port=$CdpPort --user-data-dir=$profileDir --window-size=1440,900 about:blank"
    $proc = Start-Process -FilePath $chrome -ArgumentList $argString -PassThru -WindowStyle Hidden
}

try {
    # ── 2. 等待 DevTools HTTP 端點就緒 ──
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    $ready = $false
    while ([DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 500
        try {
            $null = Invoke-RestMethod -Uri "http://127.0.0.1:$CdpPort/json/version" -TimeoutSec 2
            $ready = $true
            break
        } catch {}
    }
    if (-not $ready) { Write-Error "DevTools endpoint not ready (port $CdpPort)"; exit 1 }
    Write-Host "CDP ready on port $CdpPort"

    # ── 3. 取得 about:blank 的 page target ──
    $targets = Invoke-RestMethod -Uri "http://127.0.0.1:$cdpPort/json/list" -TimeoutSec 5
    $page = $targets | Where-Object { $_.type -eq "page" } | Select-Object -First 1
    if (-not $page) { Write-Error "no page target"; exit 1 }
    $ws = New-Object System.Net.WebSockets.ClientWebSocket
    $ws.ConnectAsync([Uri]$page.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

    # ── 4. CDP 助手（Text frame；PS5.1 ArraySegment 用 New-Object @(,$bytes)；每幀逾時 5s）──
    function New-Segment([byte[]]$bytes) {
        $seg = New-Object System.ArraySegment[byte] -ArgumentList @(,$bytes)
        return ,$seg
    }
    $script:nextId = 1
    function Send-Cdp($ws, $method, $params) {
        $msg = @{ id = $script:nextId; method = $method }
        if ($params) { $msg.params = $params }
        $id = $script:nextId
        $script:nextId = $script:nextId + 1
        $json = ConvertTo-Json -InputObject $msg -Compress -Depth 6
        $bytes = [Text.Encoding]::UTF8.GetBytes($json)
        $seg = New-Segment $bytes
        $mt = [System.Net.WebSockets.WebSocketMessageType]::Text
        $null = $ws.SendAsync($seg, $mt, $true, [System.Threading.CancellationToken]::None).GetAwaiter().GetResult()
        return $id
    }
    function Receive-Cdp($ws, $waitId) {
        $buffer = New-Object byte[] 262144
        $ms = New-Object System.IO.MemoryStream
        while ($true) {
            $seg = New-Segment $buffer
            $task = $ws.ReceiveAsync($seg, [System.Threading.CancellationToken]::None)
            if (-not $task.Wait(15000)) { throw "cdp receive timeout (15s) waiting for id=$waitId" }
            $r = $task.Result
            if ($r.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) { throw "ws closed" }
            $ms.Write($buffer, 0, $r.Count)
            if ($r.EndOfMessage) {
                $text = [Text.Encoding]::UTF8.GetString($ms.ToArray())
                $ms.SetLength(0)
                $obj = $text | ConvertFrom-Json
                if ($obj.id -eq $waitId) { return $obj }
            }
        }
    }

    # ── 5. 視口鎖定 1440x900 ──
    $id = Send-Cdp $ws "Emulation.setDeviceMetricsOverride" @{ width = 1440; height = 900; deviceScaleFactor = 1.0; mobile = $false }
    $null = Receive-Cdp $ws $id

    # ── 6. CDP 登入：導航到 /login → Runtime.evaluate 填表 → form.submit()（zero-JS 表單）──
    $id = Send-Cdp $ws "Page.navigate" @{ url = "$BaseUrl/login" }
    $null = Receive-Cdp $ws $id
    Start-Sleep -Milliseconds 1200
    $id = Send-Cdp $ws "Runtime.evaluate" @{ expression = "document.querySelector('#username').value = '$Username'; document.querySelector('#password').value = '$Password'; document.querySelector('form').submit(); void 0" }
    $null = Receive-Cdp $ws $id
    Start-Sleep -Milliseconds 2000  # 登入 POST → 303 → dashboard

    # ── 7. 逐頁導航 + 截圖 ──
    $pages = @(
        @{ n = "dashboard";   u = "/" },
        @{ n = "transactions"; u = "/transactions" },
        @{ n = "accounts";    u = "/accounts" },
        @{ n = "bankpay";     u = "/bankpay" },
        @{ n = "ledger";      u = "/reports/ledger?account=CASH-001" },
        @{ n = "import";      u = "/import" },
        @{ n = "audit";       u = "/audit" },
        @{ n = "status";      u = "/status" }
    )
    foreach ($p in $pages) {
        $id = Send-Cdp $ws "Page.navigate" @{ url = "$BaseUrl$($p.u)" }
        $null = Receive-Cdp $ws $id
        Start-Sleep -Milliseconds 1500   # 渲染 + 同源字體載入
        $id = Send-Cdp $ws "Page.captureScreenshot" @{ format = "png" }
        $resp = Receive-Cdp $ws $id
        $out = Join-Path $OutDir "$Prefix-$($p.n).png"
        [IO.File]::WriteAllBytes($out, [Convert]::FromBase64String($resp.result.data))
        Write-Host ("captured: {0} ({1} bytes)" -f $out, (Get-Item $out).Length)
    }
    $ws.Dispose()
    Write-Host "baseline capture complete: $((Get-ChildItem $OutDir -Filter "$Prefix-*.png").Count) pages"
}
finally {
    if ($proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
    Get-Process chrome -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -eq "" } | Out-Null
}
