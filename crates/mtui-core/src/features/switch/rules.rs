//! Chip-specific print-key gates owned by the Switch feature.

use std::collections::HashMap;

use crate::forms::{FieldPredicate, FieldRule};

const fn printed(resource_id: &'static str, field_key: &'static str) -> FieldRule {
    FieldRule {
        resource_id,
        field_key,
        visible: FieldPredicate::HasKey(field_key),
        enabled: FieldPredicate::HasKey(field_key),
    }
}

/// Chip-specific `/interface/ethernet/switch` attributes from the CLI
/// reference. Print omits keys the chip does not expose (`MediaTek-MT7621`
/// has `mirror-source`/`mirror-target` but not `cpu-flow-control` or
/// `mirror-egress-target`). Port `l3-hw-offloading` follows
/// `/interface/ethernet/switch/port` print the same way.
pub(crate) const FIELD_RULES: &[FieldRule] = &[
    printed("switch", "mirror-source"),
    printed("switch", "mirror-target"),
    printed("switch", "mirror-egress-target"),
    printed("switch", "cpu-flow-control"),
    printed("switch", "l3-hw-offloading"),
    printed("switch", "switch-all-ports"),
    printed("switch-port", "l3-hw-offloading"),
];

#[must_use]
#[allow(clippy::implicit_hasher)]
pub(crate) fn form_field_state(
    resource_id: &str,
    key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    crate::forms::evaluate_field_rules(FIELD_RULES, resource_id, key, values)
}
