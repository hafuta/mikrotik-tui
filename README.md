# MikroTik TUI

A keyboard-first RouterOS control deck for Linux and Windows terminals. The
initial release is deliberately read-only: it makes common home-router state
easy to inspect without exposing configuration mutations.

## Features

- Responsive live dashboard with per-core CPU and memory Braille histories
- Auto-detected PPPoE/Ethernet throughput chart and firewall rule hit heatmap
- Interfaces, interface lists, Ethernet, PPP sessions and PPPoE clients
- Bridges, ports and VLANs
- ARP, addresses, DHCP servers, networks, leases and firewall filter rules
- Users, RouterBOARD, NTP client, clock and RouterOS logs
- Search, sorting, inspector, responsive layouts and contextual keyboard help
- HTTPS with custom CA or pinned self-signed certificates
- Persistent named connection profile and machine-local credentials
- Structured, redacted application logs

RouterOS v7 with `www-ssl` and REST access is required. Use a dedicated,
least-privileged RouterOS account with read and REST API permissions.

## Run

```sh
go run ./cmd/mikrotik-tui
```

On first launch, enter the RouterOS HTTPS URL, username and password. For a
self-signed certificate, the app presents a SHA-256 fingerprint and asks you
to trust it before credentials are sent. Saved files follow XDG directories.
An approved fingerprint is treated as the router identity even when a local
certificate is expired or lacks an IP subject; any certificate replacement is
blocked until its new fingerprint is reviewed.
The credential file is protected with owner-only permissions but is not
encrypted; use an appropriately protected machine or a secret-file override.

### Environment overrides

- `MIKROTIK_TUI_URL`
- `MIKROTIK_TUI_USERNAME`
- `MIKROTIK_TUI_PASSWORD`
- `MIKROTIK_TUI_PASSWORD_FILE` (preferred for containers)
- `MIKROTIK_TUI_CA_FILE`
- `MIKROTIK_TUI_CERT_FINGERPRINT`

## Keyboard

`↑/↓` or `j/k` moves, `←/→` or `h/l` scrolls columns, `tab` changes pane,
`enter` inspects, `/` filters, `s` sorts, `r` refreshes, `ctrl+p` opens the
command palette, `ctrl+l` logs out, `?` opens help, `esc` goes back, and `q`
quits. Logging out removes the saved local profile and credential; quitting
keeps them for automatic reconnection.

The Logs page keeps a bounded 500-event local stream with stable deduplication
and automatic retry. `space` pauses the view while ingestion continues, `f`
returns to the newest event, `e` cycles severity, and `c` clears only the local
buffer (it never deletes logs from RouterOS). Moving upward detaches from the
tail and shows an unread-event counter.

## Build and test

```sh
make check
make build
make release
docker build -t mikrotik-tui .
```

PowerShell equivalents:

```powershell
go test ./...
go vet ./...
go build -o bin/mikrotik-tui.exe ./cmd/mikrotik-tui
```

Optional read-only hardware tests require `MIKROTIK_TUI_INTEGRATION=1` and the
connection environment variables. Secrets are never checked into fixtures.

## Docker

The TUI requires an interactive terminal and direct network reachability:

```sh
docker run --rm -it --network host \
  -v mikrotik-tui-data:/data \
  mikrotik-tui
```

Mount `/data` to retain the profile, certificate pin, credential, and logs.

## Architecture

Bubble Tea owns application state and stdout. Network calls run as cancellable
commands behind a transport-neutral RouterOS client. Feature screens are data
descriptors over shared table and inspector components. Theme styles, profile
storage, credentials, and logging each have isolated packages and test seams.

REST is used for the RouterOS v7 snapshot-oriented release. The domain boundary
allows a native API-SSL transport to be introduced later for subscriptions
without rewriting screens.
