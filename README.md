# RouterOS TUI

[![Rust](https://img.shields.io/badge/Rust-1.98+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/hafuta/routeros-tui/ci.yml?branch=master&label=CI)](https://github.com/hafuta/routeros-tui/actions/workflows/ci.yml)

> [!CAUTION]
> This project is new. Expect bugs and unstable behavior. Needs RouterOS
> 7.18 or newer.

<p align="center">
  <img src="assets/banner.png" alt="RouterOS TUI: keyboard-first RouterOS" width="520">
</p>

Keyboard-first terminal client for MikroTik RouterOS. It speaks the RouterOS
API over TCP: `api-ssl` (TLS, default port 8729) or the plaintext `api`
service (default port 8728). It shows live device state: interfaces,
addressing, DHCP, firewall, hardware, and logs, without leaving the terminal.

You can keep several devices open in tabs, each with its own connection. Create,
edit, and run the usual row actions (enable/disable, copy, remove, torch,
backups) through confirmations and a properties sheet. The same pattern covers
PPP, Bridge, Switch, IP, IPv6, Routing, Queues, Files, Tools, RADIUS, and
System.

## Features

- Several device tabs at once, each with its own live session. IP Neighbors can open a new tab from a discovered address.
- Live dashboard for CPU, memory, WAN throughput, and firewall activity
- Tables for the common WebFig operator menus, with search, sort, and a detail pane
- Named device profiles, last-used reconnect, optional saved password, and TOTP at connect
- Reconnect after a dropped session, with a per-tab LINK DOWN badge and writes blocked until the tab is live again
- READ MODE when the RouterOS group has no write policy: actions stay listed, but edit and mutate are blocked with a reason
- Demo mode (`--demo`) to learn the UI without a router
- TLS with the OS trust store, a CA file, or a pinned device certificate. Optional plaintext `api`.
- Safe Mode (`F4`): take, release, or unroll on this tab the same way WinBox does, including a prompt when another session already holds it
- Confirmed reboot, shutdown, disk format/eject, backup save/load, and System History undo
- Hide menus you do not use; restore them later
- In-app help (`?`) and a short description of the current screen (`i`)

RouterOS 7.18 or newer is required, with `api-ssl` or `api` enabled
(`/ip service`). Connect refuses older builds. Prefer `api-ssl`. Use a
dedicated, least-privileged account. Inspect-only access still works when
you only have read permission.

## Install

macOS:

```sh
brew tap hafuta/routeros-tui
brew install routeros-tui
```

Linux (amd64 and arm64):

```sh
curl -fsSL https://raw.githubusercontent.com/hafuta/routeros-tui/master/scripts/install-linux.sh | sh
```

The script prefers a user-owned directory (`~/.local/bin`, then `~/bin`) so
you do not need root. On a terminal it asks where to install; pass `--yes`
to take the default, or `--prefix DIR` to pick a directory. System paths
such as `/usr/local/bin` are offered last. It asks before replacing an
existing copy.

Windows: download an archive from
[Releases](https://github.com/hafuta/routeros-tui/releases).

The command is `routeros-tui`. An existing `mikrotik-tui` config directory
still applies until `routeros-tui` is created.

On first launch, add a device: host (`192.168.88.1` or `host:port`), username,
and password. TLS is on by default (port 8729). Turn TLS off for the
plaintext `api` service (port 8728). A CA-signed certificate is trusted from
the OS store (Windows certificate store, macOS keychain, Linux CA bundle).
You can also point CA file at a PEM or DER file. On that field, press enter
to browse folders in the terminal (Windows drive letters, macOS, and Linux).
For a self-signed certificate, compare the SHA-256 fingerprint before
trusting it.

Apple Terminal.app before macOS 26 Tahoe does not support 24-bit color; the
app falls back to 256 colors there. iTerm2 and Terminal on Tahoe use truecolor.
Set `ROUTEROS_TUI_COLOR=truecolor` or `256` to override.

## Keyboard

Press `?` for shortcuts that apply to the current screen. The bindings below
are the ones you use constantly.

### Sessions

| Keys | Action |
|------|--------|
| `ctrl+t` / `ctrl+w` | New / close device tab |
| `ctrl+tab` / `ctrl+shift+tab` | Next / previous device tab |

### Panes

| Keys | Action |
|------|--------|
| `tab` / `shift+tab` | Cycle panes |
| `r` | Refresh, or reconnect a dropped tab |
| `F4` | Take or release Safe Mode |

### Table

| Keys | Action |
|------|--------|
| `j` `k` / `↑` `↓` | Move |
| `h` / `l` | Scroll columns |
| `enter` | Open, expand, or edit |
| `/` | Filter |
| `e` / `n` | Edit / add |
| `d` / `x` | Enable-disable / remove |
| `space` / `*` | Check row / check all filtered (batch enable, disable, remove) |
| `a` | Action menu |
| `ctrl+s` | Save a form |

### App

| Keys | Action |
|------|--------|
| `?` / `i` | Help / about this screen |
| `ctrl+k` | Command palette |
| `esc` | Close overlay or clear filter |
| `q` | Quit |

Screen-specific keys (login, logs, torch, ping, backups, and so on) are listed
in `?` and in the footer.

## License

This project is licensed under the [MIT License](LICENSE).

MikroTik and RouterOS are trademarks of MikroTik. This project is not
affiliated with, endorsed by, or sponsored by MikroTik.
