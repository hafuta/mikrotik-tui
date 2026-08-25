//! Sectioned property-sheet schemas for resource editors.

use std::collections::BTreeMap;
use std::collections::HashMap;

/// One field in a properties sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    Toggle,
    Enum {
        values: &'static [&'static str],
    },
    Readonly,
    Secret,
    Lookup {
        resource_id: &'static str,
        value_key: &'static str,
        multiple: bool,
    },
    /// Comma-separated API list edited as add/remove rows.
    Repeat,
}

impl FieldKind {
    #[must_use]
    pub fn writable(self) -> bool {
        !matches!(self, Self::Readonly)
    }

    /// Short kind tag shown beside the field label (`text`, `select`, …).
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "num",
            Self::Toggle => "toggle",
            Self::Enum { .. } => "select",
            Self::Readonly => "read",
            Self::Secret => "secret",
            Self::Lookup { .. } => "lookup",
            Self::Repeat => "list",
        }
    }

    /// Footer hint for the focused control.
    #[must_use]
    pub fn edit_hint(self) -> &'static str {
        match self {
            Self::Text | Self::Number | Self::Secret | Self::Repeat => "type value",
            Self::Toggle => "space toggle",
            Self::Enum { .. } | Self::Lookup { .. } => "space pick",
            Self::Readonly => "read only",
        }
    }

    /// Whether printable keys, including digits, should go into this field.
    /// Lookup typing happens only inside the picker filter, not on the sheet.
    #[must_use]
    pub fn takes_typed_input(self) -> bool {
        matches!(
            self,
            Self::Text | Self::Number | Self::Secret | Self::Repeat
        )
    }

    /// Whether `ch` may be appended to `current` for this control.
    ///
    /// Number fields take ASCII digits only. TCP/UDP port keys (`port`,
    /// `*-port`) stop at five digits. Extra or non-digit keys are ignored,
    /// not reported as an error.
    #[must_use]
    pub fn accepts_char(self, key: &str, current: &str, ch: char) -> bool {
        match self {
            Self::Number => accepts_number_char(key, current, ch),
            Self::Text | Self::Secret | Self::Repeat => true,
            _ => false,
        }
    }
}

/// TCP/UDP port fields (1-65535) are five digits at most.
pub const TCP_UDP_PORT_DIGIT_CAP: usize = 5;

/// `port` and `*-port`, but not list keys such as `*-ports`.
#[must_use]
pub fn is_tcp_udp_port_key(key: &str) -> bool {
    let key = key.trim();
    key == "port" || (key.ends_with("-port") && !key.ends_with("-ports"))
}

/// Digit-only typing for `FieldKind::Number` (and Torch port).
#[must_use]
pub fn accepts_number_char(key: &str, current: &str, ch: char) -> bool {
    if !ch.is_ascii_digit() {
        return false;
    }
    if is_tcp_udp_port_key(key)
        && current.chars().filter(char::is_ascii_digit).count() >= TCP_UDP_PORT_DIGIT_CAP
    {
        return false;
    }
    true
}

/// Split a comma-separated API list, skipping blanks and keeping first-seen order.
#[must_use]
pub fn split_ros_list(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|item| item == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

/// Join list items for a PATCH body. Empty entries are dropped.
#[must_use]
pub fn join_ros_list(values: &[String]) -> String {
    values
        .iter()
        .map(String::as_str)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

/// Default typed into an empty writable control (enum first option, NTP key `none`).
#[must_use]
pub fn default_writable_value(kind: FieldKind) -> String {
    match kind {
        FieldKind::Enum { values } => values.first().copied().unwrap_or("").to_string(),
        FieldKind::Lookup {
            resource_id: "ntp-keys",
            ..
        } => "none".to_string(),
        _ => String::new(),
    }
}

/// Put `none` first when the API combo includes an unspecified choice.
#[must_use]
pub fn with_leading_none(options: Vec<String>) -> Vec<String> {
    prepend_unique("none", options)
}

/// Put `all` first when the combo includes the `RouterOS` wildcard interface or queue.
#[must_use]
pub fn with_leading_all(options: Vec<String>) -> Vec<String> {
    prepend_unique("all", options)
}

fn prepend_unique(lead: &str, options: Vec<String>) -> Vec<String> {
    if options.iter().any(|item| item == lead) {
        return options;
    }
    let mut out = Vec::with_capacity(options.len().saturating_add(1));
    out.push(lead.to_string());
    out.extend(options);
    out
}

/// Extra combo values that are not rows of the lookup resource (`none`, `all`).
#[must_use]
pub fn prepare_lookup_options(
    sheet_resource_id: &str,
    lookup_resource_id: &str,
    options: Vec<String>,
) -> Vec<String> {
    if lookup_resource_id == "ntp-keys" {
        return with_leading_none(options);
    }
    if matches!(sheet_resource_id, "romon-ports" | "graphing-interface")
        && lookup_resource_id == "interfaces"
    {
        return with_leading_all(options);
    }
    if sheet_resource_id == "graphing-queue" && lookup_resource_id == "queue-simple" {
        return with_leading_all(options);
    }
    options
}

/// Whether a sheet field should appear given current values.
///
/// Logging Actions show only the knobs that belong to Type (`target`) and, for
/// remote, to Remote Log Format and Remote Protocol. NTP Server shows Broadcast
/// Addresses only when Broadcast is on, and Local Clock Stratum only when Use
/// Local Clock is on. Traffic Flow shows Sampling Interval and Sampling Space
/// only when Packet Sampling is on. Traffic Flow Targets show v9 template
/// fields only for version `9` or `ipfix`. IGMP Proxy Interfaces show
/// Alternative Subnets only when Upstream is on. Inapplicable rows are omitted,
/// not locked. Check Certificate appears only when protocol is `tls`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn field_visible(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    match resource_id {
        "logging-actions" => logging_action_field_visible(key, values),
        "ntp-server" => ntp_server_field_visible(key, values),
        "traffic-flow" => traffic_flow_field_visible(key, values),
        "traffic-flow-targets" => traffic_flow_target_field_visible(key, values),
        "igmp-proxy-interfaces" => igmp_proxy_interface_field_visible(key, values),
        "lte-apn" => lte_apn_field_visible(key, values),
        _ => true,
    }
}

fn flag_on(values: &HashMap<String, String>, key: &str) -> bool {
    crate::actions::truthy(values.get(key).map(String::as_str))
}

fn logging_action_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    let target = values.get("target").map_or("", String::as_str);
    let format = values
        .get("remote-log-format")
        .map_or("default", String::as_str);
    let format = if format.is_empty() { "default" } else { format };
    let protocol = values.get("remote-protocol").map_or("udp", String::as_str);
    let protocol = if protocol.is_empty() { "udp" } else { protocol };
    match key {
        "memory-lines" | "memory-stop-on-full" => target == "memory",
        "disk-file-name" | "disk-lines-per-file" | "disk-file-count" | "disk-stop-on-full" => {
            target == "disk"
        }
        "remote" | "remote-port" | "src-address" | "remote-protocol" | "remote-log-format"
        | "vrf" | "add-topics-string" => target == "remote",
        "check-certificate" => target == "remote" && protocol == "tls",
        "syslog-facility" | "syslog-severity" => target == "remote" && format == "syslog",
        "syslog-time-format" => target == "remote" && matches!(format, "syslog" | "cef"),
        "cef-event-delimiter" => target == "remote" && format == "cef",
        "email-to" | "email-cc" | "email-start-tls" => target == "email",
        "script" => target == "script",
        "remember" => matches!(target, "memory" | "echo"),
        _ => true,
    }
}

fn ntp_server_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    match key {
        "broadcast-addresses" => flag_on(values, "broadcast"),
        "local-clock-stratum" => flag_on(values, "use-local-clock"),
        _ => true,
    }
}

fn traffic_flow_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    match key {
        "sampling-interval" | "sampling-space" => flag_on(values, "packet-sampling"),
        _ => true,
    }
}

fn traffic_flow_version_uses_templates(values: &HashMap<String, String>) -> bool {
    matches!(
        values
            .get("version")
            .map_or("", String::as_str)
            .to_ascii_lowercase()
            .as_str(),
        "9" | "ipfix"
    )
}

fn traffic_flow_target_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    match key {
        "v9-template-refresh" | "v9-template-timeout" => {
            traffic_flow_version_uses_templates(values)
        }
        _ => true,
    }
}

fn igmp_proxy_interface_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    match key {
        "alternative-subnets" => flag_on(values, "upstream"),
        _ => true,
    }
}

fn lte_apn_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    match key {
        "user" | "password" => {
            matches!(
                values.get("authentication").map(String::as_str),
                Some("pap") | Some("chap")
            )
        }
        "passthrough-mac" | "passthrough-subnet-selection" => values
            .get("passthrough-interface")
            .is_some_and(|value| !value.trim().is_empty()),
        _ => true,
    }
}

/// Whether a visible sheet field can be edited.
///
/// Association is handled by [`field_visible`]. Locked (read-only styling)
/// is not a stand-in for hiding fields that do not belong to the selection.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn field_enabled(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    field_visible(resource_id, key, values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormSection {
    pub id: &'static str,
    pub label: &'static str,
    pub fields: &'static [FieldSpec],
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormSchema {
    pub title_key: &'static str,
    pub subtitle_keys: &'static [&'static str],
    pub sections: &'static [FormSection],
    pub create_sections: &'static [FormSection],
}

impl FormSchema {
    #[must_use]
    pub fn sections_for(&self, create: bool) -> &'static [FormSection] {
        if create && !self.create_sections.is_empty() {
            self.create_sections
        } else {
            self.sections
        }
    }

    #[must_use]
    pub fn field(&self, key: &str) -> Option<&'static FieldSpec> {
        self.sections
            .iter()
            .chain(self.create_sections.iter())
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
    }

    #[must_use]
    pub fn writable_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for section in self.sections.iter().chain(self.create_sections.iter()) {
            if section.read_only {
                continue;
            }
            for field in section.fields {
                if field.kind.writable() && !keys.contains(&field.key) {
                    keys.push(field.key);
                }
            }
        }
        keys
    }

    #[must_use]
    pub fn known_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for section in self.sections.iter().chain(self.create_sections.iter()) {
            for field in section.fields {
                if !keys.contains(&field.key) {
                    keys.push(field.key);
                }
            }
        }
        keys
    }
}

/// Keys present on the live row but not in the schema (shown as Status extras).
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extra_status_fields(
    schema: &FormSchema,
    row: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let known = schema.known_keys();
    let mut extras: Vec<(String, String)> = row
        .iter()
        .filter(|(key, _)| key.as_str() != ".id" && !known.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    extras.sort_by(|left, right| left.0.cmp(&right.0));
    extras
}

/// PATCH/PUT body: only writable keys that actually changed.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn patch_body(
    schema: &FormSchema,
    original: &HashMap<String, String>,
    current: &HashMap<String, String>,
    masked_token: &str,
) -> BTreeMap<String, String> {
    let original_map: BTreeMap<_, _> = original
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let current_map: BTreeMap<_, _> = current
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    crate_changed(
        &original_map,
        &current_map,
        &schema.writable_keys(),
        masked_token,
    )
}

fn crate_changed(
    original: &BTreeMap<String, String>,
    current: &BTreeMap<String, String>,
    writable: &[&str],
    masked_token: &str,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for key in writable {
        let Some(now) = current.get(*key) else {
            continue;
        };
        if now == masked_token {
            continue;
        }
        match original.get(*key) {
            Some(was) if was == now => {}
            _ => {
                out.insert((*key).to_string(), now.clone());
            }
        }
    }
    out
}

/// Changed writable fields as `(label, display value)` for a save preview.
///
/// Secret fields never show the typed value — only `masked_token`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn preview_changes(
    resource_id: &str,
    schema: &FormSchema,
    original: &HashMap<String, String>,
    current: &HashMap<String, String>,
    masked_token: &str,
) -> Vec<(String, String)> {
    let mut body = patch_body(schema, original, current, masked_token);
    body.retain(|key, _| field_enabled(resource_id, key, current));
    body.into_iter()
        .map(|(key, value)| {
            let spec = schema.field(&key);
            let label = spec.map_or_else(|| key.clone(), |field| field.label.to_string());
            let secret = spec.is_some_and(|field| matches!(field.kind, FieldKind::Secret))
                || looks_secret_key(&key);
            let shown = if secret {
                masked_token.to_string()
            } else {
                value
            };
            (label, shown)
        })
        .collect()
}

fn looks_secret_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase().replace('_', "-");
    normalized.contains("password")
        || normalized.contains("passphrase")
        || normalized.contains("private-key")
        || normalized.contains("pre-shared")
        || normalized.ends_with("-secret")
        || normalized == "secret"
        || normalized == "secrets"
        || normalized == "key-val"
}

pub const ARP_VALUES: &[&str] = &[
    "enabled",
    "disabled",
    "proxy-arp",
    "reply-only",
    "local-proxy-arp",
];

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: FormSchema = FormSchema {
        title_key: "name",
        subtitle_keys: &["type"],
        sections: &[
            FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "name",
                        label: "Name",
                        kind: FieldKind::Text,
                    },
                    FieldSpec {
                        key: "comment",
                        label: "Comment",
                        kind: FieldKind::Text,
                    },
                ],
            },
            FormSection {
                id: "status",
                label: "Status",
                read_only: true,
                fields: &[FieldSpec {
                    key: "running",
                    label: "Running",
                    kind: FieldKind::Readonly,
                }],
            },
        ],
        create_sections: &[],
    };

    #[test]
    fn patch_body_skips_readonly_and_unchanged() {
        let mut original = HashMap::new();
        original.insert("name".into(), "vlan10".into());
        original.insert("running".into(), "true".into());
        let mut current = original.clone();
        current.insert("comment".into(), "office".into());
        current.insert("running".into(), "false".into());
        let body = patch_body(&SAMPLE, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("office"));
        assert!(!body.contains_key("running"));
        assert!(!body.contains_key("name"));
    }

    #[test]
    fn preview_changes_lists_only_dirty_writable_fields() {
        let mut original = HashMap::new();
        original.insert("name".into(), "vlan10".into());
        let mut current = original.clone();
        current.insert("comment".into(), "office".into());
        current.insert("running".into(), "false".into());
        let lines = preview_changes("interfaces", &SAMPLE, &original, &current, "********");
        assert_eq!(lines, vec![("Comment".into(), "office".into())]);
    }

    #[test]
    fn field_kind_names_the_control() {
        assert_eq!(FieldKind::Text.tag(), "text");
        assert_eq!(FieldKind::Number.tag(), "num");
        assert_eq!(FieldKind::Toggle.tag(), "toggle");
        assert_eq!(
            FieldKind::Enum {
                values: &["a", "b"]
            }
            .tag(),
            "select"
        );
        assert_eq!(FieldKind::Readonly.tag(), "read");
        assert_eq!(FieldKind::Secret.tag(), "secret");
        assert_eq!(
            FieldKind::Lookup {
                resource_id: "interfaces",
                value_key: "name",
                multiple: false,
            }
            .tag(),
            "lookup"
        );
        assert_eq!(FieldKind::Repeat.tag(), "list");
        assert_eq!(FieldKind::Text.edit_hint(), "type value");
        assert_eq!(FieldKind::Toggle.edit_hint(), "space toggle");
        assert_eq!(FieldKind::Enum { values: &["a"] }.edit_hint(), "space pick");
        assert_eq!(
            FieldKind::Lookup {
                resource_id: "interfaces",
                value_key: "name",
                multiple: true,
            }
            .edit_hint(),
            "space pick"
        );
        assert!(FieldKind::Number.takes_typed_input());
        assert!(
            !FieldKind::Lookup {
                resource_id: "interfaces",
                value_key: "name",
                multiple: false,
            }
            .takes_typed_input()
        );
        assert!(
            FieldKind::Lookup {
                resource_id: "interfaces",
                value_key: "name",
                multiple: false,
            }
            .writable()
        );
        assert!(!FieldKind::Toggle.takes_typed_input());
        assert!(FieldKind::Repeat.takes_typed_input());
    }

    #[test]
    fn ros_list_split_skips_blanks_and_join_drops_empty() {
        assert_eq!(
            split_ros_list(" 10.0.0.255, ,10.0.1.255,10.0.0.255 "),
            vec!["10.0.0.255".to_string(), "10.0.1.255".to_string()]
        );
        assert_eq!(
            join_ros_list(&["10.0.0.255".into(), String::new(), "10.0.1.255".into()]),
            "10.0.0.255,10.0.1.255"
        );
        assert_eq!(
            with_leading_none(vec!["1".into(), "2".into()]),
            vec!["none".to_string(), "1".into(), "2".into()]
        );
        assert_eq!(
            with_leading_all(vec!["ether1".into()]),
            vec!["all".to_string(), "ether1".into()]
        );
        assert_eq!(
            prepare_lookup_options("ntp-server", "ntp-keys", vec!["1".into()]),
            vec!["none".to_string(), "1".into()]
        );
        assert_eq!(
            prepare_lookup_options("romon-ports", "interfaces", vec!["ether1".into()]),
            vec!["all".to_string(), "ether1".into()]
        );
        assert_eq!(
            prepare_lookup_options("graphing-interface", "interfaces", vec!["ether1".into()]),
            vec!["all".to_string(), "ether1".into()]
        );
        assert_eq!(
            prepare_lookup_options("graphing-queue", "queue-simple", vec!["guest".into()]),
            vec!["all".to_string(), "guest".into()]
        );
        assert_eq!(
            prepare_lookup_options("sniffer", "interfaces", vec!["ether1".into()]),
            vec!["ether1".to_string()]
        );
        assert_eq!(
            default_writable_value(FieldKind::Lookup {
                resource_id: "ntp-keys",
                value_key: "key-id",
                multiple: false,
            }),
            "none"
        );
    }

    #[test]
    fn number_fields_reject_non_digits_and_cap_ports() {
        assert!(FieldKind::Number.accepts_char("vlan-id", "10", '1'));
        assert!(!FieldKind::Number.accepts_char("vlan-id", "10", 'a'));
        assert!(!FieldKind::Number.accepts_char("vlan-id", "10", '-'));
        assert!(FieldKind::Number.accepts_char("remote-port", "6553", '5'));
        assert!(!FieldKind::Number.accepts_char("remote-port", "65535", '0'));
        assert!(FieldKind::Number.accepts_char("memory-lines", "65535", '0'));
        assert!(!is_tcp_udp_port_key("src-ports"));
        assert!(is_tcp_udp_port_key("remote-port"));
        assert!(is_tcp_udp_port_key("port"));
    }

    #[test]
    fn logging_action_fields_follow_type_and_log_format() {
        let mut values = HashMap::new();
        values.insert("target".into(), "memory".into());
        assert!(field_visible("logging-actions", "name", &values));
        assert!(field_visible("logging-actions", "memory-lines", &values));
        assert!(field_visible("logging-actions", "remember", &values));
        assert!(!field_visible("logging-actions", "remote", &values));
        assert!(!field_visible("logging-actions", "disk-file-name", &values));
        assert!(!field_visible("logging-actions", "email-to", &values));
        assert!(!field_visible("logging-actions", "script", &values));

        values.insert("target".into(), "disk".into());
        assert!(field_visible("logging-actions", "disk-file-name", &values));
        assert!(!field_visible("logging-actions", "memory-lines", &values));

        values.insert("target".into(), "email".into());
        assert!(field_visible("logging-actions", "email-to", &values));
        assert!(!field_visible("logging-actions", "remote", &values));

        values.insert("target".into(), "script".into());
        assert!(field_visible("logging-actions", "script", &values));

        values.insert("target".into(), "echo".into());
        assert!(field_visible("logging-actions", "remember", &values));
        assert!(!field_visible("logging-actions", "memory-lines", &values));

        values.insert("target".into(), "remote".into());
        values.insert("remote-log-format".into(), "default".into());
        assert!(field_visible("logging-actions", "remote", &values));
        assert!(field_visible(
            "logging-actions",
            "remote-log-format",
            &values
        ));
        assert!(!field_visible(
            "logging-actions",
            "syslog-facility",
            &values
        ));
        assert!(!field_visible(
            "logging-actions",
            "syslog-time-format",
            &values
        ));
        assert!(!field_visible(
            "logging-actions",
            "cef-event-delimiter",
            &values
        ));
        assert!(!field_visible("logging-actions", "memory-lines", &values));
        assert!(!field_visible(
            "logging-actions",
            "check-certificate",
            &values
        ));
        values.insert("remote-protocol".into(), "tcp".into());
        assert!(!field_visible(
            "logging-actions",
            "check-certificate",
            &values
        ));
        values.insert("remote-protocol".into(), "tls".into());
        assert!(field_visible(
            "logging-actions",
            "check-certificate",
            &values
        ));

        values.insert("remote-log-format".into(), "syslog".into());
        assert!(field_visible("logging-actions", "syslog-facility", &values));
        assert!(field_visible(
            "logging-actions",
            "syslog-time-format",
            &values
        ));
        assert!(!field_visible(
            "logging-actions",
            "cef-event-delimiter",
            &values
        ));

        values.insert("remote-log-format".into(), "cef".into());
        assert!(!field_visible(
            "logging-actions",
            "syslog-facility",
            &values
        ));
        assert!(field_visible(
            "logging-actions",
            "syslog-time-format",
            &values
        ));
        assert!(field_visible(
            "logging-actions",
            "cef-event-delimiter",
            &values
        ));
    }

    #[test]
    fn ntp_server_fields_follow_broadcast_and_local_clock() {
        let mut values = HashMap::new();
        values.insert("broadcast".into(), "false".into());
        values.insert("use-local-clock".into(), "false".into());
        assert!(field_visible("ntp-server", "enabled", &values));
        assert!(field_visible("ntp-server", "vrf", &values));
        assert!(!field_visible("ntp-server", "broadcast-addresses", &values));
        assert!(!field_visible("ntp-server", "local-clock-stratum", &values));

        values.insert("broadcast".into(), "true".into());
        assert!(field_visible("ntp-server", "broadcast-addresses", &values));
        assert!(!field_visible("ntp-server", "local-clock-stratum", &values));

        values.insert("use-local-clock".into(), "yes".into());
        assert!(field_visible("ntp-server", "local-clock-stratum", &values));
        assert!(field_enabled("ntp-server", "local-clock-stratum", &values));

        values.insert("broadcast".into(), "false".into());
        assert!(!field_enabled("ntp-server", "broadcast-addresses", &values));
    }

    #[test]
    fn traffic_flow_and_igmp_fields_follow_flags_and_version() {
        let cases = [
            (
                "traffic-flow",
                "packet-sampling",
                "false",
                "sampling-interval",
                false,
            ),
            (
                "traffic-flow",
                "packet-sampling",
                "yes",
                "sampling-space",
                true,
            ),
            (
                "traffic-flow-targets",
                "version",
                "5",
                "v9-template-refresh",
                false,
            ),
            (
                "traffic-flow-targets",
                "version",
                "9",
                "v9-template-timeout",
                true,
            ),
            (
                "traffic-flow-targets",
                "version",
                "ipfix",
                "v9-template-refresh",
                true,
            ),
            (
                "traffic-flow-targets",
                "version",
                "IPFIX",
                "v9-template-timeout",
                true,
            ),
            (
                "igmp-proxy-interfaces",
                "upstream",
                "no",
                "alternative-subnets",
                false,
            ),
            (
                "igmp-proxy-interfaces",
                "upstream",
                "true",
                "alternative-subnets",
                true,
            ),
        ];
        for (resource, flag, value, field, visible) in cases {
            let values = HashMap::from([(flag.to_string(), value.to_string())]);
            assert_eq!(
                field_visible(resource, field, &values),
                visible,
                "{resource} {field} with {flag}={value}"
            );
            assert_eq!(
                field_enabled(resource, field, &values),
                visible,
                "{resource} {field} enabled with {flag}={value}"
            );
            assert!(field_visible(resource, flag, &values));
        }
    }

    #[test]
    fn lte_apn_hides_auth_and_passthrough_until_selected() {
        let mut values = HashMap::new();
        values.insert("authentication".into(), "none".into());
        assert!(field_visible("lte-apn", "apn", &values));
        assert!(!field_visible("lte-apn", "user", &values));
        assert!(!field_visible("lte-apn", "password", &values));
        assert!(!field_enabled("lte-apn", "password", &values));
        assert!(field_visible("lte-apn", "passthrough-interface", &values));
        assert!(!field_visible("lte-apn", "passthrough-mac", &values));
        assert!(!field_visible(
            "lte-apn",
            "passthrough-subnet-selection",
            &values
        ));

        values.insert("authentication".into(), "chap".into());
        assert!(field_visible("lte-apn", "user", &values));
        assert!(field_visible("lte-apn", "password", &values));
        assert!(field_enabled("lte-apn", "password", &values));

        values.insert("passthrough-interface".into(), "ether2".into());
        assert!(field_visible("lte-apn", "passthrough-mac", &values));
        assert!(field_visible(
            "lte-apn",
            "passthrough-subnet-selection",
            &values
        ));
    }
}
