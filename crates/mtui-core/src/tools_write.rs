//! Form schemas for the Tools nav group.
//!
//! Catalog wiring (do not register here):
//! - `netwatch` → `/rest/tool/netwatch` (`NETWATCH_FORM`)
//! - `email` → `/rest/tool/email` (`EMAIL_FORM`)
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

const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const HOST: FieldSpec = f!("host", "Host", FieldKind::Text);

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
                f!("up-script", "Up script", FieldKind::Text),
                f!("down-script", "Down script", FieldKind::Text),
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

    #[test]
    fn netwatch_create_is_host_only() {
        assert_eq!(create_keys(&NETWATCH_FORM), ["host"]);
        assert!(!NETWATCH_FORM.writable_keys().contains(&"status"));
        assert!(!NETWATCH_FORM.writable_keys().contains(&"done-tests"));
        assert!(NETWATCH_FORM.writable_keys().contains(&"up-script"));
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
