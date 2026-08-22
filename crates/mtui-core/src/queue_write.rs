//! Form schemas for the Queue nav group.
//!
//! Catalog wiring (do not register here):
//! - `queue-simple` → `/rest/queue/simple` (`QUEUE_SIMPLE_FORM`)
//! - `queue-tree` → `/rest/queue/tree` (`QUEUE_TREE_FORM`)
//! - `queue-type` → `/rest/queue/type` (`QUEUE_TYPE_FORM`)
//! - `queue-interface` → `/rest/queue/interface` (`QUEUE_INTERFACE_FORM`)
//!
//! Group id: `queue-group`.

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

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const MAX_LIMIT: FieldSpec = f!("max-limit", "Max limit", FieldKind::Text);

pub static QUEUE_SIMPLE_FORM: FormSchema = FormSchema {
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
                MAX_LIMIT,
                f!("burst-limit", "Burst limit", FieldKind::Text),
                f!("burst-threshold", "Burst threshold", FieldKind::Text),
                f!("burst-time", "Burst time", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("rate", "Rate", FieldKind::Readonly),
                f!("packet-rate", "Packet rate", FieldKind::Readonly),
                f!("queued-bytes", "Queued bytes", FieldKind::Readonly),
                f!("queued-packets", "Queued packets", FieldKind::Readonly),
                f!("dropped", "Dropped", FieldKind::Readonly),
                f!("borrowed", "Borrowed", FieldKind::Readonly),
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

pub static QUEUE_TREE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["parent"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("parent", "Parent", FieldKind::Text),
            f!("packet-mark", "Packet mark", FieldKind::Text),
            MAX_LIMIT,
            f!("limit-at", "Limit at", FieldKind::Text),
            f!("priority", "Priority", FieldKind::Text),
            f!("bucket-size", "Bucket size", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("parent", "Parent", FieldKind::Text)],
    }],
};

pub static QUEUE_TYPE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["kind"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("kind", "Kind", FieldKind::Text),
            f!("pfifo-limit", "PFIFO limit", FieldKind::Text),
            f!("sfq-perturb", "SFQ perturb", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("kind", "Kind", FieldKind::Text)],
    }],
};

pub static QUEUE_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["queue"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", FieldKind::Readonly),
            f!("queue", "Queue", FieldKind::Text),
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
    fn queue_simple_status_is_runtime_only() {
        assert_eq!(create_keys(&QUEUE_SIMPLE_FORM), ["name", "target"]);
        assert!(!QUEUE_SIMPLE_FORM.writable_keys().contains(&"rate"));
        assert!(!QUEUE_SIMPLE_FORM.writable_keys().contains(&"dropped"));
        assert!(QUEUE_SIMPLE_FORM.writable_keys().contains(&"burst-time"));
    }

    #[test]
    fn queue_tree_and_type_create() {
        assert_eq!(create_keys(&QUEUE_TREE_FORM), ["name", "parent"]);
        assert_eq!(create_keys(&QUEUE_TYPE_FORM), ["name", "kind"]);
        assert!(QUEUE_TREE_FORM.writable_keys().contains(&"packet-mark"));
        assert!(QUEUE_TYPE_FORM.writable_keys().contains(&"pfifo-limit"));
    }

    #[test]
    fn queue_interface_has_no_create() {
        assert!(QUEUE_INTERFACE_FORM.create_sections.is_empty());
        assert_eq!(QUEUE_INTERFACE_FORM.writable_keys(), ["queue"]);
        assert_eq!(QUEUE_INTERFACE_FORM.known_keys(), ["interface", "queue"]);
    }

    #[test]
    fn patch_body_skips_simple_counters() {
        let mut original = HashMap::new();
        original.insert("name".into(), "lan".into());
        original.insert("target".into(), "192.168.1.0/24".into());
        original.insert("rate".into(), "0/0".into());
        let mut current = original.clone();
        current.insert("max-limit".into(), "10M/10M".into());
        current.insert("rate".into(), "1M/2M".into());
        let body = patch_body(&QUEUE_SIMPLE_FORM, &original, &current, "********");
        assert_eq!(body.get("max-limit").map(String::as_str), Some("10M/10M"));
        assert!(!body.contains_key("rate"));
    }
}
