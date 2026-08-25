//! Top-level application model.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;

use mtui_config::{
    Credential, CredentialStore, EnvOverrides, LogLevel, LogRecord, LogStore,
    PlatformCredentialStore, Profile, ProfileStore, read_ca_file, shared_log_store,
};
use mtui_core::{
    ALL_RESOURCES, DASHBOARD_ID, MISSING_PATH_REASON, ResourceSpec, ThemeRegistry, ThemeSet,
    edit_resource_for_interface_type, installed_package_names, is_missing_command_prefix,
    merge_unavailable_menus, resource_by_id, unavailable_menus_for_device,
};

use mtui_routeros::{
    ErrorKind, Resource, header_host, merge_listen_record, migrate_connection_target_for,
    parse_connection_target,
};
use mtui_ui::{
    ActionMenuState, Command, ConsoleEntry, ConsoleLevel, DashboardGeometry, FirewallHitChart,
    FormSession, InspectorState, LayoutMetrics, LoginPane, ProbeState, Row, SavedProfileRow,
    Signal, SignalLevel, TableState, ToggleHidden, TorchState, chrome_band_height,
    console_pane_height, format_rate, tab_strip_height,
};

use crate::demo::{DEMO_PROFILE_NAME, DEMO_URL, DemoStore, is_demo_target};
use crate::event::{AppEvent, WorkerMsg};
use crate::safe_mode::SafeModeAfter;
use crate::session::{MAX_SESSIONS, Session, SessionId};
use crate::telemetry::select_wan_interface;
use crate::write::{ConfirmSession, MutationOp};

const LOG_BUFFER_CAP: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectIntent {
    Login,
    Reauth,
    Reconnect,
}

#[derive(Debug, Clone, Default)]
pub struct ReauthState {
    pub password: String,
    pub totp: String,
    pub totp_focus: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
    Connecting,
    Trust,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Nav,
    Content,
    Inspector,
    Console,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    None,
    Help,
    About,
    Palette,
    Confirm(ConfirmSession),
    HideMenu {
        id: String,
        title: String,
        body: String,
    },
    Form(FormSession),
    ActionMenu(ActionMenuState),
    TypePicker(ActionMenuState),
    Torch(TorchState),
    Probe(ProbeState),
    FilePicker(mtui_ui::FilePickerState),
    ForgetProfile {
        name: String,
    },
    Reauth,
    SafeModeConflict {
        owner: String,
        user: String,
    },
    SafeModeLeave {
        next: SafeModeAfter,
    },
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    Quit,
    CloseSession {
        session: SessionId,
    },
    Connect {
        session: SessionId,
        url: String,
        username: String,
        password: String,
        pin: Option<String>,
        ca_pem: Option<Vec<u8>>,
        use_tls: bool,
    },
    FetchResource {
        session: SessionId,
        request_id: u64,
        generation: u64,
        resource_id: String,
    },
    FetchDashboard {
        session: SessionId,
        request_id: u64,
        generation: u64,
    },
    FetchHeader {
        session: SessionId,
        request_id: u64,
        generation: u64,
    },
    FetchAccess {
        session: SessionId,
        request_id: u64,
        generation: u64,
    },
    ProbeMenuPaths {
        session: SessionId,
        generation: u64,
    },
    ForgetProfile {
        session: SessionId,
        name: String,
    },
    Mutate {
        session: SessionId,
        request_id: u64,
        generation: u64,
        op: MutationOp,
    },
    FetchTorch {
        session: SessionId,
        request_id: u64,
        generation: u64,
        interface: String,
        src: String,
        dst: String,
        protocol: String,
        port: String,
    },
    FetchPing {
        session: SessionId,
        request_id: u64,
        generation: u64,
        address: String,
        count: String,
        src: String,
    },
    FetchTraceroute {
        session: SessionId,
        request_id: u64,
        generation: u64,
        address: String,
        count: String,
        src: String,
        protocol: String,
    },
    FetchProbe {
        session: SessionId,
        request_id: u64,
        generation: u64,
        endpoint: String,
        command: String,
        fields: BTreeMap<String, String>,
    },
    CopyToClipboard {
        session: SessionId,
        text: String,
    },
    ReadLocalFile {
        session: SessionId,
        request_id: u64,
        generation: u64,
        path: String,
        remote_name: String,
    },
    WriteLocalFile {
        session: SessionId,
        request_id: u64,
        generation: u64,
        path: String,
        contents: String,
    },
    FetchRecord {
        session: SessionId,
        request_id: u64,
        generation: u64,
        endpoint: String,
        id: String,
        local_path: String,
    },
    FetchLookup {
        session: SessionId,
        request_id: u64,
        generation: u64,
        resource_id: String,
        value_key: String,
    },
    FetchFormRecord {
        session: SessionId,
        request_id: u64,
        generation: u64,
        resource_id: String,
        endpoint: String,
        id: String,
    },
    ListLocalDir {
        session: SessionId,
        generation: u64,
        path: String,
    },
    FetchSafeMode {
        session: SessionId,
        generation: u64,
    },
}

impl AppCommand {
    #[must_use]
    pub fn session(&self) -> Option<SessionId> {
        match self {
            Self::Quit => None,
            Self::CloseSession { session }
            | Self::Connect { session, .. }
            | Self::FetchResource { session, .. }
            | Self::FetchDashboard { session, .. }
            | Self::FetchHeader { session, .. }
            | Self::FetchAccess { session, .. }
            | Self::ProbeMenuPaths { session, .. }
            | Self::ForgetProfile { session, .. }
            | Self::Mutate { session, .. }
            | Self::FetchTorch { session, .. }
            | Self::FetchPing { session, .. }
            | Self::FetchTraceroute { session, .. }
            | Self::FetchProbe { session, .. }
            | Self::CopyToClipboard { session, .. }
            | Self::ReadLocalFile { session, .. }
            | Self::WriteLocalFile { session, .. }
            | Self::FetchRecord { session, .. }
            | Self::FetchLookup { session, .. }
            | Self::FetchFormRecord { session, .. }
            | Self::ListLocalDir { session, .. }
            | Self::FetchSafeMode { session, .. } => Some(*session),
        }
    }

    fn assign_session(&mut self, id: SessionId) {
        match self {
            Self::Quit => {}
            Self::CloseSession { session }
            | Self::Connect { session, .. }
            | Self::FetchResource { session, .. }
            | Self::FetchDashboard { session, .. }
            | Self::FetchHeader { session, .. }
            | Self::FetchAccess { session, .. }
            | Self::ProbeMenuPaths { session, .. }
            | Self::ForgetProfile { session, .. }
            | Self::Mutate { session, .. }
            | Self::FetchTorch { session, .. }
            | Self::FetchPing { session, .. }
            | Self::FetchTraceroute { session, .. }
            | Self::FetchProbe { session, .. }
            | Self::CopyToClipboard { session, .. }
            | Self::ReadLocalFile { session, .. }
            | Self::WriteLocalFile { session, .. }
            | Self::FetchRecord { session, .. }
            | Self::FetchLookup { session, .. }
            | Self::FetchFormRecord { session, .. }
            | Self::ListLocalDir { session, .. }
            | Self::FetchSafeMode { session, .. } => *session = id,
        }
    }
}

pub struct App {
    pub sessions: Vec<Session>,
    pub active: SessionId,
    next_session_id: u64,
    pub themes: ThemeRegistry,
    pub theme: ThemeSet,
    pub should_quit: bool,
    pub alt_screen: bool,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub(crate) log_store: Arc<LogStore>,
    pub(crate) profiles: ProfileStore,
    credentials: Box<dyn CredentialStore>,
}

impl Deref for App {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        self.session(self.active)
            .expect("active session must exist")
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let id = self.active;
        self.session_mut(id).expect("active session must exist")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSeverity {
    All,
    Info,
    Warning,
    Error,
}

impl LogSeverity {
    pub(crate) fn cycle(self) -> Self {
        match self {
            Self::All => Self::Info,
            Self::Info => Self::Warning,
            Self::Warning => Self::Error,
            Self::Error => Self::All,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl App {
    pub fn new(alt_screen: bool) -> anyhow::Result<Self> {
        let profiles = ProfileStore::discover()?;
        let credentials = Box::new(PlatformCredentialStore::discover()?);
        Ok(Self::compose(alt_screen, profiles, credentials))
    }

    pub(crate) fn compose(
        alt_screen: bool,
        profiles: ProfileStore,
        credentials: Box<dyn CredentialStore>,
    ) -> Self {
        let themes = ThemeRegistry::with_default();
        let theme = ThemeSet::from_theme(themes.active().as_ref());
        let first = SessionId::raw(1);
        let mut app = Self {
            sessions: vec![Session::new(first)],
            active: first,
            next_session_id: 2,
            themes,
            theme,
            should_quit: false,
            alt_screen,
            terminal_width: 80,
            terminal_height: 24,
            log_store: shared_log_store(),
            profiles,
            credentials,
        };

        app.sync_table_viewport();
        app.load_saved_session();
        app
    }

    #[must_use]
    pub fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id == id)
    }

    pub fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|session| session.id == id)
    }

    pub(crate) fn with_active<R>(&mut self, f: impl FnOnce(&mut Session) -> R) -> R {
        let id = self.active;
        f(self.session_mut(id).expect("active session must exist"))
    }

    #[must_use]
    pub fn test_session(&self) -> SessionId {
        self.active
    }

    pub fn new_session(&mut self) -> Option<SessionId> {
        if self.sessions.len() >= MAX_SESSIONS {
            self.status = format!("Tab limit reached ({MAX_SESSIONS})");
            return None;
        }
        let id = SessionId::raw(self.next_session_id);
        self.next_session_id += 1;
        self.sessions.push(Session::new(id));
        self.active = id;
        self.reload_profile_rows();
        Some(id)
    }

    pub fn close_session(&mut self, id: SessionId) {
        if self.sessions.len() <= 1 {
            return;
        }
        if let Some(session) = self.session_mut(id) {
            session.client = None;
            session.pending_password = None;
            session.login.password.clear();
            session.poll_generation = session.poll_generation.wrapping_add(1);
            session.torch_generation = session.torch_generation.wrapping_add(1);
            session.probe_generation = session.probe_generation.wrapping_add(1);
        }
        let closing_active = self.active == id;
        let idx = self.sessions.iter().position(|session| session.id == id);
        self.sessions.retain(|session| session.id != id);
        if closing_active {
            let fallback = idx
                .and_then(|i| i.checked_sub(1))
                .unwrap_or(0)
                .min(self.sessions.len().saturating_sub(1));
            self.active = self.sessions[fallback].id;
        }
    }

    pub fn cycle_session(&mut self, delta: isize) -> Vec<AppCommand> {
        let Some(idx) = self
            .sessions
            .iter()
            .position(|session| session.id == self.active)
        else {
            return Vec::new();
        };
        let len = isize::try_from(self.sessions.len()).unwrap_or(1);
        if len == 0 {
            return Vec::new();
        }
        let next = (isize::try_from(idx).unwrap_or(0) + delta).rem_euclid(len);
        let next = usize::try_from(next).unwrap_or(0);
        self.active = self.sessions[next].id;
        self.fetch_safe_mode_if_ready()
    }

    fn apply_to(
        &mut self,
        id: SessionId,
        f: impl FnOnce(&mut Self) -> Vec<AppCommand>,
    ) -> Vec<AppCommand> {
        if self.session(id).is_none() {
            return Vec::new();
        }
        let previous = self.active;
        self.active = id;
        let cmds = f(self);
        if self.session(self.active).is_none()
            && let Some(session) = self.sessions.first()
        {
            self.active = session.id;
        }
        let cmds = self.stamp(cmds);
        if self.session(previous).is_some() {
            self.active = previous;
        }
        cmds
    }

    pub(crate) fn stamp(&self, mut cmds: Vec<AppCommand>) -> Vec<AppCommand> {
        let id = self.active;
        for cmd in &mut cmds {
            match cmd {
                AppCommand::Quit | AppCommand::CloseSession { .. } => {}
                other if other.session().is_some_and(|session| session.get() == 0) => {
                    other.assign_session(id);
                }
                _ => {}
            }
        }
        cmds
    }

    fn load_saved_session(&mut self) {
        let overrides = EnvOverrides::from_env();
        self.reload_profile_rows();
        let last_used = self.profiles.last_used().ok().flatten();
        let mut profile = if let Some(name) = last_used.as_deref() {
            self.profiles
                .load()
                .ok()
                .and_then(|list| list.into_iter().find(|item| item.name == name))
        } else {
            self.profiles
                .load()
                .ok()
                .and_then(|list| list.into_iter().next())
        }
        .unwrap_or_default();

        if let Err(err) = overrides.apply_to_profile(&mut profile) {
            self.status = format!("Saved session overrides failed: {err}");
        }

        if profile.name.is_empty() {
            if !profile.url.is_empty() {
                self.login.url = migrate_connection_target_for(&profile.url, profile.use_tls);
                self.login.use_tls = profile.use_tls;
                self.login.ca_file.clone_from(&profile.ca_file);
            }
            if !profile.username.is_empty() {
                self.login.username.clone_from(&profile.username);
            }
            self.login.pane = LoginPane::Form;
            self.login.focus = mtui_ui::LoginField::Url;
        } else {
            self.apply_profile(&profile, true);
        }

        match overrides.resolve_password(
            if self.current_profile.is_empty() {
                profile.name.as_str()
            } else {
                self.current_profile.as_str()
            },
            Some(&*self.credentials),
        ) {
            Ok(Some(password)) => self.login.password = password,
            Ok(None) => {}
            Err(err) => {
                self.status = format!("Saved credentials unavailable: {err}");
                return;
            }
        }

        let has_router =
            is_router_target(&self.login.url) && !self.login.username.trim().is_empty();
        if has_router && !self.login.uses_totp {
            self.login.pane = LoginPane::List;
            self.status = if self.login.password.is_empty() && !self.login.remember_password {
                format!(
                    "Loaded {} · enter password to connect",
                    self.profile_label()
                )
            } else {
                format!("Loaded {} · enter to open", self.profile_label())
            };
        } else if has_router && self.login.uses_totp {
            self.login.pane = LoginPane::Form;
            self.login.focus = mtui_ui::LoginField::Totp;
            self.status = format!("Enter TOTP for {}", self.profile_label());
        } else if !self.login.profiles.is_empty() {
            self.login.pane = LoginPane::List;
            self.status = "Select a router · n new · enter connect".into();
        }
    }

    fn reload_profile_rows(&mut self) {
        let rows = self
            .profiles
            .load()
            .unwrap_or_default()
            .into_iter()
            .map(|profile| SavedProfileRow {
                name: profile.name,
                url: migrate_connection_target_for(&profile.url, profile.use_tls),
                username: profile.username,
                remember_password: profile.remember_password,
                uses_totp: profile.uses_totp,
                use_tls: profile.use_tls,
                ca_file: profile.ca_file,
            })
            .collect::<Vec<_>>();
        let mut rows = rows;
        rows.insert(
            0,
            SavedProfileRow {
                name: DEMO_PROFILE_NAME.into(),
                url: DEMO_URL.into(),
                username: "demo".into(),
                remember_password: false,
                uses_totp: false,
                use_tls: true,
                ca_file: String::new(),
            },
        );
        self.login.profiles = rows;
        if self.login.selected_profile >= self.login.profiles.len() {
            self.login.selected_profile = self.login.profiles.len().saturating_sub(1);
        }
    }

    pub(crate) fn apply_profile(&mut self, profile: &Profile, load_secret: bool) {
        let same_profile = self.current_profile == profile.name || self.login.name == profile.name;
        self.current_profile.clone_from(&profile.name);
        self.login.name.clone_from(&profile.name);
        self.login.url = migrate_connection_target_for(&profile.url, profile.use_tls);
        self.login.username.clone_from(&profile.username);
        self.login.remember_password = profile.remember_password;
        self.login.uses_totp = profile.uses_totp;
        self.login.use_tls = profile.use_tls;
        self.login.ca_file.clone_from(&profile.ca_file);
        self.login.totp.clear();
        if let Some(idx) = self
            .login
            .profiles
            .iter()
            .position(|row| row.name == profile.name)
        {
            self.login.selected_profile = idx;
        }
        if let Some(theme_id) = profile.theme_id() {
            let _ = self.themes.set_active(theme_id);
            self.theme = ThemeSet::from_theme(self.themes.active().as_ref());
        }
        self.nav.set_hidden_ids(profile.hidden_nav_ids());
        self.rebuild_palette();
        if profile.certificate_fingerprint.is_empty() {
            self.saved_fingerprint = None;
            self.saved_url = None;
        } else {
            self.saved_fingerprint = Some(profile.certificate_fingerprint.clone());
            self.saved_url = Some(normalize_router_url(&profile.url, profile.use_tls));
        }
        if profile.custom_ca.is_empty() {
            self.custom_ca = None;
        } else {
            self.custom_ca = Some(profile.custom_ca.clone().into_bytes());
        }
        if load_secret && profile.remember_password {
            match EnvOverrides::from_env().resolve_password(&profile.name, Some(&*self.credentials))
            {
                Ok(Some(password)) if !password.is_empty() => self.login.password = password,
                Ok(_) if same_profile && !self.login.password.is_empty() => {}
                Ok(_) => self.login.password.clear(),
                Err(err) => {
                    self.status = format!("Saved credentials unavailable: {err}");
                    if !same_profile {
                        self.login.password.clear();
                    }
                }
            }
        } else if load_secret {
            self.login.password.clear();
        }
        if !self.login.profiles.is_empty() {
            self.login.pane = LoginPane::List;
        }
    }

    pub(crate) fn profile_label(&self) -> String {
        if self.login.name.trim().is_empty() {
            "device".into()
        } else {
            self.login.name.trim().to_string()
        }
    }

    /// Kept for TOTP and empty-session startup; saved profiles no longer auto-connect.
    pub fn startup_commands(&mut self) -> Vec<AppCommand> {
        if !self.restore_on_start {
            return Vec::new();
        }
        self.restore_on_start = false;
        if self.login.uses_totp {
            return Vec::new();
        }
        if !is_router_target(&self.login.url) || self.login.username.trim().is_empty() {
            return Vec::new();
        }
        self.begin_connect()
    }

    pub(crate) fn connect_command(&self) -> AppCommand {
        let url = normalize_router_url(&self.login.url, self.login.use_tls);
        tracing::info!(
            url = url.as_str(),
            username = self.login.username.trim(),
            profile = self.current_profile.as_str(),
            use_tls = self.login.use_tls,
            "connecting"
        );
        AppCommand::Connect {
            session: SessionId::UNSTAMPED,
            url: url.clone(),
            username: self.login.username.trim().to_string(),
            password: self.pending_password.clone().unwrap_or_default(),
            pin: if self.login.use_tls {
                self.pin_for_url(&url)
            } else {
                None
            },
            ca_pem: if self.login.use_tls {
                self.custom_ca.clone()
            } else {
                None
            },
            use_tls: self.login.use_tls,
        }
    }

    pub(crate) fn begin_connect(&mut self) -> Vec<AppCommand> {
        if is_demo_target(&self.login.url)
            || self
                .login
                .name
                .trim()
                .eq_ignore_ascii_case(DEMO_PROFILE_NAME)
        {
            return self.enter_demo();
        }
        if !is_router_target(&self.login.url) {
            self.login.error = Some("Enter a router host (host or host:port)".into());
            self.status = "Enter a router host (host or host:port)".into();
            return Vec::new();
        }
        if self.login.username.trim().is_empty() {
            self.login.error = Some("Username is required".into());
            self.status = "Username is required".into();
            return Vec::new();
        }
        if self.login.name.trim().is_empty() {
            self.login.name = suggested_profile_name(
                &self.login.url,
                self.login.username.trim(),
                &self
                    .login
                    .profiles
                    .iter()
                    .map(|row| row.name.clone())
                    .collect::<Vec<_>>(),
            );
        }
        if let Err(err) = self.load_ca_file_if_needed() {
            self.login.error = Some(err.clone());
            self.status = err;
            return Vec::new();
        }
        self.current_profile = self.login.name.trim().to_string();
        self.login.error = None;
        self.connect_intent = ConnectIntent::Login;
        self.pending_password = Some(self.login.connect_secret());
        self.screen = Screen::Connecting;
        self.status = format!("Connecting to {}…", self.profile_label());
        vec![self.connect_command()]
    }

    pub(crate) fn open_ca_file_picker(&mut self) -> Vec<AppCommand> {
        let path = crate::files_io::default_browse_dir(&self.login.ca_file);
        let generation = self.next_request();
        self.overlay =
            Overlay::FilePicker(mtui_ui::FilePickerState::loading(path.clone(), generation));
        self.status = "Browse for a CA file".into();
        vec![AppCommand::ListLocalDir {
            session: SessionId::UNSTAMPED,
            generation,
            path,
        }]
    }

    pub(crate) fn list_picker_dir(&mut self, path: String) -> Vec<AppCommand> {
        if !matches!(self.overlay, Overlay::FilePicker(_)) {
            return Vec::new();
        }
        let generation = self.next_request();
        if let Overlay::FilePicker(picker) = &mut self.overlay {
            picker.begin_list(path.clone(), generation);
        }
        vec![AppCommand::ListLocalDir {
            session: SessionId::UNSTAMPED,
            generation,
            path,
        }]
    }

    fn load_ca_file_if_needed(&mut self) -> std::result::Result<(), String> {
        if !self.login.use_tls {
            return Ok(());
        }
        let path = self.login.ca_file.trim();
        if path.is_empty() {
            return Ok(());
        }
        match read_ca_file(path) {
            Ok(bytes) if bytes.is_empty() => Err("CA file is empty".into()),
            Ok(bytes) => {
                self.custom_ca = Some(bytes);
                Ok(())
            }
            Err(err) => Err(format!("Cannot read CA file: {err}")),
        }
    }

    fn pin_for_url(&self, url: &str) -> Option<String> {
        if let Some(fingerprint) = self.trust_fingerprint.as_deref()
            && !fingerprint.is_empty()
        {
            return Some(fingerprint.to_string());
        }
        match (&self.saved_url, &self.saved_fingerprint) {
            (Some(saved_url), Some(fingerprint)) if saved_url == url && !fingerprint.is_empty() => {
                Some(fingerprint.clone())
            }
            _ => None,
        }
    }

    fn password_to_remember(&self) -> String {
        let totp = self.login.totp.trim();
        if let Some(pending) = self.pending_password.as_deref() {
            let static_part = if !totp.is_empty() && pending.ends_with(totp) {
                pending[..pending.len().saturating_sub(totp.len())].to_string()
            } else {
                pending.to_string()
            };
            if !static_part.is_empty() {
                return static_part;
            }
        }
        self.login.password.clone()
    }

    fn store_or_forget_password(&mut self, name: &str) {
        if !self.login.remember_password {
            let _ = self.credentials.delete(name);
            return;
        }
        let password = self.password_to_remember();
        if password.is_empty() {
            return;
        }
        if self.credentials.put(name, Credential { password }).is_err() {
            self.status = "Password could not be stored".into();
        }
    }

    /// Save a remembered draft from the login screen so a typed password
    /// survives quitting without requiring another successful connect.
    pub(crate) fn persist_login_draft(&mut self) {
        if self.demo.is_some()
            || self
                .login
                .name
                .trim()
                .eq_ignore_ascii_case(DEMO_PROFILE_NAME)
            || is_demo_target(&self.login.url)
        {
            return;
        }
        if !is_router_target(&self.login.url) || self.login.username.trim().is_empty() {
            return;
        }
        if !self.login.remember_password && self.login.password.is_empty() {
            return;
        }
        let url = normalize_router_url(&self.login.url, self.login.use_tls);
        let name = if !self.login.name.trim().is_empty() {
            self.login.name.trim().to_string()
        } else if !self.current_profile.trim().is_empty() {
            self.current_profile.trim().to_string()
        } else {
            suggested_profile_name(
                &url,
                self.login.username.trim(),
                &self
                    .login
                    .profiles
                    .iter()
                    .map(|row| row.name.clone())
                    .collect::<Vec<_>>(),
            )
        };
        if name.eq_ignore_ascii_case(DEMO_PROFILE_NAME) {
            return;
        }
        self.current_profile.clone_from(&name);
        self.login.name.clone_from(&name);
        let mut profile = self.named_profile().unwrap_or_else(|| Profile {
            name: name.clone(),
            ..Profile::default()
        });
        profile.name.clone_from(&name);
        profile.url.clone_from(&url);
        profile.username = self.login.username.trim().to_string();
        profile.remember_password = self.login.remember_password;
        profile.use_tls = self.login.use_tls;
        profile.ca_file.clone_from(&self.login.ca_file);
        if self.profiles.upsert(profile).is_err() {
            return;
        }
        let _ = self.profiles.set_last_used(&name);
        self.store_or_forget_password(&name);
        self.reload_profile_rows();
    }

    fn persist_connected_session(&mut self) {
        if self.demo.is_some()
            || self
                .login
                .name
                .trim()
                .eq_ignore_ascii_case(DEMO_PROFILE_NAME)
        {
            return;
        }
        if cfg!(test) && self.current_profile.is_empty() {
            return;
        }
        let url = normalize_router_url(&self.login.url, self.login.use_tls);
        let fingerprint = if self.login.use_tls {
            self.pin_for_url(&url).unwrap_or_default()
        } else {
            String::new()
        };
        let name = if self.current_profile.trim().is_empty() {
            suggested_profile_name(
                &url,
                self.login.username.trim(),
                &self
                    .login
                    .profiles
                    .iter()
                    .map(|row| row.name.clone())
                    .collect::<Vec<_>>(),
            )
        } else {
            self.current_profile.trim().to_string()
        };
        self.current_profile.clone_from(&name);
        self.login.name.clone_from(&name);
        let mut profile = self.named_profile().unwrap_or_else(|| Profile {
            name: name.clone(),
            ..Profile::default()
        });
        profile.name.clone_from(&name);
        profile.url.clone_from(&url);
        profile.username = self.login.username.trim().to_string();
        profile.certificate_fingerprint.clone_from(&fingerprint);
        profile.use_tls = self.login.use_tls;
        profile.ca_file.clone_from(&self.login.ca_file);
        if self.login.use_tls
            && profile.ca_file.trim().is_empty()
            && let Some(pem) = &self.custom_ca
        {
            profile.custom_ca = String::from_utf8_lossy(pem).into_owned();
        }
        if !self.login.use_tls {
            profile.custom_ca.clear();
            profile.certificate_fingerprint.clear();
        }
        profile.remember_password = self.login.remember_password;
        profile.uses_totp = self.login.uses_totp || !self.login.totp.trim().is_empty();
        self.login.uses_totp = profile.uses_totp;
        profile.set_theme_id(self.theme.id.as_str());
        profile.set_hidden_nav_ids(self.nav.hidden.iter().cloned());
        if self.profiles.upsert(profile).is_err() {
            self.status = "Connected · profile could not be saved".into();
            return;
        }
        let _ = self.profiles.set_last_used(&name);
        self.store_or_forget_password(&name);
        self.login.totp.clear();
        self.reload_profile_rows();
        self.saved_url = Some(url);
        self.saved_fingerprint = if fingerprint.is_empty() {
            None
        } else {
            Some(fingerprint)
        };
    }

    pub(crate) fn forget_profile(&mut self, name: &str) {
        let _ = self.profiles.delete(name);
        let _ = self.credentials.delete(name);
        if self.current_profile == name {
            self.current_profile.clear();
            self.saved_fingerprint = None;
            self.saved_url = None;
        }
        self.reload_profile_rows();
        let saved = self
            .login
            .profiles
            .iter()
            .any(|row| !is_demo_target(&row.url));
        if saved {
            if let Some(row) = self.login.selected_row().cloned()
                && !is_demo_target(&row.url)
                && let Some(profile) = self
                    .profiles
                    .load()
                    .ok()
                    .into_iter()
                    .flatten()
                    .find(|item| item.name == row.name)
            {
                self.apply_profile(&profile, true);
            }
            self.login.pane = LoginPane::List;
            self.status = format!("Forgot {name}");
        } else {
            self.login.pane = LoginPane::List;
            self.status = "Device forgotten · Demo is still available".into();
        }
        self.restore_on_start = false;
    }

    pub(crate) fn bump_request_generation(&mut self) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.torch_generation = self.torch_generation.wrapping_add(1);
        self.probe_generation = self.probe_generation.wrapping_add(1);
        self.menu_paths_generation = self.menu_paths_generation.wrapping_add(1);
    }

    pub(crate) fn disconnect_to_profiles(&mut self) {
        self.bump_request_generation();
        self.client = None;
        self.demo = None;
        self.missing_path_ids.clear();
        self.nav.set_unavailable(HashMap::new());
        self.router = Resource::default();
        self.overlay = Overlay::None;
        self.connect_intent = ConnectIntent::Login;
        self.pending_password = None;
        self.trust_fingerprint = None;
        self.login.totp.clear();
        self.login.password.clear();
        self.reload_profile_rows();
        if let Some(name) = self.profiles.last_used().ok().flatten()
            && let Some(profile) = self
                .profiles
                .load()
                .ok()
                .into_iter()
                .flatten()
                .find(|item| item.name == name)
        {
            self.apply_profile(&profile, true);
        } else if self.login.profiles.is_empty() {
            self.login.pane = LoginPane::Form;
        } else {
            self.login.pane = LoginPane::List;
        }
        self.reset_link();
        self.screen = Screen::Login;
        self.status = "Disconnected · profiles kept".into();
    }

    pub(crate) fn open_reauth(&mut self, message: String) {
        if matches!(self.overlay, Overlay::Reauth) {
            return;
        }
        self.reauth = ReauthState {
            password: self.login.password.clone(),
            totp: String::new(),
            totp_focus: self.login.uses_totp,
            error: Some(message),
        };
        self.overlay = Overlay::Reauth;
        self.status = "Session expired · sign in again".into();
    }

    pub(crate) fn begin_reauth_connect(&mut self) -> Vec<AppCommand> {
        self.connect_intent = ConnectIntent::Reauth;
        let password = self.reauth.password.clone();
        let totp = self.reauth.totp.clone();
        self.login.password = password;
        self.login.totp = totp;
        self.pending_password = Some(format!(
            "{}{}",
            self.reauth.password,
            self.reauth.totp.trim()
        ));
        self.status = "Reconnecting…".into();
        vec![self.connect_command()]
    }

    #[must_use]
    pub fn styles(&self) -> mtui_ui::Styles {
        mtui_ui::Styles::from_palette(&self.theme.palette)
    }

    pub fn update(&mut self, event: AppEvent) -> Vec<AppCommand> {
        self.pull_console_logs();
        let cmds = match event {
            AppEvent::Input(key) => {
                let cmds = self.on_key(key);
                self.stamp(cmds)
            }
            AppEvent::Worker(msg) => {
                let id = msg.session();
                self.apply_to(id, |app| app.on_worker(msg))
            }
            AppEvent::Tick => self.on_tick(),
            AppEvent::Resize { width, height } => {
                self.terminal_width = width.max(1);
                self.terminal_height = height.max(1);
                let ids: Vec<SessionId> = self.sessions.iter().map(|session| session.id).collect();
                for id in ids {
                    let _ = self.apply_to(id, |app| {
                        app.palette.width = width.saturating_sub(4).min(64);
                        app.sync_table_viewport();
                        app.sync_console_viewport();
                        app.clamp_overlay_scroll();
                        Vec::new()
                    });
                }
                Vec::new()
            }
        };
        self.sync_activity();
        cmds
    }

    fn sync_activity(&mut self) {
        if self.loading || self.refreshing {
            self.activity_since.get_or_insert_with(Instant::now);
        } else {
            self.activity_since = None;
        }
    }

    #[must_use]
    pub fn show_activity(&self) -> bool {
        mtui_ui::activity_shown(self.activity_since, Instant::now())
    }

    fn on_tick(&mut self) -> Vec<AppCommand> {
        let ids: Vec<SessionId> = self.sessions.iter().map(|session| session.id).collect();
        let mut out = Vec::new();
        for id in ids {
            out.extend(self.apply_to(id, |app| {
                let mut cmds = app.reconnect_tick();
                cmds.extend(app.poll_tick_if_due());
                cmds
            }));
        }
        out
    }

    fn poll_tick_if_due(&mut self) -> Vec<AppCommand> {
        if self.screen != Screen::Main || self.client.is_none() || !self.session_ready() {
            return Vec::new();
        }
        match &self.overlay {
            Overlay::None | Overlay::Form(_) | Overlay::Torch(_) | Overlay::Probe(_) => {
                tracing::debug!(resource = self.current_resource.as_str(), "scheduled poll");
                self.poll_tick()
            }
            _ => Vec::new(),
        }
    }

    fn poll_tick(&mut self) -> Vec<AppCommand> {
        let generation = self.poll_generation;
        let mut cmds = if self.current_resource == DASHBOARD_ID {
            self.refreshing = true;
            vec![AppCommand::FetchDashboard {
                session: SessionId::UNSTAMPED,
                request_id: self.next_request(),
                generation,
            }]
        } else {
            let mut cmds = vec![AppCommand::FetchHeader {
                session: SessionId::UNSTAMPED,
                request_id: self.next_request(),
                generation,
            }];
            if resource_by_id(&self.current_resource).is_some_and(ResourceSpec::is_singleton) {
                self.refreshing = true;
                cmds.insert(
                    0,
                    AppCommand::FetchResource {
                        session: SessionId::UNSTAMPED,
                        request_id: self.next_request(),
                        generation,
                        resource_id: self.current_resource.clone(),
                    },
                );
            }
            cmds
        };
        cmds.extend(self.fetch_safe_mode_command());
        cmds
    }

    #[allow(clippy::too_many_lines)]
    fn on_worker(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        match msg {
            WorkerMsg::ProbeResult {
                fingerprint, error, ..
            } => {
                if let Some(err) = error {
                    self.screen = Screen::Login;
                    self.login.error = Some(classify_connect_error(ErrorKind::Tls, &err));
                    self.status = "Certificate probe failed".into();
                    return Vec::new();
                }
                self.trust_fingerprint = fingerprint;
                self.screen = Screen::Trust;
                self.status = "Certificate approval required".into();
                Vec::new()
            }
            WorkerMsg::Connected {
                client,
                router,
                error,
                error_kind,
                ..
            } => {
                if let Some(err) = error {
                    tracing::error!(error = %err, "connection failed");
                    let kind = error_kind.unwrap_or(ErrorKind::Transport);
                    let copy = classify_connect_error(kind, &err);
                    if self.connect_intent == ConnectIntent::Reconnect {
                        return self.on_reconnect_failed(kind, copy);
                    }
                    if self.connect_intent == ConnectIntent::Reauth {
                        self.reauth.error = Some(copy.clone());
                        self.status = copy;
                        return Vec::new();
                    }
                    self.screen = Screen::Login;
                    self.login.error = Some(copy.clone());
                    self.status = copy;
                    return Vec::new();
                }
                tracing::info!("connected");
                self.client = client;
                self.mark_live();
                if let Some(router) = router {
                    self.apply_system_resource(router);
                } else {
                    self.router = Resource::default();
                }
                self.login.error = None;
                self.persist_connected_session();
                if self.connect_intent == ConnectIntent::Reauth
                    || self.connect_intent == ConnectIntent::Reconnect
                {
                    self.connect_intent = ConnectIntent::Login;
                    self.overlay = Overlay::None;
                    self.reauth = ReauthState::default();
                    self.screen = Screen::Main;
                    self.status = "Reconnected".into();
                    let mut cmds = self.poll_current();
                    cmds.extend(self.fetch_packages_command());
                    cmds.extend(self.fetch_menu_paths_command());
                    cmds.extend(self.fetch_access_command());
                    cmds.extend(self.on_reconnect_safe_mode());
                    return cmds;
                }
                self.screen = Screen::Main;
                self.status = format!("Connected · {}", self.profile_label());
                let start = self
                    .nav
                    .first_openable_id()
                    .unwrap_or_else(|| DASHBOARD_ID.to_string());
                self.select_resource(&start);
                let mut cmds = self.poll_current();
                cmds.extend(self.fetch_packages_command());
                cmds.extend(self.fetch_menu_paths_command());
                cmds.extend(self.fetch_access_command());
                cmds.extend(self.fetch_safe_mode_command());
                cmds
            }
            WorkerMsg::AuthRequired { message, .. } => {
                if self.screen == Screen::Main {
                    self.open_reauth(message);
                }
                Vec::new()
            }
            WorkerMsg::SessionLost {
                generation, reason, ..
            } => self.mark_session_lost(generation, reason),
            WorkerMsg::AccessResult {
                generation,
                users,
                groups,
                error,
                ..
            } => {
                if generation != self.poll_generation {
                    return Vec::new();
                }
                self.apply_access(&users, &groups, error.as_deref());
                Vec::new()
            }
            WorkerMsg::MenuPathsResult {
                generation,
                missing_ids,
                error,
                ..
            } => self.apply_menu_paths_result(generation, missing_ids, error),
            WorkerMsg::ResourceResult {
                request_id,
                generation,
                resource_id,
                rows,
                error,
                ..
            } => {
                if generation != self.poll_generation
                    || request_id < self.request_id.saturating_sub(1)
                {
                    // Stale — still accept if it's the latest for this generation and resource.
                }
                if generation != self.poll_generation {
                    return Vec::new();
                }
                if resource_id == "packages" {
                    self.apply_installed_packages(&rows);
                    if self.current_resource != "packages" {
                        return Vec::new();
                    }
                }
                if resource_id != self.current_resource {
                    return Vec::new();
                }
                let announce = self.loading || self.refreshing;
                self.loading = false;
                self.refreshing = false;
                if let Some(err) = error {
                    if is_missing_command_prefix(&err) {
                        return self.hide_missing_path_resource(&resource_id);
                    }
                    tracing::warn!(resource_id = resource_id.as_str(), error = %err, "resource refresh failed");
                    self.status = if mtui_core::is_permission_trap(&err) {
                        Self::classify_write_error(&err)
                    } else {
                        format!("Refresh failed: {err}")
                    };
                    return Vec::new();
                }
                self.note_data_ok();
                if resource_id == "history" {
                    self.note_history_rows(&rows);
                }
                let loaded = resource_loaded_message(&resource_id, &rows);
                tracing::debug!(
                    resource_id = resource_id.as_str(),
                    id = loaded_entity_id(&rows).unwrap_or(""),
                    rows = rows.len(),
                    "{}",
                    loaded
                );
                if resource_id == "logs" {
                    self.ingest_logs(rows);
                } else {
                    let selected_id = self
                        .table
                        .selected_row()
                        .and_then(|row| row.get(".id").cloned());
                    self.apply_table_rows(self.row_to_display(rows));
                    if let Some(id) = selected_id {
                        self.table.select_id(&id);
                    }
                }
                if announce {
                    self.status = resource_id;
                }
                self.hydrate_selected_typed_interface()
            }
            WorkerMsg::DashboardResult {
                generation,
                cpu,
                cpu_error,
                system,
                system_error,
                interfaces,
                interface_error,
                firewall,
                firewall_error,
                ..
            } => {
                if generation != self.poll_generation || self.current_resource != DASHBOARD_ID {
                    return Vec::new();
                }
                let announce = self.loading || self.refreshing;
                self.loading = false;
                self.refreshing = false;
                self.apply_dashboard(
                    &cpu,
                    cpu_error.as_deref(),
                    system.as_ref(),
                    system_error.as_deref(),
                    &interfaces,
                    interface_error.as_deref(),
                    &firewall,
                    firewall_error.as_deref(),
                    announce,
                );
                if cpu_error.is_none() || system_error.is_none() || interface_error.is_none() {
                    self.note_data_ok();
                }
                Vec::new()
            }
            WorkerMsg::HeaderResult {
                generation,
                system,
                interfaces,
                interface_error,
                ..
            } => {
                if generation != self.poll_generation || self.screen != Screen::Main {
                    return Vec::new();
                }
                if system.is_some() || interface_error.is_none() {
                    self.note_data_ok();
                }
                self.apply_header_telemetry(system, &interfaces, interface_error.as_deref());
                Vec::new()
            }
            WorkerMsg::MutateResult { .. } => self.apply_mutate_result(msg),
            WorkerMsg::SafeModeResult { .. } => self.apply_safe_mode_result(msg),
            WorkerMsg::TorchResult {
                generation,
                rows,
                error,
                done,
                ..
            } => self.apply_torch_result(generation, rows, error, done),
            WorkerMsg::ReadLocalFileResult { .. } => self.apply_read_local_file(msg),
            WorkerMsg::WriteLocalFileResult { .. } => self.apply_write_local_file(msg),
            WorkerMsg::RecordResult { .. } => self.apply_record_result(msg),
            WorkerMsg::PingTraceResult {
                generation,
                rows,
                error,
                done,
                ..
            } => self.apply_probe_result(generation, rows, error, done),
            WorkerMsg::LookupResult {
                request_id,
                generation,
                options,
                error,
                ..
            } => {
                if let Overlay::Form(session) = &mut self.overlay {
                    session.apply_lookup_result(request_id, generation, options, error);
                }
                Vec::new()
            }
            WorkerMsg::FormRecordResult {
                request_id,
                generation,
                resource_id,
                id,
                fields,
                error,
                ..
            } => self.apply_form_record(request_id, generation, &resource_id, &id, fields, error),
            WorkerMsg::ListLocalDirResult {
                generation,
                dir,
                entries,
                error,
                ..
            } => {
                if let Overlay::FilePicker(picker) = &mut self.overlay {
                    picker.apply_listing(generation, dir, entries, error);
                }
                Vec::new()
            }
            WorkerMsg::ListenDelta {
                generation,
                resource_id,
                row,
                ..
            } => {
                if generation != self.poll_generation || resource_id != self.current_resource {
                    return Vec::new();
                }
                if resource_id == "logs" {
                    self.ingest_logs(vec![row]);
                    return Vec::new();
                }
                let selected_id = self
                    .table
                    .selected_row()
                    .and_then(|row| row.get(".id").cloned());
                let offset = self.inspector.offset;
                let inspector_selected = self.inspector.selected;
                let mut resources: Vec<Resource> = self
                    .table
                    .rows
                    .iter()
                    .map(|display| {
                        let mut fields = display.clone();
                        let id = fields.remove(".id").unwrap_or_default();
                        Resource { id, fields }
                    })
                    .collect();
                merge_listen_record(&mut resources, row);
                self.apply_table_rows(self.row_to_display(resources));
                if let Some(id) = selected_id {
                    self.table.select_id(&id);
                }
                self.inspector.selected = inspector_selected;
                self.inspector.offset = offset;
                let visible = self.inspector_visible_rows();
                self.inspector.clamp_to_visible(visible);
                self.note_data_ok();
                self.hydrate_selected_typed_interface()
            }
            WorkerMsg::WanSample {
                generation,
                interface,
                sample,
                ..
            } => {
                if generation != self.poll_generation || self.screen != Screen::Main {
                    return Vec::new();
                }
                self.dash.update_wan_monitor(&interface, &sample);
                Vec::new()
            }
        }
    }

    /// Header identity: board and host, matching the Deck mock (`CCR2004 · 192.0.2.1`).
    #[must_use]
    pub fn session_identity(&self) -> String {
        let board = nonempty_field(&self.router, "board-name").unwrap_or("RouterOS");
        let host = header_host(&self.login.url);
        if host.is_empty() {
            board.to_string()
        } else {
            format!("{board} · {host}")
        }
    }

    /// Live header metrics: CPU, memory, and WAN rate.
    #[must_use]
    pub fn header_signals(&self) -> Vec<Signal> {
        let mut signals = self.link_signals();
        signals.extend([self.cpu_signal(), self.memory_signal(), self.wan_signal()]);
        signals
    }

    fn cpu_signal(&self) -> Signal {
        match self.cpu_percent() {
            Some(load) => {
                let level = percent_signal_level(load);
                let level = if matches!(level, SignalLevel::Good) {
                    SignalLevel::Idle
                } else {
                    level
                };
                Signal::new("CPU", format!("{load:.0}%"), level)
            }
            None => Signal::new("CPU", "—", SignalLevel::Idle),
        }
    }

    fn memory_signal(&self) -> Signal {
        let used = self.dash.memory_used_bytes;
        let total = self.dash.memory_total_bytes;
        if total == 0 {
            return Signal::new("MEM", "—", SignalLevel::Idle);
        }
        let percent = memory_percent(used, total);
        let level = percent_signal_level(percent);
        let level = if matches!(level, SignalLevel::Good) {
            SignalLevel::Idle
        } else {
            level
        };
        Signal::new("MEM", format!("{percent:.0}%"), level)
    }

    fn wan_signal(&self) -> Signal {
        if !self.dash.traffic_has_base {
            return Signal::new("WAN", "—", SignalLevel::Idle);
        }
        Signal::new(
            "WAN",
            format_rate(self.dash.traffic_rx_rate),
            SignalLevel::Good,
        )
    }

    fn cpu_percent(&self) -> Option<f64> {
        if let Some(load) = nonempty_field(&self.router, "cpu-load").and_then(parse_percent) {
            return Some(load);
        }
        if self.dash.cpu_core_loads.is_empty() {
            return None;
        }
        let total: f64 = self.dash.cpu_core_loads.values().copied().sum();
        let count = f64::from(u32::try_from(self.dash.cpu_core_loads.len()).unwrap_or(1));
        Some(total / count.max(1.0))
    }

    pub(crate) fn poll_current(&mut self) -> Vec<AppCommand> {
        let generation = self.poll_generation;
        let mut cmds = if self.current_resource == DASHBOARD_ID {
            vec![AppCommand::FetchDashboard {
                session: SessionId::UNSTAMPED,
                request_id: self.next_request(),
                generation,
            }]
        } else {
            vec![
                AppCommand::FetchResource {
                    session: SessionId::UNSTAMPED,
                    request_id: self.next_request(),
                    generation,
                    resource_id: self.current_resource.clone(),
                },
                AppCommand::FetchHeader {
                    session: SessionId::UNSTAMPED,
                    request_id: self.next_request(),
                    generation,
                },
            ]
        };
        cmds.extend(self.fetch_safe_mode_command());
        cmds
    }

    pub(crate) fn next_request(&mut self) -> u64 {
        self.request_id = self.request_id.wrapping_add(1);
        self.request_id
    }

    pub(crate) fn copy_current_view(&mut self) -> Vec<AppCommand> {
        match self.pane {
            Pane::Content => {
                if let Some(row) = self.table.selected_row() {
                    let text = format_row_for_copy(row);
                    vec![AppCommand::CopyToClipboard {
                        session: SessionId::UNSTAMPED,
                        text,
                    }]
                } else {
                    Vec::new()
                }
            }
            Pane::Inspector => {
                let text = format_inspector_for_copy(&self.inspector);
                vec![AppCommand::CopyToClipboard {
                    session: SessionId::UNSTAMPED,
                    text,
                }]
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn copy_filtered_table(&mut self) -> Vec<AppCommand> {
        let rows = self.table.visible_rows();
        if rows.is_empty() {
            self.status = "Nothing to copy".into();
            return Vec::new();
        }
        let text = rows
            .into_iter()
            .map(format_row_for_copy)
            .collect::<Vec<_>>()
            .join("\n\n");
        vec![AppCommand::CopyToClipboard {
            session: SessionId::UNSTAMPED,
            text,
        }]
    }

    pub(crate) fn enter_demo(&mut self) -> Vec<AppCommand> {
        let store = DemoStore::new();
        self.apply_system_resource(store.system());
        self.apply_installed_packages(&store.rows("packages"));
        self.demo = Some(store);
        self.client = None;
        self.login.name = DEMO_PROFILE_NAME.into();
        self.login.url = DEMO_URL.into();
        self.login.username = "demo".into();
        self.login.password.clear();
        self.login.totp.clear();
        self.login.uses_totp = false;
        self.login.remember_password = false;
        self.login.error = None;
        self.current_profile = DEMO_PROFILE_NAME.into();
        self.connect_intent = ConnectIntent::Login;
        self.access = mtui_core::SessionAccess::full("demo", "full");
        self.mark_live();
        self.screen = Screen::Main;
        self.status = "Demo profile · fixture data, no router".into();
        let start = self
            .nav
            .first_openable_id()
            .unwrap_or_else(|| DASHBOARD_ID.to_string());
        self.select_resource(&start);
        self.poll_current()
    }

    fn fetch_packages_command(&mut self) -> Vec<AppCommand> {
        if self.current_resource == "packages" {
            return Vec::new();
        }
        vec![AppCommand::FetchResource {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
            resource_id: "packages".into(),
        }]
    }

    fn fetch_menu_paths_command(&mut self) -> Vec<AppCommand> {
        self.menu_paths_generation = self.menu_paths_generation.wrapping_add(1);
        vec![AppCommand::ProbeMenuPaths {
            session: SessionId::UNSTAMPED,
            generation: self.menu_paths_generation,
        }]
    }

    fn apply_menu_paths_result(
        &mut self,
        generation: u64,
        missing_ids: HashSet<String>,
        error: Option<String>,
    ) -> Vec<AppCommand> {
        if generation != self.menu_paths_generation {
            return Vec::new();
        }
        if let Some(err) = error {
            tracing::debug!(error = %err, "menu path probe failed; keeping package gates");
            return Vec::new();
        }
        self.missing_path_ids = missing_ids;
        self.refresh_unavailable_menus();
        self.leave_if_current_unavailable()
    }

    fn hide_missing_path_resource(&mut self, resource_id: &str) -> Vec<AppCommand> {
        tracing::info!(
            resource_id,
            "hiding menu; command path is absent on this device"
        );
        self.missing_path_ids.insert(resource_id.to_string());
        self.refresh_unavailable_menus();
        if self.current_resource != resource_id {
            return Vec::new();
        }
        let cmds = self.leave_if_current_unavailable();
        if cmds.is_empty() {
            self.status = format!("{resource_id} is not available on this device");
        }
        cmds
    }

    fn leave_if_current_unavailable(&mut self) -> Vec<AppCommand> {
        if !self.nav.unavailable.contains_key(&self.current_resource) {
            return Vec::new();
        }
        let start = self
            .nav
            .first_openable_id()
            .unwrap_or_else(|| DASHBOARD_ID.to_string());
        self.select_resource(&start);
        self.status = "Hidden menus this device does not provide".into();
        self.poll_current()
    }

    fn apply_installed_packages(&mut self, rows: &[Resource]) {
        self.installed_packages =
            installed_package_names(rows.iter().map(|row| row.fields.clone()));
        self.refresh_unavailable_menus();
    }

    fn refresh_unavailable_menus(&mut self) {
        let arch = self
            .router
            .fields
            .get("architecture-name")
            .cloned()
            .unwrap_or_default();
        let cpu = self.router.fields.get("cpu").cloned().unwrap_or_default();
        let packages = unavailable_menus_for_device(&self.installed_packages, &arch, &cpu);
        let paths = self
            .missing_path_ids
            .iter()
            .map(|id| (id.clone(), MISSING_PATH_REASON.to_string()))
            .collect();
        let missing = merge_unavailable_menus(packages, paths);
        self.nav.set_unavailable(missing);
        self.rebuild_palette();
    }

    pub(crate) fn select_resource(&mut self, id: &str) {
        tracing::trace!(resource_id = id, "opened pane");
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.torch_generation = self.torch_generation.wrapping_add(1);
        self.probe_generation = self.probe_generation.wrapping_add(1);
        self.overlay = Overlay::None;
        self.current_resource = id.to_string();
        self.refreshing = false;
        let _ = self.nav.select_id(id);
        if id == DASHBOARD_ID {
            self.activate_dashboard();
        } else {
            self.loading = true;
        }
        if !self.session_ready() {
            self.loading = false;
            self.status = self.link_status_message();
        }
        if let Some(spec) = resource_by_id(id) {
            self.table = TableState::new(spec.columns);
        } else {
            self.table = TableState::new(&[]);
        }
        self.sync_table_viewport();
        self.inspector = InspectorState::default();
    }

    fn activate_dashboard(&mut self) {
        self.dash.activate();
        self.loading = self.dash.traffic_samples.is_empty();
        self.status = if self.dash.traffic_samples.is_empty() {
            "Detecting active WAN interface…".into()
        } else {
            "Resuming recent WAN telemetry…".into()
        };
    }

    pub(crate) fn dashboard_inner_size(&self) -> (usize, usize) {
        self.content_inner_size(false)
    }

    pub(crate) fn table_inner_size(&self) -> (usize, usize) {
        self.content_inner_size(true)
    }

    fn content_inner_size(&self, include_inspector: bool) -> (usize, usize) {
        let metrics = LayoutMetrics::new(self.terminal_width, self.terminal_height);
        let inspector = if include_inspector {
            metrics.inspector_width
        } else {
            0
        };
        let inner_w = self
            .terminal_width
            .saturating_sub(metrics.nav_width)
            .saturating_sub(inspector)
            .saturating_sub(4)
            .max(1);
        let band = chrome_band_height(self.terminal_height);
        let inner_h = self
            .terminal_height
            .saturating_sub(tab_strip_height(self.terminal_height))
            .saturating_sub(band)
            .saturating_sub(band)
            .saturating_sub(2)
            .saturating_sub(self.console_layout_height())
            .max(1);
        (usize::from(inner_w), usize::from(inner_h))
    }

    pub(crate) fn console_layout_height(&self) -> u16 {
        console_pane_height(
            self.terminal_height,
            self.console.visible,
            self.console.fullscreen,
        )
    }

    pub(crate) fn sync_table_viewport(&mut self) {
        let (width, height) = self.table_inner_size();
        self.table.sync_viewport(width, height);
        self.nav.sync_viewport(height);
        let visible = self.inspector_visible_rows();
        self.inspector.clamp_to_visible(visible);
        self.sync_console_viewport();
    }

    pub(crate) fn sync_console_viewport(&mut self) {
        let height = usize::from(self.console_layout_height().saturating_sub(2).max(1));
        let id = self.active;
        let session = self.session_mut(id).expect("active session must exist");
        session
            .console
            .ensure_visible(&session.console_entries, height);
    }

    pub(crate) fn pull_console_logs(&mut self) {
        let records = self.log_store.snapshot();
        if records.is_empty() {
            return;
        }
        let last_id = records.last().map_or(0, |record| record.id);
        if last_id == self.console_log_seq && self.console_entries.len() == records.len() {
            return;
        }
        let filtered_before = self.console.filtered_indices(&self.console_entries).len();
        let follow = filtered_before == 0 || self.console.selected + 1 >= filtered_before;
        self.console_entries = records.iter().map(console_entry_from_record).collect();
        self.console_log_seq = last_id;
        let filtered_len = self.console.filtered_indices(&self.console_entries).len();
        if follow {
            self.console.select_last(filtered_len);
        } else {
            self.console.clamp_selection(filtered_len);
        }
        self.sync_console_viewport();
    }

    pub(crate) fn toggle_console(&mut self) {
        self.pull_console_logs();
        let showing = self.console.toggle_visible();
        if showing {
            if self.pane != Pane::Console {
                self.pane_before_console = self.pane;
            }
            self.pane = Pane::Console;
            tracing::trace!(fullscreen = self.console.fullscreen, "opened pane");
        } else {
            self.pane = match self.pane_before_console {
                Pane::Console => Pane::Content,
                other => other,
            };
            tracing::trace!("closed console pane");
        }
        self.status = if showing {
            "Console shown".into()
        } else {
            "Console hidden".into()
        };
        self.sync_table_viewport();
    }

    pub(crate) fn cycle_pane(&mut self, forward: bool) {
        if self.console.fullscreen {
            return;
        }
        let console = self.console.visible;
        self.pane = if forward {
            match self.pane {
                Pane::Nav => Pane::Content,
                Pane::Content => Pane::Inspector,
                Pane::Inspector if console => Pane::Console,
                Pane::Inspector | Pane::Console => Pane::Nav,
            }
        } else {
            match self.pane {
                Pane::Nav if console => Pane::Console,
                Pane::Nav | Pane::Console => Pane::Inspector,
                Pane::Content => Pane::Nav,
                Pane::Inspector => Pane::Content,
            }
        };
        if self.pane == Pane::Console {
            tracing::trace!(pane = "console", "focused pane");
        }
    }

    /// Move among the visible nav / content / inspector panes, clamping at the ends.
    pub(crate) fn shift_main_pane(&mut self, forward: bool) {
        if self.console.fullscreen {
            return;
        }
        let panes = self.visible_main_panes();
        let Some(idx) = panes.iter().position(|&pane| pane == self.pane) else {
            return;
        };
        let next = if forward {
            idx.saturating_add(1).min(panes.len().saturating_sub(1))
        } else {
            idx.saturating_sub(1)
        };
        if let Some(pane) = panes.get(next).copied() {
            self.pane = pane;
        }
    }

    fn visible_main_panes(&self) -> Vec<Pane> {
        let metrics = LayoutMetrics::new(self.terminal_width, self.terminal_height);
        let mut panes = Vec::new();
        if metrics.nav_width > 0 {
            panes.push(Pane::Nav);
        }
        panes.push(Pane::Content);
        if metrics.inspector_width > 0 && self.current_resource != DASHBOARD_ID {
            panes.push(Pane::Inspector);
        }
        panes
    }

    pub(crate) fn console_body_height(&self) -> usize {
        usize::from(self.console_layout_height().saturating_sub(2).max(1))
    }

    pub(crate) fn inspector_visible_rows(&self) -> usize {
        self.table_inner_size().1.max(1)
    }

    pub(crate) fn scroll_firewall(&mut self, delta: isize) {
        let max_offset = self.firewall_max_offset();
        let next = isize::try_from(self.dash.firewall_offset).unwrap_or(0) + delta;
        self.dash.firewall_offset =
            usize::try_from(next.clamp(0, isize::try_from(max_offset).unwrap_or(0))).unwrap_or(0);
    }

    pub(crate) fn scroll_firewall_to(&mut self, offset: usize) {
        self.dash.firewall_offset = offset.min(self.firewall_max_offset());
    }

    fn firewall_max_offset(&self) -> usize {
        let (width, height) = self.dashboard_inner_size();
        let geo = DashboardGeometry::new(width, height, self.dash.cpu_core_order.len());
        FirewallHitChart {
            rules: &self.dash.firewall_rules,
            width,
            height: geo.firewall_height.max(1),
            offset: self.dash.firewall_offset,
        }
        .max_offset()
    }

    pub(crate) fn firewall_page_size(&self) -> usize {
        let (width, height) = self.dashboard_inner_size();
        let geo = DashboardGeometry::new(width, height, self.dash.cpu_core_order.len());
        geo.firewall_height.saturating_sub(1).max(1)
    }

    fn ingest_logs(&mut self, rows: Vec<Resource>) {
        for row in rows {
            let key = if row.id.is_empty() {
                format!(
                    "{}|{}|{}",
                    row.field("time").unwrap_or(""),
                    row.field("topics").unwrap_or(""),
                    row.field("message").unwrap_or("")
                )
            } else {
                row.id.clone()
            };
            if !self.log_seen.insert(key) {
                continue;
            }
            if !self.log_follow {
                self.log_unread = self.log_unread.saturating_add(1);
            }
            self.log_buffer.push_back(row);
            while self.log_buffer.len() > LOG_BUFFER_CAP {
                if let Some(old) = self.log_buffer.pop_front() {
                    let old_key = if old.id.is_empty() {
                        format!(
                            "{}|{}|{}",
                            old.field("time").unwrap_or(""),
                            old.field("topics").unwrap_or(""),
                            old.field("message").unwrap_or("")
                        )
                    } else {
                        old.id.clone()
                    };
                    self.log_seen.remove(&old_key);
                }
            }
        }
        if !self.log_paused {
            self.rebuild_log_table();
        }
    }

    pub(crate) fn rebuild_log_table(&mut self) {
        let sev = self.log_severity;
        let rows: Vec<_> = self
            .log_buffer
            .iter()
            .filter(|r| match sev {
                LogSeverity::All => true,
                other => r
                    .field("topics")
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(other.label()),
            })
            .map(Resource::masked_fields)
            .collect();
        let previous = self.table.selected;
        self.table.set_rows(rows);
        if self.log_follow && self.table.row_count() > 0 {
            self.table.select_last();
        }
        self.sync_table_viewport();
        self.refresh_inspector(self.table.selected == previous && !self.log_follow);
    }

    fn apply_table_rows(&mut self, rows: Vec<Row>) {
        let previous = self.table.selected;
        self.table.set_rows(rows);
        self.sync_table_viewport();
        self.refresh_inspector(self.table.selected == previous);
    }

    pub(crate) fn refresh_inspector(&mut self, preserve_offset: bool) {
        let offset = self.inspector.offset;
        let selected = self.inspector.selected;
        let spec = resource_by_id(&self.current_resource);
        let typed = spec.filter(|item| item.id == "interfaces").and_then(|_| {
            self.table
                .selected_row()
                .and_then(|row| row.get("type"))
                .and_then(|iface_type| edit_resource_for_interface_type(iface_type))
                .and_then(resource_by_id)
        });
        let (schema_id, schema) = typed.map_or_else(
            || (spec.map(|item| item.id), spec.and_then(|item| item.form)),
            |item| (Some(item.id), item.form),
        );
        self.inspector =
            InspectorState::from_row_with_schema_for(self.table.selected_row(), schema_id, schema);
        if preserve_offset {
            self.inspector.selected = selected;
            self.inspector.offset = offset;
            let visible = self.inspector_visible_rows();
            self.inspector.clamp_to_visible(visible);
        }
    }

    fn apply_form_record(
        &mut self,
        request_id: u64,
        generation: u64,
        resource_id: &str,
        id: &str,
        fields: Option<HashMap<String, String>>,
        error: Option<String>,
    ) -> Vec<AppCommand> {
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Overlay::Form(session) = &mut self.overlay
            && session.hydrate_request_id == Some(request_id)
            && session.resource_id == resource_id
            && session.record_id == id
        {
            session.hydrate_request_id = None;
            if let Some(err) = error {
                session.error = Some(err);
            } else if let Some(ref row) = fields
                && let Some(schema) = resource_by_id(resource_id).and_then(|spec| spec.form)
            {
                session.absorb_record(row, schema);
                session.clamp(schema);
            }
        }
        if let Some(row) = fields {
            let selected = self
                .table
                .selected_row()
                .and_then(|row| row.get(".id"))
                .map(String::as_str)
                == Some(id);
            for table_row in &mut self.table.rows {
                if table_row.get(".id").map(String::as_str) == Some(id) {
                    for (key, value) in row {
                        if key != ".id" {
                            table_row.insert(key, value);
                        }
                    }
                    break;
                }
            }
            if selected {
                self.refresh_inspector(true);
            }
        }
        Vec::new()
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_dashboard(
        &mut self,
        cpu: &[Resource],
        cpu_error: Option<&str>,
        system: Option<&Resource>,
        system_error: Option<&str>,
        interfaces: &[Resource],
        interface_error: Option<&str>,
        firewall: &[Resource],
        firewall_error: Option<&str>,
        announce: bool,
    ) {
        if let Some(system) = system {
            self.apply_system_resource(system.clone());
        }
        if cpu_error.is_none() {
            self.dash.update_cpu(cpu, system);
        } else if system_error.is_none() {
            self.dash.update_cpu(&[], system);
        }
        if firewall_error.is_none() {
            self.dash.update_firewall(firewall);
        }

        if let Some(err) = interface_error {
            if announce {
                self.status = format!("System telemetry live · WAN {err} · retrying");
            }
            return;
        }
        match select_wan_interface(interfaces) {
            Ok(iface) => {
                self.dash.update_wan(iface, Instant::now());
                let mut unavailable = Vec::new();
                if system_error.is_some() {
                    unavailable.push("CPU/memory");
                }
                if firewall_error.is_some() {
                    unavailable.push("firewall");
                }
                if announce {
                    self.status = if unavailable.is_empty() {
                        format!("WAN telemetry live · {}", self.dash.traffic_interface)
                    } else {
                        format!("WAN live · {} telemetry retrying", unavailable.join(" + "))
                    };
                }
            }
            Err(err) => {
                if announce {
                    self.status = format!("System telemetry live · WAN {err} · retrying");
                }
            }
        }
    }

    fn apply_system_resource(&mut self, system: Resource) {
        let prev_arch = self.router.fields.get("architecture-name").cloned();
        let prev_cpu = self.router.fields.get("cpu").cloned();
        self.dash.update_system(&system);
        self.router = system;
        let arch = self.router.fields.get("architecture-name");
        let cpu = self.router.fields.get("cpu");
        if arch != prev_arch.as_ref() || cpu != prev_cpu.as_ref() {
            self.refresh_unavailable_menus();
        }
    }

    fn apply_header_telemetry(
        &mut self,
        system: Option<Resource>,
        interfaces: &[Resource],
        interface_error: Option<&str>,
    ) {
        if let Some(system) = system {
            self.apply_system_resource(system);
        }
        if interface_error.is_some() {
            return;
        }
        if let Ok(iface) = select_wan_interface(interfaces) {
            self.dash.update_wan(iface, Instant::now());
        }
    }

    fn named_profile(&self) -> Option<Profile> {
        let name = self.current_profile.as_str();
        if name.is_empty() {
            return None;
        }
        self.profiles
            .load()
            .ok()
            .and_then(|list| list.into_iter().find(|profile| profile.name == name))
    }

    pub(crate) fn rebuild_palette(&mut self) {
        self.palette.commands = palette_commands_filtered(
            &self.nav.hidden,
            &self.nav.unavailable,
            self.nav.show_hidden,
        );
    }

    fn persist_nav_hidden(&mut self) {
        if cfg!(test) {
            return;
        }
        let Some(mut profile) = self.named_profile() else {
            return;
        };
        profile.set_hidden_nav_ids(self.nav.hidden.iter().cloned());
        let _ = self.profiles.upsert(profile);
    }

    pub(crate) fn toggle_show_hidden_menus(&mut self) {
        let showing = self.nav.toggle_show_hidden();
        self.rebuild_palette();
        self.status = if showing {
            "Showing hidden menus · − restore · . done".into()
        } else if self.nav.hidden.is_empty() {
            "No menus are hidden".into()
        } else {
            "Hidden menus tucked away".into()
        };
    }

    pub(crate) fn toggle_selected_nav_hidden(&mut self) {
        let Some(id) = self.nav.selected_id().map(str::to_owned) else {
            return;
        };
        let label = self
            .nav
            .entries
            .get(self.nav.selected)
            .map_or_else(|| id.clone(), |entry| entry.label.clone());
        let already_hidden = self
            .nav
            .entries
            .get(self.nav.selected)
            .is_some_and(|entry| entry.hidden);
        if already_hidden {
            self.apply_toggle_hidden(&id, &label);
            return;
        }
        if self.nav.would_hide_last_leaf(&id) {
            self.status = "Keep at least one menu visible".into();
            return;
        }
        let is_group = self
            .nav
            .entries
            .get(self.nav.selected)
            .is_some_and(|entry| entry.is_group);
        let body = if let Some(parent_id) = self.nav.hide_collapses_parent(&id) {
            let parent = self.nav.label_of(parent_id).unwrap_or(parent_id);
            format!(
                "Hide {label} from the sidebar?\n\n{parent} will hide too because no screens would remain."
            )
        } else if is_group {
            format!("Hide {label} and its screens from the sidebar?")
        } else {
            format!("Hide {label} from the sidebar?")
        };
        self.overlay = Overlay::HideMenu {
            id,
            title: format!("Hide {label}"),
            body,
        };
        tracing::trace!(overlay = "hide-menu", "opened pane");
    }

    pub(crate) fn confirm_hide_menu(&mut self) {
        let Overlay::HideMenu { id, title, .. } = &self.overlay else {
            return;
        };
        let id = id.clone();
        let label = title.strip_prefix("Hide ").unwrap_or(title).to_string();
        self.overlay = Overlay::None;
        self.apply_toggle_hidden(&id, &label);
    }

    fn apply_toggle_hidden(&mut self, id: &str, label: &str) {
        match self.nav.toggle_hidden(id) {
            ToggleHidden::Hidden => {
                self.status = format!("Hidden {label}");
                self.rebuild_palette();
                self.persist_nav_hidden();
            }
            ToggleHidden::Restored => {
                self.status = format!("Restored {label}");
                self.rebuild_palette();
                self.persist_nav_hidden();
            }
            ToggleHidden::LastVisible => {
                self.status = "Keep at least one menu visible".into();
            }
        }
    }

    pub(crate) fn reset_hidden_menus(&mut self) {
        self.nav.set_hidden_ids(Vec::new());
        self.nav.set_show_hidden(false);
        self.rebuild_palette();
        self.persist_nav_hidden();
        self.status = "All menus restored".into();
    }
}

pub(crate) fn palette_commands() -> Vec<Command> {
    palette_commands_filtered(&HashSet::new(), &HashMap::new(), true)
}

fn palette_commands_filtered(
    hidden: &HashSet<String>,
    unavailable: &HashMap<String, String>,
    show_hidden: bool,
) -> Vec<Command> {
    let show_title = if show_hidden {
        "Done showing hidden menus"
    } else {
        "Show hidden menus"
    };
    let mut commands = vec![
        Command::new("refresh", "Refresh").with_description("reload the current resource"),
        Command::new("logout", "Log out").with_description("disconnect and keep saved devices"),
        Command::new("switch-device", "Switch device")
            .with_description("return to the router list without deleting profiles"),
        Command::new("forget-device", "Forget this device")
            .with_description("delete this profile and its remembered password"),
        Command::new("help", "Keyboard help").with_description("show all shortcuts"),
        Command::new("about", "About this screen")
            .with_description("RouterOS summary for the open menu"),
        Command::new("console", "Toggle console")
            .with_description("show or hide the application log console"),
        Command::new("show-hidden-menus", show_title)
            .with_description("reveal tucked-away sidebar items so they can be restored"),
        Command::new("reset-hidden-menus", "Restore all menus")
            .with_description("put every hidden sidebar item back"),
        Command::new("dashboard", "Dashboard").with_description("live WAN overview"),
        Command::new("safe-mode", "Toggle Safe Mode").with_description(
            "take or release Safe Mode on this tab so a dropped session unrolls edits",
        ),
        Command::new("safe-mode-unroll", "Unroll Safe Mode")
            .with_description("undo changes tagged while Safe Mode was on"),
    ];
    commands.extend(ALL_RESOURCES.iter().map(|spec| {
        Command::new(spec.id, spec.cli_path())
            .with_description(spec.label)
            .with_path(spec.cli_path())
    }));
    commands
        .into_iter()
        .filter(|command| {
            !palette_target_unavailable(&command.id, unavailable)
                && (show_hidden || !palette_target_user_hidden(&command.id, hidden))
        })
        .collect()
}

fn palette_target_user_hidden(id: &str, hidden: &HashSet<String>) -> bool {
    if hidden.contains(id) {
        return true;
    }
    resource_by_id(id).is_some_and(|spec| hidden.contains(spec.group))
}

fn palette_target_unavailable(id: &str, unavailable: &HashMap<String, String>) -> bool {
    if unavailable.contains_key(id) {
        return true;
    }
    resource_by_id(id).is_some_and(|spec| unavailable.contains_key(spec.group))
}

fn nonempty_field<'src>(resource: &'src Resource, key: &str) -> Option<&'src str> {
    resource
        .field(key)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_router_url(url: &str, use_tls: bool) -> String {
    migrate_connection_target_for(url, use_tls)
}

pub(crate) fn is_router_target(url: &str) -> bool {
    parse_connection_target(url, "login").is_ok()
}

pub(crate) fn classify_connect_error(kind: ErrorKind, message: &str) -> String {
    match kind {
        ErrorKind::Auth => {
            "Wrong username, password, or TOTP. The router rejected this login.".into()
        }
        ErrorKind::Tls => {
            "TLS or certificate mismatch. Confirm the fingerprint, CA file, or OS-trusted CA."
                .into()
        }
        ErrorKind::Transport | ErrorKind::Timeout => {
            "Cannot reach the API. Check the host, port (8729 for api-ssl, 8728 for api), and that the service is enabled.".into()
        }
        ErrorKind::Canceled => "Connection canceled.".into(),
        _ => {
            if message.to_ascii_lowercase().contains("cannot log in")
                || message.to_ascii_lowercase().contains("invalid user")
            {
                "Wrong username, password, or TOTP. The router rejected this login.".into()
            } else {
                message.to_string()
            }
        }
    }
}

fn suggested_profile_name(url: &str, username: &str, taken: &[String]) -> String {
    let host = header_host(url);
    let host = if host.is_empty() {
        "router".to_string()
    } else {
        host
    };
    let base = if username.is_empty() {
        host
    } else {
        format!("{host} · {username}")
    };
    if !taken.iter().any(|name| name == &base) {
        return base;
    }
    for index in 2..100 {
        let candidate = format!("{base} {index}");
        if !taken.iter().any(|name| name == &candidate) {
            return candidate;
        }
    }
    format!("{base} {}", taken.len().saturating_add(1))
}

fn loaded_entity_id(rows: &[Resource]) -> Option<&str> {
    match rows {
        [row] => {
            let id = row.id.trim();
            (!id.is_empty()).then_some(id)
        }
        _ => None,
    }
}

fn resource_loaded_message(resource_id: &str, rows: &[Resource]) -> String {
    match loaded_entity_id(rows) {
        Some(id) => format!("resource loaded {resource_id} {id}"),
        None => format!("resource loaded {resource_id}"),
    }
}

fn console_entry_from_record(record: &LogRecord) -> ConsoleEntry {
    let mut fields = vec![("target".into(), record.target.clone())];
    for (key, value) in &record.fields {
        if key != "target" {
            fields.push((key.clone(), value.clone()));
        }
    }
    ConsoleEntry {
        time: record.timestamp_label(),
        level: console_level(record.level),
        message: record.message.clone(),
        fields,
    }
}

fn console_level(level: LogLevel) -> ConsoleLevel {
    match level {
        LogLevel::Trace => ConsoleLevel::Trace,
        LogLevel::Debug => ConsoleLevel::Debug,
        LogLevel::Info => ConsoleLevel::Info,
        LogLevel::Warn => ConsoleLevel::Warn,
        LogLevel::Error => ConsoleLevel::Error,
    }
}

fn parse_percent(value: &str) -> Option<f64> {
    let trimmed = value.trim().trim_end_matches('%');
    let load = trimmed.parse::<f64>().ok()?;
    (load.is_finite() && load >= 0.0).then_some(load)
}

#[allow(clippy::cast_precision_loss)]
fn memory_percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64) * 100.0
}

fn percent_signal_level(percent: f64) -> SignalLevel {
    if percent >= 90.0 {
        SignalLevel::Error
    } else if percent >= 75.0 {
        SignalLevel::Warning
    } else {
        SignalLevel::Good
    }
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    use crate::event::WorkerMsg;

    fn dashboard_app() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.current_resource = DASHBOARD_ID.to_string();
        app.pane = Pane::Content;
        app.poll_generation = 3;
        app.terminal_width = 140;
        app.terminal_height = 28;
        app
    }

    fn header_labels(app: &App) -> Vec<String> {
        app.header_signals()
            .into_iter()
            .map(|signal| {
                format!("{} {}", signal.label, signal.value)
                    .trim()
                    .to_string()
            })
            .collect()
    }

    fn system_resource(cpu_load: &str) -> Resource {
        let mut system = Resource::default();
        system.fields.insert("board-name".into(), "hEX S".into());
        system.fields.insert("cpu-load".into(), cpu_load.into());
        system.fields.insert("uptime".into(), "2h".into());
        system
            .fields
            .insert("total-memory".into(), "268435456".into());
        system
            .fields
            .insert("free-memory".into(), "134217728".into());
        system
    }

    fn wan_interface(rx_byte: &str) -> Resource {
        let mut iface = Resource {
            id: "*1".into(),
            ..Resource::default()
        };
        iface.fields.insert("name".into(), "pppoe-out1".into());
        iface.fields.insert("type".into(), "pppoe-out".into());
        iface.fields.insert("running".into(), "true".into());
        iface.fields.insert("rx-byte".into(), rx_byte.into());
        iface.fields.insert("tx-byte".into(), "1000".into());
        iface
    }

    fn header_result(
        generation: u64,
        system: Option<Resource>,
        interfaces: Vec<Resource>,
        interface_error: Option<String>,
    ) -> WorkerMsg {
        WorkerMsg::HeaderResult {
            session: SessionId::raw(1),
            request_id: 1,
            generation,
            system,
            system_error: None,
            interfaces,
            interface_error,
        }
    }

    fn is_fetch_dashboard(cmd: &AppCommand) -> bool {
        matches!(cmd, AppCommand::FetchDashboard { .. })
    }

    fn is_fetch_header(cmd: &AppCommand) -> bool {
        matches!(cmd, AppCommand::FetchHeader { .. })
    }

    fn is_fetch_safe_mode(cmd: &AppCommand) -> bool {
        matches!(cmd, AppCommand::FetchSafeMode { .. })
    }

    fn is_fetch_resource(cmd: &AppCommand, resource_id: &str) -> bool {
        matches!(
            cmd,
            AppCommand::FetchResource { resource_id: id, .. } if id == resource_id
        )
    }

    #[test]
    fn stale_dashboard_generation_is_ignored() {
        let mut app = dashboard_app();
        let before = app.status.clone();
        let cmds = app.update(AppEvent::Worker(WorkerMsg::DashboardResult {
            session: app.test_session(),
            request_id: 1,
            generation: 2,
            cpu: Vec::new(),
            cpu_error: None,
            system: None,
            system_error: None,
            interfaces: Vec::new(),
            interface_error: None,
            firewall: Vec::new(),
            firewall_error: None,
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.status, before);
        assert!(app.dash.cpu_core_order.is_empty());
    }

    #[test]
    fn tick_poll_requests_header_off_dashboard_and_dashboard_only_on_dashboard() {
        let mut app = dashboard_app();
        let on_dashboard = app.poll_current();
        assert_eq!(on_dashboard.len(), 2);
        assert!(is_fetch_dashboard(&on_dashboard[0]));
        assert!(!on_dashboard.iter().any(is_fetch_header));
        assert!(on_dashboard.iter().any(is_fetch_safe_mode));

        app.current_resource = "interfaces".into();
        let off_dashboard = app.poll_current();
        assert_eq!(off_dashboard.len(), 3);
        assert!(is_fetch_resource(&off_dashboard[0], "interfaces"));
        assert!(is_fetch_header(&off_dashboard[1]));
        assert!(off_dashboard.iter().any(is_fetch_safe_mode));
        assert!(!off_dashboard.iter().any(is_fetch_dashboard));
    }

    #[test]
    fn header_result_updates_cpu_memory_and_wan_off_dashboard() {
        let mut app = dashboard_app();
        app.current_resource = "interfaces".into();
        app.screen = Screen::Main;
        app.login.url = "https://192.0.2.1".into();
        app.status = "interfaces".into();
        let t0 = Instant::now()
            .checked_sub(std::time::Duration::from_secs(2))
            .expect("monotonic clock can go back two seconds");
        app.dash.update_wan(&wan_interface("1000"), t0);

        let cmds = app.update(AppEvent::Worker(header_result(
            app.poll_generation,
            Some(system_resource("18")),
            vec![wan_interface("125000")],
            None,
        )));
        assert!(cmds.is_empty());
        assert_eq!(app.status, "interfaces");
        assert_eq!(app.dash.memory_used_bytes, 134_217_728);
        assert_eq!(app.dash.memory_total_bytes, 268_435_456);
        let header = header_labels(&app);
        assert!(
            header.iter().any(|part| part == "MEM 50%"),
            "memory percent missing: {header:?}"
        );
        assert!(header.iter().any(|part| part == "CPU 18%"));
        assert!(app.dash.traffic_has_base);
        assert!(app.dash.traffic_rx_rate > 0.0);
        assert!(header.iter().any(|part| part.starts_with("WAN ")));
        assert_eq!(app.session_identity(), "hEX S · 192.0.2.1");
        assert!(app.dash.cpu_core_order.is_empty());
    }

    #[test]
    fn header_cpu_prefers_system_load_over_stale_cores() {
        let mut app = dashboard_app();
        app.current_resource = "interfaces".into();
        app.dash.cpu_core_loads.insert("cpu0".into(), 90.0);
        app.dash.cpu_core_loads.insert("cpu1".into(), 80.0);
        app.dash.cpu_core_order = vec!["cpu0".into(), "cpu1".into()];

        let cmds = app.update(AppEvent::Worker(header_result(
            app.poll_generation,
            Some(system_resource("22")),
            Vec::new(),
            Some("WAN unavailable".into()),
        )));
        assert!(cmds.is_empty());
        let header = header_labels(&app);
        assert!(
            header.iter().any(|part| part == "CPU 22%"),
            "stale core average still used: {header:?}"
        );
        assert!((app.dash.cpu_core_loads["cpu0"] - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stale_header_generation_is_ignored_after_select_resource() {
        let mut app = dashboard_app();
        app.current_resource = "interfaces".into();
        let stale_generation = app.poll_generation;
        app.select_resource("logs");
        let cmds = app.update(AppEvent::Worker(header_result(
            stale_generation,
            Some(system_resource("41")),
            vec![wan_interface("5000")],
            None,
        )));
        assert!(cmds.is_empty());
        assert!(!app.router.fields.contains_key("cpu-load"));
        assert_eq!(app.dash.memory_total_bytes, 0);
        assert!(!app.dash.traffic_has_base);
    }

    #[test]
    fn listen_delta_ignored_after_leaving_the_resource() {
        let mut app = dashboard_app();
        app.select_resource("interfaces");
        let generation = app.poll_generation;
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: 1,
            generation,
            resource_id: "interfaces".into(),
            rows: vec![wan_interface("1")],
            error: None,
        }));
        assert_eq!(app.table.row_count(), 1);
        app.select_resource("logs");
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ListenDelta {
            session: app.test_session(),
            generation,
            resource_id: "interfaces".into(),
            row: wan_interface("2"),
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.current_resource, "logs");
        assert_eq!(app.table.row_count(), 0);
    }

    #[test]
    fn dashboard_result_ignored_when_not_on_dashboard() {
        let mut app = dashboard_app();
        app.current_resource = "interfaces".into();
        app.status = "interfaces".into();
        let cmds = app.update(AppEvent::Worker(WorkerMsg::DashboardResult {
            session: app.test_session(),
            request_id: 1,
            generation: app.poll_generation,
            cpu: Vec::new(),
            cpu_error: None,
            system: Some(system_resource("77")),
            system_error: None,
            interfaces: vec![wan_interface("9000")],
            interface_error: None,
            firewall: Vec::new(),
            firewall_error: None,
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.status, "interfaces");
        assert!(!app.router.fields.contains_key("cpu-load"));
        assert!(!app.dash.traffic_has_base);
    }

    #[test]
    fn header_shows_identity_and_live_metrics() {
        let mut app = dashboard_app();
        app.login.url = "https://192.168.88.1:8443".into();
        app.router
            .fields
            .insert("board-name".into(), "hEX S".into());
        app.router.fields.insert("cpu-load".into(), "18".into());
        app.dash.memory_used_bytes = 128 * 1024 * 1024;
        app.dash.memory_total_bytes = 256 * 1024 * 1024;
        app.dash.traffic_has_base = true;
        app.dash.traffic_rx_rate = 84_200_000.0;

        let labels: Vec<_> = app
            .header_signals()
            .into_iter()
            .map(|signal| {
                format!("{} {}", signal.label, signal.value)
                    .trim()
                    .to_string()
            })
            .collect();
        assert_eq!(app.session_identity(), "hEX S · 192.168.88.1");
        assert_eq!(
            labels,
            [
                "CPU 18%".to_string(),
                "MEM 50%".to_string(),
                "WAN 84.2 Mb/s".to_string(),
            ]
        );
        assert!(!app.session_identity().contains("https"));
        assert!(!app.session_identity().contains("8443"));
    }

    #[test]
    fn header_host_strips_scheme_and_port() {
        assert_eq!(header_host("https://10.0.0.1"), "10.0.0.1");
        assert_eq!(header_host("https://10.0.0.1:443/"), "10.0.0.1");
        assert_eq!(header_host("https://[2001:db8::1]:8729"), "2001:db8::1");
    }

    #[test]
    fn resource_loaded_message_includes_entity_id_when_present() {
        let many = vec![
            Resource {
                id: "*1".into(),
                ..Resource::default()
            },
            Resource {
                id: "*2".into(),
                ..Resource::default()
            },
        ];
        assert_eq!(
            resource_loaded_message("interfaces", &many),
            "resource loaded interfaces"
        );

        let one = vec![Resource {
            id: "*1".into(),
            ..Resource::default()
        }];
        assert_eq!(
            resource_loaded_message("ethernet", &one),
            "resource loaded ethernet *1"
        );

        let singleton = vec![Resource::default()];
        assert_eq!(
            resource_loaded_message("clock", &singleton),
            "resource loaded clock"
        );
    }
}

#[cfg(test)]
mod activity_tests {
    use super::*;
    use mtui_ui::{ACTIVITY_SHOW_AFTER, activity_shown};

    #[test]
    fn activity_clock_arms_with_busy_flags_but_stays_hidden_until_delay() {
        let mut app = App::new(false).expect("app");
        assert!(!app.show_activity());
        app.loading = true;
        app.sync_activity();
        assert!(app.activity_since.is_some());
        assert!(!app.show_activity());

        let started = app.activity_since.expect("armed");
        assert!(activity_shown(Some(started), started + ACTIVITY_SHOW_AFTER));

        app.loading = false;
        app.refreshing = false;
        app.sync_activity();
        assert!(app.activity_since.is_none());
        assert!(!app.show_activity());
    }
}

#[cfg(test)]
mod palette_catalog_tests {
    use super::*;

    #[test]
    fn palette_commands_cover_every_routeros_path() {
        let commands = palette_commands();
        let mut by_id = std::collections::HashMap::new();
        for command in &commands {
            assert!(
                by_id.insert(command.id.as_str(), command).is_none(),
                "duplicate palette command {}",
                command.id
            );
        }
        assert_eq!(by_id["dashboard"].title, "Dashboard");
        for spec in ALL_RESOURCES.iter() {
            let command = by_id
                .get(spec.id)
                .unwrap_or_else(|| panic!("missing palette command for {}", spec.id));
            assert_eq!(command.path, spec.cli_path());
            assert_eq!(command.title, spec.cli_path());
            assert_eq!(command.description, spec.label);
        }
    }
}

#[cfg(test)]
mod secret_mask_tests {
    use super::*;
    use crate::event::{AppEvent, WorkerMsg};
    use mtui_routeros::{MASKED_VALUE, Resource};

    fn assert_inspector_hides_markers(fields: &[(String, String)]) {
        assert!(
            !fields.iter().any(|(_, value)| value.contains("MARKER")),
            "{fields:?}"
        );
    }

    #[test]
    fn resource_rows_mask_marker_secrets_before_table_and_inspector() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("wireguard");

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "wg0".into());
        fields.insert("private-key".into(), "MARKER-SECRET".into());
        fields.insert("preshared-key".into(), "MARKER-PSK".into());
        let row = Resource {
            id: "*1".into(),
            fields,
        };

        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "wireguard".into(),
            rows: vec![row],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(table_row.get("name").map(String::as_str), Some("wg0"));
        assert_eq!(
            table_row.get("private-key").map(String::as_str),
            Some(MASKED_VALUE)
        );
        assert_eq!(
            table_row.get("preshared-key").map(String::as_str),
            Some(MASKED_VALUE)
        );
        assert_inspector_hides_markers(&app.inspector.fields);
        assert!(
            app.inspector
                .fields
                .iter()
                .any(|(label, value)| { label == "Private key" && value == MASKED_VALUE }),
            "{:?}",
            app.inspector.fields
        );
        assert!(
            app.inspector
                .fields
                .iter()
                .any(|(label, value)| { label == "preshared-key" && value == MASKED_VALUE }),
            "{:?}",
            app.inspector.fields
        );
    }

    #[test]
    fn smb_user_password_is_masked_in_table_and_inspector() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("smb-users");

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "mtuser".into());
        fields.insert("password".into(), "MARKER-SECRET".into());
        fields.insert("read-only".into(), "false".into());
        let row = Resource {
            id: "*1".into(),
            fields,
        };

        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "smb-users".into(),
            rows: vec![row],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(table_row.get("name").map(String::as_str), Some("mtuser"));
        assert_eq!(
            table_row.get("password").map(String::as_str),
            Some(MASKED_VALUE)
        );
        assert_inspector_hides_markers(&app.inspector.fields);
        assert!(
            app.inspector
                .fields
                .iter()
                .any(|(label, value)| { label == "Password" && value == MASKED_VALUE }),
            "{:?}",
            app.inspector.fields
        );
    }

    #[test]
    fn ipsec_psk_key_is_masked_without_treating_container_env_key_as_secret() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("ipsec-key-psk");

        let mut fields = std::collections::HashMap::new();
        fields.insert("peer".into(), "office".into());
        fields.insert("id".into(), "user@example.com".into());
        fields.insert("key".into(), "MARKER-PSK".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "ipsec-key-psk".into(),
            rows: vec![Resource {
                id: "*1".into(),
                fields,
            }],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(table_row.get("peer").map(String::as_str), Some("office"));
        assert_eq!(table_row.get("key").map(String::as_str), Some(MASKED_VALUE));
        assert!(
            !app.inspector
                .fields
                .iter()
                .any(|(_, value)| value.contains("MARKER"))
        );

        app.select_resource("container-envs");
        let mut env = std::collections::HashMap::new();
        env.insert("list".into(), "app".into());
        env.insert("key".into(), "LOG_LEVEL".into());
        env.insert("value".into(), "info".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "container-envs".into(),
            rows: vec![Resource {
                id: "*2".into(),
                fields: env,
            }],
            error: None,
        }));
        assert!(cmds.is_empty());
        let env_row = app.table.selected_row().expect("env");
        assert_eq!(env_row.get("key").map(String::as_str), Some("LOG_LEVEL"));
    }

    #[test]
    fn stale_smb_share_result_is_ignored_after_navigation() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("smb-shares");
        let stale_generation = app.poll_generation;
        app.select_resource("smb-users");
        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "stale-share".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: stale_generation,
            resource_id: "smb-shares".into(),
            rows: vec![Resource {
                id: "*9".into(),
                fields,
            }],
            error: None,
        }));
        assert!(cmds.is_empty());
        assert!(app.table.selected_row().is_none());
        assert_eq!(app.current_resource, "smb-users");
    }

    #[test]
    fn smb_share_refresh_error_keeps_status_and_does_not_panic() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("smb-shares");
        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "backup".into());
        fields.insert("directory".into(), "backup".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "smb-shares".into(),
            rows: vec![Resource {
                id: "*2".into(),
                fields,
            }],
            error: None,
        }));
        assert_eq!(
            app.table
                .selected_row()
                .and_then(|row| row.get("name").cloned()),
            Some("backup".into())
        );
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "smb-shares".into(),
            rows: Vec::new(),
            error: Some("failure: request timed out".into()),
        }));
        assert!(cmds.is_empty());
        assert!(app.status.contains("Refresh failed"));
        assert_eq!(
            app.table
                .selected_row()
                .and_then(|row| row.get("name").cloned()),
            Some("backup".into())
        );
    }

    #[test]
    fn macsec_cak_is_masked_in_table_and_inspector() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("macsec");

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "macsec1".into());
        fields.insert("interface".into(), "ether1".into());
        fields.insert("cak".into(), "MARKER-CAK".into());
        fields.insert("ckn".into(), "visible-ckn".into());
        let row = Resource {
            id: "*1".into(),
            fields,
        };

        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "macsec".into(),
            rows: vec![row],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(table_row.get("cak").map(String::as_str), Some(MASKED_VALUE));
        assert_eq!(
            table_row.get("ckn").map(String::as_str),
            Some("visible-ckn")
        );
        assert!(
            !app.inspector
                .fields
                .iter()
                .any(|(_, value)| value.contains("MARKER"))
        );
    }

    #[test]
    fn romon_secrets_are_masked_in_table_and_inspector() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("romon");

        let mut fields = std::collections::HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("id".into(), "00:00:00:00:00:00".into());
        fields.insert("secrets".into(), "MARKER-SECRET".into());
        fields.insert("current-id".into(), "74:4D:28:00:00:01".into());
        let row = Resource {
            id: String::new(),
            fields,
        };

        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "romon".into(),
            rows: vec![row],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(
            table_row.get("secrets").map(String::as_str),
            Some(MASKED_VALUE)
        );
        assert_eq!(
            table_row.get("current-id").map(String::as_str),
            Some("74:4D:28:00:00:01")
        );
        assert!(
            !app.inspector
                .fields
                .iter()
                .any(|(_, value)| value.contains("MARKER"))
        );
    }

    #[test]
    fn lte_apn_password_is_masked_in_table_and_inspector() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("lte-apn");

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "carrier".into());
        fields.insert("apn".into(), "internet".into());
        fields.insert("password".into(), "MARKER-SECRET".into());
        let row = Resource {
            id: "*1".into(),
            fields,
        };

        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "lte-apn".into(),
            rows: vec![row],
            error: None,
        }));
        assert!(cmds.is_empty());

        let table_row = app.table.selected_row().expect("row");
        assert_eq!(table_row.get("apn").map(String::as_str), Some("internet"));
        assert_eq!(
            table_row.get("password").map(String::as_str),
            Some(MASKED_VALUE)
        );
        assert!(
            !app.inspector
                .fields
                .iter()
                .any(|(_, value)| value.contains("MARKER"))
        );
    }

    #[test]
    fn stale_lte_apn_refresh_is_ignored() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("lte-apn");
        let stale = app.poll_generation;
        app.poll_generation = stale.wrapping_add(1);

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "stale".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: stale,
            resource_id: "lte-apn".into(),
            rows: vec![Resource {
                id: "*9".into(),
                fields,
            }],
            error: None,
        }));
        assert!(cmds.is_empty());
        assert!(app.table.selected_row().is_none());
    }

    #[test]
    fn y_key_copies_current_row_to_clipboard() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("interfaces");
        app.pane = Pane::Content;

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".into(), "ether1".into());
        fields.insert("type".into(), "ether".into());
        fields.insert("mtu".into(), "1500".into());
        let row = Resource {
            id: "*1".into(),
            fields,
        };

        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: vec![row],
            error: None,
        }));

        let cmds = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        )));

        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                AppCommand::CopyToClipboard { text, .. }
                    if text.contains("name: ether1") && text.contains("mtu: 1500")
            )),
            "expected copy command, got {cmds:?}"
        );
    }
}

fn format_row_for_copy(row: &std::collections::HashMap<String, String>) -> String {
    let mut pairs: Vec<_> = row.iter().collect();
    pairs.sort_by_key(|(k, _)| *k);
    pairs
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_inspector_for_copy(inspector: &mtui_ui::InspectorState) -> String {
    inspector
        .fields
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod session_profile_tests {
    use super::*;
    use mtui_config::{Credential, CredentialStore, FileCredentialStore, ProfileStore};
    use mtui_ui::LoginPane;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "mtui-app-session-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn isolated_app(label: &str) -> (App, TempDir) {
        let dir = TempDir::new(label);
        let app = App::compose(
            false,
            ProfileStore::new(&dir.0),
            Box::new(FileCredentialStore::new(&dir.0)),
        );
        (app, dir)
    }

    fn sample_profile(name: &str, user: &str) -> Profile {
        Profile {
            name: name.into(),
            url: "192.168.88.1:8729".into(),
            username: user.into(),
            remember_password: true,
            ..Profile::default()
        }
    }

    #[test]
    fn persist_upserts_without_wiping_siblings() {
        let (mut app, dir) = isolated_app("upsert");
        let store = ProfileStore::new(&dir.0);
        store.upsert(sample_profile("edge", "admin")).unwrap();
        app.login.url = "10.0.0.1:8729".into();
        app.login.username = "reader".into();
        app.login.password = "pw".into();
        app.login.name = "core".into();
        app.login.remember_password = true;
        app.current_profile = "core".into();
        app.pending_password = Some("pw".into());
        app.persist_connected_session();
        let names: Vec<_> = store.load().unwrap().into_iter().map(|p| p.name).collect();
        assert!(names.contains(&"edge".to_string()));
        assert!(names.contains(&"core".to_string()));
        assert_eq!(store.last_used().unwrap().as_deref(), Some("core"));
        let creds = FileCredentialStore::new(&dir.0);
        assert_eq!(creds.get("core").unwrap().password, "pw");
    }

    #[test]
    fn persist_keeps_plaintext_api_and_ca_file() {
        let (mut app, dir) = isolated_app("plain-api");
        let store = ProfileStore::new(&dir.0);
        app.login.url = "192.168.88.1".into();
        app.login.username = "admin".into();
        app.login.password = "pw".into();
        app.login.name = "lab".into();
        app.login.use_tls = false;
        app.login.ca_file = "/tmp/router-ca.pem".into();
        app.login.remember_password = true;
        app.current_profile = "lab".into();
        app.pending_password = Some("pw".into());
        app.persist_connected_session();
        let loaded = store.load().unwrap();
        let lab = loaded.iter().find(|p| p.name == "lab").expect("lab");
        assert!(!lab.use_tls);
        assert_eq!(lab.url, "192.168.88.1:8728");
        assert_eq!(lab.ca_file, "/tmp/router-ca.pem");
        assert!(lab.certificate_fingerprint.is_empty());
        assert!(lab.custom_ca.is_empty());
    }

    #[test]
    fn remember_off_deletes_the_stored_secret() {
        let (mut app, dir) = isolated_app("remember-off");
        let creds = FileCredentialStore::new(&dir.0);
        creds
            .put(
                "core",
                Credential {
                    password: "old".into(),
                },
            )
            .unwrap();
        app.login.url = "10.0.0.1:8729".into();
        app.login.username = "admin".into();
        app.login.password = "typed".into();
        app.login.name = "core".into();
        app.login.remember_password = false;
        app.current_profile = "core".into();
        app.persist_connected_session();
        assert!(matches!(
            creds.get("core"),
            Err(mtui_config::ConfigError::CredentialsNotFound(_))
        ));
    }

    #[test]
    fn logout_keeps_profiles_and_auth_failure_does_not_wipe_them() {
        let (mut app, dir) = isolated_app("logout-keep");
        app.login.url = "10.0.0.1:8729".into();
        app.login.username = "admin".into();
        app.login.password = "pw".into();
        app.login.name = "core".into();
        app.current_profile = "core".into();
        app.persist_connected_session();
        app.screen = Screen::Main;
        app.disconnect_to_profiles();
        assert_eq!(app.screen, Screen::Login);
        let store = ProfileStore::new(&dir.0);
        assert_eq!(store.load().unwrap().len(), 1);

        app.screen = Screen::Connecting;
        let _ = app.update(AppEvent::Worker(WorkerMsg::Connected {
            session: app.test_session(),
            client: None,
            router: None,
            error: Some("cannot log in".into()),
            error_kind: Some(ErrorKind::Auth),
        }));
        assert_eq!(app.screen, Screen::Login);
        assert!(
            app.login
                .error
                .as_ref()
                .is_some_and(|msg| msg.contains("Wrong username"))
        );
        assert_eq!(store.load().unwrap().len(), 1);
    }

    #[test]
    fn totp_profile_does_not_auto_reconnect() {
        let (_app, dir) = isolated_app("totp");
        let mut profile = sample_profile("core", "admin");
        profile.uses_totp = true;
        ProfileStore::new(&dir.0).upsert(profile).unwrap();
        FileCredentialStore::new(&dir.0)
            .put(
                "core",
                Credential {
                    password: "pw".into(),
                },
            )
            .unwrap();
        let mut app = App::compose(
            false,
            ProfileStore::new(&dir.0),
            Box::new(FileCredentialStore::new(&dir.0)),
        );
        assert!(!app.restore_on_start);
        assert_eq!(app.login.focus, mtui_ui::LoginField::Totp);
        assert!(app.startup_commands().is_empty());
        drop(app);
    }

    #[test]
    fn last_used_profile_is_selected_without_connecting() {
        let (_app, dir) = isolated_app("no-auto");
        ProfileStore::new(&dir.0)
            .upsert(sample_profile("core", "admin"))
            .unwrap();
        FileCredentialStore::new(&dir.0)
            .put(
                "core",
                Credential {
                    password: "pw".into(),
                },
            )
            .unwrap();
        let mut app = App::compose(
            false,
            ProfileStore::new(&dir.0),
            Box::new(FileCredentialStore::new(&dir.0)),
        );
        assert!(!app.restore_on_start);
        assert_eq!(app.screen, Screen::Login);
        assert!(app.login.error.is_none());
        assert!(app.startup_commands().is_empty());
        assert_eq!(app.screen, Screen::Login);
        assert!(app.login.profiles.iter().any(|row| row.name == "core"));
        assert_eq!(app.login.password, "pw");
        drop(app);
    }

    #[test]
    fn apply_profile_keeps_a_typed_password_when_none_is_stored() {
        let (mut app, dir) = isolated_app("keep-typed");
        ProfileStore::new(&dir.0)
            .upsert(sample_profile("core", "admin"))
            .unwrap();
        app.reload_profile_rows();
        app.login.name = "core".into();
        app.current_profile = "core".into();
        app.login.password = "typed".into();
        app.login.remember_password = true;
        let profile = ProfileStore::new(&dir.0)
            .load()
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        app.apply_profile(&profile, true);
        assert_eq!(app.login.password, "typed");
    }

    #[test]
    fn persist_stores_the_pending_password_when_the_form_field_is_empty() {
        let (mut app, dir) = isolated_app("pending-secret");
        app.login.url = "10.0.0.1:8729".into();
        app.login.username = "admin".into();
        app.login.name = "core".into();
        app.login.password.clear();
        app.login.remember_password = true;
        app.current_profile = "core".into();
        app.pending_password = Some("secret".into());
        app.persist_connected_session();
        let creds = FileCredentialStore::new(&dir.0);
        assert_eq!(creds.get("core").unwrap().password, "secret");
    }

    #[test]
    fn persist_login_draft_remembers_password_without_connecting() {
        let (mut app, dir) = isolated_app("draft");
        app.login.url = "10.0.0.1:8729".into();
        app.login.username = "admin".into();
        app.login.name = "core".into();
        app.login.password = "typed".into();
        app.login.remember_password = true;
        app.persist_login_draft();
        let creds = FileCredentialStore::new(&dir.0);
        assert_eq!(creds.get("core").unwrap().password, "typed");
        let app = App::compose(
            false,
            ProfileStore::new(&dir.0),
            Box::new(FileCredentialStore::new(&dir.0)),
        );
        assert_eq!(app.login.password, "typed");
        assert!(app.login.remember_password);
        drop(app);
    }

    #[test]
    fn auth_required_keeps_the_open_screen() {
        let (mut app, _dir) = isolated_app("reauth");
        app.screen = Screen::Main;
        app.current_resource = "interfaces".into();
        app.login.username = "admin".into();
        let cmds = app.update(AppEvent::Worker(WorkerMsg::AuthRequired {
            session: app.test_session(),
            message: "Wrong username, password, or TOTP.".into(),
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.current_resource, "interfaces");
        assert!(matches!(app.overlay, Overlay::Reauth));
    }

    #[test]
    fn forget_removes_one_profile_only() {
        let (mut app, dir) = isolated_app("forget");
        let store = ProfileStore::new(&dir.0);
        store.upsert(sample_profile("core", "admin")).unwrap();
        store.upsert(sample_profile("edge", "admin")).unwrap();
        app.reload_profile_rows();
        app.forget_profile("core");
        let names: Vec<_> = store.load().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, vec!["edge".to_string()]);
        assert_eq!(app.login.pane, LoginPane::List);
    }

    #[test]
    fn demo_profile_opens_without_a_client() {
        let mut app = App::new(false).expect("app");
        let cmds = app.enter_demo();
        assert_eq!(app.screen, Screen::Main);
        assert!(app.client.is_none());
        assert!(app.demo.is_some());
        assert!(app.nav.unavailable.contains_key("wifi"));
        assert!(!cmds.is_empty());
        assert!(
            app.login
                .profiles
                .iter()
                .any(|row| row.name == crate::demo::DEMO_PROFILE_NAME)
        );
    }

    #[test]
    fn y_key_copies_filtered_table() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("interfaces");
        app.pane = Pane::Content;
        let mut first = std::collections::HashMap::new();
        first.insert("name".into(), "ether1".into());
        let mut second = std::collections::HashMap::new();
        second.insert("name".into(), "ether2".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: vec![
                Resource {
                    id: "*1".into(),
                    fields: first,
                },
                Resource {
                    id: "*2".into(),
                    fields: second,
                },
            ],
            error: None,
        }));
        let cmds = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('Y'),
            KeyModifiers::SHIFT,
        )));
        assert!(
            cmds.iter().any(|cmd| matches!(
                cmd,
                AppCommand::CopyToClipboard { text, .. }
                    if text.contains("name: ether1") && text.contains("name: ether2")
            )),
            "expected table copy, got {cmds:?}"
        );
    }
}

#[cfg(test)]
mod menu_path_gate_tests {
    use super::*;
    use crate::event::{AppEvent, WorkerMsg};

    #[test]
    fn menu_path_probe_hides_port_controller_and_leaves_the_screen() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("bridge-port-controller");
        let mut missing_ids = HashSet::new();
        missing_ids.insert("bridge-port-controller".into());
        let cmds = app.update(AppEvent::Worker(WorkerMsg::MenuPathsResult {
            session: app.test_session(),
            generation: app.menu_paths_generation,
            missing_ids,
            error: None,
        }));
        assert!(app.nav.unavailable.contains_key("bridge-port-controller"));
        assert_ne!(app.current_resource, "bridge-port-controller");
        assert!(!cmds.is_empty());
    }

    #[test]
    fn missing_prefix_error_hides_the_open_menu() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("bridge-port-controller");
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "bridge-port-controller".into(),
            rows: Vec::new(),
            error: Some("no such command prefix".into()),
        }));
        assert!(app.nav.unavailable.contains_key("bridge-port-controller"));
        assert_ne!(app.current_resource, "bridge-port-controller");
        assert!(!cmds.is_empty());
    }

    #[test]
    fn inspect_failure_does_not_hide_catalog_menus() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        let _ = app.update(AppEvent::Worker(WorkerMsg::MenuPathsResult {
            session: app.test_session(),
            generation: app.menu_paths_generation,
            missing_ids: HashSet::new(),
            error: Some("inspect failed".into()),
        }));
        assert!(!app.nav.unavailable.contains_key("bridge-port-controller"));
        assert!(!app.nav.unavailable.contains_key("bridges"));
    }

    #[test]
    fn stale_menu_path_probe_is_ignored() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        let mut missing_ids = HashSet::new();
        missing_ids.insert("bridge-port-controller".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::MenuPathsResult {
            session: app.test_session(),
            generation: app.menu_paths_generation.wrapping_add(1),
            missing_ids,
            error: None,
        }));
        assert!(!app.nav.unavailable.contains_key("bridge-port-controller"));
    }

    #[test]
    fn package_gate_still_hides_wifi_when_path_probe_is_empty() {
        let mut app = App::new(false).expect("app");
        let _ = app.enter_demo();
        assert!(app.nav.unavailable.contains_key("wifi"));
        let _ = app.update(AppEvent::Worker(WorkerMsg::MenuPathsResult {
            session: app.test_session(),
            generation: app.menu_paths_generation,
            missing_ids: HashSet::new(),
            error: None,
        }));
        assert!(app.nav.unavailable.contains_key("wifi"));
        assert!(!app.nav.unavailable.contains_key("interfaces"));
    }
}
