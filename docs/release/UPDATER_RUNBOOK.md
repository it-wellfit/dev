# 更新鏈 Runbook（W10／計畫 1793033568961）

> 狀態：v1.0（2026-11-10）｜**所有產物現為 unsigned (test-only)**——購證前禁止生產分發聲明。

## 1. 金鑰保管（minisign／Ed25519）

| 項目 | 位置 | 紀律 |
|---|---|---|
| 私鑰 | `%USERPROFILE%\.tauri\flaps-erp\flaps-erp.key` | **repo 外**；遺失＝無法向已裝機推送更新（公鑰燒錄於已分發安裝包） |
| 公鑰 | 同目錄 `flaps-erp.key.pub`（內容已錄 `tauri.conf.json` plugins.updater.pubkey） | 可公開；變更＝全用戶重裝 |
| 密碼 | 同目錄 custody 記錄（本 runbook 不錄值） | **生產前必須輪換**（現值語義：flaps-updater-2026-rotate-before-production） |

備份：私鑰＋密碼離線備份至組織密碼管理器；本機遺失即輪換。

## 2. 構建與簽章

```powershell
# 1) 構建（產出 -setup.exe 與 .sig；createUpdaterArtifacts=true）
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = "$env:USERPROFILE\.tauri\flaps-erp\flaps-erp.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<custody 密碼>"
cargo tauri build   # 於 web/src-tauri

# 2) 產物三件：target/release/bundle/nsis/*.exe、*.exe.sig、*.json（updater metadata）
# 3) 簽章（Authenticode）：購證後以 bundle.windows.signCommand 接 Azure Trusted Signing；
#    未購證前所有產物標注 unsigned (test-only)，禁止對外分發。
```

## 3. Feed（latest.json）

endpoint（tauri.conf）：`https://updates.flaps.internal/erp/latest.json`——**內部 HTTPS 主機尚未建置**（未竟項）。格式：

```json
{ "version": "0.3.0", "notes": "...", "pub_date": "2026-11-10T00:00:00Z",
  "platforms": { "windows-x86_64": { "signature": "<.sig 內容>", "url": "https://.../Flaps-ERP_0.3.0_x64-setup.exe" } } }
```

驗證為硬門檻：簽章不符即拒裝（無停用開關）。

## 4. 端到端驗證清單（未竟——下一切片）

1. `cargo tauri build` 全綠 → 三件產物存在。
2. 安裝 v0.2 → 舊版啟動 → feed 指向 v0.3 → passive 更新 → 重啟後版本正確。
3. 負向用例：改 latest.json 一字符 → 更新必敗。
4. 安裝版 `/console/index.html` 200（NSIS resources 已捆綁 knowledge/console）。
5. 簽章（Authenticode）後 SmartScreen 無黃條（購證後）。
