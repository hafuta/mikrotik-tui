//! Form schemas for the Routing nav group.
//!
//! Catalog wiring (do not register here):
//! - `routing-tables` → `/rest/routing/table` (`ROUTING_TABLE_FORM`)
//! - `routing-rules` → `/rest/routing/rule` (`ROUTING_RULE_FORM`)
//! - `ospf-instances` → `/rest/routing/ospf/instance` (`OSPF_INSTANCE_FORM`)
//! - `bgp-connections` → `/rest/routing/bgp/connection` (`BGP_CONNECTION_FORM`)
//!
//! Group id: `routing-group`.

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

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const TABLE: FieldSpec = f!("table", "Table", FieldKind::Text);

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
    fn routing_table_and_rule_create() {
        assert_eq!(create_keys(&ROUTING_TABLE_FORM), ["name"]);
        assert!(ROUTING_TABLE_FORM.writable_keys().contains(&"fib"));
        assert_eq!(create_keys(&ROUTING_RULE_FORM), ["action", "table"]);
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
}
