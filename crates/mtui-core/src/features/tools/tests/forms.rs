use crate::features::tools::forms::*;
use crate::features::tools::guides::GUIDES;
use crate::features::tools::resources::RESOURCES;
use crate::features::tools::rules::form_field_state;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &NETWATCH_FORM,
    &EMAIL_FORM,
    &ROMON_FORM,
    &ROMON_PORT_FORM,
    &GRAPHING_FORM,
    &GRAPHING_INTERFACE_FORM,
    &GRAPHING_QUEUE_FORM,
    &GRAPHING_RESOURCE_FORM,
    &SNIFFER_FORM,
    &WOL_PROMPT,
    &SMS_PROMPT,
];

const ENABLED_FORMS: &[&FormSchema] = &[
    &NETWATCH_FORM,
    &ROMON_PORT_FORM,
    &GRAPHING_INTERFACE_FORM,
    &GRAPHING_QUEUE_FORM,
    &GRAPHING_RESOURCE_FORM,
];

const STORE_EVERY: &[&str] = &["5min", "hour", "24hours"];
const NETWATCH_TYPE: &[&str] = &["icmp", "simple", "tcp-conn", "http", "https", "dns"];
const TLS_VALUES: &[&str] = &["yes", "starttls", "no"];

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
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

fn assert_enum(schema: &FormSchema, key: &str, values: &'static [&'static str]) {
    assert_eq!(
        schema.field(key).map(|field| field.kind),
        Some(FieldKind::Enum { values })
    );
}

fn assert_label(schema: &FormSchema, key: &str, label: &str) {
    assert_eq!(schema.field(key).map(|field| field.label), Some(label));
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

fn no_advanced(schema: &FormSchema) -> bool {
    schema
        .sections
        .iter()
        .all(|section| section.id != "advanced")
}

#[test]
fn catalog_and_guides_cover_the_tools_group() {
    assert_eq!(RESOURCES.len(), 18);
    assert_eq!(GUIDES.len(), 18);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
    }
    for id in [
        "ping",
        "traceroute",
        "bandwidth-test",
        "flood-ping",
        "mac-scan",
        "ip-scan",
        "profiler",
        "wol",
        "sms",
    ] {
        let spec = RESOURCES
            .iter()
            .find(|spec| spec.id == id)
            .unwrap_or_else(|| panic!("missing catalog id {id}"));
        assert!(spec.form.is_none(), "{id} must stay overlay-only");
    }
    assert!(form_field_state("netwatch", "type", &HashMap::new()).is_none());
}

#[test]
fn create_matches_writable_sheet_and_omits_status() {
    for form in FORMS {
        assert!(form.create_sections.is_empty(), "{}", form.title_key);
        assert_eq!(create_keys(form), form.writable_keys());
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| section.id != "status" && !section.hidden_on_create())
        );
        assert_status_readonly(form);
    }
}

#[test]
fn disabled_fields_are_enabled_inverted_toggles() {
    for form in ENABLED_FORMS {
        assert_eq!(
            form.field("disabled").map(|field| field.label),
            Some("Enabled")
        );
        assert_eq!(
            form.field("disabled").map(|field| field.kind),
            Some(FieldKind::InvertedToggle)
        );
    }
    assert!(NETWATCH_FORM.field("enabled").is_none());
    assert_eq!(
        ROMON_FORM.field("enabled").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
}

#[test]
fn netwatch_create_is_full_writable_sheet() {
    assert_eq!(create_keys(&NETWATCH_FORM), NETWATCH_FORM.writable_keys());
    assert!(!NETWATCH_FORM.writable_keys().contains(&"status"));
    assert!(!NETWATCH_FORM.writable_keys().contains(&"done-tests"));
    assert!(NETWATCH_FORM.writable_keys().contains(&"up-script"));
    assert_lookup(&NETWATCH_FORM, "up-script", "scripts", "name");
    assert_lookup(&NETWATCH_FORM, "down-script", "scripts", "name");
    assert_eq!(
        NETWATCH_FORM.field("host").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        NETWATCH_FORM.field("interval").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_enum(&NETWATCH_FORM, "type", NETWATCH_TYPE);
    assert_label(&NETWATCH_FORM, "up-script", "Up script");
    assert_label(&NETWATCH_FORM, "start-delay", "Start delay");
    assert_label(&NETWATCH_FORM, "done-tests", "Done tests");
}

#[test]
fn email_is_singleton_without_create() {
    assert!(EMAIL_FORM.create_sections.is_empty());
    assert_eq!(
        EMAIL_FORM.writable_keys(),
        ["server", "from", "user", "password", "tls", "port"]
    );
    assert_eq!(
        EMAIL_FORM.field("password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_enum(&EMAIL_FORM, "tls", TLS_VALUES);
    assert_eq!(
        EMAIL_FORM.field("port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
}

#[test]
fn patch_body_keeps_masked_email_password() {
    let mut original = HashMap::new();
    original.insert("server".into(), "smtp.example.com".into());
    original.insert("password".into(), "********".into());
    original.insert("port".into(), "587".into());
    let mut current = original.clone();
    current.insert("port".into(), "465".into());
    current.insert("password".into(), "********".into());
    let body = patch_body(&EMAIL_FORM, &original, &current, "********");
    assert_eq!(body.get("port").map(String::as_str), Some("465"));
    assert!(!body.contains_key("password"));
}

#[test]
fn romon_is_singleton_with_status_and_secret_list() {
    assert!(ROMON_FORM.create_sections.is_empty());
    assert_eq!(ROMON_FORM.writable_keys(), ["enabled", "id", "secrets"]);
    assert!(!ROMON_FORM.writable_keys().contains(&"current-id"));
    assert!(
        ROMON_FORM
            .sections
            .iter()
            .find(|section| section.id == "status")
            .is_some_and(|section| section.read_only)
    );
    assert!(no_advanced(&ROMON_FORM));
    assert_eq!(
        ROMON_FORM.field("enabled").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        ROMON_FORM.field("id").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        ROMON_FORM.field("secrets").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        ROMON_FORM.field("current-id").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_label(&ROMON_FORM, "id", "ID");
    assert_label(&ROMON_FORM, "secrets", "Secrets");
    assert_label(&ROMON_FORM, "current-id", "Current ID");
}

#[test]
fn romon_port_create_is_full_sheet_with_lookup_and_secret() {
    assert_eq!(
        create_keys(&ROMON_PORT_FORM),
        ROMON_PORT_FORM.writable_keys()
    );
    assert_eq!(
        ROMON_PORT_FORM.writable_keys(),
        [
            "interface",
            "forbid",
            "cost",
            "secrets",
            "comment",
            "disabled"
        ]
    );
    assert!(no_advanced(&ROMON_PORT_FORM));
    assert_lookup(&ROMON_PORT_FORM, "interface", "interfaces", "name");
    assert_eq!(
        ROMON_PORT_FORM.field("forbid").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        ROMON_PORT_FORM.field("cost").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        ROMON_PORT_FORM.field("secrets").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_label(&ROMON_PORT_FORM, "forbid", "Forbid");
    assert_label(&ROMON_PORT_FORM, "cost", "Cost");
    assert!(FieldKind::Number.accepts_char("cost", "100", '0'));
    assert!(!FieldKind::Number.accepts_char("cost", "100", 'a'));
}

#[test]
fn romon_patch_omits_masked_secrets_and_readonly_current_id() {
    let mut original = HashMap::new();
    original.insert("enabled".into(), "true".into());
    original.insert("id".into(), "00:00:00:00:00:00".into());
    original.insert("secrets".into(), "********".into());
    original.insert("current-id".into(), "DC:2C:6E:9E:11:27".into());
    let mut current = original.clone();
    current.insert("enabled".into(), "false".into());
    current.insert("secrets".into(), "********".into());
    current.insert("current-id".into(), "aa:bb:cc:dd:ee:ff".into());
    let body = patch_body(&ROMON_FORM, &original, &current, "********");
    assert_eq!(body.get("enabled").map(String::as_str), Some("false"));
    assert!(!body.contains_key("secrets"));
    assert!(!body.contains_key("current-id"));
}

#[test]
fn graphing_settings_use_store_every_enum() {
    assert!(GRAPHING_FORM.create_sections.is_empty());
    assert_eq!(
        GRAPHING_FORM.writable_keys(),
        ["store-every", "page-refresh"]
    );
    assert!(no_advanced(&GRAPHING_FORM));
    assert_enum(&GRAPHING_FORM, "store-every", STORE_EVERY);
    assert_eq!(STORE_EVERY[0], "5min");
    assert_eq!(
        GRAPHING_FORM.field("page-refresh").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_label(&GRAPHING_FORM, "store-every", "Store Every");
    assert_label(&GRAPHING_FORM, "page-refresh", "Page Refresh");
}

#[test]
fn graphing_children_use_lookups_toggles_and_full_create() {
    assert_eq!(
        create_keys(&GRAPHING_INTERFACE_FORM),
        GRAPHING_INTERFACE_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&GRAPHING_QUEUE_FORM),
        GRAPHING_QUEUE_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&GRAPHING_RESOURCE_FORM),
        GRAPHING_RESOURCE_FORM.writable_keys()
    );
    assert_lookup(&GRAPHING_INTERFACE_FORM, "interface", "interfaces", "name");
    assert_lookup(&GRAPHING_QUEUE_FORM, "simple-queue", "queue-simple", "name");
    assert_eq!(
        GRAPHING_INTERFACE_FORM
            .field("store-on-disk")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        GRAPHING_QUEUE_FORM
            .field("allow-target")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        GRAPHING_INTERFACE_FORM
            .field("allow-address")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_eq!(
        GRAPHING_INTERFACE_FORM
            .field("comment")
            .map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert!(no_advanced(&GRAPHING_INTERFACE_FORM));
    assert!(no_advanced(&GRAPHING_QUEUE_FORM));
    assert!(no_advanced(&GRAPHING_RESOURCE_FORM));
    assert_label(&GRAPHING_INTERFACE_FORM, "allow-address", "Allow Address");
    assert_label(&GRAPHING_INTERFACE_FORM, "store-on-disk", "Store On Disk");
    assert_label(&GRAPHING_QUEUE_FORM, "simple-queue", "Simple Queue");
    assert_label(&GRAPHING_QUEUE_FORM, "allow-target", "Allow Target");
    assert_eq!(
        GRAPHING_INTERFACE_FORM.writable_keys(),
        [
            "interface",
            "allow-address",
            "store-on-disk",
            "comment",
            "disabled"
        ]
    );
    assert_eq!(
        GRAPHING_QUEUE_FORM.writable_keys(),
        [
            "simple-queue",
            "allow-address",
            "allow-target",
            "store-on-disk",
            "comment",
            "disabled"
        ]
    );
    assert_eq!(
        GRAPHING_RESOURCE_FORM.writable_keys(),
        ["allow-address", "store-on-disk", "comment", "disabled"]
    );
}

#[test]
fn graphing_optional_comment_is_omitted_from_unchanged_patch() {
    let mut original = HashMap::new();
    original.insert("interface".into(), "all".into());
    original.insert("allow-address".into(), "0.0.0.0/0".into());
    original.insert("store-on-disk".into(), "true".into());
    let mut current = original.clone();
    current.insert("store-on-disk".into(), "false".into());
    let body = patch_body(&GRAPHING_INTERFACE_FORM, &original, &current, "********");
    assert_eq!(body.get("store-on-disk").map(String::as_str), Some("false"));
    assert!(!body.contains_key("comment"));
    assert!(!body.contains_key("interface"));
}

#[test]
fn sniffer_and_prompts_keep_lookups_and_channel_number() {
    assert_lookup(&SNIFFER_FORM, "interface", "interfaces", "name");
    assert_lookup(&SNIFFER_FORM, "filter-interface", "interfaces", "name");
    assert_eq!(
        SNIFFER_FORM.field("filter-stream").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_lookup(&WOL_PROMPT, "interface", "interfaces", "name");
    assert_eq!(
        SMS_PROMPT.field("channel").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert!(SMS_PROMPT.create_keys().contains(&"channel"));
}

#[test]
fn unknown_netwatch_keys_land_on_status_extras() {
    let mut row = HashMap::new();
    row.insert("host".into(), "1.1.1.1".into());
    row.insert("loss-count".into(), "2".into());
    let extras = extra_status_fields(&NETWATCH_FORM, &row);
    assert_eq!(extras, vec![("loss-count".into(), "2".into())]);
}
