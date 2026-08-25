//! `RouterOS` API record representation.

use std::collections::HashMap;

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};
use serde_json::Value;

use crate::secret::mask_value;

/// A single `RouterOS` record.
///
/// `RouterOS` represents every record value as a string; [`Resource`]
/// preserves those raw string values unmodified rather than parsing them
/// into booleans/integers, so callers can decide how to interpret each
/// field. The record's `.id` value (when present) is split out into [`id`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub fields: HashMap<String, String>,
}

impl Resource {
    /// Builds a record from API `=name=value` attributes. `.id` is split out.
    #[must_use]
    pub fn from_attributes(mut fields: HashMap<String, String>) -> Self {
        let id = fields.remove(".id").unwrap_or_default();
        Self { id, fields }
    }

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

    /// Display row for tables: masked fields plus the opaque `.id` when present.
    #[must_use]
    pub fn display_row(&self) -> HashMap<String, String> {
        let mut fields = self.masked_fields();
        if !self.id.is_empty() {
            fields.insert(".id".to_string(), self.id.clone());
        }
        fields
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
    fn history_rows_decode_optional_fields_and_reject_malformed() {
        let cases: [(&str, bool, &str, Option<&str>); 4] = [
            (
                r#"{".id":"*h1","time":"aug/25/2026 01:00:00","action":"set","by":"admin","policy":"write","floating-undo":"false"}"#,
                true,
                "*h1",
                Some("set"),
            ),
            (
                r#"{".id":"*h2","action":"remove","by":"ops"}"#,
                true,
                "*h2",
                Some("remove"),
            ),
            (
                r#"{".id":"*h3","action":"set","note":"extra-unknown"}"#,
                true,
                "*h3",
                Some("set"),
            ),
            (r#"{".id":"*h4","action":1}"#, false, "", None),
        ];
        for (json, ok, id, action) in cases {
            let parsed: Result<Resource, _> = serde_json::from_str(json);
            assert_eq!(parsed.is_ok(), ok, "{json}");
            if ok {
                let row = parsed.expect("row");
                assert_eq!(row.id, id);
                assert_eq!(row.field("action"), action);
                let display = row.display_row();
                if !id.is_empty() {
                    assert_eq!(display.get(".id").map(String::as_str), Some(id));
                }
            }
        }
    }

    #[test]
    fn missing_id_defaults_to_empty_string() {
        let resource: Resource =
            serde_json::from_str(r#"{"enabled":"true"}"#).expect("valid resource JSON");
        assert_eq!(resource.id, "");
        assert_eq!(resource.field("enabled"), Some("true"));
    }

    #[test]
    fn ipv6_firewall_connection_json_optional_and_malformed() {
        let full: Resource = serde_json::from_str(
            r#"{".id":"*36","src-address":"2001:db8:1::10","dst-address":"2001:db8:2::1","protocol":"tcp","src-port":"53100","timeout":"23h59m"}"#,
        )
        .expect("full connection");
        assert_eq!(full.id, "*36");
        assert_eq!(full.field("src-port"), Some("53100"));
        assert_eq!(full.field("tcp-state"), None);

        let missing_id: Resource =
            serde_json::from_str(r#"{"protocol":"udp","timeout":"10s"}"#).expect("optional id");
        assert_eq!(missing_id.id, "");
        assert_eq!(missing_id.field("timeout"), Some("10s"));

        let malformed: Result<Resource, _> =
            serde_json::from_str(r#"{"src-address":"2001:db8::1","timeout":10}"#);
        assert!(malformed.is_err());
    }

    #[test]
    fn romon_and_graphing_json_optional_malformed_and_secrets() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            (
                r#"{".id":"","enabled":"true","id":"00:00:00:00:00:00","secrets":"alpha,beta","current-id":"DC:2C:6E:9E:11:27"}"#,
                "current-id",
                Some("DC:2C:6E:9E:11:27"),
            ),
            (r#"{"enabled":"false"}"#, "enabled", Some("false")),
            (
                r#"{".id":"*1","interface":"all","forbid":"false","cost":"100"}"#,
                "cost",
                Some("100"),
            ),
            (
                r#"{"store-every":"5min","page-refresh":"never"}"#,
                "page-refresh",
                Some("never"),
            ),
            (
                r#"{".id":"*gi1","interface":"ether1","allow-address":"192.0.2.0/24","store-on-disk":"true"}"#,
                "comment",
                None,
            ),
        ];
        for (json, key, expected) in cases {
            let resource: Resource = serde_json::from_str(json).expect("valid tool JSON");
            assert_eq!(resource.field(key), *expected, "json={json}");
        }

        let masked: Resource =
            serde_json::from_str(r#"{"enabled":"yes","secrets":"shared-secret"}"#)
                .expect("romon secrets");
        let row = masked.display_row();
        assert_eq!(
            row.get("secrets").map(String::as_str),
            Some(crate::secret::MASKED_VALUE)
        );
        assert_eq!(row.get("enabled").map(String::as_str), Some("yes"));

        let bad: Result<Resource, _> = serde_json::from_str(r#"{"enabled":true,"secrets":1}"#);
        assert!(bad.is_err());
    }

    #[test]
    fn display_row_includes_id_and_masks_secrets() {
        let resource: Resource =
            serde_json::from_str(r#"{".id":"*1","name":"wlan1","password":"hunter2"}"#)
                .expect("valid resource JSON");
        let row = resource.display_row();
        assert_eq!(row.get(".id").map(String::as_str), Some("*1"));
        assert_eq!(row.get("name").map(String::as_str), Some("wlan1"));
        assert_eq!(
            row.get("password").map(String::as_str),
            Some(crate::secret::MASKED_VALUE)
        );
    }

    #[test]
    fn traffic_flow_and_igmp_decode_optional_and_reject_malformed() {
        let cases = [
            (
                r#"{"enabled":"yes","interfaces":"all","cache-entries":"4k"}"#,
                "",
                "interfaces",
                Some("all"),
            ),
            (
                r#"{".id":"*tf1","dst-address":"192.0.2.10","port":"2055","version":"ipfix"}"#,
                "*tf1",
                "version",
                Some("ipfix"),
            ),
            (
                r#"{"query-interval":"2m5s","quick-leave":"false"}"#,
                "",
                "query-interval",
                Some("2m5s"),
            ),
            (
                r#"{".id":"*ig1","interface":"ether1","upstream":"true"}"#,
                "*ig1",
                "upstream",
                Some("true"),
            ),
        ];
        for (json, id, key, expected) in cases {
            let resource: Resource = serde_json::from_str(json).expect(json);
            assert_eq!(resource.id, id, "{json}");
            assert_eq!(resource.field(key), expected, "{json}");
            assert!(resource.field("missing-optional").is_none(), "{json}");
        }

        let malformed: Result<Resource, _> =
            serde_json::from_str(r#"{"dst-address":192,"port":"2055"}"#);
        assert!(malformed.is_err());
        let not_object: Result<Resource, _> = serde_json::from_str("[1,2]");
        assert!(not_object.is_err());
    }

    #[test]
    fn smb_rest_payloads_decode_optional_fields_and_reject_malformed() {
        let cases = [
            (
                r#"{".id":"*1","name":"backup","directory":"backup","require-encryption":"false"}"#,
                "*1",
                "name",
                Some("backup"),
            ),
            (
                r#"{".id":"*2","name":"pub","directory":"/pub"}"#,
                "*2",
                "valid-users",
                None,
            ),
            (
                r#"{".id":"*3","name":"mtuser","password":"hunter2","read-only":"yes"}"#,
                "*3",
                "password",
                Some("hunter2"),
            ),
        ];
        for (json, id, key, expected) in cases {
            let resource: Resource = serde_json::from_str(json).expect(json);
            assert_eq!(resource.id, id, "{json}");
            assert_eq!(resource.field(key), expected, "{json}");
        }

        let list: Vec<Resource> = serde_json::from_str(
            r#"[{".id":"*1","name":"pub"},{".id":"*2","name":"backup","valid-users":"mtuser"}]"#,
        )
        .expect("share list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[1].field("valid-users"), Some("mtuser"));

        assert!(serde_json::from_str::<Resource>(r#"{"name":"pub","directory":1}"#).is_err());
        assert!(
            serde_json::from_str::<Vec<Resource>>(r#"{"error":"no such command prefix"}"#).is_err()
        );

        let user: Resource =
            serde_json::from_str(r#"{".id":"*3","name":"mtuser","password":"hunter2"}"#)
                .expect("user");
        assert_eq!(
            user.display_row().get("password").map(String::as_str),
            Some(crate::secret::MASKED_VALUE)
        );
        assert_eq!(user.field("password"), Some("hunter2"));
    }

    #[test]
    fn lte_apn_decode_table_covers_optional_malformed_and_secrets() {
        struct Case {
            json: &'static str,
            ok: bool,
            id: &'static str,
            apn: Option<&'static str>,
            password: Option<&'static str>,
        }
        let cases = [
            Case {
                json: r#"{".id":"*1","name":"default","apn":"internet","authentication":"none"}"#,
                ok: true,
                id: "*1",
                apn: Some("internet"),
                password: None,
            },
            Case {
                json: r#"{".id":"*2","name":"carrier","apn":"lte.provider","user":"u","password":"pw"}"#,
                ok: true,
                id: "*2",
                apn: Some("lte.provider"),
                password: Some("pw"),
            },
            Case {
                json: r#"{"name":"partial"}"#,
                ok: true,
                id: "",
                apn: None,
                password: None,
            },
            Case {
                json: r#"{".id":"*3","apn":true}"#,
                ok: false,
                id: "",
                apn: None,
                password: None,
            },
            Case {
                json: "not-json",
                ok: false,
                id: "",
                apn: None,
                password: None,
            },
        ];
        for case in cases {
            let parsed: Result<Resource, _> = serde_json::from_str(case.json);
            if !case.ok {
                assert!(parsed.is_err(), "{}", case.json);
                continue;
            }
            let resource = parsed.expect(case.json);
            assert_eq!(resource.id, case.id, "{}", case.json);
            assert_eq!(resource.field("apn"), case.apn, "{}", case.json);
            assert_eq!(resource.field("password"), case.password, "{}", case.json);
            if case.password.is_some() {
                assert_eq!(
                    resource.masked_fields().get("password").map(String::as_str),
                    Some(crate::secret::MASKED_VALUE)
                );
            }
        }
    }
}
