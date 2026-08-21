//! Ratatui rendering — pure view from [`App`] state + theme styles.

use mtui_core::DASHBOARD_ID;
use mtui_ui::{
    CpuCoreView, DashboardView, LayoutMetrics, Modal, ModalButton, ModalButtonKind, ModalPanel,
    constrain_lines, dashboard_content, fit_line, footer_hints, format_fingerprint, header_line,
    render_modal, signal_rail, status_line,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::{App, Overlay, Pane, Screen};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let styles = app.styles();

    match app.screen {
        Screen::Login => draw_login(frame, area, app),
        Screen::Connecting => {
            let msg = Paragraph::new(format!(" {} ", app.status)).style(styles.signal);
            frame.render_widget(msg, area);
        }
        Screen::Trust => draw_trust(frame, area, app),
        Screen::Main => {
            draw_main(frame, area, app);
            match app.overlay {
                Overlay::Help => {
                    let modal = Modal::new("Keyboard help", HELP_TEXT).scroll(app.overlay_scroll);
                    render_modal(frame, area, &modal, &styles);
                }
                Overlay::Palette => {
                    app.palette.render(frame, area, &styles);
                }
                Overlay::None => {}
            }
        }
    }
}

fn draw_login(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(header_line("MikroTik TUI", "(Rust)", &styles)),
        chunks[0],
    );

    let fields = [
        (
            "URL",
            app.login.url.as_str(),
            app.login.focus == mtui_ui::LoginField::Url,
        ),
        (
            "Username",
            app.login.username.as_str(),
            app.login.focus == mtui_ui::LoginField::Username,
        ),
        (
            "Password",
            &"*".repeat(app.login.password.len()),
            app.login.focus == mtui_ui::LoginField::Password,
        ),
    ];
    for (i, (label, value, focused)) in fields.iter().enumerate() {
        let style = if *focused { styles.focus } else { styles.text };
        let block = Block::default()
            .title(*label)
            .borders(Borders::ALL)
            .border_style(if *focused {
                styles.focus
            } else {
                styles.border
            });
        frame.render_widget(
            Paragraph::new(value.to_string()).style(style).block(block),
            chunks[i + 1],
        );
    }
    let err = app.login.error.clone().unwrap_or_default();
    frame.render_widget(Paragraph::new(err).style(styles.error), chunks[4]);
    frame.render_widget(Paragraph::new(status_line(&app.status, &styles)), chunks[5]);
}

fn draw_trust(frame: &mut Frame<'_>, area: Rect, app: &App) {
    draw_login(frame, area, app);
    let styles = app.styles();
    let fingerprint = format_fingerprint(app.trust_fingerprint.as_deref().unwrap_or_default());
    let buttons = [
        ModalButton {
            label: "Trust",
            keys: "y / enter",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Cancel",
            keys: "n / esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let modal = Modal::new(
        "Certificate",
        "Verify this SHA-256 fingerprint through a trusted channel before sending credentials.",
    )
    .alert()
    .kicker("Unrecognized router certificate")
    .panel(ModalPanel {
        label: "SHA-256",
        value: &fingerprint,
    })
    .hint("Approval pins this certificate for the saved session.")
    .buttons(&buttons);
    render_modal(frame, area, &modal, &styles);
}

fn draw_main(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let metrics = LayoutMetrics::new(area.width, area.height);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let inner_width = usize::from(area.width.saturating_sub(2).max(1));
    let mut spans = vec![Span::raw(" ")];
    spans.extend(signal_rail(&app.header_signals(), inner_width, &styles).spans);
    let header = fit_line(Line::from(spans), usize::from(area.width.max(1)));
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let body = chunks[1];
    let mut constraints = Vec::new();
    if metrics.nav_width > 0 {
        constraints.push(Constraint::Length(metrics.nav_width));
    }
    constraints.push(Constraint::Min(20));
    if metrics.inspector_width > 0 && app.current_resource != DASHBOARD_ID {
        constraints.push(Constraint::Length(metrics.inspector_width));
    }
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(body);

    let mut idx = 0;
    if metrics.nav_width > 0 {
        draw_nav(frame, panes[idx], app);
        idx += 1;
    }
    let content = panes[idx];
    if app.current_resource == DASHBOARD_ID {
        draw_dashboard(frame, content, app);
    } else {
        draw_table(frame, content, app);
    }
    idx += 1;
    if metrics.inspector_width > 0 && app.current_resource != DASHBOARD_ID && idx < panes.len() {
        draw_inspector(frame, panes[idx], app);
    }

    let status = if app.refreshing {
        format!("{} · refreshing", app.status)
    } else if app.loading {
        format!("{} · loading", app.status)
    } else {
        app.status.clone()
    };
    frame.render_widget(Paragraph::new(status_line(&status, &styles)), chunks[2]);
    frame.render_widget(
        Paragraph::new(footer_hints(
            &[
                ("?", "help"),
                ("ctrl+k", "commands"),
                ("r", "refresh"),
                ("tab", "pane"),
                ("q", "quit"),
            ],
            &styles,
        )),
        chunks[3],
    );
}

fn draw_nav(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let items: Vec<ListItem> = app
        .nav
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let prefix = if e.depth > 0 { "  " } else { "" };
            let marker = if i == app.nav.selected { "› " } else { "  " };
            let style = if i == app.nav.selected {
                styles.focus
            } else if e.is_group {
                styles.muted
            } else {
                styles.text
            };
            ListItem::new(Line::from(Span::styled(
                format!("{marker}{prefix}{}", e.label),
                style,
            )))
        })
        .collect();
    let border = if app.pane == Pane::Nav {
        styles.focus
    } else {
        styles.border
    };
    let list = List::new(items).block(
        Block::default()
            .title(" Nav ")
            .borders(Borders::ALL)
            .border_style(border),
    );
    frame.render_widget(list, area);
}

fn draw_table(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let border = if app.pane == Pane::Content {
        styles.focus
    } else {
        styles.border
    };
    let title = if app.table.filter.is_empty() {
        format!(" {} ", app.current_resource)
    } else {
        format!(" {}  /{} ", app.current_resource, app.table.filter)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let empty = if app.loading && app.table.row_count() == 0 {
        "Loading…"
    } else {
        "No matching resources"
    };
    let lines = app.table.lines(
        &styles,
        usize::from(inner.width),
        usize::from(inner.height),
        empty,
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_inspector(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let border = if app.pane == Pane::Inspector {
        styles.focus
    } else {
        styles.border
    };
    let block = Block::default()
        .title(" Inspector ")
        .borders(Borders::ALL)
        .border_style(border);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = constrain_lines(
        app.inspector.render_lines(&styles),
        usize::from(inner.width),
        usize::from(inner.height),
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_dashboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let palette = app.theme.palette;
    let border = if app.pane == Pane::Content {
        styles.focus
    } else {
        styles.border
    };
    let block = Block::default()
        .title(" DASHBOARD ")
        .borders(Borders::ALL)
        .border_style(border);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cores: Vec<CpuCoreView<'_>> = app
        .dash
        .cpu_core_order
        .iter()
        .map(|name| CpuCoreView {
            name,
            load: app.dash.cpu_core_loads.get(name).copied().unwrap_or(0.0),
            samples: app
                .dash
                .cpu_core_samples
                .get(name)
                .map_or(&[] as &[f64], Vec::as_slice),
        })
        .collect();
    let view = DashboardView {
        cpu_cores: &cores,
        memory_used_bytes: app.dash.memory_used_bytes,
        memory_total_bytes: app.dash.memory_total_bytes,
        memory_samples: &app.dash.memory_samples,
        wan_interface: &app.dash.traffic_interface,
        traffic_has_base: app.dash.traffic_has_base,
        rx_rate: app.dash.traffic_rx_rate,
        tx_rate: app.dash.traffic_tx_rate,
        traffic_samples: &app.dash.traffic_samples,
        firewall_rules: &app.dash.firewall_rules,
        firewall_offset: app.dash.firewall_offset,
    };
    let lines = dashboard_content(
        usize::from(inner.width),
        usize::from(inner.height),
        &view,
        &styles,
        &palette,
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

const HELP_TEXT: &str = r#"↑↓ / j k   move
pgup/pgdn   page
g / G       first / last
h / l       columns
tab         cycle panes
enter       open / inspect
/           filter
s           cycle sort
r           refresh
ctrl+k      command palette
ctrl+l      log out
?           help
q           quit

Logs: space pause · f follow · e severity · c clear local
"#;
