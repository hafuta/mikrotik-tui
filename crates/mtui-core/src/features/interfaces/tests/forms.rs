use crate::features::interfaces::forms::*;
use crate::forms::{FieldKind, FormSchema, default_writable_value, patch_body};
use std::collections::HashMap;

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
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
fn list_and_member_create_fields() {
    assert_eq!(create_keys(&LIST_FORM), LIST_FORM.writable_keys());
    assert!(
        LIST_FORM
            .writable_keys()
            .iter()
            .all(|key| *key != "disabled")
    );
    assert_eq!(create_keys(&MEMBER_FORM), MEMBER_FORM.writable_keys());
    assert_eq!(
        MEMBER_FORM.field("disabled").map(|field| field.label),
        Some("Enabled")
    );
    assert_eq!(
        MEMBER_FORM.field("disabled").map(|field| field.kind),
        Some(FieldKind::InvertedToggle)
    );
}

#[test]
fn macsec_create_matches_webfig() {
    assert!(create_keys(&MACSEC_FORM).contains(&"name"));
    assert!(create_keys(&MACSEC_FORM).contains(&"interface"));
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
}

#[test]
fn interface_lookups_use_catalog_resources() {
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
    assert_lookup(&WIFI_FORM, "master-interface", "wifi", false);
    assert_lookup(&BONDING_FORM, "slaves", "interfaces", true);
    assert_eq!(
        VRF_FORM.field("interfaces").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert!(matches!(
        VXLAN_FORM.field("group").map(|field| field.kind),
        Some(FieldKind::Optional { .. })
    ));
    assert!(matches!(
        BONDING_FORM.field("mode").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
}

#[test]
fn list_and_vrf_lookups() {
    assert_lookup(&MEMBER_FORM, "list", "interface-lists", false);
    assert_lookup(&LIST_FORM, "include", "interface-lists", true);
    assert_lookup(&LIST_FORM, "exclude", "interface-lists", true);
    assert_lookup(&VXLAN_FORM, "vtep-vrf", "vrf", false);
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
fn veth_keeps_static_controls_when_dhcp_is_on() {
    assert_eq!(
        VETH_FORM.field("address").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        VETH_FORM.field("dhcp").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert!(crate::forms::field_visible(
        "veth",
        "gateway",
        &HashMap::from([("dhcp".into(), "true".into())])
    ));
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

#[test]
fn lte_sheet_matches_webfig_7215() {
    assert!(LTE_FORM.create_sections.is_empty());
    assert_eq!(
        LTE_FORM
            .sections
            .iter()
            .map(|section| section.id)
            .collect::<Vec<_>>(),
        ["general", "status", "cellular", "capabilities", "traffic"]
    );
    assert_lookup(&LTE_FORM, "apn-profiles", "lte-apn", true);
    assert_eq!(
        LTE_FORM.field("network-mode").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        LTE_FORM.field("pin").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        LTE_FORM.field("allow-roaming").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert!(LTE_FORM.field("sms-protocol").is_none());
    assert!(
        LTE_FORM
            .sections
            .iter()
            .find(|section| section.id == "status")
            .is_some_and(|section| section.read_only)
    );
}

#[test]
fn lte_apn_kinds_match_webfig() {
    assert!(create_keys(&LTE_APN_FORM).contains(&"name"));
    assert!(create_keys(&LTE_APN_FORM).contains(&"apn"));
    assert!(matches!(
        LTE_APN_FORM.field("ip-type").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
    assert_eq!(
        LTE_APN_FORM
            .field("ip-type")
            .unwrap()
            .kind
            .display_value("auto"),
        "Auto"
    );
    assert!(LTE_APN_FORM.field("passthrough-subnet-selection").is_none());
    assert!(LTE_APN_FORM.field("passthrough-subnet-size").is_some());
    assert_eq!(
        LTE_APN_FORM.field("password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_lookup(&LTE_APN_FORM, "passthrough-interface", "interfaces", false);
    assert_eq!(
        default_writable_value(LTE_APN_FORM.field("authentication").unwrap().kind),
        "none"
    );

    let mut original = HashMap::new();
    original.insert("name".into(), "default".into());
    original.insert("apn".into(), "internet".into());
    original.insert("password".into(), "********".into());
    original.insert("authentication".into(), "chap".into());
    let mut current = original.clone();
    current.insert("apn".into(), "lte.provider".into());
    current.insert("password".into(), "********".into());
    let body = patch_body(&LTE_APN_FORM, &original, &current, "********");
    assert_eq!(body.get("apn").map(String::as_str), Some("lte.provider"));
    assert!(!body.contains_key("password"));
}

#[test]
fn vlan_and_ethernet_drop_legacy_tabs() {
    assert_eq!(
        VLAN_FORM
            .sections
            .iter()
            .map(|section| section.id)
            .collect::<Vec<_>>(),
        ["general", "loop-protect", "status", "traffic"]
    );
    assert_eq!(VLAN_FORM.field("l2mtu").unwrap().kind, FieldKind::Readonly);
    assert!(!ETHERNET_FORM.field("full-duplex").unwrap().kind.writable());
    assert!(INTERFACES_FORM.field("mac-address").is_none());
}
