# secure-pack.ps1 — Secure Encapsulated Kit (.sek) (PowerShell 7+ LTS)
# Single-file format combining: deterministic serialization + compression + encryption + integrity
# Format: Magic(4) Version(1) Flags(1) AlgoID(1) KDF(1) KeyInfoLen(1) KeyInfo(var) IV(16) Ciphertext(var) HMAC-SHA256(32)
# Flags: bit0 = compressed, bit1 = encrypted, bit2 = signed(reserved)
# AlgoID: 0x01 = AES-256-CBC, 0x02 = none (plaintext)
# KDF: 0x01 = PBKDF2-HMAC-SHA256
# Requires: PowerShell 7.0+
# Usage: pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\secure-pack.ps1 pack <input> <output.sek> [-Password <str>] [-Compress] [-Iterations <int>]
#        pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\secure-pack.ps1 unpack <input.sek> <output> [-Password <str>] [-Iterations <int>]

#requires -Version 7.0

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('pack', 'unpack')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$InputPath,

    [Parameter(Position = 2)]
    [string]$OutputPath,

    [Parameter()]
    [string]$Password,

    [Parameter()]
    [switch]$Compress,

    [Parameter()]
    [int]$Iterations = 100000
)

$ErrorActionPreference = 'Stop'

# --- Safety limits ---
$maxFileSizeMB = if ($env:SECUREPACK_MAX_FILE_SIZE_MB) { [int]$env:SECUREPACK_MAX_FILE_SIZE_MB } else { 512 }
$script:maxFileSizeBytes = [int64]$maxFileSizeMB * 1MB

function Test-FileSizeLimit {
    param([string]$FilePath)
    if (-not (Test-Path -Path $FilePath)) { return $true }
    $info = Get-Item -Path $FilePath
    if ($info.Length -gt $script:maxFileSizeBytes) {
        Write-Error ("File too large for in-memory processing: {0:N2} MB (limit {1} MB). Set SECUREPACK_MAX_FILE_SIZE_MB to increase." -f ($info.Length / 1MB), $maxFileSizeMB)
        return $false
    }
    return $true
}

# --- Constants ---
$magic = [byte[]](0x53, 0x45, 0x4B, 0x31) # "SEK1"
$version = 0x01
$flagCompressed = 0x01
$flagEncrypted = 0x02
$algoAes256Cbc = 0x01
$algoNone = 0x02
$kdfPbkdf2 = 0x01
$saltLen = 16
$ivLen = 16
$hmacKeyLen = 32
$hmacTagLen = 32

# --- Simple deterministic serialization (CBOR-like subset) ---
function Serialize-Value {
    param($Value)
    $ms = New-Object System.IO.MemoryStream
    $writer = New-Object System.IO.BinaryWriter($ms)

    if ($Value -is [string]) {
        $writer.Write([byte]0x73) # 's' type tag
        $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($Value)
        $writer.Write([int]$bytes.Length)
        $writer.Write($bytes)
    } elseif ($Value -is [int]) {
        $writer.Write([byte]0x69) # 'i' type tag
        $writer.Write([int]$Value)
    } elseif ($Value -is [bool]) {
        $writer.Write([byte]0x62) # 'b' type tag
        $writer.Write([byte](if ($Value) { 1 } else { 0 }))
    } elseif ($Value -is [byte[]]) {
        $writer.Write([byte]0x58) # 'X' type tag (bytes)
        $writer.Write([int]$Value.Length)
        $writer.Write($Value)
    } elseif ($Value -is [System.Collections.IDictionary]) {
        $writer.Write([byte]0x64) # 'd' type tag (dict)
        $writer.Write([int]$Value.Count)
        $keys = @($Value.Keys) | Sort-Object { $_.ToString() }
        foreach ($k in $keys) {
            Serialize-Value -Value $k
            Serialize-Value -Value $Value[$k]
        }
    } elseif ($Value -is [System.Collections.IList]) {
        $writer.Write([byte]0x6C) # 'l' type tag (list)
        $writer.Write([int]$Value.Count)
        foreach ($item in $Value) {
            Serialize-Value -Value $item
        }
    } else {
        $str = $Value.ToString()
        $writer.Write([byte]0x73)
        $bytes = [System.Text.UTF8Encoding]::new($false).GetBytes($str)
        $writer.Write([int]$bytes.Length)
        $writer.Write($bytes)
    }
    $writer.Flush()
    return $ms.ToArray()
}

function Deserialize-Value {
    param([byte[]]$Data, [ref]$Offset)
    $ms = New-Object System.IO.MemoryStream($Data, $false)
    $reader = New-Object System.IO.BinaryReader($ms)
    $ms.Position = $Offset.Value
    $tag = [char]$reader.ReadByte()
    $Offset.Value = $ms.Position
    switch ($tag) {
        's' {
            $len = $reader.ReadInt32()
            $bytes = $reader.ReadBytes($len)
            $Offset.Value = $ms.Position
            return [System.Text.UTF8Encoding]::new($false).GetString($bytes)
        }
        'i' {
            $val = $reader.ReadInt32()
            $Offset.Value = $ms.Position
            return $val
        }
        'b' {
            $val = $reader.ReadByte()
            $Offset.Value = $ms.Position
            return ($val -ne 0)
        }
        'X' {
            $len = $reader.ReadInt32()
            $bytes = $reader.ReadBytes($len)
            $Offset.Value = $ms.Position
            return $bytes
        }
        'd' {
            $count = $reader.ReadInt32()
            $dict = New-Object System.Collections.Hashtable
            0..($count - 1) | ForEach-Object {
                $k = Deserialize-Value -Data $Data -Offset ([ref]$Offset.Value)
                $v = Deserialize-Value -Data $Data -Offset ([ref]$Offset.Value)
                $dict[$k] = $v
            }
            $Offset.Value = $ms.Position
            return $dict
        }
        'l' {
            $count = $reader.ReadInt32()
            $list = New-Object System.Collections.ArrayList
            0..($count - 1) | ForEach-Object {
                $v = Deserialize-Value -Data $Data -Offset ([ref]$Offset.Value)
                $null = $list.Add($v)
            }
            $Offset.Value = $ms.Position
            return $list
        }
        default {
            throw "Unknown type tag: $tag"
        }
    }
}

# --- Key derivation (PBKDF2-HMAC-SHA256) ---
function Derive-Key {
    param([string]$Password, [byte[]]$Salt, [int]$Iterations)
    $kdf = New-Object System.Security.Cryptography.Rfc2898DeriveBytes(
        [System.Text.UTF8Encoding]::new($false).GetBytes($Password),
        $Salt,
        $Iterations,
        [System.Security.Cryptography.HashAlgorithmName]::SHA256)
    $keyMaterial = $kdf.GetBytes(64)
    return @{
        AesKey = $keyMaterial[0..31]
        HmacKey = $keyMaterial[32..63]
    }
}

# --- HMAC-SHA256 ---
function Compute-Hmac {
    param([byte[]]$Key, [byte[]]$Data)
    $hmac = New-Object System.Security.Cryptography.HMACSHA256
    $hmac.Key = $Key
    return $hmac.ComputeHash($Data)
}

# --- AES-256-CBC encrypt ---
function Encrypt-Aes256Cbc {
    param([byte[]]$Key, [byte[]]$Plaintext)
    $iv = [byte[]](0)*$ivLen
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $rng.GetBytes($iv)
    $aes = New-Object System.Security.Cryptography.AesCryptoServiceProvider
    $aes.Key = $Key
    $aes.IV = $iv
    $aes.Mode = [System.Security.Cryptography.CipherMode]::CBC
    $aes.Padding = [System.Security.Cryptography.PaddingMode]::PKCS7
    $encryptor = $aes.CreateEncryptor()
    $ms = New-Object System.IO.MemoryStream
    $cs = New-Object System.Security.Cryptography.CryptoStream($ms, $encryptor, [System.Security.Cryptography.CryptoStreamMode]::Write)
    $cs.Write($Plaintext, 0, $Plaintext.Length)
    $cs.FlushFinalBlock()
    $ciphertext = $ms.ToArray()
    $cs.Close()
    $encryptor.Dispose()
    $aes.Dispose()
    return @{ IV = $iv; Ciphertext = $ciphertext }
}

# --- AES-256-CBC decrypt ---
function Decrypt-Aes256Cbc {
    param([byte[]]$Key, [byte[]]$Iv, [byte[]]$Ciphertext)
    $aes = New-Object System.Security.Cryptography.AesCryptoServiceProvider
    $aes.Key = $Key
    $aes.IV = $Iv
    $aes.Mode = [System.Security.Cryptography.CipherMode]::CBC
    $aes.Padding = [System.Security.Cryptography.PaddingMode]::PKCS7
    $decryptor = $aes.CreateDecryptor()
    $ms = New-Object System.IO.MemoryStream($Ciphertext, $false)
    $cs = New-Object System.Security.Cryptography.CryptoStream($ms, $decryptor, [System.Security.Cryptography.CryptoStreamMode]::Read)
    $out = New-Object System.IO.MemoryStream
    $buffer = [byte[]](0)*4096
    while ($true) {
        $read = $cs.Read($buffer, 0, $buffer.Length)
        if ($read -eq 0) { break }
        $out.Write($buffer, 0, $read)
    }
    $plaintext = $out.ToArray()
    $cs.Close()
    $decryptor.Dispose()
    $aes.Dispose()
    return $plaintext
}

# --- Compression (Deflate) ---
function Compress-Data {
    param([byte[]]$Data)
    $ms = New-Object System.IO.MemoryStream
    $ds = New-Object System.IO.Compression.DeflateStream($ms, [System.IO.Compression.CompressionMode]::Compress, $true)
    $ds.Write($Data, 0, $Data.Length)
    $ds.Close()
    return $ms.ToArray()
}

function Decompress-Data {
    param([byte[]]$Data)
    $ms = New-Object System.IO.MemoryStream($Data, $false)
    $ds = New-Object System.IO.Compression.DeflateStream($ms, [System.IO.Compression.CompressionMode]::Decompress)
    $out = New-Object System.IO.MemoryStream
    $ds.CopyTo($out)
    $ds.Close()
    return $out.ToArray()
}

# --- Pack ---
function Invoke-Pack {
    param([string]$InputFile, [string]$Output, [string]$Password, [switch]$Compress)
    if (-not (Test-FileSizeLimit -FilePath $InputFile)) { return }
    $data = [System.IO.File]::ReadAllBytes($InputFile)

    $flags = 0
    $algo = $algoNone
    $keyInfo = [byte[]]@()
    $keys = $null

    if ($Password) {
        $flags = $flags -bor $flagEncrypted
        $algo = $algoAes256Cbc
        $salt = [byte[]](0)*$saltLen
        $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
        $rng.GetBytes($salt)
        $keys = Derive-Key -Password $Password -Salt $salt -Iterations $Iterations
        $keyInfo = $salt
    }

    $toEncrypt = $data
    if ($Compress) {
        $compressed = Compress-Data -Data $data
        if ($compressed.Length -lt $data.Length) {
            $flags = $flags -bor $flagCompressed
            $toEncrypt = $compressed
        }
    }

    $iv = [byte[]](0)*$ivLen
    $ciphertext = $toEncrypt
    if ($Password) {
        $enc = Encrypt-Aes256Cbc -Key $keys.AesKey -Plaintext $toEncrypt
        $iv = $enc.IV
        $ciphertext = $enc.Ciphertext
    }

    # Build header + body
    $ms = New-Object System.IO.MemoryStream
    $ms.Write($magic, 0, 4)
    $ms.WriteByte([byte]$version)
    $ms.WriteByte([byte]$flags)
    $ms.WriteByte([byte]$algo)
    $ms.WriteByte([byte]$kdfPbkdf2)
    $ms.WriteByte([byte]$keyInfo.Length)
    if ($keyInfo.Length -gt 0) { $ms.Write($keyInfo, 0, $keyInfo.Length) }
    $ms.Write($iv, 0, $ivLen)
    $ms.Write($ciphertext, 0, $ciphertext.Length)

    # HMAC over header + body
    $hmacKey = if ($Password) { $keys.HmacKey } else { [byte[]](0)*$hmacKeyLen }
    $headerAndBody = $ms.ToArray()
    $tag = Compute-Hmac -Key $hmacKey -Data $headerAndBody
    $ms.Write($tag, 0, $tag.Length)
    $ms.Flush()
    $final = $ms.ToArray()

    [System.IO.File]::WriteAllBytes($Output, $final)
    Write-Output ("PACK | {0} -> {1} ({2} bytes, flags=0x{3:X2}, algo=0x{4:X2})" -f $InputFile, $Output, $final.Length, $flags, $algo)
}

# --- Unpack ---
function Invoke-Unpack {
    param([string]$InputFile, [string]$Output, [string]$Password)
    if (-not (Test-FileSizeLimit -FilePath $InputFile)) { return }
    $all = [System.IO.File]::ReadAllBytes($InputFile)
    $ms = New-Object System.IO.MemoryStream($all, $false)
    $reader = New-Object System.IO.BinaryReader($ms)

    # Read magic
    $readMagic = $reader.ReadBytes(4)
    $magicOk = $true
    0..3 | ForEach-Object {
        if ($readMagic[$_] -ne $magic[$_]) { $magicOk = $false }
    }
    if (-not $magicOk) { throw "Invalid magic: not a .sek file" }

    $version = $reader.ReadByte()
    $flags = $reader.ReadByte()
    $algo = $reader.ReadByte()
    $kdf = $reader.ReadByte()
    $keyInfoLen = $reader.ReadByte()
    $keyInfo = $reader.ReadBytes($keyInfoLen)
    $iv = $reader.ReadBytes($ivLen)

    # Remaining bytes: ciphertext + HMAC tag
    $remaining = $all.Length - $ms.Position
    $ciphertextAndTag = $reader.ReadBytes($remaining)
    $ciphertextLen = $remaining - $hmacTagLen
    $ciphertext = $ciphertextAndTag[0..($ciphertextLen - 1)]
    $tag = $ciphertextAndTag[$ciphertextLen..($remaining - 1)]

    # Verify HMAC
    $headerAndBodyLen = $all.Length - $hmacTagLen
    $headerAndBody = New-Object byte[] $headerAndBodyLen
    [System.Array]::Copy($all, 0, $headerAndBody, 0, $headerAndBodyLen)
    $hmacKey = if ($Password) {
        $keys = Derive-Key -Password $Password -Salt $keyInfo -Iterations $Iterations
        $keys.HmacKey
    } else {
        [byte[]](0)*$hmacKeyLen
    }
    $computedTag = Compute-Hmac -Key $hmacKey -Data $headerAndBody
    $match = $true
    0..($hmacTagLen - 1) | ForEach-Object {
        if ($computedTag[$_] -ne $tag[$_]) { $match = $false }
    }
    if (-not $match) {
        throw "HMAC verification failed: file may be tampered or password is incorrect"
    }

    # Decrypt
    $plaintext = $ciphertext
    if ($flags -band $flagEncrypted) {
        if (-not $Password) { throw "Password required to decrypt" }
        $keys = Derive-Key -Password $Password -Salt $keyInfo -Iterations $Iterations
        $plaintext = Decrypt-Aes256Cbc -Key $keys.AesKey -Iv $iv -Ciphertext $ciphertext
    }

    # Decompress
    if ($flags -band $flagCompressed) {
        $plaintext = Decompress-Data -Data $plaintext
    }

    [System.IO.File]::WriteAllBytes($Output, $plaintext)
    Write-Output ("UNPACK | {0} -> {1} ({2} bytes)" -f $InputFile, $Output, $plaintext.Length)
}

switch ($Command) {
    'pack' { Invoke-Pack -InputFile $InputPath -Output $OutputPath -Password $Password -Compress:$Compress -Iterations $Iterations }
    'unpack' { Invoke-Unpack -InputFile $InputPath -Output $OutputPath -Password $Password -Iterations $Iterations }
}
