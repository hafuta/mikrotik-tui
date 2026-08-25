//! Wifi Interfaces resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

pub const WIFI: ResourceSpec = ResourceSpec {
    id: "wifi",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("default-name", "Default name", 16),
        col!("configuration", "Configuration", 20),
        col!("master-interface", "Master", 16),
        col!("mac-address", "MAC address", 18),
        col!("radio-mac", "Radio MAC", 18),
        col!("current-channel", "Channel", 16),
        col!("ssid", "SSID", 20),
        col!("mtu", "MTU", 7),
        col!("l2mtu", "L2 MTU", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::RADIO_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_FORM),
};

pub const WIFI_SECURITY: ResourceSpec = ResourceSpec {
    id: "wifi-security",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi Security",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi/security",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("authentication-types", "Auth", 18),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_SECURITY_FORM),
};

pub const WIFI_CHANNEL: ResourceSpec = ResourceSpec {
    id: "wifi-channel",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi Channel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi/channel",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("band", "Band", 12),
        col!("frequency", "Frequency", 14),
        col!("width", "Width", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_CHANNEL_FORM),
};

pub const WIFI_DATAPATH: ResourceSpec = ResourceSpec {
    id: "wifi-datapath",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi Datapath",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi/datapath",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("bridge", "Bridge", 16),
        col!("vlan-id", "VLAN", 6),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_DATAPATH_FORM),
};

pub const WIFI_CONFIGURATION: ResourceSpec = ResourceSpec {
    id: "wifi-configuration",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi Configuration",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi/configuration",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("ssid", "SSID", 20),
        col!("country", "Country", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_CONFIGURATION_FORM),
};

pub const WIFI_PROVISIONING: ResourceSpec = ResourceSpec {
    id: "wifi-provisioning",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi Provisioning",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/wifi/provisioning",
    },
    columns: &[
        col!("action", "Action", 14),
        col!("supported-bands", "Bands", 16),
        col!("master-configuration", "Master", 18),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_PROVISIONING_FORM),
};

pub const WIFI_CAP: ResourceSpec = ResourceSpec {
    id: "wifi-cap",
    group: "interfaces-group",
    cli_path: None,
    label: "WiFi CAP",
    fetch: FetchKind::System {
        endpoint: "/rest/interface/wifi/cap",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("caps-man-addresses", "CAPsMAN", 24),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_CAP_FORM),
};

pub const WIFI_CAPSMAN: ResourceSpec = ResourceSpec {
    id: "wifi-capsman",
    group: "interfaces-group",
    cli_path: None,
    label: "CAPsMAN",
    fetch: FetchKind::System {
        endpoint: "/rest/interface/wifi/capsman",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("ca-certificate", "CA", 18),
        col!("certificate", "Certificate", 18),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::interfaces::forms::WIFI_CAPSMAN_FORM),
};
