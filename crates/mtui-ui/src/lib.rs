//! Pure TUI widgets and layouts. No networking.
//!
//! Colors resolve through [`mtui_core::Palette`] / the active theme.

mod charts;
mod chrome;
mod dashboard;
mod firewall;
mod inspector;
mod layout;
mod login;
mod navigation;
mod overlay;
mod palette;
mod styles;
mod table;

pub use charts::{
    BrailleSparkline, TrafficChart, TrafficSample, format_bytes, format_rate, format_traffic_rate,
};
pub use chrome::{Signal, SignalLevel, footer_hints, header_line, signal_rail, status_line};
pub use dashboard::{CpuCoreView, DashboardGeometry, DashboardView, dashboard_content};
pub use firewall::{FirewallHitChart, FirewallRuleMetric, MAX_FIREWALL_RULES};
pub use inspector::InspectorState;
pub use layout::{
    Breakpoint, LayoutMetrics, clip_line, constrain_lines, fit_cell, fit_line, line_plain,
    line_width, lines_plain,
};
pub use login::{LoginField, LoginForm};
pub use navigation::{FlatNavEntry, NavState, flatten_nav};
pub use overlay::{
    Modal, ModalButton, ModalButtonKind, ModalKind, ModalPanel, compact_modal_rect, dim_canvas,
    format_fingerprint, modal_rect, render_modal, render_modal_frame,
};
pub use palette::{Command, CommandPalette, PALETTE_VISIBLE_ROWS, highlight_match};
pub use styles::{Styles, rgb_color};
pub use table::{Row, SortDir, TableState};
