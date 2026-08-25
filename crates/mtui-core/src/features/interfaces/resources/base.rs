//! Base Interfaces resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

pub const INTERFACES: ResourceSpec = ResourceSpec {
    id: "interfaces",
    group: "interfaces-group",
    cli_path: None,
    label: "Interface",
    fetch: FetchKind::List {
        endpoint: "/rest/interface",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("type", "Type", 14),
        col!("mtu", "MTU", 7),
        col!("actual-mtu", "Actual MTU", 11),
        col!("l2mtu", "L2 MTU", 8),
        col!("max-l2mtu", "Max L2 MTU", 11),
        col!("mac-address", "MAC address", 18),
        col!("tx-byte", "TX", 12),
        col!("rx-byte", "RX", 12),
        col!("tx-packet", "TX pkt", 10),
        col!("rx-packet", "RX pkt", 10),
        col!("fp-tx-byte", "FP TX", 12),
        col!("fp-rx-byte", "FP RX", 12),
        col!("fp-tx-packet", "FP TX pkt", 10),
        col!("fp-rx-packet", "FP RX pkt", 10),
        col!("last-link-up-time", "Last link up", 20),
        col!("last-link-down-time", "Last link down", 20),
        col!("link-downs", "Link downs", 11),
        col!("tx-drop", "TX drop", 9),
        col!("rx-drop", "RX drop", 9),
        col!("tx-queue-drop", "TX q-drop", 10),
        col!("rx-error", "RX error", 9),
        col!("tx-error", "TX error", 9),
        col!("running", "Run", 5),
        col!("slave", "Slave", 6),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::INTERFACE_LIST_ACTIONS,
    form: Some(&crate::features::interfaces::forms::INTERFACES_FORM),
};

pub const INTERFACE_LISTS: ResourceSpec = ResourceSpec {
    id: "interface-lists",
    group: "interfaces-group",
    cli_path: None,
    label: "Lists",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/list",
    },
    columns: &[
        col!("name", "Name", 20),
        col!("include", "Include", 24),
        col!("exclude", "Exclude", 24),
        col!("builtin", "Builtin", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::INTERFACE_LIST_DEF_ACTIONS,
    form: Some(&crate::features::interfaces::forms::LIST_FORM),
};

pub const INTERFACE_LIST_MEMBERS: ResourceSpec = ResourceSpec {
    id: "interface-list-members",
    group: "interfaces-group",
    cli_path: None,
    label: "List members",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/list/member",
    },
    columns: &[
        col!("list", "List", 16),
        col!("interface", "Interface", 18),
        col!("dynamic", "Dyn", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::features::interfaces::actions::INTERFACE_LIST_MEMBER_ACTIONS,
    form: Some(&crate::features::interfaces::forms::MEMBER_FORM),
};

pub const ETHERNET: ResourceSpec = ResourceSpec {
    id: "ethernet",
    group: "interfaces-group",
    cli_path: None,
    label: "Ethernet",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/ethernet",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("default-name", "Default name", 16),
        col!("mtu", "MTU", 7),
        col!("l2mtu", "L2 MTU", 8),
        col!("mac-address", "MAC address", 18),
        col!("orig-mac-address", "Orig MAC", 18),
        col!("arp", "ARP", 16),
        col!("auto-negotiation", "Auto-neg", 9),
        col!("advertise", "Advertise", 28),
        col!("speed", "Speed", 16),
        col!("full-duplex", "Duplex", 8),
        col!("switch", "Switch", 12),
        col!("loop-protect", "Loop protect", 13),
        col!("loop-protect-status", "Loop status", 12),
        col!("running", "Run", 5),
        col!("slave", "Slave", 6),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::features::interfaces::actions::ETHERNET_ACTIONS,
    form: Some(&crate::features::interfaces::forms::ETHERNET_FORM),
};
