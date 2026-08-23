//! Login form state and the device-picker screen.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use crate::chrome::{
    Signal, SignalLevel, center_in_band, chrome_band_height, footer_bar, session_header,
};
use crate::layout::{clip_line, fit_cell};
use crate::overlay::{Modal, ModalButton, ModalButtonKind, render_modal};
use crate::paint::{fill_rect, line_on_bg};
use crate::styles::Styles;

/// True for runes that may be typed into a text field (not control/modifier noise).
#[must_use]
pub fn is_printable_char(ch: char) -> bool {
    !ch.is_control()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginPane {
    List,
    Form,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Name,
    Url,
    Username,
    Password,
    Totp,
    Remember,
    Connect,
}

impl LoginField {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Url,
            Self::Url => Self::Username,
            Self::Username => Self::Password,
            Self::Password => Self::Totp,
            Self::Totp => Self::Remember,
            Self::Remember => Self::Connect,
            Self::Connect => Self::Name,
        }
    }

    #[must_use]
    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::Connect,
            Self::Url => Self::Name,
            Self::Username => Self::Url,
            Self::Password => Self::Username,
            Self::Totp => Self::Password,
            Self::Remember => Self::Totp,
            Self::Connect => Self::Remember,
        }
    }

    #[must_use]
    pub fn is_secret(self) -> bool {
        matches!(self, Self::Password | Self::Totp)
    }

    #[must_use]
    pub fn is_text(self) -> bool {
        matches!(
            self,
            Self::Name | Self::Url | Self::Username | Self::Password | Self::Totp
        )
    }

    #[must_use]
    pub fn next_in_form(self) -> Option<Self> {
        match self {
            Self::Name => Some(Self::Url),
            Self::Url => Some(Self::Username),
            Self::Username => Some(Self::Password),
            Self::Password => Some(Self::Totp),
            Self::Totp => Some(Self::Remember),
            Self::Remember => Some(Self::Connect),
            Self::Connect => None,
        }
    }

    #[must_use]
    pub fn prev_in_form(self) -> Option<Self> {
        match self {
            Self::Name => None,
            Self::Url => Some(Self::Name),
            Self::Username => Some(Self::Url),
            Self::Password => Some(Self::Username),
            Self::Totp => Some(Self::Password),
            Self::Remember => Some(Self::Totp),
            Self::Connect => Some(Self::Remember),
        }
    }
}

/// One saved device row in the connection list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedProfileRow {
    pub name: String,
    pub url: String,
    pub username: String,
    pub remember_password: bool,
    pub uses_totp: bool,
}

#[derive(Debug, Clone)]
pub struct LoginForm {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub totp: String,
    pub remember_password: bool,
    pub uses_totp: bool,
    pub focus: LoginField,
    pub pane: LoginPane,
    pub error: Option<String>,
    pub profiles: Vec<SavedProfileRow>,
    pub selected_profile: usize,
    pub list_offset: usize,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            username: String::new(),
            password: String::new(),
            totp: String::new(),
            remember_password: true,
            uses_totp: false,
            focus: LoginField::Url,
            pane: LoginPane::Form,
            error: None,
            profiles: Vec::new(),
            selected_profile: 0,
            list_offset: 0,
        }
    }
}

impl LoginForm {
    pub fn focused_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            LoginField::Name => Some(&mut self.name),
            LoginField::Url => Some(&mut self.url),
            LoginField::Username => Some(&mut self.username),
            LoginField::Password => Some(&mut self.password),
            LoginField::Totp => Some(&mut self.totp),
            LoginField::Remember | LoginField::Connect => None,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if !is_printable_char(ch) || !self.focus.is_text() {
            return;
        }
        if self.focus == LoginField::Totp {
            if ch.is_ascii_digit() && self.totp.len() < 8 {
                self.totp.push(ch);
            }
            return;
        }
        if let Some(field) = self.focused_mut() {
            field.push(ch);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.focused_mut() {
            field.pop();
        }
    }

    pub fn toggle_remember(&mut self) {
        self.remember_password = !self.remember_password;
    }

    pub fn set_remember(&mut self, value: bool) {
        self.remember_password = value;
    }

    /// Tab from the form walks fields; the last step jumps to the router list.
    pub fn tab_forward(&mut self) {
        if self.profiles.is_empty() {
            self.focus = self.focus.next();
            return;
        }
        match self.pane {
            LoginPane::List => {
                self.pane = LoginPane::Form;
                self.focus = LoginField::Name;
            }
            LoginPane::Form => match self.focus.next_in_form() {
                Some(field) => self.focus = field,
                None => self.pane = LoginPane::List,
            },
        }
    }

    /// Shift+Tab walks fields backward; the first step from Name returns to the list.
    pub fn tab_back(&mut self) {
        if self.profiles.is_empty() {
            self.focus = self.focus.prev();
            return;
        }
        match self.pane {
            LoginPane::List => {
                self.pane = LoginPane::Form;
                self.focus = LoginField::Connect;
            }
            LoginPane::Form => match self.focus.prev_in_form() {
                Some(field) => self.focus = field,
                None => self.pane = LoginPane::List,
            },
        }
    }

    pub fn move_profile(&mut self, delta: isize) {
        let len = self.profiles.len();
        if len == 0 {
            self.selected_profile = 0;
            return;
        }
        let cur = isize::try_from(self.selected_profile).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        self.selected_profile = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
        self.ensure_list_visible(8);
    }

    pub fn ensure_list_visible(&mut self, visible: usize) {
        let visible = visible.max(1);
        if self.selected_profile < self.list_offset {
            self.list_offset = self.selected_profile;
        } else if self.selected_profile >= self.list_offset.saturating_add(visible) {
            self.list_offset = self
                .selected_profile
                .saturating_add(1)
                .saturating_sub(visible);
        }
    }

    #[must_use]
    pub fn selected_row(&self) -> Option<&SavedProfileRow> {
        self.profiles.get(self.selected_profile)
    }

    /// Secret sent to `RouterOS`: static password with optional TOTP digits.
    #[must_use]
    pub fn connect_secret(&self) -> String {
        format!("{}{}", self.password, self.totp.trim())
    }

    pub fn apply_row(&mut self, row: &SavedProfileRow) {
        self.name.clone_from(&row.name);
        self.url.clone_from(&row.url);
        self.username.clone_from(&row.username);
        self.remember_password = row.remember_password;
        self.uses_totp = row.uses_totp;
        self.totp.clear();
        self.focus = self.open_focus();
        self.pane = LoginPane::Form;
    }

    /// Saved profiles land on Login when credentials are already present.
    #[must_use]
    pub fn open_focus(&self) -> LoginField {
        if self.uses_totp && self.totp.is_empty() {
            LoginField::Totp
        } else if self.password.is_empty() && !self.remember_password {
            LoginField::Password
        } else {
            LoginField::Connect
        }
    }
}

/// View-only bits the login canvas needs besides [`LoginForm`].
pub struct LoginView<'a> {
    pub form: &'a LoginForm,
    pub status: &'a str,
    pub connecting: bool,
    pub clock: &'a str,
}

pub fn render_login(frame: &mut Frame<'_>, area: Rect, view: &LoginView<'_>, styles: &Styles) {
    fill_rect(frame, area, styles.void);
    let band = chrome_band_height(area.height);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(band),
            Constraint::Min(8),
            Constraint::Length(band),
        ])
        .split(area);

    fill_rect(frame, chunks[0], styles.band);
    let subtitle = if view.form.profiles.is_empty() {
        "connect"
    } else {
        "devices"
    };
    let clock_signals = if view.clock.is_empty() {
        Vec::new()
    } else {
        vec![Signal::new("", view.clock, SignalLevel::Idle)]
    };
    frame.render_widget(
        Paragraph::new(center_in_band(
            &session_header(
                "mikrotik-tui",
                subtitle,
                &clock_signals,
                usize::from(area.width.max(1)),
                styles,
                false,
            ),
            band,
            usize::from(area.width.max(1)),
        )),
        chunks[0],
    );

    render_login_body(frame, chunks[1], view, styles);

    fill_rect(frame, chunks[2], styles.inset);
    frame.render_widget(
        Paragraph::new(center_in_band(
            &footer_bar(
                view.status,
                &login_hints(view.form),
                usize::from(area.width.max(1)),
                styles,
            ),
            band,
            usize::from(area.width.max(1)),
        )),
        chunks[2],
    );

    if view.connecting {
        let name = if view.form.name.trim().is_empty() {
            view.form.url.as_str()
        } else {
            view.form.name.as_str()
        };
        let body =
            format!("Negotiating api-ssl with {name}\nEsc cancels without deleting the profile.");
        let buttons = [ModalButton {
            label: "Cancel",
            keys: "esc",
            kind: ModalButtonKind::Secondary,
        }];
        let modal = Modal::new("Connecting", &body)
            .kicker("Secure session")
            .hint("In-flight polls stop when you switch devices.")
            .buttons(&buttons);
        render_modal(frame, area, &modal, styles);
    }
}

fn login_hints(form: &LoginForm) -> Vec<(&'static str, &'static str)> {
    if form.profiles.is_empty() {
        vec![
            (
                "enter",
                if form.focus == LoginField::Connect {
                    "login"
                } else {
                    "next"
                },
            ),
            ("tab", "field"),
            ("space", "remember"),
            ("q", "quit"),
        ]
    } else if form.pane == LoginPane::List {
        vec![
            ("enter", "open"),
            ("→", "open"),
            ("n", "new"),
            ("x", "forget"),
            ("tab", "fields"),
            ("q", "quit"),
        ]
    } else {
        vec![
            (
                "enter",
                if form.focus == LoginField::Connect {
                    "login"
                } else {
                    "next"
                },
            ),
            ("tab", "field"),
            ("space", "remember"),
            ("esc", "list"),
            ("q", "quit"),
        ]
    }
}

fn inset_h(area: Rect, pad: u16) -> Rect {
    let pad = pad.min(area.width / 2);
    Rect {
        x: area.x.saturating_add(pad),
        y: area.y,
        width: area.width.saturating_sub(pad.saturating_mul(2)),
        height: area.height,
    }
}

fn render_login_body(frame: &mut Frame<'_>, area: Rect, view: &LoginView<'_>, styles: &Styles) {
    let area = inset_h(area, 1);
    if view.form.profiles.is_empty() || area.width < 72 {
        render_form_column(frame, inset_h(area, 1), view.form, styles, true);
        return;
    }
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(36)])
        .split(area);
    render_profile_list(frame, cols[0], view.form, styles);
    render_form_column(frame, inset_h(cols[1], 1), view.form, styles, false);
}

fn render_profile_list(frame: &mut Frame<'_>, area: Rect, form: &LoginForm, styles: &Styles) {
    let list_focus = form.pane == LoginPane::List;
    let block = Block::default()
        .title(Span::styled(" Routers ", styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if list_focus {
            styles.focus
        } else {
            styles.border
        })
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let visible = usize::from(inner.height.max(1));
    let mut offset = form.list_offset;
    if form.selected_profile < offset {
        offset = form.selected_profile;
    } else if form.selected_profile >= offset.saturating_add(visible) {
        offset = form
            .selected_profile
            .saturating_add(1)
            .saturating_sub(visible);
    }

    for row_i in 0..visible {
        let y = inner.y.saturating_add(u16::try_from(row_i).unwrap_or(0));
        let row_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        };
        let idx = offset.saturating_add(row_i);
        let Some(profile) = form.profiles.get(idx) else {
            break;
        };
        let selected = idx == form.selected_profile;
        let mark = if selected { "›" } else { " " };
        let lock = if profile.uses_totp {
            " 2FA"
        } else if profile.remember_password {
            ""
        } else {
            " · ask"
        };
        let label = format!(
            "{mark} {}{lock}",
            clip_line(&profile.name, usize::from(inner.width.saturating_sub(2)))
        );
        let mut line = Line::from(Span::styled(
            fit_cell(&label, usize::from(inner.width)),
            if selected && list_focus {
                styles.focus
            } else if selected {
                styles.text
            } else {
                styles.muted
            },
        ));
        if selected {
            line = line_on_bg(line, styles.selection);
            fill_rect(frame, row_area, styles.selection);
        }
        frame.render_widget(Paragraph::new(line), row_area);
    }
}

fn render_form_column(
    frame: &mut Frame<'_>,
    area: Rect,
    form: &LoginForm,
    styles: &Styles,
    stacked: bool,
) {
    let form_focus = form.pane == LoginPane::Form || form.profiles.is_empty();
    let mut constraints = vec![Constraint::Length(2)];
    if stacked && !form.profiles.is_empty() {
        constraints.insert(0, Constraint::Length(7));
    }
    constraints.extend([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(1),
    ]);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    if stacked && !form.profiles.is_empty() {
        render_profile_list(frame, chunks[idx], form, styles);
        idx += 1;
    }

    let identity = identity_kicker(form);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ", styles.muted),
            Span::styled(identity, styles.title.add_modifier(Modifier::BOLD)),
        ])),
        chunks[idx],
    );
    idx += 1;

    let password_shown = "*".repeat(form.password.len());
    let totp_shown = if form.totp.is_empty() {
        String::new()
    } else {
        "•".repeat(form.totp.len())
    };

    let fields = [
        ("Name", form.name.as_str(), LoginField::Name),
        ("Host", form.url.as_str(), LoginField::Url),
        ("Username", form.username.as_str(), LoginField::Username),
        ("Password", password_shown.as_str(), LoginField::Password),
        ("TOTP", totp_shown.as_str(), LoginField::Totp),
    ];
    for (label, value, field) in fields {
        let focused = form_focus && form.focus == field;
        render_field(frame, chunks[idx], label, value, focused, field, styles);
        idx += 1;
    }
    render_remember_field(
        frame,
        chunks[idx],
        form.remember_password,
        form_focus && form.focus == LoginField::Remember,
        styles,
    );
    idx += 1;
    render_connect_button(
        frame,
        chunks[idx],
        form_focus && form.focus == LoginField::Connect,
        styles,
    );
    idx += 1;

    let err = form.error.clone().unwrap_or_default();
    frame.render_widget(Paragraph::new(err).style(styles.error), chunks[idx]);
}

fn identity_kicker(form: &LoginForm) -> String {
    let host = form.url.trim();
    let user = form.username.trim();
    match (host.is_empty(), user.is_empty()) {
        (true, true) => "New device".into(),
        (false, true) => host.to_string(),
        (true, false) => user.to_string(),
        (false, false) => format!("{host}  ·  {user}"),
    }
}

fn render_field(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    field: LoginField,
    styles: &Styles,
) {
    let hint = match field {
        LoginField::Totp => " optional · 6 digits · never saved",
        _ => "",
    };
    let title = if hint.is_empty() {
        format!(" {label} ")
    } else {
        format!(" {label}{hint} ")
    };
    let style = if focused { styles.focus } else { styles.text };
    let block = Block::default()
        .title(Span::styled(
            title,
            if focused { styles.focus } else { styles.muted },
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if focused { styles.focus } else { styles.border })
        .padding(Padding::new(1, 1, 0, 0));
    let shown = if focused {
        format!("{value}▏")
    } else {
        value.to_string()
    };
    frame.render_widget(Paragraph::new(shown).style(style).block(block), area);
}

fn render_remember_field(
    frame: &mut Frame<'_>,
    area: Rect,
    on: bool,
    focused: bool,
    styles: &Styles,
) {
    let block = Block::default()
        .title(Span::styled(
            " Remember password ",
            if focused { styles.focus } else { styles.muted },
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if focused { styles.focus } else { styles.border })
        .padding(Padding::new(1, 1, 0, 0));
    let on_mark = if on { "(•)" } else { "( )" };
    let off_mark = if on { "( )" } else { "(•)" };
    let on_style = if focused && on {
        styles.focus
    } else if on {
        styles.text
    } else {
        styles.muted
    };
    let off_style = if focused && !on {
        styles.focus
    } else if !on {
        styles.text
    } else {
        styles.muted
    };
    let line = Line::from(vec![
        Span::styled(format!("{on_mark} On"), on_style),
        Span::styled("    ", styles.muted),
        Span::styled(format!("{off_mark} Off"), off_style),
        Span::styled("  space toggles", styles.muted),
    ]);
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn render_connect_button(frame: &mut Frame<'_>, area: Rect, focused: bool, styles: &Styles) {
    if focused {
        fill_rect(frame, area, styles.selection);
    }
    let bracket = if focused { styles.focus } else { styles.border };
    let label = if focused {
        styles.focus.add_modifier(Modifier::BOLD)
    } else {
        styles.text
    };
    let mut line = Line::from(vec![
        Span::styled("[ ", bracket),
        Span::styled("Login", label),
        Span::styled(" ]", bracket),
        Span::styled(" enter", styles.muted),
    ]);
    if focused {
        line = line_on_bg(line, styles.selection);
    }
    frame.render_widget(Paragraph::new(line), area);
}

/// Re-auth overlay copy while a live session is still on screen.
pub struct ReauthView<'a> {
    pub username: &'a str,
    pub password_len: usize,
    pub totp_len: usize,
    pub totp_focus: bool,
    pub error: Option<&'a str>,
}

pub fn render_reauth(frame: &mut Frame<'_>, area: Rect, view: &ReauthView<'_>, styles: &Styles) {
    let pwd = "*".repeat(view.password_len);
    let totp = if view.totp_len == 0 {
        String::from("(optional)")
    } else {
        "•".repeat(view.totp_len)
    };
    let body = format!(
        "The router rejected this session for {}.\nProfiles and this screen stay put.\n\nPassword  {pwd}\nTOTP      {totp}\n\n{}",
        view.username,
        view.error
            .unwrap_or("Enter the password (and TOTP if User Manager 2FA is on).")
    );
    let buttons = [
        ModalButton {
            label: "Reconnect",
            keys: "enter",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Cancel",
            keys: "esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let focus = if view.totp_focus {
        "TOTP is focused · tab switches"
    } else {
        "Password is focused · tab switches"
    };
    let modal = Modal::new("Sign in again", &body)
        .alert()
        .kicker("Credentials expired")
        .hint(focus)
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn insert_char_appends_printable_runes() {
        let mut form = LoginForm {
            focus: LoginField::Username,
            ..LoginForm::default()
        };
        form.insert_char('a');
        form.insert_char('d');
        form.insert_char('m');
        form.insert_char('i');
        form.insert_char('n');
        assert_eq!(form.username, "admin");
    }

    #[test]
    fn insert_char_ignores_control_runes() {
        let mut form = LoginForm {
            focus: LoginField::Password,
            ..LoginForm::default()
        };
        form.insert_char('\0');
        form.insert_char('\r');
        form.insert_char('\n');
        form.insert_char('\u{1b}');
        form.insert_char('\u{8}');
        assert_eq!(form.password, "");
        form.insert_char('P');
        form.insert_char('ä');
        assert_eq!(form.password, "Pä");
    }

    #[test]
    fn totp_accepts_digits_only_and_is_appended_at_connect() {
        let mut form = LoginForm {
            password: "secret".into(),
            focus: LoginField::Totp,
            ..LoginForm::default()
        };
        form.insert_char('1');
        form.insert_char('a');
        form.insert_char('2');
        form.insert_char('\0');
        assert_eq!(form.totp, "12");
        assert_eq!(form.connect_secret(), "secret12");
    }

    #[test]
    fn backspace_removes_last_character_of_focused_field() {
        let mut form = LoginForm {
            url: "192.168.88.1".into(),
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.url, "192.168.88.");
        form.focus = LoginField::Username;
        form.username = "admin".into();
        form.backspace();
        assert_eq!(form.username, "admi");
        assert_eq!(form.url, "192.168.88.");
    }

    #[test]
    fn backspace_on_empty_field_is_a_no_op() {
        let mut form = LoginForm {
            focus: LoginField::Username,
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.username, "");
    }

    #[test]
    fn backspace_pops_a_whole_unicode_scalar() {
        let mut form = LoginForm {
            focus: LoginField::Password,
            password: "Päss".into(),
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.password, "Päs");
    }

    #[test]
    fn url_next_is_still_username() {
        assert_eq!(LoginField::Url.next(), LoginField::Username);
        assert_eq!(LoginField::Password.prev(), LoginField::Username);
        assert_eq!(LoginField::Remember.next(), LoginField::Connect);
        assert_eq!(LoginField::Connect.prev(), LoginField::Remember);
    }

    fn sample_rows() -> Vec<SavedProfileRow> {
        vec![SavedProfileRow {
            name: "core".into(),
            url: "192.168.88.1:8729".into(),
            username: "admin".into(),
            remember_password: true,
            uses_totp: false,
        }]
    }

    #[test]
    fn tab_on_form_walks_fields_then_returns_to_the_list() {
        let mut form = LoginForm {
            profiles: sample_rows(),
            pane: LoginPane::Form,
            focus: LoginField::Name,
            ..LoginForm::default()
        };
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Url);
        assert_eq!(form.pane, LoginPane::Form);
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Username);
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Password);
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Totp);
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Remember);
        form.tab_forward();
        assert_eq!(form.focus, LoginField::Connect);
        form.tab_forward();
        assert_eq!(form.pane, LoginPane::List);
        form.tab_forward();
        assert_eq!(form.pane, LoginPane::Form);
        assert_eq!(form.focus, LoginField::Name);
        form.tab_back();
        assert_eq!(form.pane, LoginPane::List);
        form.tab_back();
        assert_eq!(form.pane, LoginPane::Form);
        assert_eq!(form.focus, LoginField::Connect);
    }

    #[test]
    fn remember_field_draws_on_and_off_choices() {
        let backend = TestBackend::new(90, 28);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let form = LoginForm {
            remember_password: true,
            pane: LoginPane::Form,
            focus: LoginField::Remember,
            profiles: sample_rows(),
            ..LoginForm::default()
        };
        terminal
            .draw(|frame| {
                render_login(
                    frame,
                    frame.area(),
                    &LoginView {
                        form: &form,
                        status: "ok",
                        connecting: false,
                        clock: "",
                    },
                    &styles,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut found_on = false;
        let mut found_off = false;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains("On") {
                found_on = true;
            }
            if row.contains("Off") {
                found_off = true;
            }
        }
        assert!(found_on, "remember On choice missing");
        assert!(found_off, "remember Off choice missing");
    }

    #[test]
    fn connect_button_draws_login_label() {
        let backend = TestBackend::new(90, 28);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let form = LoginForm {
            pane: LoginPane::Form,
            focus: LoginField::Connect,
            profiles: sample_rows(),
            ..LoginForm::default()
        };
        terminal
            .draw(|frame| {
                render_login(
                    frame,
                    frame.area(),
                    &LoginView {
                        form: &form,
                        status: "ok",
                        connecting: false,
                        clock: "",
                    },
                    &styles,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut found = false;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains("Login") {
                found = true;
                break;
            }
        }
        assert!(found, "Login button missing from the form");
    }

    #[test]
    fn login_layout_keeps_header_and_footer_on_a_narrow_terminal() {
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let form = LoginForm {
            name: "core".into(),
            url: "192.168.88.1:8729".into(),
            username: "admin".into(),
            profiles: vec![SavedProfileRow {
                name: "core".into(),
                url: "192.168.88.1:8729".into(),
                username: "admin".into(),
                remember_password: true,
                uses_totp: false,
            }],
            pane: LoginPane::List,
            ..LoginForm::default()
        };
        terminal
            .draw(|frame| {
                render_login(
                    frame,
                    frame.area(),
                    &LoginView {
                        form: &form,
                        status: "pick a router",
                        connecting: false,
                        clock: "2026-08-23  14:12:00",
                    },
                    &styles,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut found_title = false;
        let mut found_footer = false;
        let mut found_clock = false;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains("mikrotik-tui") {
                found_title = true;
            }
            if row.contains("2026-08-23") {
                found_clock = true;
            }
            if row.contains("enter") {
                found_footer = true;
            }
        }
        assert!(found_title);
        assert!(found_footer);
        assert!(found_clock);
    }

    #[test]
    fn selected_profile_row_fill_stays_inside_the_list() {
        let backend = TestBackend::new(90, 28);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let form = LoginForm {
            profiles: vec![
                SavedProfileRow {
                    name: "alpha".into(),
                    url: "10.0.0.1:8729".into(),
                    username: "admin".into(),
                    remember_password: true,
                    uses_totp: false,
                },
                SavedProfileRow {
                    name: "bravo".into(),
                    url: "10.0.0.2:8729".into(),
                    username: "reader".into(),
                    remember_password: false,
                    uses_totp: true,
                },
            ],
            selected_profile: 0,
            pane: LoginPane::List,
            ..LoginForm::default()
        };
        terminal
            .draw(|frame| {
                fill_rect(frame, frame.area(), styles.void);
                render_login(
                    frame,
                    frame.area(),
                    &LoginView {
                        form: &form,
                        status: "ok",
                        connecting: false,
                        clock: "2026-08-23  14:12:00",
                    },
                    &styles,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut selected_rows = 0_u16;
        for y in 0..buf.area.height {
            let mut painted = 0_u16;
            for x in 0..buf.area.width {
                if buf[(x, y)].bg == styles.selection {
                    painted += 1;
                }
            }
            if painted > 0 {
                selected_rows += 1;
                assert!(
                    painted < buf.area.width,
                    "selection bleed on row {y}: {painted} cells"
                );
            }
        }
        assert!(selected_rows >= 1);
        assert!(selected_rows <= 3, "selection painted {selected_rows} rows");
    }
}
