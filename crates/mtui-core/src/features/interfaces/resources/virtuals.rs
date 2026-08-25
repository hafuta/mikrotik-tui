//! Virtuals Interfaces resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

pub const VLAN: ResourceSpec = ResourceSpec {
    id: "vlan",
    group: "interfaces-group",
    cli_path: None,
    label: "VLAN",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/vlan",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("vlan-id", "VLAN ID", 8),
        col!("interface", "Interface", 16),
        col!("mtu", "MTU", 7),
        col!("l2mtu", "L2 MTU", 8),
        col!("mac-address", "MAC address", 18),
        col!("arp", "ARP", 16),
        col!("use-service-tag", "Service tag", 12),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::VLAN_FORM),
};

pub const VXLAN: ResourceSpec = ResourceSpec {
    id: "vxlan",
    group: "interfaces-group",
    cli_path: None,
    label: "VXLAN",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/vxlan",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("vni", "VNI", 8),
        col!("port", "Port", 8),
        col!("group", "Group", 16),
        col!("local", "Local", 18),
        col!("interface", "Interface", 16),
        col!("vrf", "VRF", 12),
        col!("mtu", "MTU", 7),
        col!("mac-address", "MAC address", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::VXLAN_FORM),
};

pub const VRRP: ResourceSpec = ResourceSpec {
    id: "vrrp",
    group: "interfaces-group",
    cli_path: None,
    label: "VRRP",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/vrrp",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("vrid", "VRID", 6),
        col!("priority", "Priority", 9),
        col!("interval", "Interval", 10),
        col!("version", "Version", 8),
        col!("v3-protocol", "V3 proto", 10),
        col!("preemption-mode", "Preempt", 8),
        col!("mac-address", "MAC address", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::VRRP_FORM),
};

pub const BONDING: ResourceSpec = ResourceSpec {
    id: "bonding",
    group: "interfaces-group",
    cli_path: None,
    label: "Bonding",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/bonding",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("slaves", "Slaves", 28),
        col!("mode", "Mode", 16),
        col!("link-monitoring", "Monitor", 16),
        col!("transmit-hash-policy", "Hash", 16),
        col!("primary", "Primary", 16),
        col!("mtu", "MTU", 7),
        col!("mac-address", "MAC address", 18),
        col!("arp", "ARP", 16),
        col!("min-links", "Min links", 10),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::BONDING_FORM),
};

pub const MACVLAN: ResourceSpec = ResourceSpec {
    id: "macvlan",
    group: "interfaces-group",
    cli_path: None,
    label: "MACVLAN",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/macvlan",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("mode", "Mode", 12),
        col!("mac-address", "MAC address", 18),
        col!("mtu", "MTU", 7),
        col!("arp", "ARP", 16),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::MACVLAN_FORM),
};

pub const VETH: ResourceSpec = ResourceSpec {
    id: "veth",
    group: "interfaces-group",
    cli_path: None,
    label: "VETH",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/veth",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("address", "Address", 24),
        col!("gateway", "Gateway", 16),
        col!("dhcp", "DHCP", 6),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::VETH_FORM),
};

pub const MACSEC: ResourceSpec = ResourceSpec {
    id: "macsec",
    group: "interfaces-group",
    cli_path: None,
    label: "MACsec",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/macsec",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("profile", "Profile", 14),
        col!("mtu", "MTU", 7),
        col!("status", "Status", 16),
        col!("ckn", "CKN", 24),
        col!("cak", "CAK", 10),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::MACSEC_FORM),
};

pub const MACSEC_PROFILES: ResourceSpec = ResourceSpec {
    id: "macsec-profiles",
    group: "interfaces-group",
    cli_path: None,
    label: "MACsec Profile",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/macsec/profile",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("server-priority", "Server priority", 16),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::MACSEC_PROFILE_FORM),
};

pub const VRF: ResourceSpec = ResourceSpec {
    id: "vrf",
    group: "interfaces-group",
    cli_path: None,
    label: "VRF",
    fetch: FetchKind::List {
        endpoint: "/rest/ip/vrf",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interfaces", "Interfaces", 36),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::VRF_ACTIONS,
    form: Some(&crate::features::interfaces::forms::VRF_FORM),
};

pub const DETECT_INTERNET: ResourceSpec = ResourceSpec {
    id: "detect-internet",
    group: "interfaces-group",
    cli_path: None,
    label: "Detect Internet",
    fetch: FetchKind::System {
        endpoint: "/rest/interface/detect-internet",
    },
    columns: &[
        col!("detect-interface-list", "Detect", 16),
        col!("lan-interface-list", "LAN", 16),
        col!("wan-interface-list", "WAN", 16),
        col!("internet-interface-list", "Internet", 16),
        col!("state", "State", 12),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::features::interfaces::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::interfaces::forms::DETECT_INTERNET_FORM),
};
