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
connects over the classic TCP API (`api-ssl`, port 8729) and presents live
operational state - interfaces, addressing, DHCP, firewall, hardware, and logs
- so any RouterOS device can be inspected without leaving the terminal.
Interface screens can create, edit, and run per-row actions (enable/disable,
copy, remove, torch, reset counters) through confirmation dialogs and a
sectioned properties sheet. Operator menus beyond Interfaces — PPP, Bridge,
Switch, IP, IPv6, Routing, Queues, Files, Tools, RADIUS, and System — use the
same action catalog. On Files, `f` runs `/tool/fetch` so the router pulls a
package or backup from HTTP(S). Tools screens open diagnostic overlays that
stream until you stop them. Runtime-only views (logs, health,
RouterBOARD, neighbor/session tables) stay non-editable except where WebFig
allows remove/disconnect.

## Features

- Live dashboard for CPU, memory, WAN throughput, and firewall activity
- IPsec peers, identities, policies, proposals, profiles, installed SAs, and settings
- Browse live RouterOS operational state across the common WebFig operator menus
- Search, sorting, detail inspector, application log console, command palette, in-app keyboard help, and per-screen RouterOS summaries
- Hide unused sidebar menus (categories or screens) and reveal them later to restore
- `api-ssl` with a custom CA or a pinned device certificate (the pin takes
  precedence when both are set)
- Confirmed reboot and shutdown from System Resources, plus named backup save
  and `.backup` load from Files
- Named device profiles with last-used auto-reconnect, optional remember-password, and TOTP at connect time
- Demo profile (`--demo` or the Demo row on the login list) for learning navigation without a router
- Save preview of changed fields before `ctrl+s`, copy of the selected row (`y`) or filtered table (`Y`), and bulk check on firewall, DHCP, and queues
- Menus that need a missing RouterOS package stay tucked away and show a `!package` badge when hidden menus are revealed
- Structured, redacted application logs on disk and in the in-app console

RouterOS v7 with `api-ssl` enabled is required. Use a dedicated,
least-privileged RouterOS account. Screens you edit require write permission
for those menus; inspect-only views still work with read-only API access.

## Roadmap

Work is tracked as GitHub milestones. Open issues and acceptance notes live
on each milestone; the full list is
[hafuta/mikrotik-tui milestones](https://github.com/hafuta/mikrotik-tui/milestones).

Shipped: [session, profiles, and login](https://github.com/hafuta/mikrotik-tui/milestone/2)
and [classic TCP API](https://github.com/hafuta/mikrotik-tui/milestone/3) (`api-ssl`).

Still open, in this order:

1. [Operator completeness](https://github.com/hafuta/mikrotik-tui/milestone/1) -
   remaining WebFig menus and row actions for a stock RouterOS 7 router
2. [Safe operations and session health](https://github.com/hafuta/mikrotik-tui/milestone/6) -
   reconnect and stale-data cues, permission-aware inspect-only, keyring, undo-lite
3. [Everyday operator UX](https://github.com/hafuta/mikrotik-tui/milestone/7) -
   per-screen action hints, save preview, copy/export, bulk select, demo profile
4. [Distribution and adoption](https://github.com/hafuta/mikrotik-tui/milestone/8) -
   signed releases, install channels, and a documented RouterOS version matrix

## Crates

| Crate | Role |
|-------|------|
| `mtui-core` | Resource catalog, shared types, **pluggable themes** |
| `mtui-routeros` | Classic TCP API client (`api-ssl`; TLS pin / custom CA; print/set/add/remove/command plus listen/streams) |
| `mtui-config` | Profiles, credentials, env overrides, file logging |
| `mtui-ui` | Pure widgets/layouts (no networking); styles from theme palette |
| `mtui-app` | State machine, polling, orchestration |
| `mikrotik-tui` | Binary entrypoint |

Semantic colors live in `mtui_core::Palette`. The built-in look is
`DefaultTheme` (`id = "default"`), registered in `ThemeRegistry`. UI code
never hard-codes product hex values - it uses `Styles::from_palette`.

- Profiles may store `preferences.theme = "default"` (see
`mtui_config::THEME_PREFERENCE_KEY`) so additional themes can be selected later
without schema changes. Hidden sidebar items persist as
`preferences.hidden_nav` (`mtui_config::HIDDEN_NAV_PREFERENCE_KEY`), a
comma-separated list of navigation ids.

Named device profiles live in `profiles.json`. Passwords are optional per
profile (`remember_password`) and are stored in the OS keychain when available,
falling back to an owner-only `credentials.json`. User Manager TOTP is typed at
connect time, appended to the static password, and never saved. The last-used
profile auto-reconnects unless it requires a fresh TOTP.

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

Flags: `--version`, `--no-alt-screen`, `--demo` (fixture profile, no router).

On first launch, pick a saved router or add a named profile: host
(`192.168.88.1` or `host:8729`), username, password, and an optional TOTP.
For a self-signed certificate, the app presents a
SHA-256 fingerprint and asks you to trust it before credentials are sent. An
approved fingerprint is treated as the router identity even when a local
certificate is expired or lacks an IP subject; any certificate replacement is
blocked until its new fingerprint is reviewed. Saved `https://` profiles are
migrated once to `host:8729`.

Saved files land under the platform config and state directories
(`~/.config/mikrotik-tui` on Linux, `%APPDATA%\mikrotik-tui` on Windows,
`~/Library/Application Support/mikrotik-tui` on macOS). `XDG_CONFIG_HOME` and
`XDG_STATE_HOME` override those locations on every platform. Remembered
passwords go to the OS keychain when it is available; otherwise they use an
owner-only file that is not encrypted. Shared or kiosk machines should leave
**Remember password** off, or use `MIKROTIK_TUI_PASSWORD_FILE`. TOTP codes are
never written to disk.

### Environment overrides

- `MIKROTIK_TUI_HOST` (host or `host:8729`)
- `MIKROTIK_TUI_URL` (deprecated; migrated like a saved HTTPS profile)
- `MIKROTIK_TUI_USERNAME`
- `MIKROTIK_TUI_PASSWORD`
- `MIKROTIK_TUI_PASSWORD_FILE` (preferred for containers)
- `MIKROTIK_TUI_CA_FILE`
- `MIKROTIK_TUI_CERT_FINGERPRINT`
- `MIKROTIK_TUI_LOG` (tracing filter; default `info,mtui_app=trace,mtui_routeros=info,mtui_config=info`)

## Keyboard

`↑/↓` or `j/k` moves, `h/l` pans table columns, `←/→` pans a wide table then
moves between the menu, central, and details panes, `tab` / `shift+tab`
cycle panes (including the log console), `enter` expands a nav category (accordion; first screen opens)
or opens the selected item. On an Interfaces table, `enter` edits the selected
row. `/` filters the table, `s`
cycles sort (not on Logs), `r` refreshes, `g`/`G` or Home/End jump, `pgup`/
`pgdn` and `ctrl+u`/`ctrl+d` page, `e` edits, `n` adds, `d` enables or disables,
`c` copies, `x` removes, `y` copies the selected row or inspector details to clipboard,
`Y` copies every filtered table row, `space` checks rows on firewall/DHCP/queue lists (`*` all, `esc` clears),
`z` resets counters, `[` / `]` move a filter-like rule
up or down, `m` makes a DHCP lease static, `t` opens torch, `b` reboots on
Resources or saves a backup on Files, `o` shuts down (power off) on Resources,
`f` fetches a URL onto the router, and load-backup is on the Files action menu
(`a`) for `.backup` rows. `p` opens ping on the Ping tool screen (Enter starts
traceroute on Traceroute). On Certificates, `g` signs, `p` imports a file already
on the router, and `w` exports PEM/PKCS12 (passphrases stay secret). `a` opens the
action menu, `ctrl+s` previews changed fields then saves a properties sheet, `ctrl+k` opens the command palette,
`ctrl+l` logs out (saved devices stay), `` ` `` toggles the application log console, `-` hides the
selected sidebar item after confirmation (or restores it when hidden menus are
showing), `.` shows hidden menus (marked with `×`
and strikethrough) so they can be restored, `?` opens help, `i` or `F1` opens a short description
of the current screen, `esc` closes overlays or clears the
filter, and `q` quits. Logging out returns to the device list and keeps saved
profiles and remembered passwords. `x` on the list (or **Forget this device**
in the palette) deletes one profile and its credential. Quitting also keeps
them so the last-used router can reconnect, unless that profile uses TOTP.

The log console is hidden by default and docks to the lower quarter of the
screen. Focus it with `` ` `` or `tab`, press `f` for fullscreen (header and
footer stay), `/` for case-insensitive search, `enter` to expand extra fields
(one row at a time; moving `j`/`k` keeps the expansion), `pgup`/`pgdn` to page,
and `c` to copy the focused record.

The Logs page keeps a bounded 500-event local stream with stable deduplication
and follows `/log/print` after the first print. `space` pauses the view while
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
