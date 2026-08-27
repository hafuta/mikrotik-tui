use std::collections::HashMap;

use crate::features::radius::forms::*;
use crate::features::radius::guides::GUIDES;
use crate::features::radius::resources::RESOURCES;
use crate::features::radius::rules::form_field_state;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};

const FORMS: &[&FormSchema] = &[&RADIUS_FORM, &RADIUS_INCOMING_FORM];

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

fn assert_create_omits_status(schema: &FormSchema) {
    assert!(
        schema
            .sections_for(true)
            .iter()
            .all(|section| section.id != "status" && !section.hidden_on_create())
    );
}

#[test]
fn catalog_and_guides_cover_the_radius_group() {
    assert_eq!(RESOURCES.len(), 2);
    assert_eq!(GUIDES.len(), 2);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
        assert!(spec.form.is_some());
    }
    assert_eq!(RESOURCES[0].id, "radius");
    assert_eq!(RESOURCES[1].id, "radius-incoming");
    assert!(form_field_state("radius", "secret", &HashMap::new()).is_none());
}

#[test]
fn create_matches_writable_sheet_and_omits_status() {
    for form in FORMS {
        assert!(form.create_sections.is_empty());
        assert_eq!(create_keys(form), form.writable_keys());
        assert_create_omits_status(form);
        assert_status_readonly(form);
    }
}

#[test]
fn radius_form_matches_webfig_kinds() {
    assert_eq!(
        RADIUS_FORM.writable_keys(),
        [
            "address",
            "protocol",
            "secret",
            "service",
            "authentication-port",
            "accounting-port",
            "timeout",
            "src-address",
            "comment",
            "disabled",
        ]
    );
    assert_eq!(
        RADIUS_FORM.field("secret").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        RADIUS_FORM.field("disabled").map(|field| field.label),
        Some("Enabled")
    );
    assert_eq!(
        RADIUS_FORM.field("disabled").map(|field| field.kind),
        Some(FieldKind::InvertedToggle)
    );
    assert_eq!(
        RADIUS_FORM.field("protocol").map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &["udp", "tcp", "radsec"],
        })
    );
    assert_eq!(
        RADIUS_FORM.field("service").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        RADIUS_FORM
            .field("authentication-port")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        RADIUS_FORM.field("timeout").map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_eq!(
        RADIUS_FORM.field("src-address").map(|field| field.kind),
        Some(FieldKind::Ip)
    );
    assert!(
        RADIUS_FORM
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    );
}

#[test]
fn radius_incoming_is_singleton_general() {
    assert_eq!(RADIUS_INCOMING_FORM.writable_keys(), ["accept", "port"]);
    assert_eq!(
        RADIUS_INCOMING_FORM.field("accept").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        RADIUS_INCOMING_FORM.field("port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert!(RADIUS_INCOMING_FORM.field("disabled").is_none());
    assert!(
        RADIUS_INCOMING_FORM
            .sections
            .iter()
            .all(|section| section.id != "status")
    );
}

#[test]
fn patch_body_keeps_masked_radius_secret() {
    let mut original = HashMap::new();
    original.insert("address".into(), "192.0.2.10".into());
    original.insert("secret".into(), "********".into());
    original.insert("service".into(), "login".into());
    original.insert("timeout".into(), "300ms".into());
    let mut current = original.clone();
    current.insert("timeout".into(), "1s".into());
    current.insert("secret".into(), "********".into());
    let body = patch_body(&RADIUS_FORM, &original, &current, "********");
    assert_eq!(body.get("timeout").map(String::as_str), Some("1s"));
    assert!(!body.contains_key("secret"));
    assert!(!body.contains_key("address"));
}

#[test]
fn patch_body_sends_changed_radius_secret() {
    let mut original = HashMap::new();
    original.insert("secret".into(), "********".into());
    let mut current = original.clone();
    current.insert("secret".into(), "new-shared-secret".into());
    let body = patch_body(&RADIUS_FORM, &original, &current, "********");
    assert_eq!(
        body.get("secret").map(String::as_str),
        Some("new-shared-secret")
    );
}

#[test]
fn unknown_radius_keys_land_on_status_extras() {
    let mut row = HashMap::new();
    row.insert("address".into(), "192.0.2.10".into());
    row.insert("called-id".into(), "isp".into());
    let extras = extra_status_fields(&RADIUS_FORM, &row);
    assert_eq!(extras, vec![("called-id".into(), "isp".into())]);
}
