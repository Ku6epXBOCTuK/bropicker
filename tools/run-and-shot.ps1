param(
    [string]$Out = "tools/out/app.png",
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

if (-not $NoBuild) {
    cargo build
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

$exe = Join-Path $PWD "target\debug\bropicker.exe"
if (-not (Test-Path $exe)) {
    Write-Error "bropicker.exe not found, build first"
    exit 1
}

$proc = Start-Process -FilePath $exe -WorkingDirectory $PWD -PassThru
Start-Sleep -Milliseconds 1800

& "$PSScriptRoot\shot.ps1" -ProcId $proc.Id -Out $Out
$shotOk = ($LASTEXITCODE -eq 0)

Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue

if (-not $shotOk) { exit 1 }
