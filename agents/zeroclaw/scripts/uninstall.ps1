<#
.SYNOPSIS
    Roll back a ZeroClaw Windows install.

.DESCRIPTION
    Stops and removes the "ZeroClaw Daemon" scheduled task when present, removes
    the extracted binary and restores the user PATH. With -Purge it also removes
    the config directory, which deletes agent memory and conversation history.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\uninstall.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\uninstall.ps1 -Purge -Yes
#>
[CmdletBinding()]
param(
    [string] $ConfigDir,
    [switch] $Purge,
    [switch] $Yes
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"

function Step { param([string] $M) Write-Host "==> $M" -ForegroundColor Cyan }
function Ok   { param([string] $M) Write-Host "    [OK]   $M" -ForegroundColor Green }
function Warn { param([string] $M) Write-Host "    [WARN] $M" -ForegroundColor Yellow }

if (-not $ConfigDir) { $ConfigDir = Join-Path $env:USERPROFILE ".zeroclaw" }
$BinDir = Join-Path $ConfigDir "bin"
$ExePath = Join-Path $BinDir "zeroclaw.exe"

Step "ZeroClaw rollback (Windows)"

# --- scheduled task ---------------------------------------------------------
$TaskName = "ZeroClaw Daemon"
$task = schtasks /Query /TN $TaskName 2>&1
if ($LASTEXITCODE -eq 0) {
    Step "Removing scheduled task '$TaskName'"
    if (Test-Path -LiteralPath $ExePath) {
        & $ExePath service stop 2>&1 | Out-Null
    }
    schtasks /Change /TN $TaskName /Disable 2>&1 | Out-Null
    schtasks /Delete /TN $TaskName /F 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) { Ok "task removed" } else { Warn "task removal failed; remove it manually via taskschd.msc" }
} else {
    Ok "no '$TaskName' task present"
}

# --- binary -----------------------------------------------------------------
if (Test-Path -LiteralPath $BinDir) {
    Step "Removing $BinDir"
    Remove-Item -LiteralPath $BinDir -Recurse -Force -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $BinDir) { Warn "could not remove $BinDir (is the daemon running?)" }
    else { Ok "binary removed" }
} else {
    Ok "no binary directory"
}

# --- PATH -------------------------------------------------------------------
Step "Restoring user PATH"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $UserPath) { $UserPath = "" }
$Parts = @($UserPath -split ";" | Where-Object { $_ -and $_.Trim() })
$Kept = @($Parts | Where-Object { $_.Trim() -ne $BinDir.TrimEnd("\") -and $_.Trim() -ne $BinDir })
if ($Kept.Count -ne $Parts.Count) {
    $new = $Kept -join ";"
    [Environment]::SetEnvironmentVariable("Path", $new, "User")
    Ok "removed $BinDir from user PATH"
} else {
    Ok "PATH unchanged"
}

# --- config / data ----------------------------------------------------------
if (Test-Path -LiteralPath $ConfigDir) {
    if ($Purge) {
        if (-not $Yes) {
            $answer = Read-Host "Delete $ConfigDir (agent memory and history will be lost)? [y/N]"
            if ($answer -notmatch '^(?i)y') { Warn "kept $ConfigDir"; return }
        }
        Step "Removing $ConfigDir"
        Remove-Item -LiteralPath $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue
        if (Test-Path -LiteralPath $ConfigDir) { Warn "could not fully remove $ConfigDir" } else { Ok "config and data removed" }
    } else {
        Warn "kept $ConfigDir (use -Purge to delete config, memory and logs)"
    }
} else {
    Ok "no config directory"
}

Write-Host ""
Ok "rollback complete"