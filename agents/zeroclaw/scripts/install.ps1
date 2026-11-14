<#
.SYNOPSIS
    Install ZeroClaw on Windows from the pinned GitHub release.

.DESCRIPTION
    Downloads the pinned release archive, verifies its SHA-256 against both the
    expected digest and the release SHA256SUMS file, extracts the binary into the
    user profile, adds it to the user PATH (no administrator rights required) and
    renders the portable config template into the ZeroClaw config directory.

    The script is idempotent: an already-installed matching version is left alone.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1 -Force -SkipConfig
#>
[CmdletBinding()]
param(
    [string] $Version = "0.9.0",
    [string] $PrimaryModel = "qwen3.5:4b",
    [string] $FallbackModel = "gemini-3.8-flash",
    [string] $ConfigDir,
    [switch] $Force,
    [switch] $SkipConfig,
    [switch] $RenderOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

# Expected digest of zeroclaw-x86_64-pc-windows-msvc.zip for the pinned version.
# Source: GitHub release asset digest for v0.9.0. Update this constant together
# with -Version; never install from "latest".
$ExpectedSha256 = @{
    "0.9.0" = "f81fd10b8663c22b3f7c66e622f5f7249a2404a6973c32a5bd7f4af86a2afd6d"
}

$AssetName = "zeroclaw-x86_64-pc-windows-msvc.zip"

function Write-Step { param([string] $Message) Write-Host "==> $Message" -ForegroundColor Cyan }
function Write-Ok   { param([string] $Message) Write-Host "    [OK]   $Message" -ForegroundColor Green }
function Write-Warn { param([string] $Message) Write-Host "    [WARN] $Message" -ForegroundColor Yellow }
function Fail       { param([string] $Message) Write-Host "    [FAIL] $Message" -ForegroundColor Red; exit 1 }

if (-not $ExpectedSha256.ContainsKey($Version)) {
    Fail "No pinned SHA-256 for version '$Version'. Add it to `$ExpectedSha256 after verifying the release."
}
$Sha256 = $ExpectedSha256[$Version]

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
} catch {
    Write-Warn "Could not force TLS 1.2; continuing with the system default."
}

$Home0      = $env:USERPROFILE
$InstallDir = Join-Path $Home0 ".zeroclaw\bin"
$ExePath    = Join-Path $InstallDir "zeroclaw.exe"
if (-not $ConfigDir) { $ConfigDir = Join-Path $Home0 ".zeroclaw" }
$ConfigPath = Join-Path $ConfigDir "config.toml"

Write-Step "ZeroClaw installer (Windows, pinned v$Version)"

# --- 1. Already installed? --------------------------------------------------
$InstalledVersion = ""
if (Test-Path -LiteralPath $ExePath) {
    try {
        $raw = & $ExePath --version 2>$null
        if ($raw) {
            $m = [regex]::Match(($raw -join " "), "\d+\.\d+\.\d+")
            if ($m.Success) { $InstalledVersion = $m.Value }
        }
    } catch {
        Write-Warn "Existing binary could not be executed; reinstalling."
    }
}

if ($RenderOnly) {
    Write-Warn "Render-only mode: no binary download, no PATH change."
} elseif ($InstalledVersion -eq $Version -and -not $Force) {
    Write-Ok "zeroclaw $Version already installed at $ExePath"
} else {
    if ($InstalledVersion -and $InstalledVersion -ne $Version) {
        Write-Warn "Replacing installed version $InstalledVersion with $Version."
    }

    $TempDir = Join-Path $env:TEMP ("zeroclaw-install-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
    $ZipPath = Join-Path $TempDir $AssetName
    $SumsPath = Join-Path $TempDir "SHA256SUMS"
    $BaseUrl = "https://github.com/zeroclaw-labs/zeroclaw/releases/download/v$Version"

    try {
        Write-Step "Downloading $AssetName"
        Invoke-WebRequest -Uri "$BaseUrl/$AssetName" -OutFile $ZipPath -UseBasicParsing
        Write-Ok "Downloaded $([math]::Round((Get-Item -LiteralPath $ZipPath).Length / 1MB, 1)) MiB"

        Write-Step "Downloading SHA256SUMS"
        Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS" -OutFile $SumsPath -UseBasicParsing

        Write-Step "Verifying SHA-256"
        $Actual = (Get-FileHash -LiteralPath $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($Actual -ne $Sha256) {
            Fail "SHA-256 mismatch. expected=$Sha256 actual=$Actual"
        }
        Write-Ok "Archive digest matches the pinned value"

        $SumsText = ""
        if (Test-Path -LiteralPath $SumsPath) { $SumsText = (Get-Content -LiteralPath $SumsPath -Raw) }
        if ($SumsText -match [regex]::Escape($AssetName)) {
            if ($SumsText.ToLowerInvariant().Contains($Actual)) {
                Write-Ok "Release SHA256SUMS lists the same digest"
            } else {
                Fail "Release SHA256SUMS does not list the computed digest."
            }
        } else {
            Write-Warn "SHA256SUMS has no entry for $AssetName; pinned digest check is authoritative."
        }

        Write-Step "Extracting"
        if (-not (Test-Path -LiteralPath $InstallDir)) {
            New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        }
        Expand-Archive -LiteralPath $ZipPath -DestinationPath $InstallDir -Force
        if (-not (Test-Path -LiteralPath $ExePath)) {
            Fail "zeroclaw.exe not found after extraction in $InstallDir"
        }
        Write-Ok "Extracted to $InstallDir"
    } finally {
        try { Remove-Item -LiteralPath $TempDir -Recurse -Force -ErrorAction SilentlyContinue } catch { }
    }
}

# --- 2. PATH (user scope, no admin) -----------------------------------------
if ($RenderOnly) {
    Write-Warn "PATH left untouched (render-only)."
} else {
Write-Step "Checking user PATH"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $UserPath) { $UserPath = "" }
$Parts = @($UserPath -split ";" | Where-Object { $_ -and $_.Trim() })
if ($Parts -notcontains $InstallDir) {
    $NewPath = (@($InstallDir) + $Parts) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Ok "Added $InstallDir to the user PATH (open a new shell)"
} else {
    Write-Ok "$InstallDir already on the user PATH"
}
if (($env:Path -split ";") -notcontains $InstallDir) {
    $env:Path = "$InstallDir;$env:Path"
}
}

# --- 3. Render config -------------------------------------------------------
if ($SkipConfig) {
    Write-Warn "Skipped config rendering (-SkipConfig)."
} else {
    $Template = Join-Path $PSScriptRoot "..\config\zeroclaw.toml.tmpl"
    if (-not (Test-Path -LiteralPath $Template)) {
        Fail "Template not found: $Template"
    }
    if ((Test-Path -LiteralPath $ConfigPath) -and -not $Force) {
        Write-Warn "Config exists and was left untouched: $ConfigPath (use -Force to re-render)."
    } else {
        Write-Step "Rendering config"
        if (-not (Test-Path -LiteralPath $ConfigDir)) {
            New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null
        }

        $Workspace = Join-Path $ConfigDir "workspace"
        $Forbidden = @(
            (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))),
            (Join-Path $Home0 "Documents"),
            (Join-Path $Home0 "Desktop"),
            (Join-Path $Home0 "Downloads"),
            (Join-Path $Home0 ".ssh"),
            (Join-Path $Home0 ".aws"),
            (Join-Path $Home0 "AppData")
        ) | Where-Object { $_ -and (Test-Path -LiteralPath $_) } | Select-Object -Unique

        $TomlList = ($Forbidden | ForEach-Object { '    "' + ($_ -replace "\\", "\\") + '",' }) -join "`r`n"
        $Body = Get-Content -LiteralPath $Template -Raw -Encoding UTF8
        $Body = $Body.Replace("__WORKSPACE__", ($Workspace -replace "\\", "\\"))
        $Body = $Body.Replace("__FORBIDDEN__", $TomlList)
        $Body = $Body.Replace("__PRIMARY_MODEL__", $PrimaryModel)
        $Body = $Body.Replace("__FALLBACK_MODEL__", $FallbackModel)

        $Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        [IO.File]::WriteAllText($ConfigPath, $Body, $Utf8NoBom)
        Write-Ok "Wrote $ConfigPath"
    }
}

Write-Host ""
Write-Step "Installed. Next steps (run in a NEW shell):"
Write-Host '    zeroclaw --version'
Write-Host '    zeroclaw quickstart                # provider / risk / memory / channels'
Write-Host '    zeroclaw agent -a assistant -m "Reply with the single word Ready. Do not use tools."'
Write-Host '    zeroclaw daemon                    # gateway + dashboard on 127.0.0.1:42617'
Write-Host ""
Write-Warn "Set the Gemini key without writing it to disk:"
Write-Host "    `$env:ZEROCLAW_providers__models__gemini__free__api_key = '<key>'"
Write-Host "    (or) zeroclaw config set providers.models.gemini.free.api_key"
Write-Host ""
Write-Warn "Then run: powershell -ExecutionPolicy Bypass -File .\verify.ps1"