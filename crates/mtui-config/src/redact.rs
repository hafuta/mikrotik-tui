//! Best-effort redaction of password-like values from log strings.
//!
//! This is defense-in-depth, not a substitute for never logging secrets:
//! callers should still avoid formatting [`crate::credentials::Credential`]
//! or raw env overrides into log messages. [`redact`] scans for
//! `key=value`, `key: value`, and `"key":"value"` pairs whose key matches a
//! known credential-ish name and replaces the value with `[REDACTED]`.

/// Key names (case-insensitive, matched as substrings) treated as
/// credential-bearing.
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "credential",
    "api_key",
    "apikey",
    "authorization",
];

/// Redacts password-like values in `input`, preserving everything else
/// (keys, structure, surrounding text) unchanged.
///
/// ```
/// use mtui_config::redact;
///
/// assert_eq!(
///     redact(r#"connecting url=https://r1 password="hunter2" ok"#),
///     r#"connecting url=https://r1 password="[REDACTED]" ok"#
/// );
/// assert_eq!(
///     redact(r#"{"username":"admin","password":"hunter2"}"#),
///     r#"{"username":"admin","password":"[REDACTED]"}"#
/// );
/// ```
#[must_use]
pub fn redact(input: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;

    loop {
        let Some((_, key_end)) = next_sensitive_key(&lower, i) else {
            out.push_str(&input[i..]);
            break;
        };

        out.push_str(&input[i..key_end]);

        let mut j = key_end;
        if bytes.get(j) == Some(&b'"') {
            j += 1;
        }
        j = skip_ascii_whitespace(bytes, j);

        let Some(&sep) = bytes.get(j) else {
            i = key_end;
            continue;
        };
        if sep != b'=' && sep != b':' {
            i = key_end;
            continue;
        }
        j += 1;
        j = skip_ascii_whitespace(bytes, j);

        let quoted = bytes.get(j) == Some(&b'"');
        if quoted {
            j += 1;
        }
        let value_start = j;
        let value_end = find_value_end(bytes, value_start);

        out.push_str(&input[key_end..value_start]);
        out.push_str("[REDACTED]");

        if quoted && bytes.get(value_end) == Some(&b'"') {
            out.push('"');
            i = value_end + 1;
        } else {
            i = value_end;
        }
    }

    out
}

fn next_sensitive_key(lower: &str, from: usize) -> Option<(usize, usize)> {
    SENSITIVE_KEYS
        .iter()
        .filter_map(|key| {
            lower[from..]
                .find(key)
                .map(|offset| (from + offset, from + offset + key.len()))
        })
        .min_by_key(|(start, _)| *start)
}

fn skip_ascii_whitespace(bytes: &[u8], mut j: usize) -> usize {
    while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
        j += 1;
    }
    j
}

fn find_value_end(bytes: &[u8], start: usize) -> usize {
    let mut j = start;
    while let Some(&b) = bytes.get(j) {
        if b == b'"' || b == b',' || b == b'}' || b == b']' || b.is_ascii_whitespace() {
            break;
        }
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn redacts_equals_form() {
        assert_eq!(
            redact("password=hunter2 next=ok"),
            "password=[REDACTED] next=ok"
        );
    }

    #[test]
    fn redacts_colon_form_with_quotes() {
        assert_eq!(
            redact(r#"{"password": "hunter2", "url": "x"}"#),
            r#"{"password": "[REDACTED]", "url": "x"}"#
        );
    }

    #[test]
    fn leaves_unrelated_text_alone() {
        assert_eq!(
            redact("connecting to router at 10.0.0.1"),
            "connecting to router at 10.0.0.1"
        );
    }

    #[test]
    fn redacts_multiple_occurrences() {
        assert_eq!(
            redact("password=one token=two"),
            "password=[REDACTED] token=[REDACTED]"
        );
    }
}
