param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$installDir = Join-Path $env:LOCALAPPDATA "bropicker"
$classes = "HKCU:\Software\Classes\bropicker"
$caps = "HKCU:\Software\bropicker-Capabilities"
$regApps = "HKCU:\Software\RegisteredApplications"

if ($Uninstall) {
    Remove-Item -Recurse -Force $classes -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $caps -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path $regApps -Name "bropicker" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $installDir -ErrorAction SilentlyContinue
    Write-Host "bropicker unregistered and removed from $installDir"
    exit 0
}

$src = Join-Path $PSScriptRoot "..\target\release\bropicker.exe"
if (-not (Test-Path $src)) {
    Write-Error "target\release\bropicker.exe not found. Build first: cargo build --release"
    exit 1
}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item $src (Join-Path $installDir "bropicker.exe") -Force
Copy-Item (Join-Path $PSScriptRoot "..\logos") (Join-Path $installDir "logos") -Recurse -Force
Copy-Item (Join-Path $PSScriptRoot "..\icons") (Join-Path $installDir "icons") -Recurse -Force

$exe = Join-Path $installDir "bropicker.exe"
Write-Host "Installed to: $exe"

New-Item -Path "$classes\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path $classes -Name "(default)" -Value "bropicker"
Set-ItemProperty -Path $classes -Name "URL Protocol" -Value ""
Set-ItemProperty -Path "$classes\shell\open\command" -Name "(default)" -Value "`"$exe`" `"%1`""

New-Item -Path $caps -Force | Out-Null
Set-ItemProperty -Path $caps -Name "ApplicationName" -Value "bropicker"
Set-ItemProperty -Path $caps -Name "ApplicationDescription" -Value "Lightweight link picker: choose which browser opens the URL"

New-Item -Path "$caps\URLAssociations" -Force | Out-Null
Set-ItemProperty -Path "$caps\URLAssociations" -Name "https" -Value "bropicker"
Set-ItemProperty -Path "$caps\URLAssociations" -Name "http" -Value "bropicker"

New-Item -Path $regApps -Force | Out-Null
Set-ItemProperty -Path $regApps -Name "bropicker" -Value "Software\bropicker-Capabilities"

Write-Host "Registered. Now select bropicker for HTTP/HTTPS in the opened Settings window."
Start-Process "ms-settings:defaultapps"
