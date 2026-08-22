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
            fields: &[NAME, COMMENT, DISABLED],
        },
        FormSection {
            id: "apn",
            label: "APN",
            read_only: false,
            fields: &[
                f!("apn", "APN", FieldKind::Text),
                f!("network-mode", "Network", FieldKind::Text),
                MTU,
                MAC,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[DEFAULT_NAME, RUNNING],
        },
    ],
    create_sections: &[],
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
}
