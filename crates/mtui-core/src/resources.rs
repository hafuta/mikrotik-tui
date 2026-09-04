//! Descriptor-driven `RouterOS` resource catalog and navigation tree.

use std::collections::HashSet;
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

use crate::actions::ActionSpec;
use crate::forms::FormSchema;

/// Dashboard nav / content id (not a list resource).
pub const DASHBOARD_ID: &str = "dashboard";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchKind {
    /// List-like collection.
    List { endpoint: &'static str },
    /// Singleton system resource.
    System { endpoint: &'static str },
    /// Overlay-driven screen; never polled.
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub fetch: FetchKind,
    pub columns: &'static [ColumnSpec],
    pub refresh: Duration,
    pub actions: &'static [ActionSpec],
    pub form: Option<&'static FormSchema>,
    /// CLI path for inspect, about, and the command palette when it differs
    /// from [`Self::endpoint`]. Nav group is not this prefix: Certificates
    /// sits under System but lives at `/certificate`.
    pub cli_path: Option<&'static str>,
}

impl ResourceSpec {
    #[must_use]
    pub fn endpoint(&self) -> &'static str {
        match self.fetch {
            FetchKind::List { endpoint } | FetchKind::System { endpoint } => endpoint,
            FetchKind::Local => "",
        }
    }

    #[must_use]
    pub fn cli_path(&self) -> &str {
        if let Some(path) = self.cli_path {
            return path;
        }
        match self.fetch {
            FetchKind::Local => self.id,
            FetchKind::List { endpoint } | FetchKind::System { endpoint } => endpoint,
        }
    }

    #[must_use]
    pub fn is_singleton(&self) -> bool {
        matches!(self.fetch, FetchKind::System { .. })
    }

    /// Watchdog, Reset Configuration, and `RouterBOARD` extra pages render the
    /// property sheet in the content pane instead of a table plus modal.
    #[must_use]
    pub fn is_inline_form(&self) -> bool {
        matches!(
            self.id,
            "watchdog"
                | "reset-configuration"
                | "routerboard-settings"
                | "routerboard-mode-button"
                | "routerboard-reset-button"
        )
    }

    #[must_use]
    #[allow(clippy::implicit_hasher)]
    pub fn resolved_actions(
        &self,
        row: Option<&std::collections::HashMap<String, String>>,
    ) -> Vec<&ActionSpec> {
        crate::actions::resolve_actions(self.actions, self.is_singleton(), row)
    }
}

/// Empty after every `NAVIGATION` group moved into `features/`. Kept so
/// `build_catalog` still concatenates owned slices then leftover.
static LEGACY_RESOURCES: &[ResourceSpec] = &[];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogError {
    pub duplicate_id: &'static str,
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "duplicate active resource id `{}`",
            self.duplicate_id
        )
    }
}

impl std::error::Error for CatalogError {}

fn build_catalog(
    features: &[&[ResourceSpec]],
    legacy: &[ResourceSpec],
) -> Result<Vec<ResourceSpec>, CatalogError> {
    let feature_len: usize = features.iter().map(|slice| slice.len()).sum();
    let mut ids = HashSet::with_capacity(feature_len + legacy.len());
    let mut resources = Vec::with_capacity(feature_len + legacy.len());
    for spec in features.iter().copied().flatten().chain(legacy) {
        if !ids.insert(spec.id) {
            return Err(CatalogError {
                duplicate_id: spec.id,
            });
        }
        resources.push(*spec);
    }
    Ok(resources)
}

/// Active catalog: owned feature slices in `NAVIGATION` order, then leftover.
pub static ALL_RESOURCES: LazyLock<Vec<ResourceSpec>> = LazyLock::new(|| {
    build_catalog(
        &[
            crate::features::interfaces::resources::RESOURCES,
            crate::features::wireguard::resources::RESOURCES,
            crate::features::ppp::resources::RESOURCES,
            crate::features::bridge::resources::RESOURCES,
            crate::features::switch::resources::RESOURCES,
            crate::features::ip::resources::RESOURCES,
            crate::features::ipv6::resources::RESOURCES,
            crate::features::routing::resources::RESOURCES,
            crate::features::queues::resources::RESOURCES,
            crate::features::files::resources::RESOURCES,
            crate::features::tools::resources::RESOURCES,
            crate::features::radius::resources::RESOURCES,
            crate::features::container::resources::RESOURCES,
            crate::features::system::resources::RESOURCES,
        ],
        LEGACY_RESOURCES,
    )
    .unwrap_or_else(|error| panic!("{error}"))
});

pub fn validate_active_catalog() -> Result<(), CatalogError> {
    LazyLock::force(&ALL_RESOURCES);
    Ok(())
}

fn ensure_valid_catalog() {
    if let Err(error) = validate_active_catalog() {
        panic!("{error}");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItem {
    pub id: String,
    pub label: String,
    pub children: Vec<NavItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavGroup {
    pub id: &'static str,
    pub label: &'static str,
}

/// Top-level navigation tree (dashboard + resource groups).
pub static NAVIGATION: &[NavGroup] = &[
    NavGroup {
        id: "interfaces-group",
        label: "Interfaces",
    },
    NavGroup {
        id: "wireguard-group",
        label: "WireGuard",
    },
    NavGroup {
        id: "ppp-group",
        label: "PPP",
    },
    NavGroup {
        id: "bridge-group",
        label: "Bridge",
    },
    NavGroup {
        id: "switch-group",
        label: "Switch",
    },
    NavGroup {
        id: "ip-group",
        label: "IP",
    },
    NavGroup {
        id: "ipv6-group",
        label: "IPv6",
    },
    NavGroup {
        id: "routing-group",
        label: "Routing",
    },
    NavGroup {
        id: "queue-group",
        label: "Queues",
    },
    NavGroup {
        id: "files-group",
        label: "Files",
    },
    NavGroup {
        id: "tools-group",
        label: "Tools",
    },
    NavGroup {
        id: "radius-group",
        label: "RADIUS",
    },
    NavGroup {
        id: "container-group",
        label: "Container",
    },
    NavGroup {
        id: "system-group",
        label: "System",
    },
];

#[must_use]
pub fn resource_by_id(id: &str) -> Option<&'static ResourceSpec> {
    ensure_valid_catalog();
    ALL_RESOURCES.iter().find(|spec| spec.id == id)
}

#[must_use]
pub fn navigation_tree() -> Vec<NavItem> {
    ensure_valid_catalog();
    let mut items = vec![NavItem {
        id: DASHBOARD_ID.to_string(),
        label: "Dashboard".to_string(),
        children: Vec::new(),
    }];
    for group in NAVIGATION {
        let children = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group.id)
            .map(|spec| NavItem {
                id: spec.id.to_string(),
                label: spec.label.to_string(),
                children: Vec::new(),
            })
            .collect();
        items.push(NavItem {
            id: group.id.to_string(),
            label: group.label.to_string(),
            children,
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_ids() {
        let mut ids: Vec<_> = ALL_RESOURCES.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL_RESOURCES.len());
        assert_eq!(validate_active_catalog(), Ok(()));
    }

    #[test]
    fn hybrid_catalog_includes_the_entire_feature_inventory() {
        let features: &[&[ResourceSpec]] = &[
            crate::features::interfaces::resources::RESOURCES,
            crate::features::wireguard::resources::RESOURCES,
            crate::features::ppp::resources::RESOURCES,
            crate::features::bridge::resources::RESOURCES,
            crate::features::switch::resources::RESOURCES,
            crate::features::ip::resources::RESOURCES,
            crate::features::ipv6::resources::RESOURCES,
            crate::features::routing::resources::RESOURCES,
            crate::features::queues::resources::RESOURCES,
            crate::features::files::resources::RESOURCES,
            crate::features::tools::resources::RESOURCES,
            crate::features::radius::resources::RESOURCES,
            crate::features::container::resources::RESOURCES,
            crate::features::system::resources::RESOURCES,
        ];
        let mut offset = 0;
        for feature in features {
            assert_eq!(&ALL_RESOURCES[offset..offset + feature.len()], *feature);
            offset += feature.len();
            for expected in *feature {
                assert_eq!(resource_by_id(expected.id), Some(expected));
            }
        }
        assert_eq!(ALL_RESOURCES.len(), offset + LEGACY_RESOURCES.len());
        assert_eq!(
            resource_by_id("switch").map(|spec| spec.group),
            Some("switch-group")
        );
        assert!(LEGACY_RESOURCES.is_empty());
    }

    #[test]
    fn hybrid_catalog_rejects_duplicate_active_ids() {
        let duplicate = std::slice::from_ref(
            crate::features::interfaces::resources::RESOURCES
                .first()
                .expect("Interfaces feature inventory"),
        );
        assert_eq!(
            build_catalog(&[duplicate, duplicate], &[]),
            Err(CatalogError {
                duplicate_id: "interfaces"
            })
        );
    }

    #[test]
    fn logging_actions_is_list_not_singleton() {
        let spec = resource_by_id("logging-actions").expect("logging-actions");
        assert!(!spec.is_singleton());
        assert_eq!(spec.endpoint(), "/system/logging/action");
        assert_eq!(spec.cli_path(), "/system/logging/action");
        assert_eq!(spec.group, "system-group");
        assert_eq!(spec.label, "Logging Actions");
        assert!(spec.form.is_some());
        let action_ids: Vec<_> = spec.actions.iter().map(|action| action.id).collect();
        assert_eq!(
            action_ids,
            crate::actions::LIST_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert!(matches!(
            spec.fetch,
            FetchKind::List {
                endpoint: "/system/logging/action"
            }
        ));
        assert_eq!(
            column_keys("logging-actions"),
            ["name", "target", "remote", "remote-port", "remote-protocol"]
        );
    }

    #[test]
    fn system_submenu_parity_catalog_ids() {
        let console = resource_by_id("system-console").expect("system-console");
        assert_eq!(console.group, "system-group");
        assert_eq!(console.endpoint(), "/system/console");
        assert_eq!(console.cli_path(), "/system/console");
        assert!(!console.is_singleton());

        let leds = resource_by_id("leds").expect("leds");
        assert_eq!(leds.endpoint(), "/system/led");
        let settings = resource_by_id("led-settings").expect("led-settings");
        assert!(settings.is_singleton());
        assert_eq!(settings.endpoint(), "/system/led/settings");

        let ports = resource_by_id("ports").expect("ports");
        assert_eq!(ports.endpoint(), "/port");
        assert_eq!(ports.cli_path(), "/port");

        let special = resource_by_id("special-login").expect("special-login");
        assert_eq!(special.cli_path(), "/special-login");
        assert_eq!(special.endpoint(), "/special-login");

        let reboot = resource_by_id("reboot").expect("reboot");
        assert!(matches!(reboot.fetch, FetchKind::Local));
        assert_eq!(reboot.cli_path(), "/system/reboot");
        assert!(reboot.actions.is_empty());
        assert!(resource_by_id("shutdown").is_some_and(|spec| spec.actions.is_empty()));

        let reset = resource_by_id("reset-configuration").expect("reset-configuration");
        assert!(reset.is_inline_form());
        assert!(reset.form.is_some());
        assert!(reset.actions.is_empty());

        assert!(resource_by_id("watchdog").is_some_and(ResourceSpec::is_inline_form));
        assert!(resource_by_id("routerboard-settings").is_some_and(ResourceSpec::is_inline_form));
        assert_eq!(
            resource_by_id("routerboard-settings")
                .expect("settings")
                .cli_path(),
            "/system/routerboard/settings"
        );
        assert!(ALL_RESOURCES.iter().all(|spec| spec.id != "regulatory"));
    }

    #[test]
    fn smb_shares_and_users_are_lists() {
        let shares = resource_by_id("smb-shares").expect("smb-shares");
        let users = resource_by_id("smb-users").expect("smb-users");
        let service = resource_by_id("smb").expect("smb");
        assert!(service.is_singleton());
        assert_eq!(service.endpoint(), "/ip/smb");
        assert!(!shares.is_singleton());
        assert!(!users.is_singleton());
        assert_eq!(shares.endpoint(), "/ip/smb/shares");
        assert_eq!(users.endpoint(), "/ip/smb/users");
        assert_eq!(shares.cli_path(), "/ip/smb/shares");
        assert_eq!(users.cli_path(), "/ip/smb/users");
        assert_eq!(shares.group, "ip-group");
        assert_eq!(users.group, "ip-group");
        assert_eq!(shares.label, "SMB Shares");
        assert_eq!(users.label, "SMB Users");
        assert!(shares.form.is_some());
        assert!(users.form.is_some());
        assert_eq!(
            column_keys("smb-shares"),
            [
                "name",
                "directory",
                "valid-users",
                "invalid-users",
                "read-only",
                "require-encryption",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("smb-users"),
            ["name", "password", "read-only", "disabled", "comment"]
        );
        let share_actions: Vec<_> = shares.actions.iter().map(|action| action.id).collect();
        assert_eq!(
            share_actions,
            crate::actions::MEMBER_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        let user_actions: Vec<_> = users.actions.iter().map(|action| action.id).collect();
        assert_eq!(user_actions, share_actions);
    }

    #[test]
    fn ntp_server_is_system_singleton() {
        let spec = resource_by_id("ntp-server").expect("ntp-server");
        assert!(spec.is_singleton());
        assert_eq!(spec.endpoint(), "/system/ntp/server");
        assert_eq!(spec.cli_path(), "/system/ntp/server");
        assert!(spec.form.is_some());
        assert_eq!(spec.group, "system-group");
        assert_eq!(spec.label, "NTP Server");
        assert!(matches!(
            spec.fetch,
            FetchKind::System {
                endpoint: "/system/ntp/server"
            }
        ));
        let ntp = resource_by_id("ntp").expect("ntp");
        assert_eq!(ntp.label, "NTP Client");
        assert_eq!(ntp.endpoint(), "/system/ntp/client");
        let keys = resource_by_id("ntp-keys").expect("ntp-keys");
        assert!(!keys.is_singleton());
        assert_eq!(keys.endpoint(), "/system/ntp/key");
        assert_eq!(column_keys("ntp-keys"), ["key-id"]);
        assert!(keys.form.is_some());
    }

    #[test]
    fn ospf_interface_runtime_is_not_the_template_menu() {
        let live = resource_by_id("ospf-interfaces").expect("ospf-interfaces");
        let templates =
            resource_by_id("ospf-interface-templates").expect("ospf-interface-templates");
        assert!(!live.is_singleton());
        assert_eq!(live.endpoint(), "/routing/ospf/interface");
        assert_eq!(live.cli_path(), "/routing/ospf/interface");
        assert_eq!(live.group, "routing-group");
        assert_eq!(live.label, "OSPF Interface");
        assert_eq!(templates.label, "OSPF Interface Templates");
        assert_eq!(templates.endpoint(), "/routing/ospf/interface-template");
        assert_ne!(live.endpoint(), templates.endpoint());
        assert_eq!(
            live.actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["edit"]
        );
        assert!(live.form.is_some());
        assert!(
            live.form
                .is_some_and(|form| form.writable_keys().is_empty())
        );
        assert!(
            templates
                .form
                .is_some_and(|form| !form.writable_keys().is_empty())
        );
        assert_eq!(
            column_keys("ospf-interfaces"),
            [
                "address",
                "area",
                "state",
                "network-type",
                "cost",
                "dr",
                "bdr",
            ]
        );
        assert!(!column_keys("ospf-interface-templates").contains(&"cost"));
        assert!(!column_keys("ospf-interface-templates").contains(&"state"));
        assert!(!column_keys("ospf-interfaces").contains(&"interfaces"));
        assert!(!column_keys("ospf-interfaces").contains(&"disabled"));
        let neighbors = resource_by_id("ospf-neighbors").expect("ospf-neighbors");
        assert!(neighbors.form.is_none());
        assert!(neighbors.actions.is_empty());
    }

    #[test]
    fn bgp_sessions_are_live_not_connections() {
        let sessions = resource_by_id("bgp-sessions").expect("bgp-sessions");
        let connections = resource_by_id("bgp-connections").expect("bgp-connections");
        assert_eq!(sessions.endpoint(), "/routing/bgp/session");
        assert_eq!(sessions.cli_path(), "/routing/bgp/session");
        assert_eq!(sessions.group, "routing-group");
        assert_eq!(sessions.label, "BGP Sessions");
        assert_eq!(connections.endpoint(), "/routing/bgp/connection");
        assert_ne!(sessions.endpoint(), connections.endpoint());
        assert_eq!(
            sessions
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["edit"]
        );
        assert!(
            sessions
                .form
                .is_some_and(|form| form.writable_keys().is_empty())
        );
        assert!(
            connections
                .form
                .is_some_and(|form| !form.writable_keys().is_empty())
        );
        assert_eq!(
            column_keys("bgp-sessions"),
            [
                "name",
                "remote.address",
                "remote.as",
                "established",
                "uptime",
                "prefix-count",
                "ebgp",
            ]
        );
        assert!(!column_keys("bgp-sessions").contains(&"disabled"));
        assert!(!column_keys("bgp-connections").contains(&"established"));
    }

    #[test]
    fn traffic_flow_and_igmp_proxy_are_ip_group_screens() {
        let flow = resource_by_id("traffic-flow").expect("traffic-flow");
        assert!(flow.is_singleton());
        assert_eq!(flow.endpoint(), "/ip/traffic-flow");
        assert_eq!(flow.cli_path(), "/ip/traffic-flow");
        assert_eq!(flow.group, "ip-group");
        assert!(flow.form.is_some());
        assert_eq!(
            column_keys("traffic-flow"),
            ["enabled", "interfaces", "cache-entries", "packet-sampling"]
        );

        let targets = resource_by_id("traffic-flow-targets").expect("traffic-flow-targets");
        assert!(!targets.is_singleton());
        assert_eq!(targets.endpoint(), "/ip/traffic-flow/target");
        assert_eq!(
            targets
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            crate::actions::MEMBER_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            column_keys("traffic-flow-targets"),
            ["src-address", "dst-address", "port", "version", "disabled"]
        );

        let ipfix = resource_by_id("traffic-flow-ipfix").expect("traffic-flow-ipfix");
        assert!(ipfix.is_singleton());
        assert_eq!(ipfix.endpoint(), "/ip/traffic-flow/ipfix");

        let proxy = resource_by_id("igmp-proxy").expect("igmp-proxy");
        assert!(proxy.is_singleton());
        assert_eq!(proxy.endpoint(), "/routing/igmp-proxy");
        assert_eq!(proxy.cli_path(), "/routing/igmp-proxy");
        assert_eq!(proxy.group, "ip-group");

        let ifaces = resource_by_id("igmp-proxy-interfaces").expect("igmp-proxy-interfaces");
        assert!(!ifaces.is_singleton());
        assert_eq!(ifaces.endpoint(), "/routing/igmp-proxy/interface");
        assert!(ifaces.form.is_some());

        let mfc = resource_by_id("igmp-proxy-mfc").expect("igmp-proxy-mfc");
        assert_eq!(mfc.endpoint(), "/routing/igmp-proxy/mfc");
        assert_eq!(
            mfc.actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            crate::actions::LIST_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert_unique_endpoints("ip-group");
    }

    #[test]
    fn ping_and_traceroute_are_local_fetch() {
        let ping = resource_by_id("ping").expect("ping");
        let traceroute = resource_by_id("traceroute").expect("traceroute");
        assert!(matches!(ping.fetch, FetchKind::Local));
        assert!(matches!(traceroute.fetch, FetchKind::Local));
        assert!(ping.form.is_none());
        assert!(traceroute.form.is_none());
        assert_eq!(ping.cli_path(), "/tool/ping");
        assert_eq!(traceroute.cli_path(), "/tool/traceroute");
    }

    #[test]
    fn cli_path_override_does_not_follow_nav_group() {
        let cert = resource_by_id("certificates").expect("certificates");
        assert_eq!(cert.group, "system-group");
        assert_eq!(cert.endpoint(), "/certificate");
        assert_eq!(cert.cli_path, Some("/certificate"));
        assert_eq!(cert.cli_path(), "/certificate");
        assert_eq!(
            crate::menu_path_segments(cert).as_deref(),
            Some(["certificate"].as_slice())
        );
        assert_eq!(
            resource_by_id("users").map(ResourceSpec::cli_path),
            Some("/user")
        );
        assert_eq!(
            resource_by_id("files").map(ResourceSpec::cli_path),
            Some("/file")
        );
        assert_eq!(
            resource_by_id("logging").map(ResourceSpec::cli_path),
            Some("/system/logging")
        );
    }

    #[test]
    fn navigation_includes_dashboard_and_logs() {
        let tree = navigation_tree();
        assert_eq!(tree[0].id, DASHBOARD_ID);
        assert!(resource_by_id("logs").is_some());
        assert!(resource_by_id("firewall-filter").is_some());
    }

    #[test]
    fn interface_tables_expose_webfig_columns() {
        assert_eq!(
            column_keys("interfaces"),
            [
                "name",
                "type",
                "mtu",
                "actual-mtu",
                "l2mtu",
                "max-l2mtu",
                "mac-address",
                "tx-byte",
                "rx-byte",
                "tx-packet",
                "rx-packet",
                "fp-tx-byte",
                "fp-rx-byte",
                "fp-tx-packet",
                "fp-rx-packet",
                "last-link-up-time",
                "last-link-down-time",
                "link-downs",
                "tx-drop",
                "rx-drop",
                "tx-queue-drop",
                "rx-error",
                "tx-error",
                "running",
                "slave",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("interface-lists"),
            ["name", "include", "exclude", "builtin", "comment"]
        );
        assert_eq!(
            column_keys("interface-list-members"),
            ["list", "interface", "dynamic", "disabled", "comment"]
        );
        assert_eq!(
            column_keys("ethernet"),
            [
                "name",
                "default-name",
                "mtu",
                "l2mtu",
                "mac-address",
                "orig-mac-address",
                "arp",
                "auto-negotiation",
                "advertise",
                "speed",
                "full-duplex",
                "switch",
                "loop-protect",
                "loop-protect-status",
                "running",
                "slave",
                "disabled",
                "comment",
            ]
        );
    }

    #[test]
    fn interface_group_covers_webfig_screens() {
        let ids: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == "interfaces-group")
            .map(|spec| spec.id)
            .collect();
        assert_eq!(
            ids,
            [
                "interfaces",
                "interface-lists",
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
                "lte-apn",
                "wifi",
                "wifi-security",
                "wifi-channel",
                "wifi-datapath",
                "wifi-configuration",
                "wifi-provisioning",
                "wifi-cap",
                "wifi-capsman",
                "wifi-registration-table",
                "wireless",
                "wireless-security-profiles",
                "wireless-access-list",
                "wireless-registration-table",
                "macvlan",
                "veth",
                "macsec",
                "macsec-profiles",
                "vrf",
                "detect-internet",
            ]
        );
        let endpoints: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == "interfaces-group")
            .map(ResourceSpec::endpoint)
            .collect();
        let mut unique = endpoints.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), endpoints.len());
        let wifi_reg = resource_by_id("wifi-registration-table").expect("wifi-registration-table");
        assert_eq!(wifi_reg.endpoint(), "/interface/wifi/registration-table");
        assert!(wifi_reg.form.is_none());
        assert_eq!(
            wifi_reg
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["remove"]
        );
        assert!(resource_by_id("detect-internet").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("vlan").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            column_keys("macsec"),
            [
                "name",
                "interface",
                "profile",
                "mtu",
                "status",
                "ckn",
                "cak",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(column_keys("macsec-profiles"), ["name", "server-priority"]);
        assert_eq!(
            resource_by_id("macsec").map(ResourceSpec::endpoint),
            Some("/interface/macsec")
        );
        assert_eq!(
            resource_by_id("macsec-profiles").map(ResourceSpec::endpoint),
            Some("/interface/macsec/profile")
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "interfaces-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn lte_apn_is_a_list_under_interfaces() {
        assert_eq!(
            resource_by_id("lte-apn").map(|spec| spec.label),
            Some("LTE APN")
        );
        assert_eq!(
            resource_by_id("lte-apn").map(ResourceSpec::endpoint),
            Some("/interface/lte/apn")
        );
        assert_eq!(
            resource_by_id("lte-apn").map(ResourceSpec::cli_path),
            Some("/interface/lte/apn")
        );
        assert_eq!(
            column_keys("lte"),
            [
                "name",
                "default-name",
                "mtu",
                "mac-address",
                "network-mode",
                "apn-profiles",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("lte-apn"),
            [
                "name",
                "apn",
                "authentication",
                "ip-type",
                "use-network-apn",
                "add-default-route",
                "password",
                "comment",
            ]
        );
        assert!(resource_by_id("lte-apn").is_some_and(|spec| spec.form.is_some()));
        assert!(!resource_by_id("lte-apn").is_some_and(ResourceSpec::is_singleton));
    }

    #[test]
    fn interface_lists_and_members_are_distinct() {
        assert_eq!(
            resource_by_id("interface-lists").map(|spec| spec.label),
            Some("Lists")
        );
        assert_eq!(
            resource_by_id("interface-list-members").map(|spec| spec.label),
            Some("List members")
        );
        assert_eq!(
            resource_by_id("interface-lists").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("New list"))
        );
        assert_eq!(
            resource_by_id("interface-list-members").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("New list member"))
        );
        assert_eq!(
            resource_by_id("dhcp-networks").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("Add"))
        );
    }

    #[test]
    fn wireguard_is_its_own_nav_group() {
        assert_eq!(
            resource_by_id("wireguard").map(|spec| spec.group),
            Some("wireguard-group")
        );
        assert_eq!(
            resource_by_id("wireguard-peers").map(|spec| spec.group),
            Some("wireguard-group")
        );
        let tree = navigation_tree();
        let group = tree
            .iter()
            .find(|item| item.id == "wireguard-group")
            .expect("wireguard nav group");
        let child_ids: Vec<_> = group.children.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(child_ids, ["wireguard", "wireguard-peers"]);
        assert!(
            tree.iter()
                .find(|item| item.id == "interfaces-group")
                .expect("interfaces nav group")
                .children
                .iter()
                .all(|item| item.id != "wireguard" && item.id != "wireguard-peers")
        );
    }

    #[test]
    fn wireguard_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("wireguard-group"),
            ["wireguard", "wireguard-peers"]
        );
        assert_unique_endpoints("wireguard-group");
        assert_eq!(
            column_keys("wireguard"),
            [
                "name",
                "listen-port",
                "public-key",
                "private-key",
                "mtu",
                "vrf",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("wireguard-peers"),
            [
                "name",
                "interface",
                "public-key",
                "endpoint-address",
                "endpoint-port",
                "allowed-address",
                "persistent-keepalive",
                "responder",
                "current-endpoint-address",
                "current-endpoint-port",
                "last-handshake",
                "rx",
                "tx",
                "disabled",
                "comment",
            ]
        );
        assert!(resource_by_id("wireguard").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("wireguard-peers").is_some_and(|spec| spec.form.is_some()));
        let wg_actions: Vec<_> = resource_by_id("wireguard")
            .expect("wireguard")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            wg_actions,
            [
                "add",
                "edit",
                "toggle-disabled",
                "copy",
                "remove",
                "reset-counters"
            ]
        );
        let peer_actions: Vec<_> = resource_by_id("wireguard-peers")
            .expect("wireguard-peers")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            peer_actions,
            ["add", "edit", "toggle-disabled", "copy", "remove"]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "wireguard-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn ppp_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("ppp-group"),
            [
                "ppp-secrets",
                "ppp-profiles",
                "ppp-active",
                "ppp-aaa",
                "ppp-client",
                "pppoe-clients",
                "pppoe-servers",
                "pppoe-server-ifaces",
                "pptp-client",
                "pptp-server-ifaces",
                "pptp-server",
                "l2tp-client",
                "l2tp-server-ifaces",
                "l2tp-server",
                "sstp-client",
                "sstp-server-ifaces",
                "sstp-server",
                "ovpn-client",
                "ovpn-server-ifaces",
                "ovpn-server",
            ]
        );
        assert_unique_endpoints("ppp-group");
        assert!(resource_by_id("ppp-aaa").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("l2tp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("sstp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("ovpn-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("pptp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("ppp-secrets").is_some_and(ResourceSpec::is_singleton));
        assert!(column_keys("ppp-secrets").contains(&"password"));
        assert!(column_keys("l2tp-client").contains(&"ipsec-secret"));
        assert!(resource_by_id("ppp-secrets").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("ppp-aaa").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("ppp-active").is_some_and(|spec| spec.form.is_none()));
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "ppp-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn bridge_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("bridge-group"),
            [
                "bridges",
                "bridge-ports",
                "bridge-hosts",
                "bridge-vlans",
                "bridge-mdb",
                "bridge-msti",
                "bridge-filter",
                "bridge-nat",
                "bridge-settings",
                "bridge-port-controller",
                "bridge-port-controller-device",
                "bridge-port-controller-port",
                "bridge-port-extender",
            ]
        );
        assert_unique_endpoints("bridge-group");
        assert!(resource_by_id("bridge-settings").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("bridge-port-controller").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("bridge-port-extender").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("bridge-hosts").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            column_keys("bridges"),
            [
                "name",
                "protocol-mode",
                "vlan-filtering",
                "pvid",
                "igmp-snooping",
                "dhcp-snooping",
                "arp",
                "mac-address",
                "mtu",
                "fast-forward",
                "frame-types",
                "ingress-filtering",
                "priority",
                "region-name",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "bridge-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn switch_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("switch-group"),
            [
                "switch",
                "switch-port",
                "switch-vlan",
                "switch-host",
                "switch-rule",
                "switch-port-isolation",
                "switch-l3hw",
            ]
        );
        assert_unique_endpoints("switch-group");
        assert!(resource_by_id("switch-l3hw").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("switch-rule").is_some_and(ResourceSpec::is_singleton));
        let tree = navigation_tree();
        let group = tree
            .iter()
            .find(|item| item.id == "switch-group")
            .expect("switch nav group");
        let child_ids: Vec<_> = group.children.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(
            child_ids,
            [
                "switch",
                "switch-port",
                "switch-vlan",
                "switch-host",
                "switch-rule",
                "switch-port-isolation",
                "switch-l3hw",
            ]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "switch-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn ip_group_covers_webfig_operator_screens() {
        assert_eq!(
            group_ids("ip-group"),
            [
                "arp",
                "addresses",
                "dhcp-servers",
                "dhcp-networks",
                "dhcp-leases",
                "dhcp-relay",
                "dhcp-options",
                "dhcp-option-sets",
                "firewall-filter",
                "neighbors",
                "dhcp-clients",
                "dns",
                "dns-static",
                "routes",
                "pools",
                "ip-services",
                "ip-settings",
                "firewall-nat",
                "firewall-mangle",
                "firewall-raw",
                "firewall-connections",
                "address-list",
                "firewall-layer7",
                "firewall-service-port",
                "ipsec-peers",
                "ipsec-identities",
                "ipsec-policies",
                "ipsec-proposals",
                "ipsec-profiles",
                "ipsec-installed-sa",
                "ipsec-settings",
                "ipsec-mode-config",
                "ipsec-key-rsa",
                "ipsec-key-psk",
                "ipsec-key-qkd",
                "cloud",
                "kid-control",
                "kid-control-devices",
                "socks",
                "smb",
                "smb-shares",
                "smb-users",
                "upnp",
                "upnp-interfaces",
                "dns-cache",
                "dhcp-alerts",
                "connection-tracking",
                "neighbor-discovery",
                "ip-ssh",
                "traffic-flow",
                "traffic-flow-targets",
                "traffic-flow-ipfix",
                "igmp-proxy",
                "igmp-proxy-interfaces",
                "igmp-proxy-mfc",
                "proxy",
                "proxy-access",
                "proxy-cache",
                "proxy-direct",
                "hotspot",
                "hotspot-profiles",
                "hotspot-users",
                "hotspot-user-profiles",
                "hotspot-cookies",
                "hotspot-hosts",
                "hotspot-ip-bindings",
                "hotspot-walled-garden",
                "hotspot-walled-garden-ip",
            ]
        );
        assert_unique_endpoints("ip-group");
        assert!(resource_by_id("dns").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("ipsec-settings").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("ipsec-key-qkd").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("ipsec-key-rsa").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            resource_by_id("ipsec-key-rsa").map(ResourceSpec::endpoint),
            Some("/ip/ipsec/key/rsa")
        );
        assert_eq!(
            resource_by_id("ipsec-key-psk").map(ResourceSpec::endpoint),
            Some("/ip/ipsec/key/psk")
        );
        assert_eq!(
            resource_by_id("ipsec-key-qkd").map(ResourceSpec::endpoint),
            Some("/ip/ipsec/key/qkd")
        );
        assert_eq!(column_keys("ipsec-key-psk"), ["peer", "id"]);
        assert!(resource_by_id("routes").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("neighbors").is_some_and(|spec| spec.form.is_none()));
        assert!(resource_by_id("ipsec-installed-sa").is_some_and(|spec| spec.form.is_none()));
        assert!(
            !column_keys("ipsec-installed-sa")
                .iter()
                .any(|key| { key.contains("key") || *key == "secret" || key.contains("auth-key") })
        );
        let connections = resource_by_id("firewall-connections").expect("firewall-connections");
        assert_eq!(connections.endpoint(), "/ip/firewall/connection");
        assert!(connections.form.is_none());
        let connection_actions: Vec<_> =
            connections.actions.iter().map(|action| action.id).collect();
        assert_eq!(connection_actions, ["remove"]);
        assert_eq!(
            column_keys("firewall-connections"),
            [
                "src-address",
                "dst-address",
                "protocol",
                "src-port",
                "dst-port",
                "tcp-state",
                "timeout",
                "orig-rate",
                "repl-rate",
                "connection-mark",
            ]
        );
    }

    #[test]
    fn new_webfig_groups_exist() {
        assert_eq!(
            group_ids("ipv6-group"),
            [
                "ipv6-addresses",
                "ipv6-neighbors",
                "ipv6-nd",
                "ipv6-nd-prefix",
                "ipv6-routes",
                "ipv6-pool",
                "ipv6-dhcp-client",
                "ipv6-dhcp-server",
                "ipv6-settings",
                "ipv6-firewall-filter",
                "ipv6-firewall-nat",
                "ipv6-address-list",
                "ipv6-dhcp-relay",
                "ipv6-dhcp-bindings",
                "ipv6-firewall-mangle",
                "ipv6-firewall-raw",
                "ipv6-firewall-connections",
            ]
        );
        assert_eq!(
            group_ids("routing-group"),
            [
                "routing-tables",
                "routing-rules",
                "ospf-instances",
                "ospf-areas",
                "ospf-interface-templates",
                "ospf-interfaces",
                "bgp-connections",
                "bgp-sessions",
                "bgp-templates",
                "rip-instances",
                "rip-interface-templates",
                "bfd",
                "routing-filters",
                "routing-id",
                "ospf-neighbors",
                "ospf-lsa",
                "bgp-advertisements",
            ]
        );
        assert_eq!(
            group_ids("queue-group"),
            [
                "queue-simple",
                "queue-tree",
                "queue-type",
                "queue-interface",
            ]
        );
        assert_eq!(group_ids("files-group"), ["files"]);
        assert_eq!(
            group_ids("tools-group"),
            [
                "netwatch",
                "email",
                "romon",
                "romon-ports",
                "graphing",
                "graphing-interface",
                "graphing-queue",
                "graphing-resource",
                "ping",
                "traceroute",
                "sniffer",
                "bandwidth-test",
                "flood-ping",
                "mac-scan",
                "ip-scan",
                "profiler",
                "wol",
                "sms",
            ]
        );
        let email = resource_by_id("email").expect("email");
        assert_eq!(email.endpoint(), "/tool/e-mail");
        assert_eq!(email.cli_path(), "/tool/e-mail");
        assert_eq!(group_ids("radius-group"), ["radius", "radius-incoming"]);
        for group in [
            "ipv6-group",
            "routing-group",
            "queue-group",
            "files-group",
            "tools-group",
            "radius-group",
        ] {
            assert_unique_endpoints(group);
        }
        let tree = navigation_tree();
        let labels: Vec<_> = tree.iter().map(|item| item.id.as_str()).collect();
        assert!(labels.contains(&"ipv6-group"));
        assert!(labels.contains(&"radius-group"));
        assert_eq!(labels.last().copied(), Some("system-group"));
    }

    #[test]
    fn container_group_exists() {
        assert_eq!(
            group_ids("container-group"),
            [
                "containers",
                "container-config",
                "container-envs",
                "container-mounts",
                "apps",
            ]
        );
        assert_unique_endpoints("container-group");
        let tree = navigation_tree();
        let labels: Vec<_> = tree.iter().map(|item| item.id.as_str()).collect();
        assert!(labels.contains(&"container-group"));
    }

    #[test]
    fn ipv6_firewall_connections_mirror_ipv4() {
        let ipv6_connections =
            resource_by_id("ipv6-firewall-connections").expect("ipv6-firewall-connections");
        assert_eq!(ipv6_connections.endpoint(), "/ipv6/firewall/connection");
        assert!(ipv6_connections.form.is_none());
        let ipv6_connection_actions: Vec<_> = ipv6_connections
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(ipv6_connection_actions, ["remove"]);
        assert_eq!(
            column_keys("ipv6-firewall-connections"),
            [
                "src-address",
                "dst-address",
                "protocol",
                "src-port",
                "dst-port",
                "tcp-state",
                "timeout",
                "orig-rate",
                "repl-rate",
                "connection-mark",
            ]
        );
        assert_eq!(
            column_keys("ipv6-firewall-connections"),
            column_keys("firewall-connections")
        );
    }

    #[test]
    fn romon_and_graphing_cover_webfig_tools() {
        assert!(resource_by_id("romon").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("graphing").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("romon-ports").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            resource_by_id("romon").expect("romon").endpoint(),
            "/tool/romon"
        );
        assert_eq!(
            resource_by_id("romon-ports").expect("ports").endpoint(),
            "/tool/romon/port"
        );
        assert_eq!(
            resource_by_id("graphing").expect("graphing").endpoint(),
            "/tool/graphing"
        );
        assert_eq!(
            resource_by_id("graphing-interface").expect("gi").endpoint(),
            "/tool/graphing/interface"
        );
        assert_eq!(
            resource_by_id("graphing-queue").expect("gq").endpoint(),
            "/tool/graphing/queue"
        );
        assert_eq!(
            resource_by_id("graphing-resource").expect("gr").endpoint(),
            "/tool/graphing/resource"
        );
        assert!(resource_by_id("romon").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("romon-ports").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("graphing").is_some_and(|spec| spec.form.is_some()));
        assert!(column_keys("romon").contains(&"secrets"));
        assert!(column_keys("romon-ports").contains(&"forbid"));
        assert_eq!(column_keys("graphing"), ["store-every", "page-refresh"]);
        let port_actions: Vec<_> = resource_by_id("romon-ports")
            .expect("ports")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            port_actions,
            ["add", "edit", "toggle-disabled", "copy", "remove"]
        );
    }

    #[test]
    fn neighbors_connect_is_overlay_without_a_form() {
        let neighbors = resource_by_id("neighbors").expect("neighbors");
        assert!(neighbors.form.is_none());
        assert_eq!(neighbors.endpoint(), "/ip/neighbor");
        assert_eq!(
            neighbors
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["connect", "remove"]
        );
        assert_eq!(
            neighbors.actions[0].kind,
            crate::actions::ActionKind::Overlay {
                id: "connect-neighbor"
            }
        );
    }

    #[test]
    fn mutations_require_forms_except_remove_only_rows() {
        use crate::actions::ActionKind;

        for spec in ALL_RESOURCES.iter() {
            if spec.actions.is_empty() {
                continue;
            }
            let ids: Vec<_> = spec.actions.iter().map(|action| action.id).collect();
            let overlay_only = spec
                .actions
                .iter()
                .all(|action| matches!(action.kind, crate::actions::ActionKind::Overlay { .. }));
            if ids == ["remove"] {
                assert!(spec.form.is_none(), "{} should be remove-only", spec.id);
                continue;
            }
            if spec.id == "files" {
                assert!(
                    spec.form.is_none(),
                    "files uses transfer prompts, not a sheet"
                );
                continue;
            }
            if spec.id == "history" {
                assert!(
                    spec.form.is_none(),
                    "history uses undo confirm, not a sheet"
                );
                assert!(
                    spec.actions.iter().any(|action| action.id == "undo"),
                    "history should offer undo"
                );
                continue;
            }
            if overlay_only {
                assert!(
                    spec.form.is_none(),
                    "{} is overlay-only and should not have a form",
                    spec.id
                );
                continue;
            }
            let needs_sheet = spec
                .actions
                .iter()
                .any(|action| matches!(action.kind, ActionKind::Edit | ActionKind::Create));
            if needs_sheet {
                assert!(spec.form.is_some(), "{} needs a property sheet", spec.id);
            }
        }
        assert!(resource_by_id("logs").is_some_and(|spec| spec.actions.is_empty()));
        assert!(
            resource_by_id("routerboard")
                .is_some_and(|spec| spec.actions.iter().any(|action| action.id == "upgrade"))
        );
        assert!(
            resource_by_id("resources")
                .is_some_and(|spec| spec.form.is_none() && spec.actions.is_empty())
        );
        assert!(resource_by_id("reboot").is_some_and(|spec| spec.cli_path() == "/system/reboot"));
        assert!(
            resource_by_id("files").is_some_and(|spec| spec.form.is_none()
                && spec.actions.iter().any(|action| action.id == "backup-save")
                && spec.actions.iter().any(|action| action.id == "fetch"))
        );
    }

    #[test]
    fn history_is_undo_confirm_with_no_property_sheet() {
        use crate::actions::{ActionCommand, ActionKind};

        let spec = resource_by_id("history").expect("history");
        assert!(spec.form.is_none(), "history must not ship a Text sheet");
        assert_eq!(spec.endpoint(), "/system/history");
        assert_eq!(spec.actions.len(), 1);
        let undo = spec.actions[0];
        assert_eq!(undo.id, "undo");
        assert_eq!(undo.label, "Undo");
        assert_eq!(undo.key, Some('u'));
        assert!(undo.danger);
        assert!(undo.needs_selection);
        assert!(matches!(
            undo.kind,
            ActionKind::Confirm {
                command: ActionCommand::Undo
            }
        ));
        assert_eq!(
            column_keys("history"),
            ["floating-undo", "time", "action", "by", "policy"]
        );
    }

    fn group_ids(group: &str) -> Vec<&'static str> {
        ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group)
            .map(|spec| spec.id)
            .collect()
    }

    fn assert_unique_endpoints(group: &str) {
        let endpoints: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group)
            .filter(|spec| !matches!(spec.fetch, FetchKind::Local))
            .map(ResourceSpec::endpoint)
            .collect();
        let mut unique = endpoints.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), endpoints.len());
    }

    fn column_keys(id: &str) -> Vec<&'static str> {
        resource_by_id(id)
            .expect("catalog resource")
            .columns
            .iter()
            .map(|col| col.key)
            .collect()
    }
}
