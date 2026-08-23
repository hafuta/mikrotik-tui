//! Form schemas for the Tools nav group.
//!
//! Catalog wiring (do not register here):
//! - `netwatch` → `/rest/tool/netwatch` (`NETWATCH_FORM`)
//! - `email` → `/rest/tool/email` (`EMAIL_FORM`)
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

    #[test]
    fn netwatch_create_is_host_only() {
        assert_eq!(create_keys(&NETWATCH_FORM), ["host"]);
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
}
