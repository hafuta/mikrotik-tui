//! Feature-owned catalog entries for the complete Bridge navigation group.

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
    BRIDGES,
    BRIDGE_PORTS,
    BRIDGE_HOSTS,
    BRIDGE_VLANS,
    BRIDGE_MDB,
    BRIDGE_MSTI,
    BRIDGE_FILTER,
    BRIDGE_NAT,
    BRIDGE_SETTINGS,
    BRIDGE_PORT_CONTROLLER,
    BRIDGE_PORT_CONTROLLER_DEVICE,
    BRIDGE_PORT_CONTROLLER_PORT,
    BRIDGE_PORT_EXTENDER,
];

const BRIDGES: ResourceSpec = ResourceSpec {
    id: "bridges",
    group: "bridge-group",
    cli_path: None,
    label: "Bridge",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("protocol-mode", "Protocol", 12),
        col!("vlan-filtering", "VLAN", 6),
        col!("pvid", "PVID", 7),
        col!("igmp-snooping", "IGMP", 6),
        col!("dhcp-snooping", "DHCP snoop", 11),
        col!("arp", "ARP", 16),
        col!("mac-address", "MAC address", 18),
        col!("mtu", "MTU", 7),
        col!("fast-forward", "Fast fwd", 9),
        col!("frame-types", "Frames", 18),
        col!("ingress-filtering", "Ingress", 8),
        col!("priority", "Priority", 10),
        col!("region-name", "Region", 16),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_FORM),
};

const BRIDGE_PORTS: ResourceSpec = ResourceSpec {
    id: "bridge-ports",
    group: "bridge-group",
    cli_path: None,
    label: "Ports",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/port",
    },
    columns: &[
        col!("interface", "Interface", 18),
        col!("bridge", "Bridge", 18),
        col!("pvid", "PVID", 7),
        col!("hw", "HW", 4),
        col!("role", "Role", 16),
        col!("edge", "Edge", 12),
        col!("frame-types", "Frames", 18),
        col!("ingress-filtering", "Ingress", 8),
        col!("trusted", "Trusted", 8),
        col!("horizon", "Horizon", 8),
        col!("path-cost", "Cost", 8),
        col!("priority", "Priority", 9),
        col!("bpdu-guard", "BPDU", 6),
        col!("restricted-role", "Root guard", 11),
        col!("learn", "Learn", 8),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_PORT_FORM),
};

const BRIDGE_HOSTS: ResourceSpec = ResourceSpec {
    id: "bridge-hosts",
    group: "bridge-group",
    cli_path: None,
    label: "Hosts",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/host",
    },
    columns: &[
        col!("mac-address", "MAC address", 18),
        col!("vid", "VID", 6),
        col!("on-interface", "On interface", 18),
        col!("bridge", "Bridge", 16),
        col!("dynamic", "Dyn", 5),
        col!("local", "Local", 6),
        col!("external", "Ext", 5),
        col!("invalid", "Bad", 5),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::HOST_TABLE_ACTIONS,
    form: None,
};

const BRIDGE_VLANS: ResourceSpec = ResourceSpec {
    id: "bridge-vlans",
    group: "bridge-group",
    cli_path: None,
    label: "VLANs",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/vlan",
    },
    columns: &[
        col!("bridge", "Bridge", 16),
        col!("vlan-ids", "VLAN IDs", 14),
        col!("tagged", "Tagged", 24),
        col!("untagged", "Untagged", 24),
        col!("current-tagged", "Current tagged", 24),
        col!("current-untagged", "Current untagged", 24),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_VLAN_FORM),
};

const BRIDGE_MDB: ResourceSpec = ResourceSpec {
    id: "bridge-mdb",
    group: "bridge-group",
    cli_path: None,
    label: "MDB",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/mdb",
    },
    columns: &[
        col!("group", "Group", 22),
        col!("vid", "VID", 6),
        col!("on-ports", "On ports", 28),
        col!("bridge", "Bridge", 16),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_MDB_FORM),
};

const BRIDGE_MSTI: ResourceSpec = ResourceSpec {
    id: "bridge-msti",
    group: "bridge-group",
    cli_path: None,
    label: "MSTIs",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/msti",
    },
    columns: &[
        col!("bridge", "Bridge", 16),
        col!("identifier", "MSTI", 6),
        col!("vlan-mapping", "VLANs", 28),
        col!("priority", "Priority", 10),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_MSTI_FORM),
};

const BRIDGE_FILTER: ResourceSpec = ResourceSpec {
    id: "bridge-filter",
    group: "bridge-group",
    cli_path: None,
    label: "Filter",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/filter",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 12),
        col!("mac-protocol", "MAC proto", 12),
        col!("src-mac-address", "Src MAC", 20),
        col!("dst-mac-address", "Dst MAC", 20),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("ip-protocol", "IP proto", 10),
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("src-port", "Src port", 10),
        col!("dst-port", "Dst port", 10),
        col!("packets", "Packets", 12),
        col!("bytes", "Bytes", 14),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_FILTER_FORM),
};

const BRIDGE_NAT: ResourceSpec = ResourceSpec {
    id: "bridge-nat",
    group: "bridge-group",
    cli_path: None,
    label: "NAT",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/nat",
    },
    columns: &[
        col!("chain", "Chain", 10),
        col!("action", "Action", 14),
        col!("mac-protocol", "MAC proto", 12),
        col!("src-mac-address", "Src MAC", 20),
        col!("dst-mac-address", "Dst MAC", 20),
        col!("in-interface", "In interface", 16),
        col!("out-interface", "Out interface", 16),
        col!("to-src-mac-address", "To src MAC", 20),
        col!("to-dst-mac-address", "To dst MAC", 20),
        col!("packets", "Packets", 12),
        col!("bytes", "Bytes", 14),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_NAT_FORM),
};

const BRIDGE_SETTINGS: ResourceSpec = ResourceSpec {
    id: "bridge-settings",
    group: "bridge-group",
    cli_path: None,
    label: "Settings",
    fetch: FetchKind::System {
        endpoint: "/interface/bridge/settings",
    },
    columns: &[
        col!("use-ip-firewall", "IP firewall", 12),
        col!("use-ip-firewall-for-vlan", "VLAN FW", 8),
        col!("use-ip-firewall-for-pppoe", "PPPoE FW", 9),
        col!("allow-fast-path", "Fast path", 10),
        col!("bridge-fast-path-active", "FP active", 10),
        col!("bridge-fast-path-packets", "FP pkts", 12),
        col!("bridge-fast-path-bytes", "FP bytes", 12),
        col!("bridge-fast-forward-packets", "FF pkts", 12),
        col!("bridge-fast-forward-bytes", "FF bytes", 12),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_SETTINGS_FORM),
};

const BRIDGE_PORT_CONTROLLER: ResourceSpec = ResourceSpec {
    id: "bridge-port-controller",
    group: "bridge-group",
    cli_path: None,
    label: "Port Controller",
    fetch: FetchKind::System {
        endpoint: "/interface/bridge/port-controller",
    },
    columns: &[
        col!("bridge", "Bridge", 16),
        col!("switch", "Switch", 12),
        col!("cascade-ports", "Cascade", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_PORT_CONTROLLER_FORM),
};

const BRIDGE_PORT_CONTROLLER_DEVICE: ResourceSpec = ResourceSpec {
    id: "bridge-port-controller-device",
    group: "bridge-group",
    cli_path: None,
    label: "Controller Devices",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/port-controller/device",
    },
    columns: &[
        col!("name", "Name", 16),
        col!("pe-mac", "PE MAC", 18),
        col!("descr", "Description", 36),
        col!("control-ports", "Control ports", 28),
        col!("status", "Status", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_PORT_CONTROLLER_DEVICE_FORM),
};

const BRIDGE_PORT_CONTROLLER_PORT: ResourceSpec = ResourceSpec {
    id: "bridge-port-controller-port",
    group: "bridge-group",
    cli_path: None,
    label: "Controller Ports",
    fetch: FetchKind::List {
        endpoint: "/interface/bridge/port-controller/port",
    },
    columns: &[
        col!("name", "Name", 20),
        col!("device", "Device", 16),
        col!("status", "Status", 12),
        col!("port-status", "Port status", 12),
        col!("rate", "Rate", 10),
        col!("pcid", "PCID", 8),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_PORT_CONTROLLER_PORT_FORM),
};

const BRIDGE_PORT_EXTENDER: ResourceSpec = ResourceSpec {
    id: "bridge-port-extender",
    group: "bridge-group",
    cli_path: None,
    label: "Port Extender",
    fetch: FetchKind::System {
        endpoint: "/interface/bridge/port-extender",
    },
    columns: &[
        col!("switch", "Switch", 12),
        col!("control-ports", "Control", 28),
        col!("excluded-ports", "Excluded", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::bridge::forms::BRIDGE_PORT_EXTENDER_FORM),
};
