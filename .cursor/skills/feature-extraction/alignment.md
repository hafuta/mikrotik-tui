# Form alignment (canvas → agent rules)

Source of truth for *why*: the WebFig alignment canvas decisions. Source of
truth for *FieldKind* tables: `routeros-resource-development`. This file is
what to apply **during extraction**.

Baseline: RouterOS **7.21.5** WebFig. “Basic” alignment = own contract per
resource, WebFig labels and control kinds, named sections where that menu
has named pages. It is **not** a live probe of every field on a router.

## One contract, three presentations

| Surface | Fields |
|---|---|
| **Add (New)** | Every writable section. No Status heading, Status fields, or runtime extras. Optional controls stay inactive until `+`. |
| **Edit** | Same section sequence as New, plus Status. |
| **Details** | Same section **captions and order** as the entity form. Unknown API keys still appear as extras unless the screen opts out. |

Implementation: `FormSchema::sections_for(create)`. `FormSection::hidden_on_create`
is `read_only` or `id` `status`. Do not maintain a short `create_sections`
identity stub. `create_sections` is only for prompt-only schemas whose
`sections` slice is empty.

UI: one vertically scrollable modal; WebFig headings inline; **no section tabs**.

## Field kinds

Sheet **labels** are the full WebFig caption. The API key is not the control
(`/system logging action` Type is `target`).

| WebFig / WinBox | `FieldKind` |
|---|---|
| Fixed enum combo | `Enum { values }` (API strings) |
| Combo of other objects | `Lookup { resource_id, value_key, multiple }` |
| Checkbox / yes-no | `Toggle` |
| Enable mapped from `disabled` | `InvertedToggle` (label `Enabled`; inversion not shown) |
| Number spinner | `Number` or `ConstrainedNumber` |
| Password / secret | `Secret` |
| Free text | `Text` |
| Repeater (`+` rows) | `Repeat` (comma list on the wire) |
| Optional scalar | `Optional { kind, unset, unset_label }` — opt-in `+` row first |
| Time | `Time` / `Optional` + `ScalarKind::Time` |
| Runtime / not editable | `Readonly` |

Numeric input: digits and field-valid syntax only; never put `auto` in the
input. `port` / `*-port` (not `*-ports`) cap at five digits.

Enum **display** may differ from the API value (`syslog` → `BSD syslog`).
Select/PATCH still use the API string.

Do not default string-like keys to `Text`. Assert kinds in tests.

## Visibility vs locked vs enabled

When WebFig shows or hides a control from Type, format, chip, or another
selection, encode it in feature `rules.rs` → `form_field_state` →
`(visible, enabled)`. `forms.rs` `declared_field_state` must call that
function (chain `if let Some` like existing features).

- **Hidden**: not typed, not cycled, not sent on save. Do not leave a flat
  list of inapplicable rows marked locked.
- **Locked**: Status / `Readonly`, or a control that is present but not
  editable.
- **Enabled**: edit/save gate; follows visibility for associated fields.

Chip-specific print keys (Switch `cpu-flow-control` only on some ASICs):
`FieldPredicate::HasKey` so fields appear only when print included them.

## Sections (do not invent a tiny General)

Match **that menu’s** WinBox/WebFig grouping.

- **General**: identity and the main Properties page (`name`, `comment`,
  Enabled/`disabled`, everyday knobs, type-defining keys).
- **Type-specific** (Ethernet PHY, Radio, APN): only when WebFig has that
  page. Not a leftover drawer.
- **Advanced**: infrequent or easy-to-break knobs WebFig parks off the main
  page. Omit the heading if nothing belongs there. Do not move MTU/MAC to
  Advanced to shrink General.
- **Match**: packet/classifier criteria when WebFig uses Match (bridge
  filter/NAT, switch-rule, IP firewall filter/nat/mangle/raw, IPsec
  **policy**). IPsec **peer** Advanced stays Advanced. IPv6 firewall
  matchers stayed in General (no Match rename) — follow WebFig per menu,
  not a global “all firewalls get Match”.
- **Status**: runtime only. Never mix editable fields in.

Unlabeled single sheets are valid when WebFig is a single page (lists, APN,
some WiFi helpers). Do not force General/Advanced onto those.

## `disabled` → Enabled

API key `disabled`. Label `Enabled`. Kind `InvertedToggle`. Operators never
see inversion. Enable/disable may also remain a row action.

## New is not a stub

PPP Secrets New includes caller-id, addresses, IPv6 prefix, Enabled, and
the rest — not only name/password/service/profile. Same for every form in
the extracted group.

## Details / inspector

After isolation, hydration tests that look for abbreviated captions will
fail. Update the expected **form** label (`L3 HW Offloading`), not the
schema, unless the schema was wrongly shortened.

## Package gates and secrets

`wifi-qcom` / wireless / container / VETH / architecture rules stay the
same. Mask password, secret, passphrase, private-key, PSK on every table
and inspector path.

## What extraction is not

- Not live WebFig field-order sign-off.
- Not fixing menu visibility vs WebFig (6to4-style gaps) unless asked.
- Not adding SIT or other intentionally absent Interfaces rows.
- Not restoring short `create_sections` lists after “full New sheets”.
