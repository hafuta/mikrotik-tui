//! Feature-owned catalog entries for the `IPv6` navigation group.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        crate::resources::ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

pub const IPV6_ADDRESSES: ResourceSpec = ResourceSpec {
    id: "ipv6-addresses",
    group: "ipv6-group",
    cli_path: None,
    label: "Addresses",
    fetch: FetchKind::List {
        endpoint: "/ipv6/address",
    },
    columns: &[
        col!("address", "Address", 28),
        col!("interface", "Interface", 16),
        col!("advertise", "Adv", 5),
        col!("eui-64", "EUI-64", 7),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_ADDRESS_FORM),
};

pub const IPV6_NEIGHBORS: ResourceSpec = ResourceSpec {
    id: "ipv6-neighbors",
    group: "ipv6-group",
    cli_path: None,
    label: "Neighbors",
    fetch: FetchKind::List {
        endpoint: "/ipv6/neighbor",
    },
    columns: &[
        col!("address", "Address", 28),
        col!("interface", "Interface", 16),
        col!("mac-address", "MAC address", 18),
        col!("status", "Status", 12),
        col!("origin", "Origin", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_NEIGHBOR_FORM),
};

pub const IPV6_ND: ResourceSpec = ResourceSpec {
    id: "ipv6-nd",
    group: "ipv6-group",
    cli_path: None,
    label: "ND",
    fetch: FetchKind::List {
        endpoint: "/ipv6/nd",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("ra-interval", "RA interval", 12),
        col!("advertise-mac-address", "Adv MAC", 8),
        col!("advertise-dns", "Adv DNS", 8),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_ND_FORM),
};

pub const IPV6_ND_PREFIX: ResourceSpec = ResourceSpec {
    id: "ipv6-nd-prefix",
    group: "ipv6-group",
    cli_path: None,
    label: "ND Prefix",
    fetch: FetchKind::List {
        endpoint: "/ipv6/nd/prefix",
    },
    columns: &[
        col!("prefix", "Prefix", 28),
        col!("interface", "Interface", 16),
        col!("advertise", "Adv", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_ND_PREFIX_FORM),
};

pub const IPV6_ROUTES: ResourceSpec = ResourceSpec {
    id: "ipv6-routes",
    group: "ipv6-group",
    cli_path: None,
    label: "Routes",
    fetch: FetchKind::List {
        endpoint: "/ipv6/route",
    },
    columns: &[
        col!("dst-address", "Dst", 28),
        col!("gateway", "Gateway", 28),
        col!("distance", "Dist", 6),
        col!("routing-table", "Table", 12),
        col!("active", "Act", 5),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_ROUTE_FORM),
};

pub const IPV6_POOL: ResourceSpec = ResourceSpec {
    id: "ipv6-pool",
    group: "ipv6-group",
    cli_path: None,
    label: "Pool",
    fetch: FetchKind::List {
        endpoint: "/ipv6/pool",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("prefix", "Prefix", 28),
        col!("prefix-length", "Len", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_POOL_FORM),
};

pub const IPV6_DHCP_CLIENT: ResourceSpec = ResourceSpec {
    id: "ipv6-dhcp-client",
    group: "ipv6-group",
    cli_path: None,
    label: "DHCP Client",
    fetch: FetchKind::List {
        endpoint: "/ipv6/dhcp-client",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("status", "Status", 12),
        col!("prefix", "Prefix", 28),
        col!("pool-name", "Pool", 16),
        col!("expires-after", "Expires", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_DHCP_CLIENT_FORM),
};

pub const IPV6_DHCP_SERVER: ResourceSpec = ResourceSpec {
    id: "ipv6-dhcp-server",
    group: "ipv6-group",
    cli_path: None,
    label: "DHCP Server",
    fetch: FetchKind::List {
        endpoint: "/ipv6/dhcp-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("address-pool", "Pool", 18),
        col!("lease-time", "Lease time", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_DHCP_SERVER_FORM),
};

pub const IPV6_SETTINGS: ResourceSpec = ResourceSpec {
    id: "ipv6-settings",
    group: "ipv6-group",
    cli_path: None,
    label: "Settings",
    fetch: FetchKind::System {
        endpoint: "/ipv6/settings",
    },
    columns: &[
        col!("forward", "Forward", 8),
        col!("accept-redirects", "Redirects", 10),
        col!("max-neighbor-entries", "ND max", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_SETTINGS_FORM),
};

pub const IPV6_FIREWALL_FILTER: ResourceSpec = ResourceSpec {
    id: "ipv6-firewall-filter",
    group: "ipv6-group",
    cli_path: None,
    label: "Firewall",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/filter",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 12),
        col!("src-address", "Source", 24),
        col!("dst-address", "Destination", 24),
        col!("protocol", "Protocol", 9),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("packets", "Packets", 12),
        col!("bytes", "Bytes", 14),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_FIREWALL_FILTER_FORM),
};

pub const IPV6_FIREWALL_NAT: ResourceSpec = ResourceSpec {
    id: "ipv6-firewall-nat",
    group: "ipv6-group",
    cli_path: None,
    label: "NAT",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/nat",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 14),
        col!("protocol", "Protocol", 9),
        col!("src-address", "Source", 24),
        col!("dst-address", "Destination", 24),
        col!("to-addresses", "To addr", 24),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_FIREWALL_NAT_FORM),
};

pub const IPV6_ADDRESS_LIST: ResourceSpec = ResourceSpec {
    id: "ipv6-address-list",
    group: "ipv6-group",
    cli_path: None,
    label: "Address List",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/address-list",
    },
    columns: &[
        col!("list", "List", 16),
        col!("address", "Address", 28),
        col!("timeout", "Timeout", 12),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_ADDRESS_LIST_FORM),
};

pub const IPV6_DHCP_RELAY: ResourceSpec = ResourceSpec {
    id: "ipv6-dhcp-relay",
    group: "ipv6-group",
    cli_path: None,
    label: "DHCP Relay",
    fetch: FetchKind::List {
        endpoint: "/ipv6/dhcp-relay",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("dhcp-server", "Server", 24),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_DHCP_RELAY_FORM),
};

pub const IPV6_DHCP_BINDINGS: ResourceSpec = ResourceSpec {
    id: "ipv6-dhcp-bindings",
    group: "ipv6-group",
    cli_path: None,
    label: "DHCP Bindings",
    fetch: FetchKind::List {
        endpoint: "/ipv6/dhcp-server/binding",
    },
    columns: &[
        col!("address", "Address", 28),
        col!("duid", "DUID", 20),
        col!("server", "Server", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::LEASE_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_DHCP_BINDING_FORM),
};

pub const IPV6_FIREWALL_MANGLE: ResourceSpec = ResourceSpec {
    id: "ipv6-firewall-mangle",
    group: "ipv6-group",
    cli_path: None,
    label: "Mangle",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/mangle",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 12),
        col!("src-address", "Source", 24),
        col!("dst-address", "Destination", 24),
        col!("packets", "Packets", 12),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_FIREWALL_MANGLE_FORM),
};

pub const IPV6_FIREWALL_RAW: ResourceSpec = ResourceSpec {
    id: "ipv6-firewall-raw",
    group: "ipv6-group",
    cli_path: None,
    label: "Raw",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/raw",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 12),
        col!("src-address", "Source", 24),
        col!("dst-address", "Destination", 24),
        col!("packets", "Packets", 12),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ipv6::forms::IPV6_FIREWALL_RAW_FORM),
};

pub const IPV6_FIREWALL_CONNECTIONS: ResourceSpec = ResourceSpec {
    id: "ipv6-firewall-connections",
    group: "ipv6-group",
    cli_path: None,
    label: "Connections",
    fetch: FetchKind::List {
        endpoint: "/ipv6/firewall/connection",
    },
    columns: &[
        col!("src-address", "Source", 28),
        col!("dst-address", "Destination", 28),
        col!("protocol", "Protocol", 9),
        col!("src-port", "Src port", 10),
        col!("dst-port", "Dst port", 10),
        col!("tcp-state", "TCP state", 12),
        col!("timeout", "Timeout", 10),
        col!("orig-rate", "Orig rate", 12),
        col!("repl-rate", "Repl rate", 12),
        col!("connection-mark", "Mark", 16),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    IPV6_ADDRESSES,
    IPV6_NEIGHBORS,
    IPV6_ND,
    IPV6_ND_PREFIX,
    IPV6_ROUTES,
    IPV6_POOL,
    IPV6_DHCP_CLIENT,
    IPV6_DHCP_SERVER,
    IPV6_SETTINGS,
    IPV6_FIREWALL_FILTER,
    IPV6_FIREWALL_NAT,
    IPV6_ADDRESS_LIST,
    IPV6_DHCP_RELAY,
    IPV6_DHCP_BINDINGS,
    IPV6_FIREWALL_MANGLE,
    IPV6_FIREWALL_RAW,
    IPV6_FIREWALL_CONNECTIONS,
];
