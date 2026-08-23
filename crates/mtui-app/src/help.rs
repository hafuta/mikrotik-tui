//! Contextual keyboard help for the current screen and resource.

use mtui_core::{DASHBOARD_ID, action_label, resource_by_id};

use crate::app::{App, Pane, Screen};

const KEY_COL: usize = 24;

/// Shortcuts that apply right now. Page-specific keys stay off other screens.
#[must_use]
pub fn keyboard_help(app: &App) -> String {
    let mut out = String::new();
    push_section(
        &mut out,
        "Sessions",
        &[
            ("ctrl+t", "new device tab"),
            ("ctrl+w", "close tab"),
            ("ctrl+tab / ctrl+pgdn", "next tab"),
            ("ctrl+shift+tab / ctrl+pgup", "previous tab"),
        ],
    );
    match app.screen {
        Screen::Login | Screen::Connecting => push_section(
            &mut out,
            "Login",
            &[
                ("enter", "select device, browse CA file, or connect"),
                ("tab", "next field"),
                ("n", "new device"),
                ("x", "forget device"),
                ("space", "remember password or TLS"),
                ("esc", "cancel connect"),
            ],
        ),
        Screen::Trust => push_section(
            &mut out,
            "Certificate",
            &[
                ("y / enter", "trust this fingerprint"),
                ("n / esc", "reject and go back"),
            ],
        ),
        Screen::Main => push_main_help(&mut out, app),
    }
    push_section(
        &mut out,
        "App",
        &[
            ("?", "this help"),
            ("i / F1", "about this screen"),
            ("ctrl+k", "command palette"),
            ("ctrl+l", "log out (keeps saved devices)"),
            ("q", "quit"),
        ],
    );
    out.push_str("Destructive actions ask for confirmation.\n");
    out
}

fn push_main_help(out: &mut String, app: &App) {
    push_section(
        out,
        "Panes",
        &[
            ("tab / shift+tab", "cycle panes"),
            ("← →", "panes after column scroll"),
            ("`", "toggle log console"),
            ("r", "refresh"),
        ],
    );
    if app.current_resource == DASHBOARD_ID {
        push_section(
            out,
            "Dashboard",
            &[("j k / ↑ ↓", "move"), ("pgup / pgdn", "page")],
        );
    } else if app.current_resource == "logs" {
        push_section(
            out,
            "Logs",
            &[
                ("j k / ↑ ↓", "move"),
                ("pgup / pgdn", "page"),
                ("g / G", "first / last"),
                ("/", "filter"),
                ("space", "pause"),
                ("f", "follow"),
                ("e", "severity"),
                ("c", "clear local"),
            ],
        );
    } else {
        let mut rows = vec![
            ("j k / ↑ ↓", "move"),
            ("pgup / pgdn", "page"),
            ("h / l", "scroll columns"),
            ("/", "filter"),
            ("s", "cycle sort"),
            ("enter", "open or edit"),
            ("y", "copy row or inspector"),
            ("Y", "copy filtered table"),
        ];
        if app.action_key_consumed('g') {
            rows.insert(2, ("G / Home", "last / first"));
        } else {
            rows.insert(2, ("g / G", "first / last"));
        }
        if mtui_core::supports_bulk_select(&app.current_resource) {
            rows.push(("space", "check row"));
            rows.push(("*", "check all filtered"));
        }
        push_section(out, "Table", &rows);
        push_page_actions(out, app);
    }
    if app.pane == Pane::Nav {
        push_section(
            out,
            "Menu",
            &[
                ("enter", "open category or page"),
                ("-", "hide or restore this menu"),
                (".", "show hidden menus"),
            ],
        );
    }
    if app.console.visible || app.pane == Pane::Console {
        push_section(
            out,
            "Console",
            &[
                ("f", "fullscreen"),
                ("/", "search"),
                ("n / N", "next / previous match"),
                ("enter", "expand"),
                ("h / l", "inspect JSON"),
                ("c", "copy focused log"),
                ("`", "hide"),
            ],
        );
    }
}

fn push_page_actions(out: &mut String, app: &App) {
    let Some(spec) = resource_by_id(&app.current_resource) else {
        return;
    };
    let row = app.table.selected_row();
    let mut rows = Vec::new();
    for action in spec.resolved_actions(row) {
        let Some(key) = action.key else {
            continue;
        };
        if !app.action_offered_in_pane(action) {
            continue;
        }
        rows.push((key.to_string(), action_label(action, row)));
    }
    if app.pane_allows_row_actions() && !spec.actions.is_empty() {
        rows.push(("a".into(), "action menu".into()));
    }
    if rows.is_empty() {
        return;
    }
    let owned: Vec<(String, String)> = rows;
    let refs: Vec<(&str, &str)> = owned
        .iter()
        .map(|(key, label)| (key.as_str(), label.as_str()))
        .collect();
    push_section(out, spec.label, &refs);
}

fn push_section(out: &mut String, title: &str, rows: &[(&str, &str)]) {
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(title);
    out.push('\n');
    for (key, label) in rows {
        out.push_str("  ");
        out.push_str(key);
        let pad = KEY_COL.saturating_sub(key.len()).max(1);
        out.push_str(&" ".repeat(pad));
        out.push_str(label);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use mtui_core::DASHBOARD_ID;

    use super::keyboard_help;
    use crate::app::{App, Pane, Screen};

    fn main_app(resource: &str) -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.current_resource = resource.to_string();
        app.pane = Pane::Content;
        app
    }

    #[test]
    fn dashboard_help_omits_certificate_keys() {
        let app = main_app(DASHBOARD_ID);
        let text = keyboard_help(&app);
        assert!(text.contains("Sessions"), "{text}");
        assert!(text.contains("Dashboard"), "{text}");
        assert!(!text.contains("sign"), "{text}");
        assert!(!text.contains("forget device"), "{text}");
        assert!(!text.contains("trust this fingerprint"), "{text}");
    }

    #[test]
    fn certificates_help_lists_sign_and_skips_login_keys() {
        let app = main_app("certificates");
        let text = keyboard_help(&app);
        assert!(text.contains("Certificates"), "{text}");
        assert!(text.contains("Import") || text.contains("import"), "{text}");
        assert!(!text.contains("forget device"), "{text}");
        assert!(!text.contains("remember password"), "{text}");
    }

    #[test]
    fn login_help_stays_on_device_list_keys() {
        let app = App::new(false).expect("app");
        let text = keyboard_help(&app);
        assert!(text.contains("new device"), "{text}");
        assert!(!text.contains("cycle sort"), "{text}");
        assert!(!text.contains("certificates"), "{text}");
    }
}
