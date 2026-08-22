//! Form schemas for the Switch nav group.

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

const LOOKUP_SWITCH: FieldKind = FieldKind::Lookup {
    resource_id: "switch",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SWITCH_PORT: FieldKind = FieldKind::Lookup {
    resource_id: "switch-port",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SWITCH_PORTS: FieldKind = FieldKind::Lookup {
    resource_id: "switch-port",
    value_key: "name",
    multiple: true,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const NAME_RO: FieldSpec = f!("name", "Name", FieldKind::Readonly);
const SWITCH: FieldSpec = f!("switch", "Switch", LOOKUP_SWITCH);
const SWITCH_RO: FieldSpec = f!("switch", "Switch", FieldKind::Readonly);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const PORTS: FieldSpec = f!("ports", "Ports", LOOKUP_SWITCH_PORTS);
const VLAN_ID: FieldSpec = f!("vlan-id", "VLAN ID", FieldKind::Number);
const L3HW: FieldSpec = f!("l3-hw-offloading", "L3 HW", FieldKind::Toggle);

pub static SWITCH_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("mirror-source", "Mirror source", LOOKUP_SWITCH_PORT),
                f!("mirror-target", "Mirror target", LOOKUP_SWITCH_PORT),
                f!("mirror-egress-target", "Mirror egress", LOOKUP_SWITCH_PORT),
                f!("cpu-flow-control", "CPU flow", FieldKind::Toggle),
                L3HW,
                f!("switch-all-ports", "All ports", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("type", "Type", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static SWITCH_PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["switch"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME_RO,
                SWITCH_RO,
                f!("vlan-mode", "VLAN mode", FieldKind::Text),
                f!("vlan-header", "VLAN header", FieldKind::Text),
                f!("default-vlan-id", "Default VLAN", FieldKind::Number),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("ingress-rate", "Ingress rate", FieldKind::Text),
                f!("egress-rate", "Egress rate", FieldKind::Text),
                f!("storm-rate", "Storm rate", FieldKind::Text),
                L3HW,
                f!("mirror-ingress", "Mirror in", FieldKind::Toggle),
                f!("mirror-egress", "Mirror out", FieldKind::Toggle),
            ],
        },
    ],
    create_sections: &[],
};

pub static SWITCH_VLAN_FORM: FormSchema = FormSchema {
    title_key: "switch",
    subtitle_keys: &["vlan-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            SWITCH,
            VLAN_ID,
            PORTS,
            f!("independent-learning", "IVL", FieldKind::Toggle),
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[SWITCH, VLAN_ID, PORTS],
    }],
};

pub static SWITCH_RULE_FORM: FormSchema = FormSchema {
    title_key: "switch",
    subtitle_keys: &["ports"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[SWITCH, PORTS, COMMENT, DISABLED],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("mac-protocol", "MAC proto", FieldKind::Text),
                f!("src-mac-address", "Src MAC", FieldKind::Text),
                f!("dst-mac-address", "Dst MAC", FieldKind::Text),
                f!("protocol", "IP proto", FieldKind::Text),
                f!("src-address", "Source", FieldKind::Text),
                f!("dst-address", "Destination", FieldKind::Text),
                f!("src-port", "Src port", FieldKind::Text),
                f!("dst-port", "Dst port", FieldKind::Text),
                VLAN_ID,
                f!("new-dst-ports", "New dst", LOOKUP_SWITCH_PORTS),
                f!("redirect-to-cpu", "Redirect CPU", FieldKind::Toggle),
                f!("mirror", "Mirror", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("invalid", "Invalid", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[SWITCH, PORTS],
    }],
};

pub static SWITCH_PORT_ISOLATION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["switch"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME_RO,
            SWITCH_RO,
            f!("forwarding-override", "Forward to", LOOKUP_SWITCH_PORTS),
        ],
    }],
    create_sections: &[],
};

pub static SWITCH_L3HW_FORM: FormSchema = FormSchema {
    title_key: "autorestart",
    subtitle_keys: &[],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("autorestart", "Autorestart", FieldKind::Toggle),
                f!("fasttrack-hw", "FastTrack HW", FieldKind::Toggle),
                f!("ipv6-hw", "IPv6 HW", FieldKind::Toggle),
                f!("icmp-reply-on-error", "ICMP error", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!(
                "hw-supports-fasttrack",
                "FT support",
                FieldKind::Readonly
            )],
        },
    ],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn tab_ids(schema: &FormSchema) -> Vec<&'static str> {
        schema.sections.iter().map(|section| section.id).collect()
    }

    #[test]
    fn switch_form_matches_winbox_properties() {
        assert_eq!(
            SWITCH_FORM.writable_keys(),
            [
                "name",
                "mirror-source",
                "mirror-target",
                "mirror-egress-target",
                "cpu-flow-control",
                "l3-hw-offloading",
                "switch-all-ports",
            ]
        );
        assert_eq!(
            SWITCH_FORM.known_keys(),
            [
                "name",
                "mirror-source",
                "mirror-target",
                "mirror-egress-target",
                "cpu-flow-control",
                "l3-hw-offloading",
                "switch-all-ports",
                "type",
            ]
        );
        assert_eq!(tab_ids(&SWITCH_FORM), ["general", "status"]);
        assert!(SWITCH_FORM.create_sections.is_empty());
        assert!(
            SWITCH_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        assert_eq!(
            SWITCH_FORM.field("name").map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }

    #[test]
    fn switch_port_keeps_identity_readonly() {
        assert_eq!(tab_ids(&SWITCH_PORT_FORM), ["general", "advanced"]);
        assert!(SWITCH_PORT_FORM.create_sections.is_empty());
        assert_eq!(
            SWITCH_PORT_FORM.field("name").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_eq!(
            SWITCH_PORT_FORM.field("switch").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(!SWITCH_PORT_FORM.writable_keys().contains(&"name"));
        assert!(!SWITCH_PORT_FORM.writable_keys().contains(&"switch"));
        assert_eq!(
            SWITCH_PORT_FORM.writable_keys(),
            [
                "vlan-mode",
                "vlan-header",
                "default-vlan-id",
                "ingress-rate",
                "egress-rate",
                "storm-rate",
                "l3-hw-offloading",
                "mirror-ingress",
                "mirror-egress",
            ]
        );
        assert_eq!(
            SWITCH_PORT_FORM
                .field("default-vlan-id")
                .map(|field| field.kind),
            Some(FieldKind::Number)
        );
    }

    #[test]
    fn switch_vlan_create_is_identity_only() {
        assert_eq!(tab_ids(&SWITCH_VLAN_FORM), ["general"]);
        assert_eq!(
            create_keys(&SWITCH_VLAN_FORM),
            ["switch", "vlan-id", "ports"]
        );
        assert_eq!(
            SWITCH_VLAN_FORM.writable_keys(),
            [
                "switch",
                "vlan-id",
                "ports",
                "independent-learning",
                "disabled",
            ]
        );
        assert_eq!(
            SWITCH_VLAN_FORM.field("vlan-id").map(|field| field.kind),
            Some(FieldKind::Number)
        );
    }

    #[test]
    fn switch_rule_parks_match_fields_on_advanced() {
        assert_eq!(
            tab_ids(&SWITCH_RULE_FORM),
            ["general", "advanced", "status"]
        );
        assert_eq!(create_keys(&SWITCH_RULE_FORM), ["switch", "ports"]);
        assert!(
            SWITCH_RULE_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        assert!(!SWITCH_RULE_FORM.writable_keys().contains(&"invalid"));
        assert!(SWITCH_RULE_FORM.known_keys().contains(&"invalid"));
        assert!(SWITCH_RULE_FORM.writable_keys().contains(&"mac-protocol"));
        assert!(
            SWITCH_RULE_FORM
                .writable_keys()
                .contains(&"redirect-to-cpu")
        );
        assert!(!SWITCH_RULE_FORM.create_sections.iter().any(|section| {
            section
                .fields
                .iter()
                .any(|field| field.key == "mac-protocol" || field.key == "comment")
        }));
    }

    #[test]
    fn switch_port_isolation_is_hardware_edit_only() {
        assert_eq!(tab_ids(&SWITCH_PORT_ISOLATION_FORM), ["general"]);
        assert!(SWITCH_PORT_ISOLATION_FORM.create_sections.is_empty());
        assert_eq!(
            SWITCH_PORT_ISOLATION_FORM.writable_keys(),
            ["forwarding-override"]
        );
        assert_eq!(
            SWITCH_PORT_ISOLATION_FORM.known_keys(),
            ["name", "switch", "forwarding-override"]
        );
    }

    #[test]
    fn switch_l3hw_singleton_has_status_support_flag() {
        assert_eq!(tab_ids(&SWITCH_L3HW_FORM), ["general", "status"]);
        assert!(SWITCH_L3HW_FORM.create_sections.is_empty());
        assert_eq!(
            SWITCH_L3HW_FORM.writable_keys(),
            [
                "autorestart",
                "fasttrack-hw",
                "ipv6-hw",
                "icmp-reply-on-error",
            ]
        );
        assert_eq!(
            SWITCH_L3HW_FORM.known_keys(),
            [
                "autorestart",
                "fasttrack-hw",
                "ipv6-hw",
                "icmp-reply-on-error",
                "hw-supports-fasttrack",
            ]
        );
        assert!(
            !SWITCH_L3HW_FORM
                .writable_keys()
                .contains(&"hw-supports-fasttrack")
        );
    }

    #[test]
    fn patch_body_omits_readonly_switch_type() {
        let mut original = HashMap::new();
        original.insert("name".into(), "switch1".into());
        original.insert("type".into(), "Marvell-98DX3236".into());
        original.insert("cpu-flow-control".into(), "true".into());
        let mut current = original.clone();
        current.insert("cpu-flow-control".into(), "false".into());
        current.insert("type".into(), "Marvell-98DX3236".into());
        let body = patch_body(&SWITCH_FORM, &original, &current, "********");
        assert_eq!(
            body.get("cpu-flow-control").map(String::as_str),
            Some("false")
        );
        assert!(!body.contains_key("type"));
        assert!(!body.contains_key("name"));
    }

    #[test]
    fn unknown_rule_keys_land_on_status_extras() {
        let mut row = HashMap::new();
        row.insert("switch".into(), "switch1".into());
        row.insert("dynamic".into(), "true".into());
        let extras = extra_status_fields(&SWITCH_RULE_FORM, &row);
        assert_eq!(extras, vec![("dynamic".into(), "true".into())]);
    }

    fn assert_lookup(
        schema: &FormSchema,
        key: &str,
        resource_id: &'static str,
        value_key: &'static str,
        multiple: bool,
    ) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Lookup {
                resource_id,
                value_key,
                multiple,
            })
        );
    }

    #[test]
    fn switch_lookups_target_catalog_resources() {
        assert_lookup(&SWITCH_FORM, "mirror-source", "switch-port", "name", false);
        assert_lookup(&SWITCH_FORM, "mirror-target", "switch-port", "name", false);
        assert_lookup(
            &SWITCH_FORM,
            "mirror-egress-target",
            "switch-port",
            "name",
            false,
        );
        assert_eq!(
            SWITCH_FORM.field("name").map(|field| field.kind),
            Some(FieldKind::Text)
        );

        assert_lookup(&SWITCH_VLAN_FORM, "switch", "switch", "name", false);
        assert_lookup(&SWITCH_VLAN_FORM, "ports", "switch-port", "name", true);
        assert_eq!(
            SWITCH_VLAN_FORM.field("vlan-id").map(|field| field.kind),
            Some(FieldKind::Number)
        );

        assert_lookup(&SWITCH_RULE_FORM, "switch", "switch", "name", false);
        assert_lookup(&SWITCH_RULE_FORM, "ports", "switch-port", "name", true);
        assert_lookup(
            &SWITCH_RULE_FORM,
            "new-dst-ports",
            "switch-port",
            "name",
            true,
        );
        assert_eq!(
            SWITCH_RULE_FORM.field("src-port").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            SWITCH_RULE_FORM.field("comment").map(|field| field.kind),
            Some(FieldKind::Text)
        );

        assert_lookup(
            &SWITCH_PORT_ISOLATION_FORM,
            "forwarding-override",
            "switch-port",
            "name",
            true,
        );
        assert_eq!(
            SWITCH_PORT_FORM.field("switch").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_eq!(
            SWITCH_PORT_FORM
                .field("ingress-rate")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }
}
