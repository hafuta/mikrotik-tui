//! Sectioned property-sheet schemas for resource editors.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};

/// One field in a properties sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
}

/// One labeled static choice. `label` is rendered; `value` is sent to the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumChoice {
    pub label: &'static str,
    pub value: &'static str,
}

/// Scalar controls that can be collapsed behind an optional `+` affordance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Text,
    Number { min: Option<i64>, max: Option<i64> },
    Time,
    Ip,
    Ipv6,
    Mac,
    Raw,
    Enum { choices: &'static [EnumChoice] },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    ConstrainedNumber {
        min: Option<i64>,
        max: Option<i64>,
    },
    Time,
    Ip,
    Ipv6,
    Mac,
    Raw,
    Toggle,
    /// Checkbox whose checked state is the inverse of the wire value.
    ///
    /// The `Enabled` control stores `disabled=false` when checked.
    InvertedToggle,
    Enum {
        values: &'static [&'static str],
    },
    LabeledEnum {
        choices: &'static [EnumChoice],
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
    /// A scalar omitted from the editor until explicitly activated.
    ///
    /// Deactivating an existing value writes `unset` (for example `auto` or
    /// `0.0.0.0`). Create forms omit inactive optionals entirely.
    Optional {
        kind: ScalarKind,
        unset: &'static str,
        unset_label: &'static str,
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
            Self::Number | Self::ConstrainedNumber { .. } => "num",
            Self::Time => "time",
            Self::Ip => "ip",
            Self::Ipv6 => "ipv6",
            Self::Mac => "mac",
            Self::Raw => "raw",
            Self::Toggle | Self::InvertedToggle => "toggle",
            Self::Enum { .. } | Self::LabeledEnum { .. } => "select",
            Self::Readonly => "read",
            Self::Secret => "secret",
            Self::Lookup { .. } => "lookup",
            Self::Repeat => "list",
            Self::Optional { kind, .. } => kind.tag(),
        }
    }

    /// Footer hint for the focused control.
    #[must_use]
    pub fn edit_hint(self) -> &'static str {
        match self {
            Self::Text
            | Self::Number
            | Self::ConstrainedNumber { .. }
            | Self::Time
            | Self::Ip
            | Self::Ipv6
            | Self::Mac
            | Self::Raw
            | Self::Secret
            | Self::Repeat => "type value",
            Self::Toggle | Self::InvertedToggle => "space toggle",
            Self::Enum { .. } | Self::LabeledEnum { .. } | Self::Lookup { .. } => "space pick",
            Self::Readonly => "read only",
            Self::Optional { kind, .. } => kind.edit_hint(),
        }
    }

    /// Whether printable keys, including digits, should go into this field.
    /// Lookup typing happens only inside the picker filter, not on the sheet.
    #[must_use]
    pub fn takes_typed_input(self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::Number
                | Self::ConstrainedNumber { .. }
                | Self::Time
                | Self::Ip
                | Self::Ipv6
                | Self::Mac
                | Self::Raw
                | Self::Secret
                | Self::Repeat
        ) || matches!(self, Self::Optional { kind, .. } if kind.takes_typed_input())
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
            Self::ConstrainedNumber { min, max } => {
                accepts_constrained_number_char(current, ch, min, max)
            }
            Self::Time => accepts_time_char(ch),
            Self::Ip | Self::Ipv6 => ch.is_ascii_hexdigit() || matches!(ch, '.' | ':' | '/'),
            Self::Mac => ch.is_ascii_hexdigit() || matches!(ch, ':' | '-'),
            Self::Raw | Self::Text | Self::Secret | Self::Repeat => true,
            Self::Optional { kind, .. } => kind.accepts_char(current, ch),
            _ => false,
        }
    }

    #[must_use]
    pub fn optional(self) -> Option<(ScalarKind, &'static str, &'static str)> {
        match self {
            Self::Optional {
                kind,
                unset,
                unset_label,
            } => Some((kind, unset, unset_label)),
            _ => None,
        }
    }

    #[must_use]
    pub fn display_value(self, raw: &str) -> String {
        match self {
            Self::LabeledEnum { choices } => choices
                .iter()
                .find(|choice| choice.value == raw)
                .map_or_else(|| raw.to_string(), |choice| choice.label.to_string()),
            Self::Optional {
                kind: ScalarKind::Enum { choices },
                ..
            } => choices
                .iter()
                .find(|choice| choice.value == raw)
                .map_or_else(|| raw.to_string(), |choice| choice.label.to_string()),
            Self::InvertedToggle => {
                if toggle_wire_on(raw) {
                    "no".to_string()
                } else {
                    "yes".to_string()
                }
            }
            Self::Toggle => {
                if toggle_wire_on(raw) {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            }
            _ => raw.to_string(),
        }
    }

    #[must_use]
    pub fn is_toggle(self) -> bool {
        matches!(self, Self::Toggle | Self::InvertedToggle)
    }

    #[must_use]
    pub fn toggle_is_on(self, raw: &str) -> bool {
        let on = toggle_wire_on(raw);
        match self {
            Self::InvertedToggle => !on,
            _ => on,
        }
    }

    #[must_use]
    pub fn validate(self, value: &str) -> bool {
        if value.is_empty() {
            return true;
        }
        match self {
            Self::ConstrainedNumber { min, max } => number_in_range(value, min, max),
            Self::Time => valid_routeros_time(value),
            Self::Ip => valid_ip_or_prefix(value),
            Self::Ipv6 => valid_ipv6_or_prefix(value),
            Self::Mac => valid_mac(value),
            Self::LabeledEnum { choices } => choices.iter().any(|choice| choice.value == value),
            Self::Optional { kind, unset, .. } => value == unset || kind.validate(value),
            _ => true,
        }
    }
}

impl ScalarKind {
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number { .. } => "num",
            Self::Time => "time",
            Self::Ip => "ip",
            Self::Ipv6 => "ipv6",
            Self::Mac => "mac",
            Self::Raw => "raw",
            Self::Enum { .. } => "select",
        }
    }

    #[must_use]
    pub fn edit_hint(self) -> &'static str {
        if matches!(self, Self::Enum { .. }) {
            "space pick   - remove"
        } else {
            "type value   - remove"
        }
    }

    #[must_use]
    pub fn takes_typed_input(self) -> bool {
        !matches!(self, Self::Enum { .. })
    }

    #[must_use]
    pub fn accepts_char(self, current: &str, ch: char) -> bool {
        match self {
            Self::Text | Self::Raw => true,
            Self::Number { min, max } => accepts_constrained_number_char(current, ch, min, max),
            Self::Time => accepts_time_char(ch),
            Self::Ip | Self::Ipv6 => ch.is_ascii_hexdigit() || matches!(ch, '.' | ':' | '/'),
            Self::Mac => ch.is_ascii_hexdigit() || matches!(ch, ':' | '-'),
            Self::Enum { .. } => false,
        }
    }

    #[must_use]
    pub fn validate(self, value: &str) -> bool {
        match self {
            Self::Number { min, max } => number_in_range(value, min, max),
            Self::Time => valid_routeros_time(value),
            Self::Ip => valid_ip_or_prefix(value),
            Self::Ipv6 => valid_ipv6_or_prefix(value),
            Self::Mac => valid_mac(value),
            Self::Enum { choices } => choices.iter().any(|choice| choice.value == value),
            Self::Text | Self::Raw => true,
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

#[must_use]
pub fn accepts_constrained_number_char(
    current: &str,
    ch: char,
    min: Option<i64>,
    max: Option<i64>,
) -> bool {
    if !ch.is_ascii_digit() {
        return false;
    }
    let candidate = format!("{current}{ch}");
    let Ok(value) = candidate.parse::<i64>() else {
        return false;
    };
    let _ = min;
    max.is_none_or(|limit| value <= limit)
}

fn number_in_range(value: &str, min: Option<i64>, max: Option<i64>) -> bool {
    value.parse::<i64>().is_ok_and(|value| {
        min.is_none_or(|limit| value >= limit) && max.is_none_or(|limit| value <= limit)
    })
}

fn accepts_time_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '-')
}

fn valid_routeros_time(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(accepts_time_char)
        && value.chars().any(|ch| ch.is_ascii_digit())
}

fn toggle_wire_on(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "1"
    )
}

fn valid_mac(value: &str) -> bool {
    let compact: String = value
        .chars()
        .filter(|ch| !matches!(ch, ':' | '-'))
        .collect();
    compact.len() == 12 && compact.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn valid_ip_or_prefix(value: &str) -> bool {
    let (address, prefix) = value.split_once('/').unwrap_or((value, ""));
    let Ok(address) = address.parse::<IpAddr>() else {
        return false;
    };
    if prefix.is_empty() {
        return !value.contains('/');
    }
    prefix.parse::<u8>().is_ok_and(|prefix| {
        prefix
            <= match address {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            }
    })
}

fn valid_ipv6_or_prefix(value: &str) -> bool {
    let (address, prefix) = value.split_once('/').unwrap_or((value, ""));
    if address.parse::<Ipv6Addr>().is_err() {
        return false;
    }
    if prefix.is_empty() {
        return !value.contains('/');
    }
    prefix.parse::<u8>().is_ok_and(|prefix| prefix <= 128)
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
        FieldKind::LabeledEnum { choices } => choices
            .first()
            .map_or("", |choice| choice.value)
            .to_string(),
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
    if sheet_resource_id == "lte-apn" && lookup_resource_id == "interfaces" {
        return with_leading_none(options);
    }
    if matches!(sheet_resource_id, "wifi-cap" | "wifi-capsman")
        && lookup_resource_id == "certificates"
    {
        return with_leading_none(options);
    }
    options
}

/// Declarative condition used by feature-owned form rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPredicate {
    Always,
    Truthy(&'static str),
    NonEmpty(&'static str),
    Equals {
        key: &'static str,
        value: &'static str,
    },
    /// True when `key` parses as an integer and shares any bit with `mask`.
    HasBits {
        key: &'static str,
        mask: u64,
    },
    /// True when `key` is present on the printed record (value may be empty).
    HasKey(&'static str),
    /// True when any stored attribute name starts with `prefix`.
    HasKeyPrefix(&'static str),
    /// True when any of `keys` starts with one of `prefixes` (ASCII case-insensitive).
    StartsWith {
        keys: &'static [&'static str],
        prefixes: &'static [&'static str],
    },
    Not(BoxedFieldPredicate),
    All(&'static [FieldPredicate]),
    Any(&'static [FieldPredicate]),
}

/// Indirection keeps [`FieldPredicate`] copyable and const-friendly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxedFieldPredicate(pub &'static FieldPredicate);

impl FieldPredicate {
    #[must_use]
    #[allow(clippy::implicit_hasher)]
    pub fn evaluate(self, values: &HashMap<String, String>) -> bool {
        match self {
            Self::Always => true,
            Self::Truthy(key) => flag_on(values, key),
            Self::NonEmpty(key) => values
                .get(key)
                .is_some_and(|value| !value.trim().is_empty()),
            Self::Equals { key, value } => values.get(key).is_some_and(|current| current == value),
            Self::HasBits { key, mask } => values
                .get(key)
                .and_then(|raw| parse_bitmask(raw))
                .is_some_and(|bits| bits & mask != 0),
            Self::HasKey(key) => values.contains_key(key),
            Self::HasKeyPrefix(prefix) => values.keys().any(|key| key.starts_with(prefix)),
            Self::StartsWith { keys, prefixes } => keys.iter().any(|key| {
                values.get(*key).is_some_and(|value| {
                    let value = value.trim().to_ascii_lowercase();
                    prefixes
                        .iter()
                        .any(|prefix| value.starts_with(&prefix.to_ascii_lowercase()))
                })
            }),
            Self::Not(predicate) => !predicate.0.evaluate(values),
            Self::All(predicates) => predicates
                .iter()
                .all(|predicate| predicate.evaluate(values)),
            Self::Any(predicates) => predicates
                .iter()
                .any(|predicate| predicate.evaluate(values)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldRule {
    pub resource_id: &'static str,
    pub field_key: &'static str,
    pub visible: FieldPredicate,
    pub enabled: FieldPredicate,
}

#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn evaluate_field_rules(
    rules: &[FieldRule],
    resource_id: &str,
    key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    rules
        .iter()
        .find(|rule| rule.resource_id == resource_id && rule.field_key == key)
        .map(|rule| (rule.visible.evaluate(values), rule.enabled.evaluate(values)))
}

/// Whether a sheet field should appear given current values.
///
/// Logging Actions show only the knobs that belong to Type (`target`) and, for
/// remote, to Remote Log Format and Remote Protocol. NTP Server shows Broadcast
/// Addresses only when Broadcast is on, and Local Clock Stratum only when Use
/// Local Clock is on. Traffic Flow shows Sampling Interval and Sampling Space
/// only when Packet Sampling is on. Traffic Flow Targets show v9 template
/// fields only for version `9` or `ipfix`. IGMP Proxy Interfaces show
/// Alternative Subnets only when Upstream is on. Disks show type-specific RAID,
/// network, and sharing fields. Inapplicable rows are omitted, not locked.
/// Check Certificate appears only when protocol is `tls`. Container create
/// shows Remote Image or File, not both once one is set.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn field_visible(resource_id: &str, key: &str, values: &HashMap<String, String>) -> bool {
    if let Some((visible, _)) = declared_field_state(resource_id, key, values) {
        return visible;
    }
    match resource_id {
        "logging-actions" => logging_action_field_visible(key, values),
        "ntp-server" => ntp_server_field_visible(key, values),
        "traffic-flow" => traffic_flow_field_visible(key, values),
        "traffic-flow-targets" => traffic_flow_target_field_visible(key, values),
        "igmp-proxy-interfaces" => igmp_proxy_interface_field_visible(key, values),
        "disks" => disk_field_visible(key, values),
        "containers" => container_field_visible(key, values),
        _ => true,
    }
}

fn flag_on(values: &HashMap<String, String>, key: &str) -> bool {
    crate::actions::truthy(values.get(key).map(String::as_str))
}

fn parse_bitmask(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).ok();
    }
    raw.parse::<u64>().ok()
}

fn container_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    let remote = values.get("remote-image").map_or("", String::as_str).trim();
    let file = values.get("file").map_or("", String::as_str).trim();
    match key {
        "remote-image" => file.is_empty(),
        "file" => remote.is_empty(),
        _ => true,
    }
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

fn disk_type(values: &HashMap<String, String>) -> &str {
    values.get("type").map_or("", String::as_str)
}

fn disk_raid_member(values: &HashMap<String, String>) -> bool {
    let master = values.get("raid-master").map_or("", String::as_str);
    !master.is_empty() && master != "none"
}

fn disk_field_visible(key: &str, values: &HashMap<String, String>) -> bool {
    let kind = disk_type(values);
    match key {
        "tmpfs-max-size" => kind == "tmpfs",
        "ramdisk-size" => kind == "ramdisk",
        "partition-number" | "partition-offset" | "partition-size" => kind == "partition",
        "raid-type" | "raid-device-count" | "raid-max-component-size" | "raid-chunk-size" => {
            kind == "raid"
        }
        "raid-master" | "raid-role" | "raid-member-failed" => {
            kind == "raid" || disk_raid_member(values)
        }
        "file-path" | "file-size" | "file-offset" => kind == "file",
        "crypted-backend" | "encryption-key" => kind == "crypted",
        "sshfs-address" | "sshfs-port" | "sshfs-user" | "sshfs-password" | "sshfs-path" => {
            kind == "sshfs"
        }
        "nfs-address" | "nfs-share" => kind == "nfs",
        "smb-address" | "smb-share" | "smb-user" | "smb-password" | "smb-encryption" => {
            kind == "smb"
        }
        "nvme-tcp-address" | "nvme-tcp-nqn" | "nvme-tcp-host-name" | "nvme-tcp-password"
        | "nvme-tcp-port" => kind == "nvme-tcp",
        "iscsi-address" | "iscsi-iqn" | "iscsi-port" => kind == "iscsi",
        "nvme-tcp-server-port" | "nvme-tcp-server-nqn" | "nvme-tcp-server-password" => {
            flag_on(values, "nvme-tcp-export")
        }
        "iscsi-server-port" | "iscsi-server-iqn" => flag_on(values, "iscsi-export"),
        "smb-server-user" | "smb-server-password" | "smb-server-encryption" => {
            flag_on(values, "smb-sharing")
        }
        "media-interface" => flag_on(values, "media-sharing"),
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
    if let Some((visible, enabled)) = declared_field_state(resource_id, key, values) {
        return visible && enabled;
    }
    field_visible(resource_id, key, values)
}

fn declared_field_state(
    resource_id: &str,
    key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    if let Some(state) =
        crate::features::interfaces::rules::form_field_state(resource_id, key, values)
    {
        return Some(state);
    }
    evaluate_field_rules(crate::switch_write::FIELD_RULES, resource_id, key, values)
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
    pub fn secret_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for section in self.sections.iter().chain(self.create_sections.iter()) {
            for field in section.fields {
                if matches!(field.kind, FieldKind::Secret) && !keys.contains(&field.key) {
                    keys.push(field.key);
                }
            }
        }
        keys
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
        .filter(|(key, _)| {
            key.as_str() != ".id"
                && !known.contains(&key.as_str())
                && !matches!(key.as_str(), "caps" | "sfp")
        })
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

/// Mutation body after writable, visibility, and enabled gates are applied.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn form_mutation_body(
    resource_id: &str,
    schema: &FormSchema,
    original: &HashMap<String, String>,
    current: &HashMap<String, String>,
    masked_token: &str,
) -> BTreeMap<String, String> {
    let mut body = patch_body(schema, original, current, masked_token);
    body.retain(|key, _| field_enabled(resource_id, key, current));
    body
}

/// First invalid active writable field, suitable for an inline form error.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn validate_form_values(
    resource_id: &str,
    schema: &FormSchema,
    current: &HashMap<String, String>,
) -> Option<String> {
    schema
        .sections_for(false)
        .iter()
        .chain(schema.create_sections.iter())
        .filter(|section| !section.read_only)
        .flat_map(|section| section.fields)
        .find_map(|field| {
            let value = current.get(field.key)?;
            if !field_enabled(resource_id, field.key, current) || field.kind.validate(value) {
                return None;
            }
            Some(format!("{} has an invalid value", field.label))
        })
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
    let body = form_mutation_body(resource_id, schema, original, current, masked_token);
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
        || normalized == "encryption-key"
        || normalized == "k"
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
            "passthrough-subnet-size",
            &values
        ));

        values.insert("authentication".into(), "chap".into());
        assert!(field_visible("lte-apn", "user", &values));
        assert!(field_visible("lte-apn", "password", &values));
        assert!(field_enabled("lte-apn", "password", &values));

        values.insert("passthrough-interface".into(), "ether2".into());
        assert!(field_visible("lte-apn", "passthrough-mac", &values));
        assert!(field_visible("lte-apn", "passthrough-subnet-size", &values));
    }

    #[test]
    fn veth_and_container_source_visibility() {
        let mut values = HashMap::new();
        values.insert("dhcp".into(), "true".into());
        assert!(field_visible("veth", "gateway", &values));
        assert!(field_visible("veth", "gateway6", &values));
        values.insert("dhcp".into(), "false".into());
        assert!(field_visible("veth", "gateway", &values));

        let mut values = HashMap::new();
        assert!(field_visible("containers", "remote-image", &values));
        assert!(field_visible("containers", "file", &values));
        values.insert("remote-image".into(), "pihole/pihole".into());
        assert!(field_visible("containers", "remote-image", &values));
        assert!(!field_visible("containers", "file", &values));
        values.insert("remote-image".into(), String::new());
        values.insert("file".into(), "disk1/pihole.tar".into());
        assert!(!field_visible("containers", "remote-image", &values));
        assert!(field_visible("containers", "file", &values));
    }

    #[test]
    fn constrained_numbers_filter_text_and_enforce_range() {
        let kind = FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(4094),
        };
        assert!(kind.accepts_char("vlan-id", "409", '4'));
        assert!(!kind.accepts_char("vlan-id", "4094", '5'));
        assert!(!kind.accepts_char("vlan-id", "", 'a'));
        assert!(kind.validate("1"));
        assert!(kind.validate("4094"));
        assert!(!kind.validate("0"));
        assert!(!kind.validate("auto"));
    }

    #[test]
    fn typed_network_time_and_mac_controls_validate_without_text_sentinels() {
        assert!(FieldKind::Ip.validate("192.0.2.1"));
        assert!(FieldKind::Ipv6.validate("2001:db8::1"));
        assert!(FieldKind::Mac.validate("02:00:00:00:00:01"));
        assert!(FieldKind::Time.validate("1d02:03:04"));
        assert!(!FieldKind::Ip.validate("auto"));
        assert!(!FieldKind::Mac.validate("not-a-mac"));
        assert!(!FieldKind::Time.accepts_char("timeout", "", '/'));
    }

    #[test]
    fn labeled_enum_keeps_display_and_wire_values_independent() {
        const CHOICES: &[EnumChoice] = &[
            EnumChoice {
                label: "Automatic",
                value: "auto",
            },
            EnumChoice {
                label: "1 Gbps",
                value: "1G-baseT-full",
            },
        ];
        let kind = FieldKind::LabeledEnum { choices: CHOICES };
        assert_eq!(kind.display_value("1G-baseT-full"), "1 Gbps");
        assert_eq!(default_writable_value(kind), "auto");
        assert!(kind.validate("1G-baseT-full"));
        assert!(!kind.validate("1 Gbps"));
    }

    #[test]
    fn mutation_body_omits_readonly_and_inactive_conditional_values() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "authentication",
                        label: "Authentication",
                        kind: FieldKind::Enum {
                            values: &["none", "chap"],
                        },
                    },
                    FieldSpec {
                        key: "password",
                        label: "Password",
                        kind: FieldKind::Secret,
                    },
                    FieldSpec {
                        key: "running",
                        label: "Running",
                        kind: FieldKind::Readonly,
                    },
                ],
            }],
            create_sections: &[],
        };
        let original = HashMap::from([
            ("authentication".into(), "chap".into()),
            ("password".into(), "old".into()),
            ("running".into(), "true".into()),
        ]);
        let current = HashMap::from([
            ("authentication".into(), "none".into()),
            ("password".into(), "new".into()),
            ("running".into(), "false".into()),
        ]);
        let body = form_mutation_body("lte-apn", &schema, &original, &current, "********");
        assert_eq!(body.get("authentication").map(String::as_str), Some("none"));
        assert!(!body.contains_key("password"));
        assert!(!body.contains_key("running"));
    }
}
