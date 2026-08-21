//! Input and worker messages.

use std::sync::Arc;

use crossterm::event::KeyEvent;
use mtui_routeros::{Client, Resource};

#[allow(clippy::large_enum_variant)]
pub enum AppEvent {
    Input(KeyEvent),
    Worker(WorkerMsg),
    Tick,
    Resize { width: u16, height: u16 },
}

pub enum WorkerMsg {
    ProbeResult {
        fingerprint: Option<String>,
        error: Option<String>,
    },
    Connected {
        client: Option<Arc<Client>>,
        router: Option<Resource>,
        error: Option<String>,
    },
    ResourceResult {
        request_id: u64,
        generation: u64,
        resource_id: String,
        rows: Vec<Resource>,
        error: Option<String>,
    },
    DashboardResult {
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
    SystemResult {
        system: Option<Resource>,
        error: Option<String>,
    },
}
