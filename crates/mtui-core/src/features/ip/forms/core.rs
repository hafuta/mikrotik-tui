//! Form schemas for the IP navigation group.

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
const LOOKUP_INTERFACES_MULTI: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: true,
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
const LOOKUP_SMB_USERS: FieldKind = FieldKind::Lookup {
    resource_id: "smb-users",
    value_key: "name",
    multiple: true,
};
const LOOKUP_FILES: FieldKind = FieldKind::Lookup {
    resource_id: "files",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
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
    create_sections: &[],
};

pub static ADDRESS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[ADDRESS, INTERFACE, COMMENT, ENABLED],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("network", "Network", FieldKind::Readonly), DYNAMIC],
        },
    ],
    create_sections: &[],
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
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
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
    create_sections: &[],
};

pub static DHCP_LEASE_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["mac-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[ADDRESS, MAC, DHCP_SERVER, COMMENT, ENABLED],
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
    create_sections: &[],
};

pub static FIREWALL_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[FILTER_CHAIN, ACTION, COMMENT, ENABLED],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
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
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[],
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
                ENABLED,
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
    create_sections: &[],
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
            ENABLED,
        ],
    }],
    create_sections: &[],
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
                DYNAMIC,
                f!("unreachable", "Unreachable", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
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
    create_sections: &[],
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
                ENABLED,
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
                f!("to-addresses", "To addresses", FieldKind::Text),
                f!("to-ports", "To ports", FieldKind::Text),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                DST_PORT,
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[],
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
                f!("new-routing-mark", "Routing mark", FieldKind::Text),
                f!("passthrough", "Passthrough", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                PROTOCOL,
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[],
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
                ENABLED,
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
    create_sections: &[],
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
            ENABLED,
        ],
    }],
    create_sections: &[],
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
    create_sections: &[],
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
    create_sections: &[],
};

pub static FIREWALL_RAW_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[RAW_CHAIN, ACTION, COMMENT, ENABLED],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
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
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: FW_STATUS,
        },
    ],
    create_sections: &[],
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
    create_sections: &[],
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
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static CLOUD_FORM: FormSchema = FormSchema {
    title_key: "ddns-enabled",
    subtitle_keys: &["dns-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("ddns-enabled", "DDNS", FieldKind::Toggle),
                f!("update-time", "Update time", FieldKind::Toggle),
                f!("public-address", "Public address", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("dns-name", "DNS name", FieldKind::Readonly),
                f!("status", "Status", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static KID_CONTROL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["rate-limit"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("mon-fri", "Mon-Fri", FieldKind::Text),
            f!("sat", "Saturday", FieldKind::Text),
            f!("sun", "Sunday", FieldKind::Text),
            f!("rate-limit", "Rate limit", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static KID_CONTROL_DEVICE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["mac-address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            MAC,
            f!("user", "User", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static SOCKS_FORM: FormSchema = FormSchema {
    title_key: "port",
    subtitle_keys: &["enabled"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("port", "Port", FieldKind::Number),
            f!("connection-idle-timeout", "Idle timeout", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static SMB_FORM: FormSchema = FormSchema {
    title_key: "domain",
    subtitle_keys: &["enabled"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("domain", "Domain", FieldKind::Text),
            f!("comment", "Comment", FieldKind::Text),
            f!("allow-guests", "Allow guests", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static SMB_SHARE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["directory"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("directory", "Directory", LOOKUP_FILES),
                COMMENT,
                f!("valid-users", "Valid Users", LOOKUP_SMB_USERS),
                f!("invalid-users", "Invalid Users", LOOKUP_SMB_USERS),
                f!("read-only", "Read Only", FieldKind::Toggle),
                f!(
                    "require-encryption",
                    "Require Encryption",
                    FieldKind::Toggle
                ),
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("default", "Default", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static SMB_USER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["read-only"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("password", "Password", FieldKind::Secret),
                COMMENT,
                f!("read-only", "Read Only", FieldKind::Toggle),
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("default", "Default", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static UPNP_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["allow-disable-external-interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!(
                "allow-disable-external-interface",
                "Allow disable WAN",
                FieldKind::Toggle
            ),
            f!("show-dummy-rule", "Dummy rule", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static UPNP_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["type"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            INTERFACE,
            f!("type", "Type", FieldKind::Text),
            f!("forced-external-ip", "Forced external IP", FieldKind::Text),
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static DHCP_ALERT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["valid-server"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            INTERFACE,
            f!("valid-server", "Valid server", FieldKind::Text),
            f!("alert-timeout", "Alert timeout", FieldKind::Text),
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static CONNECTION_TRACKING_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["tcp-established-timeout"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("enabled", "Enabled", FieldKind::Text),
                f!(
                    "tcp-established-timeout",
                    "TCP established",
                    FieldKind::Text
                ),
                f!("udp-timeout", "UDP timeout", FieldKind::Text),
                f!("icmp-timeout", "ICMP timeout", FieldKind::Text),
                f!("generic-timeout", "Generic timeout", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("total-entries", "Entries", FieldKind::Readonly),
                f!("max-entries", "Max entries", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static NEIGHBOR_DISCOVERY_FORM: FormSchema = FormSchema {
    title_key: "discover-interface-list",
    subtitle_keys: &["protocol"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!(
                "discover-interface-list",
                "Discover list",
                LOOKUP_INTERFACE_LISTS
            ),
            f!("protocol", "Protocol", FieldKind::Text),
            f!("lldp-med-net-policy-vlan", "LLDP-MED VLAN", FieldKind::Text),
            f!("mode", "Mode", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static IP_SSH_FORM: FormSchema = FormSchema {
    title_key: "strong-crypto",
    subtitle_keys: &["host-key-size"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("strong-crypto", "Strong crypto", FieldKind::Toggle),
            f!("host-key-size", "Host key size", FieldKind::Number),
            f!(
                "always-allow-password-login",
                "Password login",
                FieldKind::Toggle
            ),
            f!("forwarding-enabled", "Forwarding", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub(crate) const CACHE_ENTRIES: &[&str] = &[
    "1k", "2k", "4k", "8k", "16k", "32k", "64k", "128k", "256k", "512k", "1M", "2M", "4M", "8M",
    "16M", "32M", "64M",
];
pub(crate) const TRAFFIC_FLOW_VERSIONS: &[&str] = &["1", "5", "9", "ipfix"];

pub static TRAFFIC_FLOW_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["interfaces"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("interfaces", "Interfaces", FieldKind::Repeat),
            f!(
                "cache-entries",
                "Cache Entries",
                FieldKind::Enum {
                    values: CACHE_ENTRIES,
                }
            ),
            f!(
                "active-flow-timeout",
                "Active Flow Timeout",
                FieldKind::Text
            ),
            f!(
                "inactive-flow-timeout",
                "Inactive Flow Timeout",
                FieldKind::Text
            ),
            f!("packet-sampling", "Packet Sampling", FieldKind::Toggle),
            f!("sampling-interval", "Sampling Interval", FieldKind::Number),
            f!("sampling-space", "Sampling Space", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};

pub static TRAFFIC_FLOW_TARGET_FORM: FormSchema = FormSchema {
    title_key: "dst-address",
    subtitle_keys: &["version"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("src-address", "Src. Address", FieldKind::Text),
            f!("dst-address", "Dst. Address", FieldKind::Text),
            f!("port", "Port", FieldKind::Number),
            f!(
                "version",
                "Version",
                FieldKind::Enum {
                    values: TRAFFIC_FLOW_VERSIONS,
                }
            ),
            f!(
                "v9-template-refresh",
                "v9 Template Refresh",
                FieldKind::Number
            ),
            f!(
                "v9-template-timeout",
                "v9 Template Timeout",
                FieldKind::Text
            ),
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub(crate) const IPFIX_GENERAL: &[FieldSpec] = &[
    f!("bytes", "Bytes", FieldKind::Toggle),
    f!("ip-total-length", "IP Total Length", FieldKind::Toggle),
    f!("src-address", "Src. Address", FieldKind::Toggle),
    f!("dst-address", "Dst. Address", FieldKind::Toggle),
    f!("ipv6-flow-label", "IPv6 Flow Label", FieldKind::Toggle),
    f!("src-address-mask", "Src. Address Mask", FieldKind::Toggle),
    f!("dst-address-mask", "Dst. Address Mask", FieldKind::Toggle),
    f!("is-multicast", "Is Multicast", FieldKind::Toggle),
    f!("src-mac-address", "Src. MAC Address", FieldKind::Toggle),
    f!("dst-mac-address", "Dst. MAC Address", FieldKind::Toggle),
    f!("last-forwarded", "Last Forwarded", FieldKind::Toggle),
    f!("src-port", "Src. Port", FieldKind::Toggle),
    f!("dst-port", "Dst. Port", FieldKind::Toggle),
    f!("nat-dst-address", "NAT Dst. Address", FieldKind::Toggle),
    f!("sys-init-time", "Sys Init Time", FieldKind::Toggle),
    f!("first-forwarded", "First Forwarded", FieldKind::Toggle),
    f!("nat-dst-port", "NAT Dst. Port", FieldKind::Toggle),
    f!("tcp-ack-num", "TCP Ack Num", FieldKind::Toggle),
    f!("gateway", "Gateway", FieldKind::Toggle),
    f!("nat-events", "NAT Events", FieldKind::Toggle),
    f!("tcp-flags", "TCP Flags", FieldKind::Toggle),
    f!("icmp-code", "ICMP Code", FieldKind::Toggle),
    f!("nat-src-address", "NAT Src. Address", FieldKind::Toggle),
    f!("icmp-type", "ICMP Type", FieldKind::Toggle),
    f!("nat-src-port", "NAT Src. Port", FieldKind::Toggle),
    f!("tcp-seq-num", "TCP Seq Num", FieldKind::Toggle),
    f!("tcp-window-size", "TCP Window Size", FieldKind::Toggle),
    f!("igmp-type", "IGMP Type", FieldKind::Toggle),
    f!("out-interface", "Out Interface", FieldKind::Toggle),
    f!("in-interface", "In Interface", FieldKind::Toggle),
    f!("packets", "Packets", FieldKind::Toggle),
    f!("ip-header-length", "IP Header Length", FieldKind::Toggle),
    f!("protocol", "Protocol", FieldKind::Toggle),
    f!("tos", "ToS", FieldKind::Toggle),
    f!("ttl", "TTL", FieldKind::Toggle),
    f!("udp-length", "UDP Length", FieldKind::Toggle),
];

pub static TRAFFIC_FLOW_IPFIX_FORM: FormSchema = FormSchema {
    title_key: "bytes",
    subtitle_keys: &["protocol"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: IPFIX_GENERAL,
    }],
    create_sections: &[],
};

pub static IGMP_PROXY_FORM: FormSchema = FormSchema {
    title_key: "query-interval",
    subtitle_keys: &["quick-leave"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("query-interval", "Query Interval", FieldKind::Text),
            f!(
                "query-response-interval",
                "Query Response Interval",
                FieldKind::Text
            ),
            f!(
                "last-member-query-interval",
                "Last Member Query Interval",
                FieldKind::Text
            ),
            f!("robustness", "Robustness", FieldKind::Number),
            f!("quick-leave", "Quick Leave", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static IGMP_PROXY_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["upstream"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                INTERFACE,
                f!("upstream", "Upstream", FieldKind::Toggle),
                f!("threshold", "Threshold", FieldKind::Number),
                f!(
                    "alternative-subnets",
                    "Alternative Subnets",
                    FieldKind::Repeat
                ),
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("querier", "Querier", FieldKind::Readonly),
                f!(
                    "source-ip-address",
                    "Source IP Address",
                    FieldKind::Readonly
                ),
                f!("rx-bytes", "RX Bytes", FieldKind::Readonly),
                f!("rx-packets", "RX Packets", FieldKind::Readonly),
                f!("tx-bytes", "TX Bytes", FieldKind::Readonly),
                f!("tx-packets", "TX Packets", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IGMP_PROXY_MFC_FORM: FormSchema = FormSchema {
    title_key: "group",
    subtitle_keys: &["source"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("group", "Group", FieldKind::Text),
                f!("source", "Source", FieldKind::Text),
                f!(
                    "upstream-interface",
                    "Upstream Interface",
                    LOOKUP_INTERFACES
                ),
                f!(
                    "downstream-interfaces",
                    "Downstream Interfaces",
                    LOOKUP_INTERFACES_MULTI
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!(
                    "active-downstream-interfaces",
                    "Active Downstream Interfaces",
                    FieldKind::Readonly
                ),
                f!("bytes", "Bytes", FieldKind::Readonly),
                f!("packets", "Packets", FieldKind::Readonly),
                f!("wrong-packets", "Wrong Packets", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};
