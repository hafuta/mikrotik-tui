//! Feature-owned 7.21.5 form schemas for tunnel interfaces.

#![allow(clippy::wildcard_imports)]

use super::*;
use crate::forms::{EnumChoice, FieldPredicate, FieldRule, ScalarKind};

const ARP_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Enabled",
        value: "enabled",
    },
    EnumChoice {
        label: "Disabled",
        value: "disabled",
    },
    EnumChoice {
        label: "Proxy ARP",
        value: "proxy-arp",
    },
    EnumChoice {
        label: "Reply Only",
        value: "reply-only",
    },
    EnumChoice {
        label: "Local Proxy ARP",
        value: "local-proxy-arp",
    },
];
const LOOP_PROTECT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Default",
        value: "default",
    },
    EnumChoice {
        label: "On",
        value: "on",
    },
    EnumChoice {
        label: "Off",
        value: "off",
    },
];
const DONT_FRAGMENT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "No",
        value: "no",
    },
    EnumChoice {
        label: "Yes",
        value: "yes",
    },
    EnumChoice {
        label: "Inherit",
        value: "inherit",
    },
];
const DSCP_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Inherit",
        value: "inherit",
    },
    EnumChoice {
        label: "0",
        value: "0",
    },
    EnumChoice {
        label: "1",
        value: "1",
    },
    EnumChoice {
        label: "2",
        value: "2",
    },
    EnumChoice {
        label: "3",
        value: "3",
    },
    EnumChoice {
        label: "4",
        value: "4",
    },
    EnumChoice {
        label: "5",
        value: "5",
    },
    EnumChoice {
        label: "6",
        value: "6",
    },
    EnumChoice {
        label: "7",
        value: "7",
    },
    EnumChoice {
        label: "8",
        value: "8",
    },
    EnumChoice {
        label: "9",
        value: "9",
    },
    EnumChoice {
        label: "10",
        value: "10",
    },
    EnumChoice {
        label: "11",
        value: "11",
    },
    EnumChoice {
        label: "12",
        value: "12",
    },
    EnumChoice {
        label: "13",
        value: "13",
    },
    EnumChoice {
        label: "14",
        value: "14",
    },
    EnumChoice {
        label: "15",
        value: "15",
    },
    EnumChoice {
        label: "16",
        value: "16",
    },
    EnumChoice {
        label: "17",
        value: "17",
    },
    EnumChoice {
        label: "18",
        value: "18",
    },
    EnumChoice {
        label: "19",
        value: "19",
    },
    EnumChoice {
        label: "20",
        value: "20",
    },
    EnumChoice {
        label: "21",
        value: "21",
    },
    EnumChoice {
        label: "22",
        value: "22",
    },
    EnumChoice {
        label: "23",
        value: "23",
    },
    EnumChoice {
        label: "24",
        value: "24",
    },
    EnumChoice {
        label: "25",
        value: "25",
    },
    EnumChoice {
        label: "26",
        value: "26",
    },
    EnumChoice {
        label: "27",
        value: "27",
    },
    EnumChoice {
        label: "28",
        value: "28",
    },
    EnumChoice {
        label: "29",
        value: "29",
    },
    EnumChoice {
        label: "30",
        value: "30",
    },
    EnumChoice {
        label: "31",
        value: "31",
    },
    EnumChoice {
        label: "32",
        value: "32",
    },
    EnumChoice {
        label: "33",
        value: "33",
    },
    EnumChoice {
        label: "34",
        value: "34",
    },
    EnumChoice {
        label: "35",
        value: "35",
    },
    EnumChoice {
        label: "36",
        value: "36",
    },
    EnumChoice {
        label: "37",
        value: "37",
    },
    EnumChoice {
        label: "38",
        value: "38",
    },
    EnumChoice {
        label: "39",
        value: "39",
    },
    EnumChoice {
        label: "40",
        value: "40",
    },
    EnumChoice {
        label: "41",
        value: "41",
    },
    EnumChoice {
        label: "42",
        value: "42",
    },
    EnumChoice {
        label: "43",
        value: "43",
    },
    EnumChoice {
        label: "44",
        value: "44",
    },
    EnumChoice {
        label: "45",
        value: "45",
    },
    EnumChoice {
        label: "46",
        value: "46",
    },
    EnumChoice {
        label: "47",
        value: "47",
    },
    EnumChoice {
        label: "48",
        value: "48",
    },
    EnumChoice {
        label: "49",
        value: "49",
    },
    EnumChoice {
        label: "50",
        value: "50",
    },
    EnumChoice {
        label: "51",
        value: "51",
    },
    EnumChoice {
        label: "52",
        value: "52",
    },
    EnumChoice {
        label: "53",
        value: "53",
    },
    EnumChoice {
        label: "54",
        value: "54",
    },
    EnumChoice {
        label: "55",
        value: "55",
    },
    EnumChoice {
        label: "56",
        value: "56",
    },
    EnumChoice {
        label: "57",
        value: "57",
    },
    EnumChoice {
        label: "58",
        value: "58",
    },
    EnumChoice {
        label: "59",
        value: "59",
    },
    EnumChoice {
        label: "60",
        value: "60",
    },
    EnumChoice {
        label: "61",
        value: "61",
    },
    EnumChoice {
        label: "62",
        value: "62",
    },
    EnumChoice {
        label: "63",
        value: "63",
    },
];

const OPTIONAL_MTU: FieldSpec = f!(
    "mtu",
    "MTU",
    FieldKind::Optional {
        kind: ScalarKind::Number {
            min: Some(64),
            max: Some(65535)
        },
        unset: "auto",
        unset_label: "Auto"
    }
);
const OPTIONAL_ARP_TIMEOUT: FieldSpec = f!(
    "arp-timeout",
    "ARP Timeout",
    FieldKind::Optional {
        kind: ScalarKind::Time,
        unset: "auto",
        unset_label: "Auto"
    }
);
const OPTIONAL_IPV4_LOCAL: FieldSpec = f!(
    "local-address",
    "Local Address",
    FieldKind::Optional {
        kind: ScalarKind::Ip,
        unset: "0.0.0.0",
        unset_label: "Auto"
    }
);
const OPTIONAL_IPV4_REMOTE: FieldSpec = f!(
    "remote-address",
    "Remote Address",
    FieldKind::Optional {
        kind: ScalarKind::Ip,
        unset: "0.0.0.0",
        unset_label: "Auto"
    }
);
const OPTIONAL_IPV6_LOCAL: FieldSpec = f!(
    "local-address",
    "Local Address",
    FieldKind::Optional {
        kind: ScalarKind::Ipv6,
        unset: "::",
        unset_label: "Auto"
    }
);
const OPTIONAL_IPV6_REMOTE: FieldSpec = f!(
    "remote-address",
    "Remote Address",
    FieldKind::Optional {
        kind: ScalarKind::Ipv6,
        unset: "::",
        unset_label: "Auto"
    }
);
const KEEPALIVE: FieldSpec = f!(
    "keepalive",
    "Keepalive",
    FieldKind::Optional {
        kind: ScalarKind::Raw,
        unset: "disabled",
        unset_label: "Disabled"
    }
);
const DSCP: FieldSpec = f!(
    "dscp",
    "DSCP",
    FieldKind::LabeledEnum {
        choices: DSCP_CHOICES
    }
);
const DONT_FRAGMENT: FieldSpec = f!(
    "dont-fragment",
    "Don't Fragment",
    FieldKind::LabeledEnum {
        choices: DONT_FRAGMENT_CHOICES
    }
);
const TUNNEL_STATUS_FIELDS: &[FieldSpec] = &[
    RUNNING,
    f!("actual-mtu", "Actual MTU", FieldKind::Readonly),
    f!(
        "last-link-up-time",
        "Last Link Up Time",
        FieldKind::Readonly
    ),
    f!(
        "last-link-down-time",
        "Last Link Down Time",
        FieldKind::Readonly
    ),
    f!("link-downs", "Link Downs", FieldKind::Readonly),
];
const TRAFFIC_FIELDS: &[FieldSpec] = &[
    f!("rx-byte", "Rx Bytes", FieldKind::Readonly),
    f!("tx-byte", "Tx Bytes", FieldKind::Readonly),
    f!("rx-packet", "Rx Packets", FieldKind::Readonly),
    f!("tx-packet", "Tx Packets", FieldKind::Readonly),
    f!("rx-drop", "Rx Drops", FieldKind::Readonly),
    f!("tx-drop", "Tx Drops", FieldKind::Readonly),
    f!("rx-error", "Rx Errors", FieldKind::Readonly),
    f!("tx-error", "Tx Errors", FieldKind::Readonly),
];

const LOOP_PROTECT_SECTION: FormSection = FormSection {
    id: "loop-protect",
    label: "Loop Protect",
    read_only: false,
    fields: &[
        f!(
            "loop-protect",
            "Loop Protect",
            FieldKind::LabeledEnum {
                choices: LOOP_PROTECT_CHOICES
            }
        ),
        f!(
            "loop-protect-send-interval",
            "Send Interval",
            FieldKind::Time
        ),
        f!("loop-protect-disable-time", "Disable Time", FieldKind::Time),
        f!(
            "loop-protect-status",
            "Loop Protect Status",
            FieldKind::Readonly
        ),
    ],
};
const STATUS_SECTION: FormSection = FormSection {
    id: "status",
    label: "Status",
    read_only: true,
    fields: TUNNEL_STATUS_FIELDS,
};
const TRAFFIC_SECTION: FormSection = FormSection {
    id: "traffic",
    label: "Traffic",
    read_only: true,
    fields: TRAFFIC_FIELDS,
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
                OPTIONAL_MTU,
                f!("l2mtu", "L2 MTU", FieldKind::Readonly),
                f!("mac-address", "MAC Address", FieldKind::Mac),
                f!(
                    "arp",
                    "ARP",
                    FieldKind::LabeledEnum {
                        choices: ARP_CHOICES
                    }
                ),
                OPTIONAL_ARP_TIMEOUT,
                OPTIONAL_IPV4_LOCAL,
                f!("remote-address", "Remote Address", FieldKind::Ip),
                f!(
                    "tunnel-id",
                    "Tunnel ID",
                    FieldKind::ConstrainedNumber {
                        min: Some(0),
                        max: Some(65535)
                    }
                ),
                KEEPALIVE,
                DSCP,
                f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
                DONT_FRAGMENT,
                f!("allow-fast-path", "Allow Fast Path", FieldKind::Toggle),
                f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
                ENABLED,
                COMMENT,
            ],
        },
        LOOP_PROTECT_SECTION,
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            OPTIONAL_MTU,
            f!("mac-address", "MAC Address", FieldKind::Mac),
            f!(
                "arp",
                "ARP",
                FieldKind::LabeledEnum {
                    choices: ARP_CHOICES
                }
            ),
            OPTIONAL_ARP_TIMEOUT,
            OPTIONAL_IPV4_LOCAL,
            f!("remote-address", "Remote Address", FieldKind::Ip),
            f!(
                "tunnel-id",
                "Tunnel ID",
                FieldKind::ConstrainedNumber {
                    min: Some(0),
                    max: Some(65535)
                }
            ),
            KEEPALIVE,
            DSCP,
            f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
            DONT_FRAGMENT,
            f!("allow-fast-path", "Allow Fast Path", FieldKind::Toggle),
            f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
            ENABLED,
            COMMENT,
        ],
    }],
};

const IPV4_TUNNEL_FIELDS: &[FieldSpec] = &[
    NAME,
    OPTIONAL_MTU,
    OPTIONAL_IPV4_LOCAL,
    f!("remote-address", "Remote Address", FieldKind::Ip),
    KEEPALIVE,
    DSCP,
    f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
    DONT_FRAGMENT,
    f!("allow-fast-path", "Allow Fast Path", FieldKind::Toggle),
    f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
    ENABLED,
    COMMENT,
];

const fn ipv4_tunnel_form() -> FormSchema {
    FormSchema {
        title_key: "name",
        subtitle_keys: &["local-address", "remote-address"],
        sections: &[
            FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: IPV4_TUNNEL_FIELDS,
            },
            STATUS_SECTION,
            TRAFFIC_SECTION,
        ],
        create_sections: &[FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: IPV4_TUNNEL_FIELDS,
        }],
    }
}

pub static IPIP_FORM: FormSchema = ipv4_tunnel_form();
pub static GRE_FORM: FormSchema = ipv4_tunnel_form();

pub static SIX_TO_FOUR_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["local-address", "remote-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                OPTIONAL_MTU,
                OPTIONAL_IPV4_LOCAL,
                OPTIONAL_IPV4_REMOTE,
                KEEPALIVE,
                DSCP,
                f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
                DONT_FRAGMENT,
                f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
                ENABLED,
                COMMENT,
            ],
        },
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            OPTIONAL_MTU,
            OPTIONAL_IPV4_LOCAL,
            OPTIONAL_IPV4_REMOTE,
            KEEPALIVE,
            DSCP,
            f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
            DONT_FRAGMENT,
            f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
            ENABLED,
            COMMENT,
        ],
    }],
};

pub static GRE6_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["local-address", "remote-address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                OPTIONAL_MTU,
                OPTIONAL_IPV6_LOCAL,
                OPTIONAL_IPV6_REMOTE,
                KEEPALIVE,
                DSCP,
                f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
                f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
                ENABLED,
                COMMENT,
            ],
        },
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            OPTIONAL_MTU,
            OPTIONAL_IPV6_LOCAL,
            OPTIONAL_IPV6_REMOTE,
            KEEPALIVE,
            DSCP,
            f!("clamp-tcp-mss", "Clamp TCP MSS", FieldKind::Toggle),
            f!("ipsec-secret", "IPsec Secret", FieldKind::Secret),
            ENABLED,
            COMMENT,
        ],
    }],
};

const VXLAN_IP_VERSION: &[EnumChoice] = &[
    EnumChoice {
        label: "IPv4",
        value: "ipv4",
    },
    EnumChoice {
        label: "IPv6",
        value: "ipv6",
    },
];
const VXLAN_REM_CSUM: &[EnumChoice] = &[
    EnumChoice {
        label: "None",
        value: "none",
    },
    EnumChoice {
        label: "Rx",
        value: "rx",
    },
    EnumChoice {
        label: "Tx",
        value: "tx",
    },
    EnumChoice {
        label: "Both",
        value: "both",
    },
];

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
                OPTIONAL_MTU,
                f!("l2mtu", "L2 MTU", FieldKind::Readonly),
                f!("mac-address", "MAC Address", FieldKind::Mac),
                f!(
                    "arp",
                    "ARP",
                    FieldKind::LabeledEnum {
                        choices: ARP_CHOICES
                    }
                ),
                OPTIONAL_ARP_TIMEOUT,
                f!(
                    "vni",
                    "VNI",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(16_777_215)
                    }
                ),
                f!(
                    "vteps-ip-version",
                    "VTEPs IP Version",
                    FieldKind::LabeledEnum {
                        choices: VXLAN_IP_VERSION
                    }
                ),
                f!(
                    "local-address",
                    "Local Address",
                    FieldKind::Optional {
                        kind: ScalarKind::Raw,
                        unset: "",
                        unset_label: "Auto"
                    }
                ),
                f!(
                    "group",
                    "Group",
                    FieldKind::Optional {
                        kind: ScalarKind::Raw,
                        unset: "",
                        unset_label: "None"
                    }
                ),
                INTERFACE,
                f!("vtep-vrf", "VTEP VRF", LOOKUP_VRF),
                f!(
                    "port",
                    "Port",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(65535)
                    }
                ),
                DONT_FRAGMENT,
                f!("allow-fast-path", "Allow Fast Path", FieldKind::Toggle),
                f!("learning", "Learning", FieldKind::Toggle),
                f!("checksum", "Checksum", FieldKind::Toggle),
                f!(
                    "rem-csum",
                    "Remote Checksum Offload",
                    FieldKind::LabeledEnum {
                        choices: VXLAN_REM_CSUM
                    }
                ),
                f!(
                    "max-fdb-size",
                    "Max FDB Size",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(65535)
                    }
                ),
                f!(
                    "ttl",
                    "TTL",
                    FieldKind::Optional {
                        kind: ScalarKind::Number {
                            min: Some(1),
                            max: Some(255)
                        },
                        unset: "auto",
                        unset_label: "Auto"
                    }
                ),
                f!("hw", "Hardware Offload", FieldKind::Toggle),
                f!("bridge", "Bridge", LOOKUP_IFACE),
                f!(
                    "bridge-pvid",
                    "Bridge PVID",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(4094)
                    }
                ),
                ENABLED,
                COMMENT,
            ],
        },
        LOOP_PROTECT_SECTION,
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[],
};

const VRRP_VERSION: &[EnumChoice] = &[
    EnumChoice {
        label: "2",
        value: "2",
    },
    EnumChoice {
        label: "3",
        value: "3",
    },
];
const VRRP_PROTOCOL: &[EnumChoice] = &[
    EnumChoice {
        label: "IPv4",
        value: "ipv4",
    },
    EnumChoice {
        label: "IPv6",
        value: "ipv6",
    },
];
const CONNECTION_TRACKING_MODE: &[EnumChoice] = &[
    EnumChoice {
        label: "Active",
        value: "active",
    },
    EnumChoice {
        label: "Passive",
        value: "passive",
    },
];

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
                f!(
                    "vrid",
                    "VRID",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(255)
                    }
                ),
                f!(
                    "priority",
                    "Priority",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(254)
                    }
                ),
                f!("interval", "Interval", FieldKind::Time),
                f!(
                    "version",
                    "Version",
                    FieldKind::LabeledEnum {
                        choices: VRRP_VERSION
                    }
                ),
                f!(
                    "v3-protocol",
                    "V3 Protocol",
                    FieldKind::LabeledEnum {
                        choices: VRRP_PROTOCOL
                    }
                ),
                f!("preemption-mode", "Preemption Mode", FieldKind::Toggle),
                f!("group-authority", "Group Authority", FieldKind::Text),
                f!(
                    "sync-connection-tracking",
                    "Sync Connection Tracking",
                    FieldKind::Toggle
                ),
                f!(
                    "remote-address",
                    "Remote Address",
                    FieldKind::Optional {
                        kind: ScalarKind::Ip,
                        unset: "",
                        unset_label: "Auto"
                    }
                ),
                f!(
                    "connection-tracking-mode",
                    "Connection Tracking Mode",
                    FieldKind::LabeledEnum {
                        choices: CONNECTION_TRACKING_MODE
                    }
                ),
                f!(
                    "connection-tracking-port",
                    "Connection Tracking Port",
                    FieldKind::ConstrainedNumber {
                        min: Some(1),
                        max: Some(65535)
                    }
                ),
                ENABLED,
                COMMENT,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                RUNNING,
                f!("state", "State", FieldKind::Readonly),
                f!("master", "Master", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
        TRAFFIC_SECTION,
    ],
    create_sections: &[],
};

const BONDING_MODE: &[EnumChoice] = &[
    EnumChoice {
        label: "Balance RR",
        value: "balance-rr",
    },
    EnumChoice {
        label: "Active Backup",
        value: "active-backup",
    },
    EnumChoice {
        label: "Balance XOR",
        value: "balance-xor",
    },
    EnumChoice {
        label: "Broadcast",
        value: "broadcast",
    },
    EnumChoice {
        label: "802.3ad",
        value: "802.3ad",
    },
    EnumChoice {
        label: "Balance TLB",
        value: "balance-tlb",
    },
    EnumChoice {
        label: "Balance ALB",
        value: "balance-alb",
    },
];
const LINK_MONITORING: &[EnumChoice] = &[
    EnumChoice {
        label: "None",
        value: "none",
    },
    EnumChoice {
        label: "ARP",
        value: "arp",
    },
    EnumChoice {
        label: "MII",
        value: "mii",
    },
];
const HASH_POLICY: &[EnumChoice] = &[
    EnumChoice {
        label: "Layer 2",
        value: "layer-2",
    },
    EnumChoice {
        label: "Layer 2 and 3",
        value: "layer-2-and-3",
    },
    EnumChoice {
        label: "Layer 3 and 4",
        value: "layer-3-and-4",
    },
];
const LACP_RATE: &[EnumChoice] = &[
    EnumChoice {
        label: "30 seconds",
        value: "30secs",
    },
    EnumChoice {
        label: "1 second",
        value: "1sec",
    },
];
const LACP_MODE: &[EnumChoice] = &[
    EnumChoice {
        label: "Passive",
        value: "passive",
    },
    EnumChoice {
        label: "Active",
        value: "active",
    },
];

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
                OPTIONAL_MTU,
                f!("mac-address", "MAC Address", FieldKind::Mac),
                f!(
                    "arp",
                    "ARP",
                    FieldKind::LabeledEnum {
                        choices: ARP_CHOICES
                    }
                ),
                OPTIONAL_ARP_TIMEOUT,
                f!("slaves", "Slaves", LOOKUP_IFACES),
                f!(
                    "mode",
                    "Mode",
                    FieldKind::LabeledEnum {
                        choices: BONDING_MODE
                    }
                ),
                f!("primary", "Primary", LOOKUP_IFACE),
                f!(
                    "link-monitoring",
                    "Link Monitoring",
                    FieldKind::LabeledEnum {
                        choices: LINK_MONITORING
                    }
                ),
                f!("arp-interval", "ARP Interval", FieldKind::Time),
                f!("arp-ip-targets", "ARP IP Targets", FieldKind::Repeat),
                f!("mii-interval", "MII Interval", FieldKind::Time),
                f!("down-delay", "Down Delay", FieldKind::Time),
                f!("up-delay", "Up Delay", FieldKind::Time),
                f!(
                    "transmit-hash-policy",
                    "Transmit Hash Policy",
                    FieldKind::LabeledEnum {
                        choices: HASH_POLICY
                    }
                ),
                f!(
                    "lacp-rate",
                    "LACP Rate",
                    FieldKind::LabeledEnum { choices: LACP_RATE }
                ),
                f!(
                    "lacp-mode",
                    "LACP Mode",
                    FieldKind::LabeledEnum { choices: LACP_MODE }
                ),
                f!(
                    "lacp-user-key",
                    "LACP User Key",
                    FieldKind::ConstrainedNumber {
                        min: Some(0),
                        max: Some(1023)
                    }
                ),
                f!("lacp-system-id", "LACP System ID", FieldKind::Mac),
                f!(
                    "lacp-system-priority",
                    "LACP System Priority",
                    FieldKind::ConstrainedNumber {
                        min: Some(0),
                        max: Some(65535)
                    }
                ),
                f!(
                    "min-links",
                    "Minimum Links",
                    FieldKind::ConstrainedNumber {
                        min: Some(0),
                        max: Some(32)
                    }
                ),
                ENABLED,
                COMMENT,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                RUNNING,
                f!("active-ports", "Active Ports", FieldKind::Readonly),
                f!("inactive-ports", "Inactive Ports", FieldKind::Readonly),
            ],
        },
        TRAFFIC_SECTION,
    ],
    create_sections: &[],
};

const VRRP_V3: FieldPredicate = FieldPredicate::Equals {
    key: "version",
    value: "3",
};
const VRRP_SYNC: FieldPredicate = FieldPredicate::Truthy("sync-connection-tracking");
const VXLAN_GROUP: FieldPredicate = FieldPredicate::NonEmpty("group");
const VXLAN_BRIDGE: FieldPredicate = FieldPredicate::NonEmpty("bridge");
const BOND_ACTIVE_BACKUP: FieldPredicate = FieldPredicate::Equals {
    key: "mode",
    value: "active-backup",
};
const BOND_ARP: FieldPredicate = FieldPredicate::Equals {
    key: "link-monitoring",
    value: "arp",
};
const BOND_MII: FieldPredicate = FieldPredicate::Equals {
    key: "link-monitoring",
    value: "mii",
};
const BOND_LACP: FieldPredicate = FieldPredicate::Equals {
    key: "mode",
    value: "802.3ad",
};
const BOND_HASH: FieldPredicate = FieldPredicate::Any(&[
    FieldPredicate::Equals {
        key: "mode",
        value: "balance-xor",
    },
    FieldPredicate::Equals {
        key: "mode",
        value: "802.3ad",
    },
    FieldPredicate::Equals {
        key: "mode",
        value: "balance-tlb",
    },
    FieldPredicate::Equals {
        key: "mode",
        value: "balance-alb",
    },
]);

/// Feature-local rules, aggregated by `interfaces::rules`.
pub(crate) const FIELD_RULES: &[FieldRule] = &[
    FieldRule {
        resource_id: "vxlan",
        field_key: "interface",
        visible: VXLAN_GROUP,
        enabled: VXLAN_GROUP,
    },
    FieldRule {
        resource_id: "vxlan",
        field_key: "vtep-vrf",
        visible: FieldPredicate::Not(crate::forms::BoxedFieldPredicate(&VXLAN_GROUP)),
        enabled: FieldPredicate::Not(crate::forms::BoxedFieldPredicate(&VXLAN_GROUP)),
    },
    FieldRule {
        resource_id: "vxlan",
        field_key: "bridge-pvid",
        visible: VXLAN_BRIDGE,
        enabled: VXLAN_BRIDGE,
    },
    FieldRule {
        resource_id: "vrrp",
        field_key: "v3-protocol",
        visible: VRRP_V3,
        enabled: VRRP_V3,
    },
    FieldRule {
        resource_id: "vrrp",
        field_key: "remote-address",
        visible: VRRP_SYNC,
        enabled: VRRP_SYNC,
    },
    FieldRule {
        resource_id: "vrrp",
        field_key: "connection-tracking-mode",
        visible: VRRP_SYNC,
        enabled: VRRP_SYNC,
    },
    FieldRule {
        resource_id: "vrrp",
        field_key: "connection-tracking-port",
        visible: VRRP_SYNC,
        enabled: VRRP_SYNC,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "primary",
        visible: BOND_ACTIVE_BACKUP,
        enabled: BOND_ACTIVE_BACKUP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "arp-interval",
        visible: BOND_ARP,
        enabled: BOND_ARP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "arp-ip-targets",
        visible: BOND_ARP,
        enabled: BOND_ARP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "mii-interval",
        visible: BOND_MII,
        enabled: BOND_MII,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "down-delay",
        visible: BOND_MII,
        enabled: BOND_MII,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "up-delay",
        visible: BOND_MII,
        enabled: BOND_MII,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "transmit-hash-policy",
        visible: BOND_HASH,
        enabled: BOND_HASH,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "lacp-rate",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "lacp-mode",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "lacp-user-key",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "lacp-system-id",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "lacp-system-priority",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
    FieldRule {
        resource_id: "bonding",
        field_key: "min-links",
        visible: BOND_LACP,
        enabled: BOND_LACP,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{evaluate_field_rules, patch_body};
    use std::collections::HashMap;

    fn section_keys(schema: &FormSchema, id: &str) -> Vec<&'static str> {
        schema
            .sections
            .iter()
            .find(|section| section.id == id)
            .unwrap()
            .fields
            .iter()
            .map(|field| field.key)
            .collect()
    }

    #[test]
    fn eoip_matches_webfig_section_and_field_order() {
        assert_eq!(
            EOIP_FORM
                .sections
                .iter()
                .map(|section| section.id)
                .collect::<Vec<_>>(),
            ["general", "loop-protect", "status", "traffic"]
        );
        assert_eq!(
            section_keys(&EOIP_FORM, "general"),
            [
                "name",
                "mtu",
                "l2mtu",
                "mac-address",
                "arp",
                "arp-timeout",
                "local-address",
                "remote-address",
                "tunnel-id",
                "keepalive",
                "dscp",
                "clamp-tcp-mss",
                "dont-fragment",
                "allow-fast-path",
                "ipsec-secret",
                "disabled",
                "comment"
            ]
        );
        assert!(matches!(
            EOIP_FORM.field("mtu").unwrap().kind,
            FieldKind::Optional { .. }
        ));
        assert_eq!(
            EOIP_FORM.field("ipsec-secret").unwrap().kind,
            FieldKind::Secret
        );
        assert_eq!(EOIP_FORM.field("l2mtu").unwrap().kind, FieldKind::Readonly);
    }

    #[test]
    fn tunnel_variants_keep_distinct_contracts() {
        assert!(IPIP_FORM.field("allow-fast-path").is_some());
        assert!(GRE_FORM.field("allow-fast-path").is_some());
        assert!(SIX_TO_FOUR_FORM.field("allow-fast-path").is_none());
        assert!(matches!(
            SIX_TO_FOUR_FORM.field("remote-address").unwrap().kind,
            FieldKind::Optional {
                kind: ScalarKind::Ip,
                ..
            }
        ));
        assert!(GRE6_FORM.field("allow-fast-path").is_none());
        assert!(GRE6_FORM.field("dont-fragment").is_none());
        assert!(matches!(
            GRE6_FORM.field("local-address").unwrap().kind,
            FieldKind::Optional {
                kind: ScalarKind::Ipv6,
                ..
            }
        ));
    }

    #[test]
    fn display_choices_preserve_routeros_wire_values() {
        assert_eq!(ENABLED.kind.display_value("false"), "yes");
        assert_eq!(ENABLED.kind.display_value("true"), "no");
        assert_eq!(
            EOIP_FORM
                .field("arp")
                .unwrap()
                .kind
                .display_value("proxy-arp"),
            "Proxy ARP"
        );
        assert_eq!(DSCP.kind.display_value("inherit"), "Inherit");
        assert_eq!(DONT_FRAGMENT.kind.display_value("no"), "No");
        assert_eq!(
            EOIP_FORM
                .field("loop-protect")
                .unwrap()
                .kind
                .display_value("default"),
            "Default"
        );
    }

    #[test]
    fn readonly_status_and_traffic_never_enter_patch() {
        let original = HashMap::from([
            ("name".to_string(), "eoip1".to_string()),
            ("running".to_string(), "false".to_string()),
            ("rx-byte".to_string(), "1".to_string()),
        ]);
        let current = HashMap::from([
            ("name".to_string(), "eoip1".to_string()),
            ("running".to_string(), "true".to_string()),
            ("rx-byte".to_string(), "2".to_string()),
        ]);
        let body = patch_body(&EOIP_FORM, &original, &current, "********");
        assert!(!body.contains_key("running"));
        assert!(!body.contains_key("rx-byte"));
    }

    #[test]
    fn conditional_rules_match_webfig_dependencies() {
        let values = HashMap::from([
            ("mode".to_string(), "802.3ad".to_string()),
            ("link-monitoring".to_string(), "mii".to_string()),
        ]);
        assert_eq!(
            evaluate_field_rules(FIELD_RULES, "bonding", "lacp-rate", &values),
            Some((true, true))
        );
        assert_eq!(
            evaluate_field_rules(FIELD_RULES, "bonding", "arp-ip-targets", &values),
            Some((false, false))
        );
        let values = HashMap::from([("version".to_string(), "2".to_string())]);
        assert_eq!(
            evaluate_field_rules(FIELD_RULES, "vrrp", "v3-protocol", &values),
            Some((false, false))
        );
    }
}
