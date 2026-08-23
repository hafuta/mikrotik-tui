//! Ratatui rendering — pure view from [`App`] state + theme styles.

use mtui_core::{DASHBOARD_ID, WHEN_YOU_NEED_IT};
use mtui_ui::{
    CpuCoreView, DashboardView, LayoutMetrics, LoginView, Modal, ModalButton, ModalButtonKind,
    ModalPanel, ReauthView, TabLabel, center_in_band, chrome_band_height, constrain_lines,
    dashboard_content, fill_rect, footer_bar, format_fingerprint, modal_max_scroll,
    render_action_menu, render_form_sheet, render_login, render_modal, render_probe, render_reauth,
    render_tab_bar, render_torch, session_header, tab_strip_height,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use crate::app::{App, Overlay, Pane, Screen};
use crate::write::ConfirmSession;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let full = frame.area();
    let styles = app.styles();
    let tab_h = tab_strip_height(full.height);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(tab_h), Constraint::Min(1)])
        .split(full);
    draw_tab_bar(frame, chunks[0], app);
    let area = chunks[1];

    match app.screen {
        Screen::Login => {
            draw_login(frame, area, app, false);
            draw_login_overlay(frame, full, app);
        }
        Screen::Connecting => {
            draw_login(frame, area, app, true);
            draw_login_overlay(frame, full, app);
        }
        Screen::Trust => draw_trust(frame, area, full, app),
        Screen::Main => {
            draw_main(frame, area, app);
            match app.overlay {
                Overlay::Help => {
                    let help = crate::help::keyboard_help(app);
                    let modal = Modal::new("Keyboard help", &help).scroll(app.overlay_scroll);
                    render_modal(frame, full, &modal, &styles);
                }
                Overlay::About => {
                    if let Some(copy) = mtui_core::about_copy(&app.current_resource) {
                        let modal = about_modal(&copy).scroll(app.overlay_scroll);
                        render_modal(frame, full, &modal, &styles);
                    }
                }
                Overlay::Palette => {
                    app.palette.render(frame, full, &styles);
                }
                Overlay::Confirm(ref session) => draw_confirm(frame, full, session, &styles),
                Overlay::HideMenu {
                    ref title,
                    ref body,
                    ..
                } => draw_hide_menu(frame, full, title, body, &styles),
                Overlay::ForgetProfile { ref name } => draw_forget(frame, full, name, &styles),
                Overlay::Reauth => draw_reauth(frame, full, app),
                Overlay::Form(ref session) => {
                    let schema = session.overlay_schema(
                        mtui_core::resource_by_id(&session.resource_id).and_then(|spec| spec.form),
                    );
                    render_form_sheet(frame, full, session, schema, &styles);
                }
                Overlay::ActionMenu(ref menu) | Overlay::TypePicker(ref menu) => {
                    render_action_menu(frame, full, menu, &styles);
                }
                Overlay::Torch(ref torch) => {
                    render_torch(frame, full, torch, &styles);
                }
                Overlay::Probe(ref probe) => {
                    render_probe(frame, full, probe, &styles);
                }
                Overlay::None => {}
            }
        }
    }
}

fn draw_tab_bar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    let tabs: Vec<TabLabel> = app
        .sessions
        .iter()
        .map(|session| {
            TabLabel::new(
                session.id.get(),
                session.tab_title(),
                session.client.is_some() || session.demo.is_some(),
            )
        })
        .collect();
    render_tab_bar(frame, area, &tabs, app.active.get(), &styles);
}

fn login_clock() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d  %H:%M:%S")
        .to_string()
}

fn draw_login(frame: &mut Frame<'_>, area: Rect, app: &App, connecting: bool) {
    let styles = app.styles();
    render_login(
        frame,
        area,
        &LoginView {
            form: &app.login,
            status: &app.status,
            connecting,
            clock: &login_clock(),
        },
        &styles,
    );
}

fn draw_login_overlay(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    match &app.overlay {
        Overlay::ForgetProfile { name } => draw_forget(frame, area, name, &styles),
        Overlay::Help => {
            let help = crate::help::keyboard_help(app);
            let modal = Modal::new("Keyboard help", &help).scroll(app.overlay_scroll);
            render_modal(frame, area, &modal, &styles);
        }
        _ => {}
    }
}

fn draw_confirm(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &ConfirmSession,
    styles: &mtui_ui::Styles,
) {
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
    render_modal(frame, area, &modal, styles);
}

fn draw_hide_menu(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    body: &str,
    styles: &mtui_ui::Styles,
) {
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
    render_modal(frame, area, &modal, styles);
}

fn draw_forget(frame: &mut Frame<'_>, area: Rect, name: &str, styles: &mtui_ui::Styles) {
    let body = format!(
        "Forget {name}?\n\nThe saved host, pin, and remembered password for this device are removed. Other routers stay."
    );
    let buttons = [
        ModalButton {
            label: "Forget",
            keys: "y / enter",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Keep",
            keys: "n / esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let modal = Modal::new("Forget device", &body)
        .alert()
        .kicker("Explicit delete")
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

fn draw_reauth(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    render_reauth(
        frame,
        area,
        &ReauthView {
            username: app.login.username.trim(),
            password_len: app.reauth.password.len(),
            totp_len: app.reauth.totp.len(),
            totp_focus: app.reauth.totp_focus,
            error: app.reauth.error.as_deref(),
        },
        &styles,
    );
}

fn draw_trust(frame: &mut Frame<'_>, area: Rect, overlay: Rect, app: &App) {
    draw_login(frame, area, app, false);
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
    render_modal(frame, overlay, &modal, &styles);
}

fn draw_main(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let styles = app.styles();
    fill_rect(frame, area, styles.void);

    let metrics = LayoutMetrics::new(area.width, area.height);
    let console_h = app.console_layout_height();
    let band = chrome_band_height(app.terminal_height);
    let mut vertical = vec![Constraint::Length(band)];
    if app.console.fullscreen && app.console.visible {
        vertical.push(Constraint::Min(3));
    } else {
        vertical.push(Constraint::Min(3));
        if console_h > 0 {
            vertical.push(Constraint::Length(console_h));
        }
    }
    vertical.push(Constraint::Length(band));
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
    frame.render_widget(
        Paragraph::new(center_in_band(
            &header,
            band,
            usize::from(area.width.max(1)),
        )),
        chunks[0],
    );

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
        Paragraph::new(center_in_band(
            &footer_bar(&status, &hint_refs, usize::from(area.width.max(1)), &styles),
            band,
            usize::from(area.width.max(1)),
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
        Overlay::Help => {
            let help = crate::help::keyboard_help(app);
            modal_max_scroll(area, &Modal::new("Keyboard help", &help), &styles)
        }
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
