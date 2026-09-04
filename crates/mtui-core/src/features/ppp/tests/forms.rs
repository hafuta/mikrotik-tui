use crate::features::ppp::forms::*;
use crate::form_fields::KIND_USE_IPSEC_REQUIRE;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &PPP_SECRET_FORM,
    &PPP_PROFILE_FORM,
    &PPP_AAA_FORM,
    &PPP_CLIENT_FORM,
    &PPPOE_CLIENT_FORM,
    &PPPOE_SERVER_FORM,
    &PPTP_CLIENT_FORM,
    &PPTP_SERVER_FORM,
    &L2TP_CLIENT_FORM,
    &L2TP_SERVER_FORM,
    &SSTP_CLIENT_FORM,
    &SSTP_SERVER_FORM,
    &OVPN_CLIENT_FORM,
    &OVPN_SERVER_FORM,
];

fn create_keys(form: &FormSchema) -> Vec<&'static str> {
    form.create_keys()
}

#[test]
fn writable_keys_include_secret_fields() {
    for form in [
        &PPP_SECRET_FORM,
        &PPP_CLIENT_FORM,
        &PPPOE_CLIENT_FORM,
        &PPTP_CLIENT_FORM,
        &L2TP_CLIENT_FORM,
        &SSTP_CLIENT_FORM,
        &OVPN_CLIENT_FORM,
    ] {
        assert!(form.writable_keys().contains(&"password"));
        assert_eq!(
            form.field("password").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
    }
    for form in [&L2TP_CLIENT_FORM, &L2TP_SERVER_FORM] {
        assert!(form.writable_keys().contains(&"ipsec-secret"));
        assert_eq!(
            form.field("ipsec-secret").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
    }
    assert!(!PPP_CLIENT_FORM.writable_keys().contains(&"running"));
    assert!(!PPPOE_CLIENT_FORM.writable_keys().contains(&"status"));
}

#[test]
fn patch_body_omits_masked_password() {
    let mut original = HashMap::new();
    original.insert("name".into(), "user1".into());
    original.insert("password".into(), "********".into());
    original.insert("profile".into(), "default".into());
    let mut current = original.clone();
    current.insert("profile".into(), "office".into());
    current.insert("password".into(), "********".into());
    let body = patch_body(&PPP_SECRET_FORM, &original, &current, "********");
    assert_eq!(body.get("profile").map(String::as_str), Some("office"));
    assert!(!body.contains_key("password"));
}

#[test]
fn patch_body_omits_masked_ipsec_secret() {
    let mut original = HashMap::new();
    original.insert("name".into(), "l2tp1".into());
    original.insert("ipsec-secret".into(), "********".into());
    original.insert("connect-to".into(), "1.1.1.1".into());
    let mut current = original.clone();
    current.insert("connect-to".into(), "8.8.8.8".into());
    current.insert("ipsec-secret".into(), "********".into());
    let body = patch_body(&L2TP_CLIENT_FORM, &original, &current, "********");
    assert_eq!(body.get("connect-to").map(String::as_str), Some("8.8.8.8"));
    assert!(!body.contains_key("ipsec-secret"));
    assert!(!body.contains_key("running"));
}

#[test]
fn create_matches_writable_sheet_and_omits_status() {
    for form in FORMS {
        assert_eq!(form.create_keys(), form.writable_keys());
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| !section.hidden_on_create())
        );
    }
    assert!(create_keys(&PPP_SECRET_FORM).contains(&"local-address"));
    assert!(create_keys(&PPP_SECRET_FORM).contains(&"remote-address"));
    assert!(create_keys(&PPP_SECRET_FORM).contains(&"caller-id"));
    assert!(PPP_AAA_FORM.create_sections.is_empty());
    assert!(PPTP_SERVER_FORM.create_sections.is_empty());
    assert!(L2TP_SERVER_FORM.create_sections.is_empty());
    assert!(SSTP_SERVER_FORM.create_sections.is_empty());
    assert!(OVPN_SERVER_FORM.create_sections.is_empty());
}

#[test]
fn no_empty_advanced_tabs() {
    for form in FORMS {
        assert!(
            form.sections
                .iter()
                .all(|section| section.id != "advanced" || !section.fields.is_empty())
        );
        assert!(form.sections.iter().all(|section| section.id != "advanced"));
    }
}

#[test]
fn unknown_keys_land_on_status_extras() {
    let mut row = HashMap::new();
    row.insert("name".into(), "pppoe1".into());
    row.insert("dynamic".into(), "true".into());
    let extras = extra_status_fields(&PPPOE_CLIENT_FORM, &row);
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
    for form in [
        &PPP_SECRET_FORM,
        &PPP_CLIENT_FORM,
        &PPPOE_CLIENT_FORM,
        &PPTP_CLIENT_FORM,
        &L2TP_CLIENT_FORM,
        &SSTP_CLIENT_FORM,
        &OVPN_CLIENT_FORM,
    ] {
        assert_lookup(form, "profile", "ppp-profiles");
    }
    for form in [
        &PPPOE_SERVER_FORM,
        &PPTP_SERVER_FORM,
        &L2TP_SERVER_FORM,
        &SSTP_SERVER_FORM,
        &OVPN_SERVER_FORM,
    ] {
        assert_lookup(form, "default-profile", "ppp-profiles");
    }
    assert_lookup(&PPP_PROFILE_FORM, "bridge", "bridges");
    assert_lookup(&PPP_PROFILE_FORM, "interface-list", "interface-lists");
    assert_lookup(&PPP_PROFILE_FORM, "local-address", "pools");
    assert_lookup(&PPP_PROFILE_FORM, "remote-address", "pools");
    assert_lookup(&PPP_CLIENT_FORM, "port", "ports");
    assert_lookup(&PPPOE_CLIENT_FORM, "interface", "interfaces");
    assert_lookup(&PPPOE_SERVER_FORM, "interface", "interfaces");
    for form in [
        &SSTP_CLIENT_FORM,
        &SSTP_SERVER_FORM,
        &OVPN_CLIENT_FORM,
        &OVPN_SERVER_FORM,
    ] {
        assert_lookup(form, "certificate", "certificates");
    }
}

#[test]
fn disabled_uses_enabled_inverted_toggle() {
    for form in [
        &PPP_SECRET_FORM,
        &PPP_CLIENT_FORM,
        &PPPOE_CLIENT_FORM,
        &PPPOE_SERVER_FORM,
        &PPTP_CLIENT_FORM,
        &L2TP_CLIENT_FORM,
        &SSTP_CLIENT_FORM,
        &OVPN_CLIENT_FORM,
    ] {
        assert_eq!(
            form.field("disabled").map(|field| field.label),
            Some("Enabled")
        );
        assert_eq!(
            form.field("disabled").map(|field| field.kind),
            Some(FieldKind::InvertedToggle)
        );
    }
}

#[test]
fn non_resource_fields_stay_plain_text_or_secret() {
    assert_eq!(
        PPP_SECRET_FORM.field("password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        PPP_SECRET_FORM
            .field("local-address")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        PPP_CLIENT_FORM.field("phone").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        PPPOE_CLIENT_FORM
            .field("service-name")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        PPPOE_SERVER_FORM
            .field("authentication")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        PPP_PROFILE_FORM.field("dns-server").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert!(matches!(
        OVPN_CLIENT_FORM.field("auth").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
    assert!(matches!(
        OVPN_CLIENT_FORM.field("cipher").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
    assert!(matches!(
        PPP_SECRET_FORM.field("service").map(|field| field.kind),
        Some(FieldKind::LabeledEnum { .. })
    ));
    assert_eq!(
        PPPOE_SERVER_FORM
            .field("keepalive-timeout")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_eq!(
        SSTP_SERVER_FORM.field("port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        OVPN_SERVER_FORM.field("port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        OVPN_CLIENT_FORM.field("port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        PPTP_CLIENT_FORM.field("connect-to").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        PPP_CLIENT_FORM
            .field("add-default-route")
            .map(|field| field.label),
        Some("Add Default Route")
    );
    assert_eq!(
        L2TP_CLIENT_FORM.field("use-ipsec").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        L2TP_SERVER_FORM.field("use-ipsec").map(|field| field.kind),
        Some(KIND_USE_IPSEC_REQUIRE)
    );
}
