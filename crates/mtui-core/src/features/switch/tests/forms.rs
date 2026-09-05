use crate::features::switch::forms::*;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
}

fn tab_ids(schema: &FormSchema) -> Vec<&'static str> {
    schema.sections.iter().map(|section| section.id).collect()
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

fn assert_enabled_toggle(schema: &FormSchema) {
    assert_eq!(
        schema.field("disabled").map(|field| field.label),
        Some("Enabled")
    );
    assert_eq!(
        schema.field("disabled").map(|field| field.kind),
        Some(FieldKind::InvertedToggle)
    );
}

fn assert_create_omits_status(schema: &FormSchema) {
    assert!(
        schema
            .sections_for(true)
            .iter()
            .all(|section| section.id != "status" && !section.hidden_on_create())
    );
}

fn field_visible(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    crate::features::switch::rules::form_field_state(resource_id, key, values)
        .is_none_or(|(visible, _)| visible)
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
    assert_eq!(create_keys(&SWITCH_FORM), SWITCH_FORM.writable_keys());
    assert_create_omits_status(&SWITCH_FORM);
    assert_status_readonly(&SWITCH_FORM);
    assert_eq!(
        SWITCH_FORM.field("name").map(|field| field.kind),
        Some(FieldKind::Text)
    );
}

#[test]
fn switch_optional_fields_follow_print_detail_keys() {
    let mt7621 = HashMap::from([
        ("name".to_string(), "switch1".to_string()),
        ("type".to_string(), "MediaTek-MT7621".to_string()),
        ("mirror-source".to_string(), "none".to_string()),
        ("mirror-target".to_string(), "none".to_string()),
    ]);
    assert!(field_visible("switch", "name", &mt7621));
    assert!(field_visible("switch", "type", &mt7621));
    assert!(field_visible("switch", "mirror-source", &mt7621));
    assert!(field_visible("switch", "mirror-target", &mt7621));
    assert!(!field_visible("switch", "mirror-egress-target", &mt7621));
    assert!(!field_visible("switch", "cpu-flow-control", &mt7621));
    assert!(!field_visible("switch", "l3-hw-offloading", &mt7621));
    assert!(!field_visible("switch", "switch-all-ports", &mt7621));

    let marvell = HashMap::from([
        ("cpu-flow-control".to_string(), "yes".to_string()),
        ("mirror-egress-target".to_string(), "none".to_string()),
        ("l3-hw-offloading".to_string(), "no".to_string()),
    ]);
    assert!(field_visible("switch", "cpu-flow-control", &marvell));
    assert!(field_visible("switch", "mirror-egress-target", &marvell));
    assert!(field_visible("switch", "l3-hw-offloading", &marvell));
    assert!(!field_visible("switch", "mirror-source", &marvell));
}

#[test]
fn switch_port_keeps_identity_readonly() {
    assert_eq!(tab_ids(&SWITCH_PORT_FORM), ["general", "advanced"]);
    assert!(SWITCH_PORT_FORM.create_sections.is_empty());
    assert_eq!(
        create_keys(&SWITCH_PORT_FORM),
        SWITCH_PORT_FORM.writable_keys()
    );
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
fn switch_port_l3_hw_follows_port_print_attributes() {
    let row = HashMap::from([
        ("name".to_string(), "ether1".to_string()),
        ("switch".to_string(), "switch1".to_string()),
    ]);
    assert!(!field_visible("switch-port", "l3-hw-offloading", &row));
    assert!(field_visible("switch-port", "vlan-mode", &row));

    let with_l3 = HashMap::from([("l3-hw-offloading".to_string(), "yes".to_string())]);
    assert!(field_visible("switch-port", "l3-hw-offloading", &with_l3));
}

#[test]
fn switch_vlan_create_matches_writable_sheet() {
    assert_eq!(tab_ids(&SWITCH_VLAN_FORM), ["general"]);
    assert!(SWITCH_VLAN_FORM.create_sections.is_empty());
    assert_eq!(
        create_keys(&SWITCH_VLAN_FORM),
        SWITCH_VLAN_FORM.writable_keys()
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
    assert_enabled_toggle(&SWITCH_VLAN_FORM);
    assert_create_omits_status(&SWITCH_VLAN_FORM);
}

#[test]
fn switch_rule_parks_match_fields_on_match() {
    assert_eq!(tab_ids(&SWITCH_RULE_FORM), ["general", "match", "status"]);
    assert!(SWITCH_RULE_FORM.create_sections.is_empty());
    assert_eq!(
        create_keys(&SWITCH_RULE_FORM),
        SWITCH_RULE_FORM.writable_keys()
    );
    assert_create_omits_status(&SWITCH_RULE_FORM);
    assert_status_readonly(&SWITCH_RULE_FORM);
    assert_enabled_toggle(&SWITCH_RULE_FORM);
    assert!(!SWITCH_RULE_FORM.writable_keys().contains(&"invalid"));
    assert!(SWITCH_RULE_FORM.known_keys().contains(&"invalid"));
    assert!(SWITCH_RULE_FORM.writable_keys().contains(&"mac-protocol"));
    assert!(
        SWITCH_RULE_FORM
            .writable_keys()
            .contains(&"redirect-to-cpu")
    );
    assert!(SWITCH_RULE_FORM.create_keys().contains(&"mac-protocol"));
    assert!(SWITCH_RULE_FORM.create_keys().contains(&"comment"));
}

#[test]
fn switch_port_isolation_is_hardware_edit_only() {
    assert_eq!(tab_ids(&SWITCH_PORT_ISOLATION_FORM), ["general"]);
    assert!(SWITCH_PORT_ISOLATION_FORM.create_sections.is_empty());
    assert_eq!(
        create_keys(&SWITCH_PORT_ISOLATION_FORM),
        SWITCH_PORT_ISOLATION_FORM.writable_keys()
    );
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
        create_keys(&SWITCH_L3HW_FORM),
        SWITCH_L3HW_FORM.writable_keys()
    );
    assert_create_omits_status(&SWITCH_L3HW_FORM);
    assert_status_readonly(&SWITCH_L3HW_FORM);
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
        SWITCH_RULE_FORM
            .field("mac-protocol")
            .map(|field| field.kind),
        Some(crate::form_fields::KIND_MAC_PROTOCOL)
    );
    assert_eq!(
        SWITCH_RULE_FORM.field("protocol").map(|field| field.kind),
        Some(crate::form_fields::KIND_IP_PROTOCOL)
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
    assert_eq!(
        SWITCH_RULE_FORM
            .field("src-mac-address")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
}
