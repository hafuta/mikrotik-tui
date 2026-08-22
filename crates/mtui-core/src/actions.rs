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
    Reboot,
    Shutdown,
    BackupSave,
    BackupLoad,
    MoveUp,
    MoveDown,
    MakeStatic,
    Upload,
    Download,
    Fetch,
    Sign,
    Import,
    ExportCertificate,
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
            Self::Reboot => "reboot",
            Self::Shutdown => "shutdown",
            Self::BackupSave => "save",
            Self::BackupLoad => "load",
            Self::MoveUp | Self::MoveDown => "move",
            Self::MakeStatic => "make-static",
            Self::Upload => "upload",
            Self::Download => "download",
            Self::Fetch => "fetch",
            Self::Sign => "sign",
            Self::Import => "import",
            Self::ExportCertificate => "export-certificate",
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
#[allow(clippy::implicit_hasher)]
pub fn is_backup_file(row: &HashMap<String, String>) -> bool {
    row.get("name")
        .is_some_and(|name| name.rsplit('/').next().unwrap_or(name).ends_with(".backup"))
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
    if action.id == "backup-load" {
        return row.is_some_and(is_backup_file);
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

pub const ACTION_REBOOT: ActionSpec = ActionSpec {
    id: "reboot",
    label: "Reboot",
    key: Some('b'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Reboot,
    },
    when: ActionWhen::Always,
};

pub const ACTION_SHUTDOWN: ActionSpec = ActionSpec {
    id: "shutdown",
    label: "Shutdown",
    key: Some('o'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Shutdown,
    },
    when: ActionWhen::Always,
};

pub const ACTION_BACKUP_SAVE: ActionSpec = ActionSpec {
    id: "backup-save",
    label: "Save backup",
    key: Some('b'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::BackupSave,
    },
    when: ActionWhen::Always,
};

pub const ACTION_BACKUP_LOAD: ActionSpec = ActionSpec {
    id: "backup-load",
    label: "Load backup",
    key: None,
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::BackupLoad,
    },
    when: ActionWhen::HasSelection,
};

pub const RESOURCE_LIFECYCLE_ACTIONS: &[ActionSpec] = &[ACTION_REBOOT, ACTION_SHUTDOWN];

pub const ACTION_MOVE_UP: ActionSpec = ActionSpec {
    id: "move-up",
    label: "Move up",
    key: Some('['),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::MoveUp,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_MOVE_DOWN: ActionSpec = ActionSpec {
    id: "move-down",
    label: "Move down",
    key: Some(']'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::MoveDown,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_MAKE_STATIC: ActionSpec = ActionSpec {
    id: "make-static",
    label: "Make static",
    key: Some('m'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::MakeStatic,
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

pub const ACTION_PING: ActionSpec = ActionSpec {
    id: "ping",
    label: "Ping",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "ping" },
    when: ActionWhen::Always,
};

pub const ACTION_TRACEROUTE: ActionSpec = ActionSpec {
    id: "traceroute",
    label: "Traceroute",
    key: None,
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "traceroute" },
    when: ActionWhen::Always,
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

pub const PING_ACTIONS: &[ActionSpec] = &[ACTION_PING];

pub const TRACEROUTE_ACTIONS: &[ActionSpec] = &[ACTION_TRACEROUTE];

pub const VIRTUAL_IFACE_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RESET,
];

pub const LIST_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

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

/// Interface list *definitions* (`/interface/list`). Do not reuse for other `LIST_ACTIONS` screens.
pub const INTERFACE_LIST_DEF_ACTIONS: &[ActionSpec] =
    &[ACTION_NEW_LIST, ACTION_EDIT, ACTION_COPY, ACTION_REMOVE];

/// Interface list membership (`/interface/list/member`).
pub const INTERFACE_LIST_MEMBER_ACTIONS: &[ActionSpec] = &[
    ACTION_NEW_LIST_MEMBER,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
];

pub const ACTION_SIGN: ActionSpec = ActionSpec {
    id: "sign",
    label: "Sign",
    key: Some('g'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Sign,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_IMPORT: ActionSpec = ActionSpec {
    id: "import",
    label: "Import",
    key: Some('p'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Import,
    },
    when: ActionWhen::Always,
};

pub const ACTION_EXPORT_CERT: ActionSpec = ActionSpec {
    id: "export",
    label: "Export",
    key: Some('w'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::ExportCertificate,
    },
    when: ActionWhen::HasSelection,
};

pub const CERTIFICATE_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_SIGN,
    ACTION_IMPORT,
    ACTION_EXPORT_CERT,
];

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
    ACTION_MOVE_UP,
    ACTION_MOVE_DOWN,
    ACTION_REMOVE,
    ACTION_RESET,
];

pub const ACTION_FILE_UPLOAD: ActionSpec = ActionSpec {
    id: "upload",
    label: "Upload",
    key: Some('u'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Upload,
    },
    when: ActionWhen::Always,
};

pub const ACTION_FILE_FETCH: ActionSpec = ActionSpec {
    id: "fetch",
    label: "Fetch URL",
    key: Some('f'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Fetch,
    },
    when: ActionWhen::Always,
};

pub const ACTION_FILE_DOWNLOAD: ActionSpec = ActionSpec {
    id: "download",
    label: "Download",
    key: Some('w'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Download,
    },
    when: ActionWhen::HasSelection,
};

/// Files: backup, upload/download/fetch, and remove. No property sheet.
pub const FILE_ACTIONS: &[ActionSpec] = &[
    ACTION_BACKUP_SAVE,
    ACTION_BACKUP_LOAD,
    ACTION_FILE_UPLOAD,
    ACTION_FILE_FETCH,
    ACTION_FILE_DOWNLOAD,
    ACTION_REMOVE_SELECTED,
];

/// Disconnect a live session or drop an FDB/lease row.
pub const DISCONNECT_ACTIONS: &[ActionSpec] = &[ACTION_REMOVE_SELECTED];

/// Hardware switch chip: edit only (no add/remove).
pub const HARDWARE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT];

/// Enable/disable without add (packages, some system lists).
pub const TOGGLE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE];

/// DHCP leases: edit, convert dynamic → static, and remove; no copy.
pub const LEASE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_MAKE_STATIC, ACTION_REMOVE_SELECTED];

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
    ("interface-lists", "Lists"),
    ("interface-list-members", "List members"),
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
    fn filter_actions_include_move() {
        let ids: Vec<_> = FILTER_ACTIONS.iter().map(|action| action.id).collect();
        assert!(ids.contains(&"move-up"));
        assert!(ids.contains(&"move-down"));
        assert_eq!(ActionCommand::MoveUp.rest_name(), "move");
        assert_eq!(ActionCommand::MoveDown.rest_name(), "move");
    }

    #[test]
    fn lease_actions_include_make_static() {
        let ids: Vec<_> = LEASE_ACTIONS.iter().map(|action| action.id).collect();
        assert!(ids.contains(&"make-static"));
        assert_eq!(ActionCommand::MakeStatic.rest_name(), "make-static");
    }

    #[test]
    fn certificate_import_does_not_require_selection() {
        let ids: Vec<_> = resolve_actions(CERTIFICATE_ACTIONS, false, None)
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(ids.contains(&"import"));
        assert!(ids.contains(&"add"));
        assert!(!ids.contains(&"sign"));
        assert!(!ids.contains(&"export"));
        assert!(!ids.contains(&"copy"));

        let mut row = HashMap::new();
        row.insert("name".into(), "web".into());
        let with_row: Vec<_> = resolve_actions(CERTIFICATE_ACTIONS, false, Some(&row))
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            with_row,
            ["add", "edit", "copy", "remove", "sign", "import", "export"]
        );
        assert_eq!(ActionCommand::Sign.rest_name(), "sign");
        assert_eq!(ActionCommand::Import.rest_name(), "import");
        assert_eq!(
            ActionCommand::ExportCertificate.rest_name(),
            "export-certificate"
        );
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

    #[test]
    fn backup_load_only_for_backup_files() {
        let mut backup = HashMap::new();
        backup.insert("name".into(), "flash/foo.backup".into());
        let mut other = HashMap::new();
        other.insert("name".into(), "script.rsc".into());
        let with_backup: Vec<_> = resolve_actions(FILE_ACTIONS, false, Some(&backup))
            .iter()
            .map(|action| action.id)
            .collect();
        let with_other: Vec<_> = resolve_actions(FILE_ACTIONS, false, Some(&other))
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(with_backup.contains(&"backup-load"));
        assert!(!with_other.contains(&"backup-load"));
        assert!(with_other.contains(&"backup-save"));
        assert!(
            resolve_actions(FILE_ACTIONS, false, None)
                .iter()
                .any(|action| action.id == "backup-save")
        );
    }

    #[test]
    fn resource_lifecycle_without_selection() {
        let ids: Vec<_> = resolve_actions(RESOURCE_LIFECYCLE_ACTIONS, true, None)
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(ids, ["reboot", "shutdown"]);
    }

    #[test]
    fn files_actions_document_u_f_w_and_remove() {
        let keys: Vec<_> = FILE_ACTIONS
            .iter()
            .filter_map(|action| action.key)
            .collect();
        assert_eq!(keys, ['b', 'u', 'f', 'w', 'x']);
        let ids: Vec<_> = FILE_ACTIONS.iter().map(|action| action.id).collect();
        assert_eq!(
            ids,
            [
                "backup-save",
                "backup-load",
                "upload",
                "fetch",
                "download",
                "remove"
            ]
        );
        assert!(!FILE_ACTIONS[2].needs_selection);
        assert!(!FILE_ACTIONS[3].needs_selection);
        assert!(FILE_ACTIONS[4].needs_selection);
        assert!(FILE_ACTIONS[5].needs_selection);
    }

    #[test]
    fn interface_list_add_labels_are_local_not_global() {
        assert_eq!(ACTION_ADD.label, "Add");
        assert_eq!(LIST_ACTIONS[0].label, "Add");
        assert_eq!(MEMBER_ACTIONS[0].label, "Add");
        assert_eq!(INTERFACE_LIST_DEF_ACTIONS[0].id, "add");
        assert_eq!(INTERFACE_LIST_DEF_ACTIONS[0].key, Some('n'));
        assert_eq!(INTERFACE_LIST_DEF_ACTIONS[0].kind, ActionKind::Create);
        assert_eq!(INTERFACE_LIST_DEF_ACTIONS[0].label, "New list");
        assert_eq!(INTERFACE_LIST_MEMBER_ACTIONS[0].id, "add");
        assert_eq!(INTERFACE_LIST_MEMBER_ACTIONS[0].key, Some('n'));
        assert_eq!(INTERFACE_LIST_MEMBER_ACTIONS[0].kind, ActionKind::Create);
        assert_eq!(INTERFACE_LIST_MEMBER_ACTIONS[0].label, "New list member");
        assert!(
            INTERFACE_LIST_MEMBER_ACTIONS
                .iter()
                .any(|action| action.id == "toggle-disabled")
        );
        assert_eq!(
            INTERFACE_CREATE_TARGETS
                .iter()
                .copied()
                .find(|(id, _)| *id == "interface-lists")
                .map(|(_, label)| label),
            Some("Lists")
        );
        assert_eq!(
            INTERFACE_CREATE_TARGETS
                .iter()
                .copied()
                .find(|(id, _)| *id == "interface-list-members")
                .map(|(_, label)| label),
            Some("List members")
        );
    }
}
