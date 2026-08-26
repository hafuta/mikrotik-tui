//! Keyboard handling for [`super::App`].

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use mtui_core::{DASHBOARD_ID, about_copy, resource_by_id};
use mtui_ui::{FormSession, LoginField, LoginPane};

use crate::app::{App, AppCommand, Overlay, Pane, Screen};
use crate::session::SessionId;

impl App {
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if self.screen == Screen::Login {
                self.persist_login_draft();
            }
            if let Some(cmds) =
                self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::Quit)
            {
                return cmds;
            }
            self.should_quit = true;
            return vec![AppCommand::Quit];
        }

        if let Some(cmds) = self.keys_session_chrome(key) {
            return cmds;
        }

        match self.screen {
            Screen::Login => {
                if self.overlay != Overlay::None {
                    return self.keys_overlay(key);
                }
                self.keys_login(key)
            }
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

    fn keys_session_chrome(&mut self, key: KeyEvent) -> Option<Vec<AppCommand>> {
        if !key.modifiers.contains(KeyModifiers::CONTROL) {
            return None;
        }
        match key.code {
            KeyCode::Char('t') => {
                let _ = self.new_session();
                Some(Vec::new())
            }
            KeyCode::Char('w') => {
                if self.sessions.len() <= 1 {
                    return Some(Vec::new());
                }
                if let Some(cmds) =
                    self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::CloseTab)
                {
                    return Some(cmds);
                }
                let id = self.active;
                self.close_session(id);
                Some(vec![AppCommand::CloseSession { session: id }])
            }
            KeyCode::Tab | KeyCode::PageDown if !key.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(self.cycle_session(1))
            }
            KeyCode::BackTab | KeyCode::PageUp => Some(self.cycle_session(-1)),
            _ => None,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn keys_login(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('l') {
            return Vec::new();
        }
        match key.code {
            KeyCode::Tab => self.login.tab_forward(),
            KeyCode::BackTab => self.login.tab_back(),
            KeyCode::Down | KeyCode::Char('j') if self.login.pane == LoginPane::List => {
                self.login.move_profile(1);
            }
            KeyCode::Up | KeyCode::Char('k') if self.login.pane == LoginPane::List => {
                self.login.move_profile(-1);
            }
            KeyCode::Down if self.login.pane == LoginPane::Form => {
                self.login.focus = self.login.focus.next();
            }
            KeyCode::Up if self.login.pane == LoginPane::Form => {
                self.login.focus = self.login.focus.prev();
            }
            KeyCode::Right | KeyCode::Char('e') if self.login.pane == LoginPane::List => {
                self.open_selected_profile_form();
            }
            KeyCode::Left
                if self.login.pane == LoginPane::Form
                    && self.login.focus == LoginField::Remember =>
            {
                self.login.set_remember(false);
            }
            KeyCode::Right
                if self.login.pane == LoginPane::Form
                    && self.login.focus == LoginField::Remember =>
            {
                self.login.set_remember(true);
            }
            KeyCode::Left
                if self.login.pane == LoginPane::Form && self.login.focus == LoginField::Tls =>
            {
                self.login.set_tls(false);
            }
            KeyCode::Right
                if self.login.pane == LoginPane::Form && self.login.focus == LoginField::Tls =>
            {
                self.login.set_tls(true);
            }
            KeyCode::Left
                if self.login.pane == LoginPane::Form && !self.login.profiles.is_empty() =>
            {
                self.login.pane = LoginPane::List;
            }
            KeyCode::Enter => return self.login_enter(),
            KeyCode::Esc => {
                if self.login.pane == LoginPane::Form && !self.login.profiles.is_empty() {
                    self.login.pane = LoginPane::List;
                    return Vec::new();
                }
                self.persist_login_draft();
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('\u{8}' | '\u{7f}') => {
                if self.login.pane == LoginPane::Form {
                    self.login.backspace();
                }
            }
            KeyCode::Char(' ')
                if self.login.pane == LoginPane::Form
                    && self.login.focus == LoginField::Remember =>
            {
                self.login.toggle_remember();
            }
            KeyCode::Char(' ')
                if self.login.pane == LoginPane::Form && self.login.focus == LoginField::Tls =>
            {
                self.login.toggle_tls();
            }
            KeyCode::Char('n') if self.login.pane == LoginPane::List => self.start_new_profile(),
            KeyCode::Char('x' | 'd') if self.login.pane == LoginPane::List => {
                if let Some(row) = self.login.selected_row().cloned() {
                    if crate::demo::is_demo_target(&row.url)
                        || row
                            .name
                            .eq_ignore_ascii_case(crate::demo::DEMO_PROFILE_NAME)
                    {
                        self.status = "Demo profile cannot be forgotten".into();
                    } else {
                        self.overlay = Overlay::ForgetProfile { name: row.name };
                    }
                }
            }
            KeyCode::Char('q')
                if self.login.pane == LoginPane::List || !self.login.focus.is_secret() =>
            {
                self.persist_login_draft();
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Char(ch) if self.login.pane == LoginPane::Form => {
                if ch == ' ' && self.login.focus == LoginField::Remember {
                    self.login.toggle_remember();
                } else if ch == ' ' && self.login.focus == LoginField::Tls {
                    self.login.toggle_tls();
                } else {
                    self.login.insert_char(ch);
                }
            }
            _ => {}
        }
        Vec::new()
    }

    fn login_enter(&mut self) -> Vec<AppCommand> {
        if self.login.pane == LoginPane::List {
            self.open_selected_profile_form();
            return Vec::new();
        }
        match self.login.focus {
            LoginField::Connect => return self.begin_connect(),
            LoginField::Password if self.login.uses_totp && self.login.totp.is_empty() => {
                self.login.focus = LoginField::Totp;
            }
            LoginField::CaFile => return self.open_ca_file_picker(),
            LoginField::Password | LoginField::Totp | LoginField::Tls | LoginField::Remember => {
                self.login.focus = LoginField::Connect;
            }
            LoginField::Name | LoginField::Url | LoginField::Username => {
                self.login.focus = self.login.focus.next();
            }
        }
        Vec::new()
    }

    fn start_new_profile(&mut self) {
        self.current_profile.clear();
        self.login.name.clear();
        self.login.url.clear();
        self.login.username.clear();
        self.login.password.clear();
        self.login.totp.clear();
        self.login.remember_password = true;
        self.login.uses_totp = false;
        self.login.use_tls = true;
        self.login.ca_file.clear();
        self.login.error = None;
        self.login.pane = LoginPane::Form;
        self.login.focus = LoginField::Name;
        self.saved_fingerprint = None;
        self.saved_url = None;
        self.custom_ca = None;
        self.status = "New device · name the router, then connect".into();
    }

    fn apply_selected_profile(&mut self) {
        let Some(row) = self.login.selected_row().cloned() else {
            return;
        };
        if crate::demo::is_demo_target(&row.url)
            || row
                .name
                .eq_ignore_ascii_case(crate::demo::DEMO_PROFILE_NAME)
        {
            self.login.name = crate::demo::DEMO_PROFILE_NAME.into();
            self.login.url = crate::demo::DEMO_URL.into();
            self.login.username = "demo".into();
            self.login.password.clear();
            self.login.totp.clear();
            self.login.uses_totp = false;
            self.login.use_tls = true;
            self.login.ca_file.clear();
            self.login.remember_password = false;
            self.current_profile = crate::demo::DEMO_PROFILE_NAME.into();
            return;
        }
        if let Some(profile) = self
            .profiles
            .load()
            .ok()
            .into_iter()
            .flatten()
            .find(|item| item.name == row.name)
        {
            self.apply_profile(&profile, true);
        } else {
            self.login.apply_row(&row);
            self.current_profile.clone_from(&row.name);
        }
    }

    fn open_selected_profile_form(&mut self) {
        self.apply_selected_profile();
        self.login.pane = LoginPane::Form;
        self.login.error = None;
        self.login.focus = if crate::demo::is_demo_target(&self.login.url) {
            LoginField::Connect
        } else {
            self.login.open_focus()
        };
        self.status = format!("Opened {}", self.profile_label());
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
        if self.page_form.is_some()
            && (self.pane == Pane::Content
                || self
                    .page_form
                    .as_ref()
                    .is_some_and(|session| session.lookup_open() || session.confirm_save))
        {
            return self.keys_form(key);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('k') => {
                    self.overlay = Overlay::Palette;
                    self.palette.open();
                    tracing::trace!(overlay = "palette", "opened pane");
                    return Vec::new();
                }
                KeyCode::Char('l') => {
                    if let Some(cmds) =
                        self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::Logout)
                    {
                        return cmds;
                    }
                    tracing::info!("logout");
                    self.disconnect_to_profiles();
                    return Vec::new();
                }
                KeyCode::Char('u') => {
                    if self.pane == Pane::Console {
                        let page = self.console_body_height();
                        let len = self.console.filtered_indices(&self.console_entries).len();
                        self.console.page_by(-1, page, len);
                        self.sync_console_viewport();
                        return Vec::new();
                    }
                    return self.page_content(-1);
                }
                KeyCode::Char('d') => {
                    if self.pane == Pane::Console {
                        let page = self.console_body_height();
                        let len = self.console.filtered_indices(&self.console_entries).len();
                        self.console.page_by(1, page, len);
                        self.sync_console_viewport();
                        return Vec::new();
                    }
                    return self.page_content(1);
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

        if self.pane == Pane::Console {
            if matches!(key.code, KeyCode::Char('`')) && !self.console.searching {
                self.toggle_console();
                return Vec::new();
            }
            return self.keys_console(key);
        }

        match key.code {
            KeyCode::Char('`') if !self.console.searching => {
                self.toggle_console();
                return Vec::new();
            }
            KeyCode::Char('q') => {
                if let Some(cmds) =
                    self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::Quit)
                {
                    return cmds;
                }
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Char('?') => {
                self.open_help();
                return Vec::new();
            }
            KeyCode::Char('i') if !self.status.starts_with("Filter:") => {
                self.open_about();
                return Vec::new();
            }
            KeyCode::F(1) => {
                self.open_about();
                return Vec::new();
            }
            KeyCode::F(4) if !self.status.starts_with("Filter:") => {
                return self.toggle_safe_mode();
            }
            KeyCode::Char('.') if !self.status.starts_with("Filter:") => {
                self.toggle_show_hidden_menus();
                return Vec::new();
            }
            KeyCode::Char('-') if self.pane == Pane::Nav && !self.status.starts_with("Filter:") => {
                self.toggle_selected_nav_hidden();
                return Vec::new();
            }
            KeyCode::Tab => self.cycle_pane(true),
            KeyCode::BackTab => self.cycle_pane(false),
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
                    self.rebuild_log_table();
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
                return self.move_content(-1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                return self.move_content(1);
            }
            KeyCode::PageUp => return self.page_content(-1),
            KeyCode::PageDown => return self.page_content(1),
            KeyCode::Home => return self.jump_content_home(),
            KeyCode::Char('g') if !self.action_key_consumed('g') => {
                return self.jump_content_home();
            }
            KeyCode::End | KeyCode::Char('G') => return self.jump_content_end(),
            KeyCode::Char('h') if self.on_table_content() => {
                self.table.scroll_columns(-1);
            }
            KeyCode::Char('l') if self.on_table_content() => {
                self.table.scroll_columns(1);
            }
            KeyCode::Left => return self.arrow_horizontal(-1),
            KeyCode::Right => return self.arrow_horizontal(1),
            KeyCode::Char('y')
                if (self.pane == Pane::Content || self.pane == Pane::Inspector)
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:") =>
            {
                return self.copy_current_view();
            }
            KeyCode::Char('Y')
                if self.pane == Pane::Content
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:") =>
            {
                return self.copy_filtered_table();
            }
            KeyCode::Char(' ')
                if self.pane == Pane::Content
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:")
                    && mtui_core::supports_bulk_select(&self.current_resource) =>
            {
                self.table.toggle_checked();
                self.status = format!("{} selected", self.table.checked_count());
            }
            KeyCode::Char('*')
                if self.pane == Pane::Content
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:")
                    && mtui_core::supports_bulk_select(&self.current_resource) =>
            {
                self.table.check_all_filtered();
                self.status = format!("{} selected", self.table.checked_count());
            }
            KeyCode::Enter => {
                if self.pane == Pane::Nav
                    && let Some(id) = self.nav.selected_id().map(str::to_owned)
                    && self.nav.select_id(&id)
                    && let Some(open_id) = self.nav.selected_id().map(str::to_owned)
                {
                    return self.open_resource(&open_id);
                }
                if self.pane == Pane::Content
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:")
                {
                    return self.dispatch_enter_action();
                }
            }
            KeyCode::Char(ch)
                if self.pane != Pane::Console
                    && self.current_resource != "logs"
                    && !self.status.starts_with("Filter:")
                    && key.modifiers.is_empty()
                    && self.action_key_consumed(ch) =>
            {
                return self.dispatch_key_action(ch);
            }
            KeyCode::Char(ch)
                if self.pane == Pane::Content && self.status.starts_with("Filter:") =>
            {
                if ch == '\n' {
                    // ignore
                } else {
                    self.table.filter.push(ch);
                    let filter = self.table.filter.clone();
                    self.table.set_filter(filter);
                }
            }
            KeyCode::Esc => {
                if !self.table.filter.is_empty() {
                    self.table.set_filter(String::new());
                    self.status = "Filter cleared".into();
                } else if self.table.checked_count() > 0 {
                    self.table.clear_checked();
                    self.status = "Selection cleared".into();
                }
            }
            KeyCode::Backspace if !self.table.filter.is_empty() => {
                self.table.filter.pop();
                let filter = self.table.filter.clone();
                self.table.set_filter(filter);
            }
            _ => {}
        }
        Vec::new()
    }

    fn keys_overlay(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match self.overlay {
            Overlay::Help | Overlay::About => self.keys_help(key),
            Overlay::Palette => self.keys_palette(key),
            Overlay::Confirm(_) | Overlay::HideMenu { .. } | Overlay::ForgetProfile { .. } => {
                self.keys_confirm(key)
            }
            Overlay::SafeModeConflict { .. } | Overlay::SafeModeLeave { .. } => {
                self.keys_safe_mode_overlay(key)
            }
            Overlay::Reauth => self.keys_reauth(key),
            Overlay::Form(_) => self.keys_form(key),
            Overlay::ActionMenu(_) => self.keys_action_menu(key, false),
            Overlay::TypePicker(_) => self.keys_action_menu(key, true),
            Overlay::Torch(_) => self.keys_torch(key),
            Overlay::Probe(_) => self.keys_probe(key),
            Overlay::FilePicker(_) => self.keys_file_picker(key),
            Overlay::None => Vec::new(),
        }
    }

    fn keys_file_picker(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = Overlay::None;
                self.status = "CA file browse canceled".into();
                Vec::new()
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.move_selection(-1);
                }
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.move_selection(1);
                }
                Vec::new()
            }
            KeyCode::Home => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.jump_home();
                }
                Vec::new()
            }
            KeyCode::End => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.jump_end();
                }
                Vec::new()
            }
            KeyCode::PageUp => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.move_selection(-8);
                }
                Vec::new()
            }
            KeyCode::PageDown => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.move_selection(8);
                }
                Vec::new()
            }
            KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => self.file_picker_parent(),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => self.file_picker_open(),
            _ => Vec::new(),
        }
    }

    fn file_picker_parent(&mut self) -> Vec<AppCommand> {
        let Overlay::FilePicker(picker) = &self.overlay else {
            return Vec::new();
        };
        match crate::files_io::parent_browse_dir(&picker.dir) {
            Some(parent) => self.list_picker_dir(parent),
            None => Vec::new(),
        }
    }

    fn file_picker_open(&mut self) -> Vec<AppCommand> {
        let Overlay::FilePicker(picker) = &self.overlay else {
            return Vec::new();
        };
        let Some(entry) = picker.selected_entry().cloned() else {
            return Vec::new();
        };
        if entry.is_dir {
            return self.list_picker_dir(entry.path);
        }
        self.login.ca_file = entry.path;
        self.login.focus = LoginField::CaFile;
        self.overlay = Overlay::None;
        self.status = "CA file selected".into();
        Vec::new()
    }

    #[allow(clippy::too_many_lines)]
    fn keys_console(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if self.console.searching {
            return self.keys_console_search(key);
        }

        let filtered_len = self.console.filtered_indices(&self.console_entries).len();
        match key.code {
            KeyCode::Char('q') => {
                if let Some(cmds) =
                    self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::Quit)
                {
                    return cmds;
                }
                self.should_quit = true;
                return vec![AppCommand::Quit];
            }
            KeyCode::Char('?') => {
                self.open_help();
                return Vec::new();
            }
            KeyCode::Char('i') | KeyCode::F(1) => {
                self.open_about();
                return Vec::new();
            }
            KeyCode::Tab => self.cycle_pane(true),
            KeyCode::BackTab => self.cycle_pane(false),
            KeyCode::Char('r') => return self.refresh_now(),
            KeyCode::Char('f') => {
                self.console.toggle_fullscreen();
                tracing::trace!(fullscreen = self.console.fullscreen, "console fullscreen");
                self.sync_table_viewport();
                self.status = if self.console.fullscreen {
                    "Console fullscreen".into()
                } else {
                    "Console docked".into()
                };
            }
            KeyCode::Char('/') => {
                self.console.start_search();
                self.status =
                    "Console search (case-insensitive) · Enter confirm · Esc cancel".into();
            }
            KeyCode::Char('n') => {
                self.console.jump_match(true, filtered_len);
                self.sync_console_viewport();
            }
            KeyCode::Char('N') => {
                self.console.jump_match(false, filtered_len);
                self.sync_console_viewport();
            }
            KeyCode::Char('c') => {
                if let Some(entry) = self.console.selected_entry(&self.console_entries) {
                    return vec![AppCommand::CopyToClipboard {
                        session: SessionId::UNSTAMPED,
                        text: entry.copy_text(),
                    }];
                }
            }
            KeyCode::Enter => {
                self.with_active(|session| {
                    session
                        .console
                        .activate(&session.console_entries, filtered_len);
                });
                self.sync_console_viewport();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.with_active(|session| {
                    session.console.enter_detail(&session.console_entries);
                });
                self.sync_console_viewport();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.with_active(|session| {
                    session.console.leave_detail(&session.console_entries);
                });
                self.sync_console_viewport();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.with_active(|session| {
                    session
                        .console
                        .move_cursor(-1, &session.console_entries, filtered_len);
                });
                self.sync_console_viewport();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.with_active(|session| {
                    session
                        .console
                        .move_cursor(1, &session.console_entries, filtered_len);
                });
                self.sync_console_viewport();
            }
            KeyCode::PageUp => {
                let height = self.console_body_height();
                self.with_active(|session| {
                    session.console.page_by(-1, height, filtered_len);
                });
                self.sync_console_viewport();
            }
            KeyCode::PageDown => {
                let height = self.console_body_height();
                self.with_active(|session| {
                    session.console.page_by(1, height, filtered_len);
                });
                self.sync_console_viewport();
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.console.select_first();
                self.sync_console_viewport();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.console.select_last(filtered_len);
                self.sync_console_viewport();
            }
            KeyCode::Esc => {
                if self.console.escape_search() {
                    self.with_active(|session| {
                        let len = session
                            .console
                            .filtered_indices(&session.console_entries)
                            .len();
                        session.console.clamp_selection(len);
                    });
                    self.sync_console_viewport();
                    self.status = "Console search cleared".into();
                } else {
                    self.toggle_console();
                }
            }
            _ => {}
        }
        Vec::new()
    }

    fn keys_console_search(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.console.escape_search();
                let len = self.console.filtered_indices(&self.console_entries).len();
                self.console.clamp_selection(len);
                self.sync_console_viewport();
                self.status = "Console search canceled".into();
            }
            KeyCode::Enter => {
                self.console.confirm_search();
                let len = self.console.filtered_indices(&self.console_entries).len();
                self.console.clamp_selection(len);
                self.console.select_first();
                self.sync_console_viewport();
                self.status = if self.console.query.is_empty() {
                    "Console search cleared".into()
                } else {
                    format!("Console search: {}", self.console.query)
                };
            }
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('\u{8}' | '\u{7f}') => {
                self.console.search_backspace();
                let len = self.console.filtered_indices(&self.console_entries).len();
                self.console.clamp_selection(len);
                self.sync_console_viewport();
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                self.console.insert_search_char(ch);
                let len = self.console.filtered_indices(&self.console_entries).len();
                self.console.clamp_selection(len);
                self.sync_console_viewport();
            }
            _ => {}
        }
        Vec::new()
    }

    fn keys_confirm(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('n') => {
                if matches!(
                    &self.overlay,
                    Overlay::Confirm(session)
                        if matches!(session.action_id.as_str(), "reboot" | "shutdown")
                ) {
                    return self.dismiss_lifecycle_confirm();
                }
                self.overlay = Overlay::None;
                Vec::new()
            }
            KeyCode::Enter | KeyCode::Char('y') => {
                if matches!(self.overlay, Overlay::HideMenu { .. }) {
                    self.confirm_hide_menu();
                    Vec::new()
                } else if let Overlay::ForgetProfile { name } = &self.overlay {
                    let name = name.clone();
                    self.overlay = Overlay::None;
                    self.forget_profile(&name);
                    Vec::new()
                } else {
                    self.confirm_pending()
                }
            }
            _ => Vec::new(),
        }
    }

    fn keys_safe_mode_overlay(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match &self.overlay {
            Overlay::SafeModeConflict { .. } => match key.code {
                KeyCode::Char('u') => self.confirm_safe_mode_conflict('u'),
                KeyCode::Char('r') => self.confirm_safe_mode_conflict('r'),
                KeyCode::Char('d' | 'n') | KeyCode::Esc => self.confirm_safe_mode_conflict('d'),
                _ => Vec::new(),
            },
            Overlay::SafeModeLeave { .. } => match key.code {
                KeyCode::Char('u') | KeyCode::Enter => self.confirm_safe_mode_leave(true),
                KeyCode::Char('r') => self.confirm_safe_mode_leave(false),
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.overlay = Overlay::None;
                    self.status = "Still in Safe Mode".into();
                    Vec::new()
                }
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn keys_reauth(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = Overlay::None;
                self.status = "Still connected · credentials not updated".into();
                Vec::new()
            }
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                self.reauth.totp_focus = !self.reauth.totp_focus;
                Vec::new()
            }
            KeyCode::Enter => self.begin_reauth_connect(),
            KeyCode::Backspace | KeyCode::Delete | KeyCode::Char('\u{8}' | '\u{7f}') => {
                if self.reauth.totp_focus {
                    self.reauth.totp.pop();
                } else {
                    self.reauth.password.pop();
                }
                Vec::new()
            }
            KeyCode::Char(ch) if mtui_ui::is_printable_char(ch) => {
                if self.reauth.totp_focus {
                    if ch.is_ascii_digit() && self.reauth.totp.len() < 8 {
                        self.reauth.totp.push(ch);
                    }
                } else {
                    self.reauth.password.push(ch);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn keys_form_confirm(&mut self, key: KeyEvent) -> Option<Vec<AppCommand>> {
        let Some(session) = self.form_session() else {
            return Some(Vec::new());
        };
        if session.confirm_discard {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if matches!(self.overlay, Overlay::Form(_)) {
                        self.overlay = Overlay::None;
                    } else {
                        if let Some(session) = self.form_session_mut() {
                            session.confirm_discard = false;
                        }
                        self.pane = Pane::Nav;
                    }
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    if let Some(session) = self.form_session_mut() {
                        session.confirm_discard = false;
                    }
                }
                _ => {}
            }
            return Some(Vec::new());
        }
        if session.confirm_save {
            if session.save_preview_pending() {
                return Some(Vec::new());
            }
            return Some(match key.code {
                KeyCode::Char('y') | KeyCode::Enter => self.save_form(),
                KeyCode::Char('n') | KeyCode::Esc => {
                    if let Some(session) = self.form_session_mut() {
                        session.close_save_preview();
                    }
                    self.status = "Save canceled".into();
                    Vec::new()
                }
                _ => Vec::new(),
            });
        }
        None
    }

    fn keys_form(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('s')) {
            return self.save_form();
        }
        let Some(session) = self.form_session() else {
            return Vec::new();
        };
        let schema =
            session.overlay_schema(resource_by_id(&session.resource_id).and_then(|spec| spec.form));

        if let Some(cmds) = self.keys_form_confirm(key) {
            return cmds;
        }

        if self.form_session().is_some_and(FormSession::lookup_open) {
            return self.keys_lookup(key, schema);
        }

        match key.code {
            KeyCode::Left | KeyCode::Right
                if matches!(self.overlay, Overlay::None) && self.page_form.is_some() =>
            {
                let delta = if matches!(key.code, KeyCode::Right) {
                    1
                } else {
                    -1
                };
                return self.arrow_horizontal(delta);
            }
            KeyCode::Esc => {
                if matches!(self.overlay, Overlay::Form(_)) {
                    if let Some(session) = self.form_session_mut() {
                        if session.is_dirty() {
                            session.confirm_discard = true;
                        } else {
                            self.overlay = Overlay::None;
                        }
                    }
                } else {
                    self.pane = Pane::Nav;
                }
            }
            KeyCode::Tab => self.tab_form(schema, true),
            KeyCode::BackTab => self.tab_form(schema, false),
            KeyCode::Up => self.with_form(|session| {
                session.move_field(schema, -1);
                session.clamp(schema);
            }),
            KeyCode::Down => self.with_form(|session| {
                session.move_field(schema, 1);
                session.clamp(schema);
            }),
            KeyCode::Char('k') if !self.form_editing_text(schema) => {
                self.with_form(|session| {
                    session.move_field(schema, -1);
                    session.clamp(schema);
                });
            }
            KeyCode::Char('j') if !self.form_editing_text(schema) => {
                self.with_form(|session| {
                    session.move_field(schema, 1);
                    session.clamp(schema);
                });
            }
            KeyCode::Char('-') if !self.form_editing_text(schema) => {
                self.with_form(|session| session.remove_optional(schema));
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.with_form(|session| session.activate(schema));
                return self.lookup_fetch_command();
            }
            KeyCode::Backspace | KeyCode::Delete => {
                self.with_form(|session| session.backspace(schema));
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                self.with_form(|session| session.insert_char(schema, ch));
            }
            _ => {}
        }
        Vec::new()
    }

    fn tab_form(&mut self, schema: &mtui_core::FormSchema, forward: bool) {
        let delta = if forward { 1 } else { -1 };
        let leave_for_panes = matches!(self.overlay, Overlay::None)
            && self.page_form.is_some()
            && self
                .form_session()
                .is_some_and(|session| !session.can_move_field(schema, delta));
        if leave_for_panes {
            self.cycle_pane(forward);
            return;
        }
        self.with_form(|session| {
            session.move_field(schema, delta);
            session.clamp(schema);
        });
    }

    fn keys_lookup(&mut self, key: KeyEvent, schema: &mtui_core::FormSchema) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => self.with_form(FormSession::close_lookup),
            KeyCode::Up => self.with_form(|session| session.lookup_move(-1)),
            KeyCode::Down => self.with_form(|session| session.lookup_move(1)),
            KeyCode::Enter => self.with_form(|session| {
                session.lookup_confirm();
                session.clamp(schema);
            }),
            KeyCode::Char(' ') => self.with_form(|session| {
                if session
                    .lookup
                    .as_ref()
                    .is_some_and(|picker| picker.multiple)
                {
                    session.lookup_toggle_focused();
                } else {
                    session.lookup_confirm();
                    session.clamp(schema);
                }
            }),
            KeyCode::Backspace | KeyCode::Delete => {
                self.with_form(FormSession::lookup_backspace);
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                self.with_form(|session| session.lookup_insert_char(ch));
            }
            _ => {}
        }
        Vec::new()
    }

    fn lookup_fetch_command(&mut self) -> Vec<AppCommand> {
        let Some(session) = self.form_session() else {
            return Vec::new();
        };
        let Some(picker) = &session.lookup else {
            return Vec::new();
        };
        if picker.resource_id.is_empty() || !picker.loading || picker.request_id != 0 {
            return Vec::new();
        }
        let resource_id = picker.resource_id.to_string();
        let value_key = picker.value_key.to_string();
        let generation = picker.generation;
        let request_id = self.next_request();
        if let Some(session) = self.form_session_mut()
            && let Some(picker) = &mut session.lookup
        {
            picker.request_id = request_id;
        }
        vec![AppCommand::FetchLookup {
            session: SessionId::UNSTAMPED,
            request_id,
            generation,
            resource_id,
            value_key,
        }]
    }

    fn with_form(&mut self, f: impl FnOnce(&mut FormSession)) {
        if let Some(session) = self.form_session_mut() {
            f(session);
        }
    }

    fn form_editing_text(&self, schema: &mtui_core::FormSchema) -> bool {
        self.form_session()
            .is_some_and(|session| session.focused_takes_typed_input(schema))
    }

    fn keys_action_menu(&mut self, key: KeyEvent, type_picker: bool) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = Overlay::None;
                Vec::new()
            }
            KeyCode::Enter => {
                let id = match &self.overlay {
                    Overlay::ActionMenu(menu) | Overlay::TypePicker(menu) => menu.confirm(),
                    _ => None,
                };
                self.overlay = Overlay::None;
                if type_picker {
                    if let Some(id) = id {
                        return self.open_create(&id);
                    }
                    Vec::new()
                } else if let Some(id) = id {
                    self.dispatch_named_action(&id)
                } else {
                    Vec::new()
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(menu) = self.menu_mut() {
                    menu.move_selection(-1);
                }
                Vec::new()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(menu) = self.menu_mut() {
                    menu.move_selection(1);
                }
                Vec::new()
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if let Some(menu) = self.menu_mut() {
                    menu.backspace();
                }
                Vec::new()
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                if let Some(menu) = self.menu_mut() {
                    menu.insert_char(ch);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn menu_mut(&mut self) -> Option<&mut mtui_ui::ActionMenuState> {
        match &mut self.overlay {
            Overlay::ActionMenu(menu) | Overlay::TypePicker(menu) => Some(menu),
            _ => None,
        }
    }

    fn keys_torch(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.torch_generation = self.torch_generation.wrapping_add(1);
                self.overlay = Overlay::None;
                Vec::new()
            }
            KeyCode::Char(' ') => {
                if matches!(&self.overlay, Overlay::Torch(_)) {
                    self.torch_generation = self.torch_generation.wrapping_add(1);
                    let generation = self.torch_generation;
                    let Overlay::Torch(torch) = &mut self.overlay else {
                        return Vec::new();
                    };
                    torch.running = !torch.running;
                    torch.error = None;
                    torch.generation = generation;
                    if torch.running {
                        return self.torch_sample_command();
                    }
                }
                Vec::new()
            }
            KeyCode::Tab => {
                if let Overlay::Torch(torch) = &mut self.overlay {
                    torch.focus = torch.focus.next();
                }
                Vec::new()
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if let Overlay::Torch(torch) = &mut self.overlay {
                    torch.backspace();
                }
                Vec::new()
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                if let Overlay::Torch(torch) = &mut self.overlay {
                    torch.insert_char(ch);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn keys_probe(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.probe_generation = self.probe_generation.wrapping_add(1);
                self.overlay = Overlay::None;
                Vec::new()
            }
            KeyCode::Enter => self.start_probe(),
            KeyCode::Tab => {
                if let Overlay::Probe(probe) = &mut self.overlay {
                    probe.cycle_focus();
                }
                Vec::new()
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if let Overlay::Probe(probe) = &mut self.overlay {
                    probe.backspace();
                }
                Vec::new()
            }
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                if let Overlay::Probe(probe) = &mut self.overlay {
                    probe.insert_char(ch);
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn keys_help(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = Overlay::None;
            }
            KeyCode::Char('?') => {
                if self.overlay == Overlay::Help {
                    self.overlay = Overlay::None;
                } else {
                    self.open_help();
                }
            }
            KeyCode::Char('i') | KeyCode::F(1) => {
                if self.overlay == Overlay::About {
                    self.overlay = Overlay::None;
                } else {
                    self.open_about();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.overlay_scroll = self.overlay_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = crate::render::overlay_scroll_max(self);
                self.overlay_scroll = self.overlay_scroll.saturating_add(1).min(max);
            }
            _ => {}
        }
        Vec::new()
    }

    fn open_help(&mut self) {
        self.overlay = Overlay::Help;
        self.overlay_scroll = 0;
        tracing::trace!(overlay = "help", "opened pane");
    }

    fn open_about(&mut self) {
        if about_copy(&self.current_resource).is_none() {
            return;
        }
        self.overlay = Overlay::About;
        self.overlay_scroll = 0;
        tracing::trace!(
            overlay = "about",
            resource = self.current_resource.as_str(),
            "opened pane"
        );
    }

    pub(crate) fn clamp_overlay_scroll(&mut self) {
        let max = crate::render::overlay_scroll_max(self);
        if self.overlay_scroll > max {
            self.overlay_scroll = max;
        }
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

    fn arrow_horizontal(&mut self, delta: isize) -> Vec<AppCommand> {
        if self.page_form.is_none()
            && self.on_table_content()
            && self.table.can_scroll_columns(delta)
        {
            self.table.scroll_columns(delta);
            return Vec::new();
        }
        let cmds = if delta > 0 && self.pane == Pane::Nav {
            self.apply_focused_nav()
        } else {
            Vec::new()
        };
        self.shift_main_pane(delta > 0);
        cmds
    }

    /// Open the highlighted nav row if it is not the current screen.
    fn apply_focused_nav(&mut self) -> Vec<AppCommand> {
        let Some(id) = self.nav.selected_id().map(str::to_owned) else {
            return Vec::new();
        };
        if !self.nav.select_id(&id) {
            return Vec::new();
        }
        let Some(open_id) = self.nav.selected_id().map(str::to_owned) else {
            return Vec::new();
        };
        if open_id == self.current_resource {
            return Vec::new();
        }
        self.open_resource(&open_id)
    }

    fn move_content(&mut self, delta: isize) -> Vec<AppCommand> {
        if self.on_dashboard_content() {
            self.scroll_firewall(delta);
            Vec::new()
        } else {
            self.move_cursor(delta)
        }
    }

    fn page_content(&mut self, direction: isize) -> Vec<AppCommand> {
        if self.on_dashboard_content() {
            let page = isize::try_from(self.firewall_page_size()).unwrap_or(1);
            self.scroll_firewall(direction.saturating_mul(page));
            Vec::new()
        } else if self.on_table_content() {
            self.table.page_by(direction);
            self.pause_log_follow_if_leaving_newest(direction);
            self.after_table_cursor()
        } else if self.pane == Pane::Nav {
            self.nav.page_by(direction);
            Vec::new()
        } else if self.pane == Pane::Inspector {
            let visible = self.inspector_visible_rows();
            let page = isize::try_from(visible).unwrap_or(1);
            self.inspector
                .move_selection(direction.saturating_mul(page), visible);
            Vec::new()
        } else {
            Vec::new()
        }
    }

    fn jump_content_home(&mut self) -> Vec<AppCommand> {
        if self.on_dashboard_content() {
            self.scroll_firewall_to(0);
            Vec::new()
        } else if self.on_table_content() {
            self.table.select_first();
            self.after_table_cursor()
        } else if self.pane == Pane::Nav {
            self.nav.select_first();
            Vec::new()
        } else if self.pane == Pane::Inspector {
            self.inspector.select_first();
            Vec::new()
        } else {
            Vec::new()
        }
    }

    fn jump_content_end(&mut self) -> Vec<AppCommand> {
        if self.on_dashboard_content() {
            self.scroll_firewall_to(usize::MAX);
            Vec::new()
        } else if self.on_table_content() {
            self.table.select_last();
            self.pause_log_follow_if_leaving_newest(1);
            self.after_table_cursor()
        } else if self.pane == Pane::Nav {
            self.nav.select_last();
            Vec::new()
        } else if self.pane == Pane::Inspector {
            let visible = self.inspector_visible_rows();
            self.inspector.select_last(visible);
            Vec::new()
        } else {
            Vec::new()
        }
    }

    fn logout(&mut self) {
        tracing::info!("logout");
        self.disconnect_to_profiles();
    }

    pub(crate) fn open_resource(&mut self, id: &str) -> Vec<AppCommand> {
        self.select_resource(id);
        self.poll_current()
    }

    fn refresh_now(&mut self) -> Vec<AppCommand> {
        if !self.session_ready() {
            return self.try_reconnect();
        }
        self.refreshing = true;
        self.poll_current()
    }

    fn move_cursor(&mut self, delta: isize) -> Vec<AppCommand> {
        match self.pane {
            Pane::Nav => {
                self.nav.move_by(delta);
                Vec::new()
            }
            Pane::Content => {
                self.table.move_selection(delta);
                self.pause_log_follow_if_leaving_newest(delta);
                self.after_table_cursor()
            }
            Pane::Inspector => {
                let visible = self.inspector_visible_rows();
                self.inspector.move_selection(delta, visible);
                Vec::new()
            }
            Pane::Console => {
                let id = self.active;
                let session = self.session_mut(id).expect("active session must exist");
                let len = session
                    .console
                    .filtered_indices(&session.console_entries)
                    .len();
                session.console.move_selection(delta, len);
                self.sync_console_viewport();
                Vec::new()
            }
        }
    }

    fn pause_log_follow_if_leaving_newest(&mut self, delta: isize) {
        if self.current_resource == "logs" && delta > 0 {
            self.log_follow = false;
        }
    }

    fn after_table_cursor(&mut self) -> Vec<AppCommand> {
        self.refresh_inspector(false);
        self.hydrate_selected_typed_interface()
    }

    fn run_palette_command(&mut self, id: &str) -> Vec<AppCommand> {
        match id {
            "refresh" => self.refresh_now(),
            "logout" | "switch-device" => {
                if let Some(cmds) =
                    self.request_leave_with_safe_mode(crate::safe_mode::SafeModeAfter::Logout)
                {
                    return cmds;
                }
                self.logout();
                Vec::new()
            }
            "forget-device" => {
                let name = if self.current_profile.is_empty() {
                    self.login.name.clone()
                } else {
                    self.current_profile.clone()
                };
                if name.is_empty() {
                    self.status = "No device to forget".into();
                    Vec::new()
                } else {
                    self.overlay = Overlay::ForgetProfile { name };
                    Vec::new()
                }
            }
            "help" => {
                self.open_help();
                Vec::new()
            }
            "about" => {
                self.open_about();
                Vec::new()
            }
            "console" => {
                self.toggle_console();
                Vec::new()
            }
            "show-hidden-menus" => {
                self.toggle_show_hidden_menus();
                Vec::new()
            }
            "reset-hidden-menus" => {
                self.reset_hidden_menus();
                Vec::new()
            }
            "dashboard" => self.open_resource(DASHBOARD_ID),
            "safe-mode" => self.toggle_safe_mode(),
            "safe-mode-unroll" => self.unroll_safe_mode(),
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
    use mtui_ui::{LoginField, LoginForm, LoginPane};

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
        app.login.url = "192.168.88.1".into();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Backspace)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.url, "192.168.88.");
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
    fn enter_on_password_moves_to_the_login_button() {
        let mut app = login_app();
        app.login.url = "192.168.88.1:8729".into();
        app.login.username = "reader".into();
        app.login.password = "secret".into();
        app.login.focus = LoginField::Password;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.focus, LoginField::Connect);
        assert_eq!(app.screen, crate::app::Screen::Login);
    }

    #[test]
    fn enter_on_login_button_starts_a_plaintext_api_connection() {
        let mut app = login_app();
        app.login.url = "192.168.88.1".into();
        app.login.username = "reader".into();
        app.login.password = "secret".into();
        app.login.use_tls = false;
        app.login.focus = LoginField::Connect;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.screen, crate::app::Screen::Connecting);
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::Connect { url, use_tls, pin, ca_pem, .. }
                    if url == "192.168.88.1:8728" && !*use_tls && pin.is_none() && ca_pem.is_none()
            )),
            "expected plaintext connect on 8728, got {cmds:?}"
        );
    }

    #[test]
    fn enter_on_login_button_starts_a_secure_connection() {
        let mut app = login_app();
        app.login.url = "192.168.88.1:8729".into();
        app.login.username = "reader".into();
        app.login.password = "secret".into();
        app.login.focus = LoginField::Connect;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.screen, crate::app::Screen::Connecting);
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::Connect { url, username, .. }
                    if url == "192.168.88.1:8729" && username == "reader"
            )),
            "expected connect command, got {cmds:?}"
        );
    }

    #[test]
    fn enter_on_ca_file_opens_a_directory_browser() {
        let mut app = login_app();
        app.login.focus = LoginField::CaFile;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(
            cmds.iter()
                .any(|cmd| matches!(cmd, crate::app::AppCommand::ListLocalDir { .. })),
            "expected list dir, got {cmds:?}"
        );
        assert!(matches!(app.overlay, crate::app::Overlay::FilePicker(_)));
    }

    #[test]
    fn file_picker_selects_a_file_and_ignores_stale_listings() {
        let mut app = login_app();
        app.login.focus = LoginField::CaFile;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let generation = match &app.overlay {
            crate::app::Overlay::FilePicker(picker) => picker.generation,
            other => panic!("expected picker, got {other:?}"),
        };
        let _ = app.update(AppEvent::Worker(
            crate::event::WorkerMsg::ListLocalDirResult {
                session: app.test_session(),
                generation: generation.wrapping_add(1),
                dir: "/stale".into(),
                entries: vec![mtui_ui::FilePickerEntry {
                    name: "old.pem".into(),
                    path: "/stale/old.pem".into(),
                    is_dir: false,
                }],
                error: None,
            },
        ));
        assert!(matches!(
            &app.overlay,
            crate::app::Overlay::FilePicker(picker) if picker.entries.is_empty()
        ));
        let _ = app.update(AppEvent::Worker(
            crate::event::WorkerMsg::ListLocalDirResult {
                session: app.test_session(),
                generation,
                dir: "/certs".into(),
                entries: vec![
                    mtui_ui::FilePickerEntry {
                        name: "issued".into(),
                        path: "/certs/issued".into(),
                        is_dir: true,
                    },
                    mtui_ui::FilePickerEntry {
                        name: "ca.pem".into(),
                        path: "/certs/ca.pem".into(),
                        is_dir: false,
                    },
                ],
                error: None,
            },
        ));
        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.ca_file, "/certs/ca.pem");
        assert_eq!(app.overlay, crate::app::Overlay::None);
    }

    #[test]
    fn file_picker_enter_on_a_directory_requests_another_listing() {
        let mut app = login_app();
        app.login.focus = LoginField::CaFile;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let generation = match &app.overlay {
            crate::app::Overlay::FilePicker(picker) => picker.generation,
            other => panic!("expected picker, got {other:?}"),
        };
        let _ = app.update(AppEvent::Worker(
            crate::event::WorkerMsg::ListLocalDirResult {
                session: app.test_session(),
                generation,
                dir: "/certs".into(),
                entries: vec![mtui_ui::FilePickerEntry {
                    name: "issued".into(),
                    path: "/certs/issued".into(),
                    is_dir: true,
                }],
                error: None,
            },
        ));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::ListLocalDir { path, .. } if path == "/certs/issued"
            )),
            "expected nested list, got {cmds:?}"
        );
        assert!(matches!(app.overlay, crate::app::Overlay::FilePicker(_)));
    }

    #[test]
    fn file_picker_esc_closes_without_changing_the_path() {
        let mut app = login_app();
        app.login.ca_file = "/keep.pem".into();
        app.login.focus = LoginField::CaFile;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Esc)));
        assert!(cmds.is_empty());
        assert_eq!(app.overlay, crate::app::Overlay::None);
        assert_eq!(app.login.ca_file, "/keep.pem");
    }

    #[test]
    fn enter_on_url_moves_focus_instead_of_submitting() {
        let mut app = login_app();
        app.login.url = "192.168.88.1:8729".into();
        app.login.username = "reader".into();
        app.login.focus = LoginField::Url;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.focus, LoginField::Username);
        assert_eq!(app.screen, crate::app::Screen::Login);
    }

    #[test]
    fn totp_is_appended_to_the_password_at_connect_and_not_used_as_the_name() {
        let mut app = login_app();
        app.login.url = "192.168.88.1:8729".into();
        app.login.username = "reader".into();
        app.login.password = "secret".into();
        app.login.totp = "123456".into();
        app.login.focus = LoginField::Connect;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::Connect { password, .. } if password == "secret123456"
            )),
            "expected password+totp, got {cmds:?}"
        );
    }

    #[test]
    fn list_j_k_moves_saved_profiles() {
        let mut app = login_app();
        app.login.profiles = vec![
            mtui_ui::SavedProfileRow {
                name: "alpha".into(),
                url: "10.0.0.1:8729".into(),
                username: "admin".into(),
                remember_password: true,
                uses_totp: false,
                use_tls: true,
                ca_file: String::new(),
            },
            mtui_ui::SavedProfileRow {
                name: "bravo".into(),
                url: "10.0.0.2:8729".into(),
                username: "reader".into(),
                remember_password: false,
                uses_totp: true,
                use_tls: true,
                ca_file: String::new(),
            },
        ];
        app.login.pane = LoginPane::List;
        app.login.selected_profile = 0;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        assert_eq!(app.login.selected_profile, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('k'))));
        assert_eq!(app.login.selected_profile, 0);
    }

    #[test]
    fn tab_on_form_walks_fields_before_the_profile_list() {
        let mut app = login_app();
        app.login.profiles = vec![mtui_ui::SavedProfileRow {
            name: "alpha".into(),
            url: "10.0.0.1:8729".into(),
            username: "admin".into(),
            remember_password: true,
            uses_totp: false,
            use_tls: true,
            ca_file: String::new(),
        }];
        app.login.pane = LoginPane::Form;
        app.login.focus = LoginField::Name;
        let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        assert_eq!(app.login.pane, LoginPane::Form);
        assert_eq!(app.login.focus, LoginField::Url);
        for _ in 0..6 {
            let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        }
        assert_eq!(app.login.focus, LoginField::Remember);
        let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        assert_eq!(app.login.focus, LoginField::Connect);
        let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        assert_eq!(app.login.pane, LoginPane::List);
    }

    #[test]
    fn right_on_list_opens_the_selected_profile_form() {
        let mut app = login_app();
        app.login.profiles = vec![
            mtui_ui::SavedProfileRow {
                name: "alpha".into(),
                url: "10.0.0.1:8729".into(),
                username: "admin".into(),
                remember_password: true,
                uses_totp: false,
                use_tls: true,
                ca_file: String::new(),
            },
            mtui_ui::SavedProfileRow {
                name: "bravo".into(),
                url: "10.0.0.2:8729".into(),
                username: "reader".into(),
                remember_password: false,
                uses_totp: true,
                use_tls: true,
                ca_file: String::new(),
            },
        ];
        app.login.pane = LoginPane::List;
        app.login.selected_profile = 1;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.pane, LoginPane::Form);
        assert_eq!(app.login.name, "bravo");
        assert_eq!(app.login.focus, LoginField::Totp);
        assert_eq!(app.screen, crate::app::Screen::Login);

        app.login.pane = LoginPane::List;
        app.login.selected_profile = 0;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.login.pane, LoginPane::Form);
        assert_eq!(app.login.name, "alpha");
        assert_eq!(app.login.focus, LoginField::Connect);
        assert_eq!(app.screen, crate::app::Screen::Login);
    }

    #[test]
    fn ctrl_l_disconnects_without_a_forget_command() {
        let mut app = login_app();
        app.screen = crate::app::Screen::Main;
        app.current_profile = "core".into();
        let cmds = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::CONTROL,
        )));
        assert!(cmds.is_empty());
        assert_eq!(app.screen, crate::app::Screen::Login);
        assert!(app.status.contains("profiles kept"));
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
        let _ = app.nav.select_id("interfaces");
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
    fn arrows_pan_columns_then_shift_main_panes() {
        let mut app = table_app(140, 24);
        assert_eq!(app.pane, Pane::Content);
        assert_eq!(app.table.col_offset, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Nav);
        assert_eq!(app.table.col_offset, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Nav);

        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.pane, Pane::Content);

        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert!(
            app.table.col_offset > 0,
            "right should pan columns before leaving the table"
        );
        assert_eq!(app.pane, Pane::Content);

        let mut reached_inspector = false;
        for _ in 0..app.table.columns.len() {
            let offset = app.table.col_offset;
            let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
            if app.pane == Pane::Inspector {
                assert_eq!(app.table.col_offset, offset);
                reached_inspector = true;
                break;
            }
            assert!(
                app.table.col_offset > offset,
                "right should keep panning until the last column"
            );
            assert_eq!(app.pane, Pane::Content);
        }
        assert!(reached_inspector, "right should reach the details pane");

        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.pane, Pane::Inspector);

        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Content);
    }

    #[test]
    fn arrows_skip_inspector_when_that_pane_is_hidden() {
        let mut app = table_app(80, 24);
        assert_eq!(app.pane, Pane::Content);
        app.table.scroll_columns_end();
        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.pane, Pane::Content);
        app.table.scroll_columns_home();
        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Nav);
    }

    #[test]
    fn inline_form_left_arrow_focuses_nav_like_a_table() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        let _ = app.update(AppEvent::Resize {
            width: 140,
            height: 24,
        });
        app.select_resource("reset-configuration");
        assert!(app.page_form.is_some());
        assert_eq!(app.pane, Pane::Content);

        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Nav);
        assert!(app.page_form.is_some());

        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        assert_eq!(app.pane, Pane::Nav);

        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.pane, Pane::Content);
        assert_eq!(app.current_resource, "reset-configuration");
    }

    #[test]
    fn inline_form_tab_walks_fields_then_cycles_panes() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        let _ = app.update(AppEvent::Resize {
            width: 140,
            height: 24,
        });
        app.select_resource("reset-configuration");
        assert_eq!(app.pane, Pane::Content);
        let first = app.page_form.as_ref().expect("inline form").focus;

        let _ = app.update(AppEvent::Input(press(KeyCode::BackTab)));
        assert_eq!(app.pane, Pane::Nav);
        assert_eq!(
            app.page_form.as_ref().expect("inline form").focus,
            first,
            "leaving the form must keep the field cursor"
        );

        let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        assert_eq!(app.pane, Pane::Content);

        let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
        assert_eq!(app.pane, Pane::Content);
        assert!(
            app.page_form.as_ref().expect("inline form").focus > first,
            "tab should advance a field before leaving the sheet"
        );

        let mut left_form = false;
        for _ in 0..32 {
            let _ = app.update(AppEvent::Input(press(KeyCode::Tab)));
            if app.pane != Pane::Content {
                left_form = true;
                break;
            }
        }
        assert!(left_form, "tab past the last field should leave the form");
        assert_eq!(app.pane, Pane::Inspector);

        let _ = app.update(AppEvent::Input(press(KeyCode::BackTab)));
        assert_eq!(app.pane, Pane::Content);
        let _ = app.update(AppEvent::Input(press(KeyCode::BackTab)));
        assert_eq!(app.pane, Pane::Content);
        for _ in 0..32 {
            if app.pane != Pane::Content {
                break;
            }
            let _ = app.update(AppEvent::Input(press(KeyCode::BackTab)));
        }
        assert_eq!(app.pane, Pane::Nav);
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

    #[test]
    fn inspector_cursor_clamps_at_first_and_last_field() {
        let mut app = table_app(140, 12);
        let mut fields = HashMap::new();
        for i in 0..12 {
            fields.insert(format!("field-{i:02}"), format!("{i}"));
        }
        app.inspector = mtui_ui::InspectorState::from_row(Some(&fields));
        app.pane = Pane::Inspector;
        assert_eq!(app.inspector.selected, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Up)));
        assert_eq!(app.inspector.selected, 0);
        assert_eq!(app.inspector.offset, 0);

        let last = app.inspector.fields.len() - 1;
        for _ in 0..last.saturating_add(4) {
            let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        }
        assert_eq!(app.inspector.selected, last);
        assert!(
            app.inspector.offset > 0,
            "cursor should scroll the inspector window, offset={}",
            app.inspector.offset
        );

        let _ = app.update(AppEvent::Input(press(KeyCode::Home)));
        assert_eq!(app.inspector.selected, 0);
        assert_eq!(app.inspector.offset, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::End)));
        assert_eq!(app.inspector.selected, last);
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
        assert_eq!(app.nav.expanded.as_deref(), Some("ip-group"));
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

    #[test]
    fn palette_about_command_opens_about_overlay() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::CONTROL,
        )));
        for ch in "About this screen".chars() {
            let _ = app.update(AppEvent::Input(press_char(ch)));
        }
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.overlay, Overlay::About);
        assert!(!app.palette.visible);
    }
}

#[cfg(test)]
mod about_overlay_tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::{App, Overlay, Screen};
    use crate::event::AppEvent;

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
    fn i_opens_about_for_the_current_screen() {
        let mut app = main_app();
        app.select_resource("macsec");
        let _ = app.update(AppEvent::Input(press_char('i')));
        assert_eq!(app.overlay, Overlay::About);
        let copy = mtui_core::about_copy("macsec").expect("macsec about");
        assert!(copy.body.contains("802.1AE"));
        assert!(copy.body.contains("manual.mikrotik.com"));
    }

    #[test]
    fn f1_opens_about_and_esc_closes() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::F(1))));
        assert_eq!(app.overlay, Overlay::About);
        let _ = app.update(AppEvent::Input(press(KeyCode::Esc)));
        assert_eq!(app.overlay, Overlay::None);
    }

    #[test]
    fn i_does_not_open_about_while_filtering() {
        let mut app = main_app();
        app.select_resource("macsec");
        app.pane = crate::app::Pane::Content;
        let _ = app.update(AppEvent::Input(press_char('/')));
        let _ = app.update(AppEvent::Input(press_char('i')));
        assert_eq!(app.overlay, Overlay::None);
        assert!(app.table.filter.contains('i'));
    }

    #[test]
    fn question_mark_from_about_opens_keyboard_help() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press_char('i')));
        assert_eq!(app.overlay, Overlay::About);
        let _ = app.update(AppEvent::Input(press_char('?')));
        assert_eq!(app.overlay, Overlay::Help);
    }

    #[test]
    fn about_j_does_not_scroll_past_the_end() {
        let mut app = main_app();
        app.select_resource("clock");
        let _ = app.update(AppEvent::Input(press_char('i')));
        assert_eq!(app.overlay, Overlay::About);
        let max = crate::render::overlay_scroll_max(&app);
        for _ in 0..80 {
            let _ = app.update(AppEvent::Input(press_char('j')));
        }
        assert_eq!(app.overlay_scroll, max);
    }
}

#[cfg(test)]
mod nav_accordion_tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_core::DASHBOARD_ID;

    use crate::app::{App, Pane, Screen};
    use crate::event::AppEvent;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn main_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.pane = Pane::Nav;
        app.nav.set_hidden_ids(Vec::new());
        app.nav.set_show_hidden(false);
        app
    }

    #[test]
    fn enter_on_category_opens_first_screen_and_collapses_others() {
        let mut app = main_app();
        assert!(app.nav.expanded.is_none());
        assert_eq!(app.nav.selected_id(), Some("dashboard"));

        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        assert_eq!(app.nav.selected_id(), Some("interfaces-group"));

        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.current_resource, "interfaces");
        assert_eq!(app.nav.selected_id(), Some("interfaces"));
        assert_eq!(app.nav.expanded.as_deref(), Some("interfaces-group"));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::FetchResource { resource_id, .. }
                    if resource_id == "interfaces"
            )),
            "expected interfaces resource load, got {cmds:?}"
        );
        assert!(app.nav.entries.iter().any(|entry| entry.id == "ethernet"));

        app.pane = Pane::Nav;
        let ppp = app
            .nav
            .entries
            .iter()
            .position(|entry| entry.id == "ppp-group")
            .expect("ppp group");
        app.nav.selected = ppp;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.current_resource, "ppp-secrets");
        assert_eq!(app.nav.expanded.as_deref(), Some("ppp-group"));
        assert!(app.nav.entries.iter().all(|entry| entry.id != "ethernet"));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::FetchResource { resource_id, .. }
                    if resource_id == "ppp-secrets"
            )),
            "expected PPP secrets load, got {cmds:?}"
        );
    }

    #[test]
    fn right_on_nav_opens_the_focused_item_then_moves_to_content() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Resize {
            width: 140,
            height: 24,
        });
        assert_eq!(app.pane, Pane::Nav);
        assert_eq!(app.current_resource, DASHBOARD_ID);

        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        assert_eq!(app.nav.selected_id(), Some("interfaces-group"));
        assert_eq!(app.current_resource, DASHBOARD_ID);

        let cmds = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.current_resource, "interfaces");
        assert_eq!(app.nav.selected_id(), Some("interfaces"));
        assert_eq!(app.pane, Pane::Content);
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::FetchResource { resource_id, .. }
                    if resource_id == "interfaces"
            )),
            "expected interfaces resource load, got {cmds:?}"
        );

        app.pane = Pane::Nav;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Right)));
        assert_eq!(app.current_resource, "interfaces");
        assert_eq!(app.pane, Pane::Content);
        assert!(
            cmds.is_empty(),
            "right on the already-open item should only move focus, got {cmds:?}"
        );
    }

    #[test]
    fn minus_hides_the_selected_nav_item_and_dot_reveals_it() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        assert_eq!(app.nav.selected_id(), Some("interfaces-group"));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('-'))));
        assert!(
            matches!(app.overlay, crate::app::Overlay::HideMenu { ref id, .. } if id == "interfaces-group"),
            "minus should ask before hiding: {:?}",
            app.overlay
        );
        assert!(!app.nav.hidden.contains("interfaces-group"));

        let _ = app.update(AppEvent::Input(press(KeyCode::Char('n'))));
        assert!(matches!(app.overlay, crate::app::Overlay::None));
        assert!(!app.nav.hidden.contains("interfaces-group"));

        let _ = app.update(AppEvent::Input(press(KeyCode::Char('-'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        assert!(app.nav.hidden.contains("interfaces-group"));
        assert!(
            app.nav
                .entries
                .iter()
                .all(|entry| entry.id != "interfaces-group")
        );
        assert_eq!(app.nav.selected_id(), Some("wireguard-group"));
        assert!(app.status.contains("Hidden"));

        let _ = app.update(AppEvent::Input(press(KeyCode::Char('.'))));
        assert!(app.nav.show_hidden);
        assert!(
            app.nav
                .entries
                .iter()
                .any(|entry| entry.id == "interfaces-group" && entry.hidden)
        );
        assert!(
            app.palette
                .commands
                .iter()
                .any(|cmd| cmd.id == "interfaces")
        );

        app.pane = Pane::Nav;
        let idx = app
            .nav
            .entries
            .iter()
            .position(|entry| entry.id == "interfaces-group")
            .expect("hidden group visible");
        app.nav.selected = idx;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('-'))));
        assert!(!app.nav.hidden.contains("interfaces-group"));
        assert!(app.status.contains("Restored"));
    }

    #[test]
    fn palette_omits_hidden_resources_until_they_are_revealed() {
        let mut app = main_app();
        app.nav.set_hidden_ids(vec!["vlan".into()]);
        app.rebuild_palette();
        assert!(app.palette.commands.iter().all(|cmd| cmd.id != "vlan"));
        app.toggle_show_hidden_menus();
        assert!(app.palette.commands.iter().any(|cmd| cmd.id == "vlan"));
    }

    #[test]
    fn palette_omits_unavailable_resources_even_when_showing_hidden() {
        let mut app = main_app();
        let mut missing = std::collections::HashMap::new();
        missing.insert("wifi".into(), "wifi-qcom".into());
        app.nav.set_unavailable(missing);
        app.rebuild_palette();
        assert!(app.palette.commands.iter().all(|cmd| cmd.id != "wifi"));
        app.toggle_show_hidden_menus();
        assert!(app.palette.commands.iter().all(|cmd| cmd.id != "wifi"));
    }
}

#[cfg(test)]
mod console_tests {
    use std::sync::Arc;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_config::{LogLevel, LogStore};
    use mtui_core::{DefaultTheme, Theme};
    use mtui_ui::{ConsoleEntry, ConsoleLevel, Styles, line_plain};

    use crate::app::{App, Pane, Screen};
    use crate::event::AppEvent;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn main_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.console_entries = vec![
            ConsoleEntry {
                time: "2026-08-22 03:25:01.000".into(),
                level: ConsoleLevel::Info,
                message: "outbound request".into(),
                fields: vec![("endpoint".into(), "/interface".into())],
            },
            ConsoleEntry {
                time: "2026-08-22 03:25:02.000".into(),
                level: ConsoleLevel::Error,
                message: "response /interface/list/add".into(),
                fields: vec![
                    ("status".into(), "400".into()),
                    (
                        "body".into(),
                        r#"{"error":400,"message":"Bad Request","detail":"no such item"}"#.into(),
                    ),
                ],
            },
            ConsoleEntry {
                time: "2026-08-22 03:25:03.000".into(),
                level: ConsoleLevel::Warn,
                message: "opened pane".into(),
                fields: vec![("kind".into(), "help".into())],
            },
        ];
        app
    }

    #[test]
    fn console_starts_hidden_and_backtick_toggles_it() {
        let mut app = main_app();
        assert!(!app.console.visible);
        assert_eq!(app.pane, Pane::Nav);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        assert!(app.console.visible);
        assert!(!app.console.fullscreen);
        assert_eq!(app.pane, Pane::Console);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        assert!(!app.console.visible);
        assert_eq!(app.pane, Pane::Nav);
    }

    #[test]
    fn closed_console_keeps_ingested_logs_until_shown() {
        let store = Arc::new(LogStore::with_capacity(16));
        store.push(
            chrono::Local::now(),
            LogLevel::Info,
            "mtui_routeros::client".into(),
            "outbound /interface/print".into(),
            std::collections::BTreeMap::new(),
        );
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.log_store = store;
        assert!(!app.console.visible);
        assert!(app.console_entries.is_empty());

        let _ = app.update(AppEvent::Tick);
        assert!(!app.console.visible);
        assert_eq!(app.console_entries.len(), 1);
        assert_eq!(app.console_entries[0].message, "outbound /interface/print");

        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        assert!(app.console.visible);
        assert_eq!(app.console_entries.len(), 1);
        assert_eq!(
            app.console
                .selected_entry(&app.console_entries)
                .map(|e| e.message.as_str()),
            Some("outbound /interface/print")
        );
    }

    #[test]
    fn f_makes_the_focused_console_fullscreen() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('f'))));
        assert!(app.console.fullscreen);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('f'))));
        assert!(!app.console.fullscreen);
    }

    #[test]
    fn slash_search_is_case_insensitive() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('/'))));
        for ch in "OUTBOUND".chars() {
            let _ = app.update(AppEvent::Input(press(KeyCode::Char(ch))));
        }
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert_eq!(app.console.filtered_indices(&app.console_entries), vec![0]);
    }

    #[test]
    fn page_keys_move_the_console_cursor() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::PageDown)));
        assert_eq!(app.console.selected, 2);
        let _ = app.update(AppEvent::Input(press(KeyCode::PageUp)));
        assert_eq!(app.console.selected, 0);
    }

    #[test]
    fn enter_expands_and_iteration_keeps_expansion() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(app.console.expanded);
        assert_eq!(app.console.selected, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        assert_eq!(app.console.selected, 1);
        assert!(app.console.expanded);
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(!app.console.expanded);
    }

    #[test]
    fn expanded_console_can_open_json_body() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(app.console.expanded);
        assert_eq!(app.console.selected, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('l'))));
        assert_eq!(app.console.expand_cursor, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        assert_eq!(app.console.selected, 1);
        assert_eq!(app.console.expand_cursor, 2);
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let lines = app.console.lines(
            &app.console_entries,
            &Styles::from_palette(DefaultTheme::new().palette()),
            88,
            16,
            true,
        );
        let plain = lines.iter().map(line_plain).collect::<Vec<_>>().join("\n");
        assert!(plain.contains("no such item"));
    }

    #[test]
    fn c_copies_the_focused_log() {
        let mut app = main_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('c'))));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                crate::app::AppCommand::CopyToClipboard { text, .. }
                    if text.contains("outbound request") && text.contains("endpoint: /interface")
            )),
            "expected copy command, got {cmds:?}"
        );
    }

    #[test]
    fn docked_console_uses_a_quarter_of_the_terminal() {
        let mut app = main_app();
        app.terminal_height = 24;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('`'))));
        assert_eq!(app.console_layout_height(), 6);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('f'))));
        assert_eq!(app.console_layout_height(), 15);
    }
}

#[cfg(test)]
mod lookup_picker_tests {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::app::{App, AppCommand, Overlay, Screen};
    use crate::event::{AppEvent, WorkerMsg};
    use mtui_ui::{FormSession, LOOKUP_TEST_FORM};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn lookup_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        let mut values = HashMap::new();
        values.insert("interface".into(), String::new());
        values.insert("ports".into(), "ether1".into());
        app.overlay = Overlay::Form(FormSession::prompt_fields(
            "bridge",
            "",
            "copy",
            &LOOKUP_TEST_FORM,
            values,
        ));
        if let Overlay::Form(session) = &mut app.overlay {
            session.original = session.values.clone();
        }
        app
    }

    #[test]
    fn space_opens_lookup_and_returns_fetch() {
        let mut app = lookup_app();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        let picker = session.lookup.as_ref().expect("picker");
        assert!(picker.loading);
        assert_eq!(picker.resource_id, "interfaces");
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                AppCommand::FetchLookup {
                    resource_id,
                    value_key,
                    ..
                } if resource_id == "interfaces" && value_key == "name"
            )),
            "expected FetchLookup, got {cmds:?}"
        );
    }

    #[test]
    fn worker_lookup_result_applies_and_ignores_stale() {
        let mut app = lookup_app();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        let Some(AppCommand::FetchLookup {
            request_id,
            generation,
            ..
        }) = cmds.into_iter().next()
        else {
            panic!("expected fetch");
        };

        let _ = app.update(AppEvent::Worker(WorkerMsg::LookupResult {
            session: app.test_session(),
            request_id: request_id.wrapping_add(1),
            generation,
            options: vec!["stale".into()],
            error: None,
        }));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert!(session.lookup.as_ref().unwrap().options.is_empty());
        assert!(session.lookup.as_ref().unwrap().loading);

        let _ = app.update(AppEvent::Worker(WorkerMsg::LookupResult {
            session: app.test_session(),
            request_id,
            generation,
            options: vec!["ether1".into(), "ether2".into()],
            error: None,
        }));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert_eq!(
            session.lookup.as_ref().unwrap().options,
            ["ether1", "ether2"]
        );
        assert!(!session.lookup.as_ref().unwrap().loading);

        let _ = app.update(AppEvent::Worker(WorkerMsg::LookupResult {
            session: app.test_session(),
            request_id,
            generation,
            options: Vec::new(),
            error: Some("timeout".into()),
        }));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert_eq!(
            session.lookup.as_ref().unwrap().error.as_deref(),
            Some("timeout")
        );
    }

    #[test]
    fn enter_selects_single_lookup_value() {
        let mut app = lookup_app();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        let Some(AppCommand::FetchLookup {
            request_id,
            generation,
            ..
        }) = cmds.into_iter().next()
        else {
            panic!("expected fetch");
        };
        let _ = app.update(AppEvent::Worker(WorkerMsg::LookupResult {
            session: app.test_session(),
            request_id,
            generation,
            options: vec!["ether1".into(), "bridge".into()],
            error: None,
        }));
        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert!(session.lookup.is_none());
        assert_eq!(
            session.values.get("interface").map(String::as_str),
            Some("bridge")
        );
    }

    #[test]
    fn typing_on_lookup_field_does_not_write_free_text() {
        let mut app = lookup_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('x'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert!(session.lookup.is_none());
        assert_eq!(
            session.values.get("interface").map(String::as_str),
            Some("")
        );
        assert_eq!(session.focus, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Char('j'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("form closed");
        };
        assert_eq!(session.focus, 1);
    }

    #[test]
    fn esc_closes_lookup_picker_not_form() {
        let mut app = lookup_app();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Esc)));
        assert!(matches!(app.overlay, Overlay::Form(_)));
        let Overlay::Form(session) = &app.overlay else {
            unreachable!();
        };
        assert!(session.lookup.is_none());
        let _ = app.update(AppEvent::Input(press(KeyCode::Esc)));
        assert_eq!(app.overlay, Overlay::None);
    }
}
