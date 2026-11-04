# format-manager.ps1 — Unified Format Management & Conversion Entry Point (PowerShell 7+ LTS)
# ALL formats preserved — nothing is deleted. Only additive expansion.
# Commands: list, matrix, convert, validate, info
# Requires: PowerShell 7.0+
# Usage: pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\format-manager.ps1 <command> [options]

#requires -Version 7.0

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('list', 'matrix', 'convert', 'validate', 'info')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Path,

    [Parameter()]
    [string]$FromFormat,

    [Parameter()]
    [string]$ToFormat,

    [Parameter()]
    [string]$Password,

    [Parameter()]
    [switch]$Compress,

    [Parameter()]
    [switch]$Force,

    [Parameter()]
    [switch]$Recursive,

    [Parameter()]
    [string]$OutputPath
)

$ErrorActionPreference = 'Stop'
$scriptDir = Split-Path -Path $PSCommandPath -Parent
$encodingTools = Join-Path -Path $scriptDir -ChildPath 'encoding-tools.ps1'
$securePack = Join-Path -Path $scriptDir -ChildPath 'secure-pack.ps1'

# --- Format registry (additive only — never remove entries) ---
$script:formats = @(
    @{ Name='utf-8'; Category='text'; BOMCapable=$true; CJKCompatible=$true; Description='UTF-8 without BOM — Traditional/Simplified Chinese compatible' }
    @{ Name='big5'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Traditional Chinese (Taiwan/HK legacy)' }
    @{ Name='gb2312'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Simplified Chinese (Mainland legacy)' }
    @{ Name='gbk'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Simplified Chinese (GBK extended)' }
    @{ Name='gb18030'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Simplified Chinese (GB18030 full)' }
    @{ Name='shift_jis'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Japanese Shift-JIS' }
    @{ Name='euc-jp'; Category='text'; BOMCapable=$false; CJKCompatible=$true; Description='Japanese EUC-JP' }
    @{ Name='windows-1252'; Category='text'; BOMCapable=$false; CJKCompatible=$false; Description='Western Windows-1252' }
    @{ Name='iso-8859-1'; Category='text'; BOMCapable=$false; CJKCompatible=$false; Description='Western ISO-8859-1' }
    @{ Name='base64'; Category='binary-to-text'; Expansion='1.33x'; Description='Base64 — universal, preserved' }
    @{ Name='hex'; Category='binary-to-text'; Expansion='2.00x'; Description='Hexadecimal — debug/human-readable, preserved' }
    @{ Name='base91'; Category='binary-to-text'; Expansion='~1.23x'; Description='basE91 — more compact than Base64, preserved' }
    @{ Name='z85'; Category='binary-to-text'; Expansion='1.25x'; Description='ZeroMQ Z85 — string-safe, preserved' }
    @{ Name='sek'; Category='secure-kit'; Description='.sek — AES-256-CBC + PBKDF2 + HMAC-SHA256 + optional compression, preserved' }
)

# --- Command: list ---
function Invoke-List {
    Write-Output "Supported Formats (all preserved — nothing deleted):"
    Write-Output ("{0:-14} | {1:-18} | {2}" -f 'Format', 'Category', 'Description')
    Write-Output ("{0:-14} | {1:-18} | {2}" -f ('-'*14), ('-'*18), ('-'*60))
    foreach ($f in $script:formats) {
        $desc = $f.Description
        if ($f.Expansion) { $desc += " (expansion: $($f.Expansion))" }
        Write-Output ("{0:-14} | {1:-18} | {2}" -f $f.Name, $f.Category, $desc)
    }
    Write-Output ""
    Write-Output "Total formats: $($script:formats.Count)"
}

# --- Command: matrix ---
function Invoke-Matrix {
    $textFormats = $script:formats | Where-Object { $_.Category -eq 'text' }
    $b2tFormats = $script:formats | Where-Object { $_.Category -eq 'binary-to-text' }
    $secFormats = $script:formats | Where-Object { $_.Category -eq 'secure-kit' }

    Write-Output "Conversion Matrix — text encodings (all text <-> all text via UTF-8 pivot):"
    $header = ("{0:-16}" -f 'From\To') + ($textFormats | ForEach-Object { ("{0:-16}" -f $_.Name) })
    Write-Output $header
    Write-Output ("{0:-16}" -f ('-'*16)) + ($textFormats | ForEach-Object { ("{0:-16}" -f ('-'*16)) })
    foreach ($from in $textFormats) {
        $row = ("{0:-16}" -f $from.Name)
        foreach ($to in $textFormats) {
            $cell = if ($from.Name -eq $to.Name) { '  SAME  ' } else { '  UTF-8  ' }
            $row += ("{0:-16}" -f $cell)
        }
        Write-Output $row
    }

    Write-Output ""
    Write-Output "Conversion Matrix — binary-to-text (binary <-> each encoding):"
    $header2 = ("{0:-16}" -f 'Binary\B2T') + ($b2tFormats | ForEach-Object { ("{0:-16}" -f $_.Name) })
    Write-Output $header2
    Write-Output ("{0:-16}" -f ('-'*16)) + ($b2tFormats | ForEach-Object { ("{0:-16}" -f ('-'*16)) })
    $row = ("{0:-16}" -f 'raw-binary')
    foreach ($to in $b2tFormats) { $row += ("{0:-16}" -f '  encode  ') }
    Write-Output $row
    $row = ("{0:-16}" -f 'encoded-text')
    foreach ($to in $b2tFormats) { $row += ("{0:-16}" -f '  decode  ') }
    Write-Output $row

    Write-Output ""
    Write-Output "Secure kit (.sek): pack (raw -> .sek) and unpack (.sek -> raw), optional password + compression."
}

# --- Command: convert ---
function Invoke-Convert {
    param([string]$Path, [string]$From, [string]$To, [string]$Password, [switch]$Compress, [switch]$Force, [string]$Output)

    if (-not $From -or -not $To) {
        Write-Error "convert requires -FromFormat and -ToFormat"
        return
    }

    $lowerFrom = $From.ToLower()
    $lowerTo = $To.ToLower()

    # Text <-> text via encoding-tools.ps1 convert
    $textSet = @('utf-8','big5','gb2312','gbk','gb18030','shift_jis','euc-jp','windows-1252','iso-8859-1')
    if ($textSet -contains $lowerFrom -and $textSet -contains $lowerTo) {
        & $encodingTools convert -Path $Path -FromEncoding $From -ToEncoding $To -Force:$Force -OutputPath $Output
        return
    }

    # Binary <-> binary-to-text
    $b2tSet = @('base64','hex','base91','z85')
    if ($lowerFrom -eq 'raw-binary' -and $b2tSet -contains $lowerTo) {
        $cmd = $lowerTo + '-encode'
        & $encodingTools $cmd -Path $Path -OutputPath $Output
        return
    }
    if ($lowerTo -eq 'raw-binary' -and $b2tSet -contains $lowerFrom) {
        $cmd = $lowerFrom + '-decode'
        & $encodingTools $cmd -Path $Path -OutputPath $Output
        return
    }

    # Secure kit
    if ($lowerFrom -eq 'raw-binary' -and $lowerTo -eq 'sek') {
        & $securePack pack -InputPath $Path -OutputPath $Output -Password $Password -Compress:$Compress
        return
    }
    if ($lowerFrom -eq 'sek' -and $lowerTo -eq 'raw-binary') {
        & $securePack unpack -InputPath $Path -OutputPath $Output -Password $Password
        return
    }

    Write-Error "Unsupported conversion: $From -> $To. Run 'list' for supported formats."
}

# --- Command: validate ---
function Invoke-Validate {
    param([string]$Path)
    & $encodingTools validate -Path $Path -Recursive:$Recursive
}

# --- Command: info ---
function Invoke-Info {
    param([string]$Path)
    if (-not $Path) { Write-Error "info requires a file path"; return }
    if (-not (Test-Path -Path $Path)) { Write-Error "File not found: $Path"; return }

    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $ext = [System.IO.Path]::GetExtension($Path).ToLower()
    Write-Output "File: $Path"
    Write-Output ("Size: {0} bytes" -f $bytes.Length)

    # Detect BOM
    $bom = $null
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { $bom = 'UTF-8 BOM' }
    elseif ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE) { $bom = 'UTF-16 LE BOM' }
    elseif ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFE -and $bytes[1] -eq 0xFF) { $bom = 'UTF-16 BE BOM' }
    $bomDisplay = if ($bom) { $bom } else { 'none' }
    Write-Output ("BOM: {0}" -f $bomDisplay)

    # Null byte check (binary indicator)
    $hasNull = $false
    $limit = [Math]::Min($bytes.Length, 4096)
    0..($limit - 1) | ForEach-Object {
        if ($bytes[$_] -eq 0) { $hasNull = $true }
    }
    Write-Output ("Binary indicator (null byte in first 4KB): {0}" -f $hasNull)

    # Extension hint
    $known = $script:formats | Where-Object { $_.Name -eq $ext.TrimStart('.') }
    if ($known) { Write-Output ("Format hint: {0} — {1}" -f $known.Name, $known.Description) }
    else { Write-Output "Format hint: unknown extension" }

    # Try UTF-8 decode
    try {
        $text = [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false))
        Write-Output "UTF-8 decodable: yes"
        if ($text -contains [char]0xFFFD) { Write-Output "Mojibake marker (U+FFFD): yes — possible encoding issue" }
        else { Write-Output "Mojibake marker (U+FFFD): no" }
    } catch {
        Write-Output "UTF-8 decodable: no — likely non-UTF-8 or binary"
    }
}

switch ($Command) {
    'list' { Invoke-List }
    'matrix' { Invoke-Matrix }
    'convert' { Invoke-Convert -Path $Path -From $FromFormat -To $ToFormat -Password $Password -Compress:$Compress -Force:$Force -Output $OutputPath }
    'validate' { Invoke-Validate -Path $Path }
    'info' { Invoke-Info -Path $Path }
}
