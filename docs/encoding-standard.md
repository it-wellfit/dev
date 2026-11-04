# Encoding Standard — UTF-8 First, All Languages, Zero Mojibake

## Scope
Applies to all text/source/data files in the workspace: `ekpi-platform/`, `Wellfit/`, `SyncMesh/`, and root-level docs/configs.

## Core Rule
| Property | Value | Rationale |
|----------|-------|-----------|
| Default charset | `utf-8` | Unicode superset; covers Traditional Chinese (Big5 superset) and Simplified Chinese (GB2312/GBK/GB18030 superset) |
| Line ending | `lf` | Cross-platform consistency, Git-native |
| BOM | `without BOM` | Avoids BOM-related tooling issues on Windows/Linux/macOS |
| Trailing newline | `insert` | POSIX-friendly |
| Trailing whitespace | `trim` | Prevents diff noise |

## Binary / Positional / Base-N Files (never normalize)
Base64, Base32, Base58, Base85, Hex, Octal, raw binary, images, audio, video, fonts, archives, executables, lock files are marked `binary` in `.editorconfig` and `.gitattributes`. They are never transcoded or line-ending normalized.

## Supported Text Extensions
md, rs, toml, json, yml, yaml, ps1, sh, html, css, ts, tsx, js, jsx, py, go, java, c, cpp, h, cs, rb, php, sql, xml, svg, txt, ini, cfg, conf, env, env.example, po, pot, properties, resx, strings, stringsdict, proto, thrift, Cargo.toml.

## Tooling
`scripts/encoding-tools.ps1` — PowerShell toolkit:

- `detect <path>` — Detect encoding per file (BOM-aware, falls back to Big5/GBK/GB18030/Shift-JIS/UTF-8/ISO-8859-1/Windows-1252)
- `convert <path> -FromEncoding <enc> -ToEncoding <enc> [-Force] [-NoBom] [-OutputPath <dir>]` — Transcode text files between any encodings
- `validate <path>` — Validate all text files are decodable and free of mojibake (replacement char, PUA, surrogates)
- `base64-encode <path> [-OutputPath]` — Encode binary to Base64 text
- `base64-decode <path> [-OutputPath]` — Decode Base64 text to binary
- `hex-encode <path> [-OutputPath]` — Encode binary to Hex text
- `hex-decode <path> [-OutputPath]` — Decode Hex text to binary
- `batch-validate <path>` — Recursive validation with exit code 1 on failure

### Examples
```powershell
# Detect all file encodings under ekpi-platform
.\scripts\encoding-tools.ps1 detect ekpi-platform

# Convert Big5 file to UTF-8 without BOM
.\scripts\encoding-tools.ps1 convert docs\legacy.md -FromEncoding big5 -ToEncoding utf-8 -NoBom -Force

# Validate entire workspace
.\scripts\encoding-tools.ps1 batch-validate .

# Encode binary to Base64
.\scripts\encoding-tools.ps1 base64-encode assets\icon.png -OutputPath assets\icon.base64
```

## CJK Compatibility
UTF-8 natively encodes:
- Traditional Chinese: U+4E00–U+9FFF (CJK Unified Ideographs), U+3400–U+4DBF (Extension A), etc.
- Simplified Chinese: same code points; glyph selection is font-level, not encoding-level
- No mojibake occurs when source and target are both valid UTF-8.

## Verification
Run `.\scripts\encoding-tools.ps1 batch-validate .` before any commit. CI gate should enforce zero mojibake and UTF-8-only text files.

## Format Manager — Unified Conversion Entry Point (additive only — nothing deleted)

`scripts/format-manager.ps1` is the single entry point for all format management and conversion. Every existing format is preserved; new formats are added without removing any.

### Supported formats (14 total, all preserved)

| Format | Category | Expansion | Description |
|--------|----------|-----------|-------------|
| utf-8 | text | n/a | UTF-8 without BOM — Traditional/Simplified Chinese compatible |
| big5 | text | n/a | Traditional Chinese (Taiwan/HK legacy) |
| gb2312 | text | n/a | Simplified Chinese (Mainland legacy) |
| gbk | text | n/a | Simplified Chinese (GBK extended) |
| gb18030 | text | n/a | Simplified Chinese (GB18030 full) |
| shift_jis | text | n/a | Japanese Shift-JIS |
| euc-jp | text | n/a | Japanese EUC-JP |
| windows-1252 | text | n/a | Western Windows-1252 |
| iso-8859-1 | text | n/a | Western ISO-8859-1 |
| base64 | binary-to-text | 1.33x | Base64 — universal, **preserved** |
| hex | binary-to-text | 2.00x | Hexadecimal — debug/human-readable, **preserved** |
| base91 | binary-to-text | ~1.23x | basE91 — more compact than Base64, **preserved** |
| z85 | binary-to-text | 1.25x | ZeroMQ Z85 — string-safe, **preserved** |
| sek | secure-kit | n/a | .sek — AES-256-CBC + PBKDF2 + HMAC-SHA256 + optional compression, **preserved** |

### Commands

```powershell
# List all supported formats (additive — nothing deleted)
.\scripts\format-manager.ps1 list

# Show conversion matrix
.\scripts\format-manager.ps1 matrix

# Show file format info (BOM, binary indicator, UTF-8 decodability)
.\scripts\format-manager.ps1 info <path>

# Validate files for mojibake and decodability
.\scripts\format-manager.ps1 validate <path> [-Recursive]

# Convert between any supported formats
.\scripts\format-manager.ps1 convert <path> -FromFormat <from> -ToFormat <to> [-Password <str>] [-Compress] [-Force] [-OutputPath <dir>]
```

### Conversion examples

```powershell
# Text encoding conversion (all text <-> all text via UTF-8 pivot)
.\scripts\format-manager.ps1 convert docs\legacy.md -FromFormat big5 -ToFormat utf-8 -NoBom -Force

# Binary to Base64 (preserved)
.\scripts\format-manager.ps1 convert assets\icon.png -FromFormat raw-binary -ToFormat base64 -OutputPath assets\icon.base64

# Binary to Base91 (more compact than Base64, preserved)
.\scripts\format-manager.ps1 convert assets\icon.png -FromFormat raw-binary -ToFormat base91 -OutputPath assets\icon.base91

# Binary to Z85 (string-safe, preserved)
.\scripts\format-manager.ps1 convert assets\icon.png -FromFormat raw-binary -ToFormat z85 -OutputPath assets\icon.z85

# Binary to .sek (encrypted, compressed, integrity-verified, preserved)
.\scripts\format-manager.ps1 convert data\secret.bin -FromFormat raw-binary -ToFormat sek -Password "mypass" -Compress -OutputPath data\secret.sek

# .sek back to raw binary
.\scripts\format-manager.ps1 convert data\secret.sek -FromFormat sek -ToFormat raw-binary -Password "mypass" -OutputPath data\secret_restored.bin
```

### No-format-deletion guarantee

All formats present before this update remain available:
- `base64`, `hex` — unchanged, still fully supported in `encoding-tools.ps1`
- `base91`, `z85` — added, never replace base64/hex
- `.sek` — added, standalone secure kit format
- All text encodings (utf-8, big5, gb2312, gbk, gb18030, shift_jis, euc-jp, windows-1252, iso-8859-1) — unchanged

No format is ever removed. New capability is strictly additive.

