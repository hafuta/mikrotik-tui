# Hybrid catalog cutover

There is no `catalog/registry.rs`. Assembly is
`build_catalog(&[feature slices…], LEGACY_RESOURCES)` behind `ALL_RESOURCES`
in `crates/mtui-core/src/resources.rs`.

## Files to touch (every extraction)

| File | Change |
|---|---|
| `features/mod.rs` | `pub(crate) mod <feature>;` |
| `features/<feature>/…` | New owned tree |
| `resources.rs` | Prepend `crate::features::<feature>::resources::RESOURCES` in **both** `ALL_RESOURCES` and `hybrid_catalog_includes_the_entire_feature_inventory`. Delete those specs from `LEGACY_RESOURCES`. |
| `about.rs` | Chain `crate::features::<feature>::guides::GUIDES` in `screen_guide` **and** the uniqueness test helper. Remove the same ids from the leftover `GUIDES` table. |
| `forms.rs` `declared_field_state` | Call `crate::features::<feature>::rules::form_field_state` (same `if let Some` pattern). Last feature in the chain returns without wrapping. |
| `*_write.rs` | Facade only: `pub(crate) use crate::features::<feature>::forms::*;` plus a one-line rustdoc. IP used three facades (`ip_write`, `ipsec_write`, `hotspot_write`) all pointing at `features::ip::forms`. |
| `lib.rs` | Keep `mod <name>_write;` until callers stop using the facade. Do not delete the module in the same PR as isolation unless nothing imports it. |
| `capabilities.rs` | Only if this group owned package or bulk-select lists; Interfaces already routes wifi/wireless. Container menus stay in `capabilities` until Container is extracted. |

Do **not** change `NAVIGATION` group ids or labels.

When several groups are extracted in one request, write each
`features/<name>/` tree (and its `*_write.rs` facade) in parallel. Merge
this table’s shared files in one parent pass: append owned slices in
`NAVIGATION` order, delete every extracted `group` from `LEGACY_RESOURCES`
together, chain all new `GUIDES` and `form_field_state` calls once.

## Hybrid catalog order

Owned slices first, in nav order already used:

1. interfaces
2. wireguard
3. ppp
4. bridge
5. switch
6. ip
7. ipv6
8. **new group** (append here in `build_catalog` and the hybrid test)

Then `LEGACY_RESOURCES` (whatever remains, in leftover-table order).

`build_catalog` rejects duplicate `id`s. Never leave the group in both a
feature slice and `LEGACY_RESOURCES`.

Update `assert_eq!(LEGACY_RESOURCES[0].id, …)` to the first remaining spec.
That is **not** necessarily the next nav group to extract.

## Guides

Move paraphrased `ScreenGuide` rows; do not invent protocol claims or copy
property tables. Prefer a conceptual `manual.mikrotik.com` URL when one
exists. Every catalog id needs an entry (including `form: None` screens).

Feature `guides.rs` copies the `guide!` macro from an existing feature.

## Tests outside the feature crate

- `mtui-core` catalog tests: unique ids, hybrid prepend equality.
- `mtui-app` write/inspector tests: captions and section presence after
  hydrate.
- Do not require a router. Use fakes / print-shaped maps.

## Reference implementations

| Size | Copy |
|---|---|
| Small (≤ ~20 screens, one forms file) | `features/switch/` or `features/wireguard/` |
| Large / several write modules | `features/ip/` (`forms/core.rs`, `ipsec.rs`, `hotspot.rs`) |
| Families + subtype routing | `features/interfaces/` — **do not** copy unless the group needs aggregate→subtype edit |

## Canvas (if present)

Path (Cursor project, not always in git):
`interface-webfig-alignment-plan.canvas.tsx`

Update: decisions row for this group, isolation phase, family row, feature
roots path, hybrid catalog blurb, remaining-nav pills, stats (owned
resource count, groups extracted / 14), callout, next-feature todo.
