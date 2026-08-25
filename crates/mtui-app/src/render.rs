//! Ratatui rendering — pure view from [`App`] state + theme styles.

use mtui_core::{DASHBOARD_ID, WHEN_YOU_NEED_IT};
use mtui_ui::{
    CpuCoreView, DashboardView, LayoutMetrics, LoginView, Modal, ModalButton, ModalButtonKind,
    ModalPanel, ReauthView, TabLabel, center_in_band, chrome_band_height, constrain_lines,
    dashboard_content, fill_rect, footer_bar, format_fingerprint, modal_max_scroll,
    render_action_menu, render_file_picker, render_form_sheet, render_login, render_modal,
    render_probe, render_reauth, render_tab_bar, render_torch, session_header, tab_strip_height,
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
                Overlay::ActionMenu(ref menu) => {
                    render_action_menu(frame, full, menu, &styles, "Actions");
                }
                Overlay::TypePicker(ref menu) => {
                    render_action_menu(frame, full, menu, &styles, "New interface");
                }
                Overlay::Torch(ref torch) => {
                    render_torch(frame, full, torch, &styles);
                }
                Overlay::Probe(ref probe) => {
                    render_probe(frame, full, probe, &styles);
                }
                Overlay::FilePicker(ref picker) => {
                    render_file_picker(frame, full, picker, &styles);
                }
                Overlay::SafeModeConflict {
                    ref owner,
                    ref user,
                } => draw_safe_mode_conflict(frame, full, owner, user, &styles),
                Overlay::SafeModeLeave { .. } => draw_safe_mode_leave(frame, full, &styles),
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
        .map(|session| TabLabel::new(session.id.get(), session.tab_title(), session.is_live()))
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
        Overlay::FilePicker(picker) => render_file_picker(frame, area, picker, &styles),
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

fn draw_safe_mode_conflict(
    frame: &mut Frame<'_>,
    area: Rect,
    owner: &str,
    user: &str,
    styles: &mtui_ui::Styles,
) {
    let holder = match (owner, user) {
        ("", "") => "another session".to_string(),
        (owner, "") => owner.to_string(),
        ("", user) => user.to_string(),
        (owner, user) => format!("{owner} ({user})"),
    };
    let body = format!(
        "Safe Mode is held by {holder}.\n\nUnroll undoes that session's pending changes and takes Safe Mode. Keep takes Safe Mode and leaves those changes. Leave does nothing."
    );
    let buttons = [
        ModalButton {
            label: "Unroll",
            keys: "u",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Keep",
            keys: "r",
            kind: ModalButtonKind::Secondary,
        },
        ModalButton {
            label: "Leave",
            keys: "d / esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let modal = Modal::new("Safe Mode taken", &body)
        .alert()
        .kicker("One owner at a time")
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

fn draw_safe_mode_leave(frame: &mut Frame<'_>, area: Rect, styles: &mtui_ui::Styles) {
    let body = "This tab holds Safe Mode. Unroll undoes tagged changes. Keep commits them, then this tab can close.";
    let buttons = [
        ModalButton {
            label: "Unroll",
            keys: "u / enter",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Keep",
            keys: "r",
            kind: ModalButtonKind::Secondary,
        },
        ModalButton {
            label: "Stay",
            keys: "esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let modal = Modal::new("Leave Safe Mode", body)
        .alert()
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
        &app.safe_mode_signals(),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_routeros::Resource;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::draw;
    use crate::app::{App, Overlay, Screen};
    use crate::event::{AppEvent, WorkerMsg};
    use crate::safe_mode::SafeModeAfter;
    use crate::session::LinkState;

    fn live_main() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.link = LinkState::Live;
        app.terminal_width = 80;
        app.terminal_height = 24;
        app
    }

    fn canvas(app: &App) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal.draw(|frame| draw(frame, app)).expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        rendered
    }

    #[test]
    fn conflict_overlay_lists_unroll_keep_and_leave() {
        let mut app = live_main();
        app.overlay = Overlay::SafeModeConflict {
            owner: "api".into(),
            user: "admin".into(),
        };
        let rendered = canvas(&app);
        assert!(rendered.contains("Safe Mode taken"), "{rendered}");
        assert!(rendered.contains("One owner at a time"), "{rendered}");
        assert!(rendered.contains("api (admin)"), "{rendered}");
        assert!(rendered.contains("[ Unroll ]"), "{rendered}");
        assert!(rendered.contains("[ Keep ]"), "{rendered}");
        assert!(rendered.contains("[ Leave ]"), "{rendered}");
        assert!(rendered.contains("d / esc"), "{rendered}");
    }

    #[test]
    fn leave_overlay_lists_unroll_keep_and_stay() {
        let mut app = live_main();
        app.overlay = Overlay::SafeModeLeave {
            next: SafeModeAfter::Quit,
        };
        let rendered = canvas(&app);
        assert!(rendered.contains("Leave Safe Mode"), "{rendered}");
        assert!(rendered.contains("[ Unroll ]"), "{rendered}");
        assert!(rendered.contains("[ Keep ]"), "{rendered}");
        assert!(rendered.contains("[ Stay ]"), "{rendered}");
        assert!(rendered.contains("u / enter"), "{rendered}");
    }

    #[test]
    fn bulk_confirm_on_interfaces_is_a_centered_overlay() {
        fn row(id: &str, name: &str) -> Resource {
            let mut fields = HashMap::new();
            fields.insert("name".into(), name.into());
            Resource {
                id: id.into(),
                fields,
            }
        }

        let mut app = live_main();
        app.select_resource("interfaces");
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: vec![row("*1", "ether1"), row("*2", "ether2")],
            error: None,
        }));
        app.pane = crate::app::Pane::Content;
        let press = |code| KeyEvent::new(code, KeyModifiers::NONE);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        app.table.move_selection(1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('d'))));

        let rendered = canvas(&app);
        assert!(rendered.contains("ether1"), "{rendered}");
        assert!(rendered.contains("2 items"), "{rendered}");
        assert!(rendered.contains("[ Confirm ]"), "{rendered}");
        assert!(rendered.contains("[ Cancel ]"), "{rendered}");
        let first = rendered.lines().next().expect("row");
        assert!(
            !first.contains("2 items"),
            "confirm dialog should not append below the layout: {rendered}"
        );
    }

    #[test]
    fn narrow_interfaces_table_does_not_panic_with_checks() {
        let mut app = live_main();
        app.terminal_width = 40;
        app.terminal_height = 10;
        app.select_resource("interfaces");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "ether1".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "interfaces".into(),
            rows: vec![Resource {
                id: "*1".into(),
                fields,
            }],
            error: None,
        }));
        app.table.toggle_checked();
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| draw(frame, &app))
            .expect("narrow draw");
    }

    #[test]
    fn ipv6_firewall_connections_table_and_remove_overlay() {
        let mut app = live_main();
        app.select_resource("ipv6-firewall-connections");
        let mut fields = HashMap::new();
        fields.insert("src-address".into(), "2001:db8:1::10".into());
        fields.insert("dst-address".into(), "2001:db8:2::1".into());
        fields.insert("protocol".into(), "tcp".into());
        fields.insert("src-port".into(), "53100".into());
        fields.insert("dst-port".into(), "443".into());
        fields.insert("tcp-state".into(), "established".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "ipv6-firewall-connections".into(),
            rows: vec![Resource {
                id: "*36".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = crate::app::Pane::Content;
        let populated = canvas(&app);
        assert!(populated.contains("2001:db8:1::10"), "{populated}");
        assert!(populated.contains("2001:db8:2::1"), "{populated}");
        assert!(populated.contains("x Remove"), "{populated}");
        assert!(!populated.contains("[ Confirm ]"), "{populated}");

        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::NONE,
        )));
        let rendered = canvas(&app);
        assert!(rendered.contains("[ Confirm ]"), "{rendered}");
        assert!(rendered.contains("[ Cancel ]"), "{rendered}");
        let first = rendered.lines().next().expect("row");
        assert!(
            !first.contains("Confirm"),
            "remove dialog should not append below the layout: {rendered}"
        );

        app.terminal_width = 40;
        app.terminal_height = 12;
        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| draw(frame, &app))
            .expect("narrow connections draw");
    }

    #[test]
    fn ospf_interface_inspect_overlay_is_centered_status_sheet() {
        let mut app = live_main();
        app.select_resource("ospf-interfaces");
        let mut fields = HashMap::new();
        fields.insert("address".into(), "10.1.1.1%ether1".into());
        fields.insert("area".into(), "backbone".into());
        fields.insert("state".into(), "dr".into());
        fields.insert("network-type".into(), "broadcast".into());
        fields.insert("cost".into(), "10".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "ospf-interfaces".into(),
            rows: vec![Resource {
                id: "*1".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = crate::app::Pane::Content;
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.overlay, Overlay::Form(_)));
        let rendered = canvas(&app);
        assert!(rendered.contains("Address"), "{rendered}");
        assert!(rendered.contains("Network Type"), "{rendered}");
        assert!(rendered.contains("10.1.1.1%ether1"), "{rendered}");
        assert!(!rendered.contains("[1 Status]"), "{rendered}");
        assert!(rendered.contains("esc"), "{rendered}");
    }

    #[test]
    fn romon_and_graphing_sheets_render_with_pinned_hints() {
        let mut app = live_main();
        app.select_resource("romon");
        let mut fields = HashMap::new();
        fields.insert("enabled".into(), "true".into());
        fields.insert("id".into(), "00:00:00:00:00:00".into());
        fields.insert("secrets".into(), "shared".into());
        fields.insert("current-id".into(), "74:4D:28:00:00:01".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "romon".into(),
            rows: vec![Resource {
                id: String::new(),
                fields,
            }],
            error: None,
        }));
        app.pane = crate::app::Pane::Content;
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(app.overlay, Overlay::Form(_)));
        let rendered = canvas(&app);
        assert!(rendered.contains("Enabled"), "{rendered}");
        assert!(rendered.contains("Secrets"), "{rendered}");
        assert!(rendered.contains("Status"), "{rendered}");
        assert!(!rendered.contains("shared"), "{rendered}");
        assert!(rendered.contains("space toggle"), "{rendered}");
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));

        app.select_resource("graphing");
        let mut gfields = HashMap::new();
        gfields.insert("store-every".into(), "5min".into());
        gfields.insert("page-refresh".into(), "300".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "graphing".into(),
            rows: vec![Resource {
                id: String::new(),
                fields: gfields,
            }],
            error: None,
        }));
        let _ = app.update(AppEvent::Input(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        let rendered = canvas(&app);
        assert!(rendered.contains("Store Every"), "{rendered}");
        assert!(rendered.contains("Page Refresh"), "{rendered}");
        assert!(rendered.contains("space pick"), "{rendered}");
        assert!(rendered.contains("esc"), "{rendered}");
    }
}
