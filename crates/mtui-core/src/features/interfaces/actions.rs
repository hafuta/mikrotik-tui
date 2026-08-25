//! Action policy selected by feature-owned Interfaces resources.

use crate::actions::{
    ACTION_ADD, ACTION_ADD_TYPE, ACTION_AT_CHAT, ACTION_COPY, ACTION_EDIT, ACTION_REMOVE,
    ACTION_RESET, ACTION_SCAN, ACTION_TOGGLE, ACTION_TORCH, ActionKind, ActionSpec, ActionWhen,
};
pub(crate) use crate::actions::{
    DISCONNECT_ACTIONS, LIST_ACTIONS, MEMBER_ACTIONS, SINGLETON_EDIT_ACTIONS, VIRTUAL_IFACE_ACTIONS,
};

pub const INTERFACE_LIST_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD_TYPE,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RESET,
    ACTION_TORCH,
];

pub const ETHERNET_ACTIONS: &[ActionSpec] =
    &[ACTION_EDIT, ACTION_TOGGLE, ACTION_RESET, ACTION_TORCH];

pub const ACTION_NEW_LIST: ActionSpec = ActionSpec {
    id: "add",
    label: "New list",
    key: Some('n'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Create,
    when: ActionWhen::Always,
};

pub const ACTION_NEW_LIST_MEMBER: ActionSpec = ActionSpec {
    id: "add",
    label: "New list member",
    key: Some('n'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Create,
    when: ActionWhen::Always,
};

pub const INTERFACE_LIST_DEF_ACTIONS: &[ActionSpec] =
    &[ACTION_NEW_LIST, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

pub const INTERFACE_LIST_MEMBER_ACTIONS: &[ActionSpec] = &[
    ACTION_NEW_LIST_MEMBER,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
];

pub const RADIO_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RESET,
    ACTION_SCAN,
];

pub const LTE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE, ACTION_RESET, ACTION_AT_CHAT];

pub const VRF_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

/// Create targets offered from the generic Interface screen.
pub const INTERFACE_CREATE_TARGETS: &[(&str, &str)] = &[
    ("vlan", "VLAN"),
    ("eoip", "EoIP Tunnel"),
    ("ipip", "IP Tunnel"),
    ("gre", "GRE Tunnel"),
    ("vxlan", "VXLAN"),
    ("vrrp", "VRRP"),
    ("bonding", "Bonding"),
    ("macvlan", "MACVLAN"),
    ("veth", "VETH"),
    ("macsec", "MACsec"),
    ("macsec-profiles", "MACsec Profile"),
    ("lte-apn", "LTE APN"),
    ("interface-lists", "Lists"),
    ("interface-list-members", "List members"),
    ("vrf", "VRF"),
    ("wifi", "WiFi"),
    ("6to4", "6to4 Tunnel"),
    ("gre6", "GRE6 Tunnel"),
];
