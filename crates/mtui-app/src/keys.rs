//! Keyboard handling for [`super::App`].

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use mtui_core::{DASHBOARD_ID, resource_by_id};
use mtui_routeros::Resource;
use mtui_ui::LoginField;

use crate::app::{App, AppCommand, Overlay, Pane, Screen, is_https_router_url};

impl App {
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return vec![AppCommand::Quit];
        }

        match self.screen {
            Screen::Login => self.keys_login(key),
            Screen::Connecting => {
                if key.code == KeyCode::Esc {
                    self.screen = Screen::Login;
                    self.status = "Canceled".into();
                }
                Vec::new()
            }
            Screen::Trust => self.keys_trust(key),
            Screen::Main => self.keys_main(key),
        }
    }

    fn keys_login(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Tab | KeyCode::Down => self.login.focus = self.login.focus.next(),
            KeyCode::BackTab | KeyCode::Up => self.login.focus = self.login.focus.prev(),
            KeyCode::Enter => {
                if self.login.focus != LoginField::Password {
                    self.login.focus = self.login.focus.next();
                    return Vec::new();
                }
                if !is_https_router_url(&self.login.url) {
                    self.login.error = Some("Router must be a valid HTTPS URL".into());
                    self.status = "Router must be a valid HTTPS URL".into();
                    return Vec::new();
                }
                if self.login.username.trim().is_empty() {
                    self.login.error = Some("Username is required".into());
                    self.status = "Username is required".into();
                    return Vec::new();
                }
                self.login.error = None;
                self.pending_password = Some(self.login.password.clone());
                self.screen = Screen::Connecting;
                self.status = "Negotiating secure connection…".into();
                return vec![self.connect_command()];
            }
            KeyCode::Esc => {
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            // Some Windows terminals report Backspace as ASCII BS (0x08) or DEL (0x7f).
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('\u{8}' | '\u{7f}') => {
                self.login.backspace();
            }
            KeyCode::Char('q') if self.login.focus != LoginField::Password => {
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Char(ch) => self.login.insert_char(ch),
            _ => {}
        }
        Vec::new()
    }

    fn keys_trust(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.screen = Screen::Connecting;
                self.status = "Verifying pinned certificate…".into();
                vec![self.connect_command()]
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.trust_fingerprint = None;
                self.screen = Screen::Login;
                self.status = "Certificate was not trusted".into();
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn keys_main(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.overlay != Overlay::None {
            return self.keys_overlay(key);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('k') => {
                    self.overlay = Overlay::Palette;
                    self.palette.open();
                    return Vec::new();
                }
                KeyCode::Char('l') => {
                    self.logout();
                    return vec![AppCommand::ClearSession];
                }
                KeyCode::Char('u') => {
                    self.page_content(-1);
                    return Vec::new();
                }
                KeyCode::Char('d') => {
                    self.page_content(1);
                    return Vec::new();
                }
                KeyCode::Left => {
                    if self.on_table_content() {
                        self.table.scroll_columns_home();
                    }
                    return Vec::new();
                }
                KeyCode::Right => {
                    if self.on_table_content() {
                        self.table.scroll_columns_end();
                    }
                    return Vec::new();
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Char('?') => {
                self.overlay = Overlay::Help;
                self.overlay_scroll = 0;
                return Vec::new();
            }
            KeyCode::Tab => {
                self.pane = match self.pane {
                    Pane::Nav => Pane::Content,
                    Pane::Content => Pane::Inspector,
                    Pane::Inspector => Pane::Nav,
                };
            }
            KeyCode::BackTab => {
                self.pane = match self.pane {
                    Pane::Nav => Pane::Inspector,
                    Pane::Content => Pane::Nav,
                    Pane::Inspector => Pane::Content,
                };
            }
            KeyCode::Char('r') => return self.refresh_now(),
            KeyCode::Char('/') if self.pane == Pane::Content => {
                // simple: clear filter on empty, otherwise append handled below
                self.status = "Filter: type and press Enter — Esc clears".into();
            }
            KeyCode::Char('s') if self.current_resource != "logs" => {
                self.table.cycle_sort();
            }
            KeyCode::Char(' ') if self.current_resource == "logs" => {
                self.log_paused = !self.log_paused;
                self.status = if self.log_paused {
                    "Ⅱ PAUSED".into()
                } else {
                    "● LIVE".into()
                };
            }
            KeyCode::Char('f') if self.current_resource == "logs" => {
                self.log_follow = true;
                self.log_unread = 0;
                self.rebuild_log_table();
            }
            KeyCode::Char('e') if self.current_resource == "logs" => {
                self.log_severity = self.log_severity.cycle();
                self.rebuild_log_table();
                self.status = format!("Severity: {}", self.log_severity.label());
            }
            KeyCode::Char('c') if self.current_resource == "logs" => {
                self.log_buffer.clear();
                self.log_seen.clear();
                self.log_unread = 0;
                self.rebuild_log_table();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_content(-1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_content(1);
            }
            KeyCode::PageUp => self.page_content(-1),
            KeyCode::PageDown => self.page_content(1),
            KeyCode::Home | KeyCode::Char('g') => self.jump_content_home(),
            KeyCode::End | KeyCode::Char('G') => self.jump_content_end(),
            KeyCode::Left | KeyCode::Char('h') if self.on_table_content() => {
                self.table.scroll_columns(-1);
            }
            KeyCode::Right | KeyCode::Char('l') if self.on_table_content() => {
                self.table.scroll_columns(1);
            }
            KeyCode::Enter => {
                if self.pane == Pane::Nav
                    && let Some(id) = self.nav.selected_id().map(str::to_owned)
                    && !id.ends_with("-group")
                {
                    return self.open_resource(&id);
                }
            }
            KeyCode::Char(ch)
                if self.pane == Pane::Content && self.status.starts_with("Filter:") =>
            {
                if ch == '\n' {
                    // ignore
                } else {
                    self.table.filter.push(ch);
                    self.table.set_filter(self.table.filter.clone());
                }
            }
            KeyCode::Esc => {
                if !self.table.filter.is_empty() {
                    self.table.set_filter(String::new());
                    self.status = "Filter cleared".into();
                }
            }
            KeyCode::Backspace if !self.table.filter.is_empty() => {
                self.table.filter.pop();
                self.table.set_filter(self.table.filter.clone());
            }
            _ => {}
        }
        Vec::new()
    }

    fn keys_overlay(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match self.overlay {
            Overlay::Help => self.keys_help(key),
            Overlay::Palette => self.keys_palette(key),
            Overlay::None => Vec::new(),
        }
    }

    fn keys_help(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => {
                self.overlay = Overlay::None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.overlay_scroll = self.overlay_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.overlay_scroll = self.overlay_scroll.saturating_add(1);
            }
            _ => {}
        }
        Vec::new()
    }

    fn keys_palette(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return Vec::new();
        }
        match key.code {
            KeyCode::Esc => {
                self.palette.close();
                self.overlay = Overlay::None;
            }
            KeyCode::Enter => {
                if let Some(id) = self.palette.confirm() {
                    self.overlay = Overlay::None;
                    return self.run_palette_command(&id);
                }
            }
            KeyCode::Up => self.palette.move_selection(-1),
            KeyCode::Down => self.palette.move_selection(1),
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('\u{8}' | '\u{7f}') => {
                self.palette.backspace();
            }
            KeyCode::Char(ch) => self.palette.insert_char(ch),
            _ => {}
        }
        Vec::new()
    }

    fn on_dashboard_content(&self) -> bool {
        self.current_resource == DASHBOARD_ID && self.pane == Pane::Content
    }

    fn on_table_content(&self) -> bool {
        self.current_resource != DASHBOARD_ID && self.pane == Pane::Content
    }

    fn move_content(&mut self, delta: isize) {
        if self.on_dashboard_content() {
            self.scroll_firewall(delta);
        } else {
            self.move_cursor(delta);
        }
    }

    fn page_content(&mut self, direction: isize) {
        if self.on_dashboard_content() {
            let page = isize::try_from(self.firewall_page_size()).unwrap_or(1);
            self.scroll_firewall(direction.saturating_mul(page));
        } else if self.on_table_content() {
            self.table.page_by(direction);
            self.after_table_cursor();
        } else if self.pane == Pane::Inspector {
            let page = isize::try_from(self.inspector_visible_rows()).unwrap_or(1);
            self.inspector.scroll_by(
                direction.saturating_mul(page),
                self.inspector_visible_rows(),
            );
        }
    }

    fn jump_content_home(&mut self) {
        if self.on_dashboard_content() {
            self.scroll_firewall_to(0);
        } else if self.on_table_content() {
            self.table.select_first();
            self.after_table_cursor();
        } else if self.pane == Pane::Inspector {
            self.inspector.offset = 0;
        }
    }

    fn jump_content_end(&mut self) {
        if self.on_dashboard_content() {
            self.scroll_firewall_to(usize::MAX);
        } else if self.on_table_content() {
            self.table.select_last();
            self.after_table_cursor();
        } else if self.pane == Pane::Inspector {
            let visible = self.inspector_visible_rows();
            self.inspector.offset = self.inspector.fields.len().saturating_sub(visible);
        }
    }

    fn logout(&mut self) {
        self.client = None;
        self.router = Resource::default();
        self.screen = Screen::Login;
        self.login.password.clear();
        self.pending_password = None;
        self.trust_fingerprint = None;
        self.status = "Logged out · saved session removed".into();
    }

    fn open_resource(&mut self, id: &str) -> Vec<AppCommand> {
        self.select_resource(id);
        self.poll_current()
    }

    fn refresh_now(&mut self) -> Vec<AppCommand> {
        self.refreshing = true;
        self.poll_current()
    }

    fn move_cursor(&mut self, delta: isize) {
        match self.pane {
            Pane::Nav => self.nav.move_by(delta),
            Pane::Content => {
                self.table.move_selection(delta);
                if self.current_resource == "logs" && delta < 0 {
                    self.log_follow = false;
                }
                self.after_table_cursor();
            }
            Pane::Inspector => self
                .inspector
                .scroll_by(delta, self.inspector_visible_rows()),
        }
    }

    fn after_table_cursor(&mut self) {
        self.refresh_inspector(false);
    }

    fn run_palette_command(&mut self, id: &str) -> Vec<AppCommand> {
        match id {
            "refresh" => self.refresh_now(),
            "logout" => {
                self.logout();
                vec![AppCommand::ClearSession]
            }
            "help" => {
                self.overlay = Overlay::Help;
                self.overlay_scroll = 0;
                Vec::new()
            }
            "dashboard" => self.open_resource(DASHBOARD_ID),
            other => {
                if resource_by_id(other).is_some() {
                    self.open_resource(other)
                } else {
                    Vec::new()
                }
            }
        }
    }
}

#[cfg(test)]
mod login_edit_tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::App;
    use crate::event::AppEvent;
    use mtui_ui::{LoginField, LoginForm};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn login_app() -> App {
        let mut app = App::new(false).expect("app should start on the login screen");
        app.login = LoginForm::default();
        app
    }

    #[test]
    fn backspace_deletes_the_last_character_in_the_focused_field() {
        let mut app = login_app();
        app.login.url = "https://router".into();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Backspace)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.url, "https://route");
    }

    #[test]
    fn delete_deletes_the_last_character_in_the_focused_field() {
        let mut app = login_app();
        app.login.focus = LoginField::Username;
        app.login.username = "admin".into();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Delete)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.username, "admi");
    }

    #[test]
    fn ascii_bs_and_del_chars_act_as_backspace() {
        let mut app = login_app();
        app.login.focus = LoginField::Password;
        app.login.password = "secret".into();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('\u{8}'))));
        assert_eq!(app.login.password, "secre");
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('\u{7f}'))));
        assert_eq!(app.login.password, "secr");
    }

    #[test]
    fn enter_on_password_starts_a_secure_connection() {
        let mut app = login_app();
        app.login.url = "https://192.168.88.1:8443".into();
        app.login.username = "reader".into();
        app.login.password = "secret".into();
        app.login.focus = LoginField::Password;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.screen, crate::app::Screen::Connecting);
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::Connect { url, username, .. }
                    if url == "https://192.168.88.1:8443" && username == "reader"
            )),
            "expected connect command, got {cmds:?}"
        );
    }

    #[test]
    fn enter_on_url_moves_focus_instead_of_submitting() {
        let mut app = login_app();
        app.login.url = "https://192.168.88.1:8443".into();
        app.login.username = "reader".into();
        app.login.focus = LoginField::Url;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.focus, LoginField::Username);
        assert_eq!(app.screen, crate::app::Screen::Login);
    }

    #[test]
    fn dashboard_firewall_scrolls_with_keys() {
        use crate::app::{Pane, Screen};
        use mtui_core::DASHBOARD_ID;
        use mtui_ui::FirewallRuleMetric;

        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.current_resource = DASHBOARD_ID.to_string();
        app.pane = Pane::Content;
        app.terminal_width = 140;
        app.terminal_height = 28;
        app.dash.cpu_core_order = vec!["cpu0".into(), "cpu1".into(), "cpu2".into(), "cpu3".into()];
        app.dash.firewall_rules = (0..14)
            .map(|index| {
                let n = u32::try_from(14 - index).unwrap_or(1);
                FirewallRuleMetric {
                    id: format!("*{index}"),
                    label: format!("rule-{index:02}"),
                    action: "accept".into(),
                    packets: u64::from(n) * 10,
                    bytes: 0,
                    recent_packets: u64::from(n),
                    recent_bytes: 0,
                    history: vec![f64::from(n)],
                }
            })
            .collect();
        assert_eq!(app.dash.firewall_offset, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        assert_eq!(app.dash.firewall_offset, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('k'))));
        assert_eq!(app.dash.firewall_offset, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::End)));
        assert!(app.dash.firewall_offset > 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Home)));
        assert_eq!(app.dash.firewall_offset, 0);
    }
}

#[cfg(test)]
mod table_scroll_tests {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::{App, Pane, Screen};
    use crate::event::AppEvent;
    use mtui_core::resource_by_id;
    use mtui_ui::TableState;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn table_app(width: u16, height: u16) -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.current_resource = "interfaces".into();
        app.pane = Pane::Content;
        let spec = resource_by_id("interfaces").expect("interfaces resource");
        app.table = TableState::new(spec.columns);
        let rows = (0..20)
            .map(|i| {
                let mut row = HashMap::new();
                row.insert("name".into(), format!("iface-{i:02}"));
                row
            })
            .collect();
        app.table.set_rows(rows);
        let _ = app.update(AppEvent::Resize { width, height });
        app
    }

    #[test]
    fn table_end_and_home_scroll_the_row_window() {
        let mut app = table_app(80, 10);
        assert_eq!(app.table.selected, 0);
        assert_eq!(app.table.row_offset, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::End)));
        assert_eq!(app.table.selected, 19);
        assert!(
            app.table.row_offset > 0,
            "end should scroll, offset={}",
            app.table.row_offset
        );

        let _ = app.update(AppEvent::Input(press(KeyCode::Home)));
        assert_eq!(app.table.selected, 0);
        assert_eq!(app.table.row_offset, 0);
    }

    #[test]
    fn table_jk_keeps_selection_inside_the_window() {
        let mut app = table_app(80, 10);
        let page = app.table.page_size();
        assert!(page < 20, "fixture should overflow the pane, page={page}");
        for _ in 0..page {
            let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        }
        assert_eq!(app.table.selected, page);
        assert_eq!(app.table.row_offset, 1);
        assert_eq!(app.table.visible_window(page).first().copied(), Some(1));

        for _ in 0..page {
            let _ = app.update(AppEvent::Input(press(KeyCode::Char('k'))));
        }
        assert_eq!(app.table.selected, 0);
        assert_eq!(app.table.row_offset, 0);
    }

    #[test]
    fn table_hl_pans_columns_on_a_narrow_pane() {
        let mut app = table_app(40, 12);
        assert_eq!(app.table.col_offset, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('l'))));
        assert!(
            app.table.col_offset > 0,
            "l should pan columns, offset={}",
            app.table.col_offset
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('h'))));
        assert_eq!(app.table.col_offset, 0);
    }

    #[test]
    fn table_resize_collapses_offsets_when_the_pane_grows() {
        let mut app = table_app(80, 10);
        let _ = app.update(AppEvent::Input(press(KeyCode::End)));
        assert!(app.table.row_offset > 0);
        let _ = app.update(AppEvent::Resize {
            width: 140,
            height: 40,
        });
        assert_eq!(app.table.selected, 19);
        assert_eq!(app.table.row_offset, 0);
        assert_eq!(app.table.col_offset, 0);
    }
}

#[cfg(test)]
mod palette_tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::{App, Overlay, Screen};
    use crate::event::AppEvent;
    use mtui_ui::lines_plain;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press_char(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
    }

    fn main_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app
    }

    #[test]
    fn ctrl_k_opens_command_palette_and_ctrl_p_does_not() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(app.overlay, Overlay::None);
        assert!(!app.palette.visible);

        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(app.overlay, Overlay::Palette);
        assert!(app.palette.visible);
        assert!(app.palette.query.is_empty());
    }

    #[test]
    fn palette_navigates_to_routeros_path() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));
        for ch in "/IP/firewall".chars() {
            let _ = app.update(AppEvent::Input(press_char(ch)));
        }
        let styles = app.styles();
        let view = lines_plain(&app.palette.render_lines(&styles));
        assert!(
            view.contains("/ip/firewall/filter"),
            "palette missing firewall path: {view}"
        );
        assert!(
            !view.contains("/ip/arp"),
            "unrelated /ip path remained visible: {view}"
        );

        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.overlay, Overlay::None);
        assert!(!app.palette.visible);
        assert_eq!(app.current_resource, "firewall-filter");
        assert_eq!(app.nav.selected_id(), Some("firewall-filter"));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::FetchResource { resource_id, .. }
                    if resource_id == "firewall-filter"
            )),
            "expected firewall resource load, got {cmds:?}"
        );
    }

    #[test]
    fn palette_types_j_and_k_instead_of_scrolling() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));
        let _ = app.update(AppEvent::Input(press_char('j')));
        let _ = app.update(AppEvent::Input(press_char('k')));
        assert_eq!(app.palette.query, "jk");
        assert!(app.palette.matches().is_empty());
    }

    #[test]
    fn palette_help_command_opens_help_overlay() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));
        for ch in "Keyboard help".chars() {
            let _ = app.update(AppEvent::Input(press_char(ch)));
        }
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.overlay, Overlay::Help);
        assert!(!app.palette.visible);
    }
}
