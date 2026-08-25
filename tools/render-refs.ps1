param(
    [string]$OutDir = "tools/out"
)

$ErrorActionPreference = "Stop"

$candidates = @(
    "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe",
    "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe",
    "${env:ProgramFiles}\Google\Chrome\Application\chrome.exe",
    "${env:ProgramFiles(x86)}\Google\Chrome\Application\chrome.exe"
)
$browser = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $browser) {
    Write-Error "msedge.exe / chrome.exe not found in standard locations"
    exit 1
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$refs = @(
    @{ Html = "refs/dark.html";  Png = "ref-dark.png" },
    @{ Html = "refs/light.html"; Png = "ref-light.png" }
)

foreach ($ref in $refs) {
    $outAbs = [System.IO.Path]::GetFullPath((Join-Path $OutDir $ref.Png))
    $uri = ([System.Uri]([System.IO.Path]::GetFullPath($ref.Html))).AbsoluteUri

    foreach ($mode in @("--headless=new", "--headless")) {
        $args = @(
            $mode,
            "--disable-gpu",
            "--hide-scrollbars",
            "--no-first-run",
            "--user-data-dir=$env:TEMP\bp-headless-profile",
            "--window-size=488,800",
            "--screenshot=`"$outAbs`"",
            $uri
        )
        Start-Process -FilePath $browser -ArgumentList $args -Wait | Out-Null
        if (Test-Path $outAbs) { break }
    }

    if (Test-Path $outAbs) {
        Write-Host "OK $($ref.Png)"
    } else {
        Write-Error "failed to render $($ref.Html)"
        exit 1
    }
}
