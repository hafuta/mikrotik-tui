//! Navigation tree state and rendering.

use mtui_core::NavItem;
use ratatui::text::{Line, Span};

use crate::styles::Styles;

#[derive(Debug, Clone)]
pub struct FlatNavEntry {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub is_group: bool,
}

#[must_use]
pub fn flatten_nav(items: &[NavItem]) -> Vec<FlatNavEntry> {
    let mut out = Vec::new();
    for item in items {
        let is_group = !item.children.is_empty();
        out.push(FlatNavEntry {
            id: item.id.clone(),
            label: item.label.clone(),
            depth: 0,
            is_group,
        });
        for child in &item.children {
            out.push(FlatNavEntry {
                id: child.id.clone(),
                label: child.label.clone(),
                depth: 1,
                is_group: false,
            });
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct NavState {
    pub entries: Vec<FlatNavEntry>,
    pub selected: usize,
}

impl NavState {
    #[must_use]
    pub fn new(items: &[NavItem]) -> Self {
        Self {
            entries: flatten_nav(items),
            selected: 0,
        }
    }

    pub fn move_by(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let step = delta.signum();
        if step == 0 {
            return;
        }
        let Some(len) = isize::try_from(self.entries.len()).ok() else {
            return;
        };
        let Some(selected) = isize::try_from(self.selected).ok() else {
            return;
        };
        let mut idx = selected.saturating_add(delta);
        // Skip group headers in the same direction. Stop at the first and last
        // selectable items instead of wrapping.
        for _ in 0..self.entries.len() {
            if idx < 0 || idx >= len {
                return;
            }
            let Ok(index) = usize::try_from(idx) else {
                return;
            };
            if !self.entries[index].is_group {
                self.selected = index;
                return;
            }
            idx += step;
        }
    }

    #[must_use]
    pub fn selected_id(&self) -> Option<&str> {
        self.entries.get(self.selected).map(|e| e.id.as_str())
    }

    /// Select `id` so command-palette navigation reveals the matching row.
    pub fn select_id(&mut self, id: &str) -> bool {
        match self.entries.iter().position(|entry| entry.id == id) {
            Some(idx) => {
                self.selected = idx;
                true
            }
            None => false,
        }
    }

    /// Build styled lines for every row, reflecting the current selection
    /// and whether the navigation pane holds focus. Foreground-only styling
    /// (per shared-style theme rules): the selected row is conveyed via a
    /// leading marker and the theme's focus color, never a background fill.
    #[must_use]
    pub fn render_lines(&self, focused: bool, styles: &Styles) -> Vec<Line<'static>> {
        self.entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = idx == self.selected;
                let marker = if is_selected {
                    if focused { "> " } else { "- " }
                } else {
                    "  "
                };
                let indent = "  ".repeat(entry.depth);
                let style = if is_selected {
                    if focused { styles.focus } else { styles.text }
                } else if entry.is_group {
                    styles.title
                } else {
                    styles.text
                };
                Line::from(Span::styled(
                    format!("{marker}{indent}{}", entry.label),
                    style,
                ))
            })
            .collect()
    }
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
                children: vec![NavItem {
                    id: "bridge-vlans".into(),
                    label: "VLANs".into(),
                    children: vec![],
                }],
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

    #[test]
    fn move_up_skips_group_header_into_previous_group() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        state.move_by(-1);
        assert_eq!(state.selected_id(), Some("bridge-vlans"));
    }

    #[test]
    fn move_down_skips_group_header_into_next_group() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("bridge-vlans"));
        state.move_by(1);
        assert_eq!(state.selected_id(), Some("arp"));
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
    fn selected_and_focused_row_uses_focus_style() {
        use mtui_core::{DefaultTheme, Theme};
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        let lines = state.render_lines(true, &styles);
        let selected = state.selected;
        assert_eq!(lines[selected].spans[0].style, styles.focus);
        assert!(lines[selected].spans[0].content.starts_with("> "));
    }

    #[test]
    fn group_rows_use_title_style_when_not_selected() {
        use mtui_core::{DefaultTheme, Theme};
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let state = NavState::new(&tree());
        let lines = state.render_lines(true, &styles);
        assert_eq!(lines[1].spans[0].style, styles.title);
    }

    #[test]
    fn select_id_reveals_nested_resource() {
        let mut state = NavState::new(&tree());
        assert!(state.select_id("arp"));
        assert_eq!(state.selected_id(), Some("arp"));
        assert!(!state.select_id("missing"));
        assert_eq!(state.selected_id(), Some("arp"));
    }
}
