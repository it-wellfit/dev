# session-end.ps1 - finalize session manifest and update index
# Usage: powershell -ExecutionPolicy Bypass -File scripts\session-end.ps1 -SessionId <id> [-Status "complete"]

param(
    [Parameter(Mandatory = $true)]
    [string]$SessionId,
    [string]$Status = "complete"
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$sessionRoot = Join-Path $root ".kilo\sessions\$SessionId"
$manifestPath = Join-Path $sessionRoot "manifest.md"

if (-not (Test-Path $manifestPath)) {
    throw "Manifest not found: $manifestPath. Run session-start.ps1 first."
}

# Resolve git revision at end
$gitHead = "nogit"
$dirty = "false"
try {
    Push-Location $root
    $gh = & git rev-parse --short HEAD 2>$null
    if ($LASTEXITCODE -eq 0 -and $gh) { $gitHead = $gh.Trim() }
    $st = & git status --porcelain 2>$null
    if ($st) { $dirty = "true" }
} catch {
    $gitHead = "nogit"
} finally {
    Pop-Location
}

# Update manifest fields
$raw = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8
$raw = $raw -replace 'git_head_end: \(pending\)', "git_head_end: $gitHead"
$raw = $raw -replace 'dirty_end: \(pending\)', "dirty_end: $dirty"
# Append status line to Notes if not present
if ($raw -notmatch 'Status:') {
    $raw = $raw.TrimEnd() + "`n- Status: $Status`n"
}
Set-Content -LiteralPath $manifestPath -Value $raw -Encoding UTF8

# Update index.md
$indexPath = Join-Path $root ".kilo\sessions\index.md"
$created = (Get-Date).ToString("yyyy-MM-dd")
$projDirs = Get-ChildItem -LiteralPath $sessionRoot -Directory -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name
# Determine projects from artifacts dirs
$projList = @()
Get-ChildItem -LiteralPath $root -Directory | ForEach-Object {
    $art = Join-Path $_.FullName "artifacts\$SessionId"
    if (Test-Path $art) { $projList += $_.Name }
}
$projStr = ($projList -join ", ")
if ($projStr -eq "") { $projStr = "(none)" }

if (Test-Path $indexPath) {
    $indexRaw = Get-Content -LiteralPath $indexPath -Raw -Encoding UTF8
    if ($indexRaw -notmatch [regex]::Escape($SessionId)) {
        $row = "| $SessionId | $projStr | (see manifest) | $created | (start..$gitHead) | $Status |"
        # Insert row right after the table separator line
        $lines = $indexRaw -split "`r?`n"
        $inserted = $false
        $out = @()
        foreach ($ln in $lines) {
            $out += $ln
            if (-not $inserted -and $ln -match '^\|[-| ]+\|$') {
                $out += $row
                $inserted = $true
            }
        }
        if (-not $inserted) { $out += $row }
        $indexRaw = $out -join "`n"
        Set-Content -LiteralPath $indexPath -Value $indexRaw -Encoding UTF8
    }
}

Write-Host "Session finalized: $SessionId (status=$Status, head=$gitHead, dirty=$dirty)"
