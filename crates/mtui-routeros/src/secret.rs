//! Secret field masking for `RouterOS` record values.
//!
//! `RouterOS` returns credentials (PPP secrets, `WiFi` passphrases, `IPsec`
//! pre-shared keys, ...) as plain-text REST field values. Any UI or log
//! surface that renders [`crate::Resource`] fields must mask these before
//! display.

/// Placeholder rendered in place of a masked secret value.
pub const MASKED_VALUE: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

/// Reports whether `key` names a `RouterOS` field that carries a secret.
///
/// Matches (case-insensitively, treating `_` and `-` as equivalent):
/// `password`, `secret`, `passphrase`, `private-key`, `psk`, `pin`, `cak`,
/// any key containing `password` or a pre-shared-key spelling, and any key
/// ending in `-secret`.
#[must_use]
pub fn is_secret_key(key: &str) -> bool {
    let normalized = key.trim().to_lowercase().replace('_', "-");
    matches!(
        normalized.as_str(),
        "password" | "secret" | "passphrase" | "private-key" | "psk" | "pin" | "cak"
    ) || normalized.contains("password")
        || normalized.contains("pre-shared-key")
        || normalized.contains("preshared-key")
        || normalized.ends_with("-secret")
}

/// Masks `value` to [`MASKED_VALUE`] when `key` is a secret field.
#[must_use]
pub fn mask_value(key: &str, value: &str) -> String {
    if is_secret_key(key) {
        MASKED_VALUE.to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_known_secret_keys() {
        for key in [
            "password",
            "Password",
            "secret",
            "passphrase",
            "private-key",
            "private_key",
            "wpa-password",
            "vpn-secret",
            "PSK-SECRET",
            "psk",
            "pin",
            "cak",
            "preshared-key",
            "pre-shared-key",
            "wpa2-pre-shared-key",
            "ipsec-secret",
        ] {
            assert!(is_secret_key(key), "expected {key:?} to be a secret key");
        }
    }

    #[test]
    fn does_not_flag_ordinary_fields() {
        for key in [
            "name",
            "address",
            "mac-address",
            "comment",
            "running",
            "ckn",
            "my-id",
            "remote-id",
        ] {
            assert!(!is_secret_key(key), "expected {key:?} to be ordinary");
        }
    }

    #[test]
    fn masks_secret_values_only() {
        assert_eq!(mask_value("password", "hunter2"), MASKED_VALUE);
        assert_eq!(mask_value("name", "ether1"), "ether1");
    }
}
