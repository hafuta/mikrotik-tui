//! Typed `RouterOS` API sentences (`!re` / `!done` / `!trap` / `!fatal`).

use std::collections::HashMap;

use crate::error::{Error, ErrorKind};
use crate::resource::Resource;
use crate::secret::{is_secret_key, mask_value};

/// One decoded API sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    pub words: Vec<String>,
}

impl Sentence {
    #[must_use]
    pub fn new(words: Vec<String>) -> Self {
        Self { words }
    }

    #[must_use]
    pub fn reply_kind(&self) -> Option<&str> {
        self.words
            .first()
            .filter(|word| word.starts_with('!'))
            .map(String::as_str)
    }

    #[must_use]
    pub fn tag(&self) -> Option<&str> {
        api_attr(&self.words, "tag")
    }

    #[must_use]
    pub fn attr(&self, name: &str) -> Option<&str> {
        for word in &self.words {
            let Some(rest) = word.strip_prefix('=') else {
                continue;
            };
            let Some((key, value)) = rest.split_once('=') else {
                continue;
            };
            if key == name {
                return Some(value);
            }
        }
        None
    }

    #[must_use]
    pub fn attributes(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        for word in &self.words {
            let Some(rest) = word.strip_prefix('=') else {
                continue;
            };
            let Some((key, value)) = rest.split_once('=') else {
                continue;
            };
            fields.insert(key.to_string(), value.to_string());
        }
        fields
    }

    #[must_use]
    pub fn into_resource(self) -> Resource {
        Resource::from_attributes(self.attributes())
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        self.reply_kind() == Some("!done")
    }

    #[must_use]
    pub fn is_re(&self) -> bool {
        self.reply_kind() == Some("!re")
    }

    #[must_use]
    pub fn is_trap(&self) -> bool {
        self.reply_kind() == Some("!trap")
    }

    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.reply_kind() == Some("!fatal")
    }

    pub fn trap_error(&self, operation: &'static str) -> Error {
        let message = self
            .attr("message")
            .or_else(|| self.attr("detail"))
            .unwrap_or("request failed");
        let category = self.attr("category").map(ToOwned::to_owned);
        let kind = kind_for_trap(message, category.as_deref());
        Error::trap(kind, operation, category, message)
    }

    /// Redacted one-line form for logs. Secret attribute values are masked.
    #[must_use]
    pub fn log_line(&self) -> String {
        let mut parts = Vec::with_capacity(self.words.len());
        for word in &self.words {
            parts.push(redact_word(word));
        }
        parts.join(" ")
    }
}

fn api_attr<'a>(words: &'a [String], name: &str) -> Option<&'a str> {
    let prefix = format!(".{name}=");
    words.iter().find_map(|word| word.strip_prefix(&prefix))
}

fn kind_for_trap(message: &str, category: Option<&str>) -> ErrorKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains("invalid user")
        || lower.contains("cannot log in")
        || lower.contains("login failure")
        || lower.contains("authentication")
        || lower.contains("not logged in")
    {
        return ErrorKind::Auth;
    }
    if lower.contains("no such item")
        || lower.contains("not found")
        || lower.contains("no such command prefix")
        || category == Some("1")
    {
        return ErrorKind::NotFound;
    }
    if mtui_core::is_permission_trap(message) {
        return ErrorKind::Permission;
    }
    ErrorKind::Api
}

fn redact_word(word: &str) -> String {
    if let Some(rest) = word.strip_prefix('=')
        && let Some((key, value)) = rest.split_once('=')
    {
        return format!("={key}={}", mask_value(key, value));
    }
    if let Some(rest) = word.strip_prefix('.')
        && let Some((key, value)) = rest.split_once('=')
        && is_secret_key(key)
    {
        return format!(".{key}={}", mask_value(key, value));
    }
    word.to_string()
}

/// Merge a `.listen` `!re` into `rows`. `.dead=true` removes the record.
pub fn merge_listen_record(rows: &mut Vec<Resource>, update: Resource) {
    let dead = update
        .field(".dead")
        .or_else(|| update.field("dead"))
        .is_some_and(|value| matches!(value, "true" | "yes" | "1"));
    if dead {
        if !update.id.is_empty() {
            rows.retain(|row| row.id != update.id);
        }
        return;
    }
    if !update.id.is_empty()
        && let Some(existing) = rows.iter_mut().find(|row| row.id == update.id)
    {
        for (key, value) in update.fields {
            existing.fields.insert(key, value);
        }
        return;
    }
    rows.push(update);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_re_attributes_and_tag() {
        let sentence = Sentence::new(vec![
            "!re".into(),
            "=.id=*1".into(),
            "=name=ether1".into(),
            ".tag=4".into(),
        ]);
        assert!(sentence.is_re());
        assert_eq!(sentence.tag(), Some("4"));
        assert_eq!(sentence.attr("name"), Some("ether1"));
        let resource = sentence.into_resource();
        assert_eq!(resource.id, "*1");
        assert_eq!(resource.field("name"), Some("ether1"));
    }

    #[test]
    fn login_trap_is_auth() {
        let sentence = Sentence::new(vec![
            "!trap".into(),
            "=message=cannot log in".into(),
            "=category=0".into(),
        ]);
        let err = sentence.trap_error("login");
        assert_eq!(err.kind(), ErrorKind::Auth);
        assert_eq!(err.message(), "cannot log in");
    }

    #[test]
    fn missing_command_prefix_is_not_found() {
        let sentence = Sentence::new(vec![
            "!trap".into(),
            "=message=no such command prefix".into(),
        ]);
        let err = sentence.trap_error("system");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.message(), "no such command prefix");
    }

    #[test]
    fn permission_trap_is_permission() {
        let sentence = Sentence::new(vec![
            "!trap".into(),
            "=message=failure: not enough permissions (write)".into(),
        ]);
        let err = sentence.trap_error("patch");
        assert_eq!(err.kind(), ErrorKind::Permission);
        assert!(err.message().contains("not enough permissions"));
    }

    #[test]
    fn redacts_password_words_in_log_line() {
        let sentence = Sentence::new(vec![
            "/login".into(),
            "=name=admin".into(),
            "=password=hunter2".into(),
        ]);
        let line = sentence.log_line();
        assert!(!line.contains("hunter2"));
        assert!(line.contains("password"));
    }

    #[test]
    fn log_line_keeps_non_secret_re_attributes() {
        let sentence = Sentence::new(vec![
            "!re".into(),
            "=.id=*1".into(),
            "=name=ether1".into(),
            "=poe-out=auto-on".into(),
            ".tag=2".into(),
        ]);
        let line = sentence.log_line();
        assert!(line.contains("!re"));
        assert!(line.contains("=name=ether1"));
        assert!(line.contains("=poe-out=auto-on"));
        assert!(line.contains(".tag=2"));
    }

    #[test]
    fn ipv6_firewall_connection_re_preserves_optional_fields() {
        struct Case {
            words: &'static [&'static str],
            id: &'static str,
            src_port: Option<&'static str>,
            tcp_state: Option<&'static str>,
        }
        let cases = [
            Case {
                words: &[
                    "!re",
                    "=.id=*36",
                    "=src-address=2001:db8:1::10",
                    "=dst-address=2001:db8:2::1",
                    "=protocol=tcp",
                    "=src-port=53100",
                    "=dst-port=443",
                    "=tcp-state=established",
                    "=timeout=23h59m",
                    "=orig-rate=1200",
                    "=repl-rate=8500",
                    "=connection-mark=",
                    "=reply-dst-address=2001:db8:1::10",
                ],
                id: "*36",
                src_port: Some("53100"),
                tcp_state: Some("established"),
            },
            Case {
                words: &[
                    "!re",
                    "=.id=*37",
                    "=src-address=2001:db8:1::20",
                    "=dst-address=2001:db8::53",
                    "=protocol=udp",
                    "=timeout=10s",
                ],
                id: "*37",
                src_port: None,
                tcp_state: None,
            },
        ];
        for case in cases {
            let sentence =
                Sentence::new(case.words.iter().map(|word| (*word).to_string()).collect());
            assert!(sentence.is_re(), "{:?}", case.words);
            let resource = sentence.into_resource();
            assert_eq!(resource.id, case.id);
            assert!(resource.field("src-address").unwrap().contains("2001:db8"));
            assert_eq!(resource.field("src-port"), case.src_port);
            assert_eq!(resource.field("tcp-state"), case.tcp_state);
        }
    }

    #[test]
    fn ipv6_firewall_connection_remove_trap_is_not_found() {
        let sentence = Sentence::new(vec![
            "!trap".into(),
            "=message=no such item".into(),
            "=category=1".into(),
        ]);
        let err = sentence.trap_error("delete");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.message(), "no such item");
        assert_eq!(err.operation(), "delete");
    }

    #[test]
    fn listen_merge_updates_and_removes() {
        let mut rows = vec![Resource {
            id: "*1".into(),
            fields: HashMap::from([("name".into(), "ether1".into())]),
        }];
        merge_listen_record(
            &mut rows,
            Resource {
                id: "*1".into(),
                fields: HashMap::from([("comment".into(), "wan".into())]),
            },
        );
        assert_eq!(rows[0].field("name"), Some("ether1"));
        assert_eq!(rows[0].field("comment"), Some("wan"));
        merge_listen_record(
            &mut rows,
            Resource {
                id: "*1".into(),
                fields: HashMap::from([(".dead".into(), "true".into())]),
            },
        );
        assert!(rows.is_empty());
    }
}
