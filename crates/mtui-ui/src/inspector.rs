//! Inspector viewport for a selected record.

use std::collections::{HashMap, HashSet};

use mtui_core::{FieldKind, FormSchema, field_visible};
use ratatui::text::{Line, Span};

use crate::layout::fit_line;
use crate::paint::line_on_bg;
use crate::styles::Styles;

const HEADING_MARK: &str = "\u{001D}";

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
        Self::from_row_with_schema_for(row, None, None)
    }

    #[must_use]
    pub fn from_row_with_schema(
        row: Option<&HashMap<String, String>>,
        schema: Option<&FormSchema>,
    ) -> Self {
        Self::from_row_with_schema_for(row, None, schema)
    }

    #[must_use]
    pub fn from_row_with_schema_for(
        row: Option<&HashMap<String, String>>,
        resource_id: Option<&str>,
        schema: Option<&FormSchema>,
    ) -> Self {
        let fields = match (row, schema) {
            (Some(map), Some(schema)) => schema_fields(map, resource_id, schema),
            (Some(map), None) => {
                let mut v: Vec<_> = map
                    .iter()
                    .filter(|(key, _)| !key.starts_with('.'))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                v.sort_by(|a, b| field_order(&a.0).cmp(&field_order(&b.0)));
                v
            }
            _ => Vec::new(),
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
                let line = if value == HEADING_MARK {
                    Line::from(vec![Span::styled(key.clone(), styles.text)])
                } else {
                    Line::from(vec![
                        Span::styled(format!("{key:<key_w$}"), styles.muted),
                        Span::styled("   ", styles.muted),
                        Span::styled(value.clone(), value_style(value, styles)),
                    ])
                };
                if focused && idx == self.selected {
                    fit_line(line_on_bg(line, styles.selection), width)
                } else {
                    line
                }
            })
            .collect()
    }
}

fn schema_fields(
    map: &HashMap<String, String>,
    resource_id: Option<&str>,
    schema: &FormSchema,
) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    for section in schema.sections {
        let mut rows = Vec::new();
        for field in section.fields {
            seen.insert(field.key);
            if resource_id.is_some_and(|id| !field_visible(id, field.key, map)) {
                continue;
            }
            let raw = map.get(field.key).cloned().unwrap_or_default();
            if raw.is_empty() && matches!(field.kind, FieldKind::Readonly) {
                continue;
            }
            let value = if matches!(field.kind, FieldKind::Secret) && !raw.is_empty() {
                "••••••••".to_string()
            } else {
                field.kind.display_value(&raw)
            };
            rows.push((field.label.to_string(), value));
        }
        if rows.is_empty() {
            continue;
        }
        if !section.label.is_empty() {
            fields.push((section.label.to_string(), HEADING_MARK.to_string()));
        }
        fields.extend(rows);
    }
    let mut extras: Vec<_> = map
        .iter()
        .filter(|(key, value)| {
            !key.starts_with('.')
                && !seen.contains(key.as_str())
                && !matches!(key.as_str(), "caps" | "sfp")
                && !value.is_empty()
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    extras.sort_by(|left, right| left.0.cmp(&right.0));
    if !extras.is_empty() {
        fields.push(("Other".to_string(), HEADING_MARK.to_string()));
        fields.extend(extras);
    }
    fields
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

    #[test]
    fn schema_inspector_uses_section_captions_in_webfig_order() {
        const SCHEMA: FormSchema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[
                mtui_core::FormSection {
                    id: "general",
                    label: "General",
                    read_only: false,
                    fields: &[
                        mtui_core::FieldSpec {
                            key: "disabled",
                            label: "Enabled",
                            kind: FieldKind::InvertedToggle,
                        },
                        mtui_core::FieldSpec {
                            key: "name",
                            label: "Name",
                            kind: FieldKind::Text,
                        },
                    ],
                },
                mtui_core::FormSection {
                    id: "status",
                    label: "Status",
                    read_only: true,
                    fields: &[mtui_core::FieldSpec {
                        key: "running",
                        label: "Running",
                        kind: FieldKind::Readonly,
                    }],
                },
            ],
            create_sections: &[],
        };
        let mut row = HashMap::new();
        row.insert("disabled".to_string(), "false".to_string());
        row.insert("name".to_string(), "ether1".to_string());
        row.insert("running".to_string(), "true".to_string());
        let labels: Vec<_> = InspectorState::from_row_with_schema(Some(&row), Some(&SCHEMA))
            .fields
            .into_iter()
            .map(|(label, _)| label)
            .collect();
        assert_eq!(labels, ["General", "Enabled", "Name", "Status", "Running"]);
    }

    #[test]
    fn ethernet_inspector_gates_sfp_on_print_attributes_not_name() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let named = HashMap::from([
            ("name".to_string(), "sfp1".to_string()),
            ("default-name".to_string(), "sfp1".to_string()),
        ]);
        let labels: Vec<_> =
            InspectorState::from_row_with_schema_for(Some(&named), Some("ethernet"), Some(schema))
                .fields
                .into_iter()
                .map(|(label, _)| label)
                .collect();
        assert!(!labels.iter().any(|label| label == "SFP"));
        assert!(!labels.iter().any(|label| label == "PoE"));

        let printed = HashMap::from([
            ("name".to_string(), "uplink".to_string()),
            ("sfp-shutdown-temperature".to_string(), "95C".to_string()),
        ]);
        let labels: Vec<_> = InspectorState::from_row_with_schema_for(
            Some(&printed),
            Some("ethernet"),
            Some(schema),
        )
        .fields
        .into_iter()
        .map(|(label, _)| label)
        .collect();
        assert!(labels.iter().any(|label| label == "SFP"));
        assert!(labels.iter().any(|label| label == "Ignore Rx LOS"));
    }
}
