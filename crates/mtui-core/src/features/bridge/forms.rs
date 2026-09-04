//! Feature-owned form schemas for the Bridge navigation group.

use crate::form_fields::{
    FIELD_BRIDGE_FILTER_ACTION, FIELD_BRIDGE_FILTER_CHAIN, FIELD_BRIDGE_NAT_ACTION,
    FIELD_BRIDGE_NAT_CHAIN, FIELD_IP_PROTOCOL, FIELD_MAC_PROTOCOL, KIND_BRIDGE_PORT_PRIORITY,
    KIND_BRIDGE_PRIORITY, LOOKUP_INTERFACES as LOOKUP_IFACE,
    LOOKUP_INTERFACES_MULTI as LOOKUP_IFACES,
};
use crate::forms::{EnumChoice, FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_BRIDGE: FieldKind = FieldKind::Lookup {
    resource_id: "bridges",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SWITCH: FieldKind = FieldKind::Lookup {
    resource_id: "switch",
    value_key: "name",
    multiple: false,
};

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

const FRAME_TYPES_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "admit all",
        value: "admit-all",
    },
    EnumChoice {
        label: "admit only VLAN tagged",
        value: "admit-only-vlan-tagged",
    },
    EnumChoice {
        label: "admit only untagged and priority tagged",
        value: "admit-only-untagged-and-priority-tagged",
    },
];
const PROTOCOL_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "stp",
        value: "stp",
    },
    EnumChoice {
        label: "rstp",
        value: "rstp",
    },
    EnumChoice {
        label: "mstp",
        value: "mstp",
    },
];
const EDGE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "yes",
        value: "yes",
    },
    EnumChoice {
        label: "no",
        value: "no",
    },
];
const LEARN_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "yes",
        value: "yes",
    },
    EnumChoice {
        label: "no",
        value: "no",
    },
];
const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const BRIDGE: FieldSpec = f!("bridge", "Bridge", LOOKUP_BRIDGE);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);
const PVID: FieldSpec = f!(
    "pvid",
    "PVID",
    FieldKind::ConstrainedNumber {
        min: Some(1),
        max: Some(4095)
    }
);
const FRAME_TYPES: FieldSpec = f!(
    "frame-types",
    "Frame Types",
    FieldKind::LabeledEnum {
        choices: FRAME_TYPES_CHOICES
    }
);
const INGRESS: FieldSpec = f!("ingress-filtering", "Ingress filtering", FieldKind::Toggle);
const BRIDGE_PRIORITY: FieldSpec = f!("priority", "Priority", KIND_BRIDGE_PRIORITY);
const PORT_PRIORITY: FieldSpec = f!("priority", "Priority", KIND_BRIDGE_PORT_PRIORITY);
const SRC_MAC: FieldSpec = f!("src-mac-address", "Src MAC", FieldKind::Text);
const DST_MAC: FieldSpec = f!("dst-mac-address", "Dst MAC", FieldKind::Text);
const IN_IFACE: FieldSpec = f!("in-interface", "In interface", LOOKUP_IFACE);
const OUT_IFACE: FieldSpec = f!("out-interface", "Out interface", LOOKUP_IFACE);
const PACKETS: FieldSpec = f!("packets", "Packets", FieldKind::Readonly);
const BYTES: FieldSpec = f!("bytes", "Bytes", FieldKind::Readonly);
const SWITCH: FieldSpec = f!("switch", "Switch", LOOKUP_SWITCH);
const CONTROL_PORTS: FieldSpec = f!("control-ports", "Control ports", LOOKUP_IFACES);
const STATUS: FieldSpec = f!("status", "Status", FieldKind::Readonly);
const DYNAMIC: FieldSpec = f!("dynamic", "Dynamic", FieldKind::Readonly);
const VLAN_IDS: FieldSpec = f!("vlan-ids", "VLAN IDs", FieldKind::Repeat);
const ARP: FieldSpec = f!(
    "arp",
    "ARP",
    FieldKind::LabeledEnum {
        choices: ARP_CHOICES
    }
);

pub static BRIDGE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["protocol-mode"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                ARP,
                f!(
                    "mtu",
                    "MTU",
                    FieldKind::ConstrainedNumber {
                        min: Some(64),
                        max: Some(65_535)
                    }
                ),
                f!("mac-address", "MAC Address", FieldKind::Mac),
                f!("fast-forward", "Fast Forward", FieldKind::Toggle),
                f!("igmp-snooping", "IGMP Snooping", FieldKind::Toggle),
                f!("dhcp-snooping", "DHCP Snooping", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "stp",
            label: "STP",
            read_only: false,
            fields: &[
                f!(
                    "protocol-mode",
                    "Protocol Mode",
                    FieldKind::LabeledEnum {
                        choices: PROTOCOL_MODE_CHOICES
                    }
                ),
                BRIDGE_PRIORITY,
                f!("region-name", "Region Name", FieldKind::Text),
            ],
        },
        FormSection {
            id: "vlan",
            label: "VLAN",
            read_only: false,
            fields: &[
                f!("vlan-filtering", "VLAN Filtering", FieldKind::Toggle),
                PVID,
                FRAME_TYPES,
                INGRESS,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("running", "Running", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, COMMENT],
    }],
};

pub static BRIDGE_PORT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["bridge"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                INTERFACE,
                BRIDGE,
                PVID,
                f!("hw", "Hardware Offload", FieldKind::Toggle),
                FRAME_TYPES,
                INGRESS,
                f!("trusted", "Trusted", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "stp",
            label: "STP",
            read_only: false,
            fields: &[
                f!(
                    "edge",
                    "Edge",
                    FieldKind::LabeledEnum {
                        choices: EDGE_CHOICES
                    }
                ),
                f!("horizon", "Horizon", FieldKind::Text),
                f!("path-cost", "Path Cost", FieldKind::Text),
                PORT_PRIORITY,
                f!("bpdu-guard", "BPDU Guard", FieldKind::Toggle),
                f!("restricted-role", "Restricted Role", FieldKind::Toggle),
                f!(
                    "learn",
                    "Learn",
                    FieldKind::LabeledEnum {
                        choices: LEARN_CHOICES
                    }
                ),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[INTERFACE, BRIDGE],
    }],
};

pub static BRIDGE_VLAN_FORM: FormSchema = FormSchema {
    title_key: "vlan-ids",
    subtitle_keys: &["bridge"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                BRIDGE,
                VLAN_IDS,
                f!("tagged", "Tagged", LOOKUP_IFACES),
                f!("untagged", "Untagged", LOOKUP_IFACES),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("current-tagged", "Current tagged", FieldKind::Readonly),
                f!("current-untagged", "Current untagged", FieldKind::Readonly),
                DYNAMIC,
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[BRIDGE, VLAN_IDS],
    }],
};

pub static BRIDGE_MDB_FORM: FormSchema = FormSchema {
    title_key: "group",
    subtitle_keys: &["bridge"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("group", "Group", FieldKind::Text),
                f!("vid", "VID", FieldKind::Text),
                f!("on-ports", "On ports", LOOKUP_IFACES),
                BRIDGE,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[DYNAMIC],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("group", "Group", FieldKind::Text), BRIDGE],
    }],
};

pub static BRIDGE_MSTI_FORM: FormSchema = FormSchema {
    title_key: "identifier",
    subtitle_keys: &["bridge"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            BRIDGE,
            f!("identifier", "Identifier", FieldKind::Text),
            f!("vlan-mapping", "VLAN mapping", FieldKind::Text),
            BRIDGE_PRIORITY,
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[BRIDGE, f!("identifier", "Identifier", FieldKind::Text)],
    }],
};

pub static BRIDGE_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                FIELD_BRIDGE_FILTER_CHAIN,
                FIELD_BRIDGE_FILTER_ACTION,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                FIELD_MAC_PROTOCOL,
                SRC_MAC,
                DST_MAC,
                IN_IFACE,
                OUT_IFACE,
                FIELD_IP_PROTOCOL,
                f!("src-address", "Source", FieldKind::Text),
                f!("dst-address", "Destination", FieldKind::Text),
                f!("src-port", "Src port", FieldKind::Text),
                f!("dst-port", "Dst port", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[PACKETS, BYTES],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FIELD_BRIDGE_FILTER_CHAIN, FIELD_BRIDGE_FILTER_ACTION],
    }],
};

pub static BRIDGE_NAT_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                FIELD_BRIDGE_NAT_CHAIN,
                FIELD_BRIDGE_NAT_ACTION,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                FIELD_MAC_PROTOCOL,
                SRC_MAC,
                DST_MAC,
                IN_IFACE,
                OUT_IFACE,
                f!("to-src-mac-address", "To src MAC", FieldKind::Text),
                f!("to-dst-mac-address", "To dst MAC", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[PACKETS, BYTES],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FIELD_BRIDGE_NAT_CHAIN, FIELD_BRIDGE_NAT_ACTION],
    }],
};

pub static BRIDGE_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "use-ip-firewall",
    subtitle_keys: &[],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("use-ip-firewall", "IP firewall", FieldKind::Toggle),
                f!(
                    "use-ip-firewall-for-vlan",
                    "VLAN firewall",
                    FieldKind::Toggle
                ),
                f!(
                    "use-ip-firewall-for-pppoe",
                    "PPPoE firewall",
                    FieldKind::Toggle
                ),
                f!("allow-fast-path", "Fast path", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!(
                    "bridge-fast-path-active",
                    "Fast path active",
                    FieldKind::Readonly
                ),
                f!(
                    "bridge-fast-path-packets",
                    "Fast path packets",
                    FieldKind::Readonly
                ),
                f!(
                    "bridge-fast-path-bytes",
                    "Fast path bytes",
                    FieldKind::Readonly
                ),
                f!(
                    "bridge-fast-forward-packets",
                    "Fast forward packets",
                    FieldKind::Readonly
                ),
                f!(
                    "bridge-fast-forward-bytes",
                    "Fast forward bytes",
                    FieldKind::Readonly
                ),
            ],
        },
    ],
    create_sections: &[],
};

pub static BRIDGE_PORT_CONTROLLER_FORM: FormSchema = FormSchema {
    title_key: "bridge",
    subtitle_keys: &["switch"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            BRIDGE,
            SWITCH,
            f!("cascade-ports", "Cascade", LOOKUP_IFACES),
        ],
    }],
    create_sections: &[],
};

pub static BRIDGE_PORT_CONTROLLER_DEVICE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["pe-mac"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("pe-mac", "PE MAC", FieldKind::Text),
                f!("descr", "Description", FieldKind::Text),
                CONTROL_PORTS,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[STATUS],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("pe-mac", "PE MAC", FieldKind::Text)],
    }],
};

pub static BRIDGE_PORT_CONTROLLER_PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["device"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("device", "Device", FieldKind::Text),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                STATUS,
                f!("port-status", "Port status", FieldKind::Readonly),
                f!("rate", "Rate", FieldKind::Readonly),
                f!("pcid", "PCID", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("device", "Device", FieldKind::Text)],
    }],
};

pub static BRIDGE_PORT_EXTENDER_FORM: FormSchema = FormSchema {
    title_key: "switch",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            SWITCH,
            CONTROL_PORTS,
            f!("excluded-ports", "Excluded", LOOKUP_IFACES),
        ],
    }],
    create_sections: &[],
};
