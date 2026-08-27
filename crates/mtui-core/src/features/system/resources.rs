//! Feature-owned catalog entries for the System navigation group.

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
    USERS,
    SPECIAL_LOGIN,
    ROUTERBOARD,
    ROUTERBOARD_SETTINGS,
    ROUTERBOARD_MODE_BUTTON,
    ROUTERBOARD_RESET_BUTTON,
    NTP,
    NTP_SERVER,
    NTP_KEYS,
    CLOCK,
    LICENSE,
    DISKS,
    DEVICE_MODE,
    USER_GROUPS,
    IDENTITY,
    RESOURCES_SCREEN,
    HEALTH,
    PACKAGES,
    PACKAGE_UPDATE,
    RESET_CONFIGURATION,
    REBOOT,
    SHUTDOWN,
    SSH_KEYS,
    HISTORY,
    SCHEDULER,
    SCRIPTS,
    LOGGING,
    LOGGING_ACTIONS,
    SYSTEM_CONSOLE,
    LEDS,
    LED_SETTINGS,
    PORTS,
    SNMP,
    SNMP_COMMUNITIES,
    CERTIFICATES,
    WATCHDOG,
    NOTE,
    LOGS,
];

const USERS: ResourceSpec = ResourceSpec {
    id: "users",
    group: "system-group",
    cli_path: Some("/user"),
    label: "Users",
    fetch: FetchKind::List { endpoint: "/user" },
    columns: &[
        col!("name", "Name", 18),
        col!("group", "Group", 14),
        col!("last-logged-in", "Last login", 22),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::system::forms::USER_FORM),
};

const SPECIAL_LOGIN: ResourceSpec = ResourceSpec {
    id: "special-login",
    group: "system-group",
    cli_path: Some("/special-login"),
    label: "Special Login",
    fetch: FetchKind::List {
        endpoint: "/special-login",
    },
    columns: &[
        col!("user", "User", 16),
        col!("port", "Port", 14),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::system::forms::SPECIAL_LOGIN_FORM),
};

const ROUTERBOARD: ResourceSpec = ResourceSpec {
    id: "routerboard",
    group: "system-group",
    cli_path: None,
    label: "RouterBOARD",
    fetch: FetchKind::System {
        endpoint: "/system/routerboard",
    },
    columns: &[
        col!("model", "Model", 18),
        col!("serial-number", "Serial", 18),
        col!("current-firmware", "Current", 12),
        col!("upgrade-firmware", "Upgrade", 12),
    ],
    refresh: Duration::from_secs(60),
    actions: crate::actions::ROUTERBOARD_ACTIONS,
    form: None,
};

const ROUTERBOARD_SETTINGS: ResourceSpec = ResourceSpec {
    id: "routerboard-settings",
    group: "system-group",
    cli_path: Some("/system/routerboard/settings"),
    label: "RouterBOARD Settings",
    fetch: FetchKind::System {
        endpoint: "/system/routerboard/settings",
    },
    columns: &[
        col!("boot-device", "Boot device", 28),
        col!("boot-os", "Boot OS", 12),
        col!("auto-upgrade", "Auto", 6),
    ],
    refresh: Duration::from_secs(60),
    actions: &[],
    form: Some(&crate::features::system::forms::ROUTERBOARD_SETTINGS_FORM),
};

const ROUTERBOARD_MODE_BUTTON: ResourceSpec = ResourceSpec {
    id: "routerboard-mode-button",
    group: "system-group",
    cli_path: Some("/system/routerboard/mode-button"),
    label: "Mode Button",
    fetch: FetchKind::System {
        endpoint: "/system/routerboard/mode-button",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("hold-time", "Hold", 10),
        col!("on-event", "On event", 24),
    ],
    refresh: Duration::from_secs(60),
    actions: &[],
    form: Some(&crate::features::system::forms::ROUTERBOARD_MODE_BUTTON_FORM),
};

const ROUTERBOARD_RESET_BUTTON: ResourceSpec = ResourceSpec {
    id: "routerboard-reset-button",
    group: "system-group",
    cli_path: Some("/system/routerboard/reset-button"),
    label: "Reset Button",
    fetch: FetchKind::System {
        endpoint: "/system/routerboard/reset-button",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("hold-time", "Hold", 10),
        col!("on-event", "On event", 24),
    ],
    refresh: Duration::from_secs(60),
    actions: &[],
    form: Some(&crate::features::system::forms::ROUTERBOARD_RESET_BUTTON_FORM),
};

const NTP: ResourceSpec = ResourceSpec {
    id: "ntp",
    group: "system-group",
    cli_path: None,
    label: "NTP Client",
    fetch: FetchKind::System {
        endpoint: "/system/ntp/client",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("mode", "Mode", 12),
        col!("servers", "Servers", 28),
        col!("status", "Status", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::NTP_CLIENT_FORM),
};

const NTP_SERVER: ResourceSpec = ResourceSpec {
    id: "ntp-server",
    group: "system-group",
    cli_path: None,
    label: "NTP Server",
    fetch: FetchKind::System {
        endpoint: "/system/ntp/server",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("broadcast", "Broadcast", 10),
        col!("multicast", "Multicast", 10),
        col!("manycast", "Manycast", 10),
        col!("vrf", "VRF", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::NTP_SERVER_FORM),
};

const NTP_KEYS: ResourceSpec = ResourceSpec {
    id: "ntp-keys",
    group: "system-group",
    cli_path: None,
    label: "NTP Keys",
    fetch: FetchKind::List {
        endpoint: "/system/ntp/key",
    },
    columns: &[col!("key-id", "Key ID", 10)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::system::forms::NTP_KEY_FORM),
};

const CLOCK: ResourceSpec = ResourceSpec {
    id: "clock",
    group: "system-group",
    cli_path: None,
    label: "Clock",
    fetch: FetchKind::System {
        endpoint: "/system/clock",
    },
    columns: &[
        col!("time", "Time", 12),
        col!("date", "Date", 14),
        col!("time-zone-name", "Time zone", 22),
        col!("gmt-offset", "Offset", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::CLOCK_FORM),
};

const LICENSE: ResourceSpec = ResourceSpec {
    id: "license",
    group: "system-group",
    cli_path: None,
    label: "License",
    fetch: FetchKind::System {
        endpoint: "/system/license",
    },
    columns: &[
        col!("software-id", "Software ID", 14),
        col!("nlevel", "Level", 7),
        col!("system-id", "System ID", 20),
        col!("level", "CHR level", 12),
        col!("features", "Features", 18),
        col!("next-renewal-at", "Renewal", 20),
        col!("deadline-at", "Deadline", 20),
        col!("expires-in", "Expires", 12),
    ],
    refresh: Duration::from_secs(60),
    actions: crate::actions::LICENSE_ACTIONS,
    form: Some(&crate::features::system::forms::LICENSE_FORM),
};

const DISKS: ResourceSpec = ResourceSpec {
    id: "disks",
    group: "system-group",
    cli_path: Some("/disk"),
    label: "Disks",
    fetch: FetchKind::List { endpoint: "/disk" },
    columns: &[
        col!("slot", "Slot", 12),
        col!("type", "Type", 12),
        col!("model", "Model", 18),
        col!("serial", "Serial", 16),
        col!("size", "Size", 12),
        col!("free", "Free", 12),
        col!("fs", "FS", 10),
        col!("raid-type", "RAID", 8),
        col!("raid-role", "Role", 6),
        col!("state", "State", 12),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::DISK_ACTIONS,
    form: Some(&crate::features::system::forms::DISK_FORM),
};

const DEVICE_MODE: ResourceSpec = ResourceSpec {
    id: "device-mode",
    group: "system-group",
    cli_path: None,
    label: "Device Mode",
    fetch: FetchKind::System {
        endpoint: "/system/device-mode",
    },
    columns: &[
        col!("mode", "Mode", 12),
        col!("flagged", "Flagged", 8),
        col!("container", "Container", 10),
        col!("scheduler", "Scheduler", 10),
        col!("traffic-gen", "Traffic gen", 12),
        col!("fetch", "Fetch", 7),
        col!("install-any-version", "Any ver", 8),
        col!("attempt-count", "Attempts", 9),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::DEVICE_MODE_FORM),
};

const USER_GROUPS: ResourceSpec = ResourceSpec {
    id: "user-groups",
    group: "system-group",
    cli_path: Some("/user/group"),
    label: "User Groups",
    fetch: FetchKind::List {
        endpoint: "/user/group",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("policy", "Policy", 36),
        col!("skin", "Skin", 12),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::system::forms::USER_GROUP_FORM),
};

const IDENTITY: ResourceSpec = ResourceSpec {
    id: "identity",
    group: "system-group",
    cli_path: None,
    label: "Identity",
    fetch: FetchKind::System {
        endpoint: "/system/identity",
    },
    columns: &[col!("name", "Name", 28)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::IDENTITY_FORM),
};

const RESOURCES_SCREEN: ResourceSpec = ResourceSpec {
    id: "resources",
    group: "system-group",
    cli_path: None,
    label: "Resources",
    fetch: FetchKind::System {
        endpoint: "/system/resource",
    },
    columns: &[
        col!("uptime", "Uptime", 12),
        col!("version", "Version", 16),
        col!("build-time", "Build", 20),
        col!("cpu-load", "CPU", 6),
        col!("free-memory", "Free mem", 12),
        col!("total-memory", "Total mem", 12),
        col!("cpu-count", "CPUs", 6),
        col!("board-name", "Board", 18),
        col!("architecture-name", "Arch", 10),
    ],
    refresh: Duration::from_secs(5),
    actions: &[],
    form: None,
};

const HEALTH: ResourceSpec = ResourceSpec {
    id: "health",
    group: "system-group",
    cli_path: None,
    label: "Health",
    fetch: FetchKind::List {
        endpoint: "/system/health",
    },
    columns: &[
        col!("name", "Name", 20),
        col!("value", "Value", 12),
        col!("type", "Type", 12),
    ],
    refresh: Duration::from_secs(10),
    actions: &[],
    form: None,
};

const PACKAGES: ResourceSpec = ResourceSpec {
    id: "packages",
    group: "system-group",
    cli_path: None,
    label: "Packages",
    fetch: FetchKind::List {
        endpoint: "/system/package",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("version", "Version", 14),
        col!("build-time", "Build", 20),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::PACKAGE_ACTIONS,
    form: Some(&crate::features::system::forms::PACKAGE_FORM),
};

const PACKAGE_UPDATE: ResourceSpec = ResourceSpec {
    id: "package-update",
    group: "system-group",
    cli_path: None,
    label: "Package Update",
    fetch: FetchKind::System {
        endpoint: "/system/package/update",
    },
    columns: &[
        col!("channel", "Channel", 12),
        col!("installed-version", "Installed", 14),
        col!("latest-version", "Latest", 14),
        col!("status", "Status", 16),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::PACKAGE_UPDATE_ACTIONS,
    form: Some(&crate::features::system::forms::PACKAGE_UPDATE_FORM),
};

const RESET_CONFIGURATION: ResourceSpec = ResourceSpec {
    id: "reset-configuration",
    group: "system-group",
    cli_path: Some("/system/reset-configuration"),
    label: "Reset Configuration",
    fetch: FetchKind::Local,
    columns: &[
        col!("keep-users", "Keep users", 12),
        col!("no-defaults", "No defaults", 12),
        col!("skip-backup", "Skip backup", 12),
        col!("caps-mode", "CAPs mode", 10),
    ],
    refresh: Duration::from_secs(3600),
    actions: &[],
    form: Some(&crate::features::system::forms::RESET_CONFIG_PROMPT),
};

const REBOOT: ResourceSpec = ResourceSpec {
    id: "reboot",
    group: "system-group",
    cli_path: Some("/system/reboot"),
    label: "Reboot",
    fetch: FetchKind::Local,
    columns: &[],
    refresh: Duration::from_secs(3600),
    actions: &[],
    form: None,
};

const SHUTDOWN: ResourceSpec = ResourceSpec {
    id: "shutdown",
    group: "system-group",
    cli_path: Some("/system/shutdown"),
    label: "Shutdown",
    fetch: FetchKind::Local,
    columns: &[],
    refresh: Duration::from_secs(3600),
    actions: &[],
    form: None,
};

const SSH_KEYS: ResourceSpec = ResourceSpec {
    id: "ssh-keys",
    group: "system-group",
    cli_path: Some("/user/ssh-keys"),
    label: "SSH Keys",
    fetch: FetchKind::List {
        endpoint: "/user/ssh-keys",
    },
    columns: &[col!("user", "User", 16), col!("key-owner", "Owner", 20)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::system::forms::SSH_KEY_FORM),
};

const HISTORY: ResourceSpec = ResourceSpec {
    id: "history",
    group: "system-group",
    cli_path: None,
    label: "History",
    fetch: FetchKind::List {
        endpoint: "/system/history",
    },
    columns: &[
        col!("floating-undo", "F", 3),
        col!("time", "Time", 20),
        col!("action", "Action", 12),
        col!("by", "By", 14),
        col!("policy", "Policy", 20),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::HISTORY_ACTIONS,
    form: None,
};

const SCHEDULER: ResourceSpec = ResourceSpec {
    id: "scheduler",
    group: "system-group",
    cli_path: None,
    label: "Scheduler",
    fetch: FetchKind::List {
        endpoint: "/system/scheduler",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("start-date", "Start date", 12),
        col!("start-time", "Start time", 12),
        col!("interval", "Interval", 12),
        col!("on-event", "On event", 24),
        col!("next-run", "Next", 16),
        col!("run-count", "Runs", 8),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SCHEDULER_ACTIONS,
    form: Some(&crate::features::system::forms::SCHEDULER_FORM),
};

const SCRIPTS: ResourceSpec = ResourceSpec {
    id: "scripts",
    group: "system-group",
    cli_path: None,
    label: "Scripts",
    fetch: FetchKind::List {
        endpoint: "/system/script",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("owner", "Owner", 14),
        col!("policy", "Policy", 28),
        col!("dont-require-permissions", "No perms", 9),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SCRIPT_ACTIONS,
    form: Some(&crate::features::system::forms::SCRIPT_FORM),
};

const LOGGING: ResourceSpec = ResourceSpec {
    id: "logging",
    group: "system-group",
    cli_path: None,
    label: "Logging",
    fetch: FetchKind::List {
        endpoint: "/system/logging",
    },
    columns: &[
        col!("topics", "Topics", 24),
        col!("action", "Action", 12),
        col!("prefix", "Prefix", 14),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::system::forms::LOGGING_FORM),
};

const LOGGING_ACTIONS: ResourceSpec = ResourceSpec {
    id: "logging-actions",
    group: "system-group",
    cli_path: None,
    label: "Logging Actions",
    fetch: FetchKind::List {
        endpoint: "/system/logging/action",
    },
    columns: &[
        col!("name", "Name", 14),
        col!("target", "Type", 10),
        col!("remote", "Remote", 16),
        col!("remote-port", "Port", 6),
        col!("remote-protocol", "Proto", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::system::forms::LOGGING_ACTION_FORM),
};

const SYSTEM_CONSOLE: ResourceSpec = ResourceSpec {
    id: "system-console",
    group: "system-group",
    cli_path: None,
    label: "Console",
    fetch: FetchKind::List {
        endpoint: "/system/console",
    },
    columns: &[
        col!("port", "Port", 14),
        col!("term", "Term", 10),
        col!("channel", "Ch", 4),
        col!("disabled", "Off", 5),
        col!("used", "Used", 6),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::system::forms::CONSOLE_FORM),
};

const LEDS: ResourceSpec = ResourceSpec {
    id: "leds",
    group: "system-group",
    cli_path: None,
    label: "LEDs",
    fetch: FetchKind::List {
        endpoint: "/system/led",
    },
    columns: &[
        col!("type", "Type", 22),
        col!("interface", "Interface", 16),
        col!("leds", "LEDs", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::system::forms::LED_FORM),
};

const LED_SETTINGS: ResourceSpec = ResourceSpec {
    id: "led-settings",
    group: "system-group",
    cli_path: None,
    label: "LED Settings",
    fetch: FetchKind::System {
        endpoint: "/system/led/settings",
    },
    columns: &[col!("all-leds-off", "All off", 14)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::LED_SETTINGS_FORM),
};

const PORTS: ResourceSpec = ResourceSpec {
    id: "ports",
    group: "system-group",
    cli_path: Some("/port"),
    label: "Ports",
    fetch: FetchKind::List { endpoint: "/port" },
    columns: &[
        col!("name", "Name", 14),
        col!("baud-rate", "Baud", 10),
        col!("data-bits", "Bits", 5),
        col!("parity", "Parity", 8),
        col!("stop-bits", "Stop", 5),
        col!("flow-control", "Flow", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::PORT_FORM),
};

const SNMP: ResourceSpec = ResourceSpec {
    id: "snmp",
    group: "system-group",
    cli_path: None,
    label: "SNMP",
    fetch: FetchKind::System { endpoint: "/snmp" },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("contact", "Contact", 20),
        col!("location", "Location", 20),
        col!("engine-id", "Engine", 18),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::SNMP_FORM),
};

const SNMP_COMMUNITIES: ResourceSpec = ResourceSpec {
    id: "snmp-communities",
    group: "system-group",
    cli_path: None,
    label: "SNMP Communities",
    fetch: FetchKind::List {
        endpoint: "/snmp/community",
    },
    columns: &[
        col!("name", "Name", 16),
        col!("addresses", "Addresses", 24),
        col!("security", "Security", 12),
        col!("authentication-password", "Auth", 10),
        col!("encryption-password", "Encrypt", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::system::forms::SNMP_COMMUNITY_FORM),
};

const CERTIFICATES: ResourceSpec = ResourceSpec {
    id: "certificates",
    group: "system-group",
    cli_path: Some("/certificate"),
    label: "Certificates",
    fetch: FetchKind::List {
        endpoint: "/certificate",
    },
    columns: &[
        col!("name", "Name", 20),
        col!("common-name", "CN", 24),
        col!("key-usage", "Usage", 24),
        col!("trusted", "Trust", 6),
        col!("invalid-after", "Expires", 20),
        col!("fingerprint", "Fingerprint", 20),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::CERTIFICATE_ACTIONS,
    form: Some(&crate::features::system::forms::CERTIFICATE_FORM),
};

const WATCHDOG: ResourceSpec = ResourceSpec {
    id: "watchdog",
    group: "system-group",
    cli_path: None,
    label: "Watchdog",
    fetch: FetchKind::System {
        endpoint: "/system/watchdog",
    },
    columns: &[
        col!("watch-address", "Watch", 18),
        col!("watch-interval", "Interval", 10),
        col!("automatic-supout", "Supout", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: &[],
    form: Some(&crate::features::system::forms::WATCHDOG_FORM),
};

const NOTE: ResourceSpec = ResourceSpec {
    id: "note",
    group: "system-group",
    cli_path: None,
    label: "Note",
    fetch: FetchKind::System {
        endpoint: "/system/note",
    },
    columns: &[col!("show-at-login", "Login", 8), col!("note", "Note", 48)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::system::forms::NOTE_FORM),
};

const LOGS: ResourceSpec = ResourceSpec {
    id: "logs",
    group: "system-group",
    cli_path: Some("/log"),
    label: "Logs",
    fetch: FetchKind::List { endpoint: "/log" },
    columns: &[
        col!("time", "Time", 19),
        col!("topics", "Topics", 24),
        col!("message", "Message", 72),
    ],
    refresh: Duration::from_secs(1),
    actions: &[],
    form: None,
};
