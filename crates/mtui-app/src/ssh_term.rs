//! Docked SSH terminal: key encoding, VT snapshot, and session helpers.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mtui_ui::{TerminalCell, TerminalLink, TerminalState};
use ratatui::style::Color;
use vt100::Parser;

use crate::app::{App, AppCommand, Overlay, Pane};
use crate::write::{ConfirmSession, MutationOp};
use mtui_core::ActionCommand;

/// Encode a key for a `RouterOS` PTY. Enter is CR.
#[must_use]
pub fn encode_key(key: KeyEvent) -> Vec<u8> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return encode_ctrl(key);
    }
    match key.code {
        KeyCode::Char(ch) => ch.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Backspace | KeyCode::Delete => vec![0x7f],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::F(n) => encode_fn(n),
        _ => Vec::new(),
    }
}

fn encode_ctrl(key: KeyEvent) -> Vec<u8> {
    match key.code {
        KeyCode::Char(ch) => {
            let lower = ch.to_ascii_lowercase();
            if lower.is_ascii_lowercase() {
                vec![lower as u8 - b'a' + 1]
            } else if ch == '@' {
                vec![0]
            } else {
                Vec::new()
            }
        }
        KeyCode::Left => b"\x1b[1;5D".to_vec(),
        KeyCode::Right => b"\x1b[1;5C".to_vec(),
        KeyCode::Up => b"\x1b[1;5A".to_vec(),
        KeyCode::Down => b"\x1b[1;5B".to_vec(),
        _ => Vec::new(),
    }
}

fn encode_fn(n: u8) -> Vec<u8> {
    match n {
        1 => b"\x1bOP".to_vec(),
        2 => b"\x1bOQ".to_vec(),
        3 => b"\x1bOR".to_vec(),
        4 => b"\x1bOS".to_vec(),
        5 => b"\x1b[15~".to_vec(),
        6 => b"\x1b[17~".to_vec(),
        7 => b"\x1b[18~".to_vec(),
        8 => b"\x1b[19~".to_vec(),
        9 => b"\x1b[20~".to_vec(),
        10 => b"\x1b[21~".to_vec(),
        11 => b"\x1b[23~".to_vec(),
        _ => Vec::new(),
    }
}

/// Tall PTY so a root `/export` stays on the grid. The dock paints a viewport.
/// `RouterOS` fills unused rows with blanks; Shift+PgUp can reach those.
const SSH_PTY_ROWS: u16 = 8192;

fn snapshot_viewport(parser: &Parser, term: &mut TerminalState, view_h: u16) {
    let screen = parser.screen();
    let (prows, cols) = screen.size();
    let view_h = view_h.min(prows).max(1);
    let (crow, ccol) = screen.cursor_position();
    let follow_top = crow.saturating_sub(view_h.saturating_sub(1));
    let max_scroll = usize::from(follow_top);
    if term.scroll_offset > max_scroll {
        term.scroll_offset = max_scroll;
    }
    let top = follow_top.saturating_sub(u16::try_from(term.scroll_offset).unwrap_or(0));
    term.resize_grid(cols, view_h);
    term.cursor_row = crow.saturating_sub(top);
    term.cursor_col = ccol.min(cols.saturating_sub(1));
    for row in 0..view_h {
        for col in 0..cols {
            let idx = usize::from(row)
                .saturating_mul(usize::from(cols))
                .saturating_add(usize::from(col));
            let cell = screen.cell(top.saturating_add(row), col);
            let ch = cell
                .and_then(|c| c.contents().chars().next())
                .unwrap_or(' ');
            let fg = cell.map_or(Color::Reset, |c| map_vt_color(c.fgcolor()));
            let bold = cell.is_some_and(vt100::Cell::bold);
            if let Some(slot) = term.cells.get_mut(idx) {
                *slot = TerminalCell { ch, fg, bold };
            }
        }
    }
}

fn map_vt_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(idx) => Color::Indexed(idx),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

impl App {
    pub(crate) fn dock_occupied(&self) -> bool {
        self.console.visible || self.terminal.visible
    }

    pub(crate) fn dock_fullscreen(&self) -> bool {
        (self.console.visible && self.console.fullscreen)
            || (self.terminal.visible && self.terminal.fullscreen)
    }

    pub(crate) fn toggle_terminal(&mut self) -> Vec<AppCommand> {
        if self.screen != crate::app::Screen::Main {
            return Vec::new();
        }
        if self.terminal.visible {
            return self.hide_terminal(true);
        }
        self.show_terminal()
    }

    fn show_terminal(&mut self) -> Vec<AppCommand> {
        if self.console.visible {
            self.console.visible = false;
            self.console.fullscreen = false;
        }
        if self.pane != Pane::Terminal {
            self.pane_before_terminal = self.pane;
        }
        self.terminal.visible = true;
        self.pane = Pane::Terminal;
        self.status = "Terminal shown".into();
        if self.terminal.link == TerminalLink::Live {
            return self.resize_terminal_pty();
        }
        self.start_terminal_session()
    }

    pub(crate) fn hide_terminal(&mut self, refresh: bool) -> Vec<AppCommand> {
        if !self.terminal.visible {
            return Vec::new();
        }
        self.terminal.visible = false;
        self.terminal.fullscreen = false;
        self.pane = self.restore_after_dock(self.pane_before_terminal);
        self.status = "Terminal hidden".into();
        if refresh && self.session_ready() {
            self.poll_generation = self.poll_generation.wrapping_add(1);
            return self.poll_current();
        }
        Vec::new()
    }

    fn start_terminal_session(&mut self) -> Vec<AppCommand> {
        if self.demo.is_some() {
            self.terminal.link = TerminalLink::Connecting;
            self.ensure_parser();
            self.terminal_generation = self.terminal_generation.wrapping_add(1);
            let (cols, rows) = self.ssh_pty_size();
            return vec![AppCommand::OpenSsh {
                session: crate::session::SessionId::UNSTAMPED,
                generation: self.terminal_generation,
                host: "demo".into(),
                port: 22,
                username: String::new(),
                password: String::new(),
                expected_fingerprint: None,
                cols: u32::from(cols),
                rows: u32::from(rows),
            }];
        }
        if !self.session_ready() {
            self.terminal.link = TerminalLink::Failed;
            self.terminal.error = Some("Connect to the router first".into());
            return Vec::new();
        }
        self.terminal.host = mtui_routeros::header_host(&self.login.url);
        if self.terminal.port == 0 {
            self.terminal.port = mtui_ssh::DEFAULT_SSH_PORT;
        }
        self.terminal.link = TerminalLink::Connecting;
        self.terminal.error = None;
        self.ensure_parser();
        self.terminal_generation = self.terminal_generation.wrapping_add(1);
        let mut cmds = self.open_ssh_after_probe(self.terminal.port);
        if !cmds.is_empty() {
            cmds.push(AppCommand::ProbeSshService {
                session: crate::session::SessionId::UNSTAMPED,
                generation: self.terminal_generation,
            });
        }
        cmds
    }

    fn ensure_parser(&mut self) {
        let (cols, pty_rows) = self.ssh_pty_size();
        let view_h = self.terminal_inner_size().1;
        let id = self.active;
        let Some(session) = self.session_mut(id) else {
            return;
        };
        if session.vt_parser.is_none() {
            session.vt_parser = Some(Parser::new(pty_rows, cols, 0));
        }
        if let Some(parser) = session.vt_parser.as_mut() {
            let (prows, pcols) = parser.screen().size();
            if prows != pty_rows || pcols != cols {
                parser.set_size(pty_rows, cols);
            }
        }
        if let Some(parser) = session.vt_parser.as_ref() {
            snapshot_viewport(parser, &mut session.terminal, view_h);
        }
    }

    pub(crate) fn terminal_inner_size(&self) -> (u16, u16) {
        let height = self.console_layout_height().saturating_sub(2).max(1);
        let width = self.terminal_width.saturating_sub(2).max(1);
        (width, height)
    }

    fn ssh_pty_size(&self) -> (u16, u16) {
        let (cols, view_rows) = self.terminal_inner_size();
        (cols.max(1), view_rows.max(SSH_PTY_ROWS))
    }

    pub(crate) fn resize_terminal_pty(&mut self) -> Vec<AppCommand> {
        if !self.terminal.visible || self.terminal.link != TerminalLink::Live {
            return Vec::new();
        }
        let (cols, pty_rows) = self.ssh_pty_size();
        let view_h = self.terminal_inner_size().1;
        let id = self.active;
        let mut size_changed = true;
        if let Some(session) = self.session_mut(id) {
            if let Some(parser) = session.vt_parser.as_mut() {
                let (prows, pcols) = parser.screen().size();
                size_changed = prows != pty_rows || pcols != cols;
                if size_changed {
                    parser.set_size(pty_rows, cols);
                }
                snapshot_viewport(parser, &mut session.terminal, view_h);
            } else {
                session.terminal.resize_grid(cols, view_h);
            }
        }
        if !size_changed {
            return Vec::new();
        }
        vec![AppCommand::SshResize {
            session: crate::session::SessionId::UNSTAMPED,
            generation: self.terminal_generation,
            cols: u32::from(cols),
            rows: u32::from(pty_rows),
        }]
    }

    pub(crate) fn apply_ssh_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.ensure_parser();
        let view_h = self.terminal_inner_size().1;
        let id = self.active;
        if let Some(session) = self.session_mut(id)
            && let Some(parser) = session.vt_parser.as_mut()
        {
            parser.process(bytes);
            snapshot_viewport(parser, &mut session.terminal, view_h);
        }
    }

    pub(crate) fn scroll_terminal(&mut self, delta: isize) {
        let view_h = self.terminal_inner_size().1;
        let id = self.active;
        let Some(session) = self.session_mut(id) else {
            return;
        };
        let Some(parser) = session.vt_parser.as_ref() else {
            return;
        };
        let (crow, _) = parser.screen().cursor_position();
        let follow_top = crow.saturating_sub(view_h.saturating_sub(1));
        let max = usize::from(follow_top);
        let next = isize::try_from(session.terminal.scroll_offset).unwrap_or(0) + delta;
        session.terminal.scroll_offset =
            usize::try_from(next.clamp(0, isize::try_from(max).unwrap_or(0))).unwrap_or(0);
        snapshot_viewport(parser, &mut session.terminal, view_h);
        session.status = if session.terminal.scroll_offset == 0 {
            "Terminal follow".into()
        } else {
            format!("Terminal scroll {}/{max}", session.terminal.scroll_offset)
        };
    }

    pub(crate) fn keys_terminal(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.code == KeyCode::F(12) {
            return self.hide_terminal(true);
        }
        if key.code == KeyCode::Char('`') && key.modifiers.is_empty() {
            self.toggle_console();
            return Vec::new();
        }
        if key.code == KeyCode::F(11) {
            self.terminal.toggle_fullscreen();
            self.status = if self.terminal.fullscreen {
                "Terminal fullscreen".into()
            } else {
                "Terminal docked".into()
            };
            return self.resize_terminal_pty();
        }
        if key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(
                key.code,
                KeyCode::PageUp | KeyCode::PageDown | KeyCode::Up | KeyCode::Down
            )
        {
            let page = isize::try_from(self.terminal.rows.saturating_sub(1).max(1)).unwrap_or(1);
            let delta = match key.code {
                KeyCode::PageUp => page,
                KeyCode::PageDown => -page,
                KeyCode::Up => 1,
                KeyCode::Down => -1,
                _ => 0,
            };
            self.scroll_terminal(delta);
            return Vec::new();
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) && matches!(key.code, KeyCode::Tab)
            || matches!(key.code, KeyCode::BackTab)
        {
            self.pane = self.restore_after_dock(self.pane_before_terminal);
            return Vec::new();
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('v') {
            let text = arboard::Clipboard::new()
                .ok()
                .and_then(|mut clip| clip.get_text().ok())
                .unwrap_or_default();
            if text.is_empty() {
                return Vec::new();
            }
            return vec![AppCommand::SshWrite {
                session: crate::session::SessionId::UNSTAMPED,
                generation: self.terminal_generation,
                bytes: text.replace('\n', "\r").into_bytes(),
            }];
        }
        let bytes = encode_key(key);
        if bytes.is_empty() {
            return Vec::new();
        }
        vec![AppCommand::SshWrite {
            session: crate::session::SessionId::UNSTAMPED,
            generation: self.terminal_generation,
            bytes,
        }]
    }

    pub(crate) fn on_ssh_service(
        &mut self,
        generation: u64,
        port: u16,
        disabled: bool,
        id: String,
        error: Option<String>,
    ) -> Vec<AppCommand> {
        if generation != self.terminal_generation {
            return Vec::new();
        }
        if let Some(error) = error {
            if self.terminal.link != TerminalLink::Live {
                self.terminal.link = TerminalLink::Failed;
                self.terminal.error = Some(error);
            }
            return Vec::new();
        }
        let dialed = self.terminal.port;
        self.terminal.port = port;
        let host = mtui_routeros::header_host(&self.login.url);
        self.terminal.host.clone_from(&host);
        if disabled {
            self.overlay = Overlay::Confirm(ConfirmSession {
                title: "Enable SSH".into(),
                body: format!(
                    "SSH is disabled on {host}:{port}. Enable `/ip service ssh` so New Terminal can open a RouterOS CLI?"
                ),
                action_id: "enable-ssh".into(),
                command: ActionCommand::Enable,
                record_id: id,
                record_ids: Vec::new(),
                record_name: "ssh".into(),
                endpoint: "/ip/service".into(),
                fields: [("disabled".into(), "false".into())].into(),
            });
            self.terminal.link = TerminalLink::Idle;
            return vec![self.close_ssh_command()];
        }
        if matches!(
            self.terminal.link,
            TerminalLink::Live | TerminalLink::Connecting
        ) && port == dialed
        {
            return Vec::new();
        }
        if matches!(
            self.terminal.link,
            TerminalLink::Live | TerminalLink::Connecting
        ) {
            self.terminal_generation = self.terminal_generation.wrapping_add(1);
        }
        self.open_ssh_after_probe(port)
    }

    fn open_ssh_after_probe(&mut self, port: u16) -> Vec<AppCommand> {
        let password = self
            .pending_password
            .clone()
            .unwrap_or_else(|| self.login.password.clone());
        if password.is_empty() && self.demo.is_none() {
            self.terminal.link = TerminalLink::Failed;
            self.terminal.error = Some("Password required for SSH".into());
            return Vec::new();
        }
        let expected = self
            .named_profile()
            .map(|profile| profile.ssh_host_key_fingerprint.clone())
            .filter(|fp| !fp.is_empty());
        let (cols, rows) = self.ssh_pty_size();
        self.terminal.port = port;
        self.terminal.link = TerminalLink::Connecting;
        vec![AppCommand::OpenSsh {
            session: crate::session::SessionId::UNSTAMPED,
            generation: self.terminal_generation,
            host: self.terminal.host.clone(),
            port,
            username: self.login.username.clone(),
            password,
            expected_fingerprint: expected,
            cols: u32::from(cols),
            rows: u32::from(rows),
        }]
    }

    pub(crate) fn confirm_enable_ssh(&mut self) -> Vec<AppCommand> {
        let Overlay::Confirm(session) = &self.overlay else {
            return Vec::new();
        };
        if session.action_id != "enable-ssh" {
            return Vec::new();
        }
        let id = session.record_id.clone();
        self.overlay = Overlay::None;
        self.ssh_enable_pending = true;
        self.status = "Enabling SSH…".into();
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("disabled".into(), "false".into());
        vec![self.mutate_command(MutationOp::Patch {
            endpoint: "/ip/service".into(),
            id: Some(id),
            fields,
        })]
    }

    pub(crate) fn finish_enable_ssh(&mut self, failed: bool) -> Vec<AppCommand> {
        self.ssh_enable_pending = false;
        if failed {
            self.terminal.link = TerminalLink::Failed;
            self.terminal.error = Some("Could not enable SSH".into());
            return Vec::new();
        }
        let port = self.terminal.port.max(mtui_ssh::DEFAULT_SSH_PORT);
        self.open_ssh_after_probe(port)
    }

    pub(crate) fn on_ssh_ready(
        &mut self,
        generation: u64,
        fingerprint: &str,
        stages_ms: &str,
    ) -> Vec<AppCommand> {
        if generation != self.terminal_generation {
            return Vec::new();
        }
        self.terminal.link = TerminalLink::Live;
        self.terminal.error = None;
        self.persist_ssh_fingerprint(fingerprint);
        self.status = if stages_ms.is_empty() {
            "Terminal connected".into()
        } else {
            format!("Terminal connected ({stages_ms})")
        };
        self.resize_terminal_pty()
    }

    pub(crate) fn on_ssh_closed(
        &mut self,
        generation: u64,
        error: Option<String>,
    ) -> Vec<AppCommand> {
        if generation != self.terminal_generation {
            return Vec::new();
        }
        if matches!(
            &self.overlay,
            Overlay::Confirm(session) if session.action_id == "enable-ssh"
        ) {
            return Vec::new();
        }
        self.terminal.link = TerminalLink::Failed;
        self.terminal.error = Some(error.unwrap_or_else(|| "SSH closed".into()));
        self.status = self
            .terminal
            .error
            .clone()
            .unwrap_or_else(|| "SSH closed".into());
        Vec::new()
    }

    fn persist_ssh_fingerprint(&mut self, fingerprint: &str) {
        if fingerprint.is_empty()
            || self.demo.is_some()
            || (cfg!(test) && self.current_profile.is_empty())
        {
            return;
        }
        let Some(mut profile) = self.named_profile() else {
            return;
        };
        if profile
            .ssh_host_key_fingerprint
            .eq_ignore_ascii_case(fingerprint)
        {
            return;
        }
        profile.ssh_host_key_fingerprint = fingerprint.to_string();
        let _ = self.profiles.upsert(profile);
    }

    pub(crate) fn restore_after_dock(&self, saved: Pane) -> Pane {
        match saved {
            Pane::Terminal if !self.terminal.visible => Pane::Content,
            Pane::Console if !self.console.visible => Pane::Content,
            Pane::Terminal | Pane::Console => saved,
            other => other,
        }
    }

    pub(crate) fn close_ssh_command(&self) -> AppCommand {
        AppCommand::CloseSsh {
            session: crate::session::SessionId::UNSTAMPED,
            generation: self.terminal_generation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::encode_key;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn enter_is_carriage_return() {
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(encode_key(key), b"\r");
    }

    #[test]
    fn ctrl_c_is_etx() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(encode_key(key), [0x03]);
    }

    #[test]
    fn tab_is_ht() {
        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(encode_key(key), b"\t");
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn demo_main() -> crate::app::App {
        let mut app = crate::app::App::new(false).expect("app");
        let _ = app.enter_demo();
        app
    }

    #[test]
    fn f12_opens_terminal_and_hides_logs() {
        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::Char('`'))));
        assert!(app.console.visible);
        let cmds = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        assert!(app.terminal.visible);
        assert!(!app.console.visible);
        assert_eq!(app.pane, crate::app::Pane::Terminal);
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, crate::app::AppCommand::OpenSsh { .. })),
            "demo should open a canned PTY, got {cmds:?}"
        );
    }

    #[test]
    fn backtick_hides_terminal_ui_and_keeps_link() {
        use mtui_ui::TerminalLink;

        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        app.terminal.link = TerminalLink::Live;
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::Char('`'))));
        assert!(app.console.visible);
        assert!(!app.terminal.visible);
        assert_eq!(app.terminal.link, TerminalLink::Live);
        assert_eq!(app.pane, crate::app::Pane::Console);
    }

    #[test]
    fn f12_hides_terminal_and_refreshes() {
        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        let cmds = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        assert!(!app.terminal.visible);
        assert_ne!(app.pane, crate::app::Pane::Terminal);
        assert!(
            !cmds.is_empty(),
            "hide should bump poll generation and refresh"
        );
    }

    #[test]
    fn ctrl_c_in_terminal_does_not_quit() {
        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        let cmds = app.update(crate::event::AppEvent::Input(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));
        assert!(!app.should_quit);
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::SshWrite { bytes, .. } if bytes.as_slice() == [0x03]
            )),
            "expected ETX write, got {cmds:?}"
        );
    }

    #[test]
    fn apply_ssh_bytes_fills_the_grid() {
        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        app.apply_ssh_bytes(b"hello");
        let text: String = app.terminal.cells.iter().map(|cell| cell.ch).collect();
        assert!(text.contains("hello"), "{text:?}");
    }

    fn visible_text(app: &crate::app::App) -> String {
        let cols = usize::from(app.terminal.cols.max(1));
        app.terminal
            .cells
            .chunks(cols)
            .map(|row| row.iter().map(|cell| cell.ch).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn chunked_pty_bytes_keep_the_full_prompt() {
        let mut app = demo_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        for byte in b"[admin@MikroTik] > " {
            app.apply_ssh_bytes(&[*byte]);
        }
        let text = visible_text(&app);
        assert!(
            text.contains("[admin@MikroTik] >"),
            "parser must not reset on each SSH chunk, got {text:?}"
        );
    }

    #[test]
    fn export_dump_stays_on_the_last_screenful_and_can_scroll() {
        let mut app = demo_main();
        app.terminal_width = 80;
        app.terminal_height = 24;
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        for i in 0..400_u32 {
            let line = format!("line {i:03}\r\n");
            app.apply_ssh_bytes(line.as_bytes());
        }
        let live = visible_text(&app);
        assert!(
            live.contains("line 399"),
            "live view should keep the tail of the dump, got {live:?}"
        );
        assert!(
            !live.contains("line 000"),
            "the first export lines should have scrolled off the live screen, got {live:?}"
        );
        app.scroll_terminal(10_000);
        let scrolled = visible_text(&app);
        assert!(
            app.terminal.scroll_offset > 0,
            "Shift+PgUp equivalent should leave follow mode"
        );
        assert!(
            scrolled.contains("line 000"),
            "scrollback should expose the start of the dump, got {scrolled:?}"
        );
    }

    #[test]
    fn shift_page_up_scrolls_locally_without_pty_bytes() {
        let mut app = demo_main();
        app.terminal_width = 80;
        app.terminal_height = 24;
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        for i in 0..40_u32 {
            app.apply_ssh_bytes(format!("line {i:02}\r\n").as_bytes());
        }
        let cmds = app.keys_terminal(KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT));
        assert!(
            cmds.is_empty(),
            "local scroll must not write the PTY, got {cmds:?}"
        );
        assert!(app.terminal.scroll_offset > 0);
    }

    fn live_main() -> crate::app::App {
        let mut app = crate::app::App::new(false).expect("app");
        app.screen = crate::app::Screen::Main;
        app.link = crate::session::LinkState::Live;
        app.login.url = "192.168.88.1".into();
        app.login.username = "admin".into();
        app.login.password = "secret".into();
        app.pending_password = Some("secret".into());
        app
    }

    #[test]
    fn live_terminal_dials_ssh_without_waiting_for_the_service_probe() {
        let mut app = live_main();
        let cmds = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        match cmds.first() {
            Some(crate::app::AppCommand::OpenSsh { host, port, .. }) => {
                assert_eq!(host, "192.168.88.1");
                assert_eq!(*port, 22);
            }
            other => panic!("expected OpenSsh first, got {other:?}"),
        }
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, crate::app::AppCommand::ProbeSshService { .. })),
            "probe should run in parallel, got {cmds:?}"
        );
    }

    #[test]
    fn enabled_ssh_probe_does_not_start_a_second_dial() {
        let mut app = live_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        let cmds = app.on_ssh_service(app.terminal_generation, 22, false, "*ssh".into(), None);
        assert!(cmds.is_empty(), "already dialing :22, got {cmds:?}");
    }

    #[test]
    fn disabled_ssh_probe_cancels_the_in_flight_dial() {
        let mut app = live_main();
        let _ = app.update(crate::event::AppEvent::Input(press(KeyCode::F(12))));
        let cmds = app.on_ssh_service(app.terminal_generation, 22, true, "*ssh".into(), None);
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, crate::app::AppCommand::CloseSsh { .. })),
            "disabled SSH should abort the TCP dial, got {cmds:?}"
        );
        assert_eq!(app.terminal.link, mtui_ui::TerminalLink::Idle);
    }

    #[tokio::test]
    #[ignore = "hits the live Admin router; not for CI"]
    async fn time_full_ssh_pty_to_admin_router() {
        use std::time::{Duration, Instant};

        use mtui_config::{CredentialStore, PlatformCredentialStore};
        use mtui_ssh::{SshConnectOptions, SshPty};
        use tokio::sync::mpsc;

        async fn once(rows: u32, password: &str) {
            let t0 = Instant::now();
            let pty = SshPty::connect(SshConnectOptions {
                host: "192.168.88.1".into(),
                port: 22,
                username: "admin".into(),
                password: password.to_string(),
                expected_fingerprint: None,
                cols: 80,
                rows,
            })
            .await
            .expect("ssh connect");
            eprintln!(
                "rows={rows} connect {} wall={:?}",
                pty.stages_ms,
                t0.elapsed()
            );
            let (out_tx, mut out_rx) = mpsc::unbounded_channel();
            let (in_tx, in_rx) = mpsc::unbounded_channel();
            let runner = tokio::spawn(async move {
                pty.run(in_rx, out_tx).await;
            });
            let mut acc = Vec::new();
            let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
            loop {
                let left = deadline.saturating_duration_since(tokio::time::Instant::now());
                if let Ok(Some(chunk)) = tokio::time::timeout(left, out_rx.recv()).await {
                    eprintln!(
                        "rows={rows} chunk wall={:?} n={} {:?}",
                        t0.elapsed(),
                        chunk.len(),
                        String::from_utf8_lossy(&chunk)
                    );
                    acc.extend_from_slice(&chunk);
                    let text = String::from_utf8_lossy(&acc);
                    if text.contains('>') || text.contains("MikroTik") {
                        let wall = t0.elapsed();
                        eprintln!("rows={rows} prompt wall={wall:?}");
                        assert!(
                            wall < Duration::from_secs(2),
                            "banner should follow DSR, not the 10s probe timeout, wall={wall:?}"
                        );
                        break;
                    }
                } else {
                    eprintln!(
                        "rows={rows} no prompt wall={:?} acc={:?}",
                        t0.elapsed(),
                        String::from_utf8_lossy(&acc)
                    );
                    break;
                }
            }
            drop(in_tx);
            let _ = runner.await;
        }

        let password = PlatformCredentialStore::discover()
            .expect("credential store")
            .get("Admin")
            .expect("Admin profile password")
            .password;
        once(24, &password).await;
        once(200, &password).await;
    }
}
