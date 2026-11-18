# CDP 頁面截圖工具（計畫 1793782615513 §7.2 視覺回歸基線管線）
# 流程：/json/new（PUT）開自有 tab → WebSocket 連該 target → 導航 → 截圖存檔 → /json/close。
# 自有 tab 不與 browser-mcp 的 CDP 連線衝突（每 target 單一 client 限制）。
# 前置：Chrome 以 automation 模式執行（user-data-dir 內 DevToolsActivePort 檔存在）。
# 用法：
#   powershell -ExecutionPolicy Bypass -File tools/capture-page.ps1 -CdpPort 51247 `
#     -Url "http://127.0.0.1:8092/" -Out baseline-dashboard.png
# 參數化：-CdpPort（DevToolsActivePort 第一行）、-Width/-Height（基線一致性：1440x900）、
#         -SettleMs（load 後靜置，等字體/渲染完成）。
param(
    [Parameter(Mandatory = $true)][int]$CdpPort,
    [Parameter(Mandatory = $true)][string]$Url,
    [Parameter(Mandatory = $true)][string]$Out,
    [int]$Width = 1440,
    [int]$Height = 900,
    [int]$SettleMs = 1200
)
$ErrorActionPreference = "Stop"

function New-Segment([byte[]]$bytes) {
    # PS 5.1 泛型建構坑：[ArraySegment[byte]]::new($bytes) 可能解析到錯誤重載
    # 送出空幀 → Chrome 靜默無回應。改用 New-Object @(,$bytes) 明確匹配 (T[]) 重載。
    # 回傳用逗號包裹防止管線展開 byte[]。
    $seg = New-Object System.ArraySegment[byte] -ArgumentList @(,$bytes)
    return ,$seg
}

function Send-Cdp($ws, $id, $method, $params) {
    $msg = @{ id = $id; method = $method }
    if ($params) { $msg.params = $params }
    $json = ConvertTo-Json -InputObject $msg -Compress -Depth 6
    $bytes = [Text.Encoding]::UTF8.GetBytes($json)
    $seg = New-Segment $bytes
    $msgType = [System.Net.WebSockets.WebSocketMessageType]::Text
    $token = [System.Threading.CancellationToken]::None
    $null = $ws.SendAsync($seg, $msgType, $true, $token).GetAwaiter().GetResult()
}

function Receive-CdpMessage($ws) {
    # WebSocket 訊息可能分多幀；迴圈組裝至 EndOfMessage。
    # 每幀 5 秒逾時——Chrome 靜默時外層 deadline 迴圈才有機會退出（不可無限阻塞）。
    $buffer = New-Object byte[] 262144
    $ms = New-Object System.IO.MemoryStream
    $token = [System.Threading.CancellationToken]::None
    do {
        $seg = New-Segment $buffer
        $task = $ws.ReceiveAsync($seg, $token)
        if (-not $task.Wait(5000)) { throw "websocket receive timeout (5s)" }
        $result = $task.Result
        if ($result.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) {
            throw "websocket closed by peer"
        }
        $ms.Write($buffer, 0, $result.Count)
    } while (-not $result.EndOfMessage)
    [Text.Encoding]::UTF8.GetString($ms.ToArray())
}

# 1. 開自有 tab（Chrome 111+ 要求 PUT）。
$enc = [Uri]::EscapeDataString($Url)
$newTarget = Invoke-RestMethod -Method Put -Uri "http://127.0.0.1:$CdpPort/json/new?$enc" -TimeoutSec 5
$targetId = $newTarget.id
$wsUrl = $newTarget.webSocketDebuggerUrl
if (-not $wsUrl) { Write-Error "no webSocketDebuggerUrl for new tab"; exit 2 }

try {
    $ws = New-Object System.Net.WebSockets.ClientWebSocket
    $ws.ConnectAsync([Uri]$wsUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

    # 2. 強制視口 1440x900（基線一致性；不受實際視窗大小影響）。
    Send-Cdp $ws 1 "Emulation.setDeviceMetricsOverride" @{
        width = $Width; height = $Height; deviceScaleFactor = 1.0; mobile = $false
    }

    # 3. 導航（新 tab 已帶 url 開啟；顯式 navigate 確保同一 target 上量測）。
    Send-Cdp $ws 2 "Page.navigate" @{ url = $Url }

    # 4. 等 Page.loadEventFired（靜態零 JS 頁；字體為同源本地檔）。
    $loaded = $false
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    while ([DateTime]::UtcNow -lt $deadline -and -not $loaded) {
        $raw = Receive-CdpMessage $ws
        $obj = $raw | ConvertFrom-Json
        if ($obj.method -eq "Page.loadEventFired") { $loaded = $true }
    }
    Start-Sleep -Milliseconds $SettleMs

    # 5. 截圖（PNG）→ base64 → 存檔。
    Send-Cdp $ws 3 "Page.captureScreenshot" @{ format = "png" }
    $saved = $false
    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while ([DateTime]::UtcNow -lt $deadline -and -not $saved) {
        $raw = Receive-CdpMessage $ws
        $obj = $raw | ConvertFrom-Json
        if ($obj.id -eq 3) {
            [IO.File]::WriteAllBytes($Out, [Convert]::FromBase64String($obj.result.data))
            $saved = $true
            Write-Host ("captured: {0} ({1} bytes)" -f $Out, (Get-Item $Out).Length)
        }
    }
    if (-not $saved) { Write-Error "captureScreenshot response timeout"; exit 1 }
    $ws.Dispose()
}
finally {
    # 6. 關閉自有 tab（不影響 browser-mcp 的分頁）。
    try { Invoke-RestMethod -Method Put -Uri "http://127.0.0.1:$CdpPort/json/close/$targetId" -TimeoutSec 3 | Out-Null } catch {}
}
