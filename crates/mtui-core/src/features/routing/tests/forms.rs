use crate::features::routing::forms::*;
use crate::features::routing::guides::GUIDES;
use crate::features::routing::resources::RESOURCES;
use crate::features::routing::rules::form_field_state;
use crate::form_fields::KIND_ORIGINATE_DEFAULT;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &ROUTING_TABLE_FORM,
    &ROUTING_RULE_FORM,
    &OSPF_INSTANCE_FORM,
    &OSPF_AREA_FORM,
    &OSPF_INTERFACE_TEMPLATE_FORM,
    &OSPF_INTERFACE_FORM,
    &BGP_CONNECTION_FORM,
    &BGP_SESSION_FORM,
    &BGP_TEMPLATE_FORM,
    &RIP_INSTANCE_FORM,
    &RIP_INTERFACE_TEMPLATE_FORM,
    &BFD_CONFIGURATION_FORM,
    &ROUTING_FILTER_FORM,
    &ROUTING_ID_FORM,
];

const OSPF_AREA_TYPE_VALUES: &[&str] = &["backbone", "standard", "stub", "nssa"];
const OSPF_NETWORK_TYPE_VALUES: &[&str] = &[
    "broadcast",
    "nbma",
    "ptp",
    "ptmp",
    "ptp-unnumbered",
    "virtual-link",
];

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
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
fn catalog_and_guides_cover_the_routing_group() {
    assert_eq!(RESOURCES.len(), 17);
    assert_eq!(GUIDES.len(), 17);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
    }
    assert!(form_field_state("routing-tables", "fib", &HashMap::new()).is_none());
}

#[test]
fn create_keys_match_writable_and_omit_status() {
    for form in FORMS {
        assert!(form.create_sections.is_empty(), "{}", form.title_key);
        assert_eq!(
            create_keys(form),
            form.writable_keys(),
            "{}",
            form.title_key
        );
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| section.id != "status" && !section.hidden_on_create()),
            "{}",
            form.title_key
        );
        assert_status_readonly(form);
    }
}

#[test]
fn disabled_uses_enabled_inverted_toggle() {
    let mut n = 0;
    for form in FORMS {
        if let Some(field) = form.field("disabled") {
            assert_eq!(field.label, "Enabled", "{}", form.title_key);
            assert_eq!(field.kind, FieldKind::InvertedToggle, "{}", form.title_key);
            n += 1;
        }
    }
    assert_eq!(n, 11);
}

#[test]
fn routing_table_and_rule_create() {
    assert_eq!(
        create_keys(&ROUTING_TABLE_FORM),
        ROUTING_TABLE_FORM.writable_keys()
    );
    assert!(ROUTING_TABLE_FORM.writable_keys().contains(&"fib"));
    assert_eq!(
        create_keys(&ROUTING_RULE_FORM),
        ROUTING_RULE_FORM.writable_keys()
    );
    assert_lookup(&ROUTING_RULE_FORM, "table", "routing-tables", "name", false);
    assert!(ROUTING_RULE_FORM.writable_keys().contains(&"routing-mark"));
    assert_eq!(
        ROUTING_RULE_FORM.field("action").map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &[
                "lookup",
                "lookup-only",
                "unreachable",
                "blackhole",
                "prohibit"
            ],
        })
    );
}

#[test]
fn ospf_instance_short_create() {
    assert_eq!(
        create_keys(&OSPF_INSTANCE_FORM),
        OSPF_INSTANCE_FORM.writable_keys()
    );
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
    assert_eq!(
        OSPF_INSTANCE_FORM.field("version").map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &["2", "3"],
        })
    );
    assert_eq!(
        OSPF_INSTANCE_FORM
            .field("originate-default")
            .map(|field| field.kind),
        Some(KIND_ORIGINATE_DEFAULT)
    );
}

#[test]
fn bgp_uses_dotted_api_keys() {
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
        BGP_CONNECTION_FORM.writable_keys()
    );
    assert!(
        BGP_CONNECTION_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    assert_eq!(
        BGP_CONNECTION_FORM.field("connect").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        BGP_CONNECTION_FORM
            .field("local.role")
            .map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &[
                "ibgp",
                "ibgp-rr",
                "ibgp-rrclient",
                "ebgp",
                "ebgp-customer",
                "ebgp-peer",
                "ebgp-provider",
                "ebgp-rs",
                "ebgp-rs-client",
            ],
        })
    );
}

#[test]
fn ospf_area_and_interface_template_forms() {
    assert_eq!(create_keys(&OSPF_AREA_FORM), OSPF_AREA_FORM.writable_keys());
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
        OSPF_INTERFACE_TEMPLATE_FORM.writable_keys()
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
    assert_eq!(
        OSPF_INTERFACE_TEMPLATE_FORM
            .field("type")
            .map(|field| field.kind),
        Some(FieldKind::Enum {
            values: OSPF_NETWORK_TYPE_VALUES
        })
    );
    assert!(OSPF_INTERFACE_TEMPLATE_FORM.field("interface").is_none());
    assert!(OSPF_INTERFACE_TEMPLATE_FORM.field("cost").is_none());
    assert!(OSPF_INTERFACE_TEMPLATE_FORM.field("state").is_none());
}

#[test]
fn ospf_interface_runtime_form_is_status_only() {
    assert!(OSPF_INTERFACE_FORM.create_sections.is_empty());
    assert!(OSPF_INTERFACE_FORM.writable_keys().is_empty());
    assert_eq!(OSPF_INTERFACE_FORM.sections.len(), 1);
    assert_eq!(OSPF_INTERFACE_FORM.sections[0].id, "status");
    assert!(OSPF_INTERFACE_FORM.sections[0].read_only);
    assert!(
        OSPF_INTERFACE_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    for field in OSPF_INTERFACE_FORM.sections[0].fields {
        assert_eq!(field.kind, FieldKind::Readonly, "{}", field.key);
    }
    assert_eq!(
        OSPF_INTERFACE_FORM
            .field("address")
            .map(|field| field.label),
        Some("Address")
    );
    assert_eq!(
        OSPF_INTERFACE_FORM
            .field("network-type")
            .map(|field| field.label),
        Some("Network Type")
    );
    assert_eq!(
        OSPF_INTERFACE_FORM
            .field("hello-interval")
            .map(|field| field.label),
        Some("Hello Interval")
    );
    assert!(OSPF_INTERFACE_FORM.field("interfaces").is_none());
    assert!(OSPF_INTERFACE_FORM.field("disabled").is_none());
    assert!(OSPF_INTERFACE_FORM.field("type").is_none());
    assert!(
        OSPF_INTERFACE_FORM
            .sections
            .iter()
            .flat_map(|section| section.fields)
            .all(|field| !matches!(
                field.kind,
                FieldKind::Text
                    | FieldKind::Enum { .. }
                    | FieldKind::Lookup { .. }
                    | FieldKind::Repeat
                    | FieldKind::Toggle
                    | FieldKind::Number
                    | FieldKind::Secret
            ))
    );
    assert_status_readonly(&OSPF_INTERFACE_FORM);
}

#[test]
fn ospf_interface_runtime_rows_keep_optional_fields() {
    struct Case {
        name: &'static str,
        fields: &'static [(&'static str, &'static str)],
        address: Option<&'static str>,
        bdr: Option<&'static str>,
        extra: &'static [&'static str],
    }
    let cases = [
        Case {
            name: "broadcast dr",
            fields: &[
                ("address", "10.1.1.1%ether1"),
                ("area", "backbone"),
                ("state", "dr"),
                ("network-type", "broadcast"),
                ("cost", "10"),
                ("bdr", "10.1.1.2"),
            ],
            address: Some("10.1.1.1%ether1"),
            bdr: Some("10.1.1.2"),
            extra: &[],
        },
        Case {
            name: "ptp omits dr bdr",
            fields: &[
                ("address", "172.16.1.1%gre1"),
                ("area", "backbone"),
                ("state", "ptp"),
                ("network-type", "ptp"),
                ("cost", "1"),
            ],
            address: Some("172.16.1.1%gre1"),
            bdr: None,
            extra: &[],
        },
        Case {
            name: "unknown instance key is a status extra",
            fields: &[
                ("address", "10.0.0.1%loopback"),
                ("area", "backbone"),
                ("state", "passive"),
                ("instance", "ospf-main"),
            ],
            address: Some("10.0.0.1%loopback"),
            bdr: None,
            extra: &["instance"],
        },
    ];
    for case in cases {
        let row: HashMap<String, String> = case
            .fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        assert_eq!(
            row.get("address").map(String::as_str),
            case.address,
            "{}",
            case.name
        );
        assert_eq!(
            row.get("bdr").map(String::as_str),
            case.bdr,
            "{}",
            case.name
        );
        let extras = extra_status_fields(&OSPF_INTERFACE_FORM, &row);
        let extra_keys: Vec<&str> = extras.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(extra_keys, case.extra, "{}", case.name);
    }
}

#[test]
fn ospf_interface_runtime_never_encodes_a_patch() {
    let mut original = HashMap::new();
    original.insert("address".into(), "10.1.1.1%ether1".into());
    original.insert("cost".into(), "10".into());
    original.insert("state".into(), "dr".into());
    let mut current = original.clone();
    current.insert("cost".into(), "20".into());
    current.insert("state".into(), "bdr".into());
    let body = patch_body(&OSPF_INTERFACE_FORM, &original, &current, "********");
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn bgp_session_runtime_form_is_status_only() {
    assert!(BGP_SESSION_FORM.create_sections.is_empty());
    assert!(BGP_SESSION_FORM.writable_keys().is_empty());
    assert_eq!(BGP_SESSION_FORM.sections.len(), 1);
    assert_eq!(BGP_SESSION_FORM.sections[0].id, "status");
    assert!(BGP_SESSION_FORM.sections[0].read_only);
    assert!(
        BGP_SESSION_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    for field in BGP_SESSION_FORM.sections[0].fields {
        assert_eq!(field.kind, FieldKind::Readonly, "{}", field.key);
    }
    assert_eq!(
        BGP_SESSION_FORM
            .field("remote.address")
            .map(|field| field.label),
        Some("Remote Address")
    );
    assert_eq!(
        BGP_SESSION_FORM
            .field("prefix-count")
            .map(|field| field.label),
        Some("Prefix Count")
    );
    assert_eq!(
        BGP_SESSION_FORM
            .field("input.last-notification")
            .map(|field| field.label),
        Some("Input Last Notification")
    );
    assert!(BGP_SESSION_FORM.field("disabled").is_none());
    assert!(BGP_SESSION_FORM.field("connect").is_none());
    assert_status_readonly(&BGP_SESSION_FORM);
}

#[test]
fn bgp_session_runtime_never_encodes_a_patch() {
    let mut original = HashMap::new();
    original.insert("name".into(), "peer1-1".into());
    original.insert("established".into(), "true".into());
    original.insert("prefix-count".into(), "12".into());
    let mut current = original.clone();
    current.insert("established".into(), "false".into());
    current.insert("prefix-count".into(), "0".into());
    let body = patch_body(&BGP_SESSION_FORM, &original, &current, "********");
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn bgp_session_runtime_rows_keep_optional_fields() {
    let row = HashMap::from([
        ("name".into(), "toR2".into()),
        ("remote.address".into(), "192.168.1.2".into()),
        ("established".into(), "true".into()),
        ("unexpected-cap".into(), "ms".into()),
    ]);
    let extras = extra_status_fields(&BGP_SESSION_FORM, &row);
    assert_eq!(extras, vec![("unexpected-cap".into(), "ms".into())]);
}

#[test]
fn bgp_template_is_smaller_than_connection() {
    assert_eq!(
        create_keys(&BGP_TEMPLATE_FORM),
        BGP_TEMPLATE_FORM.writable_keys()
    );
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
    assert!(BGP_TEMPLATE_FORM.writable_keys().len() < BGP_CONNECTION_FORM.writable_keys().len());
    assert!(
        BGP_TEMPLATE_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    assert_eq!(
        BGP_TEMPLATE_FORM
            .field("address-families")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        BGP_TEMPLATE_FORM
            .field("output.network")
            .map(|field| field.kind),
        Some(FieldKind::Lookup {
            resource_id: "address-list",
            value_key: "list",
            multiple: false,
        })
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

#[test]
fn rip_and_bfd_lookups_and_kinds() {
    assert_lookup(&RIP_INSTANCE_FORM, "vrf", "vrf", "name", false);
    assert_eq!(
        RIP_INSTANCE_FORM
            .field("originate-default")
            .map(|field| field.kind),
        Some(KIND_ORIGINATE_DEFAULT)
    );
    assert_lookup(
        &RIP_INTERFACE_TEMPLATE_FORM,
        "instance",
        "rip-instances",
        "name",
        false,
    );
    assert_lookup(
        &RIP_INTERFACE_TEMPLATE_FORM,
        "interfaces",
        "interfaces",
        "name",
        true,
    );
    assert_lookup(
        &BFD_CONFIGURATION_FORM,
        "interfaces",
        "interfaces",
        "name",
        true,
    );
    assert_eq!(
        BFD_CONFIGURATION_FORM
            .field("addresses")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        BFD_CONFIGURATION_FORM
            .field("min-tx-interval")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_eq!(
        BFD_CONFIGURATION_FORM
            .field("multiplier")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        ROUTING_ID_FORM.field("select").map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &["any", "only-dynamic", "only-static"],
        })
    );
}

#[test]
fn form_none_tables_have_no_editor() {
    for id in ["ospf-neighbors", "ospf-lsa", "bgp-advertisements"] {
        let spec = RESOURCES.iter().find(|row| row.id == id).expect(id);
        assert!(spec.form.is_none(), "{id}");
    }
}

#[test]
fn remaining_scalar_kinds() {
    assert_eq!(
        BGP_CONNECTION_FORM
            .field("remote.as")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        BGP_TEMPLATE_FORM.field("as").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        ROUTING_ID_FORM.field("id").map(|field| field.kind),
        Some(FieldKind::Ip)
    );
}
