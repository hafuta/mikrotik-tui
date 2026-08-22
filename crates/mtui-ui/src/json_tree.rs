//! Collapsible JSON outline for console expanded rows.
//!
//! `RouterOS` REST errors are small objects (`error` / `message` / `detail`).
//! A dedicated JSON-editor widget would pull in an editor crate and a
//! mismatched ratatui major; this flatten+toggle is enough to inspect them.

use std::collections::HashSet;

use serde_json::Value;

const PATH_SEP: &str = "\u{1f}";

/// One visible row in a collapsed/expanded JSON outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRow {
    pub path: String,
    pub depth: usize,
    pub expandable: bool,
    pub expanded: bool,
    pub label: String,
    pub value: String,
}

/// Parses an object or array JSON value. Scalars stay as plain log fields.
#[must_use]
pub fn parse_container(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return None;
    }
    let value = serde_json::from_str(trimmed).ok()?;
    matches!(value, Value::Object(_) | Value::Array(_)).then_some(value)
}

#[must_use]
pub fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

#[must_use]
pub fn flatten(value: &Value, root_label: &str, open: &HashSet<String>) -> Vec<JsonRow> {
    let mut rows = Vec::new();
    walk(value, &[root_label.to_string()], 0, open, &mut rows);
    rows
}

fn walk(
    value: &Value,
    path: &[String],
    depth: usize,
    open: &HashSet<String>,
    rows: &mut Vec<JsonRow>,
) {
    let path_key = encode(path);
    let expandable = value.is_object() || value.is_array();
    let expanded = expandable && open.contains(&path_key);
    let label = path.last().cloned().unwrap_or_default();
    rows.push(JsonRow {
        path: path_key,
        depth,
        expandable,
        expanded,
        label,
        value: if expanded {
            String::new()
        } else {
            preview(value)
        },
    });
    if !expanded {
        return;
    }
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let mut next = path.to_vec();
                next.push(key.clone());
                walk(child, &next, depth + 1, open, rows);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let mut next = path.to_vec();
                next.push(index.to_string());
                walk(child, &next, depth + 1, open, rows);
            }
        }
        _ => {}
    }
}

fn encode(path: &[String]) -> String {
    path.join(PATH_SEP)
}

fn preview(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return "{}".into();
            }
            if map.len() <= 4 {
                let keys = map.keys().cloned().collect::<Vec<_>>().join(", ");
                format!("{{{keys}}}")
            } else {
                format!("{{{} keys}}", map.len())
            }
        }
        Value::Array(items) => format!("[{}]", items.len()),
        Value::String(text) => format!("\"{text}\""),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn collapsed_root_shows_key_preview() {
        let value = json!({"error":400,"message":"Bad Request","detail":"no such item"});
        let rows = flatten(&value, "body", &HashSet::new());
        assert_eq!(rows.len(), 1);
        assert!(rows[0].expandable);
        assert!(!rows[0].expanded);
        assert!(rows[0].value.contains("error"));
        assert!(rows[0].value.contains("detail"));
    }

    #[test]
    fn expanding_root_lists_children() {
        let value = json!({"error":400,"message":"Bad Request"});
        let mut open = HashSet::new();
        open.insert(encode(&["body".into()]));
        let rows = flatten(&value, "body", &open);
        assert_eq!(rows.len(), 3);
        assert!(rows[0].expanded);
        assert_eq!(rows[1].label, "error");
        assert_eq!(rows[1].value, "400");
        assert_eq!(rows[2].label, "message");
        assert_eq!(rows[2].value, "\"Bad Request\"");
    }

    #[test]
    fn rejects_non_container_json() {
        assert!(parse_container("400").is_none());
        assert!(parse_container("\"x\"").is_none());
        assert!(parse_container("not json").is_none());
        assert!(parse_container(r#"{"error":400}"#).is_some());
    }
}
