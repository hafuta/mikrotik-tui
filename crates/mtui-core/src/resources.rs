//! Descriptor-driven `RouterOS` resource catalog and navigation tree.

use std::time::Duration;

/// Dashboard nav / content id (not a REST list resource).
pub const DASHBOARD_ID: &str = "dashboard";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchKind {
    /// List-like `/rest/...` collection.
    List { endpoint: &'static str },
    /// Singleton system resource.
    System { endpoint: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub fetch: FetchKind,
    pub columns: &'static [ColumnSpec],
    pub refresh: Duration,
}

impl ResourceSpec {
    #[must_use]
    pub fn endpoint(&self) -> &'static str {
        match self.fetch {
            FetchKind::List { endpoint } | FetchKind::System { endpoint } => endpoint,
        }
    }

    #[must_use]
    pub fn cli_path(&self) -> &str {
        self.endpoint().trim_start_matches("/rest")
    }

    #[must_use]
    pub fn is_singleton(&self) -> bool {
        matches!(self.fetch, FetchKind::System { .. })
    }
}

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

pub static ALL_RESOURCES: &[ResourceSpec] = &[
    ResourceSpec {
        id: "interfaces",
        group: "interfaces-group",
        label: "Interface",
        fetch: FetchKind::List {
            endpoint: "/rest/interface",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("type", "Type", 12),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("mtu", "MTU", 7),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "interface-lists",
        group: "interfaces-group",
        label: "Interface List",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/list",
        },
        columns: &[col!("name", "Name", 20), col!("comment", "Comment", 28)],
        refresh: Duration::from_secs(30),
    },
    ResourceSpec {
        id: "ethernet",
        group: "interfaces-group",
        label: "Ethernet",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("mac-address", "MAC address", 18),
            col!("speed", "Speed", 12),
            col!("full-duplex", "Duplex", 8),
            col!("running", "Run", 5),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "ppp-active",
        group: "ppp-group",
        label: "Active",
        fetch: FetchKind::List {
            endpoint: "/rest/ppp/active",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("service", "Service", 10),
            col!("caller-id", "Caller", 18),
            col!("address", "Address", 18),
            col!("uptime", "Uptime", 12),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "pppoe-clients",
        group: "ppp-group",
        label: "PPPoE Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pppoe-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("user", "User", 18),
            col!("status", "Status", 12),
            col!("running", "Run", 5),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "bridges",
        group: "bridge-group",
        label: "Bridge",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("protocol-mode", "Protocol", 12),
            col!("vlan-filtering", "VLAN", 6),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(10),
    },
    ResourceSpec {
        id: "bridge-ports",
        group: "bridge-group",
        label: "Ports",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/port",
        },
        columns: &[
            col!("interface", "Interface", 18),
            col!("bridge", "Bridge", 18),
            col!("pvid", "PVID", 7),
            col!("role", "Role", 12),
            col!("hw", "HW", 4),
        ],
        refresh: Duration::from_secs(10),
    },
    ResourceSpec {
        id: "bridge-vlans",
        group: "bridge-group",
        label: "VLANs",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/vlan",
        },
        columns: &[
            col!("bridge", "Bridge", 16),
            col!("vlan-ids", "VLAN IDs", 14),
            col!("tagged", "Tagged", 24),
            col!("untagged", "Untagged", 24),
        ],
        refresh: Duration::from_secs(15),
    },
    ResourceSpec {
        id: "arp",
        group: "ip-group",
        label: "ARP",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/arp",
        },
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC address", 18),
            col!("interface", "Interface", 16),
            col!("status", "Status", 12),
            col!("dynamic", "Dyn", 5),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "addresses",
        group: "ip-group",
        label: "Addresses",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/address",
        },
        columns: &[
            col!("address", "Address", 20),
            col!("network", "Network", 18),
            col!("interface", "Interface", 16),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
    },
    ResourceSpec {
        id: "dhcp-servers",
        group: "ip-group",
        label: "DHCP",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("address-pool", "Pool", 18),
            col!("lease-time", "Lease time", 12),
            col!("status", "Status", 10),
        ],
        refresh: Duration::from_secs(10),
    },
    ResourceSpec {
        id: "dhcp-networks",
        group: "ip-group",
        label: "Networks",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/network",
        },
        columns: &[
            col!("address", "Network", 20),
            col!("gateway", "Gateway", 18),
            col!("dns-server", "DNS", 24),
            col!("domain", "Domain", 18),
        ],
        refresh: Duration::from_secs(30),
    },
    ResourceSpec {
        id: "dhcp-leases",
        group: "ip-group",
        label: "Leases",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/lease",
        },
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC address", 18),
            col!("host-name", "Hostname", 20),
            col!("status", "Status", 10),
            col!("expires-after", "Expires", 12),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "firewall-filter",
        group: "ip-group",
        label: "Firewall",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/filter",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 20),
            col!("src-port", "Src port", 10),
            col!("dst-address", "Destination", 20),
            col!("dst-port", "Dst port", 10),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("dynamic", "Dyn", 5),
            col!("invalid", "Bad", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
    },
    ResourceSpec {
        id: "users",
        group: "system-group",
        label: "Users",
        fetch: FetchKind::List {
            endpoint: "/rest/user",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("group", "Group", 14),
            col!("last-logged-in", "Last login", 22),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
    },
    ResourceSpec {
        id: "routerboard",
        group: "system-group",
        label: "RouterBOARD",
        fetch: FetchKind::System {
            endpoint: "/rest/system/routerboard",
        },
        columns: &[
            col!("model", "Model", 18),
            col!("serial-number", "Serial", 18),
            col!("current-firmware", "Current", 12),
            col!("upgrade-firmware", "Upgrade", 12),
        ],
        refresh: Duration::from_secs(60),
    },
    ResourceSpec {
        id: "ntp",
        group: "system-group",
        label: "NTP Client",
        fetch: FetchKind::System {
            endpoint: "/rest/system/ntp/client",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("mode", "Mode", 12),
            col!("servers", "Servers", 28),
            col!("status", "Status", 12),
        ],
        refresh: Duration::from_secs(30),
    },
    ResourceSpec {
        id: "clock",
        group: "system-group",
        label: "Clock",
        fetch: FetchKind::System {
            endpoint: "/rest/system/clock",
        },
        columns: &[
            col!("time", "Time", 12),
            col!("date", "Date", 14),
            col!("time-zone-name", "Time zone", 22),
            col!("gmt-offset", "Offset", 10),
        ],
        refresh: Duration::from_secs(10),
    },
    ResourceSpec {
        id: "logs",
        group: "system-group",
        label: "Logs",
        fetch: FetchKind::List {
            endpoint: "/rest/log",
        },
        columns: &[
            col!("time", "Time", 19),
            col!("topics", "Topics", 24),
            col!("message", "Message", 72),
        ],
        refresh: Duration::from_secs(1),
    },
];

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
        id: "ppp-group",
        label: "PPP",
    },
    NavGroup {
        id: "bridge-group",
        label: "Bridge",
    },
    NavGroup {
        id: "ip-group",
        label: "IP",
    },
    NavGroup {
        id: "system-group",
        label: "System",
    },
];

#[must_use]
pub fn resource_by_id(id: &str) -> Option<&'static ResourceSpec> {
    ALL_RESOURCES.iter().find(|spec| spec.id == id)
}

#[must_use]
pub fn navigation_tree() -> Vec<NavItem> {
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
    }

    #[test]
    fn navigation_includes_dashboard_and_logs() {
        let tree = navigation_tree();
        assert_eq!(tree[0].id, DASHBOARD_ID);
        assert!(resource_by_id("logs").is_some());
        assert!(resource_by_id("firewall-filter").is_some());
    }
}
