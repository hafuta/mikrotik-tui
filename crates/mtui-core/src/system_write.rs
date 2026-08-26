//! Form schemas for the System nav group.
//!
//! Extra API endpoints for the parent catalog (not wired from this module):
//! - `/rest/system/identity`
//! - `/rest/system/resource`
//! - `/rest/system/health`
//! - `/rest/system/package`
//! - `/rest/system/scheduler`
//! - `/rest/system/script`
//! - `/rest/system/logging`
//! - `/rest/system/logging/action`
//! - `/rest/system/ntp/server`
//! - `/rest/system/ntp/key`
//! - `/rest/snmp`
//! - `/rest/snmp/community`
//! - `/rest/certificate`
//! - `/rest/system/watchdog`
//! - `/rest/system/console`
//! - `/rest/system/led`
//! - `/rest/system/led/settings`
//! - `/rest/port`
//! - `/rest/special-login`
//! - `/rest/system/routerboard/settings`
//! - `/rest/system/note`
//! - `/rest/system/license`
//! - `/rest/disk`
//! - `/rest/system/device-mode`
//! - `/rest/user/group`

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

const LOOKUP_USER_GROUP: FieldKind = FieldKind::Lookup {
    resource_id: "user-groups",
    value_key: "name",
    multiple: false,
};
const LOOKUP_USER: FieldKind = FieldKind::Lookup {
    resource_id: "users",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SCRIPT: FieldKind = FieldKind::Lookup {
    resource_id: "scripts",
    value_key: "name",
    multiple: false,
};
const LOOKUP_CERTIFICATE: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};
const LOOKUP_FILE: FieldKind = FieldKind::Lookup {
    resource_id: "files",
    value_key: "name",
    multiple: false,
};
const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};
const LOOKUP_NTP_KEY: FieldKind = FieldKind::Lookup {
    resource_id: "ntp-keys",
    value_key: "key-id",
    multiple: false,
};
const LOOKUP_DISK: FieldKind = FieldKind::Lookup {
    resource_id: "disks",
    value_key: "slot",
    multiple: false,
};
const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_PORT: FieldKind = FieldKind::Lookup {
    resource_id: "ports",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const POLICY: FieldSpec = f!("policy", "Policy", FieldKind::Text);
const PASSWORD: FieldSpec = f!("password", "Password", FieldKind::Secret);
const SOURCE: FieldSpec = f!("source", "Source", FieldKind::Text);
const GROUP: FieldSpec = f!("group", "Group", LOOKUP_USER_GROUP);
const OWNER: FieldSpec = f!("owner", "Owner", FieldKind::Readonly);
const ON_EVENT: FieldSpec = f!("on-event", "On event", LOOKUP_SCRIPT);
const INACTIVITY_POLICY: FieldSpec = f!(
    "inactivity-policy",
    "Inactivity Policy",
    FieldKind::Enum {
        values: INACTIVITY_POLICIES,
    }
);
const INACTIVITY_POLICIES: &[&str] = &["none", "logout", "lockscreen"];
const CA: FieldSpec = f!("ca", "CA", LOOKUP_CERTIFICATE);
const FILE_NAME: FieldSpec = f!("file-name", "File name", LOOKUP_FILE);

/// Type combo for `/system/logging/action` (API key `target`).
const LOGGING_ACTION_TYPES: &[&str] = &["memory", "disk", "echo", "remote", "email", "script"];
const LOGGING_ACTION_TYPE: FieldSpec = f!(
    "target",
    "Type",
    FieldKind::Enum {
        values: LOGGING_ACTION_TYPES,
    }
);
const REMOTE_PROTOCOLS: &[&str] = &["udp", "tcp", "tls"];
const REMOTE_PROTOCOL: FieldSpec = f!(
    "remote-protocol",
    "Remote Protocol",
    FieldKind::Enum {
        values: REMOTE_PROTOCOLS,
    }
);
const REMOTE_LOG_FORMATS: &[&str] = &["default", "syslog", "cef"];
const REMOTE_LOG_FORMAT: FieldSpec = f!(
    "remote-log-format",
    "Remote Log Format",
    FieldKind::Enum {
        values: REMOTE_LOG_FORMATS,
    }
);
const SYSLOG_FACILITIES: &[&str] = &[
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
const SYSLOG_SEVERITIES: &[&str] = &[
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
const SYSLOG_TIME_FORMATS: &[&str] = &["bsd-syslog", "iso8601"];
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
    f!("remote", "Remote Address", FieldKind::Text),
    f!("remote-port", "Remote Port", FieldKind::Number),
    f!("src-address", "Src Address", FieldKind::Text),
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
    DISABLED,
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
            fields: &[f!("last-logged-in", "Last login", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: USER_GENERAL,
    }],
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, POLICY],
    }],
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
                f!("mode", "Mode", FieldKind::Text),
                f!("servers", "Servers", FieldKind::Text),
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("key-id", "Key ID", FieldKind::Number),
            f!("key-val", "Key", FieldKind::Secret),
        ],
    }],
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
                f!("time-zone-name", "Time zone", FieldKind::Text),
                f!("time-zone-autodetect", "Autodetect TZ", FieldKind::Toggle),
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
    DISABLED,
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: SCHEDULER_GENERAL,
    }],
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: SCRIPT_GENERAL,
    }],
};

pub static LOGGING_FORM: FormSchema = FormSchema {
    title_key: "topics",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("topics", "Topics", FieldKind::Text),
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
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("topics", "Topics", FieldKind::Text),
            f!(
                "action",
                "Action",
                FieldKind::Lookup {
                    resource_id: "logging-actions",
                    value_key: "name",
                    multiple: false,
                }
            ),
        ],
    }],
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LOGGING_ACTION_GENERAL,
    }],
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
            f!("addresses", "Addresses", FieldKind::Text),
            f!("security", "Security", FieldKind::Text),
            f!("read-access", "Read access", FieldKind::Toggle),
            f!("write-access", "Write access", FieldKind::Toggle),
            f!(
                "authentication-password",
                "Auth password",
                FieldKind::Secret
            ),
            f!("encryption-password", "Encrypt password", FieldKind::Secret),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static CERTIFICATE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["common-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("common-name", "Common name", FieldKind::Text),
                f!("key-usage", "Key usage", FieldKind::Text),
                f!("trusted", "Trusted", FieldKind::Toggle),
                f!("days-valid", "Days valid", FieldKind::Number),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("digest-algorithm", "Digest", FieldKind::Readonly),
                f!("key-size", "Key size", FieldKind::Readonly),
                f!("invalid-before", "Valid from", FieldKind::Readonly),
                f!("invalid-after", "Valid to", FieldKind::Readonly),
                f!("serial-number", "Serial", FieldKind::Readonly),
                f!("fingerprint", "Fingerprint", FieldKind::Readonly),
                f!("akid", "AKID", FieldKind::Readonly),
                f!("skid", "SKID", FieldKind::Readonly),
                f!("expires-after", "Expires", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("common-name", "Common name", FieldKind::Text),
            f!("key-usage", "Key usage", FieldKind::Text),
        ],
    }],
};

const CERT_EXPORT_TYPES: &[&str] = &["pem", "pkcs12"];

pub static CERT_SIGN_PROMPT: FormSchema = FormSchema {
    title_key: "ca",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "sign",
        label: "Sign",
        read_only: false,
        fields: &[CA],
    }],
    create_sections: &[FormSection {
        id: "sign",
        label: "Sign",
        read_only: false,
        fields: &[CA],
    }],
};

pub static CERT_IMPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "import",
        label: "Import",
        read_only: false,
        fields: &[
            FILE_NAME,
            f!("passphrase", "Passphrase", FieldKind::Secret),
            NAME,
        ],
    }],
    create_sections: &[FormSection {
        id: "import",
        label: "Import",
        read_only: false,
        fields: &[
            FILE_NAME,
            f!("passphrase", "Passphrase", FieldKind::Secret),
            NAME,
        ],
    }],
};

pub static CERT_EXPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "export",
        label: "Export",
        read_only: false,
        fields: &[
            FILE_NAME,
            FieldSpec {
                key: "type",
                label: "Type",
                kind: FieldKind::Enum {
                    values: CERT_EXPORT_TYPES,
                },
            },
            f!("export-passphrase", "Export passphrase", FieldKind::Secret),
        ],
    }],
    create_sections: &[FormSection {
        id: "export",
        label: "Export",
        read_only: false,
        fields: &[
            FILE_NAME,
            FieldSpec {
                key: "type",
                label: "Type",
                kind: FieldKind::Enum {
                    values: CERT_EXPORT_TYPES,
                },
            },
            f!("export-passphrase", "Export passphrase", FieldKind::Secret),
        ],
    }],
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
            f!("watch-address", "Watch Address", FieldKind::Text),
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
            f!("show-at-login", "Show at login", FieldKind::Toggle),
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

pub static LICENSE_IMPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FILE_NAME, f!("k", "License Key", FieldKind::Secret)],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FILE_NAME, f!("k", "License Key", FieldKind::Secret)],
    }],
};

const DISK_TYPES: &[&str] = &[
    "hardware",
    "raid",
    "partition",
    "tmpfs",
    "ramdisk",
    "file",
    "crypted",
    "sshfs",
    "nfs",
    "smb",
    "nvme-tcp",
    "iscsi",
];
const RAID_TYPES: &[&str] = &["0", "1", "4", "5", "6", "linear", "faulty"];
const RAID_CHUNK_SIZES: &[&str] = &["64K", "128K", "256K", "512K", "1M", "2M", "4M"];
const FORMAT_FILE_SYSTEMS: &[&str] = &[
    "ext4",
    "fat32",
    "exfat",
    "xfs",
    "btrfs",
    "discard",
    "discard-secure",
    "wipe",
];
const DISK_TYPE: FieldSpec = f!("type", "Type", FieldKind::Enum { values: DISK_TYPES });
const DISK_SLOT: FieldSpec = f!("slot", "Slot", FieldKind::Text);

pub static DISK_FORM: FormSchema = FormSchema {
    title_key: "slot",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                DISK_SLOT,
                DISK_TYPE,
                f!("parent", "Parent", LOOKUP_DISK),
                f!("mount-filesystem", "Mount Filesystem", FieldKind::Toggle),
                f!("mount-read-only", "Mount Read Only", FieldKind::Toggle),
                f!("swap", "Swap", FieldKind::Toggle),
                DISABLED,
                f!("tmpfs-max-size", "Tmpfs Max Size", FieldKind::Number),
                f!("ramdisk-size", "Ramdisk Size", FieldKind::Number),
                f!("partition-number", "Partition Number", FieldKind::Number),
                f!("partition-offset", "Partition Offset", FieldKind::Number),
                f!("partition-size", "Partition Size", FieldKind::Number),
                f!(
                    "raid-type",
                    "RAID Type",
                    FieldKind::Enum { values: RAID_TYPES }
                ),
                f!("raid-device-count", "RAID Device Count", FieldKind::Number),
                f!(
                    "raid-max-component-size",
                    "RAID Max Component Size",
                    FieldKind::Number
                ),
                f!(
                    "raid-chunk-size",
                    "RAID Chunk Size",
                    FieldKind::Enum {
                        values: RAID_CHUNK_SIZES,
                    }
                ),
                f!("raid-master", "RAID Master", LOOKUP_DISK),
                f!("raid-role", "RAID Role", FieldKind::Number),
                f!(
                    "raid-member-failed",
                    "RAID Member Failed",
                    FieldKind::Toggle
                ),
                f!("file-path", "File Path", LOOKUP_FILE),
                f!("file-size", "File Size", FieldKind::Number),
                f!("file-offset", "File Offset", FieldKind::Number),
                f!("crypted-backend", "Crypted Backend", LOOKUP_DISK),
                f!("encryption-key", "Encryption Key", FieldKind::Secret),
                f!("sshfs-address", "SSHFS Address", FieldKind::Text),
                f!("sshfs-port", "SSHFS Port", FieldKind::Number),
                f!("sshfs-user", "SSHFS User", FieldKind::Text),
                f!("sshfs-password", "SSHFS Password", FieldKind::Secret),
                f!("sshfs-path", "SSHFS Path", FieldKind::Text),
                f!("nfs-address", "NFS Address", FieldKind::Text),
                f!("nfs-share", "NFS Share", FieldKind::Text),
                f!("smb-address", "SMB Address", FieldKind::Text),
                f!("smb-share", "SMB Share", FieldKind::Text),
                f!("smb-user", "SMB User", FieldKind::Text),
                f!("smb-password", "SMB Password", FieldKind::Secret),
                f!("smb-encryption", "SMB Encryption", FieldKind::Toggle),
                f!("nvme-tcp-address", "NVMe TCP Address", FieldKind::Text),
                f!("nvme-tcp-nqn", "NVMe TCP NQN", FieldKind::Text),
                f!("nvme-tcp-host-name", "NVMe TCP Host Name", FieldKind::Text),
                f!("nvme-tcp-password", "NVMe TCP Password", FieldKind::Secret),
                f!("nvme-tcp-port", "NVMe TCP Port", FieldKind::Number),
                f!("iscsi-address", "iSCSI Address", FieldKind::Text),
                f!("iscsi-iqn", "iSCSI IQN", FieldKind::Text),
                f!("iscsi-port", "iSCSI Port", FieldKind::Number),
                f!("nvme-tcp-export", "NVMe TCP Export", FieldKind::Toggle),
                f!(
                    "nvme-tcp-server-port",
                    "NVMe TCP Server Port",
                    FieldKind::Number
                ),
                f!(
                    "nvme-tcp-server-nqn",
                    "NVMe TCP Server NQN",
                    FieldKind::Text
                ),
                f!(
                    "nvme-tcp-server-password",
                    "NVMe TCP Server Password",
                    FieldKind::Secret
                ),
                f!("iscsi-export", "iSCSI Export", FieldKind::Toggle),
                f!("iscsi-server-port", "iSCSI Server Port", FieldKind::Number),
                f!("iscsi-server-iqn", "iSCSI Server IQN", FieldKind::Text),
                f!("nfs-sharing", "NFS Sharing", FieldKind::Toggle),
                f!("smb-sharing", "SMB Sharing", FieldKind::Toggle),
                f!("smb-server-user", "SMB Server User", LOOKUP_USER),
                f!(
                    "smb-server-password",
                    "SMB Server Password",
                    FieldKind::Secret
                ),
                f!(
                    "smb-server-encryption",
                    "SMB Server Encryption",
                    FieldKind::Toggle
                ),
                f!("media-sharing", "Media Sharing", FieldKind::Toggle),
                f!("media-interface", "Media Interface", LOOKUP_IFACE),
                f!(
                    "self-encryption-password",
                    "Self Encryption Password",
                    FieldKind::Secret
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("model", "Model", FieldKind::Readonly),
                f!("serial", "Serial", FieldKind::Readonly),
                f!("fw-version", "FW Version", FieldKind::Readonly),
                f!("size", "Size", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
                f!("fs", "FS", FieldKind::Readonly),
                f!("fs-label", "FS Label", FieldKind::Readonly),
                f!("fs-uuid", "FS UUID", FieldKind::Readonly),
                f!("state", "State", FieldKind::Readonly),
                f!("mount-point", "Mount Point", FieldKind::Readonly),
                f!("slot-default", "Slot Default", FieldKind::Readonly),
                f!("raid-uuid", "RAID UUID", FieldKind::Readonly),
                f!(
                    "raid-member-state",
                    "RAID Member State",
                    FieldKind::Readonly
                ),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[DISK_TYPE, DISK_SLOT],
    }],
};

pub static FORMAT_DISK_PROMPT: FormSchema = FormSchema {
    title_key: "file-system",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            FieldSpec {
                key: "file-system",
                label: "File System",
                kind: FieldKind::Enum {
                    values: FORMAT_FILE_SYSTEMS,
                },
            },
            f!("label", "Label", FieldKind::Text),
            f!(
                "mbr-partition-table",
                "MBR Partition Table",
                FieldKind::Toggle
            ),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            FieldSpec {
                key: "file-system",
                label: "File System",
                kind: FieldKind::Enum {
                    values: FORMAT_FILE_SYSTEMS,
                },
            },
            f!("label", "Label", FieldKind::Text),
            f!(
                "mbr-partition-table",
                "MBR Partition Table",
                FieldKind::Toggle
            ),
        ],
    }],
};

const DEVICE_MODE_MODES: &[&str] = &["advanced", "home", "basic", "rose"];

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
                f!("activation-timeout", "Activation Timeout", FieldKind::Text),
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
            fields: &[f!("name", "Name", FieldKind::Readonly), DISABLED],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("version", "Version", FieldKind::Readonly),
                f!("build-time", "Build time", FieldKind::Readonly),
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
            fields: &[f!("channel", "Channel", FieldKind::Text)],
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
            f!("key-owner", "Key owner", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("user", "User", LOOKUP_USER)],
    }],
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
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: RESET_CONFIG_GENERAL,
    }],
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
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("file-name", "File name", FieldKind::Text)],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("file-name", "File name", FieldKind::Text)],
    }],
};

pub static EXPORT_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "file",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("file", "File", FieldKind::Text),
            f!("hide-sensitive", "Hide sensitive", FieldKind::Toggle),
            f!("terse", "Terse", FieldKind::Toggle),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("file", "File", FieldKind::Text)],
    }],
};

pub static IMPORT_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("file-name", "File name", FieldKind::Text),
            f!("from-line", "From line", FieldKind::Text),
            f!("verbose", "Verbose", FieldKind::Toggle),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("file-name", "File name", FieldKind::Text)],
    }],
};

pub static AT_CHAT_PROMPT: FormSchema = FormSchema {
    title_key: "input",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("input", "AT command", FieldKind::Text)],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("input", "AT command", FieldKind::Text)],
    }],
};

const CONSOLE_GENERAL: &[FieldSpec] = &[
    f!("port", "Port", LOOKUP_PORT),
    f!("term", "Term", FieldKind::Text),
    f!("channel", "Channel", FieldKind::Number),
    DISABLED,
];

pub static CONSOLE_FORM: FormSchema = FormSchema {
    title_key: "port",
    subtitle_keys: &["term"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: CONSOLE_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("vcno", "VCNO", FieldKind::Readonly),
                f!("used", "Used", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
                f!("wedged", "Wedged", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: CONSOLE_GENERAL,
    }],
};

const LED_TYPES: &[&str] = &[
    "off",
    "on",
    "modem-status",
    "interface-status",
    "interface-activity",
    "wireless-status",
    "wireless-signal-strength",
    "poe-out",
    "flash-access",
    "rb-capsman",
    "rb-wps",
    "fan-fault",
    "gps-fix",
    "ap-cap",
];

const LED_GENERAL: &[FieldSpec] = &[
    FieldSpec {
        key: "type",
        label: "Type",
        kind: FieldKind::Enum { values: LED_TYPES },
    },
    f!("interface", "Interface", LOOKUP_IFACE),
    f!("modem", "Modem", LOOKUP_IFACE),
    f!("leds", "LEDs", FieldKind::Repeat),
    DISABLED,
];

pub static LED_FORM: FormSchema = FormSchema {
    title_key: "type",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LED_GENERAL,
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LED_GENERAL,
    }],
};

const LED_ALL_OFF: &[&str] = &["never", "immediately", "after-1h"];

pub static LED_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "all-leds-off",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FieldSpec {
            key: "all-leds-off",
            label: "All LEDs Off",
            kind: FieldKind::Enum {
                values: LED_ALL_OFF,
            },
        }],
    }],
    create_sections: &[],
};

const PORT_BAUD: &[&str] = &[
    "auto", "110", "300", "600", "1200", "2400", "4800", "9600", "19200", "38400", "57600",
    "115200", "230400", "460800", "921600",
];
const PORT_DATA_BITS: &[&str] = &["7", "8"];
const PORT_PARITY: &[&str] = &["none", "even", "odd"];
const PORT_STOP_BITS: &[&str] = &["1", "2"];
const PORT_FLOW: &[&str] = &["none", "hardware", "xon-xoff"];

pub static PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["baud-rate"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("name", "Name", FieldKind::Readonly),
                FieldSpec {
                    key: "baud-rate",
                    label: "Baud Rate",
                    kind: FieldKind::Enum { values: PORT_BAUD },
                },
                FieldSpec {
                    key: "data-bits",
                    label: "Data Bits",
                    kind: FieldKind::Enum {
                        values: PORT_DATA_BITS,
                    },
                },
                FieldSpec {
                    key: "parity",
                    label: "Parity",
                    kind: FieldKind::Enum {
                        values: PORT_PARITY,
                    },
                },
                FieldSpec {
                    key: "stop-bits",
                    label: "Stop Bits",
                    kind: FieldKind::Enum {
                        values: PORT_STOP_BITS,
                    },
                },
                FieldSpec {
                    key: "flow-control",
                    label: "Flow Control",
                    kind: FieldKind::Enum { values: PORT_FLOW },
                },
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("used", "Used", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

const SPECIAL_LOGIN_GENERAL: &[FieldSpec] = &[
    f!("user", "User", LOOKUP_USER),
    f!("port", "Port", LOOKUP_PORT),
    DISABLED,
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: SPECIAL_LOGIN_GENERAL,
    }],
};

const BOOT_OS: &[&str] = &["router-os", "container"];
const BOOT_DEVICE: &[&str] = &[
    "nand-if-fail-then-ethernet",
    "nand-only",
    "ethernet",
    "ethernet-once",
    "try-ethernet-once-then-nand",
];
const BOOT_PROTOCOL: &[&str] = &["dhcp", "bootp", "dhcp-or-bootp"];
const PROTECTED_ROUTERBOOT: &[&str] = &["disabled", "enabled"];

pub static ROUTERBOARD_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "boot-device",
    subtitle_keys: &["boot-os"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            FieldSpec {
                key: "boot-os",
                label: "Boot OS",
                kind: FieldKind::Enum { values: BOOT_OS },
            },
            FieldSpec {
                key: "boot-device",
                label: "Boot Device",
                kind: FieldKind::Enum {
                    values: BOOT_DEVICE,
                },
            },
            FieldSpec {
                key: "boot-protocol",
                label: "Boot Protocol",
                kind: FieldKind::Enum {
                    values: BOOT_PROTOCOL,
                },
            },
            f!("cpu-frequency", "CPU Frequency", FieldKind::Text),
            f!("memory-frequency", "Memory Frequency", FieldKind::Text),
            FieldSpec {
                key: "protected-routerboot",
                label: "Protected RouterBOOT",
                kind: FieldKind::Enum {
                    values: PROTECTED_ROUTERBOOT,
                },
            },
            f!(
                "reformat-hold-button",
                "Reformat Hold Button",
                FieldKind::Time
            ),
            f!(
                "reformat-hold-button-max",
                "Reformat Hold Button Max",
                FieldKind::Time
            ),
            f!("silent-boot", "Silent Boot", FieldKind::Toggle),
            f!("auto-upgrade", "Auto Upgrade", FieldKind::Toggle),
            f!(
                "force-backup-booter",
                "Force Backup Booter",
                FieldKind::Toggle
            ),
        ],
    }],
    create_sections: &[],
};

const BUTTON_GENERAL: &[FieldSpec] = &[
    f!("enabled", "Enabled", FieldKind::Toggle),
    f!("hold-time", "Hold Time", FieldKind::Time),
    ON_EVENT,
];

pub static ROUTERBOARD_MODE_BUTTON_FORM: FormSchema = FormSchema {
    title_key: "on-event",
    subtitle_keys: &["hold-time"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: BUTTON_GENERAL,
    }],
    create_sections: &[],
};

pub static ROUTERBOARD_RESET_BUTTON_FORM: FormSchema = FormSchema {
    title_key: "on-event",
    subtitle_keys: &["hold-time"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: BUTTON_GENERAL,
    }],
    create_sections: &[],
};

pub static USB_POWER_RESET_PROMPT: FormSchema = FormSchema {
    title_key: "duration",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("duration", "Duration", FieldKind::Time)],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("duration", "Duration", FieldKind::Time)],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::patch_body;
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn assert_enum(schema: &FormSchema, key: &str, values: &'static [&'static str]) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Enum { values })
        );
    }

    fn assert_label(schema: &FormSchema, key: &str, label: &str) {
        assert_eq!(schema.field(key).map(|field| field.label), Some(label));
    }

    fn assert_lookup(
        schema: &FormSchema,
        key: &str,
        resource_id: &'static str,
        value_key: &'static str,
    ) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Lookup {
                resource_id,
                value_key,
                multiple: false,
            })
        );
    }

    fn no_advanced(schema: &FormSchema) -> bool {
        schema
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    }

    #[test]
    fn user_password_is_secret_and_omitted_when_masked() {
        assert_eq!(
            USER_FORM.writable_keys(),
            [
                "name",
                "group",
                "password",
                "address",
                "inactivity-policy",
                "inactivity-timeout",
                "comment",
                "disabled"
            ]
        );
        assert_eq!(
            USER_FORM.field("password").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            USER_FORM.field("address").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            USER_FORM
                .field("inactivity-timeout")
                .map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert_enum(&USER_FORM, "inactivity-policy", INACTIVITY_POLICIES);
        assert!(!USER_FORM.writable_keys().contains(&"last-logged-in"));
        assert_eq!(create_keys(&USER_FORM), USER_FORM.writable_keys());
        assert_lookup(&USER_FORM, "group", "user-groups", "name");
        assert!(no_advanced(&USER_FORM));

        let mut original = HashMap::new();
        original.insert("name".into(), "admin".into());
        original.insert("group".into(), "full".into());
        original.insert("password".into(), "********".into());
        let mut current = original.clone();
        current.insert("group".into(), "read".into());
        current.insert("password".into(), "********".into());
        let body = patch_body(&USER_FORM, &original, &current, "********");
        assert_eq!(body.get("group").map(String::as_str), Some("read"));
        assert!(!body.contains_key("password"));
        assert!(!body.contains_key("last-logged-in"));
    }

    #[test]
    fn user_group_create_is_name_and_policy() {
        assert_eq!(
            USER_GROUP_FORM.writable_keys(),
            ["name", "policy", "skin", "comment"]
        );
        assert_eq!(create_keys(&USER_GROUP_FORM), ["name", "policy"]);
        assert!(no_advanced(&USER_GROUP_FORM));
    }

    #[test]
    fn ntp_client_is_singleton_with_status() {
        assert!(NTP_CLIENT_FORM.create_sections.is_empty());
        assert_eq!(
            NTP_CLIENT_FORM.writable_keys(),
            ["enabled", "mode", "servers"]
        );
        assert!(!NTP_CLIENT_FORM.writable_keys().contains(&"status"));
        assert!(
            NTP_CLIENT_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        assert!(no_advanced(&NTP_CLIENT_FORM));
    }

    #[test]
    fn ntp_server_is_singleton_without_advanced() {
        assert!(NTP_SERVER_FORM.create_sections.is_empty());
        assert_eq!(
            NTP_SERVER_FORM.writable_keys(),
            [
                "enabled",
                "broadcast",
                "broadcast-addresses",
                "multicast",
                "manycast",
                "vrf",
                "use-local-clock",
                "local-clock-stratum",
                "auth-key",
            ]
        );
        assert!(no_advanced(&NTP_SERVER_FORM));
        assert_lookup(&NTP_SERVER_FORM, "vrf", "vrf", "name");
        assert_eq!(
            NTP_SERVER_FORM
                .field("broadcast-addresses")
                .map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            NTP_SERVER_FORM
                .field("local-clock-stratum")
                .map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_label(
            &NTP_SERVER_FORM,
            "broadcast-addresses",
            "Broadcast Addresses",
        );
        assert_label(&NTP_SERVER_FORM, "use-local-clock", "Use Local Clock");
        assert_label(
            &NTP_SERVER_FORM,
            "local-clock-stratum",
            "Local Clock Stratum",
        );
        assert_label(&NTP_SERVER_FORM, "auth-key", "Auth. Key");
        assert_lookup(&NTP_SERVER_FORM, "auth-key", "ntp-keys", "key-id");
        assert_ne!(
            NTP_SERVER_FORM.field("auth-key").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            NTP_KEY_FORM.field("key-val").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(create_keys(&NTP_KEY_FORM), ["key-id", "key-val"]);
    }

    #[test]
    fn clock_keeps_timezone_writable() {
        assert!(CLOCK_FORM.create_sections.is_empty());
        assert_eq!(
            CLOCK_FORM.writable_keys(),
            ["time-zone-name", "time-zone-autodetect"]
        );
        assert!(!CLOCK_FORM.writable_keys().contains(&"time"));
        assert!(!CLOCK_FORM.writable_keys().contains(&"date"));
        assert!(!CLOCK_FORM.writable_keys().contains(&"gmt-offset"));
    }

    #[test]
    fn identity_only_name() {
        assert_eq!(IDENTITY_FORM.writable_keys(), ["name"]);
        assert!(IDENTITY_FORM.create_sections.is_empty());
        assert_eq!(IDENTITY_FORM.known_keys(), ["name"]);
        assert!(no_advanced(&IDENTITY_FORM));
    }

    #[test]
    fn scheduler_create_matches_writable_general() {
        assert_eq!(
            create_keys(&SCHEDULER_FORM),
            [
                "name",
                "start-date",
                "start-time",
                "interval",
                "on-event",
                "policy",
                "comment",
                "disabled"
            ]
        );
        assert_lookup(&SCHEDULER_FORM, "on-event", "scripts", "name");
        assert_eq!(
            SCHEDULER_FORM.field("interval").map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert!(!SCHEDULER_FORM.writable_keys().contains(&"next-run"));
        assert!(!SCHEDULER_FORM.writable_keys().contains(&"run-count"));
        assert!(!SCHEDULER_FORM.writable_keys().contains(&"owner"));
        assert_eq!(
            SCHEDULER_FORM.field("owner").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(no_advanced(&SCHEDULER_FORM));
    }

    #[test]
    fn script_source_is_writable_text_not_secret() {
        assert!(SCRIPT_FORM.writable_keys().contains(&"source"));
        assert_eq!(
            SCRIPT_FORM.field("source").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_ne!(
            SCRIPT_FORM.field("source").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            create_keys(&SCRIPT_FORM),
            [
                "name",
                "source",
                "policy",
                "dont-require-permissions",
                "comment"
            ]
        );
        assert_eq!(create_keys(&SCRIPT_FORM), SCRIPT_FORM.writable_keys());
        assert_eq!(
            SCRIPT_FORM
                .field("dont-require-permissions")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            SCRIPT_FORM.field("owner").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(no_advanced(&SCRIPT_FORM));
    }

    #[test]
    fn logging_create_is_topics_and_action() {
        assert_eq!(create_keys(&LOGGING_FORM), ["topics", "action"]);
        assert_eq!(
            LOGGING_FORM.writable_keys(),
            ["topics", "action", "prefix", "comment", "disabled"]
        );
        assert_lookup(&LOGGING_FORM, "action", "logging-actions", "name");
    }

    #[test]
    fn logging_action_form_covers_remote_syslog() {
        assert_eq!(
            create_keys(&LOGGING_ACTION_FORM),
            [
                "name",
                "target",
                "memory-lines",
                "memory-stop-on-full",
                "remember",
                "disk-file-name",
                "disk-lines-per-file",
                "disk-file-count",
                "disk-stop-on-full",
                "remote",
                "remote-port",
                "src-address",
                "remote-log-format",
                "remote-protocol",
                "syslog-facility",
                "syslog-severity",
                "syslog-time-format",
                "cef-event-delimiter",
                "check-certificate",
                "vrf",
                "add-topics-string",
                "email-to",
                "email-cc",
                "email-start-tls",
                "script",
            ]
        );
        assert!(no_advanced(&LOGGING_ACTION_FORM));
        assert_lookup(&LOGGING_ACTION_FORM, "script", "scripts", "name");
        assert_label(&LOGGING_ACTION_FORM, "target", "Type");
        assert_label(&LOGGING_ACTION_FORM, "syslog-facility", "Syslog Facility");
        assert_label(&LOGGING_ACTION_FORM, "syslog-severity", "Syslog Severity");
        assert_enum(&LOGGING_ACTION_FORM, "target", LOGGING_ACTION_TYPES);
        assert_enum(&LOGGING_ACTION_FORM, "syslog-facility", SYSLOG_FACILITIES);
        assert_enum(&LOGGING_ACTION_FORM, "syslog-severity", SYSLOG_SEVERITIES);
        assert_enum(
            &LOGGING_ACTION_FORM,
            "syslog-time-format",
            SYSLOG_TIME_FORMATS,
        );
        assert_enum(&LOGGING_ACTION_FORM, "remote-protocol", REMOTE_PROTOCOLS);
        assert_eq!(REMOTE_PROTOCOLS, ["udp", "tcp", "tls"]);
        assert_eq!(REMOTE_PROTOCOLS[0], "udp");
        assert_eq!(SYSLOG_FACILITIES[0], "daemon");
        assert_eq!(SYSLOG_SEVERITIES[0], "auto");
        assert_enum(
            &LOGGING_ACTION_FORM,
            "remote-log-format",
            REMOTE_LOG_FORMATS,
        );
        assert_lookup(&LOGGING_ACTION_FORM, "vrf", "vrf", "name");
    }

    #[test]
    fn snmp_singleton_and_community_secrets() {
        assert!(SNMP_FORM.create_sections.is_empty());
        assert_eq!(
            SNMP_FORM.writable_keys(),
            ["enabled", "contact", "location", "engine-id"]
        );
        assert_eq!(create_keys(&SNMP_COMMUNITY_FORM), ["name"]);
        assert_eq!(
            SNMP_COMMUNITY_FORM
                .field("authentication-password")
                .map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            SNMP_COMMUNITY_FORM
                .field("encryption-password")
                .map(|field| field.kind),
            Some(FieldKind::Secret)
        );

        let mut original = HashMap::new();
        original.insert("name".into(), "public".into());
        original.insert("authentication-password".into(), "********".into());
        original.insert("encryption-password".into(), "********".into());
        let mut current = original.clone();
        current.insert("addresses".into(), "0.0.0.0/0".into());
        current.insert("authentication-password".into(), "********".into());
        current.insert("encryption-password".into(), "********".into());
        let body = patch_body(&SNMP_COMMUNITY_FORM, &original, &current, "********");
        assert_eq!(body.get("addresses").map(String::as_str), Some("0.0.0.0/0"));
        assert!(!body.contains_key("authentication-password"));
        assert!(!body.contains_key("encryption-password"));
    }

    #[test]
    fn certificate_has_no_writable_text_private_key() {
        assert_eq!(
            create_keys(&CERTIFICATE_FORM),
            ["name", "common-name", "key-usage"]
        );
        assert_eq!(
            CERTIFICATE_FORM.writable_keys(),
            ["name", "common-name", "key-usage", "trusted", "days-valid"]
        );
        match CERTIFICATE_FORM.field("private-key") {
            None => {}
            Some(field) => {
                assert_eq!(field.kind, FieldKind::Secret);
                assert!(!create_keys(&CERTIFICATE_FORM).contains(&"private-key"));
            }
        }
        assert!(!CERTIFICATE_FORM.writable_keys().contains(&"fingerprint"));
        assert!(!CERTIFICATE_FORM.writable_keys().contains(&"serial-number"));
    }

    #[test]
    fn certificate_prompts_cover_sign_import_export() {
        assert_eq!(CERT_SIGN_PROMPT.writable_keys(), ["ca"]);
        assert_lookup(&CERT_SIGN_PROMPT, "ca", "certificates", "name");
        assert_eq!(
            CERT_IMPORT_PROMPT.writable_keys(),
            ["file-name", "passphrase", "name"]
        );
        assert_eq!(
            CERT_IMPORT_PROMPT
                .field("passphrase")
                .map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            CERT_EXPORT_PROMPT.writable_keys(),
            ["file-name", "type", "export-passphrase"]
        );
        assert_eq!(
            CERT_EXPORT_PROMPT
                .field("export-passphrase")
                .map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_lookup(&CERT_IMPORT_PROMPT, "file-name", "files", "name");
        assert_lookup(&CERT_EXPORT_PROMPT, "file-name", "files", "name");

        let mut original = HashMap::new();
        original.insert("file-name".into(), "web.p12".into());
        original.insert("passphrase".into(), "********".into());
        original.insert("export-passphrase".into(), "********".into());
        let mut current = original.clone();
        current.insert("file-name".into(), "web.pem".into());
        current.insert("passphrase".into(), "********".into());
        current.insert("export-passphrase".into(), "********".into());
        let import_body = patch_body(&CERT_IMPORT_PROMPT, &original, &current, "********");
        assert_eq!(
            import_body.get("file-name").map(String::as_str),
            Some("web.pem")
        );
        assert!(!import_body.contains_key("passphrase"));
        let export_body = patch_body(&CERT_EXPORT_PROMPT, &original, &current, "********");
        assert!(!export_body.contains_key("export-passphrase"));
        assert_eq!(
            export_body.get("file-name").map(String::as_str),
            Some("web.pem")
        );
    }

    #[test]
    fn watchdog_and_note_are_singletons() {
        assert!(WATCHDOG_FORM.create_sections.is_empty());
        assert!(NOTE_FORM.create_sections.is_empty());
        assert!(WATCHDOG_FORM.writable_keys().contains(&"watch-address"));
        assert!(WATCHDOG_FORM.writable_keys().contains(&"automatic-supout"));
        assert_eq!(
            WATCHDOG_FORM
                .field("watchdog-timer")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            WATCHDOG_FORM
                .field("watch-interval")
                .map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert_eq!(
            WATCHDOG_FORM.field("no-ping-delay").map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert_eq!(
            WATCHDOG_FORM.field("send-email-to").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert!(WATCHDOG_FORM.create_sections.is_empty());
        assert!(no_advanced(&WATCHDOG_FORM));
        assert_eq!(NOTE_FORM.writable_keys(), ["show-at-login", "note"]);
    }

    #[test]
    fn package_name_is_readonly_disabled_toggle() {
        assert!(PACKAGE_FORM.create_sections.is_empty());
        assert_eq!(PACKAGE_FORM.writable_keys(), ["disabled"]);
        assert_eq!(
            PACKAGE_FORM.field("name").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(!PACKAGE_FORM.writable_keys().contains(&"version"));
        assert!(!PACKAGE_FORM.writable_keys().contains(&"build-time"));
        assert!(!PACKAGE_FORM.writable_keys().contains(&"scheduled"));
    }

    #[test]
    fn license_is_inspect_only_and_key_is_secret() {
        assert!(LICENSE_FORM.create_sections.is_empty());
        assert!(LICENSE_FORM.writable_keys().is_empty());
        assert!(
            LICENSE_FORM
                .sections
                .iter()
                .all(|section| section.id == "status" && section.read_only)
        );
        assert_label(&LICENSE_FORM, "software-id", "Software ID");
        assert_label(&LICENSE_FORM, "nlevel", "Level");
        assert_label(&LICENSE_FORM, "system-id", "System ID");
        assert_eq!(
            LICENSE_FORM.field("nlevel").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_eq!(
            LICENSE_IMPORT_PROMPT.field("k").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_label(&LICENSE_IMPORT_PROMPT, "k", "License Key");
        assert_lookup(&LICENSE_IMPORT_PROMPT, "file-name", "files", "name");
        let mut original = HashMap::new();
        original.insert("k".into(), "********".into());
        original.insert("file-name".into(), String::new());
        let mut current = original.clone();
        current.insert("k".into(), "********".into());
        current.insert("file-name".into(), "chr.key".into());
        let body = patch_body(&LICENSE_IMPORT_PROMPT, &original, &current, "********");
        assert_eq!(body.get("file-name").map(String::as_str), Some("chr.key"));
        assert!(!body.contains_key("k"));
    }

    #[test]
    fn disk_form_uses_webfig_field_kinds() {
        assert_eq!(create_keys(&DISK_FORM), ["type", "slot"]);
        assert_enum(&DISK_FORM, "type", DISK_TYPES);
        assert_enum(&DISK_FORM, "raid-type", RAID_TYPES);
        assert_enum(&DISK_FORM, "raid-chunk-size", RAID_CHUNK_SIZES);
        assert_lookup(&DISK_FORM, "parent", "disks", "slot");
        assert_lookup(&DISK_FORM, "raid-master", "disks", "slot");
        assert_lookup(&DISK_FORM, "crypted-backend", "disks", "slot");
        assert_lookup(&DISK_FORM, "file-path", "files", "name");
        assert_lookup(&DISK_FORM, "media-interface", "interfaces", "name");
        assert_lookup(&DISK_FORM, "smb-server-user", "users", "name");
        assert_eq!(
            DISK_FORM.field("raid-role").map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_eq!(
            DISK_FORM.field("sshfs-port").map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_eq!(
            DISK_FORM.field("mount-filesystem").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            DISK_FORM.field("encryption-key").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            DISK_FORM.field("sshfs-password").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            DISK_FORM.field("model").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(!DISK_FORM.writable_keys().contains(&"size"));
        assert!(!DISK_FORM.writable_keys().contains(&"fs"));
        assert_label(&DISK_FORM, "raid-master", "RAID Master");
        assert_enum(&FORMAT_DISK_PROMPT, "file-system", FORMAT_FILE_SYSTEMS);
        assert_eq!(FORMAT_FILE_SYSTEMS[0], "ext4");
        assert_eq!(
            FORMAT_DISK_PROMPT
                .field("mbr-partition-table")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_label(&FORMAT_DISK_PROMPT, "file-system", "File System");
    }

    #[test]
    fn device_mode_flags_are_toggles_not_text() {
        assert!(DEVICE_MODE_FORM.create_sections.is_empty());
        assert_enum(&DEVICE_MODE_FORM, "mode", DEVICE_MODE_MODES);
        assert_eq!(DEVICE_MODE_MODES, ["advanced", "home", "basic", "rose"]);
        for key in [
            "container",
            "scheduler",
            "traffic-gen",
            "fetch",
            "flagged",
            "flagging-enabled",
        ] {
            assert_eq!(
                DEVICE_MODE_FORM.field(key).map(|field| field.kind),
                Some(FieldKind::Toggle),
                "{key} must be a toggle"
            );
        }
        assert_label(&DEVICE_MODE_FORM, "traffic-gen", "Traffic Generator");
        assert_label(&DEVICE_MODE_FORM, "bandwidth-test", "Bandwidth Test");
        assert_eq!(
            DEVICE_MODE_FORM
                .field("allowed-versions")
                .map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_eq!(
            DEVICE_MODE_FORM
                .field("attempt-count")
                .map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert!(!DEVICE_MODE_FORM.writable_keys().contains(&"attempt-count"));
        assert!(no_advanced(&DEVICE_MODE_FORM));
    }

    #[test]
    fn console_port_is_lookup_and_channel_is_number() {
        assert_lookup(&CONSOLE_FORM, "port", "ports", "name");
        assert_eq!(
            CONSOLE_FORM.field("channel").map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_eq!(
            CONSOLE_FORM.field("disabled").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert!(!CONSOLE_FORM.writable_keys().contains(&"used"));
        assert_eq!(create_keys(&CONSOLE_FORM), CONSOLE_FORM.writable_keys());
        assert!(no_advanced(&CONSOLE_FORM));
    }

    #[test]
    fn led_type_is_enum_and_lookups_are_not_text() {
        assert_enum(&LED_FORM, "type", LED_TYPES);
        assert_lookup(&LED_FORM, "interface", "interfaces", "name");
        assert_lookup(&LED_FORM, "modem", "interfaces", "name");
        assert_eq!(
            LED_FORM.field("leds").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_enum(&LED_SETTINGS_FORM, "all-leds-off", LED_ALL_OFF);
        assert!(LED_SETTINGS_FORM.create_sections.is_empty());
        assert!(no_advanced(&LED_FORM));
        assert!(no_advanced(&LED_SETTINGS_FORM));
    }

    #[test]
    fn port_serial_fields_are_enums() {
        assert_eq!(
            PORT_FORM.field("name").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_enum(&PORT_FORM, "baud-rate", PORT_BAUD);
        assert_enum(&PORT_FORM, "data-bits", PORT_DATA_BITS);
        assert_enum(&PORT_FORM, "parity", PORT_PARITY);
        assert_enum(&PORT_FORM, "stop-bits", PORT_STOP_BITS);
        assert_enum(&PORT_FORM, "flow-control", PORT_FLOW);
        assert!(PORT_FORM.create_sections.is_empty());
        assert!(no_advanced(&PORT_FORM));
    }

    #[test]
    fn special_login_user_and_port_are_lookups() {
        assert_lookup(&SPECIAL_LOGIN_FORM, "user", "users", "name");
        assert_lookup(&SPECIAL_LOGIN_FORM, "port", "ports", "name");
        assert_eq!(
            create_keys(&SPECIAL_LOGIN_FORM),
            ["user", "port", "disabled"]
        );
        assert!(no_advanced(&SPECIAL_LOGIN_FORM));
    }

    #[test]
    fn reset_configuration_prompt_covers_webfig_flags() {
        assert_eq!(
            RESET_CONFIG_PROMPT.writable_keys(),
            [
                "keep-users",
                "no-defaults",
                "skip-backup",
                "caps-mode",
                "run-after-reset"
            ]
        );
        assert_eq!(
            RESET_CONFIG_PROMPT
                .field("keep-users")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            RESET_CONFIG_PROMPT
                .field("caps-mode")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_lookup(&RESET_CONFIG_PROMPT, "run-after-reset", "files", "name");
        assert_eq!(
            create_keys(&RESET_CONFIG_PROMPT),
            RESET_CONFIG_PROMPT.writable_keys()
        );
        assert!(no_advanced(&RESET_CONFIG_PROMPT));
    }

    #[test]
    fn routerboard_settings_and_buttons_use_field_kinds() {
        assert_enum(&ROUTERBOARD_SETTINGS_FORM, "boot-os", BOOT_OS);
        assert_enum(&ROUTERBOARD_SETTINGS_FORM, "boot-device", BOOT_DEVICE);
        assert_enum(
            &ROUTERBOARD_SETTINGS_FORM,
            "protected-routerboot",
            PROTECTED_ROUTERBOOT,
        );
        assert_eq!(
            ROUTERBOARD_SETTINGS_FORM
                .field("silent-boot")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            ROUTERBOARD_MODE_BUTTON_FORM
                .field("hold-time")
                .map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert_lookup(&ROUTERBOARD_MODE_BUTTON_FORM, "on-event", "scripts", "name");
        assert_lookup(
            &ROUTERBOARD_RESET_BUTTON_FORM,
            "on-event",
            "scripts",
            "name",
        );
        assert_eq!(
            USB_POWER_RESET_PROMPT
                .field("duration")
                .map(|field| field.kind),
            Some(FieldKind::Time)
        );
        assert!(ROUTERBOARD_SETTINGS_FORM.create_sections.is_empty());
        assert!(no_advanced(&ROUTERBOARD_SETTINGS_FORM));
        assert!(no_advanced(&ROUTERBOARD_MODE_BUTTON_FORM));
        assert!(no_advanced(&ROUTERBOARD_RESET_BUTTON_FORM));
    }
}
