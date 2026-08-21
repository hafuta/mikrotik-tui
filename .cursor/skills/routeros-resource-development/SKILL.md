---
name: routeros-resource-development
description: Develop and review MikroTik RouterOS resources, API mappings, mutations, and fixtures. Use when adding RouterOS endpoints, resource models, client operations, or resource tests in this repository.
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
