# MikroTik TUI

> **Early preview.** This project is a very initial release. It has been tested
> only against the latest RouterOS long-term version, remains limited in what
> it can do, and will still contain bugs. Treat it as experimental: do not
> rely on it for production operations.

MikroTik TUI is a keyboard-first terminal client for MikroTik RouterOS. It
connects over HTTPS REST and presents live operational state—interfaces,
addressing, DHCP, firewall, hardware, and logs—so any RouterOS device can be
inspected without leaving the terminal. The client is read-only: it does not
change device configuration.

## Features

- Live dashboard for CPU, memory, WAN throughput, and firewall activity
- Interface, interface-list, Ethernet, PPP session, and PPPoE client views
- Bridge, port, and VLAN inventory
- IP addresses, ARP, DHCP servers, networks, leases, and firewall filter rules
- Users, RouterBOARD, NTP client, clock, and RouterOS log streaming
- Search, sorting, detail inspector, command palette, and in-app keyboard help
- HTTPS with a custom CA or a pinned device certificate (the pin takes
  precedence when both are set)
- One saved connection profile and machine-local credentials
- Structured, redacted application logs on disk

RouterOS v7 with `www-ssl` and REST access is required. Use a dedicated,
least-privileged RouterOS account with read and REST API permissions.

## Crates

| Crate | Role |
|-------|------|
| `mtui-core` | Resource catalog, shared types, **pluggable themes** |
| `mtui-routeros` | Read-only HTTPS REST client (TLS pin / custom CA) |
| `mtui-config` | Profiles, credentials, env overrides, file logging |
| `mtui-ui` | Pure widgets/layouts (no networking); styles from theme palette |
| `mtui-app` | State machine, polling, orchestration |
| `mikrotik-tui` | Binary entrypoint |

Semantic colors live in `mtui_core::Palette`. The built-in look is
`DefaultTheme` (`id = "default"`), registered in `ThemeRegistry`. UI code
never hard-codes product hex values — it uses `Styles::from_palette`.

Profiles may store `preferences.theme = "default"` (see
`mtui_config::THEME_PREFERENCE_KEY`) so additional themes can be selected later
without schema changes.

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
- `MIKROTIK_TUI_LOG` (tracing filter for the file log; default `info`)

## Keyboard

`↑/↓` or `j/k` moves, `←/→` or `h/l` pans table columns, `tab` / `shift+tab`
cycle panes, `enter` opens the selected nav item, `/` filters the table, `s`
cycles sort (not on Logs), `r` refreshes, `g`/`G` or Home/End jump, `pgup`/
`pgdn` and `ctrl+u`/`ctrl+d` page, `ctrl+k` opens the command palette,
`ctrl+l` logs out, `?` opens help, `esc` closes overlays or clears the
filter, and `q` quits. Logging out removes the saved local profile and
credential; quitting keeps them for automatic reconnection.

The Logs page keeps a bounded 500-event local stream with stable deduplication
and continues polling after a failed fetch. `space` pauses the view while
ingestion continues, `f` returns to the newest event, `e` cycles severity, and
`c` clears only the local buffer (it never deletes logs from RouterOS). Moving
upward detaches from the tail and shows an unread-event counter.

## Build and test

```sh
just check
just build
just release
docker build -t mikrotik-tui .
```

`just check` runs `cargo fmt --all -- --check`, Clippy with `-D warnings`, and
`cargo test --workspace`. `just build` and `just release` both compile
`-p mikrotik-tui --release` and copy the binary to `bin/` and `dist/`
respectively. `just --list` shows the rest (`fmt-fix`, `clippy`, `test`,
`run`, `clean`).

## Docker

The TUI needs an interactive terminal and a path to the router. On Linux,
`--network host` reaches LAN devices as the host would:

```sh
docker run --rm -it --network host \
  -v mikrotik-tui-data:/data \
  mikrotik-tui
```

On Docker Desktop (Windows/macOS), host networking is not equivalent; publish
or attach the container so it can still reach the router's HTTPS port.

The image sets `XDG_CONFIG_HOME=/data/config` and `XDG_STATE_HOME=/data/state`.
Mount `/data` to retain the profile, certificate pin, credential, and logs.

## Architecture

`mtui-app` owns application state. Network calls run as tokio worker tasks;
stale replies are dropped when the screen or request generation changes.
Feature screens are data descriptors over shared table and inspector widgets
in `mtui-ui`. Theme styles, profile storage, credentials, and logging each
live in dedicated crates with test seams.

The current client is RouterOS v7 REST. Resource descriptors sit above that
transport so a native API-SSL path could be added later for subscriptions
without rewriting screens.
