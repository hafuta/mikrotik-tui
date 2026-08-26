//! Cellular Interfaces resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

pub const LTE: ResourceSpec = ResourceSpec {
    id: "lte",
    group: "interfaces-group",
    cli_path: None,
    label: "LTE",
    fetch: FetchKind::List {
        endpoint: "/interface/lte",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("default-name", "Default name", 16),
        col!("mtu", "MTU", 7),
        col!("mac-address", "MAC address", 18),
        col!("network-mode", "Network", 14),
        col!("apn-profiles", "APN Profiles", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::LTE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::LTE_FORM),
};

pub const LTE_APN: ResourceSpec = ResourceSpec {
    id: "lte-apn",
    group: "interfaces-group",
    cli_path: None,
    label: "LTE APN",
    fetch: FetchKind::List {
        endpoint: "/interface/lte/apn",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("apn", "APN", 18),
        col!("authentication", "Authentication", 14),
        col!("ip-type", "IP Type", 12),
        col!("use-network-apn", "Network APN", 12),
        col!("add-default-route", "Default", 8),
        col!("password", "Password", 10),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::LTE_APN_FORM),
};
