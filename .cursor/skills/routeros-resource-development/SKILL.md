---
name: routeros-resource-development
description: Develop and review MikroTik RouterOS resources, API mappings, mutations, fixtures, and property-sheet forms. Use when adding RouterOS endpoints, resource models, client operations, edit/create forms, General/Advanced/Status sections, field kinds (enum, lookup, toggle, text), or resource tests in this repository.
---
# RouterOS resource development

1. Inspect adjacent resources and the client abstraction before coding; follow their naming, registration, and error conventions. Set `cli_path: Some("/certificate")` when the RouterOS command is not the catalog `endpoint`, or when the nav group prefix would be wrong (Certificates under System, path `/certificate`). Overlay-only (`FetchKind::Local`) screens must set `cli_path`. Catalog `endpoint` values are classic API paths (`/interface`, `/log`).
2. When implementing a MikroTik screen (catalog resource / WinBox menu), always verify the type of each field and implement accordingly. Do not default string-like API keys to `FieldKind::Text`.
3. Keep RouterOS transport details out of TUI models. Represent API words and values at the resource boundary and preserve unknown values where practical.
4. Treat RouterOS IDs as opaque strings. Distinguish absent values from explicit zero/false values when encoding mutations.
5. Bound every network operation with a timeout and drop in-flight work when the screen, profile, or generation changes. Return errors with the resource and operation while preserving the wrapped cause.
6. Never log or render credentials, authorization headers, URL userinfo, or
   sensitive RouterOS fields. Every generic table/inspector path must mask
   password, secret, passphrase, private-key, and pre-shared-key values before
   they reach a component; add a marker-secret regression test.
7. Add table-driven tests for decoding, encoding, optional fields, RouterOS error replies, cancellation, and malformed data. Use fakes; tests must not require a router.
8. Run `cargo fmt`, Clippy with `-D warnings` (`just prepush` / `just check`),
   the affected crate tests, and `cargo test --workspace`. In `//!` / `///`,
   wrap CamelCase product names in backticks (`` `WireGuard` ``). Otherwise
   Clippy `doc_markdown` fails CI with `item in documentation is missing
   backticks`.
9. Add a `ScreenGuide` in `about.rs` for every new resource id (and dashboard).
   Paraphrase the RouterOS manual; do not invent protocol claims or copy
   property tables. Prefer a conceptual `manual.mikrotik.com` URL when one
   exists; the CLI reference URL is derived from the resource path.
10. Group property-sheet fields using the tab rules below. Do not invent a
   “tiny General / leftover Advanced” split.
11. When moving a whole navigation group out of `LEGACY_RESOURCES`, follow
    `.cursor/skills/feature-extraction/SKILL.md` (hybrid catalog, facades,
    New/Status, Enabled inverted toggle). This skill stays the field-kind
    and section vocabulary.

## Field kinds

Sheet **labels** use the full WebFig caption. Do not shorten (`Syslog
Facility`, not Facility; `Memory Stop On Full`, not Mem stop full). The
API key is not the control: `/system logging action` stores Type as
`target`; Remote Protocol is `remote-protocol`.

| WebFig / WinBox control | `FieldKind` |
|---|---|
| Combo with a fixed RouterOS enum | `Enum { values }` (API strings: `udp`, `memory`, `default`) |
| Combo of other objects (VRF, script, interface, logging action name) | `Lookup { resource_id, value_key, multiple }` |
| Checkbox / yes-no | `Toggle` |
| Number spinner | `Number` (ASCII digits only; `port` / `*-port` cap at 5 digits) |
| Password / secret | `Secret` |
| Free text | `Text` |
| Repeater (`+` rows: addresses, VLAN IDs, servers) | `Repeat` (stored as a comma list) |
| Runtime / not editable | `Readonly` |

Assert the kind in tests (`FieldKind::Enum { values: … }`, `assert_lookup`).
Default create values for `Enum` to the first listed option when RouterOS
has a default (Type `memory`).

`FieldKind::Number` rejects letters, signs, and other non-digits while
typing. Keys `port` and `*-port` (not list keys like `*-ports`) accept at
most five digits so a TCP/UDP port cannot grow past 65535. Ignore extra
keystrokes; do not show a validation message. Do not retag Text fields
as Number just to get this filter.

When WebFig **shows or hides** a control based on Type, format, or another
selection, encode that in `field_visible`. Render only associated fields.
Do not keep a flat list of inapplicable rows and mark them locked. Locked
is for Status / `FieldKind::Readonly` (or a control that is present but
not editable). Hidden fields must not be typed, cycled, or sent on save.

`field_enabled` is the edit/save gate. For Logging Actions it follows
visibility: Type `memory` shows Memory Lines; `remote` shows Remote
Address and Remote Log Format; Remote Protocol is `udp`, `tcp`, or `tls`;
Check Certificate appears only for `tls`; Syslog Facility and Syslog
Severity appear only for BSD syslog (`syslog`); Timestamp Format for
`syslog` or `cef`; CEF Event Delimiter only for `cef`. Remote-log TLS
exists on RouterOS 7.23 and newer. NTP Server shows Broadcast Addresses
only when Broadcast is on, and Local Clock Stratum only when Use Local
Clock is on (modes stay independent toggles, not a Type combo). VRF is a
Lookup; Local Clock Stratum is Number; Broadcast Addresses is a Repeat
under Broadcast; Auth. Key is a Lookup of NTP Keys (`none` plus `key-id`).

Enum **display** may differ from the API string (`syslog` → `BSD syslog`)
while the Select list and PATCH still use the API value. Space/Enter on an
Enum opens the same nested **Select** modal as Lookup: filter, ↑↓, a
bounded scroll viewport, Enter to set the API value.

## Property-sheet sections

Schemas live in `mtui-core` (feature `forms.rs` for isolated groups). Add
and Edit are **one vertically scrollable sheet** with inline WebFig section
headings. Do not add section tabs. Omit a heading when it would have no
fields.

Match **WinBox/WebFig for that menu**, not an arbitrary “keep General small”
heuristic. Operators already know those groupings.

### General

Identity and the fields WinBox puts on the main Properties page for this
menu: `name`, `comment`, `disabled` when they exist, plus the everyday
settable knobs for the object (for `/interface`: `mtu`, `l2mtu`,
`mac-address`). Type-defining keys belong here too (`vlan-id` + parent
`interface` on VLAN, tunnel id / local / remote on tunnels).

### Type-specific tab (optional)

Only when the menu has a distinct subsystem WinBox gives its own tab:
Ethernet PHY (`auto-negotiation`, `speed`, `advertise`, …), Radio, APN.
Do not use this tab as a junk drawer for leftover writable fields.

### Advanced

Infrequent, easy-to-break, or extra settable fields WinBox parks off the
main page (`use-service-tag`, unusual protocol flags). If nothing qualifies,
**do not add an empty Advanced tab**. Do not move MTU/MAC here just to
shorten General.

### Status

Read-only runtime only: `type`, `running`, `slave`, `actual-mtu`, counters,
link times, `default-name`, derived MAC/switch state. Unknown keys returned
by the router and missing from the schema render here as extras. Never mix
editable fields into Status.

**Status is Edit-only.** New must not show a Status heading, Status fields,
or runtime extras. Details and inspect of an existing row may show Status.

### Create (New)

New is the **full writable sheet**, not a minimal identity stub. PPP
Secrets New must include the same fields as Edit (caller-id, local/remote
address, IPv6 prefix, Enabled, and the rest), not only name/password/
service/profile. Same rule for every form in the app.

`FormSchema::sections_for(true)` uses `sections` and drops Status. Do not
maintain a short `create_sections` list to hide fields on New.
`create_sections` is only for prompt-only schemas whose `sections` slice
is empty.

Type-specific knobs belong on General (or the matching WebFig section) and
appear only when `field_visible` says they apply.

### Actions vs fields

Enable/disable can stay a row action **and** a General toggle. Counters stay
on Status; reset-counters is an action, not an editable field. Torch is an
overlay, not a tab.
