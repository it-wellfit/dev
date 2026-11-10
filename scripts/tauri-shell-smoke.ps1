# tauri-shell-smoke.ps1 — Shell + platform-server smoke assertions (ASCII-only).
# Scope (honest): server-side HTTP assertions + shell artifact/config checks.
# GUI launch remains manual (documented in plan 1793034142477 T5).
# Exit code = number of failed assertions (0 = all pass).

param(
    [string]$ServerExe = "$env:LOCALAPPDATA\Temp\dev-target\debug\flaps-server.exe",
    [int]$Port = 8090
)

$Root = Split-Path $PSScriptRoot -Parent
$fail = 0
$spawned = $null

function Assert($name, $cond, $detail) {
    if ($cond) { Write-Output ("PASS  " + $name + "  " + $detail) }
    else { Write-Output ("FAIL  " + $name + "  " + $detail); $script:fail++ }
}

# ---- 1. Artifact checks (no GUI) ----
$shellExe = Join-Path $Root "web\src-tauri\target\debug\flaps-erp-web.exe"
$shellRelease = Join-Path $Root "web\src-tauri\target\release\flaps-erp-web.exe"
Assert "shell-exe-exists" ((Test-Path $shellExe) -or (Test-Path $shellRelease)) "debug=$(Test-Path $shellExe) release=$(Test-Path $shellRelease)"

$conf = Join-Path $Root "web\src-tauri\tauri.conf.json"
$confText = [System.IO.File]::ReadAllText($conf, [System.Text.UTF8Encoding]::new($false))
Assert "conf-no-vite-1420" (-not $confText.Contains("1420")) "no 1420 residue"
Assert "conf-url-8090" ($confText.Contains("http://127.0.0.1:8090/")) "shell URL points at platform server"
Assert "conf-csp-no-script-unsafe-inline" (-not $confText.Contains("script-src") -or -not $confText.Contains("unsafe-inline")) "script-src not relaxed"

$cap = Join-Path $Root "web\src-tauri\capabilities\default.json"
Assert "capabilities-deny-all" (Test-Path $cap) "default.json present"
if (Test-Path $cap) {
    $capJson = [System.IO.File]::ReadAllText($cap, [System.Text.UTF8Encoding]::new($false)) | ConvertFrom-Json
    Assert "capabilities-empty-perms" ($capJson.permissions.Count -eq 0) "zero IPC grants"
}

$nsis = Get-ChildItem (Join-Path $Root "web\src-tauri\target\release\bundle\nsis") -Filter "*.exe" -ErrorAction SilentlyContinue
Assert "nsis-artifact" ($null -ne $nsis) ("bundle=" + $(if ($nsis) { $nsis.Name } else { "not built yet" }))

# ---- 2. Server assertions ----
$listening = $false
try { $c = New-Object Net.Sockets.TcpClient; $c.Connect("127.0.0.1", $Port); $c.Close(); $listening = $true } catch {}

if (-not $listening) {
    Assert "server-exe-exists" (Test-Path $ServerExe) $ServerExe
    if (Test-Path $ServerExe) {
        $env:FLAPS_PORT = "$Port"
        $spawned = Start-Process -FilePath $ServerExe -WorkingDirectory $Root -PassThru -WindowStyle Hidden
    }
}

$base = "http://127.0.0.1:$Port"
$ok = $false
for ($i = 0; $i -lt 60; $i++) {
    try { $r = Invoke-WebRequest -Uri "$base/login" -UseBasicParsing -TimeoutSec 3; if ($r.StatusCode -eq 200) { $ok = $true; break } } catch {}
    Start-Sleep -Milliseconds 500
}
Assert "login-200" $ok "GET /login responded 200 within 30s"

if ($ok) {
    $csp = $r.Headers["Content-Security-Policy"]
    Assert "csp-default-src-none" ($csp -and $csp.Contains("default-src 'none'")) "$csp"
    Assert "nosniff" ($r.Headers["X-Content-Type-Options"] -eq "nosniff") "X-Content-Type-Options"
    Assert "zero-js-login" (-not $r.Content.Contains("<script")) ("script tags = " + ([regex]::Matches($r.Content, "<script")).Count)

    try { $h = Invoke-WebRequest -Uri "$base/api/health" -UseBasicParsing -TimeoutSec 3; $hn = ($h.StatusCode -eq 200) } catch { $hn = $false }
    Assert "health-200" $hn "GET /api/health"
}

# ---- 3. Cleanup (only what we spawned) ----
if ($spawned) { Stop-Process -Id $spawned.Id -Force -ErrorAction SilentlyContinue; Write-Output "INFO  spawned server stopped (pid $($spawned.Id))" }

Write-Output ("SUMMARY  failures=" + $fail)
exit $fail