//! Form schemas for the Routing nav group.
//!
//! Catalog wiring (do not register here):
//! - `routing-tables` → `/rest/routing/table` (`ROUTING_TABLE_FORM`)
//! - `routing-rules` → `/rest/routing/rule` (`ROUTING_RULE_FORM`)
//! - `ospf-instances` → `/rest/routing/ospf/instance` (`OSPF_INSTANCE_FORM`)
//! - `ospf-areas` → `/rest/routing/ospf/area` (`MEMBER_ACTIONS`, `OSPF_AREA_FORM`)
//! - `ospf-interface-templates` → `/rest/routing/ospf/interface-template` (`MEMBER_ACTIONS`, `OSPF_INTERFACE_TEMPLATE_FORM`)
//! - `bgp-connections` → `/rest/routing/bgp/connection` (`BGP_CONNECTION_FORM`)
//! - `bgp-templates` → `/rest/routing/bgp/template` (`MEMBER_ACTIONS`, `BGP_TEMPLATE_FORM`)
//!
//! Group id: `routing-group`.
//! `RouterOS` 7 OSPF interface config is `/routing/ospf/interface-template`; there is no
//! `/routing/ospf/interface` form.

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

const LOOKUP_ROUTING_TABLE: FieldKind = FieldKind::Lookup {
    resource_id: "routing-tables",
    value_key: "name",
    multiple: false,
};
const LOOKUP_OSPF_INSTANCE: FieldKind = FieldKind::Lookup {
    resource_id: "ospf-instances",
    value_key: "name",
    multiple: false,
};
const LOOKUP_OSPF_AREA: FieldKind = FieldKind::Lookup {
    resource_id: "ospf-areas",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACES: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: true,
};

const OSPF_AREA_TYPE_VALUES: &[&str] = &["backbone", "standard", "stub", "nssa"];

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const TABLE: FieldSpec = f!("table", "Table", LOOKUP_ROUTING_TABLE);
const OSPF_INSTANCE: FieldSpec = f!("instance", "Instance", LOOKUP_OSPF_INSTANCE);
const OSPF_AREA: FieldSpec = f!("area", "Area", LOOKUP_OSPF_AREA);

pub static ROUTING_TABLE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("fib", "FIB", FieldKind::Toggle), COMMENT],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static ROUTING_RULE_FORM: FormSchema = FormSchema {
    title_key: "action",
    subtitle_keys: &["table"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("src-address", "Src address", FieldKind::Text),
            f!("dst-address", "Dst address", FieldKind::Text),
            f!("routing-mark", "Routing mark", FieldKind::Text),
            ACTION,
            TABLE,
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ACTION, TABLE],
    }],
};

pub static OSPF_INSTANCE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["router-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("version", "Version", FieldKind::Number),
            f!("router-id", "Router ID", FieldKind::Text),
            f!("originate-default", "Originate default", FieldKind::Text),
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

pub static OSPF_AREA_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["area-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            OSPF_INSTANCE,
            f!("area-id", "Area ID", FieldKind::Text),
            f!(
                "type",
                "Type",
                FieldKind::Enum {
                    values: OSPF_AREA_TYPE_VALUES
                }
            ),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, OSPF_INSTANCE],
    }],
};

pub static OSPF_INTERFACE_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "instance",
    subtitle_keys: &["area"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            OSPF_INSTANCE,
            OSPF_AREA,
            f!("interfaces", "Interfaces", LOOKUP_IFACES),
            f!("type", "Type", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[OSPF_INSTANCE, OSPF_AREA],
    }],
};

pub static BGP_CONNECTION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["remote.address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("remote.address", "Remote address", FieldKind::Text),
            f!("remote.as", "Remote AS", FieldKind::Text),
            f!("local.role", "Local role", FieldKind::Text),
            f!("local.address", "Local address", FieldKind::Text),
            f!("connect", "Connect", FieldKind::Text),
            f!("listen", "Listen", FieldKind::Toggle),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("remote.address", "Remote address", FieldKind::Text),
            f!("remote.as", "Remote AS", FieldKind::Text),
        ],
    }],
};

pub static BGP_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["as"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("as", "AS", FieldKind::Text),
            f!("router-id", "Router ID", FieldKind::Text),
            f!("address-families", "Address families", FieldKind::Text),
            f!("output.network", "Output network", FieldKind::Text),
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

pub static RIP_INSTANCE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vrf"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("vrf", "VRF", FieldKind::Text),
            f!("originate-default", "Originate default", FieldKind::Toggle),
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

pub static RIP_INTERFACE_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "interfaces",
    subtitle_keys: &["instance"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("instance", "Instance", FieldKind::Text),
            f!("interfaces", "Interfaces", LOOKUP_IFACES),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("instance", "Instance", FieldKind::Text)],
    }],
};

pub static BFD_CONFIGURATION_FORM: FormSchema = FormSchema {
    title_key: "interfaces",
    subtitle_keys: &["addresses"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interfaces", "Interfaces", LOOKUP_IFACES),
            f!("addresses", "Addresses", FieldKind::Text),
            f!("min-tx-interval", "Min TX", FieldKind::Text),
            f!("min-rx-interval", "Min RX", FieldKind::Text),
            f!("multiplier", "Multiplier", FieldKind::Number),
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("interfaces", "Interfaces", LOOKUP_IFACES)],
    }],
};

pub static ROUTING_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["rule"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("chain", "Chain", FieldKind::Text),
            f!("rule", "Rule", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("chain", "Chain", FieldKind::Text)],
    }],
};

pub static ROUTING_ID_FORM: FormSchema = FormSchema {
    title_key: "id",
    subtitle_keys: &["name"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("id", "ID", FieldKind::Text),
            f!("select", "Select", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("id", "ID", FieldKind::Text)],
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
    fn routing_table_and_rule_create() {
        assert_eq!(create_keys(&ROUTING_TABLE_FORM), ["name"]);
        assert!(ROUTING_TABLE_FORM.writable_keys().contains(&"fib"));
        assert_eq!(create_keys(&ROUTING_RULE_FORM), ["action", "table"]);
        assert_lookup(&ROUTING_RULE_FORM, "table", "routing-tables", "name", false);
        assert!(ROUTING_RULE_FORM.writable_keys().contains(&"routing-mark"));
    }

    #[test]
    fn ospf_instance_short_create() {
        assert_eq!(create_keys(&OSPF_INSTANCE_FORM), ["name"]);
        assert_eq!(
            OSPF_INSTANCE_FORM.writable_keys(),
            [
                "name",
                "version",
                "router-id",
                "originate-default",
                "comment",
                "disabled",
            ]
        );
    }

    #[test]
    fn bgp_uses_dotted_rest_keys() {
        assert!(
            BGP_CONNECTION_FORM
                .writable_keys()
                .contains(&"remote.address")
        );
        assert!(BGP_CONNECTION_FORM.writable_keys().contains(&"remote.as"));
        assert!(BGP_CONNECTION_FORM.writable_keys().contains(&"local.role"));
        assert!(
            BGP_CONNECTION_FORM
                .writable_keys()
                .contains(&"local.address")
        );
        assert_eq!(
            create_keys(&BGP_CONNECTION_FORM),
            ["name", "remote.address", "remote.as"]
        );
        assert!(
            BGP_CONNECTION_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn ospf_area_and_interface_template_forms() {
        assert_eq!(create_keys(&OSPF_AREA_FORM), ["name", "instance"]);
        assert_lookup(&OSPF_AREA_FORM, "instance", "ospf-instances", "name", false);
        assert_eq!(
            OSPF_AREA_FORM.field("type").map(|field| field.kind),
            Some(FieldKind::Enum {
                values: OSPF_AREA_TYPE_VALUES
            })
        );
        assert_eq!(
            OSPF_AREA_FORM.writable_keys(),
            ["name", "instance", "area-id", "type", "comment", "disabled",]
        );

        assert_eq!(
            create_keys(&OSPF_INTERFACE_TEMPLATE_FORM),
            ["instance", "area"]
        );
        assert_lookup(
            &OSPF_INTERFACE_TEMPLATE_FORM,
            "instance",
            "ospf-instances",
            "name",
            false,
        );
        assert_lookup(
            &OSPF_INTERFACE_TEMPLATE_FORM,
            "area",
            "ospf-areas",
            "name",
            false,
        );
        assert_lookup(
            &OSPF_INTERFACE_TEMPLATE_FORM,
            "interfaces",
            "interfaces",
            "name",
            true,
        );
        assert_eq!(
            OSPF_INTERFACE_TEMPLATE_FORM.writable_keys(),
            [
                "instance",
                "area",
                "interfaces",
                "type",
                "comment",
                "disabled",
            ]
        );
        assert!(OSPF_INTERFACE_TEMPLATE_FORM.field("interface").is_none());
    }

    #[test]
    fn bgp_template_is_smaller_than_connection() {
        assert_eq!(create_keys(&BGP_TEMPLATE_FORM), ["name"]);
        assert!(
            BGP_TEMPLATE_FORM
                .writable_keys()
                .contains(&"output.network")
        );
        assert_eq!(
            BGP_TEMPLATE_FORM.writable_keys(),
            [
                "name",
                "as",
                "router-id",
                "address-families",
                "output.network",
                "comment",
                "disabled",
            ]
        );
        assert!(BGP_TEMPLATE_FORM.field("remote.address").is_none());
        assert!(BGP_TEMPLATE_FORM.field("local.role").is_none());
        assert!(
            BGP_TEMPLATE_FORM.writable_keys().len() < BGP_CONNECTION_FORM.writable_keys().len()
        );
        assert!(
            BGP_TEMPLATE_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn patch_body_keeps_dotted_bgp_keys() {
        let mut original = HashMap::new();
        original.insert("name".into(), "peer1".into());
        original.insert("remote.address".into(), "192.0.2.1".into());
        original.insert("remote.as".into(), "65001".into());
        let mut current = original.clone();
        current.insert("remote.address".into(), "192.0.2.2".into());
        let body = patch_body(&BGP_CONNECTION_FORM, &original, &current, "********");
        assert_eq!(
            body.get("remote.address").map(String::as_str),
            Some("192.0.2.2")
        );
        assert!(!body.contains_key("name"));
    }

    #[test]
    fn patch_body_keeps_dotted_bgp_template_keys() {
        let mut original = HashMap::new();
        original.insert("name".into(), "default".into());
        original.insert("output.network".into(), "lan".into());
        let mut current = original.clone();
        current.insert("output.network".into(), "wan".into());
        let body = patch_body(&BGP_TEMPLATE_FORM, &original, &current, "********");
        assert_eq!(body.get("output.network").map(String::as_str), Some("wan"));
        assert!(!body.contains_key("name"));
    }
}
