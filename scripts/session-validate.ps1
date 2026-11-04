# session-validate.ps1 - validate session layout, naming, manifests, index, and scatter
# Usage: powershell -ExecutionPolicy Bypass -File scripts\session-validate.ps1 [-Strict]

param(
    [switch]$Strict
)

$ErrorActionPreference = "Continue"

$root = Split-Path -Parent $PSScriptRoot
$errors = @()
$warnings = @()

# 1. Session dirs
$sessionRoot = Join-Path $root ".kilo\sessions"
$sessionIds = @()
if (Test-Path $sessionRoot) {
    Get-ChildItem -LiteralPath $sessionRoot -Directory | ForEach-Object {
        $sessionIds += $_.Name
        $manifest = Join-Path $_.FullName "manifest.md"
        if (-not (Test-Path $manifest)) {
            $errors += "Missing manifest.md in session dir: $($_.Name)"
        } else {
            $m = Get-Content -LiteralPath $manifest -Raw -Encoding UTF8
            foreach ($field in @("git_head_start", "dirty_start", "Created:", "Project(s):")) {
                if ($m -notmatch [regex]::Escape($field)) {
                    $errors += "Manifest missing required field '$field': $($_.Name)"
                }
            }
        }
        if ($_.Name -notmatch '^\d{8}-[a-z0-9-]+-[a-z0-9-]+$') {
            $errors += "Invalid session id format: $($_.Name)"
        }
    }
}

# 2. index.md lists every session dir
$indexPath = Join-Path $sessionRoot "index.md"
if (Test-Path $indexPath) {
    $indexContent = Get-Content -LiteralPath $indexPath -Raw -Encoding UTF8
    foreach ($sid in $sessionIds) {
        if ($indexContent -notmatch [regex]::Escape($sid)) {
            $errors += "Session not listed in index.md: $sid"
        }
    }
} else {
    $errors += "Missing index.md: $indexPath"
}

# 3. Scatter check
$allowedRootFiles = @("Cargo.toml","Cargo.lock","build.rs","Dockerfile","k8s-deployment.yaml",".editorconfig",".python-version",".gitignore","DOCUMENTATION_MASTER_SPEC.md","GEMINI.md","WORKSPACE_CHARTER.md","FLAPS_ERP_COMPLETE_STATUS.md","PHASE1_COMPLETE.md","PHASE1_IMPLEMENTATION_README.md","FREE_AI_SYSTEM_UPDATE.md","flaps_erp_architecture_optimization_plan.md","flaps_erp_performance_analysis.md","flaps_erp_project_plan.md")
$allowedRootDirs = @(".github",".kilo",".playwright-mcp",".roo","agentre-bench","archive","core","data","docs","ekpi-platform","kilo-settings-skills-archive","oekap","platform","re-toolchain","rea-rust-upstream","scripts","skills.backup-%TIMESTAMP%","skills.backup-20261028","SyncMesh","target","web","Wellfit")

if (Test-Path $root) {
    Get-ChildItem -LiteralPath $root -Force | ForEach-Object {
        $name = $_.Name
        if ($_.PSIsContainer) {
            if ($allowedRootDirs -notcontains $name) {
                $warnings += "Unrecognized root directory (possible scatter): $name"
            }
        } else {
            if ($allowedRootFiles -notcontains $name) {
                # allow any .md? no - flag as stray
                $warnings += "Potential stray file at root: $name"
            }
        }
    }
}

# Desktop check (best effort; path may differ)
$desktop = [Environment]::GetFolderPath("Desktop")
$desktopExts = @(".md",".docx",".xlsx",".csv",".txt",".png",".html",".json",".pdf",".sqlite",".exe",".doc")
if (Test-Path $desktop) {
    Get-ChildItem -LiteralPath $desktop -Force -ErrorAction SilentlyContinue | ForEach-Object {
        if (-not $_.PSIsContainer) {
            $ext = [System.IO.Path]::GetExtension($_.Name).ToLower()
            if ($desktopExts -contains $ext) {
                $warnings += "Desktop work artifact (review for scatter): $($_.Name)"
            }
        }
    }
}

# Report
Write-Host "=== Session Validation ==="
Write-Host "Sessions found: $($sessionIds.Count)"
foreach ($e in $errors) { Write-Host "ERROR: $e" -ForegroundColor Red }
foreach ($w in $warnings) { Write-Host "WARN:  $w" -ForegroundColor Yellow }

if ($errors.Count -gt 0) {
    Write-Host "FAILED: $($errors.Count) error(s)." -ForegroundColor Red
    exit 1
}
if ($Strict -and $warnings.Count -gt 0) {
    Write-Host "FAILED (strict): $($warnings.Count) warning(s)." -ForegroundColor Red
    exit 1
}
Write-Host "OK."
exit 0
