# session-start.ps1 - create session directories and manifest skeleton
# Usage: powershell -ExecutionPolicy Bypass -File scripts\session-start.ps1 -SessionId <id> -Projects "Wellfit,ekpi-platform" [-Goal "..."]

param(
    [Parameter(Mandatory = $true)]
    [string]$SessionId,
    [string]$Projects = "",
    [string]$Goal = ""
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$sessionIdRegex = '^\d{8}-[a-z0-9-]+-[a-z0-9-]+$'
if ($SessionId -notmatch $sessionIdRegex) {
    throw "Invalid SessionId '$SessionId'. Expected format: YYYYMMDD-<project>-<slug> (lowercase, hyphens)."
}

$sessionRoot = Join-Path $root ".kilo\sessions\$SessionId"
$projectList = @()
if ($Projects.Trim() -ne "") {
    $projectList = $Projects.Split(",") | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
}

# Resolve git revision (best effort)
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

# Create registry dirs
$regDirs = @("conversation", "tests", "runs")
foreach ($d in $regDirs) {
    $p = Join-Path $sessionRoot $d
    if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }
}

# Create per-project artifact dirs
foreach ($proj in $projectList) {
    $projRoot = Join-Path $root $proj
    if (-not (Test-Path $projRoot)) {
        Write-Warning "Project directory not found: $proj (skipping artifact dirs)"
        continue
    }
    $artRoot = Join-Path $projRoot "artifacts\$SessionId"
    $artDirs = @("conversation", "tests", "runs", "io\input", "io\output")
    foreach ($d in $artDirs) {
        $p = Join-Path $artRoot $d
        if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p -Force | Out-Null }
    }
    # io manifest skeleton
    $ioManifest = Join-Path $artRoot "io\manifest.md"
    if (-not (Test-Path $ioManifest)) {
        $ioContent = "# IO Manifest - $SessionId`n`n| Direction | File | SHA-256 | Source / Destination | Notes |`n|-----------|------|---------|----------------------|-------|`n"
        Set-Content -LiteralPath $ioManifest -Value $ioContent -Encoding UTF8
    }
}

# Write manifest skeleton
$manifestPath = Join-Path $sessionRoot "manifest.md"
if (-not (Test-Path $manifestPath)) {
    $ts = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ss")
    $projStr = ($projectList -join ", ")
    if ($projStr -eq "") { $projStr = "(none)" }
    $lines = @()
    $lines += "# Session: $SessionId"
    $lines += ""
    $lines += "- Project(s): $projStr"
    $lines += "- Goal: $Goal"
    $lines += "- Created: $ts"
    $lines += "- Owner: (set by caller)"
    $lines += "- git_head_start: $gitHead"
    $lines += "- git_head_end: (pending)"
    $lines += "- dirty_start: $dirty"
    $lines += "- dirty_end: (pending)"
    $lines += ""
    $lines += "## Artifacts"
    $lines += ""
    $lines += "| Type | Path | Tracked? |"
    $lines += "|------|------|----------|"
    $lines += "| conversation | .kilo/sessions/$SessionId/conversation/ | yes |"
    foreach ($proj in $projectList) {
        $lines += "| tests | $proj/artifacts/$SessionId/tests/ | partial |"
        $lines += "| runs | $proj/artifacts/$SessionId/runs/ | no |"
        $lines += "| io | $proj/artifacts/$SessionId/io/ | partial |"
    }
    $lines += ""
    $lines += "## Commands run"
    $lines += ""
    $lines += "## Notes"
    $lines += ""
    $content = $lines -join "`n"
    Set-Content -LiteralPath $manifestPath -Value $content -Encoding UTF8
}

Write-Host "SESSION_ID=$SessionId"
Write-Host "SESSION_ROOT=$sessionRoot"
if ($projectList.Count -gt 0) {
    $firstProj = $projectList[0]
    $firstArt = Join-Path $root "$firstProj\artifacts\$SessionId"
    Write-Host "ARTIFACT_ROOT=$firstArt"
}
Write-Host ""
Write-Host "Session created. Source these env vars in your shell:"
Write-Host "  `$env:SESSION_ID = '$SessionId'"
Write-Host "  `$env:SESSION_ROOT = '$sessionRoot'"
