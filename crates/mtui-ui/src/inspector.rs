//! Inspector viewport for a selected record.

use std::collections::HashMap;

use ratatui::text::{Line, Span};

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

#[derive(Debug, Clone, Default)]
pub struct InspectorState {
    pub fields: Vec<(String, String)>,
    pub offset: usize,
}

impl InspectorState {
    #[must_use]
    pub fn from_row(row: Option<&HashMap<String, String>>) -> Self {
        let mut fields = match row {
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
        let _ = &mut fields;
        Self { fields, offset: 0 }
    }

    pub fn scroll_by(&mut self, delta: isize, visible: usize) {
        let max = self.fields.len().saturating_sub(visible.max(1));
        let cur = isize::try_from(self.offset).unwrap_or(0);
        let max_i = isize::try_from(max).unwrap_or(0);
        let next = cur.saturating_add(delta).clamp(0, max_i);
        self.offset = usize::try_from(next).unwrap_or(0);
    }

    /// Keep `offset` inside a `visible`-row window after the pane resizes.
    pub fn clamp_to_visible(&mut self, visible: usize) {
        let max = self.fields.len().saturating_sub(visible.max(1));
        self.offset = self.offset.min(max);
    }

    /// Key/value pairs: muted labels, values in text/signal, a wide gutter.
    #[must_use]
    pub fn render_lines(&self, styles: &Styles) -> Vec<Line<'static>> {
        let key_w = self
            .fields
            .iter()
            .map(|(key, _)| key.len())
            .max()
            .unwrap_or(0)
            .clamp(1, 14);
        self.fields
            .iter()
            .skip(self.offset)
            .map(|(key, value)| {
                Line::from(vec![
                    Span::styled(format!("{key:<key_w$}"), styles.muted),
                    Span::styled("   ", styles.muted),
                    Span::styled(value.clone(), value_style(value, styles)),
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn render_lines_uses_muted_label_and_text_value_styles() {
        use mtui_core::{DefaultTheme, Theme};
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());

        let mut row = HashMap::new();
        row.insert("name".to_string(), "ether1".to_string());
        let state = InspectorState::from_row(Some(&row));

        let lines = state.render_lines(&styles);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].spans[0].style, styles.muted);
        assert_eq!(lines[0].spans[2].style, styles.text);
        assert!(lines[0].spans[2].content.contains("ether1"));
        assert_eq!(lines[0].spans[1].content.as_ref(), "   ");
        assert!(lines[0].spans.iter().all(|span| span.style.bg.is_none()));
    }

    #[test]
    fn boolean_true_uses_signal_style() {
        use mtui_core::{DefaultTheme, Theme};
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let mut row = HashMap::new();
        row.insert("running".to_string(), "true".to_string());
        let lines = InspectorState::from_row(Some(&row)).render_lines(&styles);
        assert_eq!(lines[0].spans[2].style, styles.signal);
        assert_eq!(lines[0].spans[2].content.as_ref(), "true");
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
