//! Per-resource entity actions.

use std::collections::HashMap;

/// How an action is carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    /// Open the properties sheet for the selected row.
    Edit,
    /// Open a create sheet (or a type picker when `overlay` is `create-type`).
    Create,
    /// Confirm, then run an API command or delete.
    Confirm { command: ActionCommand },
    /// Prompt for extra fields, then run an API command.
    Prompt { command: ActionCommand },
    /// Open a dedicated overlay (`torch`, `create-type`).
    Overlay { id: &'static str },
}

/// API command word (or delete) after confirmation / prompt.
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
    Flush,
    Run,
    Release,
    Login,
    Bypass,
    Upgrade,
    UsbPowerReset,
    Install,
    ResetConfiguration,
    Export,
    CheckForUpdates,
    Start,
    Stop,
    Restart,
    Kill,
    ContainerUpdate,
    Repull,
    WakeOnLan,
    SendSms,
    AtChat,
    Undo,
    Format,
    Eject,
}

impl ActionCommand {
    #[must_use]
    pub fn api_name(self) -> &'static str {
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
            Self::Flush => "flush",
            Self::Run => "run",
            Self::Release => "release",
            Self::Login => "login",
            Self::Bypass => "bypass",
            Self::Upgrade => "upgrade",
            Self::UsbPowerReset => "usb-power-reset",
            Self::Install => "install",
            Self::ResetConfiguration => "reset-configuration",
            Self::Export => "export",
            Self::CheckForUpdates => "check-for-updates",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Kill => "kill",
            Self::ContainerUpdate => "update",
            Self::Repull => "repull",
            Self::WakeOnLan => "wol",
            Self::SendSms => "send",
            Self::AtChat => "at-chat",
            Self::Undo => "undo",
            Self::Format => "format",
            Self::Eject => "eject",
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

pub const ACTION_UNDO: ActionSpec = ActionSpec {
    id: "undo",
    label: "Undo",
    key: Some('u'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Undo,
    },
    when: ActionWhen::HasSelection,
};

pub const HISTORY_ACTIONS: &[ActionSpec] = &[ACTION_UNDO];

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

#[allow(unused_imports)]
pub use crate::features::interfaces::actions::{
    ACTION_NEW_LIST, ACTION_NEW_LIST_MEMBER, ETHERNET_ACTIONS, INTERFACE_LIST_ACTIONS,
    INTERFACE_LIST_DEF_ACTIONS, INTERFACE_LIST_MEMBER_ACTIONS, LTE_ACTIONS, RADIO_ACTIONS,
    VRF_ACTIONS,
};

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

pub const ACTION_SCAN: ActionSpec = ActionSpec {
    id: "scan",
    label: "Scan",
    key: Some('s'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Overlay { id: "wifi-scan" },
    when: ActionWhen::HasSelection,
};

pub const ACTION_AT_CHAT: ActionSpec = ActionSpec {
    id: "at-chat",
    label: "AT chat",
    key: Some('t'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::AtChat,
    },
    when: ActionWhen::HasSelection,
};

pub const SINGLETON_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT];

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

pub const ACTION_FILE_DOWNLOAD: ActionSpec = ActionSpec {
    id: "download",
    label: "Download",
    key: Some('d'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Download,
    },
    when: ActionWhen::HasSelection,
};

/// Files: backup, workstation transfer, fetch URL, export/import, and remove.
pub const FILE_ACTIONS: &[ActionSpec] = &[
    ACTION_BACKUP_SAVE,
    ACTION_BACKUP_LOAD,
    ACTION_FILE_UPLOAD,
    ACTION_FILE_DOWNLOAD,
    ACTION_FILE_FETCH,
    ACTION_EXPORT_CONFIG,
    ACTION_IMPORT_CONFIG,
    ACTION_REMOVE_SELECTED,
];

/// Disconnect a live session or drop an FDB/lease row.
pub const DISCONNECT_ACTIONS: &[ActionSpec] = &[ACTION_REMOVE_SELECTED];

/// Open a new device tab from an `/ip/neighbor` row (`WinBox` Neighbors → Connect).
pub const ACTION_CONNECT_NEIGHBOR: ActionSpec = ActionSpec {
    id: "connect",
    label: "Connect",
    key: Some('c'),
    enter: true,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Overlay {
        id: "connect-neighbor",
    },
    when: ActionWhen::HasSelection,
};

/// IP neighbors: connect a new tab, or drop a discovery row.
pub const NEIGHBOR_ACTIONS: &[ActionSpec] = &[ACTION_CONNECT_NEIGHBOR, ACTION_REMOVE_SELECTED];

/// Prefill for a new device tab from an `/ip/neighbor` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborConnectTarget {
    pub host: String,
    pub name: String,
}

/// Host from `address` / `address6`, name from identity or MAC.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn neighbor_connect_target(row: &HashMap<String, String>) -> Option<NeighborConnectTarget> {
    let host = neighbor_host(row).unwrap_or_default();
    let name = first_row_value(row, &["identity", "mac-address"]).unwrap_or_else(|| host.clone());
    if host.is_empty() && name.is_empty() {
        return None;
    }
    Some(NeighborConnectTarget { host, name })
}

fn neighbor_host(row: &HashMap<String, String>) -> Option<String> {
    for key in ["address", "address4", "address6"] {
        if let Some(host) = row.get(key).and_then(|raw| first_host_token(raw)) {
            return Some(host);
        }
    }
    None
}

fn first_row_value(row: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        row.get(*key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

fn first_host_token(raw: &str) -> Option<String> {
    raw.split([',', ' ', '\t', ';'])
        .map(str::trim)
        .find(|part| !part.is_empty())
        .map(|part| part.split('/').next().unwrap_or(part).trim().to_string())
        .filter(|part| !part.is_empty())
}

/// Hardware switch chip: edit only (no add/remove).
pub const HARDWARE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT];

/// Enable/disable without add (packages, some system lists).
pub const TOGGLE_EDIT_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE];

pub const ACTION_RELEASE: ActionSpec = ActionSpec {
    id: "release",
    label: "Release",
    key: Some('r'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Release,
    },
    when: ActionWhen::HasSelection,
};

/// DHCP leases: edit, convert dynamic → static, release, and remove; no copy.
pub const LEASE_ACTIONS: &[ActionSpec] = &[
    ACTION_EDIT,
    ACTION_MAKE_STATIC,
    ACTION_RELEASE,
    ACTION_REMOVE_SELECTED,
];

pub const ACTION_RUN: ActionSpec = ActionSpec {
    id: "run",
    label: "Run",
    key: Some('r'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Run,
    },
    when: ActionWhen::HasSelection,
};

pub const SCRIPT_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RUN,
];

pub const SCHEDULER_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_COPY,
    ACTION_REMOVE,
    ACTION_RUN,
];

pub const ACTION_FLUSH: ActionSpec = ActionSpec {
    id: "flush",
    label: "Flush",
    key: Some('f'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Flush,
    },
    when: ActionWhen::Always,
};

pub const ACTION_FLUSH_SELECTED: ActionSpec = ActionSpec {
    id: "flush",
    label: "Flush",
    key: Some('f'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Flush,
    },
    when: ActionWhen::HasSelection,
};

pub const DNS_CACHE_ACTIONS: &[ActionSpec] = &[ACTION_FLUSH, ACTION_REMOVE_SELECTED];

pub const HOST_TABLE_ACTIONS: &[ActionSpec] = &[ACTION_FLUSH, ACTION_REMOVE_SELECTED];

pub const IPSEC_SA_ACTIONS: &[ActionSpec] = &[ACTION_FLUSH_SELECTED, ACTION_REMOVE_SELECTED];

pub const ACTION_LOGIN: ActionSpec = ActionSpec {
    id: "login",
    label: "Authenticate",
    key: Some('u'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Login,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_BYPASS: ActionSpec = ActionSpec {
    id: "bypass",
    label: "Bypass",
    key: Some('y'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Bypass,
    },
    when: ActionWhen::HasSelection,
};

pub const HOTSPOT_HOST_ACTIONS: &[ActionSpec] = &[
    ACTION_EDIT,
    ACTION_LOGIN,
    ACTION_BYPASS,
    ACTION_REMOVE_SELECTED,
];

pub const ACTION_UPGRADE: ActionSpec = ActionSpec {
    id: "upgrade",
    label: "Upgrade firmware",
    key: Some('u'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Upgrade,
    },
    when: ActionWhen::Always,
};

pub const ACTION_USB_POWER_RESET: ActionSpec = ActionSpec {
    id: "usb-power-reset",
    label: "USB power reset",
    key: Some('p'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Prompt {
        command: ActionCommand::UsbPowerReset,
    },
    when: ActionWhen::Always,
};

pub const ROUTERBOARD_ACTIONS: &[ActionSpec] = &[ACTION_UPGRADE, ACTION_USB_POWER_RESET];

pub const ACTION_FORMAT: ActionSpec = ActionSpec {
    id: "format",
    label: "Format",
    key: Some('f'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Prompt {
        command: ActionCommand::Format,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_EJECT: ActionSpec = ActionSpec {
    id: "eject",
    label: "Eject",
    key: None,
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Eject,
    },
    when: ActionWhen::HasSelection,
};

/// Disks: add/edit plus confirmed format and eject. RAID knobs stay on the sheet.
pub const DISK_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_TOGGLE,
    ACTION_FORMAT,
    ACTION_EJECT,
    ACTION_REMOVE,
];

pub const ACTION_LICENSE_IMPORT: ActionSpec = ActionSpec {
    id: "import",
    label: "Apply license key",
    key: Some('p'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Prompt {
        command: ActionCommand::Import,
    },
    when: ActionWhen::Always,
};

pub const LICENSE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_LICENSE_IMPORT];

pub const ACTION_INSTALL: ActionSpec = ActionSpec {
    id: "install",
    label: "Install from file",
    key: Some('i'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Prompt {
        command: ActionCommand::Install,
    },
    when: ActionWhen::Always,
};

pub const PACKAGE_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_TOGGLE, ACTION_INSTALL];

pub const ACTION_CHECK_UPDATES: ActionSpec = ActionSpec {
    id: "check-for-updates",
    label: "Check for updates",
    key: Some('c'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::CheckForUpdates,
    },
    when: ActionWhen::Always,
};

pub const PACKAGE_UPDATE_ACTIONS: &[ActionSpec] =
    &[ACTION_EDIT, ACTION_CHECK_UPDATES, ACTION_INSTALL];

pub const ACTION_EXPORT_CONFIG: ActionSpec = ActionSpec {
    id: "export-config",
    label: "Export config",
    key: Some('e'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::Export,
    },
    when: ActionWhen::Always,
};

pub const ACTION_IMPORT_CONFIG: ActionSpec = ActionSpec {
    id: "import-config",
    label: "Import config",
    key: Some('i'),
    enter: false,
    needs_selection: false,
    danger: true,
    kind: ActionKind::Prompt {
        command: ActionCommand::Import,
    },
    when: ActionWhen::Always,
};

pub const ACTION_START: ActionSpec = ActionSpec {
    id: "start",
    label: "Start",
    key: Some('s'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Start,
    },
    when: ActionWhen::Always,
};

pub const ACTION_STOP: ActionSpec = ActionSpec {
    id: "stop",
    label: "Stop",
    key: Some('p'),
    enter: false,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Stop,
    },
    when: ActionWhen::Always,
};

pub const ACTION_CONTAINER_START: ActionSpec = ActionSpec {
    id: "start",
    label: "Start",
    key: Some('s'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Start,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_CONTAINER_STOP: ActionSpec = ActionSpec {
    id: "stop",
    label: "Stop",
    key: Some('p'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Stop,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_CONTAINER_RESTART: ActionSpec = ActionSpec {
    id: "restart",
    label: "Restart",
    key: Some('r'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Restart,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_CONTAINER_KILL: ActionSpec = ActionSpec {
    id: "kill",
    label: "Kill",
    key: Some('k'),
    enter: false,
    needs_selection: true,
    danger: true,
    kind: ActionKind::Confirm {
        command: ActionCommand::Kill,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_CONTAINER_UPDATE: ActionSpec = ActionSpec {
    id: "update",
    label: "Update image",
    key: Some('u'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::ContainerUpdate,
    },
    when: ActionWhen::HasSelection,
};

pub const ACTION_CONTAINER_REPULL: ActionSpec = ActionSpec {
    id: "repull",
    label: "Repull",
    key: Some('l'),
    enter: false,
    needs_selection: true,
    danger: false,
    kind: ActionKind::Confirm {
        command: ActionCommand::Repull,
    },
    when: ActionWhen::HasSelection,
};

pub const CONTAINER_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_REMOVE,
    ACTION_CONTAINER_START,
    ACTION_CONTAINER_STOP,
    ACTION_CONTAINER_RESTART,
    ACTION_CONTAINER_KILL,
    ACTION_CONTAINER_UPDATE,
    ACTION_CONTAINER_REPULL,
];

pub const APP_ACTIONS: &[ActionSpec] = &[
    ACTION_ADD,
    ACTION_EDIT,
    ACTION_REMOVE,
    ACTION_CONTAINER_START,
    ACTION_CONTAINER_STOP,
];

pub const SNIFFER_ACTIONS: &[ActionSpec] = &[ACTION_EDIT, ACTION_START, ACTION_STOP];

pub const ACTION_BANDWIDTH: ActionSpec = ActionSpec {
    id: "bandwidth-test",
    label: "Bandwidth test",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay {
        id: "bandwidth-test",
    },
    when: ActionWhen::Always,
};

pub const ACTION_FLOOD_PING: ActionSpec = ActionSpec {
    id: "flood-ping",
    label: "Flood ping",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "flood-ping" },
    when: ActionWhen::Always,
};

pub const ACTION_MAC_SCAN: ActionSpec = ActionSpec {
    id: "mac-scan",
    label: "MAC scan",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "mac-scan" },
    when: ActionWhen::Always,
};

pub const ACTION_IP_SCAN: ActionSpec = ActionSpec {
    id: "ip-scan",
    label: "IP scan",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "ip-scan" },
    when: ActionWhen::Always,
};

pub const ACTION_PROFILER: ActionSpec = ActionSpec {
    id: "profiler",
    label: "Profiler",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Overlay { id: "profiler" },
    when: ActionWhen::Always,
};

pub const ACTION_WOL: ActionSpec = ActionSpec {
    id: "wol",
    label: "Wake on LAN",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::WakeOnLan,
    },
    when: ActionWhen::Always,
};

pub const ACTION_SMS: ActionSpec = ActionSpec {
    id: "sms",
    label: "Send SMS",
    key: Some('p'),
    enter: true,
    needs_selection: false,
    danger: false,
    kind: ActionKind::Prompt {
        command: ActionCommand::SendSms,
    },
    when: ActionWhen::Always,
};

pub const BANDWIDTH_ACTIONS: &[ActionSpec] = &[ACTION_BANDWIDTH];
pub const FLOOD_PING_ACTIONS: &[ActionSpec] = &[ACTION_FLOOD_PING];
pub const MAC_SCAN_ACTIONS: &[ActionSpec] = &[ACTION_MAC_SCAN];
pub const IP_SCAN_ACTIONS: &[ActionSpec] = &[ACTION_IP_SCAN];
pub const PROFILER_ACTIONS: &[ActionSpec] = &[ACTION_PROFILER];
pub const WOL_ACTIONS: &[ActionSpec] = &[ACTION_WOL];
pub const SMS_ACTIONS: &[ActionSpec] = &[ACTION_SMS];

/// Static ARP: add/edit/remove including dynamic rows.
pub const ARP_ACTIONS: &[ActionSpec] = &[ACTION_ADD, ACTION_EDIT, ACTION_REMOVE_SELECTED];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::interfaces::actions::INTERFACE_CREATE_TARGETS;

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
    fn neighbor_connect_target_table() {
        struct Case {
            name: &'static str,
            fields: &'static [(&'static str, &'static str)],
            want: Option<NeighborConnectTarget>,
        }
        let cases = [
            Case {
                name: "ipv4_and_identity",
                fields: &[
                    ("address", "192.168.88.2,fe80::1"),
                    ("identity", "core-sw"),
                    ("mac-address", "4C:5E:0C:00:00:01"),
                ],
                want: Some(NeighborConnectTarget {
                    host: "192.168.88.2".into(),
                    name: "core-sw".into(),
                }),
            },
            Case {
                name: "address6_cidr_mac_name",
                fields: &[
                    ("address6", "2001:db8::2/64"),
                    ("mac-address", "4C:5E:0C:00:00:02"),
                ],
                want: Some(NeighborConnectTarget {
                    host: "2001:db8::2".into(),
                    name: "4C:5E:0C:00:00:02".into(),
                }),
            },
            Case {
                name: "address4_key",
                fields: &[("address4", "10.0.0.9"), ("identity", "ap")],
                want: Some(NeighborConnectTarget {
                    host: "10.0.0.9".into(),
                    name: "ap".into(),
                }),
            },
            Case {
                name: "whitespace_and_semicolon",
                fields: &[("address", "  ;  10.1.1.1  "), ("identity", " edge ")],
                want: Some(NeighborConnectTarget {
                    host: "10.1.1.1".into(),
                    name: "edge".into(),
                }),
            },
            Case {
                name: "ipv6_flag_is_not_a_host",
                fields: &[("ipv6", "true"), ("mac-address", "AA:BB:CC:DD:EE:FF")],
                want: Some(NeighborConnectTarget {
                    host: String::new(),
                    name: "AA:BB:CC:DD:EE:FF".into(),
                }),
            },
            Case {
                name: "empty",
                fields: &[],
                want: None,
            },
            Case {
                name: "blank_fields",
                fields: &[("address", "   "), ("identity", ""), ("mac-address", "\t")],
                want: None,
            },
            Case {
                name: "malformed_slashes_only",
                fields: &[("address", "///")],
                want: None,
            },
        ];
        for case in cases {
            let mut row = HashMap::new();
            for (key, value) in case.fields {
                row.insert((*key).to_string(), (*value).to_string());
            }
            assert_eq!(neighbor_connect_target(&row), case.want, "{}", case.name);
        }
        let ids: Vec<_> = NEIGHBOR_ACTIONS.iter().map(|action| action.id).collect();
        assert_eq!(ids, ["connect", "remove"]);
        const { assert!(ACTION_CONNECT_NEIGHBOR.enter) };
        assert_eq!(ACTION_CONNECT_NEIGHBOR.key, Some('c'));
        assert_eq!(
            ACTION_CONNECT_NEIGHBOR.kind,
            ActionKind::Overlay {
                id: "connect-neighbor"
            }
        );
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
        assert_eq!(ActionCommand::MoveUp.api_name(), "move");
        assert_eq!(ActionCommand::MoveDown.api_name(), "move");
    }

    #[test]
    fn lease_actions_include_make_static() {
        let ids: Vec<_> = LEASE_ACTIONS.iter().map(|action| action.id).collect();
        assert!(ids.contains(&"make-static"));
        assert_eq!(ActionCommand::MakeStatic.api_name(), "make-static");
        assert!(LEASE_ACTIONS.iter().any(|action| action.id == "release"));
        assert_eq!(ActionCommand::Release.api_name(), "release");
        assert!(SCRIPT_ACTIONS.iter().any(|action| action.id == "run"));
        assert!(RADIO_ACTIONS.iter().any(|action| action.id == "scan"));
        assert_eq!(ActionCommand::Flush.api_name(), "flush");
        assert_eq!(ActionCommand::Upgrade.api_name(), "upgrade");
        assert_eq!(ActionCommand::Restart.api_name(), "restart");
        assert_eq!(ActionCommand::Kill.api_name(), "kill");
        assert_eq!(ActionCommand::ContainerUpdate.api_name(), "update");
        assert_eq!(ActionCommand::Repull.api_name(), "repull");
    }

    #[test]
    fn history_undo_needs_a_selected_row() {
        assert_eq!(ActionCommand::Undo.api_name(), "undo");
        let history: Vec<_> = resolve_actions(HISTORY_ACTIONS, false, None)
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(history.is_empty());
        let mut row = HashMap::new();
        row.insert("action".into(), "set".into());
        let with_row: Vec<_> = resolve_actions(HISTORY_ACTIONS, false, Some(&row))
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(with_row, ["undo"]);
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
        assert_eq!(ActionCommand::Sign.api_name(), "sign");
        assert_eq!(ActionCommand::Import.api_name(), "import");
        assert_eq!(
            ActionCommand::ExportCertificate.api_name(),
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
    fn files_actions_document_fetch_backup_and_remove() {
        let keys: Vec<_> = FILE_ACTIONS
            .iter()
            .filter_map(|action| action.key)
            .collect();
        assert_eq!(keys, ['b', 'u', 'd', 'f', 'e', 'i', 'x']);
        let ids: Vec<_> = FILE_ACTIONS.iter().map(|action| action.id).collect();
        assert_eq!(
            ids,
            [
                "backup-save",
                "backup-load",
                "upload",
                "download",
                "fetch",
                "export-config",
                "import-config",
                "remove"
            ]
        );
        assert!(!FILE_ACTIONS[2].needs_selection);
        assert!(FILE_ACTIONS[3].needs_selection);
        assert!(!FILE_ACTIONS[4].needs_selection);
        assert!(FILE_ACTIONS[7].needs_selection);
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
