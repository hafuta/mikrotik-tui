---
name: routeros-resource-development
description: Develop and review MikroTik RouterOS resources, API mappings, mutations, fixtures, and property-sheet form tabs. Use when adding RouterOS endpoints, resource models, client operations, edit/create forms, General/Advanced/Status tabs, or resource tests in this repository.
---
# RouterOS resource development

1. Inspect adjacent resources and the client abstraction before coding; follow their naming, registration, and error conventions.
2. Keep RouterOS transport details out of TUI models. Represent API words and values at the resource boundary and preserve unknown values where practical.
3. Treat RouterOS IDs as opaque strings. Distinguish absent values from explicit zero/false values when encoding mutations.
4. Bound every network operation with a timeout and drop in-flight work when the screen, profile, or generation changes. Return errors with the resource and operation while preserving the wrapped cause.
5. Never log or render credentials, authorization headers, URL userinfo, or
   sensitive RouterOS fields. Every generic table/inspector path must mask
   password, secret, passphrase, private-key, and pre-shared-key values before
   they reach a component; add a marker-secret regression test.
6. Add table-driven tests for decoding, encoding, optional fields, RouterOS error replies, cancellation, and malformed data. Use fakes; tests must not require a router.
7. Run `cargo fmt`, the affected crate tests, and `cargo test --workspace`.
8. Add a `ScreenGuide` in `about.rs` for every new resource id (and dashboard).
   Paraphrase the RouterOS manual; do not invent protocol claims or copy
   property tables. Prefer a conceptual `manual.mikrotik.com` URL when one
   exists; the CLI reference URL is derived from the resource path.
9. Group property-sheet fields using the tab rules below. Do not invent a
   “tiny General / leftover Advanced” split.

## Property-sheet tabs

Schemas live in `mtui-core` (see `interface_write.rs` for the Interfaces
group). The sheet shows **numbered tabs**, not a left rail and not an
accordion. Omit a tab when it would have no fields. A single-section sheet
hides the tab bar.

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

### Create sheets

`create_sections` is a **short General list**: required identity and parent
keys only (name, vlan-id, interface, tunnel endpoints). Comment is optional.
Do not copy Status or Advanced into create.

### Actions vs fields

Enable/disable can stay a row action **and** a General toggle. Counters stay
on Status; reset-counters is an action, not an editable field. Torch is an
overlay, not a tab.
