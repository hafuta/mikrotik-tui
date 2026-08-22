# MikroTik TUI

[![Rust](https://img.shields.io/badge/Rust-1.98+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/hafuta/mikrotik-tui/ci.yml?branch=main&label=CI)](https://github.com/hafuta/mikrotik-tui/actions/workflows/ci.yml)

> [!CAUTION]
> **Early preview.** This project is a very initial release. It has been tested
> only against the latest RouterOS long-term version, remains limited in what
> it can do, and will still contain bugs. Treat it as experimental: do not
> rely on it for production operations.

<p align="center">
  <img src="assets/banner.png" alt="MikroTik TUI" width="520">
</p>

MikroTik TUI is a keyboard-first terminal client for MikroTik RouterOS. It
connects over HTTPS REST and presents live operational state - interfaces,
addressing, DHCP, firewall, hardware, and logs - so any RouterOS device can be
inspected without leaving the terminal. Interface screens can create, edit,
and run per-row actions (enable/disable, copy, remove, torch, reset counters)
through confirmation dialogs and a sectioned properties sheet. Other resource
groups remain read-only until they reuse the same action catalog.

## Features

- Live dashboard for CPU, memory, WAN throughput, and firewall activity
- Browse live RouterOS operational state; coverage is still expanding
- Search, sorting, detail inspector, application log console, command palette, and in-app keyboard help
- Hide unused sidebar menus (categories or screens) and reveal them later to restore
- HTTPS with a custom CA or a pinned device certificate (the pin takes
  precedence when both are set)
- One saved connection profile and machine-local credentials
- Structured, redacted application logs on disk and in the in-app console

RouterOS v7 with `www-ssl` and REST access is required. Use a dedicated,
least-privileged RouterOS account. Interface screens require write permission
for the menus you edit; other views still work with read-only REST access.

## Crates

| Crate | Role |
|-------|------|
| `mtui-core` | Resource catalog, shared types, **pluggable themes** |
| `mtui-routeros` | HTTPS REST client (TLS pin / custom CA; GET plus mutations) |
| `mtui-config` | Profiles, credentials, env overrides, file logging |
| `mtui-ui` | Pure widgets/layouts (no networking); styles from theme palette |
| `mtui-app` | State machine, polling, orchestration |
| `mikrotik-tui` | Binary entrypoint |

Semantic colors live in `mtui_core::Palette`. The built-in look is
`DefaultTheme` (`id = "default"`), registered in `ThemeRegistry`. UI code
never hard-codes product hex values - it uses `Styles::from_palette`.

Profiles may store `preferences.theme = "default"` (see
`mtui_config::THEME_PREFERENCE_KEY`) so additional themes can be selected later
without schema changes. Hidden sidebar items persist as
`preferences.hidden_nav` (`mtui_config::HIDDEN_NAV_PREFERENCE_KEY`), a
comma-separated list of navigation ids.

## Tooling

Install [rustup](https://rustup.rs/). Entering the repo installs the compiler
from `rust-toolchain.toml` (currently 1.98.0) with `rustfmt` and `clippy`.
`Cargo.toml` `rust-version` matches that channel so Cargo and rustup agree.

Task recipes live in `justfile` ([just](https://github.com/casey/just)). Install
it with `cargo install just`, or your package manager (`winget install Casey.Just`,
`brew install just`, `scoop install just`).

## Run

```sh
just run
```

```powershell
just run
```

Equivalent: `cargo run -p mikrotik-tui`.

Flags: `--version`, `--no-alt-screen`.

On first launch, enter the RouterOS HTTPS URL, username and password. For a
self-signed certificate, the app presents a SHA-256 fingerprint and asks you
to trust it before credentials are sent. An approved fingerprint is treated as
the router identity even when a local certificate is expired or lacks an IP
subject; any certificate replacement is blocked until its new fingerprint is
reviewed.

Saved files land under the platform config and state directories
(`~/.config/mikrotik-tui` on Linux, `%APPDATA%\mikrotik-tui` on Windows,
`~/Library/Application Support/mikrotik-tui` on macOS). `XDG_CONFIG_HOME` and
`XDG_STATE_HOME` override those locations on every platform. The credential
file is owner-only on Unix and is not encrypted anywhere; use a protected
machine or `MIKROTIK_TUI_PASSWORD_FILE`.

### Environment overrides

- `MIKROTIK_TUI_URL`
- `MIKROTIK_TUI_USERNAME`
- `MIKROTIK_TUI_PASSWORD`
- `MIKROTIK_TUI_PASSWORD_FILE` (preferred for containers)
- `MIKROTIK_TUI_CA_FILE`
- `MIKROTIK_TUI_CERT_FINGERPRINT`
- `MIKROTIK_TUI_LOG` (tracing filter; default `info,mtui_app=trace,mtui_routeros=info,mtui_config=info`)

## Keyboard

`↑/↓` or `j/k` moves, `←/→` or `h/l` pans table columns, `tab` / `shift+tab`
cycle panes, `enter` expands a nav category (accordion; first screen opens)
or opens the selected item. On an Interfaces table, `enter` edits the selected
row. `/` filters the table, `s`
cycles sort (not on Logs), `r` refreshes, `g`/`G` or Home/End jump, `pgup`/
`pgdn` and `ctrl+u`/`ctrl+d` page, `e` edits, `n` adds, `d` enables or disables,
`c` copies, `x` removes, `z` resets counters, `t` opens torch, `a` opens the
action menu, `ctrl+s` saves a properties sheet, `ctrl+k` opens the command palette,
`ctrl+l` logs out, `` ` `` toggles the application log console, `-` hides the
selected sidebar item after confirmation (or restores it when hidden menus are
showing), `.` shows hidden menus (marked with `×`
and strikethrough) so they can be restored, `?` opens help, `esc` closes overlays or clears the
filter, and `q` quits. Logging out removes the saved local profile and
credential; quitting keeps them for automatic reconnection.

The log console is hidden by default and docks to the lower quarter of the
screen. Focus it with `` ` `` or `tab`, press `f` for fullscreen (header and
footer stay), `/` for case-insensitive search, `enter` to expand extra fields
(one row at a time; moving `j`/`k` keeps the expansion), `pgup`/`pgdn` to page,
and `c` to copy the focused record.

The Logs page keeps a bounded 500-event local stream with stable deduplication
and continues polling after a failed fetch. `space` pauses the view while
ingestion continues, `f` returns to the newest event, `e` cycles severity, and
`c` clears only the local buffer (it never deletes logs from RouterOS). Moving
upward detaches from the tail and shows an unread-event counter.

## Build and test

```sh
just prepush
just check
just build
just release
docker build -t mikrotik-tui .
```

Use `just prepush` before pushing: it runs `cargo fmt --all`, then Clippy with
`-D warnings`, then `cargo test --workspace`. That is the local gate. GitHub
Actions runs `just check` (`fmt --check`, same Clippy, same tests), then
`just build`, then `docker build`. `just ci` is check plus build without
Docker.

Do not run Clippy and then format afterward: rustfmt can wrap code in a way
that Clippy then rejects. `just check` does not reformat, so unformatted or
half-formatted trees can look clean locally and fail in CI.

Clippy and tests still follow the host OS. `cfg(windows)` / `cfg(unix)` code
that is unused on the other platform will not fail locally; keep platform
tests in their own `cfg` modules so imports are not shared with an empty
Linux or Windows test module.

`just build` and `just release` compile `-p mikrotik-tui --release` and copy
the binary to `bin/` and `dist/` respectively. `just --list` shows the rest
(`fmt-fix`, `clippy`, `test`, `run`, `clean`).

Pushing a `v*` tag runs GitHub Actions `Release`. It publishes unsigned
archives for Linux (amd64, arm64), macOS (arm64, amd64), and Windows (amd64),
plus `checksums.txt`. Mac Gatekeeper and Windows SmartScreen may warn until
the binaries are signed.

## License

This project is licensed under the [MIT License](LICENSE).
