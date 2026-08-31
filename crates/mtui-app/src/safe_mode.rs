//! Safe Mode take/release/unroll on the control API session.

use std::collections::BTreeMap;

use mtui_core::{
    SAFE_MODE_HISTORY_LIMIT, SafeModeStatus, floating_undo_count, safe_mode_overflow_warning,
};
use mtui_routeros::Resource;
use mtui_ui::{Signal, SignalLevel};

use crate::app::{App, AppCommand, Overlay, Screen};
use crate::event::WorkerMsg;
use crate::session::SessionId;
use crate::write::MutationOp;

pub(crate) const SAFE_MODE_ENDPOINT: &str = "/safe-mode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafeModeVerb {
    Take,
    Release,
    Unroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeModeAfter {
    None,
    CloseTab,
    Quit,
    Logout,
    /// After steal-and-unroll on reconnect, leave Safe Mode so we do not stay on.
    DropHold,
}

impl App {
    pub(crate) fn fetch_safe_mode_command(&mut self) -> Vec<AppCommand> {
        vec![AppCommand::FetchSafeMode {
            session: SessionId::UNSTAMPED,
            generation: self.poll_generation,
        }]
    }

    pub(crate) fn fetch_safe_mode_if_ready(&mut self) -> Vec<AppCommand> {
        if self.screen == Screen::Main && self.session_ready() {
            self.fetch_safe_mode_command()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn apply_safe_mode_resource(&mut self, row: &Resource) {
        self.safe_mode = SafeModeStatus::from_fields(&row.fields);
        self.held_safe_mode_at_drop = self.safe_mode.we_hold();
    }

    pub(crate) fn note_history_rows(&mut self, rows: &[Resource]) {
        let maps: Vec<_> = rows.iter().map(|row| row.fields.clone()).collect();
        self.floating_undo_count = floating_undo_count(&maps);
    }

    pub(crate) fn safe_mode_signals(&self) -> Vec<Signal> {
        if self.safe_mode.we_hold() {
            let value = if safe_mode_overflow_warning(self.floating_undo_count).is_some() {
                format!(
                    "ON - history {}/{SAFE_MODE_HISTORY_LIMIT}, release and take again",
                    self.floating_undo_count
                )
            } else {
                "ON - changes unroll if this tab drops".into()
            };
            vec![Signal::new("SAFE", value, SignalLevel::Warning)]
        } else if self.safe_mode.foreign() {
            vec![Signal::new(
                "SAFE",
                format!(
                    "{} - another session holds it",
                    self.safe_mode.holder_label()
                ),
                SignalLevel::Error,
            )]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn toggle_safe_mode(&mut self) -> Vec<AppCommand> {
        if !self.session_ready() || self.screen != Screen::Main {
            self.status = self.link_status_message();
            return Vec::new();
        }
        if self.access.inspect_only() {
            self.status = "No write on this menu. This account is inspect-only (READ MODE).".into();
            return Vec::new();
        }
        if self.safe_mode.we_hold() {
            return self.safe_mode_command(SafeModeVerb::Release, SafeModeAfter::None);
        }
        if self.safe_mode.foreign() {
            self.overlay = Overlay::SafeModeConflict {
                owner: self.safe_mode.owner.clone(),
                user: self.safe_mode.user.clone(),
            };
            return Vec::new();
        }
        self.safe_mode_take("abort", SafeModeAfter::None)
    }

    pub(crate) fn unroll_safe_mode(&mut self) -> Vec<AppCommand> {
        if !self.session_ready() {
            self.status = self.link_status_message();
            return Vec::new();
        }
        if self.access.inspect_only() {
            self.status = "No write on this menu. This account is inspect-only (READ MODE).".into();
            return Vec::new();
        }
        self.safe_mode_command(SafeModeVerb::Unroll, SafeModeAfter::None)
    }

    pub(crate) fn safe_mode_take(
        &mut self,
        on_error: &str,
        after: SafeModeAfter,
    ) -> Vec<AppCommand> {
        let mut fields = BTreeMap::new();
        fields.insert("on-error".into(), on_error.to_string());
        self.last_safe_mode_verb = Some(SafeModeVerb::Take);
        self.safe_mode_after = after;
        vec![self.mutate_command(MutationOp::Command {
            endpoint: SAFE_MODE_ENDPOINT.into(),
            command: "take".into(),
            fields,
        })]
    }

    pub(crate) fn safe_mode_command(
        &mut self,
        verb: SafeModeVerb,
        after: SafeModeAfter,
    ) -> Vec<AppCommand> {
        let command = match verb {
            SafeModeVerb::Take => "take",
            SafeModeVerb::Release => "release",
            SafeModeVerb::Unroll => "unroll",
        };
        self.last_safe_mode_verb = Some(verb);
        self.safe_mode_after = after;
        vec![self.mutate_command(MutationOp::Command {
            endpoint: SAFE_MODE_ENDPOINT.into(),
            command: command.into(),
            fields: BTreeMap::new(),
        })]
    }

    pub(crate) fn confirm_safe_mode_conflict(&mut self, key: char) -> Vec<AppCommand> {
        self.overlay = Overlay::None;
        match key {
            'u' => self.safe_mode_take("unroll", SafeModeAfter::None),
            'r' => self.safe_mode_take("release", SafeModeAfter::None),
            _ => {
                self.status = "Left Safe Mode with the other session".into();
                Vec::new()
            }
        }
    }

    /// If this tab holds Safe Mode, ask before dropping the control socket.
    pub(crate) fn request_leave_with_safe_mode(
        &mut self,
        after: SafeModeAfter,
    ) -> Option<Vec<AppCommand>> {
        if !self.safe_mode.we_hold() {
            return None;
        }
        if !self.session_ready() {
            self.safe_mode = SafeModeStatus::default();
            return None;
        }
        self.overlay = Overlay::SafeModeLeave { next: after };
        Some(Vec::new())
    }

    pub(crate) fn confirm_safe_mode_leave(&mut self, unroll: bool) -> Vec<AppCommand> {
        let Overlay::SafeModeLeave { next } = self.overlay else {
            return Vec::new();
        };
        self.overlay = Overlay::None;
        let verb = if unroll {
            SafeModeVerb::Unroll
        } else {
            SafeModeVerb::Release
        };
        self.safe_mode_command(verb, next)
    }

    pub(crate) fn finish_safe_mode_mutate(
        &mut self,
        error: Option<&str>,
    ) -> Option<Vec<AppCommand>> {
        let verb = self.last_safe_mode_verb.take()?;
        if let Some(err) = error {
            self.safe_mode_after = SafeModeAfter::None;
            if verb == SafeModeVerb::Take && looks_like_safe_mode_busy(err) {
                self.overlay = Overlay::SafeModeConflict {
                    owner: self.safe_mode.owner.clone(),
                    user: self.safe_mode.user.clone(),
                };
                self.status = "Safe Mode is already taken".into();
                return Some(self.fetch_safe_mode_command());
            }
            self.status = format!("Safe Mode failed: {}", Self::classify_write_error(err));
            return Some(Vec::new());
        }
        let after = std::mem::replace(&mut self.safe_mode_after, SafeModeAfter::None);
        match verb {
            SafeModeVerb::Take => {
                self.safe_mode.enabled = true;
                self.safe_mode.current = true;
                self.held_safe_mode_at_drop = true;
            }
            SafeModeVerb::Release | SafeModeVerb::Unroll => {
                self.safe_mode = SafeModeStatus::default();
                self.held_safe_mode_at_drop = false;
            }
        }
        if matches!(
            after,
            SafeModeAfter::CloseTab | SafeModeAfter::Quit | SafeModeAfter::Logout
        ) {
            return Some(self.leave_after_safe_mode(after));
        }
        let mut cmds = self.fetch_safe_mode_command();
        if after == SafeModeAfter::DropHold {
            cmds.extend(self.safe_mode_command(SafeModeVerb::Release, SafeModeAfter::None));
        }
        if self.current_resource == "history" {
            cmds.extend(self.poll_current());
        }
        Some(cmds)
    }

    fn leave_after_safe_mode(&mut self, after: SafeModeAfter) -> Vec<AppCommand> {
        match after {
            SafeModeAfter::None | SafeModeAfter::DropHold => Vec::new(),
            SafeModeAfter::CloseTab => {
                if self.sessions.len() <= 1 {
                    return Vec::new();
                }
                let id = self.active;
                self.close_session(id);
                vec![AppCommand::CloseSession { session: id }]
            }
            SafeModeAfter::Quit => {
                self.should_quit = true;
                vec![AppCommand::Quit]
            }
            SafeModeAfter::Logout => {
                self.disconnect_to_profiles();
                vec![self.close_ssh_command()]
            }
        }
    }

    pub(crate) fn on_reconnect_safe_mode(&mut self) -> Vec<AppCommand> {
        if !self.held_safe_mode_at_drop {
            return self.fetch_safe_mode_command();
        }
        self.held_safe_mode_at_drop = false;
        self.safe_mode_take("unroll", SafeModeAfter::DropHold)
    }

    pub(crate) fn apply_safe_mode_result(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        let WorkerMsg::SafeModeResult {
            generation,
            row,
            error,
            ..
        } = msg
        else {
            return Vec::new();
        };
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Some(err) = error {
            tracing::warn!(error = %err, "safe-mode print failed");
            return Vec::new();
        }
        if let Some(row) = row {
            self.apply_safe_mode_resource(&row);
        }
        Vec::new()
    }
}

fn looks_like_safe_mode_busy(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("already") || lower.contains("taken") || lower.contains("hijack")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_core::SessionAccess;
    use std::collections::HashMap;

    use crate::app::Screen;
    use crate::event::AppEvent;
    use crate::session::LinkState;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn live_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.link = LinkState::Live;
        app.access = SessionAccess::full("admin", "full");
        app
    }

    fn holding() -> SafeModeStatus {
        SafeModeStatus {
            enabled: true,
            current: true,
            owner: "api".into(),
            user: "admin".into(),
        }
    }

    fn foreign_owner() -> SafeModeStatus {
        SafeModeStatus {
            enabled: true,
            current: false,
            owner: "api".into(),
            user: "admin".into(),
        }
    }

    fn is_safe_mode_command(cmd: &AppCommand, verb: &str) -> bool {
        matches!(
            cmd,
            AppCommand::Mutate {
                op: MutationOp::Command {
                    endpoint,
                    command,
                    ..
                },
                ..
            } if endpoint == SAFE_MODE_ENDPOINT && command == verb
        )
    }

    fn take_on_error(cmd: &AppCommand, on_error: &str) -> bool {
        matches!(
            cmd,
            AppCommand::Mutate {
                op: MutationOp::Command {
                    endpoint,
                    command,
                    fields,
                },
                ..
            } if endpoint == SAFE_MODE_ENDPOINT
                && command == "take"
                && fields.get("on-error").map(String::as_str) == Some(on_error)
        )
    }

    #[test]
    fn f4_take_when_idle() {
        let mut app = live_app();
        let cmds = app.toggle_safe_mode();
        assert!(matches!(
            cmds.as_slice(),
            [AppCommand::Mutate {
                op: MutationOp::Command { command, .. },
                ..
            }] if command == "take"
        ));
    }

    #[test]
    fn read_mode_blocks_take() {
        let mut app = live_app();
        app.access = SessionAccess::from_policies("op", "read", ["read"]);
        let cmds = app.toggle_safe_mode();
        assert!(cmds.is_empty());
        assert!(app.status.contains("READ MODE"));
    }

    #[test]
    fn foreign_owner_opens_conflict() {
        let mut app = live_app();
        app.safe_mode = foreign_owner();
        let cmds = app.toggle_safe_mode();
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::SafeModeConflict { .. }));
    }

    #[test]
    fn close_tab_asks_when_holding() {
        let mut app = live_app();
        app.safe_mode = holding();
        let _ = app.new_session();
        app.active = app.sessions[0].id;
        app.sessions[0].safe_mode = holding();
        app.sessions[0].screen = Screen::Main;
        app.sessions[0].link = LinkState::Live;
        let asked = app.request_leave_with_safe_mode(SafeModeAfter::CloseTab);
        assert!(asked.is_some());
        assert!(matches!(app.overlay, Overlay::SafeModeLeave { .. }));
    }

    #[test]
    fn keep_on_close_tab_switches_to_the_other_tab() {
        let mut app = live_app();
        app.poll_generation = 3;
        let closing = app.active;
        let kept = app.new_session().expect("second tab");
        app.active = closing;
        app.screen = Screen::Main;
        app.link = LinkState::Live;
        app.safe_mode = holding();
        app.last_safe_mode_verb = Some(SafeModeVerb::Release);
        app.safe_mode_after = SafeModeAfter::CloseTab;
        let cmds = app.update(AppEvent::Worker(WorkerMsg::MutateResult {
            session: closing,
            request_id: 1,
            generation: 3,
            error: None,
        }));
        assert!(app.session(closing).is_none());
        assert_eq!(app.active, kept);
        assert!(app.session(app.active).is_some());
        assert!(cmds.iter().any(|cmd| matches!(
            cmd,
            AppCommand::CloseSession { session } if *session == closing
        )));
        assert!(
            !cmds
                .iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
    }

    #[test]
    fn mutate_take_sets_held() {
        let mut app = live_app();
        app.last_safe_mode_verb = Some(SafeModeVerb::Take);
        let cmds = app.finish_safe_mode_mutate(None).expect("handled");
        assert!(app.safe_mode.we_hold());
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
    }

    #[test]
    fn take_shows_safe_on_in_the_header() {
        let mut app = live_app();
        app.last_safe_mode_verb = Some(SafeModeVerb::Take);
        let _ = app.finish_safe_mode_mutate(None);
        let signals = app.safe_mode_signals();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].label, "SAFE");
        assert!(
            signals[0].value.starts_with("ON -"),
            "{:?}",
            signals[0].value
        );
        assert!(
            signals[0].value.contains("unroll"),
            "{:?}",
            signals[0].value
        );
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: 1,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: Vec::new(),
            error: None,
        }));
        assert_eq!(app.safe_mode_signals()[0].label, "SAFE");
    }

    #[test]
    fn history_warns_near_limit() {
        let mut app = live_app();
        app.safe_mode.enabled = true;
        app.safe_mode.current = true;
        let mut rows = Vec::new();
        for i in 0..80 {
            let mut fields = HashMap::new();
            fields.insert("floating-undo".into(), "true".into());
            fields.insert("action".into(), format!("row-{i}"));
            rows.push(Resource {
                id: format!("*{i}"),
                fields,
            });
        }
        app.note_history_rows(&rows);
        let value = &app.safe_mode_signals()[0].value;
        assert!(value.contains("80/100"), "{value}");
    }

    #[test]
    fn print_clears_hold_when_another_session_took_it() {
        let mut app = live_app();
        app.safe_mode = holding();
        app.held_safe_mode_at_drop = true;
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("current".into(), "false".into());
        fields.insert("owner".into(), "api".into());
        fields.insert("user".into(), "admin".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::SafeModeResult {
            session: app.test_session(),
            generation: app.poll_generation,
            row: Some(Resource {
                id: "*1".into(),
                fields,
            }),
            error: None,
        }));
        assert!(cmds.is_empty());
        assert!(app.safe_mode.foreign());
        assert!(!app.safe_mode.we_hold());
        assert!(!app.held_safe_mode_at_drop);
        let value = &app.safe_mode_signals()[0].value;
        assert!(value.contains("another session holds it"), "{value}");
    }

    #[test]
    fn switching_tabs_refetches_safe_mode() {
        let mut app = live_app();
        let first = app.active;
        let second = app.new_session().expect("second tab");
        app.session_mut(second).expect("tab").screen = Screen::Main;
        app.session_mut(second).expect("tab").link = LinkState::Live;
        app.active = first;
        let cmds = app.cycle_session(1);
        assert_eq!(app.active, second);
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
    }

    #[test]
    fn f4_key_takes_safe_mode() {
        let mut app = live_app();
        let cmds = app.update(AppEvent::Input(press(KeyCode::F(4))));
        assert!(cmds.iter().any(|cmd| take_on_error(cmd, "abort")));
    }

    #[test]
    fn f4_releases_when_this_tab_holds() {
        let mut app = live_app();
        app.safe_mode = holding();
        let cmds = app.update(AppEvent::Input(press(KeyCode::F(4))));
        assert_eq!(cmds.len(), 1);
        assert!(is_safe_mode_command(&cmds[0], "release"));
    }

    #[test]
    fn conflict_unroll_takes_with_on_error_unroll() {
        let mut app = live_app();
        app.safe_mode = foreign_owner();
        let _ = app.update(AppEvent::Input(press(KeyCode::F(4))));
        assert!(matches!(app.overlay, Overlay::SafeModeConflict { .. }));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('u'))));
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(cmds.len(), 1);
        assert!(take_on_error(&cmds[0], "unroll"));
    }

    #[test]
    fn conflict_keep_takes_with_on_error_release() {
        let mut app = live_app();
        app.overlay = Overlay::SafeModeConflict {
            owner: "api".into(),
            user: "admin".into(),
        };
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('r'))));
        assert!(matches!(app.overlay, Overlay::None));
        assert!(take_on_error(&cmds[0], "release"));
    }

    #[test]
    fn conflict_leave_does_not_mutate() {
        let mut app = live_app();
        app.safe_mode = foreign_owner();
        app.overlay = Overlay::SafeModeConflict {
            owner: "api".into(),
            user: "admin".into(),
        };
        for key in [
            press(KeyCode::Char('d')),
            press(KeyCode::Esc),
            press(KeyCode::Char('n')),
        ] {
            app.overlay = Overlay::SafeModeConflict {
                owner: "api".into(),
                user: "admin".into(),
            };
            let cmds = app.update(AppEvent::Input(key));
            assert!(cmds.is_empty());
            assert!(matches!(app.overlay, Overlay::None));
            assert!(app.safe_mode.foreign());
            assert!(app.status.contains("other session"));
        }
    }

    #[test]
    fn busy_take_opens_conflict_and_refetches() {
        let mut app = live_app();
        app.safe_mode = foreign_owner();
        app.last_safe_mode_verb = Some(SafeModeVerb::Take);
        let cmds = app
            .finish_safe_mode_mutate(Some("failure: already taken"))
            .expect("handled");
        assert!(matches!(app.overlay, Overlay::SafeModeConflict { .. }));
        assert!(app.status.contains("already taken"));
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
    }

    #[test]
    fn hijack_error_also_opens_conflict() {
        let mut app = live_app();
        app.last_safe_mode_verb = Some(SafeModeVerb::Take);
        let _ = app.finish_safe_mode_mutate(Some("cannot hijack"));
        assert!(matches!(app.overlay, Overlay::SafeModeConflict { .. }));
    }

    #[test]
    fn palette_unroll_sends_unroll() {
        let mut app = live_app();
        app.safe_mode = holding();
        let _ = app.update(AppEvent::Input(ctrl(KeyCode::Char('k'))));
        for ch in "Unroll Safe Mode".chars() {
            let _ = app.update(AppEvent::Input(press(KeyCode::Char(ch))));
        }
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(matches!(app.overlay, Overlay::None));
        assert!(cmds.iter().any(|cmd| is_safe_mode_command(cmd, "unroll")));
    }

    #[test]
    fn quit_asks_when_holding_then_unroll_quits() {
        let mut app = live_app();
        app.safe_mode = holding();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('q'))));
        assert!(cmds.is_empty());
        assert!(matches!(
            app.overlay,
            Overlay::SafeModeLeave {
                next: SafeModeAfter::Quit
            }
        ));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('u'))));
        assert!(is_safe_mode_command(&cmds[0], "unroll"));
        app.poll_generation = 4;
        let cmds = app.update(AppEvent::Worker(WorkerMsg::MutateResult {
            session: app.test_session(),
            request_id: 1,
            generation: 4,
            error: None,
        }));
        assert!(app.should_quit);
        assert!(cmds.iter().any(|cmd| matches!(cmd, AppCommand::Quit)));
        assert!(
            !cmds
                .iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
    }

    #[test]
    fn leave_keep_sends_release() {
        let mut app = live_app();
        app.overlay = Overlay::SafeModeLeave {
            next: SafeModeAfter::Quit,
        };
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('r'))));
        assert!(is_safe_mode_command(&cmds[0], "release"));
    }

    #[test]
    fn leave_enter_unrolls() {
        let mut app = live_app();
        app.overlay = Overlay::SafeModeLeave {
            next: SafeModeAfter::Logout,
        };
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(is_safe_mode_command(&cmds[0], "unroll"));
    }

    #[test]
    fn leave_escape_stays_in_safe_mode() {
        let mut app = live_app();
        app.safe_mode = holding();
        app.overlay = Overlay::SafeModeLeave {
            next: SafeModeAfter::Quit,
        };
        let cmds = app.update(AppEvent::Input(press(KeyCode::Esc)));
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::None));
        assert!(app.safe_mode.we_hold());
        assert!(app.status.contains("Still in Safe Mode"));
        assert!(!app.should_quit);
    }

    #[test]
    fn logout_asks_when_holding_then_keep_disconnects() {
        let mut app = live_app();
        app.safe_mode = holding();
        let cmds = app.update(AppEvent::Input(ctrl(KeyCode::Char('l'))));
        assert!(cmds.is_empty());
        assert!(matches!(
            app.overlay,
            Overlay::SafeModeLeave {
                next: SafeModeAfter::Logout
            }
        ));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('r'))));
        app.poll_generation = 2;
        let _ = app.update(AppEvent::Worker(WorkerMsg::MutateResult {
            session: app.test_session(),
            request_id: 1,
            generation: 2,
            error: None,
        }));
        assert_eq!(app.screen, Screen::Login);
        assert!(!app.safe_mode.we_hold());
    }

    #[test]
    fn ctrl_w_asks_before_closing_a_holding_tab() {
        let mut app = live_app();
        app.safe_mode = holding();
        let _ = app.new_session();
        app.active = app.sessions[0].id;
        app.sessions[0].safe_mode = holding();
        app.sessions[0].screen = Screen::Main;
        app.sessions[0].link = LinkState::Live;
        let cmds = app.update(AppEvent::Input(ctrl(KeyCode::Char('w'))));
        assert!(cmds.is_empty());
        assert!(matches!(
            app.overlay,
            Overlay::SafeModeLeave {
                next: SafeModeAfter::CloseTab
            }
        ));
        assert_eq!(app.sessions.len(), 2);
    }

    #[test]
    fn reconnect_after_drop_unrolls_then_releases() {
        let mut app = live_app();
        app.held_safe_mode_at_drop = true;
        let cmds = app.on_reconnect_safe_mode();
        assert!(!app.held_safe_mode_at_drop);
        assert_eq!(cmds.len(), 1);
        assert!(take_on_error(&cmds[0], "unroll"));
        assert_eq!(app.safe_mode_after, SafeModeAfter::DropHold);
        let follow = app.finish_safe_mode_mutate(None).expect("take finished");
        assert!(app.safe_mode.we_hold());
        assert!(
            follow
                .iter()
                .any(|cmd| matches!(cmd, AppCommand::FetchSafeMode { .. }))
        );
        assert!(
            follow
                .iter()
                .any(|cmd| is_safe_mode_command(cmd, "release"))
        );
    }

    #[test]
    fn reconnect_without_hold_only_prints() {
        let mut app = live_app();
        let cmds = app.on_reconnect_safe_mode();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], AppCommand::FetchSafeMode { .. }));
    }

    #[test]
    fn stale_safe_mode_print_is_ignored() {
        let mut app = live_app();
        app.safe_mode = holding();
        let generation = app.poll_generation;
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("current".into(), "false".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::SafeModeResult {
            session: app.test_session(),
            generation: generation.wrapping_add(1),
            row: Some(Resource {
                id: "*1".into(),
                fields,
            }),
            error: None,
        }));
        assert!(cmds.is_empty());
        assert!(app.safe_mode.we_hold());
    }

    #[test]
    fn switching_to_a_login_tab_skips_safe_mode_fetch() {
        let mut app = live_app();
        let first = app.active;
        let second = app.new_session().expect("second tab");
        app.active = first;
        let cmds = app.cycle_session(1);
        assert_eq!(app.active, second);
        assert!(cmds.is_empty());
    }

    #[test]
    fn second_tab_unroll_then_first_tab_print_shows_foreign() {
        let mut app = live_app();
        let first = app.active;
        app.safe_mode = holding();
        let second = app.new_session().expect("second tab");
        {
            let tab = app.session_mut(second).expect("tab");
            tab.screen = Screen::Main;
            tab.link = LinkState::Live;
            tab.access = SessionAccess::full("admin", "full");
            tab.safe_mode = foreign_owner();
        }
        app.active = second;
        let _ = app.update(AppEvent::Input(press(KeyCode::F(4))));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('u'))));
        assert!(take_on_error(&cmds[0], "unroll"));
        let _ = app.finish_safe_mode_mutate(None);
        assert!(app.safe_mode.we_hold());
        let _ = app.cycle_session(-1);
        assert_eq!(app.active, first);
        assert!(app.safe_mode.we_hold());
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("current".into(), "false".into());
        fields.insert("owner".into(), "api".into());
        fields.insert("user".into(), "admin".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::SafeModeResult {
            session: first,
            generation: app.poll_generation,
            row: Some(Resource {
                id: "*1".into(),
                fields,
            }),
            error: None,
        }));
        assert!(app.safe_mode.foreign());
        assert!(!app.safe_mode.we_hold());
        let value = &app.safe_mode_signals()[0].value;
        assert!(value.contains("another session holds it"), "{value}");
    }

    #[test]
    fn mutate_unroll_clears_hold() {
        let mut app = live_app();
        app.safe_mode = holding();
        app.held_safe_mode_at_drop = true;
        app.last_safe_mode_verb = Some(SafeModeVerb::Unroll);
        let _ = app.finish_safe_mode_mutate(None);
        assert!(!app.safe_mode.we_hold());
        assert!(!app.held_safe_mode_at_drop);
    }
}
