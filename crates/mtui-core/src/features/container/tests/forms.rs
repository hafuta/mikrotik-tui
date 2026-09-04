use std::collections::HashMap;

use crate::features::container::forms::*;
use crate::features::container::guides::GUIDES;
use crate::features::container::resources::RESOURCES;
use crate::features::container::rules::form_field_state;
use crate::form_fields::STOP_SIGNAL;
use crate::forms::{FieldKind, FormSchema};

const FORMS: &[&FormSchema] = &[
    &CONTAINER_CONFIG_FORM,
    &CONTAINER_ENV_FORM,
    &CONTAINER_MOUNT_FORM,
    &CONTAINER_FORM,
    &APP_FORM,
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

fn assert_create_omits_status(schema: &FormSchema) {
    assert!(
        schema
            .sections_for(true)
            .iter()
            .all(|section| section.id != "status" && !section.hidden_on_create())
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
fn catalog_and_guides_cover_the_container_group() {
    assert_eq!(RESOURCES.len(), 5);
    assert_eq!(GUIDES.len(), 5);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
        assert_eq!(spec.group, "container-group");
        assert!(spec.form.is_some());
    }
    assert_eq!(
        RESOURCES.iter().map(|spec| spec.id).collect::<Vec<_>>(),
        [
            "containers",
            "container-config",
            "container-envs",
            "container-mounts",
            "apps",
        ]
    );
}

#[test]
fn create_is_full_writable_sheet_without_status() {
    for form in FORMS {
        assert!(form.create_sections.is_empty());
        assert_eq!(create_keys(form), form.writable_keys());
        assert_create_omits_status(form);
        assert_status_readonly(form);
        if let Some(disabled) = form.field("disabled") {
            assert_eq!(disabled.label, "Enabled");
            assert_eq!(disabled.kind, FieldKind::InvertedToggle);
        }
    }
}

#[test]
fn config_password_is_secret() {
    assert_eq!(
        CONTAINER_CONFIG_FORM
            .field("password")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        CONTAINER_CONFIG_FORM.field("memory-max").map(|f| f.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        CONTAINER_CONFIG_FORM
            .field("assumed-registry-url")
            .map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
}

#[test]
fn env_and_mount_kinds() {
    assert_eq!(
        CONTAINER_ENV_FORM.field("value").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert!(CONTAINER_ENV_FORM.writable_keys().contains(&"comment"));
    assert!(CONTAINER_MOUNT_FORM.writable_keys().contains(&"comment"));
}

#[test]
fn container_lookups_enums_and_runtime_status() {
    assert_lookup(&CONTAINER_FORM, "interface", "veth", "name", false);
    assert_lookup(&CONTAINER_FORM, "file", "files", "name", false);
    assert_lookup(&CONTAINER_FORM, "envlist", "container-envs", "list", false);
    assert_lookup(
        &CONTAINER_FORM,
        "mountlists",
        "container-mounts",
        "list",
        true,
    );
    assert_eq!(
        CONTAINER_FORM.field("hosts").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    match CONTAINER_FORM.field("restart-policy").map(|f| f.kind) {
        Some(FieldKind::Enum { values }) => assert_eq!(values, RESTART_POLICY),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        CONTAINER_FORM.field("stop-signal").map(|field| field.kind),
        Some(FieldKind::LabeledEnum {
            choices: STOP_SIGNAL,
        })
    );
    assert_eq!(
        CONTAINER_FORM
            .field("start-on-boot")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        CONTAINER_FORM.field("status").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_eq!(
        CONTAINER_FORM.field("arch").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_eq!(
        CONTAINER_FORM.field("tag").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(!CONTAINER_FORM.writable_keys().contains(&"status"));
    assert!(!CONTAINER_FORM.writable_keys().contains(&"arch"));
    assert!(!CONTAINER_FORM.writable_keys().contains(&"tag"));
}

#[test]
fn app_network_enum() {
    match APP_FORM.field("network").map(|field| field.kind) {
        Some(FieldKind::Enum { values }) => assert_eq!(values, APP_NETWORK),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        APP_FORM.field("environment").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        APP_FORM.field("pvid").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        APP_FORM.field("status").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
}

#[test]
fn rules_stub_leaves_visibility_ungated() {
    let values = HashMap::from([("remote-image".to_string(), "pihole/pihole".to_string())]);
    assert!(form_field_state("containers", "file", &values).is_none());
    assert!(form_field_state("containers", "remote-image", &HashMap::new()).is_none());
}
