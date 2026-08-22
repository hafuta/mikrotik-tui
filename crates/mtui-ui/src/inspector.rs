//! Inspector viewport for a selected record.

use std::collections::HashMap;

use ratatui::text::{Line, Span};

use crate::layout::fit_line;
use crate::paint::line_on_bg;
use crate::styles::Styles;

const LEADING_KEYS: &[&str] = &[
    "name",
    "type",
    "running",
    "disabled",
    "mac-address",
    "mac",
    "comment",
];

fn value_style(value: &str, styles: &Styles) -> ratatui::style::Style {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "running" | "up" | "enabled" => styles.signal,
        "false" | "no" | "down" | "disabled" => styles.muted,
        _ => styles.text,
    }
}

fn field_order(key: &str) -> (usize, &str) {
    match LEADING_KEYS.iter().position(|lead| *lead == key) {
        Some(index) => (index, ""),
        None => (LEADING_KEYS.len(), key),
    }
}

fn add_clamped(base: usize, delta: isize, max: usize) -> usize {
    let base_i = isize::try_from(base).unwrap_or(isize::MAX);
    let max_i = isize::try_from(max).unwrap_or(isize::MAX);
    let next = base_i.saturating_add(delta).clamp(0, max_i);
    usize::try_from(next).unwrap_or(0)
}

#[derive(Debug, Clone, Default)]
pub struct InspectorState {
    pub fields: Vec<(String, String)>,
    pub selected: usize,
    pub offset: usize,
}

impl InspectorState {
    #[must_use]
    pub fn from_row(row: Option<&HashMap<String, String>>) -> Self {
        let fields = match row {
            Some(map) => {
                let mut v: Vec<_> = map
                    .iter()
                    .filter(|(key, _)| !key.starts_with('.'))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                v.sort_by(|a, b| field_order(&a.0).cmp(&field_order(&b.0)));
                v
            }
            None => Vec::new(),
        };
        Self {
            fields,
            selected: 0,
            offset: 0,
        }
    }

    /// Move the cursor by `delta` fields, clamped to the first and last rows.
    pub fn move_selection(&mut self, delta: isize, visible: usize) {
        let Some(max) = self.fields.len().checked_sub(1) else {
            self.selected = 0;
            self.offset = 0;
            return;
        };
        self.selected = add_clamped(self.selected, delta, max);
        self.ensure_selection_visible(visible);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    pub fn select_last(&mut self, visible: usize) {
        self.selected = self.fields.len().saturating_sub(1);
        self.ensure_selection_visible(visible);
    }

    /// Keep `offset` inside a `visible`-row window after the pane resizes.
    pub fn clamp_to_visible(&mut self, visible: usize) {
        self.ensure_selection_visible(visible);
    }

    /// Recompute `offset` so the cursor stays inside a `visible`-row window.
    pub fn ensure_selection_visible(&mut self, visible: usize) {
        if visible == 0 || self.fields.is_empty() {
            self.selected = 0;
            self.offset = 0;
            return;
        }
        self.selected = self.selected.min(self.fields.len() - 1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + visible {
            self.offset = self.selected + 1 - visible;
        }
        let max_start = self.fields.len().saturating_sub(visible);
        self.offset = self.offset.min(max_start);
    }

    /// Key/value pairs: muted labels, values in text/signal, a wide gutter.
    /// When `focused`, the cursor row is a bounded selection bar like tables.
    #[must_use]
    pub fn render_lines(&self, styles: &Styles, focused: bool, width: usize) -> Vec<Line<'static>> {
        let width = width.max(1);
        let key_w = self
            .fields
            .iter()
            .map(|(key, _)| key.len())
            .max()
            .unwrap_or(0)
            .clamp(1, 14);
        self.fields
            .iter()
            .enumerate()
            .skip(self.offset)
            .map(|(idx, (key, value))| {
                let line = Line::from(vec![
                    Span::styled(format!("{key:<key_w$}"), styles.muted),
                    Span::styled("   ", styles.muted),
                    Span::styled(value.clone(), value_style(value, styles)),
                ]);
                if focused && idx == self.selected {
                    fit_line(line_on_bg(line, styles.selection), width)
                } else {
                    line
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    fn styles() -> Styles {
        use mtui_core::{DefaultTheme, Theme};
        let theme = DefaultTheme::new();
        Styles::from_palette(theme.palette())
    }

    #[test]
    fn render_lines_uses_muted_label_and_text_value_styles() {
        let styles = styles();
        let mut row = HashMap::new();
        row.insert("name".to_string(), "ether1".to_string());
        let state = InspectorState::from_row(Some(&row));

        let lines = state.render_lines(&styles, false, 24);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].style, styles.muted);
        assert_eq!(lines[0].spans[2].style, styles.text);
        assert!(lines[0].spans[2].content.contains("ether1"));
        assert_eq!(lines[0].spans[1].content.as_ref(), "   ");
        assert!(lines[0].spans.iter().all(|span| span.style.bg.is_none()));
    }

    #[test]
    fn boolean_true_uses_signal_style() {
        let styles = styles();
        let mut row = HashMap::new();
        row.insert("running".to_string(), "true".to_string());
        let lines = InspectorState::from_row(Some(&row)).render_lines(&styles, false, 24);
        assert_eq!(lines[0].spans[2].style, styles.signal);
        assert_eq!(lines[0].spans[2].content.as_ref(), "true");
    }

    #[test]
    fn focused_cursor_uses_bounded_selection_fill() {
        let styles = styles();
        let mut row = HashMap::new();
        row.insert("name".to_string(), "ether1".to_string());
        row.insert("type".to_string(), "ether".to_string());
        let mut state = InspectorState::from_row(Some(&row));
        state.selected = 1;
        let lines = state.render_lines(&styles, true, 20);
        assert_eq!(crate::layout::line_width(&lines[1]), 20);
        assert!(
            lines[1]
                .spans
                .iter()
                .all(|span| span.style.bg == Some(styles.selection)),
            "cursor row must be a bounded fill: {:?}",
            lines[1]
        );
        assert!(
            lines[0].spans.iter().all(|span| span.style.bg.is_none()),
            "unselected inspector rows stay unpainted: {:?}",
            lines[0]
        );
        let unfocused = state.render_lines(&styles, false, 20);
        assert!(
            unfocused
                .iter()
                .all(|line| line.spans.iter().all(|span| span.style.bg.is_none()))
        );
    }

    #[test]
    fn move_selection_clamps_instead_of_wrapping() {
        let mut row = HashMap::new();
        row.insert("name".to_string(), "ether1".to_string());
        row.insert("type".to_string(), "ether".to_string());
        row.insert("mtu".to_string(), "1500".to_string());
        let mut state = InspectorState::from_row(Some(&row));
        state.move_selection(-4, 8);
        assert_eq!(state.selected, 0);
        state.move_selection(10, 8);
        assert_eq!(state.selected, 2);
        state.move_selection(1, 8);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn ensure_selection_visible_scrolls_the_window() {
        let mut row = HashMap::new();
        for i in 0..6 {
            row.insert(format!("field-{i}"), format!("{i}"));
        }
        let mut state = InspectorState::from_row(Some(&row));
        state.select_last(3);
        assert_eq!(state.selected, 5);
        assert_eq!(state.offset, 3);
        state.move_selection(-3, 3);
        assert_eq!(state.selected, 2);
        assert_eq!(state.offset, 2);
    }

    #[test]
    fn leading_keys_come_before_the_rest() {
        let mut row = HashMap::new();
        row.insert("comment".to_string(), "WAN uplink".to_string());
        row.insert("mtu".to_string(), "1500".to_string());
        row.insert("name".to_string(), "ether1".to_string());
        row.insert("type".to_string(), "ether".to_string());
        let keys: Vec<_> = InspectorState::from_row(Some(&row))
            .fields
            .into_iter()
            .map(|(key, _)| key)
            .collect();
        assert_eq!(keys, ["name", "type", "comment", "mtu"]);
    }
}
