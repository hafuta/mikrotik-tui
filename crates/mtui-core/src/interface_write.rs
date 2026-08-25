//! Form schemas for the Interfaces nav group.

use crate::forms::{ARP_VALUES, FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const MTU: FieldSpec = f!("mtu", "MTU", FieldKind::Number);
const L2MTU: FieldSpec = f!("l2mtu", "L2 MTU", FieldKind::Number);
const MAC: FieldSpec = f!("mac-address", "MAC address", FieldKind::Text);
const ARP: FieldSpec = f!("arp", "ARP", FieldKind::Enum { values: ARP_VALUES });
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const SLAVE: FieldSpec = f!("slave", "Slave", FieldKind::Readonly);
const IFACE_TYPE: FieldSpec = f!("type", "Type", FieldKind::Readonly);
const DEFAULT_NAME: FieldSpec = f!("default-name", "Default name", FieldKind::Readonly);

const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACES: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: true,
};
const LOOKUP_IFACE_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACE_LISTS: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: true,
};
const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};
const LOOKUP_MACSEC_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "macsec-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_LTE_APN: FieldKind = FieldKind::Lookup {
    resource_id: "lte-apn",
    value_key: "name",
    multiple: true,
};

pub const LTE_APN_AUTHENTICATION: &[&str] = &["none", "pap", "chap"];
pub const LTE_APN_IP_TYPE: &[&str] = &["ipv4", "ipv4-ipv6", "ipv6", "auto"];
pub const LTE_APN_PASSTHROUGH_SUBNET: &[&str] = &["auto", "p2p"];
pub const LTE_SMS_PROTOCOL: &[&str] = &["auto", "at", "mbim"];

const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);

pub static INTERFACES_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[NAME, COMMENT, DISABLED, MTU, L2MTU, MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                IFACE_TYPE,
                RUNNING,
                SLAVE,
                f!("actual-mtu", "Actual MTU", FieldKind::Readonly),
                f!("tx-byte", "TX", FieldKind::Readonly),
                f!("rx-byte", "RX", FieldKind::Readonly),
                f!("tx-packet", "TX packets", FieldKind::Readonly),
                f!("rx-packet", "RX packets", FieldKind::Readonly),
                f!("last-link-up-time", "Last link up", FieldKind::Readonly),
                f!("last-link-down-time", "Last link down", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static ETHERNET_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["default-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[NAME, COMMENT, DISABLED],
        },
        FormSection {
            id: "ethernet",
            label: "Ethernet",
            read_only: false,
            fields: &[
                f!("auto-negotiation", "Auto-neg", FieldKind::Toggle),
                f!("advertise", "Advertise", FieldKind::Text),
                f!("speed", "Speed", FieldKind::Text),
                f!("full-duplex", "Full duplex", FieldKind::Toggle),
                ARP,
                f!(
                    "loop-protect",
                    "Loop protect",
                    FieldKind::Enum {
                        values: &["default", "on", "off"]
                    }
                ),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[MTU, L2MTU, MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                DEFAULT_NAME,
                f!("orig-mac-address", "Orig MAC", FieldKind::Readonly),
                f!("switch", "Switch", FieldKind::Readonly),
                f!("loop-protect-status", "Loop status", FieldKind::Readonly),
                RUNNING,
                SLAVE,
            ],
        },
    ],
    create_sections: &[],
};

pub static VLAN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vlan-id", "interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("vlan-id", "VLAN ID", FieldKind::Number),
                INTERFACE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MTU,
                L2MTU,
                MAC,
                ARP,
                f!("use-service-tag", "Service tag", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("vlan-id", "VLAN ID", FieldKind::Number),
            INTERFACE,
            COMMENT,
        ],
    }],
};

pub static EOIP_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["local-address", "remote-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("tunnel-id", "Tunnel ID", FieldKind::Number),
                f!("local-address", "Local", FieldKind::Text),
                f!("remote-address", "Remote", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MTU,
                MAC,
                ARP,
                f!("keepalive", "Keepalive", FieldKind::Text),
                f!("allow-fast-path", "Fast path", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("tunnel-id", "Tunnel ID", FieldKind::Number),
            f!("local-address", "Local", FieldKind::Text),
            f!("remote-address", "Remote", FieldKind::Text),
        ],
    }],
};

pub static IPIP_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["local-address", "remote-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("local-address", "Local", FieldKind::Text),
                f!("remote-address", "Remote", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MTU,
                f!("clamp-tcp-mss", "Clamp MSS", FieldKind::Toggle),
                f!("dscp", "DSCP", FieldKind::Text),
                f!("allow-fast-path", "Fast path", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("local-address", "Local", FieldKind::Text),
            f!("remote-address", "Remote", FieldKind::Text),
        ],
    }],
};

pub static GRE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["local-address", "remote-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("local-address", "Local", FieldKind::Text),
                f!("remote-address", "Remote", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MTU,
                f!("keepalive", "Keepalive", FieldKind::Text),
                f!("dscp", "DSCP", FieldKind::Text),
                f!("clamp-tcp-mss", "Clamp MSS", FieldKind::Toggle),
                f!("allow-fast-path", "Fast path", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("local-address", "Local", FieldKind::Text),
            f!("remote-address", "Remote", FieldKind::Text),
        ],
    }],
};

pub static VXLAN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vni"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("vni", "VNI", FieldKind::Number),
                f!("port", "Port", FieldKind::Number),
                f!("group", "Group", FieldKind::Text),
                f!("local", "Local", FieldKind::Text),
                INTERFACE,
                f!("vrf", "VRF", LOOKUP_VRF),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[MTU, MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("vni", "VNI", FieldKind::Number), INTERFACE],
    }],
};

pub static VRRP_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vrid", "interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                f!("vrid", "VRID", FieldKind::Number),
                f!("priority", "Priority", FieldKind::Number),
                f!("interval", "Interval", FieldKind::Text),
                f!("version", "Version", FieldKind::Number),
                f!("preemption-mode", "Preempt", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[f!("v3-protocol", "V3 proto", FieldKind::Text), MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE, f!("vrid", "VRID", FieldKind::Number)],
    }],
};

pub static BONDING_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["mode"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("slaves", "Slaves", LOOKUP_IFACES),
                f!("mode", "Mode", FieldKind::Text),
                f!("primary", "Primary", LOOKUP_IFACE),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("link-monitoring", "Monitor", FieldKind::Text),
                f!("transmit-hash-policy", "Hash", FieldKind::Text),
                f!("min-links", "Min links", FieldKind::Number),
                MTU,
                MAC,
                ARP,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("slaves", "Slaves", LOOKUP_IFACES),
            f!("mode", "Mode", FieldKind::Text),
        ],
    }],
};

pub static MACVLAN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "mode"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                f!("mode", "Mode", FieldKind::Text),
                MAC,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[MTU, ARP],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE],
    }],
};

pub static MACSEC_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "status"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                f!("profile", "Profile", LOOKUP_MACSEC_PROFILE),
                MTU,
                f!("cak", "CAK", FieldKind::Secret),
                f!("ckn", "CKN", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly), RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE],
    }],
};

pub static MACSEC_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["server-priority"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("server-priority", "Server priority", FieldKind::Number),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static LIST_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("include", "Include", LOOKUP_IFACE_LISTS),
            f!("exclude", "Exclude", LOOKUP_IFACE_LISTS),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            COMMENT,
            f!("include", "Include", LOOKUP_IFACE_LISTS),
            f!("exclude", "Exclude", LOOKUP_IFACE_LISTS),
        ],
    }],
};

pub static MEMBER_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["list"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("list", "List", LOOKUP_IFACE_LIST),
            INTERFACE,
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            DISABLED,
            f!("list", "List", LOOKUP_IFACE_LIST),
            INTERFACE,
            COMMENT,
        ],
    }],
};

pub static VRF_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("interfaces", "Interfaces", LOOKUP_IFACES), COMMENT],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("interfaces", "Interfaces", LOOKUP_IFACES)],
    }],
};

pub static DETECT_INTERNET_FORM: FormSchema = FormSchema {
    title_key: "state",
    subtitle_keys: &[],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("detect-interface-list", "Detect", LOOKUP_IFACE_LIST),
                f!("lan-interface-list", "LAN", LOOKUP_IFACE_LIST),
                f!("wan-interface-list", "WAN", LOOKUP_IFACE_LIST),
                f!("internet-interface-list", "Internet", LOOKUP_IFACE_LIST),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("state", "State", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static LTE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["default-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                COMMENT,
                DISABLED,
                MTU,
                MAC,
                f!("network-mode", "Network Mode", FieldKind::Repeat),
                f!("band", "Band", FieldKind::Repeat),
                f!("nr-band", "NR Band", FieldKind::Repeat),
                f!("pin", "PIN", FieldKind::Secret),
                f!("operator", "Operator", FieldKind::Text),
                f!("modem-init", "Modem Init", FieldKind::Text),
                f!("allow-roaming", "Allow Roaming", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "apn",
            label: "APN",
            read_only: false,
            fields: &[f!("apn-profiles", "APN Profiles", LOOKUP_LTE_APN)],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!(
                    "sms-protocol",
                    "SMS Protocol",
                    FieldKind::Enum {
                        values: LTE_SMS_PROTOCOL
                    }
                ),
                f!("sms-read", "SMS Read", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                IFACE_TYPE,
                DEFAULT_NAME,
                RUNNING,
                f!("advertised-mtu", "Advertised MTU", FieldKind::Readonly),
                f!("master", "Master", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static LTE_APN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["apn"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("apn", "APN", FieldKind::Text),
                f!(
                    "authentication",
                    "Authentication",
                    FieldKind::Enum {
                        values: LTE_APN_AUTHENTICATION
                    }
                ),
                f!("user", "User", FieldKind::Text),
                f!("password", "Password", FieldKind::Secret),
                f!(
                    "ip-type",
                    "IP Type",
                    FieldKind::Enum {
                        values: LTE_APN_IP_TYPE
                    }
                ),
                f!("use-network-apn", "Use Network APN", FieldKind::Toggle),
                f!("use-peer-dns", "Use Peer DNS", FieldKind::Toggle),
                f!("add-default-route", "Add Default Route", FieldKind::Toggle),
                f!(
                    "default-route-distance",
                    "Default Route Distance",
                    FieldKind::Number
                ),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!(
                    "passthrough-interface",
                    "Passthrough Interface",
                    LOOKUP_IFACE
                ),
                f!("passthrough-mac", "Passthrough MAC", FieldKind::Text),
                f!(
                    "passthrough-subnet-selection",
                    "Passthrough Subnet Selection",
                    FieldKind::Enum {
                        values: LTE_APN_PASSTHROUGH_SUBNET
                    }
                ),
                f!("ipv6-interface", "IPv6 Interface", LOOKUP_IFACE),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("default", "Default", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("apn", "APN", FieldKind::Text)],
    }],
};

pub static WIFI_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ssid"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("configuration", "Configuration", FieldKind::Text),
                f!("master-interface", "Master", LOOKUP_IFACE),
                f!("ssid", "SSID", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "radio",
            label: "Radio",
            read_only: false,
            fields: &[MTU, L2MTU, MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                DEFAULT_NAME,
                f!("radio-mac", "Radio MAC", FieldKind::Readonly),
                f!("current-channel", "Channel", FieldKind::Readonly),
                RUNNING,
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("master-interface", "Master", LOOKUP_IFACE),
            f!("ssid", "SSID", FieldKind::Text),
        ],
    }],
};

pub static WIRELESS_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ssid", "mode"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("ssid", "SSID", FieldKind::Text),
                f!("mode", "Mode", FieldKind::Text),
                f!("band", "Band", FieldKind::Text),
                f!("frequency", "Frequency", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[MTU, MAC],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[DEFAULT_NAME, RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("ssid", "SSID", FieldKind::Text),
            f!("mode", "Mode", FieldKind::Text),
        ],
    }],
};

pub static WIFI_SECURITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["authentication-types"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("authentication-types", "Auth types", FieldKind::Text),
            f!("passphrase", "Passphrase", FieldKind::Secret),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static WIFI_CHANNEL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["band", "frequency"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("band", "Band", FieldKind::Text),
            f!("frequency", "Frequency", FieldKind::Text),
            f!("width", "Width", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static WIFI_DATAPATH_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["bridge"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("bridge", "Bridge", FieldKind::Text),
            f!("vlan-id", "VLAN ID", FieldKind::Number),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static WIFI_CONFIGURATION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ssid"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("ssid", "SSID", FieldKind::Text),
            f!("country", "Country", FieldKind::Text),
            f!("security", "Security", FieldKind::Text),
            f!("datapath", "Datapath", FieldKind::Text),
            f!("channel", "Channel", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("ssid", "SSID", FieldKind::Text)],
    }],
};

pub static WIFI_PROVISIONING_FORM: FormSchema = FormSchema {
    title_key: "action",
    subtitle_keys: &["supported-bands"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("action", "Action", FieldKind::Text),
            f!("supported-bands", "Bands", FieldKind::Text),
            f!("master-configuration", "Master config", FieldKind::Text),
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("action", "Action", FieldKind::Text)],
    }],
};

pub static WIFI_CAP_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["caps-man-addresses"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("caps-man-addresses", "CAPsMAN", FieldKind::Text),
            f!("discovery-interfaces", "Discovery", LOOKUP_IFACES),
        ],
    }],
    create_sections: &[],
};

pub static WIFI_CAPSMAN_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["ca-certificate"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("ca-certificate", "CA certificate", FieldKind::Text),
            f!("certificate", "Certificate", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static WIRELESS_SECURITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["mode"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("mode", "Mode", FieldKind::Text),
            f!("authentication-types", "Auth types", FieldKind::Text),
            f!("wpa2-pre-shared-key", "WPA2 PSK", FieldKind::Secret),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static WIRELESS_ACCESS_LIST_FORM: FormSchema = FormSchema {
    title_key: "mac-address",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            MAC,
            INTERFACE,
            f!("authentication", "Authentication", FieldKind::Toggle),
            f!("forwarding", "Forwarding", FieldKind::Toggle),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[MAC],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{default_writable_value, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    #[test]
    fn list_and_member_create_fields() {
        assert_eq!(
            create_keys(&LIST_FORM),
            ["name", "comment", "include", "exclude"]
        );
        assert!(
            LIST_FORM
                .writable_keys()
                .iter()
                .all(|key| *key != "disabled")
        );
        assert_eq!(
            create_keys(&MEMBER_FORM),
            ["disabled", "list", "interface", "comment"]
        );
        assert_eq!(
            MEMBER_FORM.field("disabled").map(|field| field.label),
            Some("Disabled")
        );
        assert_eq!(
            MEMBER_FORM.field("disabled").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
    }

    #[test]
    fn macsec_create_is_name_and_parent_interface() {
        assert_eq!(create_keys(&MACSEC_FORM), ["name", "interface"]);
        assert_eq!(create_keys(&MACSEC_PROFILE_FORM), ["name"]);
        assert_eq!(
            MACSEC_FORM.field("cak").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert!(MACSEC_FORM.writable_keys().contains(&"ckn"));
        assert!(
            MACSEC_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        assert!(MACSEC_FORM.known_keys().contains(&"status"));
        assert_eq!(
            MACSEC_PROFILE_FORM.known_keys(),
            ["name", "server-priority"]
        );
    }

    fn lookup(resource_id: &'static str, multiple: bool) -> FieldKind {
        FieldKind::Lookup {
            resource_id,
            value_key: "name",
            multiple,
        }
    }

    fn assert_lookup(schema: &FormSchema, key: &str, resource_id: &'static str, multiple: bool) {
        let expected = lookup(resource_id, multiple);
        let fields: Vec<_> = schema
            .sections
            .iter()
            .chain(schema.create_sections.iter())
            .flat_map(|section| section.fields)
            .filter(|field| field.key == key)
            .collect();
        assert!(!fields.is_empty(), "missing field {key}");
        for field in fields {
            assert_eq!(field.kind, expected, "{key}");
        }
    }

    #[test]
    fn interface_lookups_use_interfaces_resource() {
        for schema in [
            &VLAN_FORM,
            &MACVLAN_FORM,
            &VRRP_FORM,
            &MACSEC_FORM,
            &VXLAN_FORM,
            &MEMBER_FORM,
        ] {
            assert_lookup(schema, "interface", "interfaces", false);
        }
        assert_lookup(&WIFI_FORM, "master-interface", "interfaces", false);
        assert_lookup(&BONDING_FORM, "slaves", "interfaces", true);
        assert_lookup(&BONDING_FORM, "primary", "interfaces", false);
        assert_lookup(&VRF_FORM, "interfaces", "interfaces", true);
        assert_eq!(
            VXLAN_FORM.field("group").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            BONDING_FORM.field("mode").map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }

    #[test]
    fn list_and_vrf_lookups() {
        assert_lookup(&MEMBER_FORM, "list", "interface-lists", false);
        assert_lookup(&LIST_FORM, "include", "interface-lists", true);
        assert_lookup(&LIST_FORM, "exclude", "interface-lists", true);
        assert_lookup(&VXLAN_FORM, "vrf", "vrf", false);
        assert_lookup(&MACSEC_FORM, "profile", "macsec-profiles", false);
        for key in [
            "detect-interface-list",
            "lan-interface-list",
            "wan-interface-list",
            "internet-interface-list",
        ] {
            assert_lookup(&DETECT_INTERNET_FORM, key, "interface-lists", false);
        }
    }

    #[test]
    fn patch_body_omits_masked_macsec_cak() {
        let mut original = HashMap::new();
        original.insert("name".into(), "macsec1".into());
        original.insert("interface".into(), "ether1".into());
        original.insert("cak".into(), "********".into());
        original.insert("ckn".into(), "aa".into());
        let mut current = original.clone();
        current.insert("comment".into(), "peer".into());
        let body = patch_body(&MACSEC_FORM, &original, &current, "********");
        assert!(!body.contains_key("cak"));
        assert_eq!(body.get("comment").map(String::as_str), Some("peer"));
    }

    fn assert_enum(schema: &FormSchema, key: &str, values: &'static [&'static str]) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Enum { values })
        );
    }

    fn assert_label(schema: &FormSchema, key: &str, label: &str) {
        assert_eq!(schema.field(key).map(|field| field.label), Some(label));
    }

    #[test]
    fn lte_sheet_puts_apn_profiles_on_the_apn_tab() {
        assert!(LTE_FORM.create_sections.is_empty());
        assert_eq!(
            LTE_FORM
                .sections
                .iter()
                .map(|section| section.id)
                .collect::<Vec<_>>(),
            ["general", "apn", "advanced", "status"]
        );
        assert_lookup(&LTE_FORM, "apn-profiles", "lte-apn", true);
        assert_eq!(
            LTE_FORM.field("network-mode").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            LTE_FORM.field("band").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            LTE_FORM.field("nr-band").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            LTE_FORM.field("pin").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            LTE_FORM.field("allow-roaming").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_enum(&LTE_FORM, "sms-protocol", LTE_SMS_PROTOCOL);
        assert_eq!(
            LTE_FORM.field("sms-read").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_label(&LTE_FORM, "apn-profiles", "APN Profiles");
        assert_label(&LTE_FORM, "network-mode", "Network Mode");
        assert_label(&LTE_FORM, "allow-roaming", "Allow Roaming");
        assert_label(&LTE_FORM, "modem-init", "Modem Init");
        assert!(!LTE_FORM.writable_keys().contains(&"apn"));
        assert!(!LTE_FORM.writable_keys().contains(&"running"));
        assert!(
            LTE_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
    }

    #[test]
    fn lte_apn_kinds_match_webfig() {
        assert_eq!(create_keys(&LTE_APN_FORM), ["name", "apn"]);
        assert_enum(&LTE_APN_FORM, "authentication", LTE_APN_AUTHENTICATION);
        assert_enum(&LTE_APN_FORM, "ip-type", LTE_APN_IP_TYPE);
        assert_enum(
            &LTE_APN_FORM,
            "passthrough-subnet-selection",
            LTE_APN_PASSTHROUGH_SUBNET,
        );
        assert_eq!(
            LTE_APN_FORM.field("password").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            LTE_APN_FORM
                .field("use-network-apn")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            LTE_APN_FORM.field("use-peer-dns").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            LTE_APN_FORM
                .field("add-default-route")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            LTE_APN_FORM
                .field("default-route-distance")
                .map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_lookup(&LTE_APN_FORM, "passthrough-interface", "interfaces", false);
        assert_lookup(&LTE_APN_FORM, "ipv6-interface", "interfaces", false);
        assert_eq!(
            LTE_APN_FORM.field("apn").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            LTE_APN_FORM.field("user").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            LTE_APN_FORM
                .field("passthrough-mac")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_label(&LTE_APN_FORM, "use-network-apn", "Use Network APN");
        assert_label(&LTE_APN_FORM, "add-default-route", "Add Default Route");
        assert_label(
            &LTE_APN_FORM,
            "default-route-distance",
            "Default Route Distance",
        );
        assert_label(
            &LTE_APN_FORM,
            "passthrough-subnet-selection",
            "Passthrough Subnet Selection",
        );
        assert!(!LTE_APN_FORM.writable_keys().contains(&"default"));
        assert_eq!(
            default_writable_value(LTE_APN_FORM.field("authentication").unwrap().kind),
            "none"
        );

        let mut original = HashMap::new();
        original.insert("name".into(), "default".into());
        original.insert("apn".into(), "internet".into());
        original.insert("password".into(), "********".into());
        original.insert("authentication".into(), "chap".into());
        let mut current = original.clone();
        current.insert("apn".into(), "lte.provider".into());
        current.insert("password".into(), "********".into());
        let body = patch_body(&LTE_APN_FORM, &original, &current, "********");
        assert_eq!(body.get("apn").map(String::as_str), Some("lte.provider"));
        assert!(!body.contains_key("password"));
    }
}
