//! Logging Actions, NTP Server, and `RouterBOARD` Settings visibility.

use std::collections::HashMap;

use crate::forms::{FieldPredicate, FieldRule};

const fn printed(resource_id: &'static str, field_key: &'static str) -> FieldRule {
    FieldRule {
        resource_id,
        field_key,
        visible: FieldPredicate::HasKey(field_key),
        enabled: FieldPredicate::HasKey(field_key),
    }
}

/// Keys `WebFig` only paints when that board's print includes them.
const ROUTERBOARD_SETTINGS_RULES: &[FieldRule] = &[
    printed("routerboard-settings", "boot-os"),
    printed("routerboard-settings", "cpu-frequency"),
    printed("routerboard-settings", "memory-frequency"),
    printed("routerboard-settings", "enable-jumper-reset"),
];

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

/// Visibility and enablement for Logging Actions, NTP Server, and
/// `RouterBOARD` Settings.
///
/// Enabled follows visibility so hidden Type-specific knobs are not typed or
/// sent on save. Other System resources return `None` (crate `field_visible`
/// still handles disks, LEDs, and watchdog until those gates move).
#[must_use]
#[allow(clippy::implicit_hasher)]
pub(crate) fn form_field_state(
    resource_id: &str,
    key: &str,
    values: &HashMap<String, String>,
) -> Option<(bool, bool)> {
    if let Some(state) =
        crate::forms::evaluate_field_rules(ROUTERBOARD_SETTINGS_RULES, resource_id, key, values)
    {
        return Some(state);
    }
    match resource_id {
        "logging-actions" => {
            let visible = logging_action_field_visible(key, values);
            Some((visible, visible))
        }
        "ntp-server" => {
            let visible = ntp_server_field_visible(key, values);
            Some((visible, visible))
        }
        _ => None,
    }
}
