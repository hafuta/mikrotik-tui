//! `RouterOS` Safe Mode snapshot parsed from `/safe-mode/print`.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::actions::truthy;

/// History rows `RouterOS` can keep while Safe Mode is on. Overflow drops Safe
/// Mode and those changes are not auto-undone.
pub const SAFE_MODE_HISTORY_LIMIT: usize = 100;

/// Warn before the hard limit so the operator can checkpoint (release + take).
pub const SAFE_MODE_HISTORY_WARN: usize = 80;

/// Live `/safe-mode` fields for this API login.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SafeModeStatus {
    pub enabled: bool,
    pub current: bool,
    pub owner: String,
    pub user: String,
}

impl SafeModeStatus {
    #[must_use]
    pub fn from_fields(fields: &HashMap<String, String>) -> Self {
        Self {
            enabled: truthy(fields.get("enabled").map(String::as_str)),
            current: truthy(fields.get("current").map(String::as_str)),
            owner: fields
                .get("owner")
                .map_or("", String::as_str)
                .trim()
                .to_string(),
            user: fields
                .get("user")
                .map_or("", String::as_str)
                .trim()
                .to_string(),
        }
    }

    /// This login owns Safe Mode.
    #[must_use]
    pub fn we_hold(&self) -> bool {
        self.enabled && self.current
    }

    /// Someone else owns Safe Mode.
    #[must_use]
    pub fn foreign(&self) -> bool {
        self.enabled && !self.current
    }

    #[must_use]
    pub fn holder_label(&self) -> String {
        match (self.owner.as_str(), self.user.as_str()) {
            ("", "") => "another session".into(),
            (owner, "") => owner.to_string(),
            ("", user) => user.to_string(),
            (owner, user) => format!("{owner} ({user})"),
        }
    }
}

/// Count history rows tagged as Safe Mode floating-undo.
#[must_use]
pub fn floating_undo_count<S: BuildHasher>(rows: &[HashMap<String, String, S>]) -> usize {
    rows.iter()
        .filter(|row| {
            truthy(row.get("floating-undo").map(String::as_str))
                || row
                    .get("flags")
                    .is_some_and(|flags| flags.to_ascii_uppercase().contains('F'))
        })
        .count()
}

#[must_use]
pub fn safe_mode_overflow_warning(count: usize) -> Option<String> {
    if count < SAFE_MODE_HISTORY_WARN {
        return None;
    }
    Some(format!(
        "Safe Mode history is {count}/{SAFE_MODE_HISTORY_LIMIT}. Release and take again before the limit or RouterOS drops Safe Mode with no auto-undo."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_hold_requires_enabled_and_current() {
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("current".into(), "true".into());
        fields.insert("owner".into(), "api".into());
        fields.insert("user".into(), "admin".into());
        let status = SafeModeStatus::from_fields(&fields);
        assert!(status.we_hold());
        assert!(!status.foreign());
        assert_eq!(status.holder_label(), "api (admin)");
    }

    #[test]
    fn foreign_owner_is_not_current() {
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "yes".into());
        fields.insert("current".into(), "false".into());
        fields.insert("owner".into(), "winbox".into());
        let status = SafeModeStatus::from_fields(&fields);
        assert!(status.foreign());
        assert!(!status.we_hold());
    }

    #[test]
    fn floating_count_reads_flag_or_letter() {
        let mut a = HashMap::new();
        a.insert("floating-undo".into(), "true".into());
        let mut b = HashMap::new();
        b.insert("flags".into(), "F".into());
        let mut c = HashMap::new();
        c.insert("action".into(), "set".into());
        assert_eq!(floating_undo_count(&[a, b, c]), 2);
        assert!(safe_mode_overflow_warning(79).is_none());
        assert!(safe_mode_overflow_warning(80).is_some());
        assert!(safe_mode_overflow_warning(SAFE_MODE_HISTORY_LIMIT).is_some());
    }

    #[test]
    fn holder_label_falls_back_when_owner_or_user_is_blank() {
        assert_eq!(SafeModeStatus::default().holder_label(), "another session");
        assert_eq!(
            SafeModeStatus {
                owner: "winbox".into(),
                ..SafeModeStatus::default()
            }
            .holder_label(),
            "winbox"
        );
        assert_eq!(
            SafeModeStatus {
                user: "admin".into(),
                ..SafeModeStatus::default()
            }
            .holder_label(),
            "admin"
        );
    }
}
