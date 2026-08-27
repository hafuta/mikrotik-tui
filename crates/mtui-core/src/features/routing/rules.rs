//! Field visibility gates owned by the `Routing` feature.

use std::collections::HashMap;

#[must_use]
#[allow(clippy::implicit_hasher)]
pub(crate) fn form_field_state(
    _resource_id: &str,
    _field_key: &str,
    _values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    None
}
