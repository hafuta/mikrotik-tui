//! `RouterOS` REST record representation.

use std::collections::HashMap;

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};
use serde_json::Value;

use crate::secret::mask_value;

/// A single `RouterOS` REST record.
///
/// `RouterOS` represents every record value as a JSON string; [`Resource`]
/// preserves those raw string values unmodified rather than parsing them
/// into booleans/integers, so callers can decide how to interpret each
/// field. The record's `.id` value (when present) is split out into [`id`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub fields: HashMap<String, String>,
}

impl Resource {
    /// Returns the raw (unmasked) value for `key`, if present.
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    /// Returns a copy of `fields` with secret values replaced by
    /// [`crate::secret::MASKED_VALUE`]. Use this for any display or log
    /// surface.
    #[must_use]
    pub fn masked_fields(&self) -> HashMap<String, String> {
        self.fields
            .iter()
            .map(|(key, value)| (key.clone(), mask_value(key, value)))
            .collect()
    }
}

impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
        let mut fields = HashMap::with_capacity(raw.len());
        let mut id = String::new();
        for (key, value) in raw {
            let Value::String(text) = value else {
                return Err(D::Error::custom(format!("field {key:?} is not a string")));
            };
            if key == ".id" {
                id = text;
            } else {
                fields.insert(key, text);
            }
        }
        Ok(Self { id, fields })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_id_and_preserves_raw_strings() {
        let resource: Resource = serde_json::from_str(
            r#"{".id":"*1","name":"ether1","disabled":"false","running":"true","rx-byte":"00123"}"#,
        )
        .expect("valid resource JSON");

        assert_eq!(resource.id, "*1");
        assert_eq!(resource.field("name"), Some("ether1"));
        assert_eq!(resource.field("rx-byte"), Some("00123"));
        assert_eq!(resource.field(".id"), None);
    }

    #[test]
    fn parses_list_of_resources() {
        let resources: Vec<Resource> =
            serde_json::from_str(r#"[{".id":"*1","name":"ether1"},{".id":"*2","name":"ether2"}]"#)
                .expect("valid resource list JSON");

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, "*1");
        assert_eq!(resources[1].field("name"), Some("ether2"));
    }

    #[test]
    fn rejects_non_string_field_values() {
        let result: Result<Resource, _> = serde_json::from_str(r#"{"name":42}"#);
        assert!(result.is_err());
    }

    #[test]
    fn missing_id_defaults_to_empty_string() {
        let resource: Resource =
            serde_json::from_str(r#"{"enabled":"true"}"#).expect("valid resource JSON");
        assert_eq!(resource.id, "");
        assert_eq!(resource.field("enabled"), Some("true"));
    }
}
