//! Package and interaction gates owned by the Interfaces feature.

use std::collections::HashMap;

use crate::forms::evaluate_field_rules;

use super::forms::{base, cellular, tunnels, virtuals, wifi};

pub const WIFI_PACKAGES: &[&str] = &["wifi-qcom", "wifi-qcom-ac"];
pub const WIRELESS_PACKAGES: &[&str] = &["wireless"];

pub(crate) const BULK_SELECT_RESOURCES: &[&str] = &[
    "interfaces",
    "interface-list-members",
    "ethernet",
    "eoip",
    "ipip",
    "gre",
    "6to4",
    "gre6",
    "vlan",
    "vxlan",
    "vrrp",
    "bonding",
    "lte",
    "wifi",
    "wireless",
    "macvlan",
    "macsec",
];

#[must_use]
pub(crate) fn form_field_state(
    resource_id: &str,
    field_key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    for rules in [
        base::FIELD_RULES,
        cellular::FIELD_RULES,
        tunnels::FIELD_RULES,
        virtuals::FIELD_RULES,
        wifi::FIELD_RULES,
    ] {
        if let Some(state) = evaluate_field_rules(rules, resource_id, field_key, values) {
            return Some(state);
        }
    }
    None
}

#[must_use]
pub(crate) fn required_packages(resource_id: &str) -> Option<&'static [&'static str]> {
    if resource_id == "wifi" || resource_id.starts_with("wifi-") {
        return Some(WIFI_PACKAGES);
    }
    if resource_id == "wireless" || resource_id.starts_with("wireless-") {
        return Some(WIRELESS_PACKAGES);
    }
    None
}
