$ErrorActionPreference = "Stop"

cargo fmt
if ($LASTEXITCODE -ne 0) { exit 1 }

cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

cargo test
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "check OK"
