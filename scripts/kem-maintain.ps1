# kem-maintain.ps1 — Phase I 自維運路由（單一事實來源）
# 排程：每 15 分鐘，週一–五 08:00–18:00（schtasks 註冊於本檔尾註解）
# 效用：kem sync → lint → finalize + Ollama 守護；全程日誌，絕不靜默
# 可逆：schtasks /end /tn kem-maintain; schtasks /delete /tn kem-maintain /f
# 規範：.kilo/plans/1793966971740-autonomous-engineering-evolution-workflow.md §10

$ErrorActionPreference = "Continue"
$Root    = "C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Dev"
$LogDir  = "$env:LOCALAPPDATA\kem-maintain"
$Log     = "$LogDir\kem-maintain.log"
$KeepDays = 7

# ── 窗口守衛：排程每 15 分鐘全時觸發；僅平日 08:00–18:00 實際執行 ──
# （schtasks CLI 無法表達「分鐘級＋平日＋時段」三條件，守衛在腳本內 enforce）
$now = Get-Date
$isWeekday = $now.DayOfWeek -in @([System.DayOfWeek]::Monday, [System.DayOfWeek]::Tuesday, [System.DayOfWeek]::Wednesday, [System.DayOfWeek]::Thursday, [System.DayOfWeek]::Friday)
$inHours = ($now.Hour -ge 8 -and $now.Hour -lt 18)
if (-not ($isWeekday -and $inHours)) {
    exit 0  # 窗口外靜默退出（無日誌噪音）
}

if (-not (Test-Path -LiteralPath $LogDir)) {
    New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
}

function Write-Log([string]$Level, [string]$Msg) {
    $ts = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    "$ts [$Level] $Msg" | Add-Content -LiteralPath $Log -Encoding UTF8
}

# 日誌輪替：僅保留最近 KeepDays 天
$cut = (Get-Date).AddDays(-$KeepDays)
Get-ChildItem -LiteralPath $LogDir -Filter "kem-maintain*.log" -ErrorAction SilentlyContinue |
    Where-Object { $_.LastWriteTime -lt $cut } |
    Remove-Item -Force -ErrorAction SilentlyContinue

Write-Log "INFO" "=== run start (pid $PID) ==="

# ── 0) Ollama 守護（冪等；不可達才拉起） ──
$ollamaUp = $false
try {
    $null = Invoke-RestMethod -Uri "http://127.0.0.1:11434/api/tags" -TimeoutSec 5
    $ollamaUp = $true
    Write-Log "INFO" "ollama: already running"
} catch {
    Write-Log "WARN" "ollama: not reachable — starting guard"
    $exe = "$env:LOCALAPPDATA\Programs\Ollama\ollama.exe"
    if (Test-Path -LiteralPath $exe) {
        Start-Process -FilePath $exe -ArgumentList "serve" -WindowStyle Hidden
        Start-Sleep -Seconds 6
        try {
            $null = Invoke-RestMethod -Uri "http://127.0.0.1:11434/api/tags" -TimeoutSec 10
            $ollamaUp = $true
            Write-Log "INFO" "ollama: guard started OK"
        } catch {
            Write-Log "ERROR" "ollama: guard start FAILED (degraded mode continues)"
        }
    } else {
        Write-Log "ERROR" "ollama: exe not found at $exe (degraded mode continues)"
    }
}

# ── 1) kem sync（重驗證引證＋信心刷新） ──
Push-Location $Root
try {
    $syncOut = cargo run --locked -q -p kem -- --root knowledge sync 2>&1
    $syncExit = $LASTEXITCODE
    $lvl = "ERROR"; if ($syncExit -eq 0) { $lvl = "INFO" }
    Write-Log $lvl "kem sync exit=$syncExit :: $($syncOut -join ' | ')"
} catch {
    Write-Log "ERROR" "kem sync threw: $_"
}

# ── 2) kem lint（L1–L3 規則） ──
try {
    $lintOut = cargo run --locked -q -p kem -- --root knowledge lint 2>&1
    $lintExit = $LASTEXITCODE
    $lvl = "WARN"; if ($lintExit -eq 0) { $lvl = "INFO" }
    Write-Log $lvl "kem lint exit=$lintExit :: $($lintOut -join ' | ')"
} catch {
    Write-Log "ERROR" "kem lint threw: $_"
}

# ── 3) kem finalize（index/log/dashboard 重生成） ──
try {
    $finOut = cargo run --locked -q -p kem -- --root knowledge finalize 2>&1
    $finExit = $LASTEXITCODE
    $lvl = "ERROR"; if ($finExit -eq 0) { $lvl = "INFO" }
    Write-Log $lvl "kem finalize exit=$finExit :: $($finOut -join ' | ')"
} catch {
    Write-Log "ERROR" "kem finalize threw: $_"
}

# ── 4) dashboard 零 JS 不變量自檢（憲章 §2.3） ──
try {
    $dash = "$Root\knowledge\dashboard.html"
    if (Test-Path -LiteralPath $dash) {
        $html = [System.IO.File]::ReadAllText($dash, [System.Text.UTF8Encoding]::new($false))
        $n = ([regex]::Matches($html, "<script")).Count
        $lvl = "ERROR"; if ($n -eq 0) { $lvl = "INFO" }
        Write-Log $lvl "dashboard script-count=$n (must be 0)"
    } else {
        Write-Log "ERROR" "dashboard.html missing after finalize"
    }
} catch {
    Write-Log "ERROR" "dashboard self-check threw: $_"
}

Pop-Location
Write-Log "INFO" "=== run end ==="
