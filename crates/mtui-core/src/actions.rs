//! Per-resource entity actions.

use std::collections::HashMap;

/// How an action is carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Open the properties sheet for the selected row.
    Edit,
    /// Open a create sheet (or a type picker when `overlay` is `create-type`).
    Create,
    /// Confirm, then run a REST command or delete.
    Confirm { command: ActionCommand },
    /// Prompt for extra fields, then run a REST command.
    Prompt { command: ActionCommand },
    /// Open a dedicated overlay (`torch`, `create-type`).
    Overlay { id: &'static str },
}

/// REST command word (or delete) after confirmation / prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCommand {
    Enable,
    Disable,
    ToggleDisabled,
    Remove,
    Copy,
    ResetCounters,
}

impl ActionCommand {
    #[must_use]
    pub fn rest_name(self) -> &'static str {
        match self {
            Self::Enable | Self::ToggleDisabled => "enable",
            Self::Disable => "disable",
            Self::Remove => "remove",
            Self::Copy => "copy",
            Self::ResetCounters => "reset-counters",
        }
    }
}

/// When an action is offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionWhen {
    Always,
    HasSelection,
    NotSingleton,
    /// Hidden when `dynamic`, `slave`, or `builtin` is true.
    MutableRecord,
}

/// Descriptor for one entity or collection action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub key: Option<char>,
    pub enter: bool,
    pub needs_selection: bool,
    pub danger: bool,
    pub kind: ActionKind,
    pub when: ActionWhen,
}

#[must_use]
pub fn truthy(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("true" | "yes" | "1")
    )
}

/// Actions that apply to the current resource and optional selected row.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn resolve_actions<'a>(
    actions: &'a [ActionSpec],
    is_singleton: bool,
    row: Option<&HashMap<String, String>>,
) -> Vec<&'a ActionSpec> {
    actions
        .iter()
        .filter(|action| action_available(action, is_singleton, row))
        .collect()
}

#[must_use]
pub fn action_available(
    action: &ActionSpec,
    is_singleton: bool,
    row: Option<&HashMap<String, String>>,
) -> bool {
    if action.needs_selection && row.is_none() {
        return false;
    }
    match action.when {
        ActionWhen::Always => true,
        ActionWhen::HasSelection => row.is_some(),
        ActionWhen::NotSingleton => !is_singleton,
        ActionWhen::MutableRecord => {
            let Some(row) = row else {
                return false;
            };
            !truthy(row.get("dynamic").map(String::as_str))
                && !truthy(row.get("slave").map(String::as_str))
                && !truthy(row.get("builtin").map(String::as_str))
        }
    }
}

/// Label that can depend on row state (enable vs disable).
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn action_label(action: &ActionSpec, row: Option<&HashMap<String, String>>) -> String {
    if action.id == "toggle-disabled" {
        if truthy(row.and_then(|row| row.get("disabled").map(String::as_str))) {
            "Enable".into()
        } else {
            "Disable".into()
        }
    } else {
        action.label.to_string()
    }
}

pub const ACTION_ADD: ActionSpec = ActionSpec {
    id: "add",
    label: "Add",
    key: Some('n'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Create,
    when: ActionWhen::Always,
};

pub const ACTION_ADD_TYPE: ActionSpec = ActionSpec {
    id: "add",
    label: "Add",
    key: Some('n'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "create-type" },
    when: ActionWhen::Always,
};

pub const ACTION_EDIT: ActionSpec = ActionSpec {
    id: "edit",
    label: "Edit",
    key: Some('e'),
    enter: true,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Edit,
    when: ActionWhen::HasSelection,
};

pub const ACTION_TOGGLE: ActionSpec = ActionSpec {
    id: "toggle-disabled",
    label: "Disable",
    key: Some('d'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::ToggleDisabled,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_COPY: ActionSpec = ActionSpec {
    id: "copy",
    label: "Copy",
    key: Some('c'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Copy,
    },
    when: ActionWhen::MutableRecord,
};

pub const ACTION_REMOVE: ActionSpec = ActionSpec {
    id: "remove",
    label: "Remove",
    key: Some('x'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Remove,
    },
    when: ActionWhen::MutableRecord,
};

/// Remove even when the row is `dynamic` (sessions, FDB hosts, leases).
pub const ACTION_REMOVE_SELECTED: ActionSpec = ActionSpec {
    id: "remove",
    label: "Remove",
    key: Some('x'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Remove,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_RESET: ActionSpec = ActionSpec {
    id: "reset-counters",
    label: "Reset counters",
    key: Some('z'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::ResetCounters,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_TORCH: ActionSpec = ActionSpec {
    id: "torch",
    label: "Torch",
    key: Some('t'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Overlay { id: "torch" },
    when: ActionWhen::HasSelection,
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

pub const VIRTUAL_IFACE_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RESET,
];

pub const LIST_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

pub const MEMBER_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
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
];

pub const LTE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE, ACTION_RESET];

pub const SINGLETON_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT];

pub const VRF_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

/// Firewall / bridge filter / NAT / mangle / switch rules.
pub const FILTER_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RESET,
];

/// Disconnect a live session or drop an FDB/lease row.
pub const DISCONNECT_ACTIONS: &[ActionSpec] = &[ACTION_REMOVE_SELECTED];

/// Hardware switch chip: edit only (no add/remove).
pub const HARDWARE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT];

/// Enable/disable without add (packages, some system lists).
pub const TOGGLE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE];

/// DHCP leases: edit (make static fields) and remove; no copy.
pub const LEASE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_REMOVE_SELECTED];

/// Static ARP: add/edit/remove including dynamic rows.
pub const ARP_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_REMOVE_SELECTED];

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
    ("macsec", "MACsec"),
    ("macsec-profiles", "MACsec Profile"),
    ("interface-lists", "Interface List"),
    ("interface-list-members", "List Member"),
    ("vrf", "VRF"),
    ("wifi", "WiFi"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slave_and_dynamic_hide_remove() {
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        row.insert("slave".into(), "true".into());
        let ids: Vec<_> = resolve_actions(INTERFACE_LIST_ACTIONS, false, Some(&row))
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(!ids.contains(&"remove"));
        assert!(ids.contains(&"edit"));
        assert!(ids.contains(&"torch"));
    }

    #[test]
    fn ethernet_has_no_add_or_remove() {
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let ids: Vec<_> = resolve_actions(ETHERNET_ACTIONS, false, Some(&row))
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(!ids.contains(&"add"));
        assert!(!ids.contains(&"remove"));
        assert!(ids.contains(&"torch"));
        assert!(ids.contains(&"edit"));
    }

    #[test]
    fn toggle_label_follows_disabled() {
        let mut row = HashMap::new();
        row.insert("disabled".into(), "true".into());
        assert_eq!(action_label(&ACTION_TOGGLE, Some(&row)), "Enable");
        row.insert("disabled".into(), "false".into());
        assert_eq!(action_label(&ACTION_TOGGLE, Some(&row)), "Disable");
    }

    #[test]
    fn wireguard_peer_dynamic_hides_remove_and_copy() {
        let mut row = HashMap::new();
        row.insert("interface".into(), "wg1".into());
        row.insert("dynamic".into(), "true".into());
        let ids: Vec<_> = resolve_actions(MEMBER_ACTIONS, false, Some(&row))
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(!ids.contains(&"remove"));
        assert!(!ids.contains(&"copy"));
        assert!(ids.contains(&"edit"));
        assert!(ids.contains(&"add"));
    }
}
