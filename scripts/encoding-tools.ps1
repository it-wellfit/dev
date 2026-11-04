# encoding-tools.ps1 — Encoding & Format Conversion Toolkit (PowerShell 7+ LTS)
# Supports: UTF-8 (Traditional/Simplified Chinese), all Unicode encodings, Base-N, Hex, Binary
# Requires: PowerShell 7.0+
# Usage: pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\encoding-tools.ps1 <command> [options]

#requires -Version 7.0

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('detect', 'convert', 'validate', 'base64-encode', 'base64-decode', 'hex-encode', 'hex-decode', 'base91-encode', 'base91-decode', 'z85-encode', 'z85-decode', 'batch-validate')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Path,

    [Parameter()]
    [string]$FromEncoding,

    [Parameter()]
    [string]$ToEncoding = 'utf-8',

    [Parameter()]
    [switch]$Force,

    [Parameter()]
    [switch]$Recursive,

    [Parameter()]
    [string]$OutputPath,

    [Parameter()]
    [switch]$NoBom,

    [Parameter()]
    [int]$MaxFileSizeMB = 512
)

$ErrorActionPreference = 'Stop'
$script:workspaceRoot = Resolve-Path "$PSScriptRoot\.." | Select-Object -ExpandProperty Path

# --- Safety limits ---
$script:maxFileSizeBytes = [int64]$MaxFileSizeMB * 1MB

# --- BOM detection ---
function Test-Bom {
    param([string]$FilePath)
    $bytes = [System.IO.File]::ReadAllBytes($FilePath)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { return 'UTF-8 BOM' }
    if ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE) { return 'UTF-16 LE BOM' }
    if ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFE -and $bytes[1] -eq 0xFF) { return 'UTF-16 BE BOM' }
    if ($bytes.Length -ge 4 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE -and $bytes[2] -eq 0 -and $bytes[3] -eq 0) { return 'UTF-32 LE BOM' }
    if ($bytes.Length -ge 4 -and $bytes[0] -eq 0 -and $bytes[1] -eq 0 -and $bytes[2] -eq 0xFE -and $bytes[3] -eq 0xFF) { return 'UTF-32 BE BOM' }
    return $null
}

function Get-EncodingFromBom {
    param([string]$BomType)
    switch ($BomType) {
        'UTF-8 BOM' { return [System.Text.Encoding]::UTF8 }
        'UTF-16 LE BOM' { return [System.Text.Encoding]::Unicode }
        'UTF-16 BE BOM' { return [System.Text.Encoding]::BigEndianUnicode }
        'UTF-32 LE BOM' { return [System.Text.Encoding]::UTF32 }
        'UTF-32 BE BOM' { return [System.Text.Encoding]::GetEncoding('utf-32BE') }
        default { return [System.Text.Encoding]::UTF8 }
    }
}

# --- Binary detection (null byte anywhere = binary) ---
function Test-IsBinary {
    param([string]$FilePath)
    $bytes = [System.IO.File]::ReadAllBytes($FilePath)
    if ($bytes.Length -eq 0) { return $false }
    # Sample up to 64KB for performance vs accuracy balance
    $limit = [Math]::Min($bytes.Length, 65536)
    for ($i = 0; $i -lt $limit; $i++) {
        if ($bytes[$i] -eq 0) { return $true }
    }
    return $false
}

function Test-Mojibake {
    param([string]$Content)
    # Replacement character U+FFFD = definitive decoding error / mojibake marker
    return $content -contains [char]0xFFFD
}

# --- File size guard ---
function Test-FileSizeLimit {
    param([string]$FilePath)
    $info = Get-Item -Path $FilePath
    if ($info.Length -gt $script:maxFileSizeBytes) {
        Write-Error ("File too large for in-memory processing: {0:N2} MB (limit {1} MB). Increase -MaxFileSizeMB." -f ($info.Length / 1MB), $MaxFileSizeMB)
        return $false
    }
    return $true
}

# --- Item resolution (handles single file or directory) ---
function Get-FileItems {
    param([string]$Path)
    $absPath = (Resolve-Path -Path $Path -ErrorAction SilentlyContinue)?.Path
    if (-not $absPath) {
        Write-Output ("PATH NOT FOUND: {0}" -f $Path)
        return $null
    }
    if (Test-Path -Path $absPath -PathType Leaf) {
        return @(Get-Item -Path $absPath)
    }
    return Get-ChildItem -Path $absPath -File -Recurse:$Recursive
}

# --- Encoding detection ---
function Invoke-Detect {
    param([string]$Path)
    $items = Get-FileItems -Path $Path
    if ($null -eq $items) { return }
    foreach ($f in $items) {
        $fullName = $f.FullName
        if (Test-IsBinary -FilePath $fullName) {
            Write-Output ("BINARY  | {0}" -f $fullName)
            continue
        }
        $bom = Test-Bom -FilePath $fullName
        if ($bom) {
            Write-Output ("{0}      | {1}" -f $bom, $fullName)
            continue
        }
        $encodings = @('utf-8', 'big5', 'gb2312', 'gbk', 'gb18030', 'shift_jis', 'euc-jp', 'windows-1252', 'iso-8859-1')
        $detected = $null
        foreach ($enc in $encodings) {
            try {
                $content = [System.IO.File]::ReadAllText($fullName, [System.Text.Encoding]::GetEncoding($enc))
                $bytes = [System.Text.Encoding]::GetEncoding($enc).GetBytes($content)
                $roundtrip = [System.Text.Encoding]::GetEncoding($enc).GetString($bytes)
                if ($roundtrip -eq $content -and -not (Test-Mojibake -Content $content)) {
                    $detected = $enc
                    break
                }
            } catch { continue }
        }
        if ($detected) {
            Write-Output ("{0}      | {1}" -f $detected, $fullName)
        } else {
            Write-Output ("UNKNOWN  | {0}" -f $fullName)
        }
    }
}

# --- Encoding conversion ---
function Invoke-Convert {
    param([string]$Path, [string]$From, [string]$To, [switch]$Force, [switch]$NoBom, [string]$Output)
    $items = Get-FileItems -Path $Path
    if ($null -eq $items) { return }
    foreach ($f in $items) {
        $src = $f.FullName
        if (Test-IsBinary -FilePath $src) {
            Write-Warning ("Skip binary: {0}" -f $src)
            continue
        }
        if (-not (Test-FileSizeLimit -FilePath $src)) { continue }
        $srcEnc = $From
        if (-not $srcEnc) {
            $bom = Test-Bom -FilePath $src
            $srcEnc = if ($bom) { (Get-EncodingFromBom -BomType $bom).WebName } else { 'utf-8' }
        }
        $dstEnc = [System.Text.Encoding]::GetEncoding($To)
        $content = [System.IO.File]::ReadAllText($src, [System.Text.Encoding]::GetEncoding($srcEnc))
        if ($NoBom) {
            $dstEnc = New-Object System.Text.UTF8Encoding($false)
        }
        $dstPath = $Output
        if (-not $dstPath) {
            $dstPath = $src
            if (-not $Force) {
                Write-Warning ("Refuse in-place overwrite without -Force: {0}" -f $src)
                continue
            }
        } elseif ($Recursive) {
            $rel = $f.FullName.Substring($absPath.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar)
            $dstPath = Join-Path -Path $Output -ChildPath $rel
            $null = New-Item -ItemType Directory -Force -Path (Split-Path -Path $dstPath -Parent)
        }
        [System.IO.File]::WriteAllText($dstPath, $content, $dstEnc)
        Write-Output ("CONVERTED {1}->{2} | {0}" -f $dstPath, $srcEnc, $To)
    }
}

# --- Validation ---
function Invoke-Validate {
    param([string]$Path)
    $items = Get-FileItems -Path $Path
    if ($null -eq $items) { return }
    $failed = 0
    foreach ($f in $items) {
        $fullName = $f.FullName
        if (Test-IsBinary -FilePath $fullName) { continue }
        $bom = Test-Bom -FilePath $fullName
        $enc = if ($bom) { (Get-EncodingFromBom -BomType $bom).WebName } else { 'utf-8' }
        try {
            $content = [System.IO.File]::ReadAllText($fullName, [System.Text.Encoding]::GetEncoding($enc))
            if (Test-Mojibake -Content $content) {
                Write-Warning ("MOJIBAKE | {0} ({1})" -f $fullName, $enc)
                $failed++
            } else {
                Write-Output ("OK       | {0} ({1})" -f $fullName, $enc)
            }
        } catch {
            Write-Error ("FAIL     | {0} | {1}" -f $fullName, $_.Exception.Message)
            $failed++
        }
    }
    if ($failed -gt 0) {
        Write-Output "`n{0} file(s) have encoding issues." -f $failed
        exit 1
    }
    Write-Output "`nAll text files validated successfully."
}

# --- Base64 ---
function Invoke-Base64Encode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $encoded = [Convert]::ToBase64String($bytes)
    $dst = $OutputPath
    if (-not $dst) { $dst = $Path + '.base64' }
    [System.IO.File]::WriteAllText($dst, $encoded, [System.Text.UTF8Encoding]::new($false))
    Write-Output ("BASE64 ENCODED | {0} -> {1}" -f $Path, $dst)
}

function Invoke-Base64Decode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $text = [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false))
    $bytes = [Convert]::FromBase64String($text)
    $dst = $OutputPath
    if (-not $dst) { $dst = [System.IO.Path]::GetFileNameWithoutExtension($Path) }
    [System.IO.File]::WriteAllBytes($dst, $bytes)
    Write-Output ("BASE64 DECODED | {0} -> {1}" -f $Path, $dst)
}

# --- Hex ---
function Invoke-HexEncode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $hex = ($bytes | ForEach-Object { "{0:X2}" -f $_ }) -join ''
    $dst = $OutputPath
    if (-not $dst) { $dst = $Path + '.hex' }
    [System.IO.File]::WriteAllText($dst, $hex, [System.Text.UTF8Encoding]::new($false))
    Write-Output ("HEX ENCODED | {0} -> {1}" -f $Path, $dst)
}

function Invoke-HexDecode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $hex = [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false)).Trim()
    $bytes = for ($i = 0; $i -lt $hex.Length; $i += 2) { [Convert]::ToByte($hex.Substring($i, 2), 16) }
    $dst = $OutputPath
    if (-not $dst) { $dst = [System.IO.Path]::GetFileNameWithoutExtension($Path) }
    [System.IO.File]::WriteAllBytes($dst, $bytes)
    Write-Output ("HEX DECODED | {0} -> {1}" -f $Path, $dst)
}

# --- Base91 (basE91) — ~1.23x expansion, better than Base64's 1.33x ---
$script:base91Encode = "!""#$%&'()*+,/0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~".ToCharArray()
$script:base91Decode = @{}
0..($script:base91Encode.Length - 1) | ForEach-Object { $script:base91Decode[$script:base91Encode[$_]] = $_ }

function Invoke-Base91Encode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $enc = $script:base91Encode
    $b = [uint32]0
    $bn = 0
    $out = [System.Text.StringBuilder]::new()
    foreach ($byte in $bytes) {
        $b = [uint32]($b -bor ([uint32]$byte -shl $bn))
        $bn += 8
        while ($bn -gt 13) {
            $ev = $b -band 0x1FFF
            if ($ev -gt 88) {
                $b = $b -shr 13
                $bn -= 13
            } else {
                $ev = $b -band 0x3FFF
                $b = [uint32]($b -shr 14)
                $bn -= 14
            }
            [void]$out.Append($enc[$ev % 91])
            [void]$out.Append($enc[[math]::floor($ev / 91)])
        }
    }
    if ($bn -gt 0) {
        [void]$out.Append($enc[$b % 91])
        if ($bn -gt 7 -or $b -gt 90) {
            [void]$out.Append($enc[[math]::floor($b / 91)])
        }
    }
    $dst = $OutputPath
    if (-not $dst) { $dst = $Path + '.base91' }
    [System.IO.File]::WriteAllText($dst, $out.ToString(), [System.Text.UTF8Encoding]::new($false))
    Write-Output ("BASE91 ENCODED | {0} -> {1}" -f $Path, $dst)
}

function Invoke-Base91Decode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $text = [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false))
    $dec = $script:base91Decode
    $chars = $text.ToCharArray()
    $len = $chars.Length
    $queue = [uint32]0
    $nbits = 0
    $out = [System.Collections.Generic.List[byte]]::new()
    $i = 0
    while ($i + 1 -lt $len) {
        $v = [uint32]($dec[$chars[$i]] + $dec[$chars[$i + 1]] * 91)
        $bits = if (([uint32]($v -band 0x1FFF)) -gt 88) { 13 } else { 14 }
        $queue = [uint32]($queue -bor ([uint32]$v -shl $nbits))
        $nbits += $bits
        while ($nbits -ge 8) {
            $out.Add([byte]($queue -band 0xFF))
            $queue = [uint32]($queue -shr 8)
            $nbits -= 8
        }
        $i += 2
    }
    # Single trailing character (only when encoding end had en <= 7)
    if ($i -lt $len) {
        $v = [uint32]$dec[$chars[$i]]
        $en = 8 - $nbits
        $queue = [uint32]($queue -bor ([uint32]$v -shl $nbits))
        $nbits += $en
        while ($nbits -ge 8) {
            $out.Add([byte]($queue -band 0xFF))
            $queue = [uint32]($queue -shr 8)
            $nbits -= 8
        }
        $i++
    }
    $dst = $OutputPath
    if (-not $dst) { $outName = [System.IO.Path]::GetFileNameWithoutExtension($Path); $dst = Join-Path -Path (Split-Path -Path $Path -Parent) -ChildPath $outName }
    [System.IO.File]::WriteAllBytes($dst, $out.ToArray())
    Write-Output ("BASE91 DECODED | {0} -> {1}" -f $Path, $dst)
}

# --- Z85 (ZeroMQ spec:32) — ~1.25x expansion, string-safe ---
$script:z85Encode = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#".ToCharArray()
$script:z85Decode = @{}
0..($script:z85Encode.Length - 1) | ForEach-Object { $script:z85Decode[$script:z85Encode[$_]] = $_ }

function Invoke-Z85Encode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $enc = $script:z85Encode
    $out = [System.Text.StringBuilder]::new()
    $len = $bytes.Length
    # Prepend original length as 4-byte big-endian prefix so decoder can restore exact length.
    $prefix = [byte[]]@(
        [byte]($len -shr 24 -band 0xFF),
        [byte]($len -shr 16 -band 0xFF),
        [byte]($len -shr 8  -band 0xFF),
        [byte]($len        -band 0xFF)
    )
    $dataWithPrefix = $prefix + $bytes
    $paddedLen = [math]::ceiling($dataWithPrefix.Length / 4) * 4
    $padded = @(0) * $paddedLen
    0..($dataWithPrefix.Length - 1) | ForEach-Object { $padded[$_] = $dataWithPrefix[$_] }
    $i = 0
    while ($i -lt $paddedLen) {
        $n = [uint32]([uint64]$padded[$i] * 16777216 + [uint64]$padded[$i+1] * 65536 + [uint64]$padded[$i+2] * 256 + [uint64]$padded[$i+3])
        # Always emit 5 characters per 4-byte group. Never use the 'z' shortcut for zero:
        # 'z' is alphabet index 35 and is also a valid base-85 digit. Using it as a shortcut
        # creates decode ambiguity — a 5-char group starting with 'z' would be misread as
        # a 1-char shortcut, shifting all subsequent group boundaries.
        $chars = @(0,0,0,0,0)
        $pos = 4
        $nn = $n
        while ($pos -ge 0) {
            $chars[$pos] = $enc[$nn % 85]
            $nn = [uint32]([math]::floor([double]$nn / 85))
            $pos--
        }
        0..4 | ForEach-Object { [void]$out.Append($chars[$_]) }
        $i += 4
    }
    $dst = $OutputPath
    if (-not $dst) { $dst = $Path + '.z85' }
    [System.IO.File]::WriteAllText($dst, $out.ToString(), [System.Text.UTF8Encoding]::new($false))
    Write-Output ("Z85 ENCODED | {0} -> {1} (origLen={2})" -f $Path, $dst, $len)
}

function Invoke-Z85Decode {
    param([string]$Path)
    if (-not (Test-FileSizeLimit -FilePath $Path)) { return }
    $text = [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false))
    $dec = $script:z85Decode
    $chars = $text.ToCharArray()
    $len = $chars.Length
    if ($len % 5 -ne 0) {
        throw "Z85 encoded text length ($len) is not a multiple of 5: file may be corrupt"
    }
    $out = [System.Collections.Generic.List[byte]]::new()
    $i = 0
    while ($i -lt $len) {
        # Read exactly 5 characters per group — no 'z' shortcut, so 'z' is a normal digit (index 35)
        $group = @(0,0,0,0,0)
        0..4 | ForEach-Object {
            if (-not $dec.ContainsKey($chars[$i + $_])) {
                throw "Invalid Z85 character at position $($i + $_): '$($chars[$i + $_])'"
            }
            $group[$_] = $dec[$chars[$i + $_]]
        }
        $n = [uint64]0
        0..4 | ForEach-Object { $n = [uint64]($n * 85 + $group[$_]) }
        $out.Add([byte](($n -shr 24) -band 0xFF))
        $out.Add([byte](($n -shr 16) -band 0xFF))
        $out.Add([byte](($n -shr 8) -band 0xFF))
        $out.Add([byte]($n -band 0xFF))
        $i += 5
    }
    $allBytes = $out.ToArray()
    if ($allBytes.Length -lt 4) {
        throw "Z85 decoded output too short (<4 bytes): missing original-length prefix"
    }
    $origLen = ([uint32]$allBytes[0] -shl 24) -bor ([uint32]$allBytes[1] -shl 16) -bor ([uint32]$allBytes[2] -shl 8) -bor [uint32]$allBytes[3]
    if ($origLen -gt $allBytes.Length - 4) {
        throw "Z85 length prefix ($origLen) exceeds available data ($($allBytes.Length - 4) bytes): file may be truncated"
    }
    $originalBytes = $allBytes[4..($origLen + 3)]
    $dst = $OutputPath
    if (-not $dst) { $outName = [System.IO.Path]::GetFileNameWithoutExtension($Path); $dst = Join-Path -Path (Split-Path -Path $Path -Parent) -ChildPath $outName }
    [System.IO.File]::WriteAllBytes($dst, $originalBytes)
    Write-Output ("Z85 DECODED | {0} -> {1} (origLen={2}, decoded={3})" -f $Path, $dst, $origLen, $originalBytes.Length)
}

switch ($Command) {
    'detect' { Invoke-Detect -Path $Path }
    'convert' { Invoke-Convert -Path $Path -From $FromEncoding -To $ToEncoding -Force:$Force -NoBom:$NoBom -Output $OutputPath }
    'validate' { Invoke-Validate -Path $Path }
    'base64-encode' { Invoke-Base64Encode -Path $Path -OutputPath $OutputPath }
    'base64-decode' { Invoke-Base64Decode -Path $Path -OutputPath $OutputPath }
    'hex-encode' { Invoke-HexEncode -Path $Path -OutputPath $OutputPath }
    'hex-decode' { Invoke-HexDecode -Path $Path -OutputPath $OutputPath }
    'base91-encode' { Invoke-Base91Encode -Path $Path -OutputPath $OutputPath }
    'base91-decode' { Invoke-Base91Decode -Path $Path -OutputPath $OutputPath }
    'z85-encode' { Invoke-Z85Encode -Path $Path -OutputPath $OutputPath }
    'z85-decode' { Invoke-Z85Decode -Path $Path -OutputPath $OutputPath }
    'batch-validate' { Invoke-Validate -Path $Path }
}
