//! `RouterOS` group policies and action permission mapping.

use std::collections::{HashMap, HashSet};

use crate::actions::{ActionCommand, ActionKind, ActionSpec};

/// `RouterOS` user-group policy that grants list/print.
pub const POLICY_READ: &str = "read";
/// `RouterOS` user-group policy that grants set/add/remove.
pub const POLICY_WRITE: &str = "write";
/// `RouterOS` user-group policy that grants user and group changes.
pub const POLICY_POLICY: &str = "policy";
/// `RouterOS` user-group policy that grants ping, traceroute, and scans.
pub const POLICY_TEST: &str = "test";
/// `RouterOS` user-group policy that grants reboot and shutdown.
pub const POLICY_REBOOT: &str = "reboot";
/// `RouterOS` user-group policy that grants packet capture tools.
pub const POLICY_SNIFF: &str = "sniff";

const POLICY_MENUS: &[(&str, &str)] = &[
    ("users", POLICY_POLICY),
    ("user-groups", POLICY_POLICY),
    ("ssh-keys", POLICY_POLICY),
];

/// Known policies for the logged-in `RouterOS` user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAccess {
    pub username: String,
    pub group: String,
    policies: HashSet<String>,
    known: bool,
}

impl Default for SessionAccess {
    fn default() -> Self {
        Self::unknown()
    }
}

impl SessionAccess {
    /// Permissions were not loaded. Write actions stay available and traps
    /// still map to a clear message after the router refuses them.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            username: String::new(),
            group: String::new(),
            policies: HashSet::new(),
            known: false,
        }
    }

    /// Full operator access (demo profile and unrestricted groups).
    #[must_use]
    pub fn full(username: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            group: group.into(),
            policies: [
                POLICY_READ,
                POLICY_WRITE,
                POLICY_POLICY,
                POLICY_TEST,
                POLICY_REBOOT,
                POLICY_SNIFF,
            ]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
            known: true,
        }
    }

    #[must_use]
    pub fn from_policies(
        username: impl Into<String>,
        group: impl Into<String>,
        policies: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        Self {
            username: username.into(),
            group: group.into(),
            policies: policies
                .into_iter()
                .map(|policy| policy.as_ref().trim().to_ascii_lowercase())
                .filter(|policy| !policy.is_empty())
                .collect(),
            known: true,
        }
    }

    /// Resolve the signed-in user against `/user` and `/user/group` rows.
    #[must_use]
    pub fn from_router_rows(
        username: &str,
        users: &[HashMap<String, String>],
        groups: &[HashMap<String, String>],
    ) -> Self {
        let username = username.trim();
        if username.is_empty() {
            return Self::unknown();
        }
        let Some(user) = users.iter().find(|row| {
            row.get("name")
                .is_some_and(|name| name.trim().eq_ignore_ascii_case(username))
        }) else {
            return Self::unknown();
        };
        let group = user.get("group").map_or("", String::as_str).trim();
        if group.is_empty() {
            return Self::unknown();
        }
        let Some(row) = groups.iter().find(|row| {
            row.get("name")
                .is_some_and(|name| name.trim().eq_ignore_ascii_case(group))
        }) else {
            return Self::unknown();
        };
        let policies = parse_policy_list(row.get("policy").map_or("", String::as_str));
        if policies.is_empty() {
            return Self::unknown();
        }
        Self {
            username: username.to_string(),
            group: group.to_string(),
            policies,
            known: true,
        }
    }

    #[must_use]
    pub fn is_known(&self) -> bool {
        self.known
    }

    #[must_use]
    pub fn allows(&self, policy: &str) -> bool {
        if !self.known {
            return true;
        }
        self.policies.contains(&policy.trim().to_ascii_lowercase())
    }

    /// True when the group is known and lacks `write`.
    #[must_use]
    pub fn inspect_only(&self) -> bool {
        self.known && !self.allows(POLICY_WRITE)
    }

    #[must_use]
    pub fn action_block_reason(&self, resource_id: &str, action: &ActionSpec) -> Option<String> {
        let policy = required_policy(resource_id, action)?;
        if self.allows(policy) {
            return None;
        }
        Some(permission_denied_copy(policy))
    }
}

/// Policy required to run `action` on `resource_id`, if any.
#[must_use]
pub fn required_policy(resource_id: &str, action: &ActionSpec) -> Option<&'static str> {
    if let Some((_, policy)) = POLICY_MENUS.iter().find(|(id, _)| *id == resource_id)
        && action_changes_state(action)
    {
        return Some(*policy);
    }
    match action.kind {
        ActionKind::Edit | ActionKind::Create => Some(POLICY_WRITE),
        ActionKind::Confirm { command } | ActionKind::Prompt { command } => {
            Some(command_policy(command))
        }
        ActionKind::Overlay { id } => overlay_policy(id),
    }
}

fn action_changes_state(action: &ActionSpec) -> bool {
    required_policy("", action).is_some()
}

fn command_policy(command: ActionCommand) -> &'static str {
    match command {
        ActionCommand::Reboot | ActionCommand::Shutdown | ActionCommand::ResetConfiguration => {
            POLICY_REBOOT
        }
        _ => POLICY_WRITE,
    }
}

fn overlay_policy(id: &str) -> Option<&'static str> {
    match id {
        "create-type" => Some(POLICY_WRITE),
        "torch" => Some(POLICY_SNIFF),
        "ping" | "traceroute" | "bandwidth-test" | "flood-ping" | "mac-scan" | "ip-scan"
        | "profiler" | "wifi-scan" => Some(POLICY_TEST),
        _ => None,
    }
}

/// Split a `RouterOS` `policy` attribute (`read,write,test`).
#[must_use]
pub fn parse_policy_list(raw: &str) -> HashSet<String> {
    raw.split([',', ';', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Operator-facing copy when a required group policy is missing.
#[must_use]
pub fn permission_denied_copy(policy: &str) -> String {
    match policy.trim().to_ascii_lowercase().as_str() {
        POLICY_WRITE => "No write on this menu. This account is inspect-only (READ MODE).".into(),
        POLICY_REBOOT => "This action needs the reboot policy on your user group.".into(),
        POLICY_TEST => "This action needs the test policy on your user group.".into(),
        POLICY_SNIFF => "This action needs the sniff policy on your user group.".into(),
        POLICY_POLICY => "This action needs the policy permission on your user group.".into(),
        POLICY_READ => "This account cannot read this menu.".into(),
        other => format!("Not enough permissions ({other})."),
    }
}

/// True when a `!trap` message is a permission failure.
#[must_use]
pub fn is_permission_trap(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("not enough permissions")
        || lower.contains("permission denied")
        || lower.contains("access denied")
        || lower.contains("not permitted")
        || lower.contains("no write")
}

/// Operator-facing copy for a permission `!trap`.
#[must_use]
pub fn trap_permission_copy(message: &str) -> String {
    if let Some(policy) = trap_policy_name(message) {
        return permission_denied_copy(policy);
    }
    if is_permission_trap(message) {
        return "Not enough permissions for this action.".into();
    }
    message.to_string()
}

fn trap_policy_name(message: &str) -> Option<&str> {
    let start = message.find('(')?;
    let end = message[start + 1..].find(')')?;
    let inner = message[start + 1..start + 1 + end].trim();
    if inner.is_empty() { None } else { Some(inner) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{ACTION_EDIT, ACTION_REBOOT, ACTION_TORCH};

    #[test]
    fn unknown_access_allows_writes() {
        let access = SessionAccess::unknown();
        assert!(!access.is_known());
        assert!(!access.inspect_only());
        assert!(access.allows(POLICY_WRITE));
        assert!(
            access
                .action_block_reason("interfaces", &ACTION_EDIT)
                .is_none()
        );
    }

    #[test]
    fn read_group_is_inspect_only() {
        let access = SessionAccess::from_policies("ops", "read", ["read", "web", "api"]);
        assert!(access.inspect_only());
        assert!(!access.allows(POLICY_WRITE));
        let reason = access
            .action_block_reason("interfaces", &ACTION_EDIT)
            .expect("blocked");
        assert!(reason.contains("READ MODE"));
        assert!(
            access
                .action_block_reason("interfaces", &ACTION_REBOOT)
                .is_some()
        );
    }

    #[test]
    fn write_without_reboot_blocks_reboot_only() {
        let access = SessionAccess::from_policies("ops", "write", ["read", "write", "api"]);
        assert!(!access.inspect_only());
        assert!(
            access
                .action_block_reason("interfaces", &ACTION_EDIT)
                .is_none()
        );
        let reason = access
            .action_block_reason("system-reboot", &ACTION_REBOOT)
            .expect("blocked");
        assert!(reason.contains("reboot"));
    }

    #[test]
    fn user_menu_needs_policy() {
        let access = SessionAccess::from_policies("ops", "write", ["read", "write"]);
        let reason = access
            .action_block_reason("users", &ACTION_EDIT)
            .expect("blocked");
        assert!(reason.contains("policy"));
    }

    #[test]
    fn torch_needs_sniff() {
        let access = SessionAccess::from_policies("ops", "write", ["read", "write"]);
        assert!(
            access
                .action_block_reason("interfaces", &ACTION_TORCH)
                .is_some()
        );
    }

    #[test]
    fn resolves_group_from_router_rows() {
        let users = vec![HashMap::from([
            ("name".into(), "ops".into()),
            ("group".into(), "read".into()),
        ])];
        let groups = vec![HashMap::from([
            ("name".into(), "read".into()),
            ("policy".into(), "local,read,web,api".into()),
        ])];
        let access = SessionAccess::from_router_rows("ops", &users, &groups);
        assert!(access.inspect_only());
        assert_eq!(access.group, "read");
    }

    #[test]
    fn missing_user_row_stays_unknown() {
        let access = SessionAccess::from_router_rows("ops", &[], &[]);
        assert!(!access.is_known());
    }

    #[test]
    fn trap_copy_uses_named_policy() {
        assert!(
            trap_permission_copy("failure: not enough permissions (write)").contains("READ MODE")
        );
        assert!(is_permission_trap("not enough permissions"));
        assert!(!is_permission_trap("no such item"));
    }
}
