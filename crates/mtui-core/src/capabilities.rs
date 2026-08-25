//! Device package requirements and bulk-capable operator menus.

use std::collections::{HashMap, HashSet};

pub use crate::features::interfaces::rules::{WIFI_PACKAGES, WIRELESS_PACKAGES};
use crate::resources::{ALL_RESOURCES, DASHBOARD_ID, ResourceSpec};

/// Extra package that exposes `/container`, VETH, and `/app`.
pub const CONTAINER_PACKAGES: &[&str] = &["container"];

/// Badge when `/console/inspect` (or a print) shows the command path is absent.
pub const MISSING_PATH_REASON: &str = "path";

const CONTAINER_MENUS: &[&str] = &[
    "veth",
    "containers",
    "container-config",
    "container-envs",
    "container-mounts",
    "apps",
];

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
    if let Some(packages) = crate::features::interfaces::rules::required_packages(resource_id) {
        return Some(packages);
    }
    if CONTAINER_MENUS.contains(&resource_id) {
        return Some(CONTAINER_PACKAGES);
    }
    None
}

/// True when this list screen offers bulk check and batch enable/disable/remove.
#[must_use]
pub fn supports_bulk_select(resource_id: &str) -> bool {
    crate::features::interfaces::rules::BULK_SELECT_RESOURCES.contains(&resource_id)
        || BULK_SELECT_RESOURCES.contains(&resource_id)
}

/// Map resource id → missing package label for menus the installed set lacks.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn unavailable_menus(installed: &HashSet<String>) -> HashMap<String, String> {
    unavailable_menus_for_device(installed, "", "")
}

/// Package plus architecture gates. Empty `architecture` skips the arch filter.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn unavailable_menus_for_device(
    installed: &HashSet<String>,
    architecture: &str,
    cpu: &str,
) -> HashMap<String, String> {
    let installed_l: HashSet<String> = installed
        .iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect();
    let mut out = HashMap::new();
    for spec in ALL_RESOURCES.iter() {
        let Some(packages) = required_packages(spec.id) else {
            continue;
        };
        if packages
            .iter()
            .any(|package| installed_l.contains(&package.to_ascii_lowercase()))
        {
            if let Some(reason) = architecture_gap(spec.id, architecture, cpu) {
                out.insert(spec.id.to_string(), reason);
            }
            continue;
        }
        let label = packages.first().copied().unwrap_or("package").to_string();
        out.insert(spec.id.to_string(), label);
    }
    out
}

/// CLI segments for a catalog screen (`["interface", "bridge", "port-controller"]`).
#[must_use]
pub fn menu_path_segments(spec: &ResourceSpec) -> Option<Vec<&str>> {
    let path = spec.cli_path();
    if path.is_empty() || spec.id == DASHBOARD_ID {
        return None;
    }
    let segments: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    (!segments.is_empty()).then_some(segments)
}

/// Parent key used with `/console/inspect request=child` (`""` is the root).
#[must_use]
pub fn inspect_parent_key(prefix: &[&str]) -> String {
    prefix.join(",")
}

/// True when every segment appears as a child of the previous inspect parent.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn cli_path_available(segments: &[&str], tree: &HashMap<String, HashSet<String>>) -> bool {
    let mut prefix = Vec::new();
    for segment in segments {
        let children = tree.get(&inspect_parent_key(&prefix));
        let Some(children) = children else {
            return false;
        };
        if !children.contains(*segment) {
            return false;
        }
        prefix.push(*segment);
    }
    true
}

/// Resource ids whose catalog path is not in the live inspect tree.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn unavailable_from_menu_tree(
    tree: &HashMap<String, HashSet<String>>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for spec in ALL_RESOURCES.iter() {
        let Some(segments) = menu_path_segments(spec) else {
            continue;
        };
        if !cli_path_available(&segments, tree) {
            out.insert(spec.id.to_string(), MISSING_PATH_REASON.to_string());
        }
    }
    out
}

/// Overlay `extra` onto `primary`. `primary` wins when both hide the same id
/// so a missing wifi package still badges `wifi-qcom` rather than `path`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn merge_unavailable_menus(
    primary: HashMap<String, String>,
    extra: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = extra;
    out.extend(primary);
    out
}

/// `RouterOS` trap when the command tree has no such menu (hardware or package).
#[must_use]
pub fn is_missing_command_prefix(message: &str) -> bool {
    message
        .to_ascii_lowercase()
        .contains("no such command prefix")
}

fn architecture_gap(resource_id: &str, architecture: &str, cpu: &str) -> Option<String> {
    let arch = architecture.trim().to_ascii_lowercase();
    if arch.is_empty() {
        return None;
    }
    if !CONTAINER_MENUS.contains(&resource_id) {
        return None;
    }
    if resource_id == "apps" {
        if cpu.to_ascii_uppercase().contains("EN7562CT") {
            return Some("architecture".into());
        }
        if matches!(arch.as_str(), "arm64" | "x86") {
            return None;
        }
        return Some("architecture".into());
    }
    if matches!(arch.as_str(), "arm" | "arm64" | "x86") {
        None
    } else {
        Some("architecture".into())
    }
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
        assert_eq!(required_packages("containers"), Some(CONTAINER_PACKAGES));
        assert_eq!(required_packages("veth"), Some(CONTAINER_PACKAGES));
        assert_eq!(required_packages("apps"), Some(CONTAINER_PACKAGES));
        assert_eq!(required_packages("interfaces"), None);
        assert_eq!(required_packages("firewall-filter"), None);
    }

    #[test]
    fn bulk_select_covers_operator_lists() {
        for id in [
            "firewall-filter",
            "dhcp-leases",
            "queue-simple",
            "interfaces",
            "ethernet",
            "vlan",
            "routes",
            "ipv6-routes",
            "address-list",
            "ipv6-address-list",
            "users",
        ] {
            assert!(supports_bulk_select(id), "{id}");
        }
        for id in [
            "logs",
            "user-groups",
            "dns-static",
            "dashboard",
            "firewall-connections",
            "ipv6-firewall-connections",
        ] {
            assert!(!supports_bulk_select(id), "{id}");
        }
    }

    #[test]
    fn bulk_select_ids_are_catalogued_resources() {
        for id in BULK_SELECT_RESOURCES {
            assert!(
                ALL_RESOURCES.iter().any(|spec| spec.id == *id),
                "unknown bulk-select resource {id}"
            );
        }
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
        assert_eq!(
            missing.get("containers").map(String::as_str),
            Some("container")
        );
        assert!(!missing.contains_key("interfaces"));
        installed.insert("wifi-qcom".into());
        let missing = unavailable_menus(&installed);
        assert!(!missing.contains_key("wifi"));
        assert!(missing.contains_key("wireless"));
    }

    #[test]
    fn container_arch_and_apps_gates() {
        let mut installed = HashSet::new();
        installed.insert("container".into());
        let missing = unavailable_menus_for_device(&installed, "mmips", "");
        assert_eq!(
            missing.get("containers").map(String::as_str),
            Some("architecture")
        );
        let missing = unavailable_menus_for_device(&installed, "arm", "");
        assert!(!missing.contains_key("containers"));
        assert_eq!(
            missing.get("apps").map(String::as_str),
            Some("architecture")
        );
        let missing = unavailable_menus_for_device(&installed, "arm", "EN7562CT");
        assert_eq!(
            missing.get("apps").map(String::as_str),
            Some("architecture")
        );
        let missing = unavailable_menus_for_device(&installed, "arm64", "");
        assert!(!missing.contains_key("apps"));
        let missing = unavailable_menus_for_device(&installed, "", "anything");
        assert!(!missing.contains_key("containers"));
    }

    fn child_set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    fn hex_like_bridge_tree() -> HashMap<String, HashSet<String>> {
        let mut tree = HashMap::new();
        tree.insert(
            String::new(),
            child_set(&["interface", "ip", "system", "tool"]),
        );
        tree.insert(
            "interface".into(),
            child_set(&["bridge", "ethernet", "vlan", "list"]),
        );
        tree.insert(
            "interface,bridge".into(),
            child_set(&[
                "port", "host", "vlan", "filter", "nat", "settings", "mdb", "msti",
            ]),
        );
        tree
    }

    #[test]
    fn port_controller_segments_match_catalog() {
        let spec = ALL_RESOURCES
            .iter()
            .find(|spec| spec.id == "bridge-port-controller")
            .expect("catalogued");
        assert_eq!(
            menu_path_segments(spec).as_deref(),
            Some(["interface", "bridge", "port-controller"].as_slice())
        );
    }

    #[test]
    fn email_tool_segments_use_hyphenated_cli_name() {
        let spec = ALL_RESOURCES
            .iter()
            .find(|spec| spec.id == "email")
            .expect("catalogued");
        assert_eq!(
            menu_path_segments(spec).as_deref(),
            Some(["tool", "e-mail"].as_slice())
        );
    }

    #[test]
    fn certificates_inspect_path_is_not_under_system() {
        let spec = ALL_RESOURCES
            .iter()
            .find(|spec| spec.id == "certificates")
            .expect("catalogued");
        assert_eq!(spec.group, "system-group");
        assert_eq!(
            menu_path_segments(spec).as_deref(),
            Some(["certificate"].as_slice())
        );
    }

    #[test]
    fn ipsec_key_print_lives_on_rsa_psk_and_qkd_children() {
        let cases: &[(&str, &[&str])] = &[
            ("ipsec-key-rsa", &["ip", "ipsec", "key", "rsa"]),
            ("ipsec-key-psk", &["ip", "ipsec", "key", "psk"]),
            ("ipsec-key-qkd", &["ip", "ipsec", "key", "qkd"]),
        ];
        for (id, expected) in cases {
            let spec = ALL_RESOURCES
                .iter()
                .find(|spec| spec.id == *id)
                .unwrap_or_else(|| panic!("{id}"));
            assert_eq!(menu_path_segments(spec).as_deref(), Some(*expected), "{id}");
        }
        assert!(ALL_RESOURCES.iter().all(|spec| spec.id != "ipsec-key"));
    }

    #[test]
    fn missing_path_hides_port_controller_when_bridge_lacks_the_child() {
        let missing = unavailable_from_menu_tree(&hex_like_bridge_tree());
        assert_eq!(
            missing.get("bridge-port-controller").map(String::as_str),
            Some(MISSING_PATH_REASON)
        );
        assert_eq!(
            missing.get("bridge-port-extender").map(String::as_str),
            Some(MISSING_PATH_REASON)
        );
        assert!(!missing.contains_key("bridges"));
        assert!(!missing.contains_key("bridge-settings"));
        assert!(!missing.contains_key("interfaces"));
    }

    #[test]
    fn path_gate_also_hides_wifi_when_the_command_is_absent() {
        let missing = unavailable_from_menu_tree(&hex_like_bridge_tree());
        assert_eq!(
            missing.get("wifi").map(String::as_str),
            Some(MISSING_PATH_REASON)
        );
        assert_eq!(
            missing.get("wifi-cap").map(String::as_str),
            Some(MISSING_PATH_REASON)
        );
    }

    #[test]
    fn package_label_wins_over_missing_path() {
        let mut packages = HashMap::new();
        packages.insert("wifi".into(), "wifi-qcom".into());
        let mut paths = HashMap::new();
        paths.insert("wifi".into(), MISSING_PATH_REASON.into());
        paths.insert("bridge-port-controller".into(), MISSING_PATH_REASON.into());
        let merged = merge_unavailable_menus(packages, paths);
        assert_eq!(merged.get("wifi").map(String::as_str), Some("wifi-qcom"));
        assert_eq!(
            merged.get("bridge-port-controller").map(String::as_str),
            Some(MISSING_PATH_REASON)
        );
    }

    #[test]
    fn architecture_still_hides_apps_when_the_path_exists() {
        let mut installed = HashSet::new();
        installed.insert("container".into());
        let packages = unavailable_menus_for_device(&installed, "arm", "EN7562CT");
        let mut tree = HashMap::new();
        tree.insert(String::new(), child_set(&["container", "app"]));
        tree.insert("container".into(), child_set(&["config", "envs", "mounts"]));
        let paths = unavailable_from_menu_tree(&tree);
        assert!(!paths.contains_key("apps"));
        let merged = merge_unavailable_menus(packages, paths);
        assert_eq!(merged.get("apps").map(String::as_str), Some("architecture"));
    }

    #[test]
    fn missing_command_prefix_detects_trap_copy() {
        assert!(is_missing_command_prefix(
            "failure: no such command prefix (6)"
        ));
        assert!(!is_missing_command_prefix("request timed out"));
    }
}
