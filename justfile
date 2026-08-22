set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

app := "mikrotik-tui"

# List available recipes
default:
    @just --list

# GitHub Actions `just check`: fail if unformatted, then Clippy, then tests.
# Does not apply rustfmt. Run `just prepush` locally so Clippy sees the
# formatted tree (fmt can introduce lints). OS-gated tests still differ.
check: fmt clippy test

# Local gate before push: format, then the same Clippy+tests as CI.
prepush: fmt-fix clippy test

# CI job except Docker (`docker build --build-arg VERSION=ci .`)
ci: check build

# Fail if rustfmt would change files
fmt:
    cargo fmt --all -- --check

# Apply rustfmt
fmt-fix:
    cargo fmt --all

# Clippy with warnings as errors (lib, bins, tests, examples)
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
    rm -rf bin dist website/dist website/node_modules website/.astro

[windows]
clean:
    cargo clean
    if (Test-Path bin) { Remove-Item -Recurse -Force bin }
    if (Test-Path dist) { Remove-Item -Recurse -Force dist }
    if (Test-Path website/dist) { Remove-Item -Recurse -Force website/dist }
    if (Test-Path website/node_modules) { Remove-Item -Recurse -Force website/node_modules }
    if (Test-Path website/.astro) { Remove-Item -Recurse -Force website/.astro }

# Build the end-user landing page (static Astro site → website/dist)
site:
    npm --prefix website install
    npm --prefix website run build

# Serve the landing page locally (http://localhost:4321/mikrotik-tui/)
site-dev:
    npm --prefix website install
    npm --prefix website run dev
