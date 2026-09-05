//! Users, clock, NTP, packages, logging, SNMP, and related System schemas.

use super::common::{
    COMMENT, ENABLED, FILE_NAME, GROUP, LOOKUP_FILE, LOOKUP_NTP_KEY, LOOKUP_PORT, LOOKUP_SCRIPT,
    LOOKUP_USER, LOOKUP_VRF, NAME, ON_EVENT, OWNER, PASSWORD, POLICY, SOURCE,
};
use crate::form_fields::{
    KIND_NTP_CLIENT_MODE, KIND_PACKAGE_CHANNEL, KIND_SNMP_SECURITY, KIND_TIME_ZONE_NAME,
};
use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

pub(crate) const INACTIVITY_POLICIES: &[&str] = &["none", "logout", "lockscreen"];
const INACTIVITY_POLICY: FieldSpec = f!(
    "inactivity-policy",
    "Inactivity Policy",
    FieldKind::Enum {
        values: INACTIVITY_POLICIES,
    }
);

/// Type combo for `/system/logging/action` (API key `target`).
pub(crate) const LOGGING_ACTION_TYPES: &[&str] =
    &["memory", "disk", "echo", "remote", "email", "script"];
const LOGGING_ACTION_TYPE: FieldSpec = f!(
    "target",
    "Type",
    FieldKind::Enum {
        values: LOGGING_ACTION_TYPES,
    }
);
pub(crate) const REMOTE_PROTOCOLS: &[&str] = &["udp", "tcp", "tls"];
const REMOTE_PROTOCOL: FieldSpec = f!(
    "remote-protocol",
    "Remote Protocol",
    FieldKind::Enum {
        values: REMOTE_PROTOCOLS,
    }
);
pub(crate) const REMOTE_LOG_FORMATS: &[&str] = &["default", "syslog", "cef"];
const REMOTE_LOG_FORMAT: FieldSpec = f!(
    "remote-log-format",
    "Remote Log Format",
    FieldKind::Enum {
        values: REMOTE_LOG_FORMATS,
    }
);
pub(crate) const SYSLOG_FACILITIES: &[&str] = &[
    "daemon", "kern", "user", "mail", "auth", "syslog", "lpr", "news", "uucp", "cron", "authpriv",
    "ftp", "ntp", "local0", "local1", "local2", "local3", "local4", "local5", "local6", "local7",
];
const SYSLOG_FACILITY: FieldSpec = f!(
    "syslog-facility",
    "Syslog Facility",
    FieldKind::Enum {
        values: SYSLOG_FACILITIES,
    }
);
pub(crate) const SYSLOG_SEVERITIES: &[&str] = &[
    "auto",
    "emergency",
    "alert",
    "critical",
    "error",
    "warning",
    "notice",
    "info",
    "debug",
];
const SYSLOG_SEVERITY: FieldSpec = f!(
    "syslog-severity",
    "Syslog Severity",
    FieldKind::Enum {
        values: SYSLOG_SEVERITIES,
    }
);
pub(crate) const SYSLOG_TIME_FORMATS: &[&str] = &["bsd-syslog", "iso8601"];
const SYSLOG_TIME_FORMAT: FieldSpec = f!(
    "syslog-time-format",
    "Timestamp Format",
    FieldKind::Enum {
        values: SYSLOG_TIME_FORMATS,
    }
);
const LOGGING_ACTION_GENERAL: &[FieldSpec] = &[
    NAME,
    LOGGING_ACTION_TYPE,
    f!("memory-lines", "Memory Lines", FieldKind::Number),
    f!(
        "memory-stop-on-full",
        "Memory Stop On Full",
        FieldKind::Toggle
    ),
    f!("remember", "Remember", FieldKind::Toggle),
    f!("disk-file-name", "Disk File Name", FieldKind::Text),
    f!(
        "disk-lines-per-file",
        "Disk Lines Per File",
        FieldKind::Number
    ),
    f!("disk-file-count", "Disk File Count", FieldKind::Number),
    f!("disk-stop-on-full", "Disk Stop On Full", FieldKind::Toggle),
    f!("remote", "Remote Address", FieldKind::Ip),
    f!("remote-port", "Remote Port", FieldKind::Number),
    f!("src-address", "Src Address", FieldKind::Ip),
    REMOTE_LOG_FORMAT,
    REMOTE_PROTOCOL,
    SYSLOG_FACILITY,
    SYSLOG_SEVERITY,
    SYSLOG_TIME_FORMAT,
    f!(
        "cef-event-delimiter",
        "CEF Event Delimiter",
        FieldKind::Text
    ),
    f!("check-certificate", "Check Certificate", FieldKind::Toggle),
    f!("vrf", "VRF", LOOKUP_VRF),
    f!("add-topics-string", "Add Topics String", FieldKind::Toggle),
    f!("email-to", "Email To", FieldKind::Text),
    f!("email-cc", "Email CC", FieldKind::Text),
    f!("email-start-tls", "Email STARTTLS", FieldKind::Toggle),
    f!("script", "Script", LOOKUP_SCRIPT),
];

const USER_GENERAL: &[FieldSpec] = &[
    NAME,
    GROUP,
    PASSWORD,
    f!("address", "Address", FieldKind::Repeat),
    INACTIVITY_POLICY,
    f!("inactivity-timeout", "Inactivity Timeout", FieldKind::Time),
    COMMENT,
    ENABLED,
];

pub static USER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["group"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: USER_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("last-logged-in", "Last Login", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static USER_GROUP_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["policy"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, POLICY, f!("skin", "Skin", FieldKind::Text), COMMENT],
    }],
    create_sections: &[],
};

pub static NTP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "servers",
    subtitle_keys: &["mode"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("enabled", "Enabled", FieldKind::Toggle),
                f!("mode", "Mode", KIND_NTP_CLIENT_MODE),
                f!("servers", "Servers", FieldKind::Repeat),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static NTP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["vrf"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("broadcast", "Broadcast", FieldKind::Toggle),
            f!(
                "broadcast-addresses",
                "Broadcast Addresses",
                FieldKind::Repeat
            ),
            f!("multicast", "Multicast", FieldKind::Toggle),
            f!("manycast", "Manycast", FieldKind::Toggle),
            f!("vrf", "VRF", LOOKUP_VRF),
            f!("use-local-clock", "Use Local Clock", FieldKind::Toggle),
            f!(
                "local-clock-stratum",
                "Local Clock Stratum",
                FieldKind::Number
            ),
            f!("auth-key", "Auth. Key", LOOKUP_NTP_KEY),
        ],
    }],
    create_sections: &[],
};

pub static NTP_KEY_FORM: FormSchema = FormSchema {
    title_key: "key-id",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("key-id", "Key ID", FieldKind::Number),
            f!("key-val", "Key", FieldKind::Secret),
        ],
    }],
    create_sections: &[],
};

pub static CLOCK_FORM: FormSchema = FormSchema {
    title_key: "time-zone-name",
    subtitle_keys: &["gmt-offset"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("time-zone-name", "Time Zone", KIND_TIME_ZONE_NAME),
                f!(
                    "time-zone-autodetect",
                    "Time Zone Autodetect",
                    FieldKind::Toggle
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("time", "Time", FieldKind::Readonly),
                f!("date", "Date", FieldKind::Readonly),
                f!("gmt-offset", "Offset", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static IDENTITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
    create_sections: &[],
};

const SCHEDULER_GENERAL: &[FieldSpec] = &[
    NAME,
    f!("start-date", "Start Date", FieldKind::Text),
    f!("start-time", "Start Time", FieldKind::Text),
    f!("interval", "Interval", FieldKind::Time),
    ON_EVENT,
    POLICY,
    COMMENT,
    ENABLED,
];

pub static SCHEDULER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interval"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: SCHEDULER_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                OWNER,
                f!("next-run", "Next Run", FieldKind::Readonly),
                f!("run-count", "Run Count", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

const SCRIPT_GENERAL: &[FieldSpec] = &[
    NAME,
    SOURCE,
    POLICY,
    f!(
        "dont-require-permissions",
        "Don't Require Permissions",
        FieldKind::Toggle
    ),
    COMMENT,
];

pub static SCRIPT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["policy"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: SCRIPT_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[OWNER],
        },
    ],
    create_sections: &[],
};

pub static LOGGING_FORM: FormSchema = FormSchema {
    title_key: "topics",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("topics", "Topics", FieldKind::Repeat),
            f!(
                "action",
                "Action",
                FieldKind::Lookup {
                    resource_id: "logging-actions",
                    value_key: "name",
                    multiple: false,
                }
            ),
            f!("prefix", "Prefix", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static LOGGING_ACTION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["target"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LOGGING_ACTION_GENERAL,
    }],
    create_sections: &[],
};

pub static SNMP_FORM: FormSchema = FormSchema {
    title_key: "contact",
    subtitle_keys: &["location"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("contact", "Contact", FieldKind::Text),
            f!("location", "Location", FieldKind::Text),
            f!("engine-id", "Engine ID", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static SNMP_COMMUNITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["security"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("addresses", "Addresses", FieldKind::Repeat),
            f!("security", "Security", KIND_SNMP_SECURITY),
            f!("read-access", "Read Access", FieldKind::Toggle),
            f!("write-access", "Write Access", FieldKind::Toggle),
            f!(
                "authentication-password",
                "Authentication Password",
                FieldKind::Secret
            ),
            f!(
                "encryption-password",
                "Encryption Password",
                FieldKind::Secret
            ),
        ],
    }],
    create_sections: &[],
};

pub static WATCHDOG_FORM: FormSchema = FormSchema {
    title_key: "watch-address",
    subtitle_keys: &["watch-interval"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("watchdog-timer", "Watchdog Timer", FieldKind::Toggle),
            f!("watch-address", "Watch Address", FieldKind::Ip),
            f!("watch-interval", "Watch Interval", FieldKind::Time),
            f!("no-ping-delay", "No Ping Delay", FieldKind::Time),
            f!("ping-start-after", "Ping Start After", FieldKind::Time),
            f!("ping-timeout", "Ping Timeout", FieldKind::Time),
            f!("automatic-supout", "Automatic Supout", FieldKind::Toggle),
            f!("auto-send-supout", "Auto Send Supout", FieldKind::Toggle),
            f!("send-email-to", "Send Email To", FieldKind::Text),
            f!("send-smtp-server", "Send SMTP Server", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static NOTE_FORM: FormSchema = FormSchema {
    title_key: "note",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("show-at-login", "Show At Login", FieldKind::Toggle),
            f!("note", "Note", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static LICENSE_FORM: FormSchema = FormSchema {
    title_key: "software-id",
    subtitle_keys: &["nlevel", "level"],
    sections: &[FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: &[
            f!("software-id", "Software ID", FieldKind::Readonly),
            f!("old-software-id", "Old Software ID", FieldKind::Readonly),
            f!("nlevel", "Level", FieldKind::Readonly),
            f!("features", "Features", FieldKind::Readonly),
            f!("expires-in", "Expires In", FieldKind::Readonly),
            f!("system-id", "System ID", FieldKind::Readonly),
            f!("level", "CHR Level", FieldKind::Readonly),
            f!("limited-upgrades", "Limited Upgrades", FieldKind::Readonly),
            f!("next-renewal-at", "Next Renewal At", FieldKind::Readonly),
            f!("deadline-at", "Deadline At", FieldKind::Readonly),
        ],
    }],
    create_sections: &[],
};

const LICENSE_IMPORT_FIELDS: &[FieldSpec] = &[FILE_NAME, f!("k", "License Key", FieldKind::Secret)];

pub static LICENSE_IMPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LICENSE_IMPORT_FIELDS,
    }],
};

pub(crate) const DEVICE_MODE_MODES: &[&str] = &["advanced", "home", "basic", "rose"];

pub static DEVICE_MODE_FORM: FormSchema = FormSchema {
    title_key: "mode",
    subtitle_keys: &["flagged"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!(
                    "mode",
                    "Mode",
                    FieldKind::Enum {
                        values: DEVICE_MODE_MODES,
                    }
                ),
                f!("scheduler", "Scheduler", FieldKind::Toggle),
                f!("socks", "SOCKS", FieldKind::Toggle),
                f!("fetch", "Fetch", FieldKind::Toggle),
                f!("pptp", "PPTP", FieldKind::Toggle),
                f!("l2tp", "L2TP", FieldKind::Toggle),
                f!("bandwidth-test", "Bandwidth Test", FieldKind::Toggle),
                f!("traffic-gen", "Traffic Generator", FieldKind::Toggle),
                f!("sniffer", "Sniffer", FieldKind::Toggle),
                f!("ipsec", "IPsec", FieldKind::Toggle),
                f!("romon", "RoMON", FieldKind::Toggle),
                f!("proxy", "Proxy", FieldKind::Toggle),
                f!("hotspot", "Hotspot", FieldKind::Toggle),
                f!("smb", "SMB", FieldKind::Toggle),
                f!("email", "Email", FieldKind::Toggle),
                f!("zerotier", "ZeroTier", FieldKind::Toggle),
                f!("container", "Container", FieldKind::Toggle),
                f!(
                    "install-any-version",
                    "Install Any Version",
                    FieldKind::Toggle
                ),
                f!("partitions", "Partitions", FieldKind::Toggle),
                f!("routerboard", "RouterBOARD", FieldKind::Toggle),
                f!("flagging-enabled", "Flagging Enabled", FieldKind::Toggle),
                f!("flagged", "Flagged", FieldKind::Toggle),
                f!("activation-timeout", "Activation Timeout", FieldKind::Time),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("allowed-versions", "Allowed Versions", FieldKind::Readonly),
                f!("attempt-count", "Attempt Count", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static PACKAGE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["version"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[f!("name", "Name", FieldKind::Readonly), ENABLED],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("version", "Version", FieldKind::Readonly),
                f!("build-time", "Build Time", FieldKind::Readonly),
                f!("scheduled", "Scheduled", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static PACKAGE_UPDATE_FORM: FormSchema = FormSchema {
    title_key: "channel",
    subtitle_keys: &["installed-version"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[f!("channel", "Channel", KIND_PACKAGE_CHANNEL)],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("installed-version", "Installed", FieldKind::Readonly),
                f!("latest-version", "Latest", FieldKind::Readonly),
                f!("status", "Status", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static SSH_KEY_FORM: FormSchema = FormSchema {
    title_key: "user",
    subtitle_keys: &["key-owner"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("user", "User", LOOKUP_USER),
            f!("key-owner", "Key Owner", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

const RESET_CONFIG_GENERAL: &[FieldSpec] = &[
    f!("keep-users", "Keep Users", FieldKind::Toggle),
    f!("no-defaults", "No Defaults", FieldKind::Toggle),
    f!("skip-backup", "Skip Backup", FieldKind::Toggle),
    f!("caps-mode", "CAPs Mode", FieldKind::Toggle),
    f!("run-after-reset", "Run After Reset", LOOKUP_FILE),
];

pub static RESET_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "keep-users",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: RESET_CONFIG_GENERAL,
    }],
};

pub static INSTALL_PACKAGE_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FILE_NAME],
    }],
};

const EXPORT_CONFIG_FIELDS: &[FieldSpec] = &[
    f!("file", "File", FieldKind::Text),
    f!("hide-sensitive", "Hide Sensitive", FieldKind::Toggle),
    f!("terse", "Terse", FieldKind::Toggle),
];

pub static EXPORT_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "file",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: EXPORT_CONFIG_FIELDS,
    }],
};

const IMPORT_CONFIG_FIELDS: &[FieldSpec] = &[
    FILE_NAME,
    f!("from-line", "From Line", FieldKind::Number),
    f!("verbose", "Verbose", FieldKind::Toggle),
];

pub static IMPORT_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: IMPORT_CONFIG_FIELDS,
    }],
};

pub static AT_CHAT_PROMPT: FormSchema = FormSchema {
    title_key: "input",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("input", "AT Command", FieldKind::Text)],
    }],
};

const SPECIAL_LOGIN_GENERAL: &[FieldSpec] = &[
    f!("user", "User", LOOKUP_USER),
    f!("port", "Port", LOOKUP_PORT),
    ENABLED,
];

pub static SPECIAL_LOGIN_FORM: FormSchema = FormSchema {
    title_key: "user",
    subtitle_keys: &["port"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: SPECIAL_LOGIN_GENERAL,
    }],
    create_sections: &[],
};
