//! Form schemas for the Tools nav group.
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
//! - `ping` / `traceroute` → overlay-only (`FetchKind::Local`, no form)
//!
//! Group id: `tools-group`.

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
const SECRETS: FieldSpec = f!("secrets", "Secrets", FieldKind::Repeat);

const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
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
                f!("type", "Type", FieldKind::Text),
                f!("interval", "Interval", FieldKind::Text),
                f!("timeout", "Timeout", FieldKind::Text),
                f!("start-delay", "Start delay", FieldKind::Text),
                UP_SCRIPT,
                DOWN_SCRIPT,
                COMMENT,
                DISABLED,
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[HOST],
    }],
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
            f!("tls", "TLS", FieldKind::Text),
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
            f!("mac", "MAC", FieldKind::Text),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", LOOKUP_IFACE),
            f!("mac", "MAC", FieldKind::Text),
        ],
    }],
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
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("phone-number", "Phone", FieldKind::Text),
            f!("message", "Message", FieldKind::Text),
        ],
    }],
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
                f!("id", "ID", FieldKind::Text),
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
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("interface", "Interface", LOOKUP_IFACE)],
    }],
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
            f!("page-refresh", "Page Refresh", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

const GRAPHING_ALLOW_ADDRESS: FieldSpec = f!("allow-address", "Allow Address", FieldKind::Text);
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
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("interface", "Interface", LOOKUP_IFACE)],
    }],
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
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("simple-queue", "Simple Queue", LOOKUP_SIMPLE_QUEUE)],
    }],
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
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[GRAPHING_ALLOW_ADDRESS],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::patch_body;
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema.create_keys()
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

    #[test]
    fn netwatch_create_is_host_only() {
        assert_eq!(create_keys(&NETWATCH_FORM), NETWATCH_FORM.writable_keys());
        assert!(!NETWATCH_FORM.writable_keys().contains(&"status"));
        assert!(!NETWATCH_FORM.writable_keys().contains(&"done-tests"));
        assert!(NETWATCH_FORM.writable_keys().contains(&"up-script"));
        assert_lookup(&NETWATCH_FORM, "up-script", "scripts", "name");
        assert_lookup(&NETWATCH_FORM, "down-script", "scripts", "name");
        assert_eq!(
            NETWATCH_FORM.field("host").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            NETWATCH_FORM.field("interval").map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }

    #[test]
    fn email_is_singleton_without_create() {
        assert!(EMAIL_FORM.create_sections.is_empty());
        assert_eq!(
            EMAIL_FORM.writable_keys(),
            ["server", "from", "user", "password", "tls", "port"]
        );
        assert_eq!(
            EMAIL_FORM.field("password").map(|f| f.kind),
            Some(FieldKind::Secret)
        );
    }

    #[test]
    fn patch_body_keeps_masked_email_password() {
        let mut original = HashMap::new();
        original.insert("server".into(), "smtp.example.com".into());
        original.insert("password".into(), "********".into());
        original.insert("port".into(), "587".into());
        let mut current = original.clone();
        current.insert("port".into(), "465".into());
        current.insert("password".into(), "********".into());
        let body = patch_body(&EMAIL_FORM, &original, &current, "********");
        assert_eq!(body.get("port").map(String::as_str), Some("465"));
        assert!(!body.contains_key("password"));
    }

    fn no_advanced(schema: &FormSchema) -> bool {
        schema
            .sections
            .iter()
            .all(|section| section.id != "advanced")
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

    #[test]
    fn romon_is_singleton_with_status_and_secret_list() {
        assert!(ROMON_FORM.create_sections.is_empty());
        assert_eq!(ROMON_FORM.writable_keys(), ["enabled", "id", "secrets"]);
        assert!(!ROMON_FORM.writable_keys().contains(&"current-id"));
        assert!(
            ROMON_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        assert!(no_advanced(&ROMON_FORM));
        assert_eq!(
            ROMON_FORM.field("enabled").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            ROMON_FORM.field("id").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            ROMON_FORM.field("secrets").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_ne!(
            ROMON_FORM.field("secrets").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            ROMON_FORM.field("current-id").map(|field| field.kind),
            Some(FieldKind::Readonly)
        );
        assert_label(&ROMON_FORM, "id", "ID");
        assert_label(&ROMON_FORM, "secrets", "Secrets");
        assert_label(&ROMON_FORM, "current-id", "Current ID");
    }

    #[test]
    fn romon_port_create_is_interface_with_lookup_and_repeat_secrets() {
        assert_eq!(
            create_keys(&ROMON_PORT_FORM),
            ROMON_PORT_FORM.writable_keys()
        );
        assert_eq!(
            ROMON_PORT_FORM.writable_keys(),
            [
                "interface",
                "forbid",
                "cost",
                "secrets",
                "comment",
                "disabled"
            ]
        );
        assert!(no_advanced(&ROMON_PORT_FORM));
        assert_lookup(&ROMON_PORT_FORM, "interface", "interfaces", "name");
        assert_eq!(
            ROMON_PORT_FORM.field("forbid").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            ROMON_PORT_FORM.field("cost").map(|field| field.kind),
            Some(FieldKind::Number)
        );
        assert_eq!(
            ROMON_PORT_FORM.field("secrets").map(|field| field.kind),
            Some(FieldKind::Repeat)
        );
        assert_eq!(
            ROMON_PORT_FORM.field("disabled").map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_label(&ROMON_PORT_FORM, "forbid", "Forbid");
        assert_label(&ROMON_PORT_FORM, "cost", "Cost");
        assert!(FieldKind::Number.accepts_char("cost", "100", '0'));
        assert!(!FieldKind::Number.accepts_char("cost", "100", 'a'));
    }

    #[test]
    fn romon_patch_omits_masked_secrets_and_readonly_current_id() {
        let mut original = HashMap::new();
        original.insert("enabled".into(), "true".into());
        original.insert("id".into(), "00:00:00:00:00:00".into());
        original.insert("secrets".into(), "********".into());
        original.insert("current-id".into(), "DC:2C:6E:9E:11:27".into());
        let mut current = original.clone();
        current.insert("enabled".into(), "false".into());
        current.insert("secrets".into(), "********".into());
        current.insert("current-id".into(), "aa:bb:cc:dd:ee:ff".into());
        let body = patch_body(&ROMON_FORM, &original, &current, "********");
        assert_eq!(body.get("enabled").map(String::as_str), Some("false"));
        assert!(!body.contains_key("secrets"));
        assert!(!body.contains_key("current-id"));
    }

    #[test]
    fn graphing_settings_use_store_every_enum() {
        assert!(GRAPHING_FORM.create_sections.is_empty());
        assert_eq!(
            GRAPHING_FORM.writable_keys(),
            ["store-every", "page-refresh"]
        );
        assert!(no_advanced(&GRAPHING_FORM));
        assert_enum(&GRAPHING_FORM, "store-every", STORE_EVERY);
        assert_eq!(STORE_EVERY[0], "5min");
        assert_eq!(
            GRAPHING_FORM.field("page-refresh").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_label(&GRAPHING_FORM, "store-every", "Store Every");
        assert_label(&GRAPHING_FORM, "page-refresh", "Page Refresh");
    }

    #[test]
    fn graphing_children_use_lookups_toggles_and_short_create() {
        assert_eq!(
            create_keys(&GRAPHING_INTERFACE_FORM),
            GRAPHING_INTERFACE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&GRAPHING_QUEUE_FORM),
            GRAPHING_QUEUE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&GRAPHING_RESOURCE_FORM),
            GRAPHING_RESOURCE_FORM.writable_keys()
        );
        assert_lookup(&GRAPHING_INTERFACE_FORM, "interface", "interfaces", "name");
        assert_lookup(&GRAPHING_QUEUE_FORM, "simple-queue", "queue-simple", "name");
        assert_eq!(
            GRAPHING_INTERFACE_FORM
                .field("store-on-disk")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            GRAPHING_QUEUE_FORM
                .field("allow-target")
                .map(|field| field.kind),
            Some(FieldKind::Toggle)
        );
        assert_eq!(
            GRAPHING_INTERFACE_FORM
                .field("allow-address")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            GRAPHING_INTERFACE_FORM
                .field("comment")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert!(no_advanced(&GRAPHING_INTERFACE_FORM));
        assert!(no_advanced(&GRAPHING_QUEUE_FORM));
        assert!(no_advanced(&GRAPHING_RESOURCE_FORM));
        assert_label(&GRAPHING_INTERFACE_FORM, "allow-address", "Allow Address");
        assert_label(&GRAPHING_INTERFACE_FORM, "store-on-disk", "Store On Disk");
        assert_label(&GRAPHING_QUEUE_FORM, "simple-queue", "Simple Queue");
        assert_label(&GRAPHING_QUEUE_FORM, "allow-target", "Allow Target");
        assert_eq!(
            GRAPHING_INTERFACE_FORM.writable_keys(),
            [
                "interface",
                "allow-address",
                "store-on-disk",
                "comment",
                "disabled"
            ]
        );
        assert_eq!(
            GRAPHING_QUEUE_FORM.writable_keys(),
            [
                "simple-queue",
                "allow-address",
                "allow-target",
                "store-on-disk",
                "comment",
                "disabled"
            ]
        );
        assert_eq!(
            GRAPHING_RESOURCE_FORM.writable_keys(),
            ["allow-address", "store-on-disk", "comment", "disabled"]
        );
    }

    #[test]
    fn graphing_optional_comment_is_omitted_from_create_and_unchanged_patch() {
        let mut original = HashMap::new();
        original.insert("interface".into(), "all".into());
        original.insert("allow-address".into(), "0.0.0.0/0".into());
        original.insert("store-on-disk".into(), "true".into());
        let mut current = original.clone();
        current.insert("store-on-disk".into(), "false".into());
        let body = patch_body(&GRAPHING_INTERFACE_FORM, &original, &current, "********");
        assert_eq!(body.get("store-on-disk").map(String::as_str), Some("false"));
        assert!(!body.contains_key("comment"));
        assert!(!body.contains_key("interface"));
    }
}
