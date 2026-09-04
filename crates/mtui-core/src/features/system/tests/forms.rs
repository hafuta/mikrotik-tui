use crate::features::system::forms::certs::CERT_EXPORT_TYPES;
use crate::features::system::forms::core::{
    DEVICE_MODE_MODES, INACTIVITY_POLICIES, LOGGING_ACTION_TYPES, REMOTE_LOG_FORMATS,
    REMOTE_PROTOCOLS, SYSLOG_FACILITIES, SYSLOG_SEVERITIES, SYSLOG_TIME_FORMATS,
};
use crate::features::system::forms::hardware::{
    BOOT_DEVICE, BOOT_OS, BOOT_PROTOCOL, CPU_FREQUENCY, DISK_TYPES, FORMAT_FILE_SYSTEMS,
    LED_ALL_OFF, LED_TYPES, MEMORY_FREQUENCY, PORT_BAUD, PORT_DATA_BITS, PORT_FLOW, PORT_PARITY,
    PORT_STOP_BITS, PROTECTED_ROUTERBOOT, RAID_CHUNK_SIZES, RAID_TYPES,
};
use crate::features::system::forms::*;
use crate::features::system::guides::GUIDES;
use crate::features::system::resources::RESOURCES;
use crate::features::system::rules::form_field_state;
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const ENTITY_FORMS: &[&FormSchema] = &[
    &USER_FORM,
    &USER_GROUP_FORM,
    &NTP_CLIENT_FORM,
    &NTP_SERVER_FORM,
    &NTP_KEY_FORM,
    &CLOCK_FORM,
    &IDENTITY_FORM,
    &SCHEDULER_FORM,
    &SCRIPT_FORM,
    &LOGGING_FORM,
    &LOGGING_ACTION_FORM,
    &SNMP_FORM,
    &SNMP_COMMUNITY_FORM,
    &CERTIFICATE_FORM,
    &WATCHDOG_FORM,
    &NOTE_FORM,
    &LICENSE_FORM,
    &DISK_FORM,
    &DEVICE_MODE_FORM,
    &PACKAGE_FORM,
    &PACKAGE_UPDATE_FORM,
    &SSH_KEY_FORM,
    &CONSOLE_FORM,
    &LED_FORM,
    &LED_SETTINGS_FORM,
    &PORT_FORM,
    &SPECIAL_LOGIN_FORM,
    &ROUTERBOARD_SETTINGS_FORM,
    &ROUTERBOARD_MODE_BUTTON_FORM,
    &ROUTERBOARD_RESET_BUTTON_FORM,
];

const PROMPT_FORMS: &[&FormSchema] = &[
    &RESET_CONFIG_PROMPT,
    &CERT_SIGN_PROMPT,
    &CERT_IMPORT_PROMPT,
    &CERT_EXPORT_PROMPT,
    &FORMAT_DISK_PROMPT,
    &INSTALL_PACKAGE_PROMPT,
    &EXPORT_CONFIG_PROMPT,
    &IMPORT_CONFIG_PROMPT,
    &AT_CHAT_PROMPT,
    &LICENSE_IMPORT_PROMPT,
    &USB_POWER_RESET_PROMPT,
];

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
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

fn no_advanced(schema: &FormSchema) -> bool {
    schema
        .sections
        .iter()
        .all(|section| section.id != "advanced")
}

fn visible(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    form_field_state(resource_id, key, values).is_none_or(|(is_visible, _)| is_visible)
}

fn enabled(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    form_field_state(resource_id, key, values)
        .is_none_or(|(is_visible, is_enabled)| is_visible && is_enabled)
}

#[test]
fn catalog_and_guides_cover_the_system_group() {
    assert_eq!(RESOURCES.len(), 38);
    assert_eq!(GUIDES.len(), 38);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
    }
    assert_eq!(
        RESOURCES.iter().map(|spec| spec.id).collect::<Vec<_>>(),
        [
            "users",
            "special-login",
            "routerboard",
            "routerboard-settings",
            "routerboard-mode-button",
            "routerboard-reset-button",
            "ntp",
            "ntp-server",
            "ntp-keys",
            "clock",
            "license",
            "disks",
            "device-mode",
            "user-groups",
            "identity",
            "resources",
            "health",
            "packages",
            "package-update",
            "reset-configuration",
            "reboot",
            "shutdown",
            "ssh-keys",
            "history",
            "scheduler",
            "scripts",
            "logging",
            "logging-actions",
            "system-console",
            "leds",
            "led-settings",
            "ports",
            "snmp",
            "snmp-communities",
            "certificates",
            "watchdog",
            "note",
            "logs",
        ]
    );
}

#[test]
fn create_keys_match_writable_and_omit_status() {
    for form in ENTITY_FORMS {
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
        if let Some(status) = form.sections.iter().find(|section| section.id == "status") {
            assert!(status.read_only, "{}", form.title_key);
            for field in status.fields {
                assert_eq!(field.kind, FieldKind::Readonly, "{}", field.key);
                assert!(!form.writable_keys().contains(&field.key), "{}", field.key);
            }
        }
    }
}

#[test]
fn prompt_only_schemas_keep_create_sections() {
    for form in PROMPT_FORMS {
        assert!(form.sections.is_empty(), "{}", form.title_key);
        assert!(!form.create_sections.is_empty(), "{}", form.title_key);
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
    }
}

#[test]
fn disabled_uses_enabled_inverted_toggle() {
    let mut n = 0;
    for form in ENTITY_FORMS.iter().chain(PROMPT_FORMS.iter()) {
        if let Some(field) = form.field("disabled") {
            assert_eq!(field.label, "Enabled", "{}", form.title_key);
            assert_eq!(field.kind, FieldKind::InvertedToggle, "{}", form.title_key);
            n += 1;
        }
    }
    assert_eq!(n, 8);
}

#[test]
fn logging_and_ntp_rules_match_crate_visibility() {
    let mut values = HashMap::new();
    values.insert("target".into(), "memory".into());
    assert!(visible("logging-actions", "name", &values));
    assert!(visible("logging-actions", "memory-lines", &values));
    assert!(visible("logging-actions", "remember", &values));
    assert!(!visible("logging-actions", "remote", &values));
    assert!(!visible("logging-actions", "disk-file-name", &values));
    assert!(!visible("logging-actions", "email-to", &values));
    assert!(!visible("logging-actions", "script", &values));

    values.insert("target".into(), "disk".into());
    assert!(visible("logging-actions", "disk-file-name", &values));
    assert!(!visible("logging-actions", "memory-lines", &values));

    values.insert("target".into(), "email".into());
    assert!(visible("logging-actions", "email-to", &values));
    assert!(!visible("logging-actions", "remote", &values));

    values.insert("target".into(), "script".into());
    assert!(visible("logging-actions", "script", &values));

    values.insert("target".into(), "echo".into());
    assert!(visible("logging-actions", "remember", &values));
    assert!(!visible("logging-actions", "memory-lines", &values));

    values.insert("target".into(), "remote".into());
    values.insert("remote-log-format".into(), "default".into());
    assert!(visible("logging-actions", "remote", &values));
    assert!(!visible("logging-actions", "syslog-facility", &values));
    assert!(!visible("logging-actions", "check-certificate", &values));
    values.insert("remote-protocol".into(), "tls".into());
    assert!(visible("logging-actions", "check-certificate", &values));
    values.insert("remote-log-format".into(), "syslog".into());
    assert!(visible("logging-actions", "syslog-facility", &values));
    values.insert("remote-log-format".into(), "cef".into());
    assert!(visible("logging-actions", "cef-event-delimiter", &values));
    assert!(!visible("logging-actions", "syslog-facility", &values));

    let mut ntp = HashMap::new();
    ntp.insert("broadcast".into(), "false".into());
    ntp.insert("use-local-clock".into(), "false".into());
    assert!(visible("ntp-server", "enabled", &ntp));
    assert!(visible("ntp-server", "vrf", &ntp));
    assert!(!visible("ntp-server", "broadcast-addresses", &ntp));
    assert!(!visible("ntp-server", "local-clock-stratum", &ntp));
    ntp.insert("broadcast".into(), "true".into());
    assert!(visible("ntp-server", "broadcast-addresses", &ntp));
    ntp.insert("use-local-clock".into(), "yes".into());
    assert!(visible("ntp-server", "local-clock-stratum", &ntp));
    assert!(enabled("ntp-server", "local-clock-stratum", &ntp));
    ntp.insert("broadcast".into(), "false".into());
    assert!(!enabled("ntp-server", "broadcast-addresses", &ntp));

    assert!(form_field_state("users", "disabled", &HashMap::new()).is_none());
}

#[test]
fn user_password_is_secret_and_omitted_when_masked() {
    assert_eq!(
        USER_FORM.writable_keys(),
        [
            "name",
            "group",
            "password",
            "address",
            "inactivity-policy",
            "inactivity-timeout",
            "comment",
            "disabled"
        ]
    );
    assert_eq!(
        USER_FORM.field("password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        USER_FORM.field("address").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        USER_FORM
            .field("inactivity-timeout")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_enum(&USER_FORM, "inactivity-policy", INACTIVITY_POLICIES);
    assert!(!USER_FORM.writable_keys().contains(&"last-logged-in"));
    assert_eq!(create_keys(&USER_FORM), USER_FORM.writable_keys());
    assert_lookup(&USER_FORM, "group", "user-groups", "name");
    assert!(no_advanced(&USER_FORM));

    let mut original = HashMap::new();
    original.insert("name".into(), "admin".into());
    original.insert("group".into(), "full".into());
    original.insert("password".into(), "********".into());
    let mut current = original.clone();
    current.insert("group".into(), "read".into());
    current.insert("password".into(), "********".into());
    let body = patch_body(&USER_FORM, &original, &current, "********");
    assert_eq!(body.get("group").map(String::as_str), Some("read"));
    assert!(!body.contains_key("password"));
    assert!(!body.contains_key("last-logged-in"));
}

#[test]
fn user_group_create_is_name_and_policy() {
    assert_eq!(
        USER_GROUP_FORM.writable_keys(),
        ["name", "policy", "skin", "comment"]
    );
    assert_eq!(
        create_keys(&USER_GROUP_FORM),
        USER_GROUP_FORM.writable_keys()
    );
    assert!(no_advanced(&USER_GROUP_FORM));
}

#[test]
fn ntp_client_is_singleton_with_status() {
    assert!(NTP_CLIENT_FORM.create_sections.is_empty());
    assert_eq!(
        NTP_CLIENT_FORM.writable_keys(),
        ["enabled", "mode", "servers"]
    );
    assert_eq!(
        NTP_CLIENT_FORM.field("servers").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        NTP_CLIENT_FORM.field("mode").map(|field| field.kind),
        Some(crate::form_fields::KIND_NTP_CLIENT_MODE)
    );
    assert!(!NTP_CLIENT_FORM.writable_keys().contains(&"status"));
    assert!(
        NTP_CLIENT_FORM
            .sections
            .iter()
            .find(|section| section.id == "status")
            .is_some_and(|section| section.read_only)
    );
    assert!(no_advanced(&NTP_CLIENT_FORM));
}

#[test]
fn ntp_server_is_singleton_without_advanced() {
    assert!(NTP_SERVER_FORM.create_sections.is_empty());
    assert_eq!(
        NTP_SERVER_FORM.writable_keys(),
        [
            "enabled",
            "broadcast",
            "broadcast-addresses",
            "multicast",
            "manycast",
            "vrf",
            "use-local-clock",
            "local-clock-stratum",
            "auth-key",
        ]
    );
    assert!(no_advanced(&NTP_SERVER_FORM));
    assert_lookup(&NTP_SERVER_FORM, "vrf", "vrf", "name");
    assert_eq!(
        NTP_SERVER_FORM
            .field("broadcast-addresses")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        NTP_SERVER_FORM
            .field("local-clock-stratum")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_label(
        &NTP_SERVER_FORM,
        "broadcast-addresses",
        "Broadcast Addresses",
    );
    assert_label(&NTP_SERVER_FORM, "use-local-clock", "Use Local Clock");
    assert_label(
        &NTP_SERVER_FORM,
        "local-clock-stratum",
        "Local Clock Stratum",
    );
    assert_label(&NTP_SERVER_FORM, "auth-key", "Auth. Key");
    assert_lookup(&NTP_SERVER_FORM, "auth-key", "ntp-keys", "key-id");
    assert_ne!(
        NTP_SERVER_FORM.field("auth-key").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        NTP_KEY_FORM.field("key-val").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(create_keys(&NTP_KEY_FORM), NTP_KEY_FORM.writable_keys());
}

#[test]
fn clock_keeps_timezone_writable() {
    assert!(CLOCK_FORM.create_sections.is_empty());
    assert_eq!(
        CLOCK_FORM.writable_keys(),
        ["time-zone-name", "time-zone-autodetect"]
    );
    assert!(!CLOCK_FORM.writable_keys().contains(&"time"));
    assert!(!CLOCK_FORM.writable_keys().contains(&"date"));
    assert!(!CLOCK_FORM.writable_keys().contains(&"gmt-offset"));
    assert_eq!(
        CLOCK_FORM.field("time-zone-name").map(|field| field.kind),
        Some(crate::form_fields::KIND_TIME_ZONE_NAME)
    );
}

#[test]
fn identity_only_name() {
    assert_eq!(IDENTITY_FORM.writable_keys(), ["name"]);
    assert!(IDENTITY_FORM.create_sections.is_empty());
    assert_eq!(IDENTITY_FORM.known_keys(), ["name"]);
    assert!(no_advanced(&IDENTITY_FORM));
}

#[test]
fn scheduler_create_matches_writable_general() {
    assert_eq!(create_keys(&SCHEDULER_FORM), SCHEDULER_FORM.writable_keys());
    assert_lookup(&SCHEDULER_FORM, "on-event", "scripts", "name");
    assert_eq!(
        SCHEDULER_FORM.field("interval").map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert!(!SCHEDULER_FORM.writable_keys().contains(&"next-run"));
    assert!(!SCHEDULER_FORM.writable_keys().contains(&"run-count"));
    assert!(!SCHEDULER_FORM.writable_keys().contains(&"owner"));
    assert_eq!(
        SCHEDULER_FORM.field("owner").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(no_advanced(&SCHEDULER_FORM));
}

#[test]
fn script_source_is_writable_text_not_secret() {
    assert!(SCRIPT_FORM.writable_keys().contains(&"source"));
    assert_eq!(
        SCRIPT_FORM.field("source").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert_ne!(
        SCRIPT_FORM.field("source").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(create_keys(&SCRIPT_FORM), SCRIPT_FORM.writable_keys());
    assert_eq!(
        SCRIPT_FORM
            .field("dont-require-permissions")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        SCRIPT_FORM.field("owner").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(no_advanced(&SCRIPT_FORM));
}

#[test]
fn logging_create_is_topics_and_action() {
    assert_eq!(create_keys(&LOGGING_FORM), LOGGING_FORM.writable_keys());
    assert_eq!(
        LOGGING_FORM.writable_keys(),
        ["topics", "action", "prefix", "comment", "disabled"]
    );
    assert_lookup(&LOGGING_FORM, "action", "logging-actions", "name");
}

#[test]
fn logging_action_form_covers_remote_syslog() {
    assert_eq!(
        create_keys(&LOGGING_ACTION_FORM),
        LOGGING_ACTION_FORM.writable_keys()
    );
    assert!(no_advanced(&LOGGING_ACTION_FORM));
    assert_lookup(&LOGGING_ACTION_FORM, "script", "scripts", "name");
    assert_label(&LOGGING_ACTION_FORM, "target", "Type");
    assert_label(&LOGGING_ACTION_FORM, "syslog-facility", "Syslog Facility");
    assert_label(&LOGGING_ACTION_FORM, "syslog-severity", "Syslog Severity");
    assert_enum(&LOGGING_ACTION_FORM, "target", LOGGING_ACTION_TYPES);
    assert_enum(&LOGGING_ACTION_FORM, "syslog-facility", SYSLOG_FACILITIES);
    assert_enum(&LOGGING_ACTION_FORM, "syslog-severity", SYSLOG_SEVERITIES);
    assert_enum(
        &LOGGING_ACTION_FORM,
        "syslog-time-format",
        SYSLOG_TIME_FORMATS,
    );
    assert_enum(&LOGGING_ACTION_FORM, "remote-protocol", REMOTE_PROTOCOLS);
    assert_eq!(REMOTE_PROTOCOLS, ["udp", "tcp", "tls"]);
    assert_eq!(REMOTE_PROTOCOLS[0], "udp");
    assert_eq!(SYSLOG_FACILITIES[0], "daemon");
    assert_eq!(SYSLOG_SEVERITIES[0], "auto");
    assert_enum(
        &LOGGING_ACTION_FORM,
        "remote-log-format",
        REMOTE_LOG_FORMATS,
    );
    assert_lookup(&LOGGING_ACTION_FORM, "vrf", "vrf", "name");
    assert_eq!(
        LOGGING_ACTION_FORM
            .field("memory-lines")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        LOGGING_ACTION_FORM
            .field("remote-port")
            .map(|field| field.kind),
        Some(FieldKind::Number)
    );
}

#[test]
fn snmp_singleton_and_community_secrets() {
    assert!(SNMP_FORM.create_sections.is_empty());
    assert_eq!(
        SNMP_FORM.writable_keys(),
        ["enabled", "contact", "location", "engine-id"]
    );
    assert_eq!(
        create_keys(&SNMP_COMMUNITY_FORM),
        SNMP_COMMUNITY_FORM.writable_keys()
    );
    assert_eq!(
        SNMP_COMMUNITY_FORM
            .field("authentication-password")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        SNMP_COMMUNITY_FORM
            .field("encryption-password")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        SNMP_COMMUNITY_FORM
            .field("addresses")
            .map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_eq!(
        SNMP_COMMUNITY_FORM
            .field("security")
            .map(|field| field.kind),
        Some(crate::form_fields::KIND_SNMP_SECURITY)
    );

    let mut original = HashMap::new();
    original.insert("name".into(), "public".into());
    original.insert("authentication-password".into(), "********".into());
    original.insert("encryption-password".into(), "********".into());
    let mut current = original.clone();
    current.insert("addresses".into(), "0.0.0.0/0".into());
    current.insert("authentication-password".into(), "********".into());
    current.insert("encryption-password".into(), "********".into());
    let body = patch_body(&SNMP_COMMUNITY_FORM, &original, &current, "********");
    assert_eq!(body.get("addresses").map(String::as_str), Some("0.0.0.0/0"));
    assert!(!body.contains_key("authentication-password"));
    assert!(!body.contains_key("encryption-password"));
}

#[test]
fn certificate_has_no_writable_text_private_key() {
    assert_eq!(
        create_keys(&CERTIFICATE_FORM),
        CERTIFICATE_FORM.writable_keys()
    );
    assert_eq!(
        CERTIFICATE_FORM.writable_keys(),
        ["name", "common-name", "key-usage", "trusted", "days-valid"]
    );
    match CERTIFICATE_FORM.field("private-key") {
        None => {}
        Some(field) => assert_eq!(field.kind, FieldKind::Secret),
    }
    assert!(!CERTIFICATE_FORM.writable_keys().contains(&"fingerprint"));
    assert!(!CERTIFICATE_FORM.writable_keys().contains(&"serial-number"));
}

#[test]
fn certificate_prompts_cover_sign_import_export() {
    assert_eq!(CERT_SIGN_PROMPT.writable_keys(), ["ca"]);
    assert_lookup(&CERT_SIGN_PROMPT, "ca", "certificates", "name");
    assert_eq!(
        CERT_IMPORT_PROMPT.writable_keys(),
        ["file-name", "passphrase", "name"]
    );
    assert_eq!(
        CERT_IMPORT_PROMPT
            .field("passphrase")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        CERT_EXPORT_PROMPT.writable_keys(),
        ["file-name", "type", "export-passphrase"]
    );
    assert_eq!(
        CERT_EXPORT_PROMPT
            .field("export-passphrase")
            .map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_lookup(&CERT_IMPORT_PROMPT, "file-name", "files", "name");
    assert_lookup(&CERT_EXPORT_PROMPT, "file-name", "files", "name");
    assert_enum(&CERT_EXPORT_PROMPT, "type", CERT_EXPORT_TYPES);

    let mut original = HashMap::new();
    original.insert("file-name".into(), "web.p12".into());
    original.insert("passphrase".into(), "********".into());
    original.insert("export-passphrase".into(), "********".into());
    let mut current = original.clone();
    current.insert("file-name".into(), "web.pem".into());
    current.insert("passphrase".into(), "********".into());
    current.insert("export-passphrase".into(), "********".into());
    let import_body = patch_body(&CERT_IMPORT_PROMPT, &original, &current, "********");
    assert_eq!(
        import_body.get("file-name").map(String::as_str),
        Some("web.pem")
    );
    assert!(!import_body.contains_key("passphrase"));
    let export_body = patch_body(&CERT_EXPORT_PROMPT, &original, &current, "********");
    assert!(!export_body.contains_key("export-passphrase"));
    assert_eq!(
        export_body.get("file-name").map(String::as_str),
        Some("web.pem")
    );
}

#[test]
fn watchdog_and_note_are_singletons() {
    assert!(WATCHDOG_FORM.create_sections.is_empty());
    assert!(NOTE_FORM.create_sections.is_empty());
    assert!(WATCHDOG_FORM.writable_keys().contains(&"watch-address"));
    assert!(WATCHDOG_FORM.writable_keys().contains(&"automatic-supout"));
    assert_eq!(
        WATCHDOG_FORM
            .field("watchdog-timer")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        WATCHDOG_FORM
            .field("watch-interval")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_eq!(
        WATCHDOG_FORM.field("no-ping-delay").map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_eq!(
        WATCHDOG_FORM.field("send-email-to").map(|field| field.kind),
        Some(FieldKind::Text)
    );
    assert!(WATCHDOG_FORM.create_sections.is_empty());
    assert!(no_advanced(&WATCHDOG_FORM));
    assert_eq!(NOTE_FORM.writable_keys(), ["show-at-login", "note"]);
}

#[test]
fn package_name_is_readonly_disabled_toggle() {
    assert!(PACKAGE_FORM.create_sections.is_empty());
    assert_eq!(PACKAGE_FORM.writable_keys(), ["disabled"]);
    assert_eq!(
        PACKAGE_FORM.field("name").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(!PACKAGE_FORM.writable_keys().contains(&"version"));
    assert!(!PACKAGE_FORM.writable_keys().contains(&"build-time"));
    assert!(!PACKAGE_FORM.writable_keys().contains(&"scheduled"));
}

#[test]
fn package_update_channel_and_file_prompts() {
    assert_eq!(
        PACKAGE_UPDATE_FORM.field("channel").map(|field| field.kind),
        Some(crate::form_fields::KIND_PACKAGE_CHANNEL)
    );
    assert_lookup(&INSTALL_PACKAGE_PROMPT, "file-name", "files", "name");
    assert_lookup(&IMPORT_CONFIG_PROMPT, "file-name", "files", "name");
    assert_eq!(
        EXPORT_CONFIG_PROMPT.field("file").map(|field| field.kind),
        Some(FieldKind::Text)
    );
}

#[test]
fn license_is_inspect_only_and_key_is_secret() {
    assert!(LICENSE_FORM.create_sections.is_empty());
    assert!(LICENSE_FORM.writable_keys().is_empty());
    assert!(
        LICENSE_FORM
            .sections
            .iter()
            .all(|section| section.id == "status" && section.read_only)
    );
    assert_label(&LICENSE_FORM, "software-id", "Software ID");
    assert_label(&LICENSE_FORM, "nlevel", "Level");
    assert_label(&LICENSE_FORM, "system-id", "System ID");
    assert_eq!(
        LICENSE_FORM.field("nlevel").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_eq!(
        LICENSE_IMPORT_PROMPT.field("k").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_label(&LICENSE_IMPORT_PROMPT, "k", "License Key");
    assert_lookup(&LICENSE_IMPORT_PROMPT, "file-name", "files", "name");
    let mut original = HashMap::new();
    original.insert("k".into(), "********".into());
    original.insert("file-name".into(), String::new());
    let mut current = original.clone();
    current.insert("k".into(), "********".into());
    current.insert("file-name".into(), "chr.key".into());
    let body = patch_body(&LICENSE_IMPORT_PROMPT, &original, &current, "********");
    assert_eq!(body.get("file-name").map(String::as_str), Some("chr.key"));
    assert!(!body.contains_key("k"));
}

#[test]
fn disk_form_uses_webfig_field_kinds() {
    assert_eq!(create_keys(&DISK_FORM), DISK_FORM.writable_keys());
    assert_enum(&DISK_FORM, "type", DISK_TYPES);
    assert_enum(&DISK_FORM, "raid-type", RAID_TYPES);
    assert_enum(&DISK_FORM, "raid-chunk-size", RAID_CHUNK_SIZES);
    assert_lookup(&DISK_FORM, "parent", "disks", "slot");
    assert_lookup(&DISK_FORM, "raid-master", "disks", "slot");
    assert_lookup(&DISK_FORM, "crypted-backend", "disks", "slot");
    assert_lookup(&DISK_FORM, "file-path", "files", "name");
    assert_lookup(&DISK_FORM, "media-interface", "interfaces", "name");
    assert_lookup(&DISK_FORM, "smb-server-user", "users", "name");
    assert_eq!(
        DISK_FORM.field("raid-role").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        DISK_FORM.field("sshfs-port").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        DISK_FORM.field("mount-filesystem").map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        DISK_FORM.field("encryption-key").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        DISK_FORM.field("sshfs-password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
    assert_eq!(
        DISK_FORM.field("model").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(!DISK_FORM.writable_keys().contains(&"size"));
    assert!(!DISK_FORM.writable_keys().contains(&"fs"));
    assert_label(&DISK_FORM, "raid-master", "RAID Master");
    assert_enum(&FORMAT_DISK_PROMPT, "file-system", FORMAT_FILE_SYSTEMS);
    assert_eq!(FORMAT_FILE_SYSTEMS[0], "ext4");
    assert_eq!(
        FORMAT_DISK_PROMPT
            .field("mbr-partition-table")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_label(&FORMAT_DISK_PROMPT, "file-system", "File System");
}

#[test]
fn device_mode_flags_are_toggles_not_text() {
    assert!(DEVICE_MODE_FORM.create_sections.is_empty());
    assert_enum(&DEVICE_MODE_FORM, "mode", DEVICE_MODE_MODES);
    assert_eq!(DEVICE_MODE_MODES, ["advanced", "home", "basic", "rose"]);
    for key in [
        "container",
        "scheduler",
        "traffic-gen",
        "fetch",
        "flagged",
        "flagging-enabled",
    ] {
        assert_eq!(
            DEVICE_MODE_FORM.field(key).map(|field| field.kind),
            Some(FieldKind::Toggle),
            "{key} must be a toggle"
        );
    }
    assert_label(&DEVICE_MODE_FORM, "traffic-gen", "Traffic Generator");
    assert_label(&DEVICE_MODE_FORM, "bandwidth-test", "Bandwidth Test");
    assert_eq!(
        DEVICE_MODE_FORM
            .field("allowed-versions")
            .map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_eq!(
        DEVICE_MODE_FORM
            .field("attempt-count")
            .map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert!(!DEVICE_MODE_FORM.writable_keys().contains(&"attempt-count"));
    assert!(no_advanced(&DEVICE_MODE_FORM));
}

#[test]
fn console_port_is_lookup_and_channel_is_number() {
    assert_lookup(&CONSOLE_FORM, "port", "ports", "name");
    assert_eq!(
        CONSOLE_FORM.field("channel").map(|field| field.kind),
        Some(FieldKind::Number)
    );
    assert_eq!(
        CONSOLE_FORM.field("disabled").map(|field| field.kind),
        Some(FieldKind::InvertedToggle)
    );
    assert!(!CONSOLE_FORM.writable_keys().contains(&"used"));
    assert_eq!(create_keys(&CONSOLE_FORM), CONSOLE_FORM.writable_keys());
    assert!(no_advanced(&CONSOLE_FORM));
}

#[test]
fn led_type_is_enum_and_lookups_are_not_text() {
    assert_enum(&LED_FORM, "type", LED_TYPES);
    assert_lookup(&LED_FORM, "interface", "interfaces", "name");
    assert_lookup(&LED_FORM, "modem", "interfaces", "name");
    assert_eq!(
        LED_FORM.field("leds").map(|field| field.kind),
        Some(FieldKind::Repeat)
    );
    assert_enum(&LED_SETTINGS_FORM, "all-leds-off", LED_ALL_OFF);
    assert!(LED_SETTINGS_FORM.create_sections.is_empty());
    assert!(no_advanced(&LED_FORM));
    assert!(no_advanced(&LED_SETTINGS_FORM));
}

#[test]
fn port_serial_fields_are_enums() {
    assert_eq!(
        PORT_FORM.field("name").map(|field| field.kind),
        Some(FieldKind::Readonly)
    );
    assert_enum(&PORT_FORM, "baud-rate", PORT_BAUD);
    assert_enum(&PORT_FORM, "data-bits", PORT_DATA_BITS);
    assert_enum(&PORT_FORM, "parity", PORT_PARITY);
    assert_enum(&PORT_FORM, "stop-bits", PORT_STOP_BITS);
    assert_enum(&PORT_FORM, "flow-control", PORT_FLOW);
    assert!(PORT_FORM.create_sections.is_empty());
    assert!(no_advanced(&PORT_FORM));
}

#[test]
fn special_login_user_and_port_are_lookups() {
    assert_lookup(&SPECIAL_LOGIN_FORM, "user", "users", "name");
    assert_lookup(&SPECIAL_LOGIN_FORM, "port", "ports", "name");
    assert_eq!(
        create_keys(&SPECIAL_LOGIN_FORM),
        SPECIAL_LOGIN_FORM.writable_keys()
    );
    assert!(no_advanced(&SPECIAL_LOGIN_FORM));
}

#[test]
fn reset_configuration_prompt_covers_webfig_flags() {
    assert_eq!(
        RESET_CONFIG_PROMPT.writable_keys(),
        [
            "keep-users",
            "no-defaults",
            "skip-backup",
            "caps-mode",
            "run-after-reset"
        ]
    );
    assert_eq!(
        RESET_CONFIG_PROMPT
            .field("keep-users")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        RESET_CONFIG_PROMPT
            .field("caps-mode")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_lookup(&RESET_CONFIG_PROMPT, "run-after-reset", "files", "name");
    assert_eq!(
        create_keys(&RESET_CONFIG_PROMPT),
        RESET_CONFIG_PROMPT.writable_keys()
    );
    assert!(RESET_CONFIG_PROMPT.sections.is_empty());
}

#[test]
fn routerboard_settings_and_buttons_use_field_kinds() {
    assert_enum(&ROUTERBOARD_SETTINGS_FORM, "boot-os", BOOT_OS);
    assert_eq!(BOOT_OS, ["router-os", "swos"]);
    assert_enum(&ROUTERBOARD_SETTINGS_FORM, "boot-device", BOOT_DEVICE);
    assert_enum(&ROUTERBOARD_SETTINGS_FORM, "boot-protocol", BOOT_PROTOCOL);
    assert_eq!(BOOT_PROTOCOL, ["bootp", "dhcp"]);
    assert_enum(&ROUTERBOARD_SETTINGS_FORM, "cpu-frequency", CPU_FREQUENCY);
    assert_enum(
        &ROUTERBOARD_SETTINGS_FORM,
        "memory-frequency",
        MEMORY_FREQUENCY,
    );
    assert_enum(
        &ROUTERBOARD_SETTINGS_FORM,
        "protected-routerboot",
        PROTECTED_ROUTERBOOT,
    );
    assert_eq!(
        ROUTERBOARD_SETTINGS_FORM
            .field("enable-jumper-reset")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_label(
        &ROUTERBOARD_SETTINGS_FORM,
        "enable-jumper-reset",
        "Enable Jumper Reset",
    );
    assert!(
        ROUTERBOARD_SETTINGS_FORM
            .field("reformat-hold-button")
            .is_none()
    );
    assert!(
        ROUTERBOARD_SETTINGS_FORM
            .field("reformat-hold-button-max")
            .is_none()
    );
    assert_eq!(
        ROUTERBOARD_SETTINGS_FORM
            .field("silent-boot")
            .map(|field| field.kind),
        Some(FieldKind::Toggle)
    );
    assert_eq!(
        ROUTERBOARD_MODE_BUTTON_FORM
            .field("hold-time")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert_lookup(&ROUTERBOARD_MODE_BUTTON_FORM, "on-event", "scripts", "name");
    assert_lookup(
        &ROUTERBOARD_RESET_BUTTON_FORM,
        "on-event",
        "scripts",
        "name",
    );
    assert_eq!(
        USB_POWER_RESET_PROMPT
            .field("duration")
            .map(|field| field.kind),
        Some(FieldKind::Time)
    );
    assert!(ROUTERBOARD_SETTINGS_FORM.create_sections.is_empty());
    assert!(no_advanced(&ROUTERBOARD_SETTINGS_FORM));
    assert!(no_advanced(&ROUTERBOARD_MODE_BUTTON_FORM));
    assert!(no_advanced(&ROUTERBOARD_RESET_BUTTON_FORM));
}

#[test]
fn routerboard_settings_hides_board_only_keys_until_printed() {
    let crs = HashMap::from([
        (
            "boot-device".to_string(),
            "nand-if-fail-then-ethernet".into(),
        ),
        ("boot-os".to_string(), "router-os".into()),
        ("cpu-frequency".to_string(), "716MHz".into()),
        ("enable-jumper-reset".to_string(), "true".into()),
    ]);
    assert_eq!(
        form_field_state("routerboard-settings", "boot-os", &crs),
        Some((true, true))
    );
    assert_eq!(
        form_field_state("routerboard-settings", "cpu-frequency", &crs),
        Some((true, true))
    );
    assert_eq!(
        form_field_state("routerboard-settings", "enable-jumper-reset", &crs),
        Some((true, true))
    );
    assert_eq!(
        form_field_state("routerboard-settings", "memory-frequency", &crs),
        Some((false, false))
    );
    let hap = HashMap::from([
        (
            "boot-device".to_string(),
            "nand-if-fail-then-ethernet".into(),
        ),
        ("auto-upgrade".to_string(), "false".into()),
    ]);
    assert_eq!(
        form_field_state("routerboard-settings", "boot-os", &hap),
        Some((false, false))
    );
    assert_eq!(
        form_field_state("routerboard-settings", "cpu-frequency", &hap),
        Some((false, false))
    );
    let extras = extra_status_fields(&ROUTERBOARD_SETTINGS_FORM, &crs);
    assert!(
        extras
            .iter()
            .all(|(key, _)| key != "enable-jumper-reset" && key != "cpu-frequency")
    );
}
