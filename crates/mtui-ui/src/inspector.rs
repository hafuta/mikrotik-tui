//! Inspector viewport for a selected record.

use std::collections::HashMap;

use ratatui::text::{Line, Span};

use crate::styles::Styles;

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
                v.sort_by(|a, b| a.0.cmp(&b.0));
                v
            }
            None => Vec::new(),
        };
        // silence unused mut warning path
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

    /// Build styled key/value lines for the current scroll offset. Labels
    /// use `styles.muted`, values use `styles.text` (foreground only).
    #[must_use]
    pub fn render_lines(&self, styles: &Styles) -> Vec<Line<'static>> {
        self.fields
            .iter()
            .skip(self.offset)
            .map(|(key, value)| {
                Line::from(vec![
                    Span::styled(format!("{key:<20} "), styles.muted),
                    Span::styled(value.clone(), styles.text),
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
        assert_eq!(lines[0].spans[1].style, styles.text);
        assert!(lines[0].spans[1].content.contains("ether1"));
    }
}
