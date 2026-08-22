//! Top-level application model.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use mtui_config::{
    Credential, CredentialStore, EnvOverrides, FileCredentialStore, LogLevel, LogRecord, LogStore,
    Profile, ProfileStore, shared_log_store,
};
use mtui_core::{
    ALL_RESOURCES, DASHBOARD_ID, ThemeRegistry, ThemeSet, navigation_tree, resource_by_id,
};

use mtui_routeros::{Client, Resource};
use mtui_ui::{
    ActionMenuState, Command, CommandPalette, ConsoleEntry, ConsoleLevel, ConsoleState,
    DashboardGeometry, FirewallHitChart, FormSession, InspectorState, LayoutMetrics, LoginForm,
    NavState, Row, Signal, SignalLevel, TableState, TorchState, console_pane_height, format_rate,
};

use crate::event::{AppEvent, WorkerMsg};
use crate::telemetry::{DashboardTelemetry, select_wan_interface};
use crate::write::{ConfirmSession, MutationOp};

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
    Console,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    None,
    Help,
    Palette,
    Confirm(ConfirmSession),
    Form(FormSession),
    ActionMenu(ActionMenuState),
    TypePicker(ActionMenuState),
    Torch(TorchState),
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
    Mutate {
        request_id: u64,
        generation: u64,
        op: MutationOp,
    },
    FetchTorch {
        request_id: u64,
        generation: u64,
        interface: String,
        src: String,
        dst: String,
        protocol: String,
        port: String,
    },
    CopyToClipboard {
        text: String,
    },
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
    activity_since: Option<Instant>,
    pub request_id: u64,
    pub poll_generation: u64,
    pub torch_generation: u64,
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
    pub console: ConsoleState,
    pub console_entries: Vec<ConsoleEntry>,
    console_log_seq: u64,
    pub(crate) log_store: Arc<LogStore>,
    pane_before_console: Pane,
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
            activity_since: None,
            request_id: 0,
            poll_generation: 0,
            torch_generation: 0,
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
            console: ConsoleState::default(),
            console_entries: Vec::new(),
            console_log_seq: 0,
            log_store: shared_log_store(),
            pane_before_console: Pane::Content,
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
        tracing::info!(
            url = url.as_str(),
            username = self.login.username.trim(),
            "connecting"
        );
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
        self.pull_console_logs();
        let cmds = match event {
            AppEvent::Input(key) => self.on_key(key),
            AppEvent::Worker(msg) => self.on_worker(msg),
            AppEvent::Tick => self.on_tick(),
            AppEvent::Resize { width, height } => {
                self.terminal_width = width.max(1);
                self.terminal_height = height.max(1);
                self.palette.width = width.saturating_sub(4).min(64);
                self.sync_table_viewport();
                self.sync_console_viewport();
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
        if self.screen != Screen::Main || self.client.is_none() {
            return Vec::new();
        }
        let mut cmds = Vec::new();
        match &self.overlay {
            Overlay::None | Overlay::Form(_) | Overlay::Torch(_) => {
                tracing::debug!(resource = self.current_resource.as_str(), "scheduled poll");
                self.refreshing = true;
                cmds.extend(self.poll_current());
            }
            _ => {}
        }
        if matches!(&self.overlay, Overlay::Torch(torch) if torch.running) {
            cmds.extend(self.torch_sample_command());
        }
        cmds
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
                    tracing::error!(error = %err, "connection failed");
                    self.screen = Screen::Login;
                    self.login.error = Some(err);
                    self.status = "Connection failed".into();
                    return Vec::new();
                }
                tracing::info!("connected");
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
                    tracing::warn!(resource_id = resource_id.as_str(), error = %err, "resource refresh failed");
                    self.status = format!("Refresh failed: {err}");
                    return Vec::new();
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
                    self.apply_table_rows(Self::row_to_display(rows));
                    if let Some(id) = selected_id {
                        self.table.select_id(&id);
                    }
                }
                self.status = resource_id;
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
            WorkerMsg::MutateResult { .. } => self.apply_mutate_result(msg),
            WorkerMsg::TorchResult {
                generation,
                rows,
                error,
            } => self.apply_torch_result(generation, rows, error),
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
        vec![self.cpu_signal(), self.memory_signal(), self.wan_signal()]
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
        if !self.dash.cpu_core_loads.is_empty() {
            let total: f64 = self.dash.cpu_core_loads.values().copied().sum();
            let count = f64::from(u32::try_from(self.dash.cpu_core_loads.len()).unwrap_or(1));
            return Some(total / count.max(1.0));
        }
        nonempty_field(&self.router, "cpu-load").and_then(parse_percent)
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
        tracing::trace!(resource_id = id, "opened pane");
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.torch_generation = self.torch_generation.wrapping_add(1);
        self.overlay = Overlay::None;
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
            .saturating_sub(4)
            .max(1);
        let inner_h = self
            .terminal_height
            .saturating_sub(4)
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
        self.inspector
            .clamp_to_visible(self.inspector_visible_rows());
        self.sync_console_viewport();
    }

    pub(crate) fn sync_console_viewport(&mut self) {
        let height = usize::from(self.console_layout_height().saturating_sub(2).max(1));
        self.console.ensure_visible(&self.console_entries, height);
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
        Command::new("console", "Toggle console")
            .with_description("show or hide the application log console"),
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
        app.login.url = "https://192.0.2.1".into();
        let mut system = Resource::default();
        system.fields.insert("board-name".into(), "hEX S".into());
        system.fields.insert("cpu-load".into(), "18".into());
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
            header.iter().any(|part| part == "MEM 50%"),
            "memory percent missing: {header:?}"
        );
        assert!(header.iter().any(|part| part == "CPU 18%"));
        assert_eq!(app.session_identity(), "hEX S · 192.0.2.1");
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

#[cfg(test)]
mod secret_mask_tests {
    use super::*;
    use crate::event::{AppEvent, WorkerMsg};
    use mtui_routeros::{MASKED_VALUE, Resource};

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
        assert!(
            app.inspector
                .fields
                .iter()
                .all(|(key, value)| { key == "name" || value == MASKED_VALUE })
        );
        assert!(
            !app.inspector
                .fields
                .iter()
                .any(|(_, value)| value.contains("MARKER"))
        );
    }
}
