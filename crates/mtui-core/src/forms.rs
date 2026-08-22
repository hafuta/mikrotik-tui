//! Sectioned property-sheet schemas for resource editors.

use std::collections::BTreeMap;
use std::collections::HashMap;

/// One field in a properties sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    Toggle,
    Enum { values: &'static [&'static str] },
    Readonly,
    Secret,
}

impl FieldKind {
    #[must_use]
    pub fn writable(self) -> bool {
        !matches!(self, Self::Readonly)
    }

    /// Short kind tag shown beside the field label (`text`, `select`, …).
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "num",
            Self::Toggle => "toggle",
            Self::Enum { .. } => "select",
            Self::Readonly => "read",
            Self::Secret => "secret",
        }
    }

    /// Footer hint for the focused control.
    #[must_use]
    pub fn edit_hint(self) -> &'static str {
        match self {
            Self::Text | Self::Number | Self::Secret => "type value",
            Self::Toggle => "space toggle",
            Self::Enum { .. } => "space cycle",
            Self::Readonly => "read only",
        }
    }

    /// Whether printable keys, including digits, should go into this field.
    #[must_use]
    pub fn takes_typed_input(self) -> bool {
        matches!(self, Self::Text | Self::Number | Self::Secret)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormSection {
    pub id: &'static str,
    pub label: &'static str,
    pub fields: &'static [FieldSpec],
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormSchema {
    pub title_key: &'static str,
    pub subtitle_keys: &'static [&'static str],
    pub sections: &'static [FormSection],
    pub create_sections: &'static [FormSection],
}

impl FormSchema {
    #[must_use]
    pub fn sections_for(&self, create: bool) -> &'static [FormSection] {
        if create && !self.create_sections.is_empty() {
            self.create_sections
        } else {
            self.sections
        }
    }

    #[must_use]
    pub fn field(&self, key: &str) -> Option<&'static FieldSpec> {
        self.sections
            .iter()
            .chain(self.create_sections.iter())
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
    }

    #[must_use]
    pub fn writable_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for section in self.sections.iter().chain(self.create_sections.iter()) {
            if section.read_only {
                continue;
            }
            for field in section.fields {
                if field.kind.writable() && !keys.contains(&field.key) {
                    keys.push(field.key);
                }
            }
        }
        keys
    }

    #[must_use]
    pub fn known_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        for section in self.sections.iter().chain(self.create_sections.iter()) {
            for field in section.fields {
                if !keys.contains(&field.key) {
                    keys.push(field.key);
                }
            }
        }
        keys
    }
}

/// Keys present on the live row but not in the schema (shown as Status extras).
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn extra_status_fields(
    schema: &FormSchema,
    row: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let known = schema.known_keys();
    let mut extras: Vec<(String, String)> = row
        .iter()
        .filter(|(key, _)| key.as_str() != ".id" && !known.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    extras.sort_by(|left, right| left.0.cmp(&right.0));
    extras
}

/// PATCH/PUT body: only writable keys that actually changed.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn patch_body(
    schema: &FormSchema,
    original: &HashMap<String, String>,
    current: &HashMap<String, String>,
    masked_token: &str,
) -> BTreeMap<String, String> {
    let original_map: BTreeMap<_, _> = original
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let current_map: BTreeMap<_, _> = current
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    crate_changed(
        &original_map,
        &current_map,
        &schema.writable_keys(),
        masked_token,
    )
}

fn crate_changed(
    original: &BTreeMap<String, String>,
    current: &BTreeMap<String, String>,
    writable: &[&str],
    masked_token: &str,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for key in writable {
        let Some(now) = current.get(*key) else {
            continue;
        };
        if now == masked_token {
            continue;
        }
        match original.get(*key) {
            Some(was) if was == now => {}
            _ => {
                out.insert((*key).to_string(), now.clone());
            }
        }
    }
    out
}

pub const ARP_VALUES: &[&str] = &[
    "enabled",
    "disabled",
    "proxy-arp",
    "reply-only",
    "local-proxy-arp",
];

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: FormSchema = FormSchema {
        title_key: "name",
        subtitle_keys: &["type"],
        sections: &[
            FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "name",
                        label: "Name",
                        kind: FieldKind::Text,
                    },
                    FieldSpec {
                        key: "comment",
                        label: "Comment",
                        kind: FieldKind::Text,
                    },
                ],
            },
            FormSection {
                id: "status",
                label: "Status",
                read_only: true,
                fields: &[FieldSpec {
                    key: "running",
                    label: "Running",
                    kind: FieldKind::Readonly,
                }],
            },
        ],
        create_sections: &[],
    };

    #[test]
    fn patch_body_skips_readonly_and_unchanged() {
        let mut original = HashMap::new();
        original.insert("name".into(), "vlan10".into());
        original.insert("running".into(), "true".into());
        let mut current = original.clone();
        current.insert("comment".into(), "office".into());
        current.insert("running".into(), "false".into());
        let body = patch_body(&SAMPLE, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("office"));
        assert!(!body.contains_key("running"));
        assert!(!body.contains_key("name"));
    }

    #[test]
    fn field_kind_names_the_control() {
        assert_eq!(FieldKind::Text.tag(), "text");
        assert_eq!(FieldKind::Number.tag(), "num");
        assert_eq!(FieldKind::Toggle.tag(), "toggle");
        assert_eq!(
            FieldKind::Enum {
                values: &["a", "b"]
            }
            .tag(),
            "select"
        );
        assert_eq!(FieldKind::Readonly.tag(), "read");
        assert_eq!(FieldKind::Secret.tag(), "secret");
        assert_eq!(FieldKind::Text.edit_hint(), "type value");
        assert_eq!(FieldKind::Toggle.edit_hint(), "space toggle");
        assert_eq!(
            FieldKind::Enum { values: &["a"] }.edit_hint(),
            "space cycle"
        );
        assert!(FieldKind::Number.takes_typed_input());
        assert!(!FieldKind::Toggle.takes_typed_input());
    }
}
