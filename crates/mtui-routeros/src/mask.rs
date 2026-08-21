//! Mask sensitive field values before they reach the UI or logs.

use std::collections::HashMap;

const MASK: &str = "••••••••";

/// Return a copy of `fields` with secret-like keys masked.
#[must_use]
pub fn mask_secrets(fields: &HashMap<String, String>) -> HashMap<String, String> {
    fields
        .iter()
        .map(|(k, v)| {
            if is_secret_key(k) {
                (k.clone(), MASK.to_string())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

#[must_use]
pub fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "password" | "secret" | "passphrase" | "private-key" | "psk"
    ) || lower.contains("password")
        || lower.ends_with("-secret")
        || lower.contains("passphrase")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_password_fields() {
        let mut fields = HashMap::new();
        fields.insert("name".into(), "admin".into());
        fields.insert("password".into(), "hunter2".into());
        fields.insert("wpa2-pre-shared-key".into(), "x".into()); // contains no password pattern as exact
        fields.insert("user-password".into(), "x".into());
        let masked = mask_secrets(&fields);
        assert_eq!(masked.get("name").unwrap(), "admin");
        assert_eq!(masked.get("password").unwrap(), MASK);
        assert_eq!(masked.get("user-password").unwrap(), MASK);
    }
}
