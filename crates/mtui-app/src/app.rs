//! Top-level application model.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use chrono::Local;
use mtui_config::{
    Credential, CredentialStore, EnvOverrides, FileCredentialStore, Profile, ProfileStore,
};
use mtui_core::{
    ALL_RESOURCES, DASHBOARD_ID, ThemeRegistry, ThemeSet, navigation_tree, resource_by_id,
};

use mtui_routeros::{Client, Resource};
use mtui_ui::{
    Command, CommandPalette, DashboardGeometry, FirewallHitChart, InspectorState, LayoutMetrics,
    LoginForm, NavState, Row, Signal, SignalLevel, TableState, format_bytes,
};

use crate::event::{AppEvent, WorkerMsg};
use crate::telemetry::{DashboardTelemetry, select_wan_interface};

const LOG_BUFFER_CAP: usize = 500;
const PROFILE_NAME: &str = "default";

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Help,
    Palette,
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    Quit,
    Connect {
        url: String,
        username: String,
        password: String,
        pin: Option<String>,
        ca_pem: Option<Vec<u8>>,
    },
    FetchResource {
        request_id: u64,
        generation: u64,
        resource_id: String,
    },
    FetchDashboard {
        request_id: u64,
        generation: u64,
    },
    FetchSystem,
    ClearSession,
}

#[allow(clippy::struct_excessive_bools)]
pub struct App {
    pub screen: Screen,
    pub login: LoginForm,
    pub themes: ThemeRegistry,
    pub theme: ThemeSet,
    pub nav: NavState,
    pub pane: Pane,
    pub overlay: Overlay,
    pub overlay_scroll: u16,
    pub palette: CommandPalette,
    pub table: TableState,
    pub inspector: InspectorState,
    pub status: String,
    pub trust_fingerprint: Option<String>,
    pub pending_password: Option<String>,
    saved_url: Option<String>,
    saved_fingerprint: Option<String>,
    custom_ca: Option<Vec<u8>>,
    restore_on_start: bool,
    pub client: Option<Arc<Client>>,
    pub current_resource: String,
    pub loading: bool,
    pub refreshing: bool,
    pub request_id: u64,
    pub poll_generation: u64,
    pub should_quit: bool,
    pub alt_screen: bool,
    pub dash: DashboardTelemetry,
    pub router: Resource,
    pub terminal_width: u16,
    pub terminal_height: u16,
    // logs
    pub log_buffer: VecDeque<Resource>,
    pub log_seen: HashSet<String>,
    pub log_paused: bool,
    pub log_follow: bool,
    pub log_severity: LogSeverity,
    pub log_unread: usize,
    profiles: ProfileStore,
    credentials: FileCredentialStore,
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
        let themes = ThemeRegistry::with_default();
        let theme = ThemeSet::from_theme(themes.active().as_ref());
        let nav = NavState::new(&navigation_tree());
        let profiles = ProfileStore::discover()?;
        let credentials = FileCredentialStore::discover()?;

        let mut app = Self {
            screen: Screen::Login,
            login: LoginForm::default(),
            themes,
            theme,
            nav,
            pane: Pane::Nav,
            overlay: Overlay::None,
            overlay_scroll: 0,
            palette: CommandPalette::new(palette_commands()),
            table: TableState::new(&[]),
            inspector: InspectorState::default(),
            status: String::from("Enter RouterOS HTTPS URL and credentials"),
            trust_fingerprint: None,
            pending_password: None,
            saved_url: None,
            saved_fingerprint: None,
            custom_ca: None,
            restore_on_start: false,
            client: None,
            current_resource: DASHBOARD_ID.to_string(),
            loading: false,
            refreshing: false,
            request_id: 0,
            poll_generation: 0,
            should_quit: false,
            alt_screen,
            dash: DashboardTelemetry::default(),
            router: Resource::default(),
            terminal_width: 80,
            terminal_height: 24,
            log_buffer: VecDeque::new(),
            log_seen: HashSet::new(),
            log_paused: false,
            log_follow: true,
            log_severity: LogSeverity::All,
            log_unread: 0,
            profiles,
            credentials,
        };

        app.sync_table_viewport();
        app.load_saved_session();
        Ok(app)
    }

    fn load_saved_session(&mut self) {
        let overrides = EnvOverrides::from_env();
        let mut profile = self
            .profiles
            .load()
            .ok()
            .and_then(|list| list.into_iter().next())
            .unwrap_or_else(|| Profile {
                name: PROFILE_NAME.into(),
                ..Profile::default()
            });
        if let Err(err) = overrides.apply_to_profile(&mut profile) {
            self.status = format!("Saved session overrides failed: {err}");
        }
        if !profile.url.is_empty() {
            self.login.url.clone_from(&profile.url);
        }
        if !profile.username.is_empty() {
            self.login.username.clone_from(&profile.username);
        }
        if let Some(theme_id) = profile.theme_id() {
            let _ = self.themes.set_active(theme_id);
            self.theme = ThemeSet::from_theme(self.themes.active().as_ref());
        }
        if !profile.certificate_fingerprint.is_empty() {
            self.saved_fingerprint = Some(profile.certificate_fingerprint.clone());
            self.saved_url = Some(normalize_router_url(&profile.url));
        }
        if !profile.custom_ca.is_empty() {
            self.custom_ca = Some(profile.custom_ca.into_bytes());
        }
        match overrides.resolve_password(&profile.name, Some(&self.credentials)) {
            Ok(Some(password)) => self.login.password = password,
            Ok(None) => {}
            Err(err) => {
                self.status = format!("Saved credentials unavailable: {err}");
                return;
            }
        }
        let has_router =
            is_https_router_url(&self.login.url) && !self.login.username.trim().is_empty();
        if has_router {
            self.restore_on_start = true;
            self.status = if self.login.password.is_empty() {
                format!("Loaded profile '{}'; press Enter to connect", profile.name)
            } else {
                "Restoring saved session…".into()
            };
        }
    }

    /// Auto-connect when a saved profile exists, matching the Go startup path.
    pub fn startup_commands(&mut self) -> Vec<AppCommand> {
        if !self.restore_on_start {
            return Vec::new();
        }
        self.restore_on_start = false;
        if !is_https_router_url(&self.login.url) || self.login.username.trim().is_empty() {
            return Vec::new();
        }
        self.pending_password = Some(self.login.password.clone());
        self.screen = Screen::Connecting;
        self.status = "Restoring saved session…".into();
        vec![self.connect_command()]
    }

    pub(crate) fn connect_command(&self) -> AppCommand {
        let url = normalize_router_url(&self.login.url);
        AppCommand::Connect {
            url: url.clone(),
            username: self.login.username.trim().to_string(),
            password: self.pending_password.clone().unwrap_or_default(),
            pin: self.pin_for_url(&url),
            ca_pem: self.custom_ca.clone(),
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

    fn persist_connected_session(&mut self) {
        let url = normalize_router_url(&self.login.url);
        let fingerprint = self.pin_for_url(&url).unwrap_or_default();
        let mut profile = Profile {
            name: PROFILE_NAME.into(),
            url: url.clone(),
            username: self.login.username.trim().to_string(),
            certificate_fingerprint: fingerprint.clone(),
            ..Profile::default()
        };
        if let Some(pem) = &self.custom_ca {
            profile.custom_ca = String::from_utf8_lossy(pem).into_owned();
        }
        profile.set_theme_id(self.theme.id.as_str());
        let password = self.pending_password.clone().unwrap_or_default();
        if self
            .profiles
            .save(std::slice::from_ref(&profile))
            .and_then(|()| self.credentials.put(&profile.name, Credential { password }))
            .is_err()
        {
            self.status = "Connected · profile could not be saved".into();
            return;
        }
        self.saved_url = Some(url);
        self.saved_fingerprint = if fingerprint.is_empty() {
            None
        } else {
            Some(fingerprint)
        };
    }

    pub(crate) fn clear_saved_session(&mut self) {
        let _ = self.profiles.save(&[]);
        let _ = self.credentials.delete(PROFILE_NAME);
        self.saved_fingerprint = None;
        self.saved_url = None;
        self.restore_on_start = false;
    }

    pub fn styles(&self) -> mtui_ui::Styles {
        mtui_ui::Styles::from_palette(&self.theme.palette)
    }

    pub fn update(&mut self, event: AppEvent) -> Vec<AppCommand> {
        match event {
            AppEvent::Input(key) => self.on_key(key),
            AppEvent::Worker(msg) => self.on_worker(msg),
            AppEvent::Tick => self.on_tick(),
            AppEvent::Resize { width, height } => {
                self.terminal_width = width.max(1);
                self.terminal_height = height.max(1);
                self.palette.width = width.saturating_sub(4).min(64);
                self.sync_table_viewport();
                Vec::new()
            }
        }
    }

    fn on_tick(&mut self) -> Vec<AppCommand> {
        if self.screen != Screen::Main || self.client.is_none() {
            return Vec::new();
        }
        if self.overlay != Overlay::None {
            return Vec::new();
        }
        self.refreshing = true;
        self.poll_current()
    }

    #[allow(clippy::too_many_lines)]
    fn on_worker(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        match msg {
            WorkerMsg::ProbeResult { fingerprint, error } => {
                if let Some(err) = error {
                    self.screen = Screen::Login;
                    self.login.error = Some(err);
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
            } => {
                if let Some(err) = error {
                    self.screen = Screen::Login;
                    self.login.error = Some(err);
                    self.status = "Connection failed".into();
                    return Vec::new();
                }
                self.client = client;
                if let Some(router) = router {
                    self.apply_system_resource(router);
                } else {
                    self.router = Resource::default();
                }
                self.screen = Screen::Main;
                self.status = "Connected".into();
                self.poll_generation = self.poll_generation.wrapping_add(1);
                self.select_resource(DASHBOARD_ID);
                self.persist_connected_session();
                vec![AppCommand::FetchDashboard {
                    request_id: self.next_request(),
                    generation: self.poll_generation,
                }]
            }
            WorkerMsg::ResourceResult {
                request_id,
                generation,
                resource_id,
                rows,
                error,
            } => {
                if generation != self.poll_generation
                    || request_id < self.request_id.saturating_sub(1)
                {
                    // Stale — still accept if it's the latest for this generation and resource.
                }
                if generation != self.poll_generation {
                    return Vec::new();
                }
                if resource_id != self.current_resource {
                    return Vec::new();
                }
                self.loading = false;
                self.refreshing = false;
                if let Some(err) = error {
                    self.status = format!("Refresh failed: {err}");
                    return Vec::new();
                }
                if resource_id == "logs" {
                    self.ingest_logs(rows);
                } else {
                    let masked: Vec<_> = rows.into_iter().map(|r| r.masked_fields()).collect();
                    self.apply_table_rows(masked);
                }
                self.status = if self.refreshing {
                    "Refreshing…".into()
                } else {
                    resource_id
                };
                Vec::new()
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
                );
                Vec::new()
            }
            WorkerMsg::SystemResult { system, error: _ } => {
                if self.screen != Screen::Main {
                    return Vec::new();
                }
                if let Some(system) = system {
                    self.apply_system_resource(system);
                }
                Vec::new()
            }
        }
    }

    /// Header signals: board, host, `RouterOS` version, memory, uptime, and local clock.
    #[must_use]
    pub fn header_signals(&self) -> Vec<Signal> {
        let board = nonempty_field(&self.router, "board-name").unwrap_or("RouterOS");
        let version = nonempty_field(&self.router, "version").unwrap_or_default();
        let uptime = nonempty_field(&self.router, "uptime").unwrap_or_default();
        vec![
            Signal::new(board, "", SignalLevel::Good),
            Signal::new(header_host(&self.login.url), "", SignalLevel::Good),
            Signal::new("RouterOS", version, SignalLevel::Idle),
            self.memory_signal(),
            Signal::new("uptime", uptime, SignalLevel::Idle),
            Signal::new(format_header_clock(Local::now()), "", SignalLevel::Idle),
        ]
    }

    fn memory_signal(&self) -> Signal {
        let used = self.dash.memory_used_bytes;
        let total = self.dash.memory_total_bytes;
        if total == 0 {
            return Signal::new("mem", "—", SignalLevel::Idle);
        }
        Signal::new(
            "mem",
            format!("{} / {}", format_bytes(used), format_bytes(total)),
            memory_signal_level(used, total),
        )
    }

    pub(crate) fn poll_current(&mut self) -> Vec<AppCommand> {
        let generation = self.poll_generation;
        if self.current_resource == DASHBOARD_ID {
            vec![AppCommand::FetchDashboard {
                request_id: self.next_request(),
                generation,
            }]
        } else {
            vec![
                AppCommand::FetchResource {
                    request_id: self.next_request(),
                    generation,
                    resource_id: self.current_resource.clone(),
                },
                AppCommand::FetchSystem,
            ]
        }
    }

    pub(crate) fn next_request(&mut self) -> u64 {
        self.request_id = self.request_id.wrapping_add(1);
        self.request_id
    }

    pub(crate) fn select_resource(&mut self, id: &str) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.current_resource = id.to_string();
        self.refreshing = false;
        let _ = self.nav.select_id(id);
        if id == DASHBOARD_ID {
            self.activate_dashboard();
        } else {
            self.loading = true;
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
            .saturating_sub(2)
            .max(1);
        let inner_h = self.terminal_height.saturating_sub(5).max(1);
        (usize::from(inner_w), usize::from(inner_h))
    }

    pub(crate) fn sync_table_viewport(&mut self) {
        let (width, height) = self.table_inner_size();
        self.table.sync_viewport(width, height);
        self.inspector
            .clamp_to_visible(self.inspector_visible_rows());
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
        self.inspector = InspectorState::from_row(self.table.selected_row());
        if preserve_offset {
            self.inspector.offset = offset;
            self.inspector
                .clamp_to_visible(self.inspector_visible_rows());
        }
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
            self.status = format!("System telemetry live · WAN {err} · retrying");
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
                self.status = if unavailable.is_empty() {
                    format!("WAN telemetry live · {}", self.dash.traffic_interface)
                } else {
                    format!("WAN live · {} telemetry retrying", unavailable.join(" + "))
                };
            }
            Err(err) => {
                self.status = format!("System telemetry live · WAN {err} · retrying");
            }
        }
    }

    fn apply_system_resource(&mut self, system: Resource) {
        self.dash.update_system(&system);
        self.router = system;
    }
}

pub(crate) fn palette_commands() -> Vec<Command> {
    let mut commands = vec![
        Command::new("refresh", "Refresh").with_description("reload the current resource"),
        Command::new("logout", "Log out").with_description("forget this router session"),
        Command::new("help", "Keyboard help").with_description("show all shortcuts"),
        Command::new("dashboard", "Dashboard").with_description("live WAN overview"),
    ];
    commands.extend(ALL_RESOURCES.iter().map(|spec| {
        Command::new(spec.id, spec.cli_path())
            .with_description(spec.label)
            .with_path(spec.cli_path())
    }));
    commands
}

fn nonempty_field<'src>(resource: &'src Resource, key: &str) -> Option<&'src str> {
    resource
        .field(key)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_router_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

/// Host or IP from a router URL, without scheme or port.
fn header_host(url: &str) -> String {
    let rest = url
        .trim()
        .split_once("://")
        .map_or(url.trim(), |(_, rest)| rest);
    let hostport = rest.split('/').next().unwrap_or(rest);
    if let Some(inner) = hostport.strip_prefix('[') {
        inner.split(']').next().unwrap_or(inner).to_string()
    } else {
        hostport.split(':').next().unwrap_or(hostport).to_string()
    }
}

pub(crate) fn is_https_router_url(url: &str) -> bool {
    let url = normalize_router_url(url);
    match url.split_once("://") {
        Some(("https", rest)) => {
            let host = rest.split('/').next().unwrap_or("");
            !host.is_empty() && host != "https:"
        }
        _ => false,
    }
}

fn format_header_clock(now: chrono::DateTime<Local>) -> String {
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
fn is_header_clock(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 19
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b' '
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 | 10 | 13 | 16 => true,
            _ => byte.is_ascii_digit(),
        })
}

fn memory_signal_level(used: u64, total: u64) -> SignalLevel {
    if total == 0 {
        return SignalLevel::Idle;
    }
    let used_percent_x100 = used.saturating_mul(100);
    if used_percent_x100 >= total.saturating_mul(90) {
        SignalLevel::Error
    } else if used_percent_x100 >= total.saturating_mul(75) {
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

    #[test]
    fn stale_dashboard_generation_is_ignored() {
        let mut app = dashboard_app();
        let before = app.status.clone();
        let cmds = app.update(AppEvent::Worker(WorkerMsg::DashboardResult {
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
    fn system_result_updates_header_memory_off_dashboard() {
        let mut app = dashboard_app();
        app.current_resource = "interfaces".into();
        app.screen = Screen::Main;
        let mut system = Resource::default();
        system.fields.insert("board-name".into(), "hEX S".into());
        system.fields.insert("version".into(), "7.23.3".into());
        system.fields.insert("uptime".into(), "2h".into());
        system
            .fields
            .insert("total-memory".into(), "268435456".into());
        system
            .fields
            .insert("free-memory".into(), "134217728".into());

        let cmds = app.update(AppEvent::Worker(WorkerMsg::SystemResult {
            system: Some(system),
            error: None,
        }));
        assert!(cmds.is_empty());
        assert_eq!(app.dash.memory_used_bytes, 134_217_728);
        assert_eq!(app.dash.memory_total_bytes, 268_435_456);
        let header = app
            .header_signals()
            .into_iter()
            .map(|signal| {
                format!("{} {}", signal.label, signal.value)
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert!(
            header
                .iter()
                .any(|part| part == "mem 128.0 MiB / 256.0 MiB")
        );
        assert!(header.iter().any(|part| part == "hEX S"));
        assert!(header.iter().any(|part| part == "uptime 2h"));
    }

    #[test]
    fn header_shows_board_host_version_and_uptime() {
        let mut app = dashboard_app();
        app.login.url = "https://192.168.88.1:8443".into();
        app.router
            .fields
            .insert("board-name".into(), "hEX S".into());
        app.router.fields.insert("version".into(), "7.23.3".into());
        app.router.fields.insert("uptime".into(), "1d".into());
        app.dash.memory_used_bytes = 128 * 1024 * 1024;
        app.dash.memory_total_bytes = 256 * 1024 * 1024;

        let labels: Vec<_> = app
            .header_signals()
            .into_iter()
            .map(|signal| {
                format!("{} {}", signal.label, signal.value)
                    .trim()
                    .to_string()
            })
            .collect();
        assert_eq!(
            &labels[..labels.len().saturating_sub(1)],
            [
                "hEX S",
                "192.168.88.1",
                "RouterOS 7.23.3",
                "mem 128.0 MiB / 256.0 MiB",
                "uptime 1d",
            ]
        );
        assert!(
            labels.last().is_some_and(|clock| is_header_clock(clock)),
            "clock missing or unpadded: {labels:?}"
        );
        assert!(!labels.iter().any(|part| part.contains("https")));
        assert!(!labels.iter().any(|part| part.contains("8443")));
        assert!(!labels.iter().any(|part| part.contains("session")));
        assert!(!labels.iter().any(|part| part.contains("MikroTik")));
        assert!(!labels.iter().any(|part| part.contains("user")));
    }

    #[test]
    fn header_host_strips_scheme_and_port() {
        assert_eq!(header_host("https://10.0.0.1"), "10.0.0.1");
        assert_eq!(header_host("https://10.0.0.1:443/"), "10.0.0.1");
        assert_eq!(header_host("https://[2001:db8::1]:8729"), "2001:db8::1");
    }

    #[test]
    fn header_clock_zero_pads_single_digits() {
        use chrono::TimeZone;
        let dt = Local
            .with_ymd_and_hms(2026, 8, 22, 1, 6, 5)
            .single()
            .expect("valid local time");
        assert_eq!(format_header_clock(dt), "2026-08-22 01:06:05");
        assert!(is_header_clock("2026-08-22 01:26:35"));
        assert!(!is_header_clock("2026-8-22 1:26:35"));
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
        for spec in ALL_RESOURCES {
            let command = by_id
                .get(spec.id)
                .unwrap_or_else(|| panic!("missing palette command for {}", spec.id));
            assert_eq!(command.path, spec.cli_path());
            assert_eq!(command.title, spec.cli_path());
            assert_eq!(command.description, spec.label);
        }
    }
}
