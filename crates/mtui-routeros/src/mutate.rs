//! Field encoding helpers for `RouterOS` API mutations.

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
/// secrets are omitted so set/add bodies only contain intentional edits.
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

    #[test]
    fn encode_omits_absent_optional_traffic_flow_keys() {
        let mut fields = BTreeMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("dst-address".into(), "192.0.2.10".into());
        let json = encode_fields(&fields);
        let object = json.as_object().expect("object");
        assert_eq!(object.get("enabled"), Some(&Value::String("true".into())));
        assert_eq!(
            object.get("dst-address"),
            Some(&Value::String("192.0.2.10".into()))
        );
        assert!(!object.contains_key("sampling-interval"));
        assert!(!object.contains_key("v9-template-timeout"));
        assert!(!object.values().any(|value| !value.is_string()));
    }

    #[test]
    fn lte_apn_encode_omits_absent_optional_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("name".into(), "carrier".into());
        fields.insert("apn".into(), "internet".into());
        fields.insert("use-network-apn".into(), "false".into());
        let json = encode_fields(&fields);
        assert_eq!(
            json.to_string(),
            r#"{"apn":"internet","name":"carrier","use-network-apn":"false"}"#
        );
        assert!(json.get("password").is_none());
        assert!(json.get("user").is_none());

        let original = BTreeMap::from([
            ("name".into(), "carrier".into()),
            ("apn".into(), "internet".into()),
            ("password".into(), "secret".into()),
        ]);
        let current = BTreeMap::from([
            ("name".into(), "carrier".into()),
            ("apn".into(), "lte.provider".into()),
            ("password".into(), "********".into()),
        ]);
        let changed = changed_fields(
            &original,
            &current,
            &["name", "apn", "password", "user"],
            "********",
        );
        assert_eq!(
            changed,
            BTreeMap::from([("apn".into(), "lte.provider".into())])
        );
    }
}
