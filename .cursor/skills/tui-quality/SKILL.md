---
name: tui-quality
description: >-
  Build and review ratatui TUI behavior for correctness, responsiveness,
  accessibility, and deterministic testing. Use when changing models, messages,
  commands, views, key bindings, terminal layouts, the event loop, listen/follow
  streams, Logs, or when the UI freezes, ignores keys, or redraws too often.
---
# TUI quality

1. Keep `update` deterministic: mutate model state there and perform I/O only in commands that return typed messages.
2. Preserve cancellation and stale-result protection when screens, profiles, or requests change.
3. Make layouts safe for narrow, short, empty, loading, and error states. Avoid panics from negative widths or unchecked indexes.
4. Provide keyboard-first navigation, visible focus, discoverable key hints, and a non-color-only state indicator. Respect terminal color capability.
5. Keep secrets out of models used by `render`, debug formatting, and logs; request credentials only at the connection boundary.
6. Test update transitions and view invariants with fixed dimensions. Cover resize, repeated keys, empty data, errors, cancellation, and stale messages without sleeps or live network access.
7. Before push run `just prepush` (format, then Clippy `-D warnings`, then tests). CI runs `just check` (fmt `--check`, same Clippy, same tests). Never Clippy then format. Keep OS-gated tests in their own `cfg` modules so unused imports cannot appear only on Linux or only on Windows. The compiler is pinned in `rust-toolchain.toml`.
8. Clippy `doc_markdown` fails CI with `item in documentation is missing backticks`. CamelCase in `//!` / `///` is treated as a type name. Write `` `WireGuard` ``, not WireGuard. Same for other product or type-like words (not `PPP`, which is all caps). Do not `allow` this lint. Fix the prose, then re-run Clippy.

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
- `FieldKind::Number` accepts ASCII digits only. Keys named `port` or
  `*-port` (not `*-ports`) stop at five digits. Drop extra keystrokes;
  do not show an error banner.
- `FieldKind::Repeat` expands into one row per value plus an add row.
  Enter on add (or on a filled item) appends a row. Backspace on an empty
  item removes it. The API value stays a comma-separated RouterOS list.
- Overlay lists that overflow (action menus, type pickers, property
  sheets) scroll an internal viewport. Keep filter/title and key hints
  pinned outside that viewport. Show overflow with a `n-m/total` range
  on the title and a right-edge track/thumb (`│` / `▐`), not a
  background fill. Focused rows must stay in view.
- New (create) shows every writable field from the schema. Do not hide
  Advanced, match, client, or other writable sections on New. Status is
  Edit-only: never render a Status heading, Status fields, or runtime
  extras on a create sheet.
- Add a regression test whenever addressing flicker, bleed, stale redraws,
  viewport jumps, or input starvation; visual stability and a live keyboard
  are part of correctness.

## Event loop and live streams

A populated screen that ignores keys for seconds is still a freeze. The Logs
page did this: `/log/print` filled the table, then `/log/print follow=`
replayed history as one `ListenDelta` per row. Each message ran `update` and
the loop redrew before the next. Keyboard polling lived on a 16ms `sleep`
branch; with the worker channel always ready that sleep never completed, and
restarting it every iteration meant keys were never read until the dump ended.

Rules:

- One frame: apply a **capped** batch of ready worker messages, poll
  crossterm with timeout 0, then draw once. Do not draw (or restart an input
  wait) after every `WorkerMsg`. `WORKER_MSGS_PER_FRAME` in
  `crates/mtui-app/src/runtime.rs` is the cap; keep input drain **after**
  `select`, not only on the idle timer branch.
- Snapshot then tail. For `/log`, print once, then `=follow-only=` — not
  `=follow=`, which replays from the oldest entry. Do not paint follow
  traffic until the print result for that generation has landed.
- Deduplicate stream rows (id and body). Evicting the buffer must not forget
  seen keys, or replay looks new again. Skip table rebuilds when nothing was
  added.
- Do not jump the viewport on live updates except follow mode (Logs: pin to
  the newest row). If the user moved off that row, keep their selection when
  newer lines arrive.
- First load may show `Loading…` only while the table is empty. After rows
  exist, keep them visible; never stall the event loop to “finish rendering”
  history. Expensive work belongs in the worker, batched into one model
  update, not in per-row redraws.
- High-rate `.listen` / torch / ping / follow must assume an unbounded
  channel will outrun the UI. Batch at the loop. Prefer protocol options that
  omit history. Add tests for batch size, duplicate follow rows, and
  selection/offset stability — no sleeps, no live router.

## Property sheets

Use numbered in-modal tabs (not a left rail, not an accordion). Arrow keys:
Up/Down move fields and clamp at the ends; Left/Right change tabs and clamp
at the first/last tab. Enum and Lookup fields open a nested Select/Lookup
dialog (dimmed canvas, centered, bounded height, internally scrolled).
Do not cycle enum values in place. Field grouping (General / type-specific /
Advanced / Status, create vs edit) is defined in
`.cursor/skills/routeros-resource-development/SKILL.md` — follow that skill
when adding or reshaping entity forms so screens stay consistent.
