//! Form schemas for the Bridge nav group.

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
const BRIDGE: FieldSpec = f!("bridge", "Bridge", FieldKind::Text);
const PVID: FieldSpec = f!("pvid", "PVID", FieldKind::Number);
const FRAME_TYPES: FieldSpec = f!("frame-types", "Frame types", FieldKind::Text);
const INGRESS: FieldSpec = f!("ingress-filtering", "Ingress filtering", FieldKind::Toggle);
const PRIORITY: FieldSpec = f!("priority", "Priority", FieldKind::Text);
const CHAIN: FieldSpec = f!("chain", "Chain", FieldKind::Text);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const MAC_PROTOCOL: FieldSpec = f!("mac-protocol", "MAC protocol", FieldKind::Text);
const SRC_MAC: FieldSpec = f!("src-mac-address", "Src MAC", FieldKind::Text);
const DST_MAC: FieldSpec = f!("dst-mac-address", "Dst MAC", FieldKind::Text);
const IN_IFACE: FieldSpec = f!("in-interface", "In interface", FieldKind::Text);
const OUT_IFACE: FieldSpec = f!("out-interface", "Out interface", FieldKind::Text);
const PACKETS: FieldSpec = f!("packets", "Packets", FieldKind::Readonly);
const BYTES: FieldSpec = f!("bytes", "Bytes", FieldKind::Readonly);
const SWITCH: FieldSpec = f!("switch", "Switch", FieldKind::Text);
const CONTROL_PORTS: FieldSpec = f!("control-ports", "Control ports", FieldKind::Text);
const STATUS: FieldSpec = f!("status", "Status", FieldKind::Readonly);
const DYNAMIC: FieldSpec = f!("dynamic", "Dynamic", FieldKind::Readonly);

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
                f!("protocol-mode", "Protocol mode", FieldKind::Text),
                f!("vlan-filtering", "VLAN filtering", FieldKind::Toggle),
                PVID,
                f!("igmp-snooping", "IGMP snooping", FieldKind::Toggle),
                f!("dhcp-snooping", "DHCP snooping", FieldKind::Toggle),
                f!("arp", "ARP", FieldKind::Enum { values: ARP_VALUES }),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("mtu", "MTU", FieldKind::Number),
                f!("mac-address", "MAC address", FieldKind::Text),
                f!("fast-forward", "Fast forward", FieldKind::Toggle),
                FRAME_TYPES,
                INGRESS,
                PRIORITY,
                f!("region-name", "Region", FieldKind::Text),
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
                f!("interface", "Interface", FieldKind::Text),
                BRIDGE,
                PVID,
                f!("hw", "HW", FieldKind::Toggle),
                f!("edge", "Edge", FieldKind::Text),
                FRAME_TYPES,
                INGRESS,
                f!("trusted", "Trusted", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("horizon", "Horizon", FieldKind::Text),
                f!("path-cost", "Path cost", FieldKind::Text),
                PRIORITY,
                f!("bpdu-guard", "BPDU guard", FieldKind::Toggle),
                f!("restricted-role", "Restricted role", FieldKind::Toggle),
                f!("learn", "Learn", FieldKind::Text),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("interface", "Interface", FieldKind::Text), BRIDGE],
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
                f!("vlan-ids", "VLAN IDs", FieldKind::Text),
                f!("tagged", "Tagged", FieldKind::Text),
                f!("untagged", "Untagged", FieldKind::Text),
                COMMENT,
                DISABLED,
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
        fields: &[BRIDGE, f!("vlan-ids", "VLAN IDs", FieldKind::Text)],
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
                f!("on-ports", "On ports", FieldKind::Text),
                BRIDGE,
                DISABLED,
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
            PRIORITY,
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
            fields: &[CHAIN, ACTION, COMMENT, DISABLED],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MAC_PROTOCOL,
                SRC_MAC,
                DST_MAC,
                IN_IFACE,
                OUT_IFACE,
                f!("ip-protocol", "IP protocol", FieldKind::Text),
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
        fields: &[CHAIN, ACTION],
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
            fields: &[CHAIN, ACTION, COMMENT, DISABLED],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                MAC_PROTOCOL,
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
        fields: &[CHAIN, ACTION],
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
            f!("cascade-ports", "Cascade", FieldKind::Text),
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
                DISABLED,
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
            f!("excluded-ports", "Excluded", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::extra_status_fields;
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn section_ids(schema: &FormSchema) -> Vec<&'static str> {
        schema.sections.iter().map(|section| section.id).collect()
    }

    fn advanced_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .sections
            .iter()
            .find(|section| section.id == "advanced")
            .map(|section| section.fields.iter().map(|field| field.key).collect())
            .unwrap_or_default()
    }

    fn assert_status_readonly(schema: &FormSchema) {
        let writable = schema.writable_keys();
        let Some(status) = schema
            .sections
            .iter()
            .find(|section| section.id == "status")
        else {
            return;
        };
        assert!(status.read_only);
        for field in status.fields {
            assert_eq!(field.kind, FieldKind::Readonly);
            assert!(
                !writable.contains(&field.key),
                "status key {} must not be writable",
                field.key
            );
        }
    }

    #[test]
    fn bridge_form_writable_status_and_short_create() {
        assert_eq!(section_ids(&BRIDGE_FORM), ["general", "advanced", "status"]);
        assert_eq!(
            BRIDGE_FORM.writable_keys(),
            [
                "name",
                "protocol-mode",
                "vlan-filtering",
                "pvid",
                "igmp-snooping",
                "dhcp-snooping",
                "arp",
                "comment",
                "disabled",
                "mtu",
                "mac-address",
                "fast-forward",
                "frame-types",
                "ingress-filtering",
                "priority",
                "region-name",
            ]
        );
        assert!(BRIDGE_FORM.known_keys().contains(&"running"));
        assert!(!BRIDGE_FORM.writable_keys().contains(&"running"));
        assert_eq!(create_keys(&BRIDGE_FORM), ["name", "comment"]);
        assert_eq!(
            advanced_keys(&BRIDGE_FORM),
            [
                "mtu",
                "mac-address",
                "fast-forward",
                "frame-types",
                "ingress-filtering",
                "priority",
                "region-name",
            ]
        );
        assert_status_readonly(&BRIDGE_FORM);
        assert!(matches!(
            BRIDGE_FORM.field("arp").map(|field| field.kind),
            Some(FieldKind::Enum { values }) if values == ARP_VALUES
        ));
    }

    #[test]
    fn bridge_port_form_splits_stp_to_advanced() {
        assert_eq!(section_ids(&BRIDGE_PORT_FORM), ["general", "advanced"]);
        assert!(!BRIDGE_PORT_FORM.writable_keys().contains(&"role"));
        assert_eq!(create_keys(&BRIDGE_PORT_FORM), ["interface", "bridge"]);
        assert_eq!(
            advanced_keys(&BRIDGE_PORT_FORM),
            [
                "horizon",
                "path-cost",
                "priority",
                "bpdu-guard",
                "restricted-role",
                "learn",
            ]
        );
        assert_status_readonly(&BRIDGE_PORT_FORM);
    }

    #[test]
    fn vlan_mdb_msti_have_no_junk_advanced() {
        assert_eq!(section_ids(&BRIDGE_VLAN_FORM), ["general", "status"]);
        assert_eq!(create_keys(&BRIDGE_VLAN_FORM), ["bridge", "vlan-ids"]);
        assert!(!BRIDGE_VLAN_FORM.writable_keys().contains(&"current-tagged"));
        assert!(!BRIDGE_VLAN_FORM.writable_keys().contains(&"dynamic"));
        assert_status_readonly(&BRIDGE_VLAN_FORM);

        assert_eq!(section_ids(&BRIDGE_MDB_FORM), ["general", "status"]);
        assert_eq!(create_keys(&BRIDGE_MDB_FORM), ["group", "bridge"]);
        assert!(!BRIDGE_MDB_FORM.writable_keys().contains(&"dynamic"));
        assert_status_readonly(&BRIDGE_MDB_FORM);

        assert_eq!(section_ids(&BRIDGE_MSTI_FORM), ["general"]);
        assert_eq!(create_keys(&BRIDGE_MSTI_FORM), ["bridge", "identifier"]);
        assert!(
            BRIDGE_MSTI_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        assert_status_readonly(&BRIDGE_MSTI_FORM);
    }

    #[test]
    fn filter_and_nat_counters_stay_on_status() {
        assert_eq!(
            section_ids(&BRIDGE_FILTER_FORM),
            ["general", "advanced", "status"]
        );
        assert_eq!(create_keys(&BRIDGE_FILTER_FORM), ["chain", "action"]);
        assert!(!BRIDGE_FILTER_FORM.writable_keys().contains(&"packets"));
        assert!(!BRIDGE_FILTER_FORM.writable_keys().contains(&"bytes"));
        assert_eq!(
            advanced_keys(&BRIDGE_FILTER_FORM),
            [
                "mac-protocol",
                "src-mac-address",
                "dst-mac-address",
                "in-interface",
                "out-interface",
                "ip-protocol",
                "src-address",
                "dst-address",
                "src-port",
                "dst-port",
            ]
        );
        assert_status_readonly(&BRIDGE_FILTER_FORM);

        assert_eq!(
            section_ids(&BRIDGE_NAT_FORM),
            ["general", "advanced", "status"]
        );
        assert_eq!(create_keys(&BRIDGE_NAT_FORM), ["chain", "action"]);
        assert!(!BRIDGE_NAT_FORM.writable_keys().contains(&"packets"));
        assert_eq!(
            advanced_keys(&BRIDGE_NAT_FORM),
            [
                "mac-protocol",
                "src-mac-address",
                "dst-mac-address",
                "in-interface",
                "out-interface",
                "to-src-mac-address",
                "to-dst-mac-address",
            ]
        );
        assert!(!advanced_keys(&BRIDGE_NAT_FORM).contains(&"ip-protocol"));
        assert_status_readonly(&BRIDGE_NAT_FORM);
    }

    #[test]
    fn singleton_forms_have_empty_create() {
        assert!(BRIDGE_SETTINGS_FORM.create_sections.is_empty());
        assert_eq!(section_ids(&BRIDGE_SETTINGS_FORM), ["general", "status"]);
        assert!(
            !BRIDGE_SETTINGS_FORM
                .writable_keys()
                .contains(&"bridge-fast-path-active")
        );
        assert!(
            BRIDGE_SETTINGS_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        assert_status_readonly(&BRIDGE_SETTINGS_FORM);

        assert!(BRIDGE_PORT_CONTROLLER_FORM.create_sections.is_empty());
        assert_eq!(section_ids(&BRIDGE_PORT_CONTROLLER_FORM), ["general"]);
        assert!(
            BRIDGE_PORT_CONTROLLER_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );

        assert!(BRIDGE_PORT_EXTENDER_FORM.create_sections.is_empty());
        assert_eq!(section_ids(&BRIDGE_PORT_EXTENDER_FORM), ["general"]);
        assert!(
            BRIDGE_PORT_EXTENDER_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn port_controller_child_forms() {
        assert_eq!(
            section_ids(&BRIDGE_PORT_CONTROLLER_DEVICE_FORM),
            ["general", "status"]
        );
        assert_eq!(
            create_keys(&BRIDGE_PORT_CONTROLLER_DEVICE_FORM),
            ["name", "pe-mac"]
        );
        assert!(
            !BRIDGE_PORT_CONTROLLER_DEVICE_FORM
                .writable_keys()
                .contains(&"status")
        );
        assert_status_readonly(&BRIDGE_PORT_CONTROLLER_DEVICE_FORM);

        assert_eq!(
            section_ids(&BRIDGE_PORT_CONTROLLER_PORT_FORM),
            ["general", "status"]
        );
        assert_eq!(
            create_keys(&BRIDGE_PORT_CONTROLLER_PORT_FORM),
            ["name", "device"]
        );
        assert!(
            !BRIDGE_PORT_CONTROLLER_PORT_FORM
                .writable_keys()
                .contains(&"pcid")
        );
        assert!(
            !BRIDGE_PORT_CONTROLLER_PORT_FORM
                .writable_keys()
                .contains(&"rate")
        );
        assert_status_readonly(&BRIDGE_PORT_CONTROLLER_PORT_FORM);
    }

    #[test]
    fn unknown_vlan_keys_land_on_status_extras() {
        let mut row = HashMap::new();
        row.insert("bridge".into(), "bridge1".into());
        row.insert("current-tagged".into(), "ether1".into());
        row.insert("hw-offload".into(), "true".into());
        let extras = extra_status_fields(&BRIDGE_VLAN_FORM, &row);
        assert_eq!(extras, vec![("hw-offload".into(), "true".into())]);
    }
}
