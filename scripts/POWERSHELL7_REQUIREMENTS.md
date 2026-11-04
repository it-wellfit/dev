# PowerShell 7+ 環境需求與安裝指引

## 為什麼需要 PowerShell 7+？

本專案的編碼與資料格式處理腳本（`scripts/encoding-tools.ps1`、`scripts/secure-pack.ps1`、`scripts/format-manager.ps1`）已優化為 **PowerShell 7 LTS 以上** 運行。

PowerShell 5.1（Windows 內建舊版）存在以下限制，已在 PowerShell 7+ 解決：

| 限制 | PowerShell 5.1 | PowerShell 7 LTS+ |
|------|---------------|-------------------|
| 鏈式命令 `&&` | 不支援 | **支援** |
| 三元運算子 `?:` | 不支援 | **支援** |
| `ForEach-Object -Parallel` | 不支援 | **支援**（大幅提升批次處理效能） |
| 管線效能 | 較慢 | **更快** |
| 跨平台 | 僅 Windows | Windows / Linux / macOS |
| .NET 執行階段 | .NET Framework | .NET 6+ / .NET 8+ LTS |

## 檢查目前版本

```powershell
# 在 PowerShell 中執行
$PSVersionTable.PSVersion
```

若 `Major` 小於 7，請安裝 PowerShell 7 LTS。

## 安裝 PowerShell 7 LTS（Windows）

### 方法 A：winget（推薦，最簡單）

```powershell
winget install Microsoft.PowerShell --source winget
```

### 方法 B：MSI 直接下載

前往：https://github.com/PowerShell/PowerShell/releases

下載 `PowerShell-7.4.x-win-x64.msi`（LTS 版本，建議 7.4.x）並安裝。

### 方法 C：Scoop

```powershell
scoop install pwsh
```

### 方法 D：Chocolatey

```powershell
choco install powershell
```

## 安裝後確認

安裝完成後，在新的 PowerShell 視窗中執行：

```powershell
$PSVersionTable.PSVersion
```

應顯示 `Major >= 7`（例如 `7.4.6`）。

## 如何使用 PowerShell 7 運行本專案腳本

```powershell
# 使用 pwsh (PowerShell 7+) 執行
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\format-manager.ps1 list
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\encoding-tools.ps1 validate .
pwsh -NoProfile -ExecutionPolicy Bypass -File .\scripts\secure-pack.ps1 pack input.bin output.sek -Password "mypass"
```

## 腳本中的版本保護

所有腳本已加入版本檢查，若偵測到 PowerShell 版本低於 7.0，將輸出警告並拒絕運行，避免在 5.1 下產生不可預期的錯誤。

```powershell
# 範例：腳本開頭會檢查
if ($PSVersionTable.PSVersion.Major -lt 7) {
    Write-Error "本腳本需要 PowerShell 7.0 以上，目前版本：$($PSVersionTable.PSVersion)"
    exit 1
}
```

## 參考資料

- PowerShell 官方：https://powershell.microsoft.com/
- PowerShell GitHub 發布：https://github.com/PowerShell/PowerShell/releases
- LTS 版本說明：https://learn.microsoft.com/zh-tw/powershell/scripting/install/installing-powershell
