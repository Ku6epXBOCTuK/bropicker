set shell := ["powershell.exe", "-NoProfile", "-Command"]

build:
    cargo build --release

run:
    cargo run

test:
    cargo nextest run --workspace

check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    cargo nextest run --workspace
    cargo test --workspace --doc

register: build
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/register.ps1

unregister:
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/register.ps1 -Uninstall
