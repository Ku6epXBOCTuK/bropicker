$ErrorActionPreference = "Stop"

cargo build
if ($LASTEXITCODE -ne 0) { exit 1 }

New-Item -ItemType Directory -Force -Path docs\img | Out-Null
$exe = Join-Path $PWD "target\debug\bropicker.exe"

$proc = Start-Process -FilePath $exe -WorkingDirectory $PWD -PassThru
Start-Sleep -Milliseconds 1800
powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\shot.ps1" -ProcId $proc.Id -Out docs\img\screenshot-dark.png -Card
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
if ($LASTEXITCODE -ne 0) { exit 1 }

$env:BP_THEME = "light"
$proc = Start-Process -FilePath $exe -WorkingDirectory $PWD -PassThru
Start-Sleep -Milliseconds 1800
powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\shot.ps1" -ProcId $proc.Id -Out docs\img\screenshot-light.png -Card
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Remove-Item Env:\BP_THEME -ErrorAction SilentlyContinue
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "screenshots updated: docs\img\screenshot-{dark,light}.png"
