<#
.SYNOPSIS
    Measure local Ollama candidates for the ZeroClaw agent: latency and whether a
    tool call is actually emitted.

.DESCRIPTION
    Sends one identical tool-call request per model to the Ollama native API and
    reports elapsed time, prompt/completion tokens and whether tool_calls came
    back. Results are advisory: fill the README table with what you observe.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\measure-models.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\measure-models.ps1 -Models qwen3.5:4b,llama3.1:8b
#>
[CmdletBinding()]
param(
    [string] $BaseUrl = "http://127.0.0.1:11434",
    [string[]] $Models = @("qwen3.5:4b", "llama3.1:8b", "qwen2.5:7b-instruct"),
    [int] $NumCtx = 4096,
    [switch] $NoTools,
    [switch] $DumpBodyAll,
    [switch] $NoChild,
    [string] $DumpBody
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

try { Add-Type -AssemblyName System.Net.Http -ErrorAction Stop } catch { }

$Tool = @{
    type     = "function"
    function = @{
        name        = "list_directory"
        description = "List the entries of a directory."
        parameters  = @{
            type       = "object"
            properties = @{ path = @{ type = "string"; description = "Absolute directory path" } }
            required   = @("path")
        }
    }
}

$Prompt = "List the files in the directory C:\Windows\Temp using the provided tool. Do not answer from memory."
if ($NoTools) { $Prompt = "List the files in the directory C:\Windows\Temp in one short sentence." }

if (-not $NoChild) {
    Write-Host "model,tool_call,elapsed_s,completion_tokens,error" -ForegroundColor Cyan
}

# Isolate each model in its own process: some Ollama builds reject a second
# tool-call request made from a long-lived client with a misleading 400.
if ($Models.Count -gt 1 -and -not $NoChild) {
    foreach ($m in $Models) {
        $cargs = @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $PSCommandPath,
            "-Models", $m, "-NoChild", "-BaseUrl", $BaseUrl, "-NumCtx", $NumCtx)
        if ($NoTools) { $cargs += "-NoTools" }
        if ($DumpBody) { $cargs += @("-DumpBody", $DumpBody, "-DumpBodyAll") }
        & powershell @cargs
    }
    return
}

foreach ($m in $Models) {
    $payload = @{
        model    = $m
        messages = @(@{ role = "user"; content = $Prompt })
        stream   = $false
        options  = @{ num_ctx = $NumCtx; temperature = 0 }
    }
    if (-not $NoTools) {
        $payload["tools"] = @($Tool)
    }
    $reqBody = $payload | ConvertTo-Json -Depth 10

    if ($DumpBody) {
        $safe = ($m -replace "[^A-Za-z0-9._-]", "_")
        $target = Join-Path $DumpBody "body-$safe.json"
        [IO.File]::WriteAllText($target, $reqBody, (New-Object Text.UTF8Encoding($false)))
        Write-Host "body written to $target"
        if (-not $DumpBodyAll) { return }
    }

    $sw = [Diagnostics.Stopwatch]::StartNew()
    $tool = "no"
    $tokens = ""
    $err = ""
    try {
        # A fresh client per request: reusing one connection makes some Ollama
        # builds reject the second request with HTTP 400.
        $handler = New-Object System.Net.Http.HttpClientHandler
        $client = New-Object System.Net.Http.HttpClient($handler)
        $client.Timeout = [TimeSpan]::FromSeconds(600)
        $content = New-Object System.Net.Http.StringContent($reqBody, [Text.Encoding]::UTF8, "application/json")
        $resp = $client.PostAsync("$BaseUrl/api/chat", $content).GetAwaiter().GetResult()
        $payload = $resp.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $sw.Stop()
        if ($resp.IsSuccessStatusCode) {
            $res = $payload | ConvertFrom-Json
            if ($res.message.PSObject.Properties.Name -contains "tool_calls") {
                if ($res.message.tool_calls -and @($res.message.tool_calls).Count -gt 0) { $tool = "yes" }
            }
            if ($null -ne $res.eval_count) { $tokens = $res.eval_count }
        } else {
            $err = "$([int]$resp.StatusCode): $payload"
        }
        $client.Dispose()
    } catch {
        $sw.Stop()
        $detail = $_.Exception.Message
        if ($_.Exception.PSObject.Properties.Name -contains "InnerException") {
            if ($_.Exception.InnerException) { $detail = $_.Exception.InnerException.Message }
        }
        $err = ($detail -replace "[`r`n]+", " " -replace ",", ";")
        if ($err.Length -gt 200) { $err = $err.Substring(0, 200) }
    }
    $secs = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    Write-Host ("{0},{1},{2},{3},{4}" -f $m, $tool, $secs, $tokens, $err)
}