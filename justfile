set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

app := "mikrotik-tui"

# List available recipes
default:
    @just --list

# Format check, Clippy, and workspace tests
check: fmt clippy test

# Fail if rustfmt would change files
fmt:
    cargo fmt --all -- --check

# Apply rustfmt
fmt-fix:
    cargo fmt --all

# Clippy with warnings as errors
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

alias lint := clippy

# Workspace tests
test:
    cargo test --workspace

# Run the TUI
run:
    cargo run -p {{app}}

[unix]
build:
    cargo build -p {{app}} --release
    mkdir -p bin
    cp target/release/{{app}} bin/{{app}}

[windows]
build:
    cargo build -p {{app}} --release
    New-Item -ItemType Directory -Force -Path bin | Out-Null
    Copy-Item "target/release/{{app}}.exe" "bin/{{app}}.exe"

[unix]
release:
    cargo build -p {{app}} --release
    mkdir -p dist
    cp target/release/{{app}} dist/{{app}}

[windows]
release:
    cargo build -p {{app}} --release
    New-Item -ItemType Directory -Force -Path dist | Out-Null
    Copy-Item "target/release/{{app}}.exe" "dist/{{app}}.exe"

[unix]
clean:
    cargo clean
    rm -rf bin dist

[windows]
clean:
    cargo clean
    if (Test-Path bin) { Remove-Item -Recurse -Force bin }
    if (Test-Path dist) { Remove-Item -Recurse -Force dist }
