//! Feature-owned catalog entries for the complete IP navigation group.

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

pub const ARP: ResourceSpec = ResourceSpec {
    id: "arp",
    group: "ip-group",
    cli_path: None,
    label: "ARP",
    fetch: FetchKind::List {
        endpoint: "/ip/arp",
    },
    columns: &[
        col!("address", "Address", 18),
        col!("mac-address", "MAC address", 18),
        col!("interface", "Interface", 16),
        col!("status", "Status", 12),
        col!("dynamic", "Dyn", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::ARP_ACTIONS,
    form: Some(&crate::features::ip::forms::ARP_FORM),
};

pub const ADDRESSES: ResourceSpec = ResourceSpec {
    id: "addresses",
    group: "ip-group",
    cli_path: None,
    label: "Addresses",
    fetch: FetchKind::List {
        endpoint: "/ip/address",
    },
    columns: &[
        col!("address", "Address", 20),
        col!("network", "Network", 18),
        col!("interface", "Interface", 16),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::ADDRESS_FORM),
};

pub const DHCP_SERVERS: ResourceSpec = ResourceSpec {
    id: "dhcp-servers",
    group: "ip-group",
    cli_path: None,
    label: "DHCP",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("address-pool", "Pool", 18),
        col!("lease-time", "Lease time", 12),
        col!("status", "Status", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_SERVER_FORM),
};

pub const DHCP_NETWORKS: ResourceSpec = ResourceSpec {
    id: "dhcp-networks",
    group: "ip-group",
    cli_path: None,
    label: "Networks",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server/network",
    },
    columns: &[
        col!("address", "Network", 20),
        col!("gateway", "Gateway", 18),
        col!("dns-server", "DNS", 24),
        col!("domain", "Domain", 18),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_NETWORK_FORM),
};

pub const DHCP_LEASES: ResourceSpec = ResourceSpec {
    id: "dhcp-leases",
    group: "ip-group",
    cli_path: None,
    label: "Leases",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server/lease",
    },
    columns: &[
        col!("address", "Address", 18),
        col!("mac-address", "MAC address", 18),
        col!("host-name", "Hostname", 20),
        col!("status", "Status", 10),
        col!("expires-after", "Expires", 12),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::LEASE_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_LEASE_FORM),
};

pub const DHCP_RELAY: ResourceSpec = ResourceSpec {
    id: "dhcp-relay",
    group: "ip-group",
    cli_path: None,
    label: "DHCP Relay",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-relay",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("dhcp-server", "DHCP server", 24),
        col!("local-address", "Local", 18),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_RELAY_FORM),
};

pub const DHCP_OPTIONS: ResourceSpec = ResourceSpec {
    id: "dhcp-options",
    group: "ip-group",
    cli_path: None,
    label: "DHCP Options",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server/option",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("code", "Code", 6),
        col!("value", "Value", 28),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_OPTION_FORM),
};

pub const DHCP_OPTION_SETS: ResourceSpec = ResourceSpec {
    id: "dhcp-option-sets",
    group: "ip-group",
    cli_path: None,
    label: "DHCP Option Sets",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server/option/sets",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("options", "Options", 36),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_OPTION_SET_FORM),
};

pub const FIREWALL_FILTER: ResourceSpec = ResourceSpec {
    id: "firewall-filter",
    group: "ip-group",
    cli_path: None,
    label: "Firewall",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/filter",
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
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ip::forms::FIREWALL_FILTER_FORM),
};

pub const NEIGHBORS: ResourceSpec = ResourceSpec {
    id: "neighbors",
    group: "ip-group",
    cli_path: None,
    label: "Neighbors",
    fetch: FetchKind::List {
        endpoint: "/ip/neighbor",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("address", "Address", 18),
        col!("mac-address", "MAC address", 18),
        col!("identity", "Identity", 20),
        col!("platform", "Platform", 14),
        col!("version", "Version", 14),
        col!("interface-name", "If name", 16),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::NEIGHBOR_ACTIONS,
    form: None,
};

pub const DHCP_CLIENTS: ResourceSpec = ResourceSpec {
    id: "dhcp-clients",
    group: "ip-group",
    cli_path: None,
    label: "DHCP Client",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-client",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("status", "Status", 12),
        col!("address", "Address", 20),
        col!("gateway", "Gateway", 18),
        col!("dhcp-server", "Server", 18),
        col!("add-default-route", "Default", 8),
        col!("use-peer-dns", "Peer DNS", 9),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_CLIENT_FORM),
};

pub const DNS: ResourceSpec = ResourceSpec {
    id: "dns",
    group: "ip-group",
    cli_path: None,
    label: "DNS",
    fetch: FetchKind::System {
        endpoint: "/ip/dns",
    },
    columns: &[
        col!("servers", "Servers", 28),
        col!("allow-remote-requests", "Remote", 8),
        col!("cache-size", "Cache", 10),
        col!("cache-max-ttl", "Max TTL", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::DNS_FORM),
};

pub const DNS_STATIC: ResourceSpec = ResourceSpec {
    id: "dns-static",
    group: "ip-group",
    cli_path: None,
    label: "Static DNS",
    fetch: FetchKind::List {
        endpoint: "/ip/dns/static",
    },
    columns: &[
        col!("name", "Name", 24),
        col!("address", "Address", 18),
        col!("type", "Type", 8),
        col!("ttl", "TTL", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::DNS_STATIC_FORM),
};

pub const ROUTES: ResourceSpec = ResourceSpec {
    id: "routes",
    group: "ip-group",
    cli_path: None,
    label: "Routes",
    fetch: FetchKind::List {
        endpoint: "/ip/route",
    },
    columns: &[
        col!("dst-address", "Dst", 20),
        col!("gateway", "Gateway", 18),
        col!("distance", "Dist", 6),
        col!("routing-table", "Table", 12),
        col!("active", "Act", 5),
        col!("static", "Static", 7),
        col!("dynamic", "Dyn", 5),
        col!("unreachable", "Unreach", 8),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::ROUTE_FORM),
};

pub const POOLS: ResourceSpec = ResourceSpec {
    id: "pools",
    group: "ip-group",
    cli_path: None,
    label: "Pool",
    fetch: FetchKind::List {
        endpoint: "/ip/pool",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("ranges", "Ranges", 36),
        col!("next-pool", "Next", 18),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::POOL_FORM),
};

pub const IP_SERVICES: ResourceSpec = ResourceSpec {
    id: "ip-services",
    group: "ip-group",
    cli_path: None,
    label: "Services",
    fetch: FetchKind::List {
        endpoint: "/ip/service",
    },
    columns: &[
        col!("name", "Name", 14),
        col!("port", "Port", 6),
        col!("address", "Address", 20),
        col!("certificate", "Certificate", 18),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::TOGGLE_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::SERVICE_FORM),
};

pub const IP_SETTINGS: ResourceSpec = ResourceSpec {
    id: "ip-settings",
    group: "ip-group",
    cli_path: None,
    label: "Settings",
    fetch: FetchKind::System {
        endpoint: "/ip/settings",
    },
    columns: &[
        col!("ip-forward", "Forward", 8),
        col!("rp-filter", "RP filter", 12),
        col!("tcp-syncookies", "Syncookies", 11),
        col!("accept-redirects", "Accept redir", 13),
        col!("send-redirects", "Send redir", 11),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::IP_SETTINGS_FORM),
};

pub const FIREWALL_NAT: ResourceSpec = ResourceSpec {
    id: "firewall-nat",
    group: "ip-group",
    cli_path: None,
    label: "NAT",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/nat",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 14),
        col!("protocol", "Protocol", 9),
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("dst-port", "Dst port", 10),
        col!("to-addresses", "To addr", 20),
        col!("to-ports", "To ports", 10),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("packets", "Packets", 12),
        col!("bytes", "Bytes", 14),
        col!("disabled", "Off", 5),
        col!("dynamic", "Dyn", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ip::forms::FIREWALL_NAT_FORM),
};

pub const FIREWALL_MANGLE: ResourceSpec = ResourceSpec {
    id: "firewall-mangle",
    group: "ip-group",
    cli_path: None,
    label: "Mangle",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/mangle",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 14),
        col!("protocol", "Protocol", 9),
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("new-routing-mark", "Mark", 16),
        col!("packets", "Packets", 12),
        col!("bytes", "Bytes", 14),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ip::forms::FIREWALL_MANGLE_FORM),
};

pub const FIREWALL_RAW: ResourceSpec = ResourceSpec {
    id: "firewall-raw",
    group: "ip-group",
    cli_path: None,
    label: "Raw",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/raw",
    },
    columns: &[
        col!("chain", "Chain", 12),
        col!("action", "Action", 14),
        col!("protocol", "Protocol", 9),
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
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
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ip::forms::FIREWALL_RAW_FORM),
};

pub const FIREWALL_CONNECTIONS: ResourceSpec = ResourceSpec {
    id: "firewall-connections",
    group: "ip-group",
    cli_path: None,
    label: "Connections",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/connection",
    },
    columns: &[
        col!("src-address", "Source", 22),
        col!("dst-address", "Destination", 22),
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

pub const ADDRESS_LIST: ResourceSpec = ResourceSpec {
    id: "address-list",
    group: "ip-group",
    cli_path: None,
    label: "Address List",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/address-list",
    },
    columns: &[
        col!("list", "List", 16),
        col!("address", "Address", 20),
        col!("timeout", "Timeout", 12),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::ADDRESS_LIST_FORM),
};

pub const FIREWALL_LAYER7: ResourceSpec = ResourceSpec {
    id: "firewall-layer7",
    group: "ip-group",
    cli_path: None,
    label: "Layer7 Protocol",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/layer7-protocol",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("regexp", "Regexp", 36),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::LAYER7_FORM),
};

pub const FIREWALL_SERVICE_PORT: ResourceSpec = ResourceSpec {
    id: "firewall-service-port",
    group: "ip-group",
    cli_path: None,
    label: "Service Port",
    fetch: FetchKind::List {
        endpoint: "/ip/firewall/service-port",
    },
    columns: &[
        col!("name", "Name", 14),
        col!("ports", "Ports", 20),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::TOGGLE_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::SERVICE_PORT_FORM),
};

pub const IPSEC_PEERS: ResourceSpec = ResourceSpec {
    id: "ipsec-peers",
    group: "ip-group",
    cli_path: None,
    label: "Peers",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/peer",
    },
    columns: &[
        col!("name", "Name", 16),
        col!("address", "Address", 20),
        col!("profile", "Profile", 14),
        col!("exchange-mode", "Exchange", 12),
        col!("passive", "Passive", 8),
        col!("send-initial-contact", "Init contact", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_PEER_FORM),
};

pub const IPSEC_IDENTITIES: ResourceSpec = ResourceSpec {
    id: "ipsec-identities",
    group: "ip-group",
    cli_path: None,
    label: "Identities",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/identity",
    },
    columns: &[
        col!("peer", "Peer", 16),
        col!("auth-method", "Auth", 16),
        col!("my-id", "My ID", 18),
        col!("remote-id", "Remote ID", 18),
        col!("generate-policy", "Gen policy", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_IDENTITY_FORM),
};

pub const IPSEC_POLICIES: ResourceSpec = ResourceSpec {
    id: "ipsec-policies",
    group: "ip-group",
    cli_path: None,
    label: "Policies",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/policy",
    },
    columns: &[
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("src-port", "Src port", 10),
        col!("dst-port", "Dst port", 10),
        col!("protocol", "Protocol", 9),
        col!("action", "Action", 10),
        col!("level", "Level", 10),
        col!("ipsec-protocols", "Protocols", 12),
        col!("proposal", "Proposal", 14),
        col!("peer", "Peer", 16),
        col!("tunnel", "Tunnel", 7),
        col!("sa-src-address", "SA src", 18),
        col!("sa-dst-address", "SA dst", 18),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_POLICY_FORM),
};

pub const IPSEC_PROPOSALS: ResourceSpec = ResourceSpec {
    id: "ipsec-proposals",
    group: "ip-group",
    cli_path: None,
    label: "Proposals",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/proposal",
    },
    columns: &[
        col!("name", "Name", 16),
        col!("auth-algorithms", "Auth", 18),
        col!("enc-algorithms", "Enc", 22),
        col!("pfs-group", "PFS", 10),
        col!("lifetime", "Lifetime", 12),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_PROPOSAL_FORM),
};

pub const IPSEC_PROFILES: ResourceSpec = ResourceSpec {
    id: "ipsec-profiles",
    group: "ip-group",
    cli_path: None,
    label: "Profiles",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/profile",
    },
    columns: &[
        col!("name", "Name", 16),
        col!("hash-algorithm", "Hash", 12),
        col!("enc-algorithm", "Enc", 16),
        col!("dh-group", "DH", 12),
        col!("proposal-check", "Check", 12),
        col!("lifetime", "Lifetime", 12),
        col!("nat-traversal", "NAT-T", 6),
        col!("dpd-interval", "DPD", 10),
        col!("dpd-maximum-failures", "DPD max", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_PROFILE_FORM),
};

pub const IPSEC_INSTALLED_SA: ResourceSpec = ResourceSpec {
    id: "ipsec-installed-sa",
    group: "ip-group",
    cli_path: None,
    label: "Installed SAs",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/installed-sa",
    },
    columns: &[
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("spi", "SPI", 12),
        col!("auth-algorithm", "Auth", 12),
        col!("enc-algorithm", "Enc", 14),
        col!("state", "State", 10),
        col!("current-bytes", "Bytes", 12),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::IPSEC_SA_ACTIONS,
    form: None,
};

pub const IPSEC_SETTINGS: ResourceSpec = ResourceSpec {
    id: "ipsec-settings",
    group: "ip-group",
    cli_path: None,
    label: "IPsec Settings",
    fetch: FetchKind::System {
        endpoint: "/ip/ipsec/settings",
    },
    columns: &[
        col!("accounting", "Accounting", 11),
        col!("interim-update", "Interim", 12),
        col!("xauth-use-radius", "XAuth RADIUS", 13),
        col!("uniq-id-accounting", "Uniq-id acct", 13),
        col!("identities-matching", "ID match", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_SETTINGS_FORM),
};

pub const IPSEC_MODE_CONFIG: ResourceSpec = ResourceSpec {
    id: "ipsec-mode-config",
    group: "ip-group",
    cli_path: None,
    label: "Mode Config",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/mode-config",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("address-pool", "Pool", 16),
        col!("address-prefix-length", "Prefix", 8),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_MODE_CONFIG_FORM),
};

pub const IPSEC_KEY_RSA: ResourceSpec = ResourceSpec {
    id: "ipsec-key-rsa",
    group: "ip-group",
    cli_path: None,
    label: "IPsec RSA Keys",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/key/rsa",
    },
    columns: &[col!("name", "Name", 18), col!("key-size", "Size", 8)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_KEY_RSA_FORM),
};

pub const IPSEC_KEY_PSK: ResourceSpec = ResourceSpec {
    id: "ipsec-key-psk",
    group: "ip-group",
    cli_path: None,
    label: "IPsec PSKs",
    fetch: FetchKind::List {
        endpoint: "/ip/ipsec/key/psk",
    },
    columns: &[col!("peer", "Peer", 18), col!("id", "ID", 24)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_KEY_PSK_FORM),
};

pub const IPSEC_KEY_QKD: ResourceSpec = ResourceSpec {
    id: "ipsec-key-qkd",
    group: "ip-group",
    cli_path: None,
    label: "IPsec QKD",
    fetch: FetchKind::System {
        endpoint: "/ip/ipsec/key/qkd",
    },
    columns: &[
        col!("address", "Address", 22),
        col!("kme-id", "KME ID", 16),
        col!("peer-sae-id", "Peer SAE", 16),
        col!("key-size", "Size", 8),
        col!("cache-state", "Cache", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::IPSEC_KEY_QKD_FORM),
};

pub const CLOUD: ResourceSpec = ResourceSpec {
    id: "cloud",
    group: "ip-group",
    cli_path: None,
    label: "Cloud",
    fetch: FetchKind::System {
        endpoint: "/ip/cloud",
    },
    columns: &[
        col!("ddns-enabled", "DDNS", 6),
        col!("dns-name", "DNS name", 28),
        col!("public-address", "Public", 18),
        col!("status", "Status", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::CLOUD_FORM),
};

pub const KID_CONTROL: ResourceSpec = ResourceSpec {
    id: "kid-control",
    group: "ip-group",
    cli_path: None,
    label: "Kid Control",
    fetch: FetchKind::List {
        endpoint: "/ip/kid-control",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("mon-fri", "Mon-Fri", 16),
        col!("rate-limit", "Rate", 12),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::KID_CONTROL_FORM),
};

pub const KID_CONTROL_DEVICES: ResourceSpec = ResourceSpec {
    id: "kid-control-devices",
    group: "ip-group",
    cli_path: None,
    label: "Kid Devices",
    fetch: FetchKind::List {
        endpoint: "/ip/kid-control/device",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("mac-address", "MAC", 18),
        col!("user", "User", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::KID_CONTROL_DEVICE_FORM),
};

pub const SOCKS: ResourceSpec = ResourceSpec {
    id: "socks",
    group: "ip-group",
    cli_path: None,
    label: "SOCKS",
    fetch: FetchKind::System {
        endpoint: "/ip/socks",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("port", "Port", 6),
        col!("connection-idle-timeout", "Idle", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::SOCKS_FORM),
};

pub const SMB: ResourceSpec = ResourceSpec {
    id: "smb",
    group: "ip-group",
    cli_path: None,
    label: "SMB",
    fetch: FetchKind::System {
        endpoint: "/ip/smb",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("domain", "Domain", 16),
        col!("allow-guests", "Guests", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::SMB_FORM),
};

pub const SMB_SHARES: ResourceSpec = ResourceSpec {
    id: "smb-shares",
    group: "ip-group",
    cli_path: None,
    label: "SMB Shares",
    fetch: FetchKind::List {
        endpoint: "/ip/smb/shares",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("directory", "Directory", 24),
        col!("valid-users", "Valid Users", 16),
        col!("invalid-users", "Invalid Users", 16),
        col!("read-only", "Read Only", 10),
        col!("require-encryption", "Require Encryption", 18),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::SMB_SHARE_FORM),
};

pub const SMB_USERS: ResourceSpec = ResourceSpec {
    id: "smb-users",
    group: "ip-group",
    cli_path: None,
    label: "SMB Users",
    fetch: FetchKind::List {
        endpoint: "/ip/smb/users",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("password", "Password", 10),
        col!("read-only", "Read Only", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::SMB_USER_FORM),
};

pub const UPNP: ResourceSpec = ResourceSpec {
    id: "upnp",
    group: "ip-group",
    cli_path: None,
    label: "UPnP",
    fetch: FetchKind::System {
        endpoint: "/ip/upnp",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("allow-disable-external-interface", "Allow WAN", 10),
        col!("show-dummy-rule", "Dummy", 7),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::UPNP_FORM),
};

pub const UPNP_INTERFACES: ResourceSpec = ResourceSpec {
    id: "upnp-interfaces",
    group: "ip-group",
    cli_path: None,
    label: "UPnP Interfaces",
    fetch: FetchKind::List {
        endpoint: "/ip/upnp/interfaces",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("type", "Type", 10),
        col!("forced-external-ip", "External IP", 18),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::UPNP_INTERFACE_FORM),
};

pub const DNS_CACHE: ResourceSpec = ResourceSpec {
    id: "dns-cache",
    group: "ip-group",
    cli_path: None,
    label: "DNS Cache",
    fetch: FetchKind::List {
        endpoint: "/ip/dns/cache",
    },
    columns: &[
        col!("name", "Name", 28),
        col!("type", "Type", 8),
        col!("data", "Data", 28),
        col!("ttl", "TTL", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::DNS_CACHE_ACTIONS,
    form: None,
};

pub const DHCP_ALERTS: ResourceSpec = ResourceSpec {
    id: "dhcp-alerts",
    group: "ip-group",
    cli_path: None,
    label: "DHCP Alert",
    fetch: FetchKind::List {
        endpoint: "/ip/dhcp-server/alert",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("valid-server", "Valid server", 20),
        col!("alert-timeout", "Timeout", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::DHCP_ALERT_FORM),
};

pub const CONNECTION_TRACKING: ResourceSpec = ResourceSpec {
    id: "connection-tracking",
    group: "ip-group",
    cli_path: None,
    label: "Conntrack",
    fetch: FetchKind::System {
        endpoint: "/ip/firewall/connection/tracking",
    },
    columns: &[
        col!("enabled", "Enabled", 10),
        col!("tcp-established-timeout", "TCP est.", 12),
        col!("udp-timeout", "UDP", 10),
        col!("total-entries", "Entries", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::CONNECTION_TRACKING_FORM),
};

pub const NEIGHBOR_DISCOVERY: ResourceSpec = ResourceSpec {
    id: "neighbor-discovery",
    group: "ip-group",
    cli_path: None,
    label: "Discovery Settings",
    fetch: FetchKind::System {
        endpoint: "/ip/neighbor/discovery-settings",
    },
    columns: &[
        col!("discover-interface-list", "Discover", 18),
        col!("protocol", "Protocol", 12),
        col!("mode", "Mode", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::NEIGHBOR_DISCOVERY_FORM),
};

pub const IP_SSH: ResourceSpec = ResourceSpec {
    id: "ip-ssh",
    group: "ip-group",
    cli_path: None,
    label: "SSH",
    fetch: FetchKind::System {
        endpoint: "/ip/ssh",
    },
    columns: &[
        col!("strong-crypto", "Strong", 8),
        col!("host-key-size", "Key size", 10),
        col!("always-allow-password-login", "Password", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::IP_SSH_FORM),
};

pub const TRAFFIC_FLOW: ResourceSpec = ResourceSpec {
    id: "traffic-flow",
    group: "ip-group",
    cli_path: None,
    label: "Traffic Flow",
    fetch: FetchKind::System {
        endpoint: "/ip/traffic-flow",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("interfaces", "Interfaces", 20),
        col!("cache-entries", "Cache", 8),
        col!("packet-sampling", "Sampling", 9),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::TRAFFIC_FLOW_FORM),
};

pub const TRAFFIC_FLOW_TARGETS: ResourceSpec = ResourceSpec {
    id: "traffic-flow-targets",
    group: "ip-group",
    cli_path: None,
    label: "Traffic Flow Targets",
    fetch: FetchKind::List {
        endpoint: "/ip/traffic-flow/target",
    },
    columns: &[
        col!("src-address", "Src. Address", 16),
        col!("dst-address", "Dst. Address", 16),
        col!("port", "Port", 6),
        col!("version", "Version", 8),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::TRAFFIC_FLOW_TARGET_FORM),
};

pub const TRAFFIC_FLOW_IPFIX: ResourceSpec = ResourceSpec {
    id: "traffic-flow-ipfix",
    group: "ip-group",
    cli_path: None,
    label: "Traffic Flow IPFIX",
    fetch: FetchKind::System {
        endpoint: "/ip/traffic-flow/ipfix",
    },
    columns: &[
        col!("bytes", "Bytes", 7),
        col!("src-address", "Src", 6),
        col!("dst-address", "Dst", 6),
        col!("protocol", "Proto", 6),
        col!("nat-events", "NAT", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::TRAFFIC_FLOW_IPFIX_FORM),
};

pub const IGMP_PROXY: ResourceSpec = ResourceSpec {
    id: "igmp-proxy",
    group: "ip-group",
    cli_path: None,
    label: "IGMP Proxy",
    fetch: FetchKind::System {
        endpoint: "/routing/igmp-proxy",
    },
    columns: &[
        col!("query-interval", "Query", 10),
        col!("query-response-interval", "Response", 10),
        col!("quick-leave", "Quick leave", 11),
        col!("robustness", "Robust", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::IGMP_PROXY_FORM),
};

pub const IGMP_PROXY_INTERFACES: ResourceSpec = ResourceSpec {
    id: "igmp-proxy-interfaces",
    group: "ip-group",
    cli_path: None,
    label: "IGMP Proxy Interfaces",
    fetch: FetchKind::List {
        endpoint: "/routing/igmp-proxy/interface",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("upstream", "Up", 5),
        col!("threshold", "TTL", 6),
        col!("querier", "Querier", 8),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::IGMP_PROXY_INTERFACE_FORM),
};

pub const IGMP_PROXY_MFC: ResourceSpec = ResourceSpec {
    id: "igmp-proxy-mfc",
    group: "ip-group",
    cli_path: None,
    label: "IGMP Proxy MFC",
    fetch: FetchKind::List {
        endpoint: "/routing/igmp-proxy/mfc",
    },
    columns: &[
        col!("group", "Group", 16),
        col!("source", "Source", 16),
        col!("upstream-interface", "Upstream", 14),
        col!("downstream-interfaces", "Downstream", 20),
        col!("packets", "Packets", 10),
        col!("bytes", "Bytes", 12),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::IGMP_PROXY_MFC_FORM),
};

pub const PROXY: ResourceSpec = ResourceSpec {
    id: "proxy",
    group: "ip-group",
    cli_path: None,
    label: "Proxy",
    fetch: FetchKind::System {
        endpoint: "/ip/proxy",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("port", "Port", 6),
        col!("parent-proxy", "Parent", 18),
        col!("max-cache-size", "Cache", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ip::forms::PROXY_FORM),
};

pub const PROXY_ACCESS: ResourceSpec = ResourceSpec {
    id: "proxy-access",
    group: "ip-group",
    cli_path: None,
    label: "Proxy Access",
    fetch: FetchKind::List {
        endpoint: "/ip/proxy/access",
    },
    columns: &[
        col!("src-address", "Source", 18),
        col!("dst-host", "Host", 22),
        col!("action", "Action", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::ip::forms::PROXY_ACCESS_FORM),
};

pub const PROXY_CACHE: ResourceSpec = ResourceSpec {
    id: "proxy-cache",
    group: "ip-group",
    cli_path: None,
    label: "Proxy Cache",
    fetch: FetchKind::List {
        endpoint: "/ip/proxy/cache",
    },
    columns: &[
        col!("dst-host", "Host", 22),
        col!("method", "Method", 10),
        col!("action", "Action", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::PROXY_CACHE_FORM),
};

pub const PROXY_DIRECT: ResourceSpec = ResourceSpec {
    id: "proxy-direct",
    group: "ip-group",
    cli_path: None,
    label: "Proxy Direct",
    fetch: FetchKind::List {
        endpoint: "/ip/proxy/direct",
    },
    columns: &[
        col!("dst-host", "Host", 22),
        col!("dst-address", "Address", 18),
        col!("action", "Action", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::PROXY_DIRECT_FORM),
};

pub const HOTSPOT: ResourceSpec = ResourceSpec {
    id: "hotspot",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("profile", "Profile", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_FORM),
};

pub const HOTSPOT_PROFILES: ResourceSpec = ResourceSpec {
    id: "hotspot-profiles",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot Profiles",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/profile",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("hotspot-address", "Address", 18),
        col!("dns-name", "DNS", 20),
        col!("html-directory", "HTML", 16),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_PROFILE_FORM),
};

pub const HOTSPOT_USERS: ResourceSpec = ResourceSpec {
    id: "hotspot-users",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot Users",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/user",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("profile", "Profile", 16),
        col!("server", "Server", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_USER_FORM),
};

pub const HOTSPOT_USER_PROFILES: ResourceSpec = ResourceSpec {
    id: "hotspot-user-profiles",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot User Profiles",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/user/profile",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("session-timeout", "Session", 12),
        col!("idle-timeout", "Idle", 10),
        col!("rate-limit", "Rate limit", 16),
        col!("shared-users", "Shared", 8),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_USER_PROFILE_FORM),
};

pub const HOTSPOT_COOKIES: ResourceSpec = ResourceSpec {
    id: "hotspot-cookies",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot Cookies",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/cookie",
    },
    columns: &[
        col!("user", "User", 16),
        col!("mac-address", "MAC", 18),
        col!("expires-in", "Expires", 12),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const HOTSPOT_HOSTS: ResourceSpec = ResourceSpec {
    id: "hotspot-hosts",
    group: "ip-group",
    cli_path: None,
    label: "Hotspot Hosts",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/host",
    },
    columns: &[
        col!("mac-address", "MAC", 18),
        col!("address", "Address", 16),
        col!("server", "Server", 16),
        col!("authorized", "Auth", 6),
        col!("bypassed", "Bypass", 7),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::HOTSPOT_HOST_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_HOST_FORM),
};

pub const HOTSPOT_IP_BINDINGS: ResourceSpec = ResourceSpec {
    id: "hotspot-ip-bindings",
    group: "ip-group",
    cli_path: None,
    label: "IP Bindings",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/ip-binding",
    },
    columns: &[
        col!("mac-address", "MAC", 18),
        col!("address", "Address", 16),
        col!("to-address", "To", 16),
        col!("type", "Type", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_IP_BINDING_FORM),
};

pub const HOTSPOT_WALLED_GARDEN: ResourceSpec = ResourceSpec {
    id: "hotspot-walled-garden",
    group: "ip-group",
    cli_path: None,
    label: "Walled Garden",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/walled-garden",
    },
    columns: &[
        col!("dst-host", "Host", 24),
        col!("dst-port", "Port", 8),
        col!("action", "Action", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_WALLED_GARDEN_FORM),
};

pub const HOTSPOT_WALLED_GARDEN_IP: ResourceSpec = ResourceSpec {
    id: "hotspot-walled-garden-ip",
    group: "ip-group",
    cli_path: None,
    label: "Walled Garden IP",
    fetch: FetchKind::List {
        endpoint: "/ip/hotspot/walled-garden-ip",
    },
    columns: &[
        col!("dst-address", "Address", 20),
        col!("action", "Action", 10),
        col!("server", "Server", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ip::forms::HOTSPOT_WALLED_GARDEN_IP_FORM),
};

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    ARP,
    ADDRESSES,
    DHCP_SERVERS,
    DHCP_NETWORKS,
    DHCP_LEASES,
    DHCP_RELAY,
    DHCP_OPTIONS,
    DHCP_OPTION_SETS,
    FIREWALL_FILTER,
    NEIGHBORS,
    DHCP_CLIENTS,
    DNS,
    DNS_STATIC,
    ROUTES,
    POOLS,
    IP_SERVICES,
    IP_SETTINGS,
    FIREWALL_NAT,
    FIREWALL_MANGLE,
    FIREWALL_RAW,
    FIREWALL_CONNECTIONS,
    ADDRESS_LIST,
    FIREWALL_LAYER7,
    FIREWALL_SERVICE_PORT,
    IPSEC_PEERS,
    IPSEC_IDENTITIES,
    IPSEC_POLICIES,
    IPSEC_PROPOSALS,
    IPSEC_PROFILES,
    IPSEC_INSTALLED_SA,
    IPSEC_SETTINGS,
    IPSEC_MODE_CONFIG,
    IPSEC_KEY_RSA,
    IPSEC_KEY_PSK,
    IPSEC_KEY_QKD,
    CLOUD,
    KID_CONTROL,
    KID_CONTROL_DEVICES,
    SOCKS,
    SMB,
    SMB_SHARES,
    SMB_USERS,
    UPNP,
    UPNP_INTERFACES,
    DNS_CACHE,
    DHCP_ALERTS,
    CONNECTION_TRACKING,
    NEIGHBOR_DISCOVERY,
    IP_SSH,
    TRAFFIC_FLOW,
    TRAFFIC_FLOW_TARGETS,
    TRAFFIC_FLOW_IPFIX,
    IGMP_PROXY,
    IGMP_PROXY_INTERFACES,
    IGMP_PROXY_MFC,
    PROXY,
    PROXY_ACCESS,
    PROXY_CACHE,
    PROXY_DIRECT,
    HOTSPOT,
    HOTSPOT_PROFILES,
    HOTSPOT_USERS,
    HOTSPOT_USER_PROFILES,
    HOTSPOT_COOKIES,
    HOTSPOT_HOSTS,
    HOTSPOT_IP_BINDINGS,
    HOTSPOT_WALLED_GARDEN,
    HOTSPOT_WALLED_GARDEN_IP,
];
