use crate::features::wireguard::forms::*;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

#[test]
fn wireguard_edit_matches_winbox_properties() {
    assert_eq!(
        WIREGUARD_FORM.writable_keys(),
        [
            "name",
            "listen-port",
            "mtu",
            "private-key",
            "vrf",
            "comment",
            "disabled",
        ]
    );
    assert_eq!(
        WIREGUARD_FORM.known_keys(),
        [
            "name",
            "listen-port",
            "mtu",
            "private-key",
            "vrf",
            "comment",
            "disabled",
            "public-key",
            "running",
        ]
    );
    let create_ids: Vec<_> = WIREGUARD_FORM
        .sections_for(true)
        .iter()
        .map(|section| section.id)
        .collect();
    assert_eq!(create_ids, ["general"]);
    assert_eq!(WIREGUARD_FORM.create_keys(), WIREGUARD_FORM.writable_keys());
    assert!(
        WIREGUARD_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
    assert_eq!(
        WIREGUARD_FORM.field("disabled").map(|field| field.label),
        Some("Enabled")
    );
    assert_eq!(
        WIREGUARD_FORM.field("disabled").map(|field| field.kind),
        Some(FieldKind::InvertedToggle)
    );
}

#[test]
fn peer_form_create_includes_client_fields_and_omits_status() {
    let tabs: Vec<_> = WIREGUARD_PEER_FORM
        .sections
        .iter()
        .map(|section| section.id)
        .collect();
    assert_eq!(tabs, ["general", "client", "status"]);
    assert!(
        WIREGUARD_PEER_FORM
            .sections
            .iter()
            .find(|section| section.id == "status")
            .is_some_and(|section| section.read_only)
    );
    assert_eq!(
        WIREGUARD_PEER_FORM.create_keys(),
        WIREGUARD_PEER_FORM.writable_keys()
    );
    assert!(
        WIREGUARD_PEER_FORM
            .sections_for(true)
            .iter()
            .all(|section| section.id != "status")
    );
    assert!(
        WIREGUARD_PEER_FORM
            .writable_keys()
            .contains(&"preshared-key")
    );
    assert!(WIREGUARD_PEER_FORM.writable_keys().contains(&"client-mtu"));
    assert!(!WIREGUARD_PEER_FORM.writable_keys().contains(&"rx"));
    assert_eq!(
        WIREGUARD_PEER_FORM
            .field("allowed-address")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
}

#[test]
fn patch_body_keeps_masked_private_key() {
    let mut original = HashMap::new();
    original.insert("name".into(), "wg1".into());
    original.insert("private-key".into(), "********".into());
    original.insert("listen-port".into(), "13231".into());
    let mut current = original.clone();
    current.insert("listen-port".into(), "51820".into());
    current.insert("private-key".into(), "********".into());
    let body = patch_body(&WIREGUARD_FORM, &original, &current, "********");
    assert_eq!(body.get("listen-port").map(String::as_str), Some("51820"));
    assert!(!body.contains_key("private-key"));
    assert!(!body.contains_key("public-key"));
}

#[test]
fn unknown_peer_keys_land_on_status_extras() {
    let mut row = HashMap::new();
    row.insert("interface".into(), "wg1".into());
    row.insert("dynamic".into(), "true".into());
    let extras = extra_status_fields(&WIREGUARD_PEER_FORM, &row);
    assert_eq!(extras, vec![("dynamic".into(), "true".into())]);
}

fn assert_lookup(form: &FormSchema, key: &str, resource_id: &'static str) {
    assert_eq!(
        form.field(key).map(|field| field.kind),
        Some(FieldKind::Lookup {
            resource_id,
            value_key: "name",
            multiple: false,
        }),
        "{key}"
    );
}

#[test]
fn lookup_fields_point_at_named_resources() {
    assert_lookup(&WIREGUARD_PEER_FORM, "interface", "wireguard");
    assert_lookup(&WIREGUARD_FORM, "vrf", "vrf");
    assert_eq!(
        WIREGUARD_PEER_FORM
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter())
            .find(|field| field.key == "interface")
            .map(|field| field.kind),
        Some(FieldKind::Lookup {
            resource_id: "wireguard",
            value_key: "name",
            multiple: false,
        })
    );
}

#[test]
fn non_resource_fields_stay_plain_text_or_secret() {
    assert_eq!(
        WIREGUARD_PEER_FORM
            .field("public-key")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        WIREGUARD_PEER_FORM
            .field("private-key")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert!(matches!(
        WIREGUARD_PEER_FORM
            .field("endpoint-address")
            .map(|field| field.kind),
        Some(FieldKind::Optional { .. })
    ));
    assert!(matches!(
        WIREGUARD_PEER_FORM
            .field("endpoint-port")
            .map(|field| field.kind),
        Some(FieldKind::Optional { .. })
    ));
    assert_eq!(
        WIREGUARD_PEER_FORM
            .field("client-address")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
}
