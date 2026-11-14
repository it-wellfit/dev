<#
.SYNOPSIS
    Verify a ZeroClaw Windows install against the acceptance criteria.

.DESCRIPTION
    Non-interactive checks. Prints PASS / FAIL / UNKNOWN per check and exits with
    the number of failed checks (UNKNOWN does not fail the run, but is reported so
    it is never mistaken for a pass).

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\verify.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\verify.ps1 -Full
#>
[CmdletBinding()]
param(
    [string] $ConfigDir,
    [string] $Version = "0.9.0",
    [switch] $Full
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

$Failures = 0
function Pass { param([string] $Id, [string] $Msg) Write-Host "[PASS]    $Id :: $Msg" -ForegroundColor Green }
function Fail { param([string] $Id, [string] $Msg) Write-Host "[FAIL]    $Id :: $Msg" -ForegroundColor Red; $script:Failures++ }
function Unknown { param([string] $Id, [string] $Msg) Write-Host "[UNKNOWN] $Id :: $Msg" -ForegroundColor Yellow }

if (-not $ConfigDir) { $ConfigDir = Join-Path $env:USERPROFILE ".zeroclaw" }
$ExePath = Join-Path $env:USERPROFILE ".zeroclaw\bin\zeroclaw.exe"

function Get-Zeroclaw {
    if (Get-Command zeroclaw -ErrorAction SilentlyContinue) { return "zeroclaw" }
    if (Test-Path -LiteralPath $ExePath) { return $ExePath }
    return $null
}

Write-Host "== ZeroClaw verification (Windows) ==" -ForegroundColor Cyan

# V1 version ---------------------------------------------------------------
$cli = Get-Zeroclaw
if (-not $cli) {
    Fail "V1" "zeroclaw not found on PATH and not at $ExePath"
} else {
    try {
        $raw = (& $cli --version 2>$null) -join " "
        $m = [regex]::Match($raw, "\d+\.\d+\.\d+")
        if (-not $m.Success) {
            Fail "V1" "could not parse version from '$raw'"
        } elseif ($m.Value -ne $Version) {
            Fail "V1" "expected $Version, found $($m.Value)"
        } else {
            Pass "V1" "zeroclaw $Version ($raw)"
        }
    } catch {
        Fail "V1" "version check failed: $($_.Exception.Message)"
    }
}

# V2 config present --------------------------------------------------------
$ConfigPath = Join-Path $ConfigDir "config.toml"
if (Test-Path -LiteralPath $ConfigPath) { Pass "V2" "config at $ConfigPath" }
else { Fail "V2" "missing $ConfigPath (run install.ps1 or zeroclaw quickstart)" }

# V3 config validates ------------------------------------------------------
if ($cli -and (Test-Path -LiteralPath $ConfigPath)) {
    try {
        $json = (& $cli config migrate --json 2>&1) -join "`n"
        if ($json -match '"valid"\s*:\s*true') {
            Pass "V3" "config validates"
        } elseif ($json -match '"valid"\s*:\s*false') {
            Fail "V3" "config invalid: $json"
        } else {
            Unknown "V3" "could not read validity from output: $json"
        }
    } catch {
        Unknown "V3" "config check error: $($_.Exception.Message)"
    }
} else {
    Unknown "V3" "skipped (need CLI and config)"
}

# V4 config hygiene: no inline secrets -------------------------------------
if (Test-Path -LiteralPath $ConfigPath) {
    $hits = Select-String -LiteralPath $ConfigPath -Pattern '^\s*api_key\s*=\s*"' -AllMatches
    if ($hits) { Fail "V4" "inline api_key found in config.toml (use env override or config set)" }
    else { Pass "V4" "no inline api_key in config.toml" }
} else {
    Unknown "V4" "skipped (no config)"
}

# V5 user PATH -------------------------------------------------------------
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $UserPath) { $UserPath = "" }
$BinDir = Join-Path $env:USERPROFILE ".zeroclaw\bin"
if (($UserPath -split ";") -contains $BinDir) { Pass "V5" "bin on user PATH: $BinDir" }
else { Fail "V5" "bin missing from user PATH: $BinDir" }

# V6 Ollama reachable and model present ------------------------------------
try {
    $tags = Invoke-RestMethod -Uri "http://127.0.0.1:11434/api/tags" -TimeoutSec 5
    $names = @($tags.models | ForEach-Object { $_.name })
    if ($names.Count -gt 0) { Pass "V6" "ollama reachable, $($names.Count) model(s)" }
    else { Fail "V6" "ollama reachable but returned no models" }

    $primary = ""
    if (Test-Path -LiteralPath $ConfigPath) {
        $m = Select-String -LiteralPath $ConfigPath -Pattern '^\s*model\s*=\s*"([^"]+)"' | Select-Object -First 1
        if ($m) { $primary = $m.Matches[0].Groups[1].Value }
    }
    if ($primary) {
        $short = ($primary -split ":")[0]
        if ($names -contains $primary -or ($names | Where-Object { $_ -like "$short*" })) {
            Pass "V7" "primary model present in ollama: $primary"
        } else {
            Fail "V7" "primary model '$primary' not found in ollama library"
        }
    } else {
        Unknown "V7" "no primary model parsed from config"
    }
} catch {
    Unknown "V6" "ollama not reachable: $($_.Exception.Message)"
    Unknown "V7" "skipped (ollama unreachable)"
}

# V8 gateway health --------------------------------------------------------
try {
    $health = Invoke-WebRequest -Uri "http://127.0.0.1:42617/health" -TimeoutSec 5 -UseBasicParsing
    if ($health.StatusCode -eq 200) { Pass "V8" "gateway health OK" }
    else { Fail "V8" "gateway responded $($health.StatusCode)" }
} catch {
    Unknown "V8" "gateway not running (start it with: zeroclaw daemon)"
}

# V9 no scheduled task (phase 1) ------------------------------------------
$task = schtasks /Query /TN "ZeroClaw Daemon" 2>&1
if ($LASTEXITCODE -ne 0) { Pass "V9" "no 'ZeroClaw Daemon' scheduled task" }
else { Fail "V9" "'ZeroClaw Daemon' task exists; phase 1 expects it absent" }

# V10 optional end-to-end completion --------------------------------------
if ($Full) {
    if ($cli) {
        try {
            $answer = (& $cli agent -a assistant -m "Reply with the single word Ready. Do not use tools." 2>&1) -join "`n"
            if ($answer -match "(?i)ready") { Pass "V10" "completion via local model" }
            else { Fail "V10" "unexpected answer: $answer" }
        } catch {
            Fail "V10" "agent call failed: $($_.Exception.Message)"
        }
    } else {
        Unknown "V10" "skipped (no CLI)"
    }
} else {
    Unknown "V10" "skipped (use -Full for the live completion test)"
}

Write-Host ""
if ($Failures -gt 0) {
    Write-Host "RESULT: $Failures check(s) failed." -ForegroundColor Red
    exit $Failures
}
Write-Host "RESULT: no failures (review UNKNOWN items; UNKNOWN is not a pass)." -ForegroundColor Yellow