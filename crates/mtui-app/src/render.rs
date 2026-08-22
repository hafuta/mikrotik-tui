//! Ratatui rendering — pure view from [`App`] state + theme styles.

use mtui_core::{DASHBOARD_ID, WHEN_YOU_NEED_IT};
use mtui_ui::{
    CpuCoreView, DashboardView, LayoutMetrics, Modal, ModalButton, ModalButtonKind, ModalPanel,
    constrain_lines, dashboard_content, fill_rect, footer_bar, format_fingerprint, header_line,
    modal_max_scroll, render_action_menu, render_form_sheet, render_modal, render_torch,
    session_header,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

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
                Overlay::About => {
                    if let Some(copy) = mtui_core::about_copy(&app.current_resource) {
                        let modal = about_modal(&copy).scroll(app.overlay_scroll);
                        render_modal(frame, area, &modal, &styles);
                    }
                }
                Overlay::Palette => {
                    app.palette.render(frame, area, &styles);
                }
                Overlay::Confirm(ref session) => {
                    let buttons = [
                        ModalButton {
                            label: "Confirm",
                            keys: "y / enter",
                            kind: ModalButtonKind::Primary,
                        },
                        ModalButton {
                            label: "Cancel",
                            keys: "n / esc",
                            kind: ModalButtonKind::Secondary,
                        },
                    ];
                    let modal = Modal::new(&session.title, &session.body)
                        .alert()
                        .buttons(&buttons);
                    render_modal(frame, area, &modal, &styles);
                }
                Overlay::HideMenu {
                    ref title,
                    ref body,
                    ..
                } => {
                    let buttons = [
                        ModalButton {
                            label: "Hide",
                            keys: "y / enter",
                            kind: ModalButtonKind::Primary,
                        },
                        ModalButton {
                            label: "Cancel",
                            keys: "n / esc",
                            kind: ModalButtonKind::Secondary,
                        },
                    ];
                    let modal = Modal::new(title, body).alert().buttons(&buttons);
                    render_modal(frame, area, &modal, &styles);
                }
                Overlay::Form(ref session) => {
                    let schema = session.overlay_schema(
                        mtui_core::resource_by_id(&session.resource_id).and_then(|spec| spec.form),
                    );
                    render_form_sheet(frame, area, session, schema, &styles);
                }
                Overlay::ActionMenu(ref menu) | Overlay::TypePicker(ref menu) => {
                    render_action_menu(frame, area, menu, &styles);
                }
                Overlay::Torch(ref torch) => {
                    render_torch(frame, area, torch, &styles);
                }
                Overlay::None => {}
            }
        }
    }
}

fn draw_login(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    fill_rect(frame, area, styles.void);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    fill_rect(frame, chunks[0], styles.band);
    frame.render_widget(
        Paragraph::new(header_line("mikrotik-tui", "connect", &styles)),
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
            .border_type(BorderType::Rounded)
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
    fill_rect(frame, chunks[6], styles.inset);
    frame.render_widget(
        Paragraph::new(footer_bar(
            &app.status,
            &[("enter", "connect"), ("tab", "field"), ("q", "quit")],
            usize::from(area.width.max(1)),
            &styles,
        )),
        chunks[6],
    );
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
    fill_rect(frame, area, styles.void);

    let metrics = LayoutMetrics::new(area.width, area.height);
    let console_h = app.console_layout_height();
    let mut vertical = vec![Constraint::Length(1)];
    if app.console.fullscreen && app.console.visible {
        vertical.push(Constraint::Min(3));
    } else {
        vertical.push(Constraint::Min(3));
        if console_h > 0 {
            vertical.push(Constraint::Length(console_h));
        }
    }
    vertical.push(Constraint::Length(1));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vertical)
        .split(area);

    fill_rect(frame, chunks[0], styles.band);
    let header = session_header(
        "mikrotik-tui",
        &app.session_identity(),
        &app.header_signals(),
        usize::from(area.width.max(1)),
        &styles,
        app.show_activity(),
    );
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let mut chunk_idx = 1;
    if !(app.console.fullscreen && app.console.visible) {
        let body = chunks[chunk_idx];
        chunk_idx += 1;
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
        if metrics.inspector_width > 0 && app.current_resource != DASHBOARD_ID && idx < panes.len()
        {
            draw_inspector(frame, panes[idx], app);
        }
    }

    if console_h > 0 {
        draw_console(frame, chunks[chunk_idx], app);
        chunk_idx += 1;
    }

    let mut status = app.status.clone();
    if !app.table.filter.is_empty() {
        status = format!("{status}  /{}", app.table.filter);
    }
    let hints = app.footer_action_hints();
    let hint_refs: Vec<(&str, &str)> = hints
        .iter()
        .map(|(key, label)| (key.as_str(), label.as_str()))
        .collect();
    fill_rect(frame, chunks[chunk_idx], styles.inset);
    frame.render_widget(
        Paragraph::new(footer_bar(
            &status,
            &hint_refs,
            usize::from(area.width.max(1)),
            &styles,
        )),
        chunks[chunk_idx],
    );
}

fn pane_block() -> Block<'static> {
    Block::default().padding(Padding::new(2, 2, 1, 1))
}

fn draw_console(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let focused = app.pane == Pane::Console;
    let border = if focused { styles.focus } else { styles.border };
    let block = Block::default()
        .title(app.console.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines = app.console.lines(
        &app.console_entries,
        &styles,
        usize::from(inner.width),
        usize::from(inner.height),
        focused,
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_nav(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let block = pane_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let width = usize::from(inner.width);
    let lines = constrain_lines(
        app.nav.render_lines(
            app.pane == Pane::Nav,
            Some(app.current_resource.as_str()),
            &styles,
            width,
        ),
        width,
        usize::from(inner.height),
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_table(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let block = pane_block();
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
    let block = pane_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let width = usize::from(inner.width);
    let lines = constrain_lines(
        app.inspector
            .render_lines(&styles, app.pane == Pane::Inspector, width),
        width,
        usize::from(inner.height),
    );
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_dashboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let palette = app.theme.palette;
    let block = pane_block();
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

const HELP_TEXT: &str = r"↑↓ / j k   move
pgup/pgdn   page
g / G       first / last
h / l       columns
← →         panes (after column scroll)
tab         cycle panes
`           toggle log console
enter       open / expand category; edit row; expand log
/           filter · console search (when focused)
s           cycle sort
r           refresh
e           edit
n           add
d           enable / disable
c           copy · console: copy focused log
x           remove
z           reset counters
t           torch
b           reboot (Resources) · save backup (Files)
o           shutdown (Resources; power off)
u           load backup (Files, *.backup)
a           action menu
ctrl+s      save properties
[ / ]       previous / next properties tab
1-9         jump to a properties tab (when not typing)
ctrl+k      command palette
ctrl+l      log out
-           hide menu (confirm) / restore (nav)
.           show hidden menus / done
?           help
i / F1      about this screen
q           quit

Logs: space pause · f follow · e severity · c clear local
Console: f fullscreen · pgup/pgdn · n/N next match · enter expand
Destructive actions ask for confirmation.
";

fn about_modal(copy: &mtui_core::AboutCopy) -> Modal<'_> {
    Modal::new(&copy.title, &copy.body)
        .kicker(&copy.kicker)
        .accent_heading(WHEN_YOU_NEED_IT)
        .hint("esc close · j/k scroll")
}

pub(crate) fn overlay_scroll_max(app: &App) -> u16 {
    let area = Rect::new(0, 0, app.terminal_width, app.terminal_height);
    let styles = app.styles();
    match app.overlay {
        Overlay::Help => modal_max_scroll(area, &Modal::new("Keyboard help", HELP_TEXT), &styles),
        Overlay::About => {
            let Some(copy) = mtui_core::about_copy(&app.current_resource) else {
                return 0;
            };
            let modal = about_modal(&copy);
            modal_max_scroll(area, &modal, &styles)
        }
        _ => 0,
    }
}
