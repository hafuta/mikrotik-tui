//! Input and worker messages.

use std::sync::Arc;

use crossterm::event::KeyEvent;
use mtui_routeros::{Client, ErrorKind, Resource};

use crate::session::SessionId;

#[allow(clippy::large_enum_variant)]
pub enum AppEvent {
    Input(KeyEvent),
    Worker(WorkerMsg),
    Tick,
    Resize { width: u16, height: u16 },
}

pub enum WorkerMsg {
    ProbeResult {
        session: SessionId,
        fingerprint: Option<String>,
        error: Option<String>,
    },
    Connected {
        session: SessionId,
        client: Option<Arc<Client>>,
        router: Option<Resource>,
        error: Option<String>,
        error_kind: Option<ErrorKind>,
    },
    AuthRequired {
        session: SessionId,
        message: String,
    },
    SessionLost {
        session: SessionId,
        generation: u64,
        reason: String,
    },
    AccessResult {
        session: SessionId,
        generation: u64,
        users: Vec<Resource>,
        groups: Vec<Resource>,
        error: Option<String>,
    },
    ResourceResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        resource_id: String,
        rows: Vec<Resource>,
        error: Option<String>,
    },
    DashboardResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        cpu: Vec<Resource>,
        cpu_error: Option<String>,
        system: Option<Resource>,
        system_error: Option<String>,
        interfaces: Vec<Resource>,
        interface_error: Option<String>,
        firewall: Vec<Resource>,
        firewall_error: Option<String>,
    },
    HeaderResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        system: Option<Resource>,
        system_error: Option<String>,
        interfaces: Vec<Resource>,
        interface_error: Option<String>,
    },
    MutateResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        error: Option<String>,
    },
    TorchResult {
        session: SessionId,
        generation: u64,
        rows: Vec<std::collections::HashMap<String, String>>,
        error: Option<String>,
        done: bool,
    },
    ReadLocalFileResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        remote_name: String,
        contents: Option<String>,
        error: Option<String>,
    },
    WriteLocalFileResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        error: Option<String>,
    },
    RecordResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        local_path: String,
        contents: Option<String>,
        error: Option<String>,
    },
    PingTraceResult {
        session: SessionId,
        generation: u64,
        rows: Vec<std::collections::HashMap<String, String>>,
        error: Option<String>,
        done: bool,
    },
    LookupResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        options: Vec<String>,
        error: Option<String>,
    },
    FormRecordResult {
        session: SessionId,
        request_id: u64,
        generation: u64,
        resource_id: String,
        id: String,
        fields: Option<std::collections::HashMap<String, String>>,
        error: Option<String>,
    },
    ListLocalDirResult {
        session: SessionId,
        generation: u64,
        dir: String,
        entries: Vec<mtui_ui::FilePickerEntry>,
        error: Option<String>,
    },
    ListenDelta {
        session: SessionId,
        generation: u64,
        resource_id: String,
        row: Resource,
    },
    WanSample {
        session: SessionId,
        generation: u64,
        interface: String,
        sample: Resource,
    },
    SafeModeResult {
        session: SessionId,
        generation: u64,
        row: Option<Resource>,
        error: Option<String>,
    },
}

impl WorkerMsg {
    #[must_use]
    pub fn session(&self) -> SessionId {
        match self {
            Self::ProbeResult { session, .. }
            | Self::Connected { session, .. }
            | Self::AuthRequired { session, .. }
            | Self::SessionLost { session, .. }
            | Self::AccessResult { session, .. }
            | Self::ResourceResult { session, .. }
            | Self::DashboardResult { session, .. }
            | Self::HeaderResult { session, .. }
            | Self::MutateResult { session, .. }
            | Self::TorchResult { session, .. }
            | Self::ReadLocalFileResult { session, .. }
            | Self::WriteLocalFileResult { session, .. }
            | Self::RecordResult { session, .. }
            | Self::PingTraceResult { session, .. }
            | Self::LookupResult { session, .. }
            | Self::FormRecordResult { session, .. }
            | Self::ListLocalDirResult { session, .. }
            | Self::ListenDelta { session, .. }
            | Self::WanSample { session, .. }
            | Self::SafeModeResult { session, .. } => *session,
        }
    }
}
