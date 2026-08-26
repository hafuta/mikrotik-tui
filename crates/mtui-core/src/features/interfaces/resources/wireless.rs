//! Wireless Interfaces resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

pub const WIRELESS: ResourceSpec = ResourceSpec {
    id: "wireless",
    group: "interfaces-group",
    cli_path: None,
    label: "Wireless",
    fetch: FetchKind::List {
        endpoint: "/interface/wireless",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("default-name", "Default name", 16),
        col!("ssid", "SSID", 20),
        col!("mode", "Mode", 14),
        col!("band", "Band", 14),
        col!("frequency", "Frequency", 12),
        col!("mac-address", "MAC address", 18),
        col!("mtu", "MTU", 7),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::RADIO_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIRELESS_FORM),
};

pub const WIRELESS_SECURITY_PROFILES: ResourceSpec = ResourceSpec {
    id: "wireless-security-profiles",
    group: "interfaces-group",
    cli_path: None,
    label: "Security Profiles",
    fetch: FetchKind::List {
        endpoint: "/interface/wireless/security-profiles",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("mode", "Mode", 12),
        col!("authentication-types", "Auth", 18),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIRELESS_SECURITY_FORM),
};

pub const WIRELESS_ACCESS_LIST: ResourceSpec = ResourceSpec {
    id: "wireless-access-list",
    group: "interfaces-group",
    cli_path: None,
    label: "Access List",
    fetch: FetchKind::List {
        endpoint: "/interface/wireless/access-list",
    },
    columns: &[
        col!("mac-address", "MAC", 18),
        col!("interface", "Interface", 16),
        col!("authentication", "Auth", 6),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::features::interfaces::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIRELESS_ACCESS_LIST_FORM),
};

pub const WIRELESS_REGISTRATION_TABLE: ResourceSpec = ResourceSpec {
    id: "wireless-registration-table",
    group: "interfaces-group",
    cli_path: None,
    label: "Registration",
    fetch: FetchKind::List {
        endpoint: "/interface/wireless/registration-table",
    },
    columns: &[
        col!("mac-address", "MAC", 18),
        col!("interface", "Interface", 16),
        col!("ap", "AP", 5),
        col!("signal-strength", "Signal", 10),
        col!("uptime", "Uptime", 10),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::DISCONNECT_ACTIONS,
    form: None,
};
