---
name: feature-extraction
description: >-
  Extract a RouterOS navigation group (menu / feature) from LEGACY_RESOURCES
  into features/<name>/ with owned catalog, forms, guides, rules, and tests,
  then align field kinds, visibility, and New/Edit/Status form state. Use when
  isolating a nav group, extracting several menus in parallel, cutting over
  the hybrid catalog, moving write-module forms, or applying WebFig section
  rules (Match, Enabled inverted toggle, Status hidden on New).
---

# Feature extraction (nav group isolation)

Do not start leftover groups the user did not name. Isolation is catalog
ownership plus **basic** form alignment, not a live 7.21.5 WebFig sign-off.

**One named group** → run this skill as a single sequential extraction.

**Several named groups in one request** → parallelize. Launch one worker per
group in the **same** turn (multiple Task/subagent calls together). Each
worker owns only `features/<name>/` (resources, forms, guides, rules, tests)
and that group's `*_write.rs` facade. Do **not** let workers race
`resources.rs`, `about.rs`, `forms.rs` `declared_field_state`,
`features/mod.rs`, catalog tests, or the canvas. After all trees exist, the
parent applies that shared cutover **once**, in nav order, then `just check`.

Independent work inside a large group (IP-style `forms/core.rs` vs `ipsec`
vs `hotspot`) may also run in parallel; join before wiring.

Read [alignment.md](alignment.md) before editing forms. Field-kind vocabulary
lives in `.cursor/skills/routeros-resource-development/SKILL.md`. Wiring
checklist: [wiring.md](wiring.md).

Copy the nearest **small** owned feature (`features/switch/` or
`features/wireguard/`), not Interfaces (subtype routing) and not IP (split
forms) unless the group is large.

## Inventory

1. In `crates/mtui-core/src/resources.rs`, take `NAVIGATION` group id
   (`routing-group`, `queue-group`, …). Collect every `ResourceSpec` in
   `LEGACY_RESOURCES` with that `group`. They may **not** be a contiguous
   prefix (today leftover starts at `users` / System while Routing sits later).
2. Count forms (`form: Some`), `form: None` tables, guides in `about.rs`,
   and the matching `*_write.rs` module(s).
3. Preserve `id`, `endpoint`, `cli_path`, `fetch`, `columns`, `actions`,
   `refresh`. Do not add catalog rows WebFig lacks. Do not drop rows the app
   already exposes.

## Layout

```
crates/mtui-core/src/features/<feature>/
  mod.rs          // rustdoc: catalog, forms, guides, tests; CamelCase in backticks
  resources.rs    // pub(crate) static RESOURCES: &[ResourceSpec]
  forms.rs        // or forms/{mod.rs, …} when one file would be huge
  guides.rs       // GUIDES: &[(&str, ScreenGuide)] — one entry per resource id
  rules.rs        // form_field_state → Option<(visible, enabled)>
  tests/mod.rs
  tests/forms.rs
```

`RESOURCES` order = previous leftover order for that group. Point
`form: Some(&crate::features::<feature>::forms::…)` at the owned schema.
Keep `form: None` for tables with no editor (hosts, neighbors, connections,
caches, cookies, and the like).

Split `forms/` only when a single file is unwieldy (IP: `core` / `ipsec` /
`hotspot`). Re-export so facades can `use crate::features::<feature>::forms::*`.

`actions.rs` is Interfaces-only (create targets, radio/list tables). Other
groups keep `crate::actions::*` on the spec.

## Forms while moving

Do not paste leftover schemas unchanged.

| Rule | Do |
|---|---|
| New | Full writable sheet. `create_sections: &[]` whenever `sections` is non-empty. `FormSchema::sections_for(true)` drops Status. |
| Status | `id: "status"`, `read_only: true`, `FieldKind::Readonly` only. Edit/Details only. |
| `disabled` | Label `Enabled`, `FieldKind::InvertedToggle`. API key stays `disabled`. |
| Captions | Full WebFig label (`L3 HW Offloading`, not `L3 HW`). |
| Kinds | Map the control, not the stringy API key. See alignment.md. |
| Sections | Match WinBox/WebFig names for **that** menu. Matchers → `match` / `Match` when WebFig does; do not invent Advanced as a junk drawer. |
| Lookups | Catalog `resource_id` + `value_key` (`name`, `key-id`, …). |
| Visibility | Hide inapplicable rows via `rules.rs` / `field_visible`. Locked ≠ hidden. |

**Do not copy** `features/wireguard/forms.rs` short `create_sections` lists.
Those predate “New is every non-Status section”.

## Cutover

Follow [wiring.md](wiring.md). Duplicate ids panic `build_catalog`. After
remove-from-legacy, `hybrid_catalog_includes_the_entire_feature_inventory`
must list the new slice and assert `LEGACY_RESOURCES[0].id` as the **actual**
first leftover spec (still `users` until System is extracted).

Update inspector tests in `mtui-app` if they assert captions that you
lengthened.

## Tests (feature-local)

Mirror `features/switch/tests/forms.rs` / `features/ip/tests/forms.rs`:

- Every `RESOURCES` id has a `GUIDES` entry; `GUIDES.len()` equals catalog len.
- Every form: `create_sections` empty, `create_keys() == writable_keys()`,
  `sections_for(true)` has no `status`.
- Status sections: `read_only`, every field `Readonly`, keys not writable.
- Every `disabled` field: label `Enabled`, kind `InvertedToggle`.
- Assert `FieldKind` for enums, lookups, repeats, secrets, numbers.
- If `rules.rs` is non-empty, assert show/hide with print-shaped `HashMap`s.
- `form_field_state` may be a stub returning `None` when nothing is gated.

## Check and canvas

1. `just check` (fmt `--check`, Clippy `-D warnings`, workspace tests). Never
   Clippy then format. `` `WireGuard` `` in `//!` / `///`.
2. If `interface-webfig-alignment-plan.canvas.tsx` is in the Cursor canvases
   folder, update owned counts, remaining nav pills, hybrid-catalog sentence,
   and “next feature” (do not imply the next group is in progress).
3. Isolation ≠ live parity. Do not claim 7.21.5 field-order sign-off.

## Scope

- Extract only groups the user named. If they named several, run those in
  parallel as above; do not serialize “to be safe” when the trees do not
  share files.
- Package gates (`wifi-qcom`, `container`, …) stay behaviorally identical.
- Do not add Interfaces-style `edit_resource_for_interface_type` unless the
  group has an aggregate row that must open a subtype schema.
