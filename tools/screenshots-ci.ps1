try {
    & "$PSScriptRoot\screenshots.ps1"
} catch {
    Write-Host "screenshots failed (headless runner?), continuing: $_"
}
exit 0
