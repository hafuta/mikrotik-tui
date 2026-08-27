//! Form schemas for the `IPv6` navigation group.
//!
//! Catalog wiring (do not register here):
//! - `ipv6-addresses` → `/ipv6/address` (`IPV6_ADDRESS_FORM`)
//! - `ipv6-neighbors` → `/ipv6/neighbor` (`IPV6_NEIGHBOR_FORM`)
//! - `ipv6-nd` → `/ipv6/nd` (`IPV6_ND_FORM`)
//! - `ipv6-routes` → `/ipv6/route` (`IPV6_ROUTE_FORM`)
//! - `ipv6-pool` → `/ipv6/pool` (`IPV6_POOL_FORM`)
//! - `ipv6-settings` → `/ipv6/settings` (`IPV6_SETTINGS_FORM`)
//! - `ipv6-firewall-filter` → `/ipv6/firewall/filter` (`IPV6_FIREWALL_FILTER_FORM`)
//! - `ipv6-dhcp-client` → `/ipv6/dhcp-client` (`IPV6_DHCP_CLIENT_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-dhcp-server` → `/ipv6/dhcp-server` (`IPV6_DHCP_SERVER_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-nd-prefix` → `/ipv6/nd/prefix` (`IPV6_ND_PREFIX_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-firewall-nat` → `/ipv6/firewall/nat` (`IPV6_FIREWALL_NAT_FORM`, `FILTER_ACTIONS`)
//! - `ipv6-address-list` → `/ipv6/firewall/address-list` (`IPV6_ADDRESS_LIST_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-firewall-connections` → `/ipv6/firewall/connection` (inspect/remove only; no form)
//!
//! Group id: `ipv6-group`.

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

const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACE_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
const LOOKUP_ROUTING_TABLE: FieldKind = FieldKind::Lookup {
    resource_id: "routing-tables",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IPV6_POOL: FieldKind = FieldKind::Lookup {
    resource_id: "ipv6-pool",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IPV6_ADDRESS_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "ipv6-address-list",
    value_key: "list",
    multiple: false,
};

const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Ipv6);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const CHAIN: FieldSpec = f!("chain", "Chain", FieldKind::Text);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const IN_INTERFACE: FieldSpec = f!("in-interface", "In interface", LOOKUP_IFACE);
const OUT_INTERFACE: FieldSpec = f!("out-interface", "Out interface", LOOKUP_IFACE);
const IN_INTERFACE_LIST: FieldSpec =
    f!("in-interface-list", "In interface list", LOOKUP_IFACE_LIST);
const OUT_INTERFACE_LIST: FieldSpec = f!(
    "out-interface-list",
    "Out interface list",
    LOOKUP_IFACE_LIST
);
const ROUTING_TABLE: FieldSpec = f!("routing-table", "Routing table", LOOKUP_ROUTING_TABLE);
const NAT_CHAIN: FieldSpec = f!(
    "chain",
    "Chain",
    FieldKind::Enum {
        values: &["srcnat", "dstnat"],
    }
);
const ADDRESS_POOL: FieldSpec = f!("address-pool", "Address pool", LOOKUP_IPV6_POOL);
const SRC_ADDRESS: FieldSpec = f!("src-address", "Src address", FieldKind::Ipv6);
const DST_ADDRESS: FieldSpec = f!("dst-address", "Dst address", FieldKind::Ipv6);
const SRC_ADDRESS_LIST: FieldSpec = f!(
    "src-address-list",
    "Src address list",
    LOOKUP_IPV6_ADDRESS_LIST
);
const DST_ADDRESS_LIST: FieldSpec = f!(
    "dst-address-list",
    "Dst address list",
    LOOKUP_IPV6_ADDRESS_LIST
);
const ADDRESS_LIST_NAME: FieldSpec = f!("list", "List", LOOKUP_IPV6_ADDRESS_LIST);
const PREFIX: FieldSpec = f!("prefix", "Prefix", FieldKind::Ipv6);

pub static IPV6_ADDRESS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                INTERFACE,
                f!("advertise", "Advertise", FieldKind::Toggle),
                f!("eui-64", "EUI-64", FieldKind::Toggle),
                f!("no-dad", "No DAD", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("actual-interface", "Actual interface", FieldKind::Readonly),
                f!("from-pool", "From pool", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
                f!("slave", "Slave", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_NEIGHBOR_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                INTERFACE,
                f!("mac-address", "MAC address", FieldKind::Mac),
                COMMENT,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("origin", "Origin", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_ND_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            INTERFACE,
            f!("ra-interval", "RA interval", FieldKind::Time),
            f!("ra-delay", "RA delay", FieldKind::Time),
            f!("mtu", "MTU", FieldKind::Number),
            f!("advertise-mac-address", "Advertise MAC", FieldKind::Toggle),
            f!("advertise-dns", "Advertise DNS", FieldKind::Toggle),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_ROUTE_FORM: FormSchema = FormSchema {
    title_key: "dst-address",
    subtitle_keys: &["gateway"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("dst-address", "Dst address", FieldKind::Ipv6),
                f!("gateway", "Gateway", FieldKind::Text),
                f!("distance", "Distance", FieldKind::Number),
                ROUTING_TABLE,
                f!("blackhole", "Blackhole", FieldKind::Flag),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("active", "Active", FieldKind::Readonly),
                f!("static", "Static", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_POOL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["prefix"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("prefix", "Prefix", FieldKind::Text),
            f!("prefix-length", "Prefix length", FieldKind::Number),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "forward",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("forward", "Forward", FieldKind::Toggle),
            f!("accept-redirects", "Accept redirects", FieldKind::Text),
            f!("max-neighbor-entries", "Max neighbors", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};

pub static IPV6_FIREWALL_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                CHAIN,
                ACTION,
                f!("src-address", "Src address", FieldKind::Text),
                f!("dst-address", "Dst address", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
                IN_INTERFACE,
                OUT_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE_LIST,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_DHCP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["status"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                INTERFACE,
                f!("pool-name", "Pool name", FieldKind::Text),
                f!("request", "Request", FieldKind::Text),
                f!("add-default-route", "Default route", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("prefix", "Prefix", FieldKind::Readonly),
                f!("expires-after", "Expires after", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_DHCP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            INTERFACE,
            ADDRESS_POOL,
            f!("lease-time", "Lease time", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_ND_PREFIX_FORM: FormSchema = FormSchema {
    title_key: "prefix",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            PREFIX,
            INTERFACE,
            f!("advertise", "Advertise", FieldKind::Toggle),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_FIREWALL_NAT_FORM: FormSchema = FormSchema {
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
                f!("protocol", "Protocol", FieldKind::Text),
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                f!("dst-port", "Dst port", FieldKind::Text),
                f!("to-addresses", "To addresses", FieldKind::Ipv6),
                f!("to-ports", "To ports", FieldKind::Text),
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_ADDRESS_LIST_FORM: FormSchema = FormSchema {
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
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("creation-time", "Creation time", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_DHCP_RELAY_FORM: FormSchema = FormSchema {
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
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_DHCP_BINDING_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["duid"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ADDRESS,
            f!("duid", "DUID", FieldKind::Text),
            f!("server", "Server", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static IPV6_FIREWALL_MANGLE_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                CHAIN,
                ACTION,
                f!("src-address", "Src address", FieldKind::Text),
                f!("dst-address", "Dst address", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
                IN_INTERFACE,
                OUT_INTERFACE,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IPV6_FIREWALL_RAW_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                CHAIN,
                ACTION,
                f!("src-address", "Src address", FieldKind::Text),
                f!("dst-address", "Dst address", FieldKind::Text),
                IN_INTERFACE,
                OUT_INTERFACE,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};
