//! Package and interaction gates owned by the IP feature.

use std::collections::HashMap;

use crate::forms::{FieldPredicate, FieldRule, evaluate_field_rules};

/// `WebFig` paints Advertise URL/Interval/Timeout only when Advertise is on.
const HOTSPOT_USER_PROFILE_RULES: &[FieldRule] = &[
    FieldRule {
        resource_id: "hotspot-user-profiles",
        field_key: "advertise-url",
        visible: FieldPredicate::Truthy("advertise"),
        enabled: FieldPredicate::Truthy("advertise"),
    },
    FieldRule {
        resource_id: "hotspot-user-profiles",
        field_key: "advertise-interval",
        visible: FieldPredicate::Truthy("advertise"),
        enabled: FieldPredicate::Truthy("advertise"),
    },
    FieldRule {
        resource_id: "hotspot-user-profiles",
        field_key: "advertise-timeout",
        visible: FieldPredicate::Truthy("advertise"),
        enabled: FieldPredicate::Truthy("advertise"),
    },
];

/// Visibility for Hotspot User Profiles advertisement fields.
///
/// Enabled follows visibility so hidden Advertise knobs are not typed or sent
/// on save. Other IP resources return `None`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub(crate) fn form_field_state(
    resource_id: &str,
    field_key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    evaluate_field_rules(HOTSPOT_USER_PROFILE_RULES, resource_id, field_key, values)
}
