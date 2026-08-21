---
name: tui-quality
description: Build and review ratatui TUI behavior for correctness, responsiveness, accessibility, and deterministic testing. Use when changing models, messages, commands, views, key bindings, or terminal layouts in this repository.
---
# TUI quality

1. Keep `update` deterministic: mutate model state there and perform I/O only in commands that return typed messages.
2. Preserve cancellation and stale-result protection when screens, profiles, or requests change.
3. Make layouts safe for narrow, short, empty, loading, and error states. Avoid panics from negative widths or unchecked indexes.
4. Provide keyboard-first navigation, visible focus, discoverable key hints, and a non-color-only state indicator. Respect terminal color capability.
5. Keep secrets out of models used by `render`, debug formatting, and logs; request credentials only at the connection boundary.
6. Test update transitions and view invariants with fixed dimensions. Cover resize, repeated keys, empty data, errors, cancellation, and stale messages without sleeps or live network access.
7. Run `just check` (or `cargo fmt`, affected crate tests, and `cargo test --workspace`). Use `cargo clippy --workspace --all-targets` when concurrency or layout code changes. The compiler is pinned in `rust-toolchain.toml`.

## Rendering invariants

- Do not paint unbounded background colors on shared theme or component
  styles. Background painting is not reliably bounded around padding, wide
  glyphs, wrapping, and differential redraws, so it causes terminal bleed.
  Use foreground color, borders, weight, and spacing for hierarchy. If a future
  design truly requires a background, isolate it to a fully painted rectangle
  and add a regression test proving no background sequence escapes it.
- Render overlays with the shared fixed-canvas modal compositor. Never append
  a dialog below the layout: center it in both axes, replace covered cells,
  dim the remaining canvas using foreground/faint styling, and constrain long
  modal content to an internally scrollable viewport.
- Never replace populated content with a loading state during polling or manual
  refresh. Keep the previous table visible, track `refreshing` separately from
  first-load `loading`, and update only after the response arrives.
- Scope every polling result and scheduled tick to the current view generation.
  A tick created before navigation must be ignored after leaving or revisiting
  that screen, otherwise each visit creates another parallel refresh loop.
- Preserve selection, table offsets, and inspector viewport position when live
  data updates the same record. Reset position only when the user changes the
  selected resource.
- Treat pane dimensions as hard bounds. Constrain content before panel
  rendering so a tall sibling can never displace the navigation pane or the
  application header. Size the header and pane row to the terminal with no
  leftover right or bottom gutter; extra space belongs inside the focused
  content, not as unused chrome around it.
- Reserve live-dashboard section geometry from terminal dimensions alone.
  Loading, empty, error, and populated states must render into the same fixed
  slots so the first telemetry sample cannot resize or push adjacent charts.
- Text inputs must accept only printable runes. Windows terminal backends can
  report modifier-only keys such as Caps Lock as zero/control runes; never
  append raw key characters without filtering them.
- Add a regression test whenever addressing flicker, bleed, stale redraws, or
  viewport jumps; visual stability is part of correctness.
