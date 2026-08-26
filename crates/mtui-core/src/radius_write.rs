//! Form schemas for the RADIUS nav group.
//!
//! Catalog wiring (do not register here):
//! - `radius` → `/radius` (`RADIUS_FORM`)
//!
//! Group id: `radius-group`.

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

const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const SECRET: FieldSpec = f!("secret", "Secret", FieldKind::Secret);
const SERVICE: FieldSpec = f!("service", "Service", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);

pub static RADIUS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["service"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ADDRESS,
            f!("protocol", "Protocol", FieldKind::Text),
            SECRET,
            SERVICE,
            f!("authentication-port", "Auth port", FieldKind::Number),
            f!("accounting-port", "Acct port", FieldKind::Number),
            f!("timeout", "Timeout", FieldKind::Text),
            f!("src-address", "Src address", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, SECRET, SERVICE],
    }],
};

pub static RADIUS_INCOMING_FORM: FormSchema = FormSchema {
    title_key: "accept",
    subtitle_keys: &["port"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("accept", "Accept", FieldKind::Toggle),
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
        schema.create_keys()
    }

    #[test]
    fn radius_create_requires_identity_and_secret() {
        assert_eq!(create_keys(&RADIUS_FORM), RADIUS_FORM.writable_keys());
        assert_eq!(
            RADIUS_FORM.field("secret").map(|f| f.kind),
            Some(FieldKind::Secret)
        );
        assert!(RADIUS_FORM.writable_keys().contains(&"authentication-port"));
        assert!(
            RADIUS_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn patch_body_keeps_masked_radius_secret() {
        let mut original = HashMap::new();
        original.insert("address".into(), "192.0.2.10".into());
        original.insert("secret".into(), "********".into());
        original.insert("service".into(), "login".into());
        original.insert("timeout".into(), "300ms".into());
        let mut current = original.clone();
        current.insert("timeout".into(), "1s".into());
        current.insert("secret".into(), "********".into());
        let body = patch_body(&RADIUS_FORM, &original, &current, "********");
        assert_eq!(body.get("timeout").map(String::as_str), Some("1s"));
        assert!(!body.contains_key("secret"));
        assert!(!body.contains_key("address"));
    }

    #[test]
    fn patch_body_sends_changed_radius_secret() {
        let mut original = HashMap::new();
        original.insert("secret".into(), "********".into());
        let mut current = original.clone();
        current.insert("secret".into(), "new-shared-secret".into());
        let body = patch_body(&RADIUS_FORM, &original, &current, "********");
        assert_eq!(
            body.get("secret").map(String::as_str),
            Some("new-shared-secret")
        );
    }
}
