use crate::features::queues::forms::*;
use crate::features::queues::guides::GUIDES;
use crate::features::queues::resources::RESOURCES;
use crate::features::queues::rules::form_field_state;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &QUEUE_SIMPLE_FORM,
    &QUEUE_TREE_FORM,
    &QUEUE_TYPE_FORM,
    &QUEUE_INTERFACE_FORM,
];

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

fn assert_lookup(
    schema: &FormSchema,
    key: &str,
    resource_id: &'static str,
    value_key: &'static str,
) {
    assert_eq!(
        schema.field(key).map(|field| field.kind),
        Some(FieldKind::Lookup {
            resource_id,
            value_key,
            multiple: false,
        })
    );
}

#[test]
fn catalog_and_guides_cover_the_queue_group() {
    assert_eq!(RESOURCES.len(), 4);
    assert_eq!(GUIDES.len(), 4);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
        assert!(spec.form.is_some(), "expected a form for {}", spec.id);
    }
    assert!(form_field_state("queue-simple", "disabled", &HashMap::new()).is_none());
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
fn disabled_fields_are_enabled_inverted_toggles() {
    assert_enabled_toggle(&QUEUE_SIMPLE_FORM);
    assert_enabled_toggle(&QUEUE_TREE_FORM);
    assert!(QUEUE_TYPE_FORM.field("disabled").is_none());
    assert!(QUEUE_INTERFACE_FORM.field("disabled").is_none());
}

#[test]
fn queue_simple_status_is_runtime_only() {
    assert_eq!(
        tab_ids(&QUEUE_SIMPLE_FORM),
        ["general", "advanced", "status"]
    );
    assert!(!QUEUE_SIMPLE_FORM.writable_keys().contains(&"rate"));
    assert!(!QUEUE_SIMPLE_FORM.writable_keys().contains(&"dropped"));
    assert!(QUEUE_SIMPLE_FORM.writable_keys().contains(&"burst-time"));
    assert!(QUEUE_SIMPLE_FORM.writable_keys().contains(&"packet-marks"));
    assert!(QUEUE_SIMPLE_FORM.create_keys().contains(&"max-limit"));
    assert_eq!(
        QUEUE_SIMPLE_FORM.field("target").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        QUEUE_SIMPLE_FORM
            .field("packet-marks")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_lookup(&QUEUE_SIMPLE_FORM, "parent", "queue-simple", "name");
    assert_eq!(
        QUEUE_SIMPLE_FORM.field("queue").map(|field| field.kind),
        Some(FieldKind::Text)
    );
}

#[test]
fn queue_tree_and_type_kinds() {
    assert_eq!(tab_ids(&QUEUE_TREE_FORM), ["general", "status"]);
    assert_lookup(&QUEUE_TREE_FORM, "parent", "queue-tree", "name");
    assert_lookup(&QUEUE_TREE_FORM, "queue", "queue-type", "name");
    assert!(QUEUE_TREE_FORM.writable_keys().contains(&"packet-mark"));
    assert!(QUEUE_TREE_FORM.writable_keys().contains(&"limit-at"));
    assert!(!QUEUE_TREE_FORM.writable_keys().contains(&"rate"));
    assert_eq!(
        QUEUE_TYPE_FORM.field("kind").map(|field| field.kind),
        Some(FieldKind::Enum {
            values: &[
                "pfifo", "red", "sfq", "pcq", "none", "mq-pfifo", "fq-codel", "cake",
            ],
        })
    );
    assert_eq!(
        QUEUE_TYPE_FORM.field("pfifo-limit").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        QUEUE_TYPE_FORM.field("sfq-perturb").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        QUEUE_TYPE_FORM.field("pcq-rate").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        QUEUE_TYPE_FORM
            .field("fq-codel-limit")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert!(QUEUE_TYPE_FORM.writable_keys().contains(&"pfifo-limit"));
}

#[test]
fn queue_interface_has_no_create() {
    assert_eq!(tab_ids(&QUEUE_INTERFACE_FORM), ["general"]);
    assert!(QUEUE_INTERFACE_FORM.create_sections.is_empty());
    assert_eq!(QUEUE_INTERFACE_FORM.writable_keys(), ["queue"]);
    assert_eq!(QUEUE_INTERFACE_FORM.known_keys(), ["interface", "queue"]);
    assert_lookup(&QUEUE_INTERFACE_FORM, "queue", "queue-type", "name");
    assert_eq!(
        QUEUE_INTERFACE_FORM
            .field("interface")
            .map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
}

#[test]
fn patch_body_skips_simple_counters() {
    let mut original = HashMap::new();
    original.insert("name".into(), "lan".into());
    original.insert("target".into(), "192.168.1.0/24".into());
    original.insert("rate".into(), "0/0".into());
    let mut current = original.clone();
    current.insert("max-limit".into(), "10M/10M".into());
    current.insert("rate".into(), "1M/2M".into());
    let body = patch_body(&QUEUE_SIMPLE_FORM, &original, &current, "********");
    assert_eq!(body.get("max-limit").map(String::as_str), Some("10M/10M"));
    assert!(!body.contains_key("rate"));
}

#[test]
fn unknown_tree_keys_land_on_status_extras() {
    let mut row = HashMap::new();
    row.insert("name".into(), "wan-out".into());
    row.insert("invalid".into(), "true".into());
    let extras = extra_status_fields(&QUEUE_TREE_FORM, &row);
    assert_eq!(extras, vec![("invalid".into(), "true".into())]);
}
