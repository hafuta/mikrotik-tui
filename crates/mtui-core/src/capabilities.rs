//! Device package requirements and bulk-capable operator menus.

use std::collections::{HashMap, HashSet};

use crate::resources::ALL_RESOURCES;

/// Extra `RouterOS` packages that can expose `WiFi` Wave2 / `wifi-qcom` menus.
pub const WIFI_PACKAGES: &[&str] = &["wifi-qcom", "wifi-qcom-ac"];

/// Legacy `wireless` package (pre-wifiwave2).
pub const WIRELESS_PACKAGES: &[&str] = &["wireless"];

/// Resource ids where space-toggled multi-select is offered.
pub const BULK_SELECT_RESOURCES: &[&str] = &[
    "firewall-filter",
    "firewall-nat",
    "firewall-mangle",
    "firewall-raw",
    "dhcp-servers",
    "dhcp-networks",
    "dhcp-leases",
    "dhcp-relay",
    "queue-simple",
    "queue-tree",
    "interfaces",
    "interface-list-members",
    "ethernet",
    "eoip",
    "ipip",
    "gre",
    "6to4",
    "sit",
    "gre6",
    "vlan",
    "vxlan",
    "vrrp",
    "bonding",
    "lte",
    "wifi",
    "wireless",
    "wireguard",
    "macvlan",
    "macsec",
    "routes",
    "ipv6-routes",
    "address-list",
    "ipv6-address-list",
    "users",
];

/// Packages that must be installed for `resource_id` to exist on the device.
///
/// `None` means the menu ships in the base `routeros` package. When several
/// names are returned, any one of them is enough (wifi-qcom *or* wifi-qcom-ac).
#[must_use]
pub fn required_packages(resource_id: &str) -> Option<&'static [&'static str]> {
    if resource_id == "wifi" || resource_id.starts_with("wifi-") {
        return Some(WIFI_PACKAGES);
    }
    if resource_id == "wireless" || resource_id.starts_with("wireless-") {
        return Some(WIRELESS_PACKAGES);
    }
    None
}

/// True when this list screen offers bulk check and batch enable/disable/remove.
#[must_use]
pub fn supports_bulk_select(resource_id: &str) -> bool {
    BULK_SELECT_RESOURCES.contains(&resource_id)
}

/// Map resource id → missing package label for menus the installed set lacks.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn unavailable_menus(installed: &HashSet<String>) -> HashMap<String, String> {
    let installed_l: HashSet<String> = installed
        .iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect();
    let mut out = HashMap::new();
    for spec in ALL_RESOURCES {
        let Some(packages) = required_packages(spec.id) else {
            continue;
        };
        if packages
            .iter()
            .any(|package| installed_l.contains(&package.to_ascii_lowercase()))
        {
            continue;
        }
        let label = packages.first().copied().unwrap_or("package").to_string();
        out.insert(spec.id.to_string(), label);
    }
    out
}

/// Enabled package names from `/system/package` rows.
#[must_use]
pub fn installed_package_names<F>(rows: impl IntoIterator<Item = F>) -> HashSet<String>
where
    F: PackageRow,
{
    rows.into_iter()
        .filter_map(|row| {
            if row.package_disabled() {
                return None;
            }
            let name = row.package_name();
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// Minimal row view used when reading package print results.
pub trait PackageRow {
    fn package_name(&self) -> &str;
    fn package_disabled(&self) -> bool;
}

impl PackageRow for std::collections::HashMap<String, String> {
    fn package_name(&self) -> &str {
        self.get("name").map_or("", String::as_str)
    }

    fn package_disabled(&self) -> bool {
        matches!(
            self.get("disabled")
                .map(|value| value.trim().to_ascii_lowercase())
                .as_deref(),
            Some("true" | "yes" | "1")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_needs_qcom_package() {
        assert_eq!(required_packages("wifi"), Some(WIFI_PACKAGES));
        assert_eq!(required_packages("wifi-cap"), Some(WIFI_PACKAGES));
        assert_eq!(required_packages("wireless"), Some(WIRELESS_PACKAGES));
        assert_eq!(required_packages("interfaces"), None);
        assert_eq!(required_packages("firewall-filter"), None);
    }

    #[test]
    fn bulk_select_covers_operator_lists() {
        assert!(supports_bulk_select("firewall-filter"));
        assert!(supports_bulk_select("dhcp-leases"));
        assert!(supports_bulk_select("queue-simple"));
        assert!(supports_bulk_select("interfaces"));
        assert!(supports_bulk_select("ethernet"));
        assert!(supports_bulk_select("vlan"));
        assert!(supports_bulk_select("routes"));
        assert!(supports_bulk_select("ipv6-routes"));
        assert!(supports_bulk_select("address-list"));
        assert!(supports_bulk_select("ipv6-address-list"));
        assert!(supports_bulk_select("users"));
        assert!(!supports_bulk_select("logs"));
        assert!(!supports_bulk_select("user-groups"));
        assert!(!supports_bulk_select("dns-static"));
    }

    #[test]
    fn unavailable_menus_badge_missing_wifi() {
        let mut installed = HashSet::new();
        installed.insert("routeros".into());
        let missing = unavailable_menus(&installed);
        assert_eq!(missing.get("wifi").map(String::as_str), Some("wifi-qcom"));
        assert_eq!(
            missing.get("wireless").map(String::as_str),
            Some("wireless")
        );
        assert!(!missing.contains_key("interfaces"));
        installed.insert("wifi-qcom".into());
        let missing = unavailable_menus(&installed);
        assert!(!missing.contains_key("wifi"));
        assert!(missing.contains_key("wireless"));
    }
}
