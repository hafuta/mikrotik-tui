//! Feature-owned catalog entries for the Switch navigation group.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        crate::resources::ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    SWITCH,
    SWITCH_PORT,
    SWITCH_VLAN,
    SWITCH_HOST,
    SWITCH_RULE,
    SWITCH_PORT_ISOLATION,
    SWITCH_L3HW,
];

const SWITCH: ResourceSpec = ResourceSpec {
    id: "switch",
    group: "switch-group",
    cli_path: None,
    label: "Switch",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch",
    },
    columns: &[
        col!("name", "Name", 12),
        col!("type", "Type", 18),
        col!("mirror-source", "Mirror src", 16),
        col!("mirror-target", "Mirror dst", 16),
        col!("mirror-egress-target", "Egress dst", 16),
        col!("cpu-flow-control", "CPU FC", 8),
        col!("l3-hw-offloading", "L3HW", 6),
        col!("switch-all-ports", "All ports", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::HARDWARE_EDIT_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_FORM),
};

const SWITCH_PORT: ResourceSpec = ResourceSpec {
    id: "switch-port",
    group: "switch-group",
    cli_path: None,
    label: "Ports",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch/port",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("switch", "Switch", 12),
        col!("vlan-mode", "VLAN mode", 10),
        col!("vlan-header", "VLAN hdr", 14),
        col!("default-vlan-id", "Default VID", 12),
        col!("ingress-rate", "Ingress", 12),
        col!("egress-rate", "Egress", 12),
        col!("storm-rate", "Storm", 8),
        col!("l3-hw-offloading", "L3HW", 6),
        col!("mirror-ingress", "Mir in", 7),
        col!("mirror-egress", "Mir out", 8),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::HARDWARE_EDIT_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_PORT_FORM),
};

const SWITCH_VLAN: ResourceSpec = ResourceSpec {
    id: "switch-vlan",
    group: "switch-group",
    cli_path: None,
    label: "VLANs",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch/vlan",
    },
    columns: &[
        col!("switch", "Switch", 12),
        col!("vlan-id", "VLAN ID", 8),
        col!("ports", "Ports", 36),
        col!("independent-learning", "IVL", 5),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_VLAN_FORM),
};

const SWITCH_HOST: ResourceSpec = ResourceSpec {
    id: "switch-host",
    group: "switch-group",
    cli_path: None,
    label: "Hosts",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch/host",
    },
    columns: &[
        col!("switch", "Switch", 12),
        col!("mac-address", "MAC address", 18),
        col!("ports", "Ports", 24),
        col!("vlan-id", "VLAN ID", 8),
        col!("drop", "Drop", 6),
        col!("mirror", "Mirror", 7),
        col!("copy-to-cpu", "Copy CPU", 9),
        col!("redirect-to-cpu", "Redir CPU", 10),
        col!("dynamic", "Dyn", 5),
        col!("invalid", "Bad", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::HOST_TABLE_ACTIONS,
    form: None,
};

const SWITCH_RULE: ResourceSpec = ResourceSpec {
    id: "switch-rule",
    group: "switch-group",
    cli_path: None,
    label: "Rules",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch/rule",
    },
    columns: &[
        col!("switch", "Switch", 12),
        col!("ports", "Ports", 24),
        col!("mac-protocol", "MAC proto", 12),
        col!("src-mac-address", "Src MAC", 20),
        col!("dst-mac-address", "Dst MAC", 20),
        col!("protocol", "IP proto", 10),
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("src-port", "Src port", 10),
        col!("dst-port", "Dst port", 10),
        col!("vlan-id", "VLAN ID", 8),
        col!("new-dst-ports", "New dst", 20),
        col!("redirect-to-cpu", "Redir CPU", 10),
        col!("mirror", "Mirror", 7),
        col!("invalid", "Bad", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_RULE_FORM),
};

const SWITCH_PORT_ISOLATION: ResourceSpec = ResourceSpec {
    id: "switch-port-isolation",
    group: "switch-group",
    cli_path: None,
    label: "Port Isolation",
    fetch: FetchKind::List {
        endpoint: "/interface/ethernet/switch/port-isolation",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("switch", "Switch", 12),
        col!("forwarding-override", "Forward to", 36),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::HARDWARE_EDIT_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_PORT_ISOLATION_FORM),
};

const SWITCH_L3HW: ResourceSpec = ResourceSpec {
    id: "switch-l3hw",
    group: "switch-group",
    cli_path: None,
    label: "L3HW Settings",
    fetch: FetchKind::System {
        endpoint: "/interface/ethernet/switch/l3hw-settings",
    },
    columns: &[
        col!("autorestart", "Autorestart", 12),
        col!("fasttrack-hw", "FastTrack HW", 13),
        col!("ipv6-hw", "IPv6 HW", 8),
        col!("icmp-reply-on-error", "ICMP error", 11),
        col!("hw-supports-fasttrack", "FT support", 11),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::switch::forms::SWITCH_L3HW_FORM),
};
