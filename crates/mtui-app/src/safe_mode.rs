//! Safe Mode take/release/unroll on the control API session.

use std::collections::BTreeMap;

use mtui_core::{SafeModeStatus, floating_undo_count, safe_mode_overflow_warning};
use mtui_routeros::Resource;
use mtui_ui::{Signal, SignalLevel};

use crate::app::{App, AppCommand, Overlay, Screen};
use crate::event::WorkerMsg;
use crate::session::SessionId;
use crate::write::MutationOp;

pub(crate) const SAFE_MODE_ENDPOINT: &str = "/rest/safe-mode";

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

    pub(crate) fn apply_safe_mode_resource(&mut self, row: &Resource) {
        self.safe_mode = SafeModeStatus::from_fields(&row.fields);
        if self.safe_mode.we_hold() {
            self.held_safe_mode_at_drop = true;
        }
        if let Some(warning) = safe_mode_overflow_warning(self.floating_undo_count)
            && self.safe_mode.we_hold()
        {
            self.status = warning;
        }
    }

    pub(crate) fn note_history_rows(&mut self, rows: &[Resource]) {
        let maps: Vec<_> = rows.iter().map(|row| row.fields.clone()).collect();
        self.floating_undo_count = floating_undo_count(&maps);
        if self.safe_mode.we_hold()
            && let Some(warning) = safe_mode_overflow_warning(self.floating_undo_count)
        {
            self.status = warning;
        }
    }

    pub(crate) fn safe_mode_signals(&self) -> Vec<Signal> {
        let mut out = Vec::new();
        if self.safe_mode.we_hold() {
            out.push(Signal::new("SAFE", "ON", SignalLevel::Warning));
        } else if self.safe_mode.foreign() {
            out.push(Signal::new(
                "SAFE",
                self.safe_mode.holder_label(),
                SignalLevel::Error,
            ));
        }
        out
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
        self.status = "Taking Safe Mode…".into();
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
        self.status = match verb {
            SafeModeVerb::Take => "Taking Safe Mode…".into(),
            SafeModeVerb::Release => "Leaving Safe Mode (commit)…".into(),
            SafeModeVerb::Unroll => "Unrolling Safe Mode…".into(),
        };
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
                self.status = "Safe Mode on. Changes unroll if this tab drops.".into();
            }
            SafeModeVerb::Release => {
                self.safe_mode = SafeModeStatus::default();
                self.held_safe_mode_at_drop = false;
                self.status = "Safe Mode off. Changes kept.".into();
            }
            SafeModeVerb::Unroll => {
                self.safe_mode = SafeModeStatus::default();
                self.held_safe_mode_at_drop = false;
                self.status = "Safe Mode unrolled. Pending changes undone.".into();
            }
        }
        let mut cmds = self.fetch_safe_mode_command();
        if after == SafeModeAfter::DropHold {
            cmds.extend(self.safe_mode_command(SafeModeVerb::Release, SafeModeAfter::None));
        } else {
            cmds.extend(self.leave_after_safe_mode(after));
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
                Vec::new()
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

    fn live_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.link = LinkState::Live;
        app.access = SessionAccess::full("admin", "full");
        app
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
        app.safe_mode = SafeModeStatus {
            enabled: true,
            current: false,
            owner: "winbox".into(),
            user: "admin".into(),
        };
        let cmds = app.toggle_safe_mode();
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::SafeModeConflict { .. }));
    }

    #[test]
    fn close_tab_asks_when_holding() {
        let mut app = live_app();
        app.safe_mode = SafeModeStatus {
            enabled: true,
            current: true,
            owner: "api".into(),
            user: "admin".into(),
        };
        let _ = app.new_session();
        app.active = app.sessions[0].id;
        app.sessions[0].safe_mode = SafeModeStatus {
            enabled: true,
            current: true,
            owner: "api".into(),
            user: "admin".into(),
        };
        app.sessions[0].screen = Screen::Main;
        app.sessions[0].link = LinkState::Live;
        let asked = app.request_leave_with_safe_mode(SafeModeAfter::CloseTab);
        assert!(asked.is_some());
        assert!(matches!(app.overlay, Overlay::SafeModeLeave { .. }));
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
        assert!(app.status.contains("80/100"));
    }

    #[test]
    fn f4_key_takes_safe_mode() {
        let mut app = live_app();
        let cmds = app.update(AppEvent::Input(press(KeyCode::F(4))));
        assert!(cmds.iter().any(|cmd| matches!(
            cmd,
            AppCommand::Mutate {
                op: MutationOp::Command { command, .. },
                ..
            } if command == "take"
        )));
    }
}
