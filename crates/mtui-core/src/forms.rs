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
        }
    }

    /// Footer hint for the focused control.
    #[must_use]
    pub fn edit_hint(self) -> &'static str {
        match self {
            Self::Text | Self::Number | Self::Secret => "type value",
            Self::Toggle => "space toggle",
            Self::Enum { .. } | Self::Lookup { .. } => "space pick",
            Self::Readonly => "read only",
        }
    }

    /// Whether printable keys, including digits, should go into this field.
    /// Lookup typing happens only inside the picker filter, not on the sheet.
    #[must_use]
    pub fn takes_typed_input(self) -> bool {
        matches!(self, Self::Text | Self::Number | Self::Secret)
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
            Self::Text | Self::Secret => true,
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

/// Whether a sheet field should appear given current values.
///
/// Logging Actions show only the knobs that belong to Type (`target`) and, for
/// remote, to Remote Log Format and Remote Protocol. Inapplicable rows are
/// omitted, not locked. Check Certificate appears only when protocol is `tls`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn field_visible(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    if resource_id != "logging-actions" {
        return true;
    }
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
}
