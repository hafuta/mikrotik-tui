//! Pure TUI widgets and layouts. No networking.
//!
//! Colors resolve through [`mtui_core::Palette`] / the active theme.

mod action_menu;
mod charts;
mod chrome;
mod console;
mod dashboard;
mod firewall;
mod form;
mod inspector;
mod layout;
mod login;
mod navigation;
mod overlay;
mod paint;
mod palette;
mod probe;
mod styles;
mod table;
mod torch;

pub use action_menu::{ActionMenuItem, ActionMenuState, render_action_menu};
pub use charts::{
    BrailleSparkline, TrafficChart, TrafficSample, format_bytes, format_rate, format_traffic_rate,
};
pub use chrome::{
    ACTIVITY_SHOW_AFTER, Signal, SignalLevel, activity_shown, footer_bar, footer_hints,
    header_line, session_header, signal_rail, status_line,
};
pub use console::{ConsoleEntry, ConsoleLevel, ConsoleState, TIME_COL, console_pane_height};
pub use dashboard::{CpuCoreView, DashboardGeometry, DashboardView, dashboard_content};
pub use firewall::{FirewallHitChart, FirewallRuleMetric, MAX_FIREWALL_RULES};
pub use form::{
    BACKUP_SAVE_FORM, COPY_FORM, DOWNLOAD_FORM, FETCH_FORM, FormMode, FormSession, UPLOAD_FORM,
    render_form_sheet,
};
pub use inspector::InspectorState;
pub use layout::{
    Breakpoint, LayoutMetrics, clip_line, constrain_lines, fit_cell, fit_line, line_plain,
    line_width, lines_plain,
};
pub use login::{LoginField, LoginForm, is_printable_char};
pub use navigation::{FlatNavEntry, NavState, ToggleHidden, flatten_nav};
pub use overlay::{
    Modal, ModalButton, ModalButtonKind, ModalKind, ModalPanel, compact_modal_rect, dim_canvas,
    format_fingerprint, modal_max_scroll, modal_rect, render_modal, render_modal_frame,
};
pub use paint::{fill_rect, line_on_bg};
pub use palette::{Command, CommandPalette, PALETTE_VISIBLE_ROWS, highlight_match};
pub use probe::{ProbeField, ProbeKind, ProbeState, render_probe};
pub use styles::{Styles, rgb_color};
pub use table::{Row, SortDir, TableState};
pub use torch::{TorchField, TorchState, render_torch};
