//! Package and interaction gates owned by the IP feature.

use std::collections::HashMap;

#[must_use]
pub(crate) fn form_field_state(
    _resource_id: &str,
    _field_key: &str,
    _values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    None
}
