use crate::features::bridge::forms::*;
use crate::forms::{ARP_VALUES, FieldKind, FormSchema, extra_status_fields};
use std::collections::HashMap;

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
}

fn section_ids(schema: &FormSchema) -> Vec<&'static str> {
    schema.sections.iter().map(|section| section.id).collect()
}

fn section_keys(schema: &FormSchema, id: &str) -> Vec<&'static str> {
    schema
        .sections
        .iter()
        .find(|section| section.id == id)
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
fn bridge_form_writable_status_and_short_create() {
    assert_eq!(
        section_ids(&BRIDGE_FORM),
        ["general", "stp", "vlan", "status"]
    );
    assert_eq!(
        BRIDGE_FORM.writable_keys(),
        [
            "name",
            "arp",
            "mtu",
            "mac-address",
            "fast-forward",
            "igmp-snooping",
            "dhcp-snooping",
            "comment",
            "disabled",
            "protocol-mode",
            "priority",
            "region-name",
            "vlan-filtering",
            "pvid",
            "frame-types",
            "ingress-filtering",
        ]
    );
    assert!(BRIDGE_FORM.known_keys().contains(&"running"));
    assert!(!BRIDGE_FORM.writable_keys().contains(&"running"));
    assert_eq!(create_keys(&BRIDGE_FORM), BRIDGE_FORM.writable_keys());
    assert_eq!(
        section_keys(&BRIDGE_FORM, "stp"),
        ["protocol-mode", "priority", "region-name"]
    );
    assert_eq!(
        section_keys(&BRIDGE_FORM, "vlan"),
        ["vlan-filtering", "pvid", "frame-types", "ingress-filtering"]
    );
    assert_status_readonly(&BRIDGE_FORM);
    assert_enabled_toggle(&BRIDGE_FORM);
    assert_eq!(
        BRIDGE_FORM.field("pvid").map(|field| field.kind),
        Some(FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(4095)
        })
    );
    assert!(matches!(
        BRIDGE_FORM.field("protocol-mode").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
    assert_eq!(
        BRIDGE_FORM.field("mac-address").map(|field| field.kind),
        Some(FieldKind::Mac)
    );
    assert_eq!(
        BRIDGE_FORM.field("priority").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_PRIORITY)
    );
    let Some(FieldKind::LabeledEnum { choices }) = BRIDGE_FORM.field("arp").map(|field| field.kind)
    else {
        panic!("bridge arp must be a labeled enum");
    };
    assert_eq!(
        choices
            .iter()
            .map(|choice| choice.value)
            .collect::<Vec<_>>(),
        ARP_VALUES
    );
}

#[test]
fn bridge_port_form_splits_stp() {
    assert_eq!(section_ids(&BRIDGE_PORT_FORM), ["general", "stp"]);
    assert!(!BRIDGE_PORT_FORM.writable_keys().contains(&"role"));
    assert_eq!(
        create_keys(&BRIDGE_PORT_FORM),
        BRIDGE_PORT_FORM.writable_keys()
    );
    assert_eq!(
        section_keys(&BRIDGE_PORT_FORM, "stp"),
        [
            "edge",
            "horizon",
            "path-cost",
            "priority",
            "bpdu-guard",
            "restricted-role",
            "learn",
        ]
    );
    assert_status_readonly(&BRIDGE_PORT_FORM);
    assert_enabled_toggle(&BRIDGE_PORT_FORM);
    assert_eq!(
        BRIDGE_PORT_FORM.field("hw").map(|field| field.label),
        Some("Hardware Offload")
    );
    assert_eq!(
        BRIDGE_PORT_FORM
            .field("bpdu-guard")
            .map(|field| field.label),
        Some("BPDU Guard")
    );
    assert_eq!(
        BRIDGE_PORT_FORM.field("pvid").map(|field| field.kind),
        Some(FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(4095)
        })
    );
    assert_eq!(
        BRIDGE_PORT_FORM.field("priority").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_PORT_PRIORITY)
    );
}

#[test]
fn vlan_mdb_msti_have_no_junk_advanced() {
    assert_eq!(section_ids(&BRIDGE_VLAN_FORM), ["general", "status"]);
    assert_eq!(
        create_keys(&BRIDGE_VLAN_FORM),
        BRIDGE_VLAN_FORM.writable_keys()
    );
    assert!(!BRIDGE_VLAN_FORM.writable_keys().contains(&"current-tagged"));
    assert!(!BRIDGE_VLAN_FORM.writable_keys().contains(&"dynamic"));
    assert_status_readonly(&BRIDGE_VLAN_FORM);
    assert_enabled_toggle(&BRIDGE_VLAN_FORM);
    assert_eq!(
        BRIDGE_VLAN_FORM.field("vlan-ids").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );

    assert_eq!(section_ids(&BRIDGE_MDB_FORM), ["general", "status"]);
    assert_eq!(
        create_keys(&BRIDGE_MDB_FORM),
        BRIDGE_MDB_FORM.writable_keys()
    );
    assert!(!BRIDGE_MDB_FORM.writable_keys().contains(&"dynamic"));
    assert_status_readonly(&BRIDGE_MDB_FORM);
    assert_enabled_toggle(&BRIDGE_MDB_FORM);

    assert_eq!(section_ids(&BRIDGE_MSTI_FORM), ["general"]);
    assert_eq!(
        create_keys(&BRIDGE_MSTI_FORM),
        BRIDGE_MSTI_FORM.writable_keys()
    );
    assert!(
        BRIDGE_MSTI_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    assert_status_readonly(&BRIDGE_MSTI_FORM);
    assert_eq!(
        BRIDGE_MSTI_FORM.field("priority").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_PRIORITY)
    );
}

#[test]
fn filter_and_nat_counters_stay_on_status() {
    assert_eq!(
        section_ids(&BRIDGE_FILTER_FORM),
        ["general", "match", "status"]
    );
    assert_eq!(
        create_keys(&BRIDGE_FILTER_FORM),
        BRIDGE_FILTER_FORM.writable_keys()
    );
    assert!(!BRIDGE_FILTER_FORM.writable_keys().contains(&"packets"));
    assert!(!BRIDGE_FILTER_FORM.writable_keys().contains(&"bytes"));
    assert_eq!(
        section_keys(&BRIDGE_FILTER_FORM, "match"),
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
    assert_enabled_toggle(&BRIDGE_FILTER_FORM);
    assert_eq!(
        BRIDGE_FILTER_FORM.field("chain").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_FILTER_CHAIN)
    );
    assert_eq!(
        BRIDGE_FILTER_FORM.field("action").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_FILTER_ACTION)
    );
    assert_eq!(
        BRIDGE_FILTER_FORM
            .field("mac-protocol")
            .map(|field| field.kind),
        Some(crate::form_fields::KIND_MAC_PROTOCOL)
    );
    assert_eq!(
        BRIDGE_FILTER_FORM
            .field("ip-protocol")
            .map(|field| field.kind),
        Some(crate::form_fields::KIND_IP_PROTOCOL)
    );

    assert_eq!(
        section_ids(&BRIDGE_NAT_FORM),
        ["general", "match", "status"]
    );
    assert_eq!(
        create_keys(&BRIDGE_NAT_FORM),
        BRIDGE_NAT_FORM.writable_keys()
    );
    assert!(!BRIDGE_NAT_FORM.writable_keys().contains(&"packets"));
    assert_eq!(
        section_keys(&BRIDGE_NAT_FORM, "match"),
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
    assert!(!section_keys(&BRIDGE_NAT_FORM, "match").contains(&"ip-protocol"));
    assert_status_readonly(&BRIDGE_NAT_FORM);
    assert_enabled_toggle(&BRIDGE_NAT_FORM);
    assert_eq!(
        BRIDGE_NAT_FORM.field("chain").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_NAT_CHAIN)
    );
    assert_eq!(
        BRIDGE_NAT_FORM.field("action").map(|field| field.kind),
        Some(crate::form_fields::KIND_BRIDGE_NAT_ACTION)
    );
    assert_eq!(
        BRIDGE_NAT_FORM
            .field("mac-protocol")
            .map(|field| field.kind),
        Some(crate::form_fields::KIND_MAC_PROTOCOL)
    );
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
        BRIDGE_PORT_CONTROLLER_DEVICE_FORM.writable_keys()
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
        BRIDGE_PORT_CONTROLLER_PORT_FORM.writable_keys()
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
    assert_enabled_toggle(&BRIDGE_PORT_CONTROLLER_PORT_FORM);
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

#[test]
fn bridge_lookups_target_catalog_resources() {
    assert_lookup(&BRIDGE_PORT_FORM, "interface", "interfaces", "name", false);
    assert_lookup(&BRIDGE_PORT_FORM, "bridge", "bridges", "name", false);
    assert_eq!(
        BRIDGE_PORT_FORM.field("comment").map(|field| field.kind),
        Some(FieldKind::Text)
    );

    assert_lookup(&BRIDGE_VLAN_FORM, "bridge", "bridges", "name", false);
    assert_lookup(&BRIDGE_VLAN_FORM, "tagged", "interfaces", "name", true);
    assert_lookup(&BRIDGE_VLAN_FORM, "untagged", "interfaces", "name", true);
    assert_eq!(
        BRIDGE_VLAN_FORM.field("vlan-ids").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );

    assert_lookup(&BRIDGE_MDB_FORM, "bridge", "bridges", "name", false);
    assert_lookup(&BRIDGE_MDB_FORM, "on-ports", "interfaces", "name", true);
    assert_eq!(
        BRIDGE_MDB_FORM.field("vid").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_lookup(&BRIDGE_MSTI_FORM, "bridge", "bridges", "name", false);

    assert_lookup(
        &BRIDGE_FILTER_FORM,
        "in-interface",
        "interfaces",
        "name",
        false,
    );
    assert_lookup(
        &BRIDGE_FILTER_FORM,
        "out-interface",
        "interfaces",
        "name",
        false,
    );
    assert_lookup(
        &BRIDGE_NAT_FORM,
        "in-interface",
        "interfaces",
        "name",
        false,
    );
    assert_lookup(
        &BRIDGE_NAT_FORM,
        "out-interface",
        "interfaces",
        "name",
        false,
    );

    assert_lookup(
        &BRIDGE_PORT_CONTROLLER_FORM,
        "switch",
        "switch",
        "name",
        false,
    );
    assert_lookup(
        &BRIDGE_PORT_CONTROLLER_FORM,
        "bridge",
        "bridges",
        "name",
        false,
    );
    assert_lookup(
        &BRIDGE_PORT_CONTROLLER_FORM,
        "cascade-ports",
        "interfaces",
        "name",
        true,
    );
    assert_lookup(
        &BRIDGE_PORT_CONTROLLER_DEVICE_FORM,
        "control-ports",
        "interfaces",
        "name",
        true,
    );
    assert_lookup(
        &BRIDGE_PORT_EXTENDER_FORM,
        "control-ports",
        "interfaces",
        "name",
        true,
    );
    assert_lookup(
        &BRIDGE_PORT_EXTENDER_FORM,
        "excluded-ports",
        "interfaces",
        "name",
        true,
    );
}

#[test]
fn rules_have_no_field_gates() {
    assert!(
        crate::features::bridge::rules::form_field_state("bridges", "pvid", &HashMap::new())
            .is_none()
    );
}
