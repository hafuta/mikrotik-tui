# Architecture

## Dependency direction

`mikrotik-tui` composes `mtui-app`. `mtui-app` depends on `mtui-ui`,
`mtui-routeros`, `mtui-config`, and `mtui-core`. `mtui-ui` contains no
networking or persistence. `mtui-routeros` contains no terminal code.
`mtui-core` holds the resource catalog and theme palettes only.

## State flow

1. The runtime reads crossterm input, ticks, and worker results into typed events.
2. `App::update` mutates deterministic screen state and returns commands.
3. Commands run on a tokio runtime, call the RouterOS client, and send typed messages.
4. Stale responses are rejected by request generation.
5. `render::draw` paints entirely from current state and semantic theme tokens.

## Security boundaries

- HTTPS verification is mandatory. Self-signed devices use custom roots or an
  explicitly approved SHA-256 leaf certificate pin.
- Profile files exclude passwords. Credentials use a separate owner-only store
  with a replaceable interface for future OS keyrings.
- Application logging is file-only while ratatui owns stdout. Redaction is
  applied before records reach the handler.
- The client mutates RouterOS only through confirmed actions and property
  sheets. Destructive commands (remove, reset counters) use an alert overlay.
  Secrets stay masked in tables and inspectors.

## Resource extension

Each resource declares a RouterOS endpoint, label, columns, identity fields,
and optional refresh interval. The generic resource screen owns filtering,
sorting, scrolling, selection preservation, loading, empty and failure states.
Add fixtures and descriptor tests whenever a resource is introduced.
