//! Form schemas for the `Tools` navigation group.
//!
//! Catalog wiring (do not register here):
//! - `netwatch` → `/tool/netwatch` (`NETWATCH_FORM`)
//! - `email` → `/tool/e-mail` (`EMAIL_FORM`)
//! - `romon` → `/tool/romon` (`ROMON_FORM`)
//! - `romon-ports` → `/tool/romon/port` (`ROMON_PORT_FORM`)
//! - `graphing` → `/tool/graphing` (`GRAPHING_FORM`)
//! - `graphing-interface` → `/tool/graphing/interface` (`GRAPHING_INTERFACE_FORM`)
//! - `graphing-queue` → `/tool/graphing/queue` (`GRAPHING_QUEUE_FORM`)
//! - `graphing-resource` → `/tool/graphing/resource` (`GRAPHING_RESOURCE_FORM`)
//! - `sniffer` → `/tool/sniffer` (`SNIFFER_FORM`)
//! - `ping` / `traceroute` / `bandwidth-test` / `flood-ping` / `mac-scan` /
//!   `ip-scan` / `profiler` → overlay-only (`FetchKind::Local`, no catalog form)
//! - `wol` / `sms` → overlay prompts (`WOL_PROMPT`, `SMS_PROMPT`), catalog `form: None`
//!
//! Group id: `tools-group`.

use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection, ScalarKind};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_SCRIPT: FieldKind = FieldKind::Lookup {
    resource_id: "scripts",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SIMPLE_QUEUE: FieldKind = FieldKind::Lookup {
    resource_id: "queue-simple",
    value_key: "name",
    multiple: false,
};

const STORE_EVERY: &[&str] = &["5min", "hour", "24hours"];
const NETWATCH_TYPE: &[&str] = &["icmp", "simple", "tcp-conn", "http", "https", "dns"];
const TLS_VALUES: &[&str] = &["yes", "starttls", "no"];

const SECRETS: FieldSpec = f!("secrets", "Secrets", FieldKind::Secret);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const HOST: FieldSpec = f!("host", "Host", FieldKind::Text);
const UP_SCRIPT: FieldSpec = f!("up-script", "Up script", LOOKUP_SCRIPT);
const DOWN_SCRIPT: FieldSpec = f!("down-script", "Down script", LOOKUP_SCRIPT);

pub static NETWATCH_FORM: FormSchema = FormSchema {
    title_key: "host",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                HOST,
                f!(
                    "type",
                    "Type",
                    FieldKind::Enum {
                        values: NETWATCH_TYPE,
                    }
                ),
                f!("interval", "Interval", FieldKind::Time),
                f!("timeout", "Timeout", FieldKind::Time),
                f!("start-delay", "Start delay", FieldKind::Time),
                UP_SCRIPT,
                DOWN_SCRIPT,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("since", "Since", FieldKind::Readonly),
                f!("done-tests", "Done tests", FieldKind::Readonly),
                f!("failed-tests", "Failed tests", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static EMAIL_FORM: FormSchema = FormSchema {
    title_key: "server",
    subtitle_keys: &["from"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("server", "Server", FieldKind::Text),
            f!("from", "From", FieldKind::Text),
            f!("user", "User", FieldKind::Text),
            f!("password", "Password", FieldKind::Secret),
            f!("tls", "TLS", FieldKind::Enum { values: TLS_VALUES }),
            f!("port", "Port", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};

pub static SNIFFER_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["file-name"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", LOOKUP_IFACE),
            f!("file-name", "File name", FieldKind::Text),
            f!("file-limit", "File limit", FieldKind::Text),
            f!("filter-stream", "Filter stream", FieldKind::Toggle),
            f!("filter-interface", "Filter interface", LOOKUP_IFACE),
        ],
    }],
    create_sections: &[],
};

pub static WOL_PROMPT: FormSchema = FormSchema {
    title_key: "mac",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", LOOKUP_IFACE),
            f!("mac", "MAC", FieldKind::Mac),
        ],
    }],
    create_sections: &[],
};

pub static SMS_PROMPT: FormSchema = FormSchema {
    title_key: "phone-number",
    subtitle_keys: &["message"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("phone-number", "Phone", FieldKind::Text),
            f!("message", "Message", FieldKind::Text),
            f!("channel", "Channel", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};

pub static ROMON_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["id"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("enabled", "Enabled", FieldKind::Toggle),
                f!(
                    "id",
                    "ID",
                    FieldKind::Optional {
                        kind: ScalarKind::Mac,
                        unset: "00:00:00:00:00:00",
                        unset_label: "auto",
                    }
                ),
                SECRETS,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("current-id", "Current ID", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static ROMON_PORT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["cost"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", LOOKUP_IFACE),
            f!("forbid", "Forbid", FieldKind::Toggle),
            f!("cost", "Cost", FieldKind::Number),
            SECRETS,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static GRAPHING_FORM: FormSchema = FormSchema {
    title_key: "store-every",
    subtitle_keys: &["page-refresh"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!(
                "store-every",
                "Store Every",
                FieldKind::Enum {
                    values: STORE_EVERY,
                }
            ),
            f!("page-refresh", "Page Refresh", FieldKind::Time),
        ],
    }],
    create_sections: &[],
};

const GRAPHING_ALLOW_ADDRESS: FieldSpec = f!("allow-address", "Allow Address", FieldKind::Ip);
const GRAPHING_STORE_ON_DISK: FieldSpec = f!("store-on-disk", "Store On Disk", FieldKind::Toggle);

pub static GRAPHING_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["allow-address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", LOOKUP_IFACE),
            GRAPHING_ALLOW_ADDRESS,
            GRAPHING_STORE_ON_DISK,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static GRAPHING_QUEUE_FORM: FormSchema = FormSchema {
    title_key: "simple-queue",
    subtitle_keys: &["allow-address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("simple-queue", "Simple Queue", LOOKUP_SIMPLE_QUEUE),
            GRAPHING_ALLOW_ADDRESS,
            f!("allow-target", "Allow Target", FieldKind::Toggle),
            GRAPHING_STORE_ON_DISK,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static GRAPHING_RESOURCE_FORM: FormSchema = FormSchema {
    title_key: "allow-address",
    subtitle_keys: &["store-on-disk"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            GRAPHING_ALLOW_ADDRESS,
            GRAPHING_STORE_ON_DISK,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};
