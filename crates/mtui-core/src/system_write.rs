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
//! - `/rest/snmp`
//! - `/rest/snmp/community`
//! - `/rest/certificate`
//! - `/rest/system/watchdog`
//! - `/rest/system/note`
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

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const POLICY: FieldSpec = f!("policy", "Policy", FieldKind::Text);
const PASSWORD: FieldSpec = f!("password", "Password", FieldKind::Secret);
const SOURCE: FieldSpec = f!("source", "Source", FieldKind::Text);
const GROUP: FieldSpec = f!("group", "Group", LOOKUP_USER_GROUP);
const ON_EVENT: FieldSpec = f!("on-event", "On event", LOOKUP_SCRIPT);
const CA: FieldSpec = f!("ca", "CA", LOOKUP_CERTIFICATE);
const FILE_NAME: FieldSpec = f!("file-name", "File name", LOOKUP_FILE);

pub static USER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["group"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[NAME, GROUP, PASSWORD, COMMENT, DISABLED],
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
        fields: &[NAME, GROUP, PASSWORD],
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
            f!("multicast", "Multicast", FieldKind::Toggle),
            f!("manycast", "Manycast", FieldKind::Toggle),
            f!(
                "broadcast-addresses",
                "Broadcast addresses",
                FieldKind::Text
            ),
            f!("vrf", "VRF", FieldKind::Text),
            f!("use-local-clock", "Use local clock", FieldKind::Toggle),
            f!(
                "local-clock-stratum",
                "Local clock stratum",
                FieldKind::Text
            ),
            f!("auth-key", "Auth key", FieldKind::Text),
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

pub static SCHEDULER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interval"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("start-date", "Start date", FieldKind::Text),
                f!("start-time", "Start time", FieldKind::Text),
                f!("interval", "Interval", FieldKind::Text),
                ON_EVENT,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("next-run", "Next run", FieldKind::Readonly),
                f!("run-count", "Run count", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, ON_EVENT],
    }],
};

pub static SCRIPT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["policy"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            SOURCE,
            POLICY,
            f!(
                "dont-require-permissions",
                "Skip permissions",
                FieldKind::Toggle
            ),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, SOURCE],
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
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("target", "Target", FieldKind::Text),
                f!("remote", "Remote", FieldKind::Text),
                f!("remote-port", "Remote port", FieldKind::Number),
                f!("src-address", "Src address", FieldKind::Text),
                f!("remote-protocol", "Protocol", FieldKind::Text),
                f!("remote-log-format", "Log format", FieldKind::Text),
                f!("syslog-facility", "Facility", FieldKind::Text),
                f!("syslog-severity", "Severity", FieldKind::Text),
                f!("syslog-time-format", "Time format", FieldKind::Text),
                f!("check-certificate", "Check cert", FieldKind::Toggle),
                f!("vrf", "VRF", FieldKind::Text),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("memory-lines", "Memory lines", FieldKind::Number),
                f!("memory-stop-on-full", "Mem stop full", FieldKind::Toggle),
                f!("disk-file-name", "Disk file", FieldKind::Text),
                f!("disk-lines-per-file", "Disk lines", FieldKind::Number),
                f!("disk-file-count", "Disk files", FieldKind::Number),
                f!("disk-stop-on-full", "Disk stop full", FieldKind::Toggle),
                f!("email-to", "Email to", FieldKind::Text),
                f!("email-cc", "Email CC", FieldKind::Text),
                f!("email-start-tls", "Email STARTTLS", FieldKind::Toggle),
                f!("script", "Script", LOOKUP_SCRIPT),
                f!("remember", "Remember", FieldKind::Toggle),
                f!("add-topics-string", "Add topics", FieldKind::Toggle),
                f!("cef-event-delimiter", "CEF delimiter", FieldKind::Text),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("target", "Target", FieldKind::Text)],
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
            f!("watch-address", "Watch address", FieldKind::Text),
            f!("watch-interval", "Watch interval", FieldKind::Text),
            f!("ping-start-after", "Ping start", FieldKind::Text),
            f!("ping-timeout", "Ping timeout", FieldKind::Text),
            f!("automatic-supout", "Auto supout", FieldKind::Toggle),
            f!("auto-send-supout", "Send supout", FieldKind::Toggle),
            f!("send-email-to", "Email to", FieldKind::Text),
            f!("send-smtp-server", "SMTP server", FieldKind::Text),
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

pub static RESET_CONFIG_PROMPT: FormSchema = FormSchema {
    title_key: "keep-users",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("keep-users", "Keep users", FieldKind::Toggle),
            f!("no-defaults", "No defaults", FieldKind::Toggle),
            f!("skip-backup", "Skip backup", FieldKind::Toggle),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("keep-users", "Keep users", FieldKind::Toggle),
            f!("no-defaults", "No defaults", FieldKind::Toggle),
        ],
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
            ["name", "group", "password", "comment", "disabled"]
        );
        assert_eq!(
            USER_FORM.field("password").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert!(!USER_FORM.writable_keys().contains(&"last-logged-in"));
        assert_eq!(create_keys(&USER_FORM), ["name", "group", "password"]);
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
                "multicast",
                "manycast",
                "broadcast-addresses",
                "vrf",
                "use-local-clock",
                "local-clock-stratum",
                "auth-key",
            ]
        );
        assert!(no_advanced(&NTP_SERVER_FORM));
        assert_eq!(
            NTP_SERVER_FORM.field("auth-key").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_ne!(
            NTP_SERVER_FORM.field("auth-key").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
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
    fn scheduler_create_is_name_and_on_event() {
        assert_eq!(create_keys(&SCHEDULER_FORM), ["name", "on-event"]);
        assert_lookup(&SCHEDULER_FORM, "on-event", "scripts", "name");
        assert_eq!(
            SCHEDULER_FORM.field("interval").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert!(!SCHEDULER_FORM.writable_keys().contains(&"next-run"));
        assert!(!SCHEDULER_FORM.writable_keys().contains(&"run-count"));
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
        assert_eq!(create_keys(&SCRIPT_FORM), ["name", "source"]);
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
        assert_eq!(create_keys(&LOGGING_ACTION_FORM), ["name", "target"]);
        let writable = LOGGING_ACTION_FORM.writable_keys();
        assert!(writable.contains(&"remote"));
        assert!(writable.contains(&"remote-port"));
        let advanced = LOGGING_ACTION_FORM
            .sections
            .iter()
            .find(|section| section.id == "advanced")
            .expect("advanced");
        assert!(!advanced.read_only);
        assert!(!advanced.fields.is_empty());
        assert_lookup(&LOGGING_ACTION_FORM, "script", "scripts", "name");
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
            WATCHDOG_FORM.field("send-email-to").map(|field| field.kind),
            Some(FieldKind::Text)
        );
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
}
