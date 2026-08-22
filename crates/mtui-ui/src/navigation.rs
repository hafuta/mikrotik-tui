//! Navigation tree state and rendering.

use mtui_core::NavItem;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::styles::Styles;

#[derive(Debug, Clone)]
pub struct FlatNavEntry {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub is_group: bool,
    pub expanded: bool,
}

#[must_use]
pub fn flatten_nav(items: &[NavItem], expanded: Option<&str>) -> Vec<FlatNavEntry> {
    let mut out = Vec::new();
    for item in items {
        let is_group = !item.children.is_empty();
        let is_expanded = is_group && expanded == Some(item.id.as_str());
        out.push(FlatNavEntry {
            id: item.id.clone(),
            label: item.label.clone(),
            depth: 0,
            is_group,
            expanded: is_expanded,
        });
        if !is_expanded {
            continue;
        }
        for child in &item.children {
            out.push(FlatNavEntry {
                id: child.id.clone(),
                label: child.label.clone(),
                depth: 1,
                is_group: false,
                expanded: false,
            });
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct NavState {
    pub tree: Vec<NavItem>,
    pub entries: Vec<FlatNavEntry>,
    pub selected: usize,
    pub expanded: Option<String>,
}

impl NavState {
    #[must_use]
    pub fn new(items: &[NavItem]) -> Self {
        let mut state = Self {
            tree: items.to_vec(),
            entries: Vec::new(),
            selected: 0,
            expanded: None,
        };
        state.rebuild();
        state
    }

    fn rebuild(&mut self) {
        self.entries = flatten_nav(&self.tree, self.expanded.as_deref());
        if self.entries.is_empty() {
            self.selected = 0;
            return;
        }
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
    }

    pub fn move_by(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        if delta == 0 {
            return;
        }
        let Some(len) = isize::try_from(self.entries.len()).ok() else {
            return;
        };
        let Some(selected) = isize::try_from(self.selected).ok() else {
            return;
        };
        let idx = selected
            .saturating_add(delta)
            .clamp(0, len.saturating_sub(1));
        if let Ok(index) = usize::try_from(idx) {
            self.selected = index;
        }
    }

    #[must_use]
    pub fn selected_id(&self) -> Option<&str> {
        self.entries.get(self.selected).map(|e| e.id.as_str())
    }

    /// Select `id`, expanding its category (and collapsing others) so the
    /// matching row is visible. Group ids select that group's first child.
    pub fn select_id(&mut self, id: &str) -> bool {
        let Some(target) = self.reveal_target(id) else {
            return false;
        };
        self.expanded.clone_from(&target.expanded);
        self.rebuild();
        match self
            .entries
            .iter()
            .position(|entry| entry.id == target.selected)
        {
            Some(idx) => {
                self.selected = idx;
                true
            }
            None => false,
        }
    }

    fn reveal_target(&self, id: &str) -> Option<RevealTarget> {
        for item in &self.tree {
            if item.id != id {
                continue;
            }
            if item.children.is_empty() {
                return Some(RevealTarget {
                    selected: item.id.clone(),
                    expanded: None,
                });
            }
            let selected = item
                .children
                .first()
                .map_or_else(|| item.id.clone(), |child| child.id.clone());
            return Some(RevealTarget {
                selected,
                expanded: Some(item.id.clone()),
            });
        }
        for item in &self.tree {
            if item.children.iter().any(|child| child.id == id) {
                return Some(RevealTarget {
                    selected: id.to_string(),
                    expanded: Some(item.id.clone()),
                });
            }
        }
        None
    }

    /// Build styled lines for every visible row.
    ///
    /// `cursor` (`selected`) is the keyboard highlight; `viewed_id` is the
    /// resource currently shown in the content pane. Those can differ while
    /// browsing the tree before Enter. Foreground-only styling: markers plus
    /// color so state is not color-only, never a background fill.
    #[must_use]
    pub fn render_lines(
        &self,
        focused: bool,
        viewed_id: Option<&str>,
        styles: &Styles,
    ) -> Vec<Line<'static>> {
        self.entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                nav_row_line(
                    entry,
                    idx == self.selected,
                    viewed_id == Some(entry.id.as_str()),
                    focused,
                    styles,
                )
            })
            .collect()
    }
}

fn nav_row_line(
    entry: &FlatNavEntry,
    is_cursor: bool,
    is_viewed: bool,
    pane_focused: bool,
    styles: &Styles,
) -> Line<'static> {
    let chevron = if entry.is_group {
        if entry.expanded { "▾ " } else { "▸ " }
    } else if entry.depth == 0 {
        "  "
    } else {
        ""
    };
    let indent = "  ".repeat(entry.depth);
    let body = format!("{chevron}{indent}{}", entry.label);
    let viewed = styles.signal.add_modifier(Modifier::BOLD);
    let body_style = if is_viewed {
        viewed
    } else if is_cursor && pane_focused {
        styles.focus
    } else if entry.is_group {
        styles.title
    } else if is_cursor {
        styles.muted
    } else {
        styles.text
    };
    let mut spans = Vec::with_capacity(3);
    if is_cursor && is_viewed && pane_focused {
        spans.push(Span::styled(">", styles.focus));
        spans.push(Span::styled("●", viewed));
    } else if is_viewed {
        spans.push(Span::styled("● ", viewed));
    } else if is_cursor && pane_focused {
        spans.push(Span::styled("> ", styles.focus));
    } else if is_cursor {
        spans.push(Span::styled("- ", styles.muted));
    } else {
        spans.push(Span::styled("  ", styles.text));
    }
    spans.push(Span::styled(body, body_style));
    Line::from(spans)
}

struct RevealTarget {
    selected: String,
    expanded: Option<String>,
}

#[cfg(test)]
mod render_tests {
    use super::*;

    fn tree() -> Vec<NavItem> {
        vec![
            NavItem {
                id: "dashboard".into(),
                label: "Dashboard".into(),
                children: vec![],
            },
            NavItem {
                id: "bridge-group".into(),
                label: "Bridge".into(),
                children: vec![
                    NavItem {
                        id: "bridges".into(),
                        label: "Bridge".into(),
                        children: vec![],
                    },
                    NavItem {
                        id: "bridge-vlans".into(),
                        label: "VLANs".into(),
                        children: vec![],
                    },
                ],
            },
            NavItem {
                id: "ip-group".into(),
                label: "IP".into(),
                children: vec![NavItem {
                    id: "arp".into(),
                    label: "ARP".into(),
                    children: vec![],
                }],
            },
        ]
    }

    fn visible_ids(state: &NavState) -> Vec<&str> {
        state
            .entries
            .iter()
            .map(|entry| entry.id.as_str())
            .collect()
    }

    fn styles() -> Styles {
        use mtui_core::{DefaultTheme, Theme};
        Styles::from_palette(DefaultTheme::new().palette())
    }

    fn line_text(line: &Line<'_>) -> String {
        crate::layout::line_plain(line)
    }

    #[test]
    fn starts_collapsed_with_groups_selectable() {
        let state = NavState::new(&tree());
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group"]
        );
        assert_eq!(state.selected_id(), Some("dashboard"));
        assert!(state.expanded.is_none());
    }

    #[test]
    fn move_down_selects_group_headers() {
        let mut state = NavState::new(&tree());
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("bridge-group"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
    }

    #[test]
    fn entering_a_group_expands_it_and_selects_the_first_child() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        assert_eq!(state.expanded.as_deref(), Some("bridge-group"));
        assert_eq!(state.selected_id(), Some("bridges"));
        assert_eq!(
            visible_ids(&state),
            [
                "dashboard",
                "bridge-group",
                "bridges",
                "bridge-vlans",
                "ip-group"
            ]
        );
    }

    #[test]
    fn expanding_another_group_collapses_the_previous() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        assert!(state.select_id("ip-group"));
        assert_eq!(state.expanded.as_deref(), Some("ip-group"));
        assert_eq!(state.selected_id(), Some("arp"));
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group", "arp"]
        );
        assert!(!visible_ids(&state).contains(&"bridges"));
    }

    #[test]
    fn move_between_expanded_children_and_next_group() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridges"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("ip-group"));
        state.move_by(-1);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
    }

    #[test]
    fn move_up_from_first_item_stays_put() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("dashboard"));
        state.move_by(-1);
        assert_eq!(state.selected_id(), Some("dashboard"));
    }

    #[test]
    fn move_down_from_last_item_stays_put() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("arp"));
    }

    #[test]
    fn cursor_on_unopened_row_uses_focus_marker() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        state.move_by(1);
        let lines = state.render_lines(true, Some("dashboard"), &styles);
        let selected = state.selected;
        assert_eq!(state.selected_id(), Some("bridge-group"));
        assert_eq!(lines[selected].spans[0].style, styles.focus);
        assert_eq!(lines[selected].spans[0].content, "> ");
        assert!(line_text(&lines[selected]).starts_with("> ▸ "));
        assert!(line_text(&lines[0]).starts_with("● "));
        assert_eq!(
            lines[0].spans.last().map(|span| span.style),
            Some(styles.signal.add_modifier(Modifier::BOLD))
        );
    }

    #[test]
    fn viewed_and_focused_row_combines_markers() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        let lines = state.render_lines(true, Some("arp"), &styles);
        let selected = state.selected;
        assert_eq!(lines[selected].spans[0].content, ">");
        assert_eq!(lines[selected].spans[0].style, styles.focus);
        assert_eq!(lines[selected].spans[1].content, "●");
        assert_eq!(
            lines[selected].spans[1].style,
            styles.signal.add_modifier(Modifier::BOLD)
        );
        assert!(line_text(&lines[selected]).starts_with(">●"));
    }

    #[test]
    fn viewed_row_keeps_open_marker_when_nav_unfocused() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        let lines = state.render_lines(false, Some("arp"), &styles);
        let selected = state.selected;
        assert!(line_text(&lines[selected]).starts_with("● "));
        assert!(!line_text(&lines[selected]).contains('>'));
    }

    #[test]
    fn group_rows_use_title_style_when_not_selected() {
        let styles = styles();
        let state = NavState::new(&tree());
        let lines = state.render_lines(true, Some("dashboard"), &styles);
        assert_eq!(
            lines[1].spans.last().map(|span| span.style),
            Some(styles.title)
        );
        assert!(line_text(&lines[1]).contains("▸ "));
        assert!(!line_text(&lines[1]).contains("▾ "));
    }

    #[test]
    fn expanded_group_uses_open_chevron() {
        let styles = styles();
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-group"));
        let lines = state.render_lines(false, Some("bridges"), &styles);
        assert!(line_text(&lines[1]).contains("▾ "));
    }

    #[test]
    fn select_id_reveals_nested_resource() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        assert_eq!(state.selected_id(), Some("arp"));
        assert_eq!(state.expanded.as_deref(), Some("ip-group"));
        assert!(!state.select_id("missing"));
        assert_eq!(state.selected_id(), Some("arp"));
    }

    #[test]
    fn selecting_dashboard_collapses_groups() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-vlans"));
        assert!(state.select_id("dashboard"));
        assert!(state.expanded.is_none());
        assert_eq!(
            visible_ids(&state),
            ["dashboard", "bridge-group", "ip-group"]
        );
    }
}
