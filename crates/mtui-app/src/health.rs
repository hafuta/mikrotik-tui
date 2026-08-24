//! Session drop, reconnect, and access-policy helpers.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use mtui_core::{ActionSpec, SessionAccess, trap_permission_copy};
use mtui_routeros::{ErrorKind, Resource};
use mtui_ui::{Signal, SignalLevel};

use crate::app::{App, AppCommand, ConnectIntent, Overlay, Screen};
use crate::session::{LinkState, SessionId};

impl App {
    pub(crate) fn note_data_ok(&mut self) {
        self.last_ok_at = Some(Instant::now());
    }

    pub(crate) fn reset_link(&mut self) {
        self.link = LinkState::Idle;
        self.last_ok_at = None;
        self.reconnect_at = None;
        self.reconnect_attempt = 0;
        self.access = SessionAccess::unknown();
        self.safe_mode = mtui_core::SafeModeStatus::default();
        self.last_safe_mode_verb = None;
        self.safe_mode_after = crate::safe_mode::SafeModeAfter::None;
    }

    pub(crate) fn mark_live(&mut self) {
        self.link = LinkState::Live;
        self.reconnect_at = None;
        self.reconnect_attempt = 0;
        self.note_data_ok();
    }

    pub(crate) fn mark_session_lost(
        &mut self,
        generation: u64,
        reason: impl Into<String>,
    ) -> Vec<AppCommand> {
        if generation != self.poll_generation {
            return Vec::new();
        }
        if self.demo.is_some() || self.screen != Screen::Main {
            return Vec::new();
        }
        if matches!(self.link, LinkState::Dropped | LinkState::Reconnecting) {
            return Vec::new();
        }
        let reason = reason.into();
        tracing::warn!(error = %reason, "session dropped");
        self.bump_request_generation();
        if self.safe_mode.we_hold() {
            self.held_safe_mode_at_drop = true;
            self.safe_mode.current = false;
        }
        self.client = None;
        self.link = LinkState::Dropped;
        self.refreshing = false;
        self.loading = false;
        self.close_remote_overlays();
        if self.login.uses_totp || !self.can_auto_reconnect() {
            self.reconnect_at = None;
            self.status = if self.held_safe_mode_at_drop {
                "Connection dropped while Safe Mode was on. The router will unroll those changes after it notices this session is gone.".into()
            } else {
                self.link_status_message()
            };
            return Vec::new();
        }
        self.begin_reconnect()
    }

    fn close_remote_overlays(&mut self) {
        if matches!(
            self.overlay,
            Overlay::Form(_)
                | Overlay::Confirm(_)
                | Overlay::ActionMenu(_)
                | Overlay::TypePicker(_)
                | Overlay::Torch(_)
                | Overlay::Probe(_)
                | Overlay::SafeModeConflict { .. }
                | Overlay::SafeModeLeave { .. }
        ) {
            self.overlay = Overlay::None;
        }
    }

    fn can_auto_reconnect(&self) -> bool {
        if self.login.uses_totp {
            return false;
        }
        self.pending_password
            .as_deref()
            .is_some_and(|password| !password.is_empty())
    }

    pub(crate) fn try_reconnect(&mut self) -> Vec<AppCommand> {
        if self.demo.is_some() {
            return Vec::new();
        }
        if self.link == LinkState::Reconnecting {
            self.status = self.link_status_message();
            return Vec::new();
        }
        if self.login.uses_totp || !self.can_auto_reconnect() {
            self.open_reauth(self.link_status_message());
            return Vec::new();
        }
        self.begin_reconnect()
    }

    pub(crate) fn begin_reconnect(&mut self) -> Vec<AppCommand> {
        self.connect_intent = ConnectIntent::Reconnect;
        self.link = LinkState::Reconnecting;
        self.reconnect_at = None;
        self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
        self.loading = false;
        self.refreshing = false;
        self.screen = Screen::Main;
        self.status = if self.held_safe_mode_at_drop {
            "Connection dropped while Safe Mode was on. Reconnecting, then unrolling that session's tagged changes.".into()
        } else {
            self.link_status_message()
        };
        vec![self.connect_command()]
    }

    pub(crate) fn on_reconnect_failed(
        &mut self,
        kind: ErrorKind,
        message: String,
    ) -> Vec<AppCommand> {
        if kind == ErrorKind::Auth {
            self.link = LinkState::Dropped;
            self.client = None;
            self.reconnect_at = None;
            self.open_reauth(message);
            return Vec::new();
        }
        self.link = LinkState::Dropped;
        self.client = None;
        self.schedule_reconnect();
        self.status = format!("{message} · {}", self.link_status_message());
        Vec::new()
    }

    fn schedule_reconnect(&mut self) {
        if !self.can_auto_reconnect() {
            self.reconnect_at = None;
            return;
        }
        self.reconnect_at = Some(Instant::now() + reconnect_delay(self.reconnect_attempt));
    }

    pub(crate) fn reconnect_tick(&mut self) -> Vec<AppCommand> {
        if self.link != LinkState::Dropped {
            return Vec::new();
        }
        let Some(at) = self.reconnect_at else {
            return Vec::new();
        };
        if Instant::now() < at {
            return Vec::new();
        }
        self.begin_reconnect()
    }

    pub(crate) fn fetch_access_command(&mut self) -> Vec<AppCommand> {
        vec![AppCommand::FetchAccess {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
        }]
    }

    pub(crate) fn apply_access(
        &mut self,
        users: &[Resource],
        groups: &[Resource],
        error: Option<&str>,
    ) {
        if error.is_some() {
            self.access = SessionAccess::unknown();
            return;
        }
        let users: Vec<HashMap<String, String>> =
            users.iter().map(|row| row.fields.clone()).collect();
        let groups: Vec<HashMap<String, String>> =
            groups.iter().map(|row| row.fields.clone()).collect();
        self.access = SessionAccess::from_router_rows(&self.login.username, &users, &groups);
    }

    pub(crate) fn link_status_message(&self) -> String {
        match self.link {
            LinkState::Dropped => {
                let age = self.data_age_label();
                if self.login.uses_totp || !self.can_auto_reconnect() {
                    format!("Connection dropped · last data {age} · press r to reconnect")
                } else {
                    format!("Connection dropped · last data {age}")
                }
            }
            LinkState::Reconnecting => {
                format!("Reconnecting… (attempt {})", self.reconnect_attempt.max(1))
            }
            LinkState::Live | LinkState::Idle => self.status.clone(),
        }
    }

    pub(crate) fn data_age_label(&self) -> String {
        format_age(self.last_ok_at)
    }

    pub(crate) fn link_signals(&self) -> Vec<Signal> {
        let mut out = Vec::new();
        match self.link {
            LinkState::Dropped => out.push(Signal::new(
                "LINK",
                format!("DOWN {}", self.data_age_label()),
                SignalLevel::Error,
            )),
            LinkState::Reconnecting => out.push(Signal::new(
                "LINK",
                format!("RETRY {}", self.reconnect_attempt.max(1)),
                SignalLevel::Warning,
            )),
            LinkState::Live | LinkState::Idle => {}
        }
        if self.access.inspect_only() && self.session_ready() {
            out.push(Signal::new("MODE", "READ", SignalLevel::Warning));
        }
        out.extend(self.safe_mode_signals());
        out
    }

    pub(crate) fn deny_if_unavailable(&mut self, action: &ActionSpec) -> bool {
        if !self.session_ready() {
            self.status = self.link_status_message();
            return true;
        }
        if let Some(reason) = self
            .access
            .action_block_reason(&self.current_resource, action)
        {
            self.status = reason;
            return true;
        }
        false
    }

    pub(crate) fn classify_write_error(message: &str) -> String {
        trap_permission_copy(message)
    }
}

fn reconnect_delay(attempt: u32) -> Duration {
    match attempt {
        0 | 1 => Duration::from_secs(2),
        2 => Duration::from_secs(5),
        3 => Duration::from_secs(15),
        _ => Duration::from_secs(30),
    }
}

fn format_age(since: Option<Instant>) -> String {
    let Some(since) = since else {
        return "unknown age".into();
    };
    let secs = Instant::now().saturating_duration_since(since).as_secs();
    if secs < 2 {
        "just now".into()
    } else if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_core::SessionAccess;
    use mtui_routeros::Resource;

    use crate::app::{Overlay, Pane, Screen};
    use crate::event::{AppEvent, WorkerMsg};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn main_with_interface() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.link = LinkState::Live;
        app.select_resource("interfaces");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "ether1".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: vec![Resource {
                id: "*1".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        app
    }

    #[test]
    fn age_labels_cover_buckets() {
        assert_eq!(format_age(Some(Instant::now())), "just now");
        assert_eq!(format_age(None), "unknown age");
    }

    #[test]
    fn backoff_grows() {
        assert_eq!(reconnect_delay(1), Duration::from_secs(2));
        assert_eq!(reconnect_delay(4), Duration::from_secs(30));
    }

    #[test]
    fn drop_keeps_rows_and_blocks_edit() {
        let mut app = main_with_interface();
        let generation = app.poll_generation;
        let _ = app.update(AppEvent::Worker(WorkerMsg::SessionLost {
            session: app.test_session(),
            generation,
            reason: "connection closed".into(),
        }));
        assert!(!app.session_ready());
        assert!(app.client.is_none());
        assert_eq!(app.table.rows.len(), 1);
        assert!(app.status.contains("Connection dropped"));
        assert!(
            app.header_signals()
                .iter()
                .any(|signal| signal.label == "LINK" && signal.value.contains("DOWN"))
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('e'))));
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.status.contains("Connection dropped"));
    }

    #[test]
    fn stale_drop_is_ignored() {
        let mut app = main_with_interface();
        let generation = app.poll_generation;
        let _ = app.update(AppEvent::Worker(WorkerMsg::SessionLost {
            session: app.test_session(),
            generation: generation.wrapping_sub(1),
            reason: "connection closed".into(),
        }));
        assert!(app.session_ready());
        assert_eq!(app.link, LinkState::Live);
    }

    #[test]
    fn totp_drop_does_not_auto_reconnect() {
        let mut app = main_with_interface();
        app.login.uses_totp = true;
        app.pending_password = Some("secret123456".into());
        let generation = app.poll_generation;
        let cmds = app.update(AppEvent::Worker(WorkerMsg::SessionLost {
            session: app.test_session(),
            generation,
            reason: "connection closed".into(),
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.link, LinkState::Dropped);
        assert!(app.status.contains("press r"));
    }

    #[test]
    fn remembered_password_auto_reconnects() {
        let mut app = main_with_interface();
        app.login.url = "192.168.88.1".into();
        app.login.username = "admin".into();
        app.pending_password = Some("secret".into());
        let generation = app.poll_generation;
        let cmds = app.update(AppEvent::Worker(WorkerMsg::SessionLost {
            session: app.test_session(),
            generation,
            reason: "connection closed".into(),
        }));
        assert_eq!(app.link, LinkState::Reconnecting);
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, AppCommand::Connect { .. })),
            "expected connect, got {cmds:?}"
        );
        assert_eq!(app.screen, Screen::Main);
    }

    #[test]
    fn inspect_only_blocks_edit_and_shows_read_mode() {
        let mut app = main_with_interface();
        app.access = SessionAccess::from_policies("ops", "read", ["read", "api"]);
        assert!(
            app.header_signals()
                .iter()
                .any(|signal| signal.label == "MODE" && signal.value == "READ")
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('e'))));
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.status.contains("READ MODE"));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('a'))));
        let Overlay::ActionMenu(menu) = &app.overlay else {
            panic!("expected action menu, got {:?}", app.overlay);
        };
        assert!(
            menu.items
                .iter()
                .any(|item| item.id == "edit" && item.note == "blocked"),
            "edit should stay listed: {:?}",
            menu.items
        );
    }

    #[test]
    fn unknown_access_still_opens_edit() {
        let mut app = main_with_interface();
        assert!(!app.access.is_known());
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('e'))));
        assert!(matches!(app.overlay, Overlay::Form(_)));
    }
}
