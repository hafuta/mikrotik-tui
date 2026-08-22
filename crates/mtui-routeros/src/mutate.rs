//! JSON encoding helpers for `RouterOS` REST mutations.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

/// Encode `RouterOS` string fields as a JSON object. Values stay strings so
/// absent keys stay absent (never coerced to `null` or `false`).
#[must_use]
pub fn encode_fields(fields: &BTreeMap<String, String>) -> Value {
    let mut map = Map::with_capacity(fields.len());
    for (key, value) in fields {
        map.insert(key.clone(), Value::String(value.clone()));
    }
    Value::Object(map)
}

/// True when `command` is a single `RouterOS` command word (`enable`, `reset-counters`).
#[must_use]
pub fn is_command_name(command: &str) -> bool {
    let mut chars = command.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// Diff writable fields. Unchanged keys, read-only keys, and still-masked
/// secrets are omitted so PATCH/PUT bodies only contain intentional edits.
#[must_use]
pub fn changed_fields(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_fields_keeps_string_values_and_key_order() {
        let mut fields = BTreeMap::new();
        fields.insert("disabled".into(), "false".into());
        fields.insert("comment".into(), "uplink".into());
        let json = encode_fields(&fields);
        assert_eq!(
            json.to_string(),
            r#"{"comment":"uplink","disabled":"false"}"#
        );
    }

    #[test]
    fn changed_fields_omits_unchanged_masked_and_readonly() {
        let original = BTreeMap::from([
            ("name".into(), "ether1".into()),
            ("comment".into(), String::new()),
            ("passphrase".into(), "secret".into()),
        ]);
        let current = BTreeMap::from([
            ("name".into(), "ether1".into()),
            ("comment".into(), "wan".into()),
            ("passphrase".into(), "********".into()),
            ("mtu".into(), "1500".into()),
        ]);
        let changed = changed_fields(
            &original,
            &current,
            &["name", "comment", "passphrase", "mtu"],
            "********",
        );
        assert_eq!(
            changed,
            BTreeMap::from([
                ("comment".into(), "wan".into()),
                ("mtu".into(), "1500".into())
            ])
        );
    }

    #[test]
    fn command_names_reject_path_segments() {
        assert!(is_command_name("reset-counters"));
        assert!(is_command_name("enable"));
        assert!(!is_command_name(""));
        assert!(!is_command_name("Enable"));
        assert!(!is_command_name("reset/counters"));
        assert!(!is_command_name("../torch"));
    }
}
