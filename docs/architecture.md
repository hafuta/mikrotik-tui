# Architecture

## Dependency direction

`routeros-tui` composes `mtui-app`. `mtui-app` depends on `mtui-ui`,
`mtui-routeros`, `mtui-config`, and `mtui-core`. `mtui-ui` contains no
networking or persistence. `mtui-routeros` contains no terminal code.
`mtui-core` holds the resource catalog and theme palettes only.

## State flow

1. The runtime reads crossterm input, ticks, and worker results into typed events.
2. `App::update` mutates deterministic screen state and returns commands.
3. Commands run on a tokio runtime, call the RouterOS client, and send typed messages.
4. Stale responses are rejected by request generation.
5. `render::draw` paints entirely from current state and semantic theme tokens.

When more than one device is open, the shell keeps tab ids and shared stores
while each session owns its client, UI, and I/O. Commands and worker messages
carry `SessionId`. See [multi-device sessions](multi-device-sessions.md).

## Security boundaries

- `api-ssl` (default 8729) verifies TLS against a pin, a custom CA file,
  or the OS trust store. Self-signed devices use an explicitly approved
  SHA-256 leaf certificate pin. Plain `api` (default 8728) is optional and
  unencrypted.
- Profile files exclude passwords. Remembered passwords use the OS keychain
  when available, with an owner-only file as fallback. TOTP is never stored.
- Application logging writes JSON to a file and a redacted in-memory buffer
  for the in-app console. Tracing never writes to stdout while ratatui owns
  the terminal.
- The client mutates RouterOS only through confirmed actions and property
  sheets. Destructive commands (remove, reset counters) use an alert overlay.
  Secrets stay masked in tables and inspectors.

## Resource extension

Each resource declares a RouterOS endpoint, label, columns, identity fields,
and optional refresh interval. The generic resource screen owns filtering,
sorting, scrolling, selection preservation, loading, empty and failure states.
Add fixtures and descriptor tests whenever a resource is introduced.

## Catalog menu visibility

The resource catalog is the allow-list. A screen appears in the sidebar only
if it is registered there. The live CLI tree never invents extra menus.

A registered id can still be omitted. Device gates mark the row unavailable
and drop it from the tree even when showing user-hidden rows. Operator hide
(`-` on a nav row, stored on the profile) tucks the row without claiming the
device lacks the command.

Device gates, in order:

1. **Package and architecture.** Wifi needs `wifi-qcom` (or `wifi-qcom-ac`),
   wireless needs `wireless`, containers need `container` plus a supported
   arch. Menus with no package requirement skip this step.
2. **Live command tree.** On connect, the client walks `/console/inspect
   request=child` for each catalog CLI path (the same inspect WebFig uses to
   learn which commands exist). A missing path hides that id. If inspect
   itself fails, package gates stay and no extra path hides are applied.

   Inspect uses `ResourceSpec::cli_path()`, not the nav group. Set
   `cli_path: Some("/certificate")` when the command does not live under the
   group prefix (Certificates is under System, path is `/certificate`).
   `None` means the catalog `endpoint` (`/interface/vlan`, `/log`). Overlay-only screens
   (`FetchKind::Local`) always set `cli_path` because they have no endpoint.
3. **Print trap.** Opening a menu that replies `no such command prefix` hides
   that id immediately, so a Hex does not stay on a CRS-only node such as
   Bridge Port Controller.

When both a package gap and a missing path apply, the package (or
architecture) badge wins. Empty tables do not hide a menu. WebFig's left rail
is a curated skin, so a type can exist as `/interface/6to4` and still have no
WebFig submenu. This app shows it whenever the catalog lists it and the path
is present.

**System → Regulatory** (country / wireless regulatory domain) is omitted on
purpose. It is not a catalog screen under System submenu parity
(https://github.com/hafuta/routeros-tui/milestone/12). Do not add
`regulatory` / `/system/regulatory` later as an inspect-driven gap; wireless
country rules are out of scope for this TUI.
