# Architecture

## Dependency direction

`cmd` composes `app`, `routeros`, `config`, `credentials`, and `logging`.
`app` depends on UI models and narrow service interfaces. `ui` and `theme`
contain no networking or persistence. `routeros` contains no terminal code.

## State flow

1. Bubble Tea receives input or an asynchronous result.
2. `Update` mutates deterministic screen state and returns commands.
3. Commands call the RouterOS client with a context and return typed messages.
4. Stale responses are rejected by request generation.
5. `View` renders entirely from current state and semantic theme tokens.

## Security boundaries

- HTTPS verification is mandatory. Self-signed devices use custom roots or an
  explicitly approved SHA-256 leaf certificate pin.
- Profile files exclude passwords. Credentials use a separate owner-only store
  with a replaceable interface for future OS keyrings.
- Application logging is file-only while Bubble Tea owns stdout. Redaction is
  applied before records reach the handler.
- The initial product has no mutation methods or destructive key bindings.

## Resource extension

Each resource declares a RouterOS endpoint, label, columns, identity fields,
and optional refresh interval. The generic resource screen owns filtering,
sorting, scrolling, selection preservation, loading, empty and failure states.
Add fixtures and descriptor tests whenever a resource is introduced.
