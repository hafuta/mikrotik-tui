//! Isolated per-tab connection state. One [`Session`] is one device.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use mtui_core::{DASHBOARD_ID, SafeModeStatus, SessionAccess, navigation_tree};
use mtui_routeros::{Client, Resource};
use mtui_ui::{
    CommandPalette, ConsoleEntry, ConsoleState, FormSession, InspectorState, LoginForm, NavState,
    TableState,
};

use crate::app::{
    ConnectIntent, LogSeverity, Overlay, Pane, ReauthState, Screen, palette_commands,
};
use crate::demo::DemoStore;
use crate::safe_mode::{SafeModeAfter, SafeModeVerb};
use crate::telemetry::DashboardTelemetry;

/// Stable id for a tab / [`Session`]. Never reused within one process.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SessionId(u64);

impl SessionId {
    pub const UNSTAMPED: Self = Self(0);

    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub(crate) fn raw(value: u64) -> Self {
        Self(value)
    }
}

pub const MAX_SESSIONS: usize = 8;

/// Live TCP health for one device tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    Idle,
    Live,
    Dropped,
    Reconnecting,
}

/// One device connection and its UI. Never share [`Client`] with another session.
#[allow(clippy::struct_excessive_bools)]
pub struct Session {
    pub id: SessionId,
    pub screen: Screen,
    pub login: LoginForm,
    pub nav: NavState,
    pub pane: Pane,
    pub overlay: Overlay,
    pub overlay_scroll: u16,
    pub page_form: Option<FormSession>,
    pub(crate) lifecycle_return_to: Option<String>,
    pub palette: CommandPalette,
    pub table: TableState,
    pub inspector: InspectorState,
    pub status: String,
    pub trust_fingerprint: Option<String>,
    pub pending_password: Option<String>,
    pub reauth: ReauthState,
    pub(crate) connect_intent: ConnectIntent,
    pub(crate) current_profile: String,
    pub(crate) saved_url: Option<String>,
    pub(crate) saved_fingerprint: Option<String>,
    pub(crate) custom_ca: Option<Vec<u8>>,
    pub(crate) restore_on_start: bool,
    pub client: Option<Arc<Client>>,
    pub current_resource: String,
    pub loading: bool,
    pub refreshing: bool,
    pub(crate) activity_since: Option<Instant>,
    pub request_id: u64,
    pub poll_generation: u64,
    pub torch_generation: u64,
    pub probe_generation: u64,
    pub dash: DashboardTelemetry,
    pub router: Resource,
    pub log_buffer: VecDeque<Resource>,
    pub log_seen: HashSet<String>,
    pub log_paused: bool,
    pub log_follow: bool,
    pub log_severity: LogSeverity,
    pub log_unread: usize,
    /// While waiting for `/log/print`, follow replay is buffered and not painted.
    pub(crate) log_hold_follow_paint: bool,
    pub console: ConsoleState,
    pub console_entries: Vec<ConsoleEntry>,
    pub(crate) console_log_seq: u64,
    pub(crate) pane_before_console: Pane,
    pub(crate) demo: Option<DemoStore>,
    pub(crate) link: LinkState,
    pub(crate) last_ok_at: Option<Instant>,
    pub(crate) reconnect_at: Option<Instant>,
    pub(crate) reconnect_attempt: u32,
    pub access: SessionAccess,
    pub(crate) safe_mode: SafeModeStatus,
    pub(crate) held_safe_mode_at_drop: bool,
    pub(crate) floating_undo_count: usize,
    pub(crate) last_safe_mode_verb: Option<SafeModeVerb>,
    pub(crate) safe_mode_after: SafeModeAfter,
    pub(crate) installed_packages: HashSet<String>,
    pub(crate) missing_path_ids: HashSet<String>,
    pub(crate) menu_paths_generation: u64,
}

impl Session {
    pub(crate) fn new(id: SessionId) -> Self {
        Self {
            id,
            screen: Screen::Login,
            login: LoginForm::default(),
            nav: NavState::new(&navigation_tree()),
            pane: Pane::Nav,
            overlay: Overlay::None,
            overlay_scroll: 0,
            page_form: None,
            lifecycle_return_to: None,
            palette: CommandPalette::new(palette_commands()),
            table: TableState::new(&[]),
            inspector: InspectorState::default(),
            status: String::from("Enter RouterOS host and credentials"),
            trust_fingerprint: None,
            pending_password: None,
            reauth: ReauthState::default(),
            connect_intent: ConnectIntent::Login,
            current_profile: String::new(),
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
            probe_generation: 0,
            dash: DashboardTelemetry::default(),
            router: Resource::default(),
            log_buffer: VecDeque::new(),
            log_seen: HashSet::new(),
            log_paused: false,
            log_follow: true,
            log_severity: LogSeverity::All,
            log_unread: 0,
            log_hold_follow_paint: false,
            console: ConsoleState::default(),
            console_entries: Vec::new(),
            console_log_seq: 0,
            pane_before_console: Pane::Content,
            demo: None,
            link: LinkState::Idle,
            last_ok_at: None,
            reconnect_at: None,
            reconnect_attempt: 0,
            access: SessionAccess::unknown(),
            safe_mode: SafeModeStatus::default(),
            held_safe_mode_at_drop: false,
            floating_undo_count: 0,
            last_safe_mode_verb: None,
            safe_mode_after: SafeModeAfter::None,
            installed_packages: HashSet::new(),
            missing_path_ids: HashSet::new(),
            menu_paths_generation: 0,
        }
    }

    #[must_use]
    pub fn session_ready(&self) -> bool {
        !matches!(self.link, LinkState::Dropped | LinkState::Reconnecting)
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        self.demo.is_some() || (self.client.is_some() && self.link == LinkState::Live)
    }

    #[must_use]
    pub fn tab_title(&self) -> String {
        let name = self.login.name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
        let host = mtui_routeros::header_host(&self.login.url);
        if !host.is_empty() {
            return host;
        }
        "Login".into()
    }
}
