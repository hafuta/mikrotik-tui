//! Form schemas for the IP nav group.
//!
//! Wire existing `resources.rs` ids:
//! - arp -> `ARP_FORM`, add/edit/remove (dynamic ARP is removable)
//! - addresses -> `ADDRESS_FORM`, `MEMBER_ACTIONS`
//! - dhcp-servers -> `DHCP_SERVER_FORM`, `MEMBER_ACTIONS`
//! - dhcp-networks -> `DHCP_NETWORK_FORM`, `LIST_ACTIONS`
//! - dhcp-leases -> `DHCP_LEASE_FORM`, `LEASE_ACTIONS`
//! - firewall-filter -> `FIREWALL_FILTER_FORM`, `FILTER_ACTIONS`
//!
//! Extra screens to paste into `ALL_RESOURCES` (parent owns `resources.rs`):
//!
//! ```ignore
//! ResourceSpec {
//!     id: "neighbors",
//!     group: "ip-group",
//!     label: "Neighbors",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/neighbor" },
//!     columns: &[
//!         col!("interface", "Interface", 16),
//!         col!("address", "Address", 18),
//!         col!("mac-address", "MAC address", 18),
//!         col!("identity", "Identity", 20),
//!         col!("platform", "Platform", 14),
//!         col!("version", "Version", 14),
//!         col!("discovered-version", "Discovered", 12),
//!         col!("unpack", "Unpack", 10),
//!         col!("ipv6", "IPv6", 6),
//!         col!("interface-name", "If name", 16),
//!     ],
//!     refresh: Duration::from_secs(10),
//!     actions: crate::actions::DISCONNECT_ACTIONS, // inspect/remove only; no form
//!     form: None,
//! }
//! ResourceSpec {
//!     id: "dhcp-clients",
//!     group: "ip-group",
//!     label: "DHCP Client",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/dhcp-client" },
//!     columns: &[
//!         col!("interface", "Interface", 16),
//!         col!("status", "Status", 12),
//!         col!("address", "Address", 20),
//!         col!("gateway", "Gateway", 18),
//!         col!("dhcp-server", "Server", 18),
//!         col!("add-default-route", "Default", 8),
//!         col!("use-peer-dns", "Peer DNS", 9),
//!         col!("disabled", "Off", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(5),
//!     actions: crate::actions::MEMBER_ACTIONS,
//!     form: Some(&crate::ip_write::DHCP_CLIENT_FORM),
//! }
//! ResourceSpec {
//!     id: "dns",
//!     group: "ip-group",
//!     label: "DNS",
//!     fetch: FetchKind::System { endpoint: "/rest/ip/dns" },
//!     columns: &[
//!         col!("servers", "Servers", 28),
//!         col!("allow-remote-requests", "Remote", 8),
//!         col!("cache-size", "Cache", 10),
//!         col!("cache-max-ttl", "Max TTL", 10),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::SINGLETON_EDIT_ACTIONS,
//!     form: Some(&crate::ip_write::DNS_FORM),
//! }
//! ResourceSpec {
//!     id: "dns-static",
//!     group: "ip-group",
//!     label: "Static DNS",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/dns/static" },
//!     columns: &[
//!         col!("name", "Name", 24),
//!         col!("address", "Address", 18),
//!         col!("type", "Type", 8),
//!         col!("ttl", "TTL", 10),
//!         col!("disabled", "Off", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::MEMBER_ACTIONS,
//!     form: Some(&crate::ip_write::DNS_STATIC_FORM),
//! }
//! ResourceSpec {
//!     id: "routes",
//!     group: "ip-group",
//!     label: "Routes",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/route" },
//!     columns: &[
//!         col!("dst-address", "Dst", 20),
//!         col!("gateway", "Gateway", 18),
//!         col!("distance", "Dist", 6),
//!         col!("routing-table", "Table", 12),
//!         col!("active", "Act", 5),
//!         col!("static", "Static", 7),
//!         col!("dynamic", "Dyn", 5),
//!         col!("unreachable", "Unreach", 8),
//!         col!("disabled", "Off", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(5),
//!     actions: crate::actions::MEMBER_ACTIONS,
//!     form: Some(&crate::ip_write::ROUTE_FORM),
//! }
//! ResourceSpec {
//!     id: "pools",
//!     group: "ip-group",
//!     label: "Pool",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/pool" },
//!     columns: &[
//!         col!("name", "Name", 18),
//!         col!("ranges", "Ranges", 36),
//!         col!("next-pool", "Next", 18),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::LIST_ACTIONS,
//!     form: Some(&crate::ip_write::POOL_FORM),
//! }
//! ResourceSpec {
//!     id: "ip-services",
//!     group: "ip-group",
//!     label: "Services",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/service" },
//!     columns: &[
//!         col!("name", "Name", 14),
//!         col!("port", "Port", 6),
//!         col!("address", "Address", 20),
//!         col!("certificate", "Certificate", 18),
//!         col!("tls-version", "TLS", 10),
//!         col!("connection-count", "Conns", 7),
//!         col!("disabled", "Off", 5),
//!     ],
//!     refresh: Duration::from_secs(15),
//!     actions: crate::actions::TOGGLE_EDIT_ACTIONS,
//!     form: Some(&crate::ip_write::SERVICE_FORM),
//! }
//! ResourceSpec {
//!     id: "ip-settings",
//!     group: "ip-group",
//!     label: "Settings",
//!     fetch: FetchKind::System { endpoint: "/rest/ip/settings" },
//!     columns: &[
//!         col!("ip-forward", "Forward", 8),
//!         col!("rp-filter", "RP filter", 12),
//!         col!("tcp-syncookies", "Syncookies", 11),
//!         col!("accept-redirects", "Accept redir", 13),
//!         col!("send-redirects", "Send redir", 11),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::SINGLETON_EDIT_ACTIONS,
//!     form: Some(&crate::ip_write::IP_SETTINGS_FORM),
//! }
//! ResourceSpec {
//!     id: "firewall-nat",
//!     group: "ip-group",
//!     label: "NAT",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/nat" },
//!     columns: &[
//!         col!("chain", "Chain", 10),
//!         col!("action", "Action", 14),
//!         col!("protocol", "Protocol", 9),
//!         col!("src-address", "Source", 20),
//!         col!("dst-address", "Destination", 20),
//!         col!("dst-port", "Dst port", 10),
//!         col!("to-addresses", "To addr", 20),
//!         col!("to-ports", "To ports", 10),
//!         col!("in-interface", "In interface", 16),
//!         col!("out-interface", "Out interface", 16),
//!         col!("packets", "Packets", 12),
//!         col!("bytes", "Bytes", 14),
//!         col!("disabled", "Off", 5),
//!         col!("dynamic", "Dyn", 5),
//!         col!("invalid", "Bad", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(5),
//!     actions: crate::actions::FILTER_ACTIONS,
//!     form: Some(&crate::ip_write::FIREWALL_NAT_FORM),
//! }
//! ResourceSpec {
//!     id: "firewall-mangle",
//!     group: "ip-group",
//!     label: "Mangle",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/mangle" },
//!     columns: &[
//!         col!("chain", "Chain", 10),
//!         col!("action", "Action", 14),
//!         col!("protocol", "Protocol", 9),
//!         col!("src-address", "Source", 20),
//!         col!("dst-address", "Destination", 20),
//!         col!("in-interface", "In interface", 16),
//!         col!("out-interface", "Out interface", 16),
//!         col!("new-routing-mark", "Mark", 16),
//!         col!("passthrough", "Pass", 6),
//!         col!("packets", "Packets", 12),
//!         col!("bytes", "Bytes", 14),
//!         col!("disabled", "Off", 5),
//!         col!("dynamic", "Dyn", 5),
//!         col!("invalid", "Bad", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(5),
//!     actions: crate::actions::FILTER_ACTIONS,
//!     form: Some(&crate::ip_write::FIREWALL_MANGLE_FORM),
//! }
//! ResourceSpec {
//!     id: "address-list",
//!     group: "ip-group",
//!     label: "Address List",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/address-list" },
//!     columns: &[
//!         col!("list", "List", 16),
//!         col!("address", "Address", 20),
//!         col!("timeout", "Timeout", 12),
//!         col!("dynamic", "Dyn", 5),
//!         col!("creation-time", "Created", 20),
//!         col!("disabled", "Off", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(10),
//!     actions: crate::actions::MEMBER_ACTIONS,
//!     form: Some(&crate::ip_write::ADDRESS_LIST_FORM),
//! }
//! ResourceSpec {
//!     id: "dhcp-relay",
//!     group: "ip-group",
//!     label: "DHCP Relay",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/dhcp-relay" },
//!     columns: &[
//!         col!("name", "Name", 18),
//!         col!("interface", "Interface", 16),
//!         col!("dhcp-server", "DHCP server", 24),
//!         col!("local-address", "Local", 18),
//!         col!("disabled", "Off", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(10),
//!     actions: crate::actions::MEMBER_ACTIONS,
//!     form: Some(&crate::ip_write::DHCP_RELAY_FORM),
//! }
//! ResourceSpec {
//!     id: "dhcp-options",
//!     group: "ip-group",
//!     label: "DHCP Options",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/dhcp-server/option" },
//!     columns: &[
//!         col!("name", "Name", 18),
//!         col!("code", "Code", 6),
//!         col!("value", "Value", 28),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::LIST_ACTIONS,
//!     form: Some(&crate::ip_write::DHCP_OPTION_FORM),
//! }
//! ResourceSpec {
//!     id: "dhcp-option-sets",
//!     group: "ip-group",
//!     label: "DHCP Option Sets",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/dhcp-server/option/sets" },
//!     columns: &[
//!         col!("name", "Name", 18),
//!         col!("options", "Options", 36),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::LIST_ACTIONS,
//!     form: Some(&crate::ip_write::DHCP_OPTION_SET_FORM),
//! }
//! ResourceSpec {
//!     id: "firewall-raw",
//!     group: "ip-group",
//!     label: "Raw",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/raw" },
//!     columns: &[
//!         col!("chain", "Chain", 12),
//!         col!("action", "Action", 14),
//!         col!("protocol", "Protocol", 9),
//!         col!("src-address", "Source", 20),
//!         col!("dst-address", "Destination", 20),
//!         col!("in-interface", "In interface", 16),
//!         col!("out-interface", "Out interface", 16),
//!         col!("packets", "Packets", 12),
//!         col!("bytes", "Bytes", 14),
//!         col!("disabled", "Off", 5),
//!         col!("dynamic", "Dyn", 5),
//!         col!("invalid", "Bad", 5),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(5),
//!     actions: crate::actions::FILTER_ACTIONS,
//!     form: Some(&crate::ip_write::FIREWALL_RAW_FORM),
//! }
//! ResourceSpec {
//!     id: "firewall-layer7",
//!     group: "ip-group",
//!     label: "Layer7 Protocol",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/layer7-protocol" },
//!     columns: &[
//!         col!("name", "Name", 18),
//!         col!("regexp", "Regexp", 36),
//!         col!("comment", "Comment", 28),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::LIST_ACTIONS,
//!     form: Some(&crate::ip_write::LAYER7_FORM),
//! }
//! ResourceSpec {
//!     id: "firewall-service-port",
//!     group: "ip-group",
//!     label: "Service Port",
//!     fetch: FetchKind::List { endpoint: "/rest/ip/firewall/service-port" },
//!     columns: &[
//!         col!("name", "Name", 14),
//!         col!("ports", "Ports", 20),
//!         col!("disabled", "Off", 5),
//!     ],
//!     refresh: Duration::from_secs(30),
//!     actions: crate::actions::TOGGLE_EDIT_ACTIONS,
//!     form: Some(&crate::ip_write::SERVICE_PORT_FORM),
//! }
//! ```

use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_INTERFACES: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_POOLS: FieldKind = FieldKind::Lookup {
    resource_id: "pools",
    value_key: "name",
    multiple: false,
};
const LOOKUP_DHCP_SERVERS: FieldKind = FieldKind::Lookup {
    resource_id: "dhcp-servers",
    value_key: "name",
    multiple: false,
};
const LOOKUP_ROUTING_TABLES: FieldKind = FieldKind::Lookup {
    resource_id: "routing-tables",
    value_key: "name",
    multiple: false,
};
const LOOKUP_CERTIFICATES: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};
const LOOKUP_INTERFACE_LISTS: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
const LOOKUP_ADDRESS_LIST_NAMES: FieldKind = FieldKind::Lookup {
    resource_id: "address-list",
    value_key: "list",
    multiple: false,
};
const LOOKUP_DHCP_OPTIONS: FieldKind = FieldKind::Lookup {
    resource_id: "dhcp-options",
    value_key: "name",
    multiple: true,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_INTERFACES);
const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const MAC: FieldSpec = f!("mac-address", "MAC address", FieldKind::Text);
const GATEWAY: FieldSpec = f!("gateway", "Gateway", FieldKind::Text);
const FILTER_CHAIN: FieldSpec = f!(
    "chain",
    "Chain",
    FieldKind::Enum {
        values: &["input", "forward", "output"],
    }
);
const RAW_CHAIN: FieldSpec = f!(
    "chain",
    "Chain",
    FieldKind::Enum {
        values: &["prerouting", "output"],
    }
);
const NAT_CHAIN: FieldSpec = f!(
    "chain",
    "Chain",
    FieldKind::Enum {
        values: &["srcnat", "dstnat"],
    }
);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const PROTOCOL: FieldSpec = f!("protocol", "Protocol", FieldKind::Text);
const SRC_ADDRESS: FieldSpec = f!("src-address", "Source", FieldKind::Text);
const DST_ADDRESS: FieldSpec = f!("dst-address", "Destination", FieldKind::Text);
const SRC_PORT: FieldSpec = f!("src-port", "Src port", FieldKind::Text);
const DST_PORT: FieldSpec = f!("dst-port", "Dst port", FieldKind::Text);
const SRC_ADDRESS_LIST: FieldSpec = f!(
    "src-address-list",
    "Src address list",
    LOOKUP_ADDRESS_LIST_NAMES
);
const DST_ADDRESS_LIST: FieldSpec = f!(
    "dst-address-list",
    "Dst address list",
    LOOKUP_ADDRESS_LIST_NAMES
);
const IN_INTERFACE: FieldSpec = f!("in-interface", "In interface", LOOKUP_INTERFACES);
const OUT_INTERFACE: FieldSpec = f!("out-interface", "Out interface", LOOKUP_INTERFACES);
const IN_INTERFACE_LIST: FieldSpec = f!(
    "in-interface-list",
    "In interface list",
    LOOKUP_INTERFACE_LISTS
);
const OUT_INTERFACE_LIST: FieldSpec = f!(
    "out-interface-list",
    "Out interface list",
    LOOKUP_INTERFACE_LISTS
);
const ADDRESS_POOL: FieldSpec = f!("address-pool", "Address pool", LOOKUP_POOLS);
const DHCP_SERVER: FieldSpec = f!("server", "Server", LOOKUP_DHCP_SERVERS);
const NEXT_POOL: FieldSpec = f!("next-pool", "Next pool", LOOKUP_POOLS);
const ROUTING_TABLE: FieldSpec = f!("routing-table", "Routing table", LOOKUP_ROUTING_TABLES);
const CERTIFICATE: FieldSpec = f!("certificate", "Certificate", LOOKUP_CERTIFICATES);
const ADDRESS_LIST_NAME: FieldSpec = f!("list", "List", LOOKUP_ADDRESS_LIST_NAMES);
const PACKETS: FieldSpec = f!("packets", "Packets", FieldKind::Readonly);
const BYTES: FieldSpec = f!("bytes", "Bytes", FieldKind::Readonly);
const DYNAMIC: FieldSpec = f!("dynamic", "Dynamic", FieldKind::Readonly);
const INVALID: FieldSpec = f!("invalid", "Invalid", FieldKind::Readonly);

const FW_STATUS: &[FieldSpec] = &[PACKETS, BYTES, DYNAMIC, INVALID];

pub static ARP_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["mac-address", "interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[ADDRESS, MAC, INTERFACE, COMMENT],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly), DYNAMIC],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, MAC, INTERFACE],
    }],
};

pub static ADDRESS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[ADDRESS, INTERFACE, COMMENT, DISABLED],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("network", "Network", FieldKind::Readonly), DYNAMIC],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, INTERFACE],
    }],
};

pub static DHCP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                ADDRESS_POOL,
                f!("lease-time", "Lease time", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE, ADDRESS_POOL],
    }],
};

pub static DHCP_NETWORK_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["gateway"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ADDRESS,
            GATEWAY,
            f!("dns-server", "DNS", FieldKind::Text),
            f!("domain", "Domain", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, GATEWAY],
    }],
};

pub static DHCP_LEASE_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["mac-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[ADDRESS, MAC, DHCP_SERVER, COMMENT, DISABLED],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("host-name", "Hostname", FieldKind::Readonly),
                f!("status", "Status", FieldKind::Readonly),
                f!("expires-after", "Expires", FieldKind::Readonly),
                DYNAMIC,
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, MAC],
    }],
};

pub static FIREWALL_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                FILTER_CHAIN,
                ACTION,
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                SRC_PORT,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                DST_PORT,
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FILTER_CHAIN, ACTION],
    }],
};

pub static DHCP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["status"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                INTERFACE,
                f!("add-default-route", "Default route", FieldKind::Toggle),
                f!("use-peer-dns", "Peer DNS", FieldKind::Toggle),
                f!("use-peer-ntp", "Peer NTP", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("address", "Address", FieldKind::Readonly),
                f!("gateway", "Gateway", FieldKind::Readonly),
                f!("dhcp-server", "DHCP server", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[INTERFACE],
    }],
};

pub static DNS_FORM: FormSchema = FormSchema {
    title_key: "servers",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("servers", "Servers", FieldKind::Text),
            f!(
                "allow-remote-requests",
                "Remote requests",
                FieldKind::Toggle
            ),
            f!("cache-size", "Cache size", FieldKind::Number),
            f!("cache-max-ttl", "Cache max TTL", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static DNS_STATIC_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            ADDRESS,
            f!("type", "Type", FieldKind::Text),
            f!("ttl", "TTL", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, ADDRESS],
    }],
};

pub static ROUTE_FORM: FormSchema = FormSchema {
    title_key: "dst-address",
    subtitle_keys: &["gateway"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("dst-address", "Dst address", FieldKind::Text),
                GATEWAY,
                f!("distance", "Distance", FieldKind::Number),
                ROUTING_TABLE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("active", "Active", FieldKind::Readonly),
                f!("static", "Static", FieldKind::Readonly),
                DYNAMIC,
                f!("unreachable", "Unreachable", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("dst-address", "Dst address", FieldKind::Text), GATEWAY],
    }],
};

pub static POOL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ranges"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("ranges", "Ranges", FieldKind::Text),
            NEXT_POOL,
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("ranges", "Ranges", FieldKind::Text)],
    }],
};

pub static SERVICE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["port"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("name", "Name", FieldKind::Readonly),
                f!("port", "Port", FieldKind::Number),
                ADDRESS,
                CERTIFICATE,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("tls-version", "TLS version", FieldKind::Readonly),
                f!("connection-count", "Connections", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IP_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "ip-forward",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("ip-forward", "IP forward", FieldKind::Toggle),
            f!("rp-filter", "RP filter", FieldKind::Text),
            f!("tcp-syncookies", "TCP syncookies", FieldKind::Toggle),
            f!("accept-redirects", "Accept redirects", FieldKind::Toggle),
            f!("send-redirects", "Send redirects", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static FIREWALL_NAT_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAT_CHAIN,
                ACTION,
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                DST_PORT,
                f!("to-addresses", "To addresses", FieldKind::Text),
                f!("to-ports", "To ports", FieldKind::Text),
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAT_CHAIN, ACTION],
    }],
};

pub static FIREWALL_MANGLE_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                FILTER_CHAIN,
                ACTION,
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                f!("new-routing-mark", "Routing mark", FieldKind::Text),
                f!("passthrough", "Passthrough", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FILTER_CHAIN, ACTION],
    }],
};

pub static ADDRESS_LIST_FORM: FormSchema = FormSchema {
    title_key: "list",
    subtitle_keys: &["address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS_LIST_NAME,
                ADDRESS,
                f!("timeout", "Timeout", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                DYNAMIC,
                f!("creation-time", "Creation time", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS_LIST_NAME, ADDRESS],
    }],
};

pub static DHCP_RELAY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            INTERFACE,
            f!("dhcp-server", "DHCP server", FieldKind::Text),
            f!("local-address", "Local address", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE],
    }],
};

pub static DHCP_OPTION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["code"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("code", "Code", FieldKind::Number),
            f!("value", "Value", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("code", "Code", FieldKind::Number)],
    }],
};

pub static DHCP_OPTION_SET_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["options"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("options", "Options", LOOKUP_DHCP_OPTIONS), COMMENT],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static FIREWALL_RAW_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                RAW_CHAIN,
                ACTION,
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                SRC_PORT,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                DST_PORT,
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[RAW_CHAIN, ACTION],
    }],
};

pub static LAYER7_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["regexp"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("regexp", "Regexp", FieldKind::Text), COMMENT],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("regexp", "Regexp", FieldKind::Text)],
    }],
};

pub static SERVICE_PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ports"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("name", "Name", FieldKind::Readonly),
            f!("ports", "Ports", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::patch_body;
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn status_readonly(schema: &FormSchema) {
        let status = schema
            .sections
            .iter()
            .find(|section| section.id == "status")
            .expect("status tab");
        assert!(status.read_only);
        for field in status.fields {
            assert!(!field.kind.writable(), "{}", field.key);
            assert!(!schema.writable_keys().contains(&field.key));
        }
    }

    #[test]
    fn create_sheets_are_short() {
        assert_eq!(
            create_keys(&ARP_FORM),
            ["address", "mac-address", "interface"]
        );
        assert_eq!(create_keys(&ADDRESS_FORM), ["address", "interface"]);
        assert_eq!(
            create_keys(&DHCP_SERVER_FORM),
            ["name", "interface", "address-pool"]
        );
        assert_eq!(create_keys(&DHCP_NETWORK_FORM), ["address", "gateway"]);
        assert_eq!(create_keys(&DHCP_LEASE_FORM), ["address", "mac-address"]);
        assert_eq!(create_keys(&FIREWALL_FILTER_FORM), ["chain", "action"]);
        assert_eq!(create_keys(&DHCP_CLIENT_FORM), ["interface"]);
        assert!(DNS_FORM.create_sections.is_empty());
        assert_eq!(create_keys(&DNS_STATIC_FORM), ["name", "address"]);
        assert_eq!(create_keys(&ROUTE_FORM), ["dst-address", "gateway"]);
        assert_eq!(create_keys(&POOL_FORM), ["name", "ranges"]);
        assert!(SERVICE_FORM.create_sections.is_empty());
        assert!(IP_SETTINGS_FORM.create_sections.is_empty());
        assert_eq!(create_keys(&FIREWALL_NAT_FORM), ["chain", "action"]);
        assert_eq!(create_keys(&FIREWALL_MANGLE_FORM), ["chain", "action"]);
        assert_eq!(create_keys(&ADDRESS_LIST_FORM), ["list", "address"]);
        assert_eq!(create_keys(&DHCP_RELAY_FORM), ["name", "interface"]);
        assert_eq!(create_keys(&DHCP_OPTION_FORM), ["name", "code"]);
        assert_eq!(create_keys(&DHCP_OPTION_SET_FORM), ["name"]);
        assert_eq!(create_keys(&FIREWALL_RAW_FORM), ["chain", "action"]);
        assert_eq!(create_keys(&LAYER7_FORM), ["name", "regexp"]);
        assert!(SERVICE_PORT_FORM.create_sections.is_empty());
    }

    #[test]
    fn firewall_status_is_readonly() {
        status_readonly(&FIREWALL_FILTER_FORM);
        status_readonly(&FIREWALL_NAT_FORM);
        status_readonly(&FIREWALL_MANGLE_FORM);
        status_readonly(&FIREWALL_RAW_FORM);
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"packets"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"bytes"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"dynamic"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"invalid"));
    }

    #[test]
    fn patch_body_skips_status_and_unchanged() {
        let mut original = HashMap::new();
        original.insert("address".into(), "192.168.1.2/24".into());
        original.insert("interface".into(), "ether1".into());
        original.insert("comment".into(), "lan".into());
        original.insert("disabled".into(), "false".into());
        original.insert("network".into(), "192.168.1.0".into());
        original.insert("dynamic".into(), "false".into());
        let mut current = original.clone();
        current.insert("comment".into(), "office".into());
        current.insert("network".into(), "10.0.0.0".into());
        let body = patch_body(&ADDRESS_FORM, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("office"));
        assert!(!body.contains_key("network"));
        assert!(!body.contains_key("dynamic"));
        assert!(!body.contains_key("address"));
    }

    #[test]
    fn service_name_is_readonly() {
        // Disabling www-ssl drops REST access to this app.
        assert!(!SERVICE_FORM.writable_keys().contains(&"name"));
        assert!(SERVICE_FORM.writable_keys().contains(&"disabled"));
        status_readonly(&SERVICE_FORM);
    }

    fn field_kind(schema: &FormSchema, key: &str) -> FieldKind {
        schema
            .sections
            .iter()
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
            .unwrap_or_else(|| panic!("missing field {key}"))
            .kind
    }

    fn create_field_kind(schema: &FormSchema, key: &str) -> FieldKind {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
            .unwrap_or_else(|| panic!("missing create field {key}"))
            .kind
    }

    fn lookup(resource_id: &'static str, value_key: &'static str) -> FieldKind {
        FieldKind::Lookup {
            resource_id,
            value_key,
            multiple: false,
        }
    }

    fn lookup_multi(resource_id: &'static str, value_key: &'static str) -> FieldKind {
        FieldKind::Lookup {
            resource_id,
            value_key,
            multiple: true,
        }
    }

    #[test]
    fn lookup_fields_use_named_resources() {
        let interfaces = lookup("interfaces", "name");
        let pools = lookup("pools", "name");
        let address_list_names = lookup("address-list", "list");
        let interface_lists = lookup("interface-lists", "name");

        for schema in [
            &ARP_FORM,
            &ADDRESS_FORM,
            &DHCP_CLIENT_FORM,
            &DHCP_SERVER_FORM,
            &DHCP_RELAY_FORM,
        ] {
            assert_eq!(field_kind(schema, "interface"), interfaces);
            assert_eq!(create_field_kind(schema, "interface"), interfaces);
        }

        assert_eq!(field_kind(&DHCP_SERVER_FORM, "address-pool"), pools);
        assert_eq!(create_field_kind(&DHCP_SERVER_FORM, "address-pool"), pools);

        assert_eq!(
            field_kind(&DHCP_LEASE_FORM, "server"),
            lookup("dhcp-servers", "name")
        );

        assert_eq!(field_kind(&POOL_FORM, "next-pool"), pools);
        assert_eq!(
            field_kind(&ROUTE_FORM, "routing-table"),
            lookup("routing-tables", "name")
        );
        assert_eq!(
            field_kind(&SERVICE_FORM, "certificate"),
            lookup("certificates", "name")
        );

        assert_eq!(field_kind(&ADDRESS_LIST_FORM, "list"), address_list_names);
        assert_eq!(
            create_field_kind(&ADDRESS_LIST_FORM, "list"),
            address_list_names
        );

        for schema in [
            &FIREWALL_FILTER_FORM,
            &FIREWALL_NAT_FORM,
            &FIREWALL_MANGLE_FORM,
            &FIREWALL_RAW_FORM,
        ] {
            assert_eq!(field_kind(schema, "in-interface"), interfaces);
            assert_eq!(field_kind(schema, "out-interface"), interfaces);
            assert_eq!(field_kind(schema, "in-interface-list"), interface_lists);
            assert_eq!(field_kind(schema, "out-interface-list"), interface_lists);
            assert_eq!(field_kind(schema, "src-address-list"), address_list_names);
            assert_eq!(field_kind(schema, "dst-address-list"), address_list_names);
            assert!(!create_keys(schema).contains(&"in-interface"));
            assert!(!create_keys(schema).contains(&"src-address-list"));
        }

        assert_eq!(
            field_kind(&DHCP_OPTION_SET_FORM, "options"),
            lookup_multi("dhcp-options", "name")
        );
        assert_eq!(field_kind(&DHCP_OPTION_FORM, "code"), FieldKind::Number);
        assert_eq!(
            create_field_kind(&DHCP_OPTION_FORM, "code"),
            FieldKind::Number
        );
        assert!(!SERVICE_PORT_FORM.writable_keys().contains(&"name"));
        assert!(SERVICE_PORT_FORM.writable_keys().contains(&"disabled"));
    }
}
