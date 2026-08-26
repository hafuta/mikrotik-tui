//! Application log console: aligned columns, vim search, expand, copy text.

use std::collections::HashSet;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::json_tree::{self, JsonRow};
use crate::layout::{constrain_lines, fit_cell};
use crate::login::is_printable_char;
use crate::styles::Styles;

/// Timestamp column width (`YYYY-MM-DD HH:MM:SS.mmm`).
pub const TIME_COL: usize = 23;
/// Level column width (`TRACE`/`DEBUG`/`INFO`/`WARN`/`ERROR`).
pub const LEVEL_COL: usize = 5;
const COL_GAP: usize = 2;
const DETAIL_KEY_COL: usize = 14;

/// How much of the terminal height the docked console occupies.
#[must_use]
pub fn console_pane_height(terminal_height: u16, visible: bool, fullscreen: bool) -> u16 {
    if !visible {
        return 0;
    }
    let chrome = crate::chrome::tab_strip_height(terminal_height)
        .saturating_add(crate::chrome::chrome_band_height(terminal_height))
        .saturating_add(crate::chrome::chrome_band_height(terminal_height));
    let available = terminal_height.saturating_sub(chrome);
    if available == 0 {
        return 0;
    }
    if fullscreen {
        return available;
    }
    let quarter = terminal_height / 4;
    let min_body = 3;
    let max_console = available.saturating_sub(min_body);
    if max_console == 0 {
        return available.saturating_sub(1).max(1);
    }
    let floor = 6.min(max_console);
    quarter.clamp(floor, max_console)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl ConsoleLevel {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// One console row. Extra fields render only when the row is expanded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleEntry {
    pub time: String,
    pub level: ConsoleLevel,
    pub message: String,
    pub fields: Vec<(String, String)>,
}

impl ConsoleEntry {
    #[must_use]
    pub fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let needle = query.to_ascii_lowercase();
        if self.time.to_ascii_lowercase().contains(&needle)
            || self.level.as_str().to_ascii_lowercase().contains(&needle)
            || self.message.to_ascii_lowercase().contains(&needle)
        {
            return true;
        }
        self.fields.iter().any(|(key, value)| {
            key.to_ascii_lowercase().contains(&needle)
                || value.to_ascii_lowercase().contains(&needle)
        })
    }

    /// Full structured text suitable for the clipboard.
    #[must_use]
    pub fn copy_text(&self) -> String {
        let mut out = format!(
            "{}  {:<5}  {}",
            self.time,
            self.level.as_str(),
            self.message
        );
        for (key, value) in &self.fields {
            out.push('\n');
            out.push_str("  ");
            out.push_str(key);
            out.push_str(": ");
            if let Some(parsed) = json_tree::parse_container(value) {
                out.push('\n');
                for line in json_tree::pretty(&parsed).lines() {
                    out.push_str("    ");
                    out.push_str(line);
                    out.push('\n');
                }
                out.pop();
            } else {
                out.push_str(value);
            }
        }
        out
    }

    fn expanded_height(&self, json_open: &HashSet<String>) -> usize {
        1 + self.detail_rows(json_open).len()
    }

    fn detail_rows(&self, json_open: &HashSet<String>) -> Vec<DetailRow> {
        if self.fields.is_empty() {
            return vec![DetailRow::Field {
                key: "target".into(),
                value: "(none)".into(),
            }];
        }
        let mut rows = Vec::new();
        for (key, value) in &self.fields {
            if let Some(parsed) = json_tree::parse_container(value) {
                rows.extend(
                    json_tree::flatten(&parsed, key, json_open)
                        .into_iter()
                        .map(DetailRow::Json),
                );
            } else {
                rows.push(DetailRow::Field {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
        }
        rows
    }
}

#[derive(Debug, Clone)]
enum DetailRow {
    Field { key: String, value: String },
    Json(JsonRow),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConsoleState {
    pub visible: bool,
    pub fullscreen: bool,
    pub selected: usize,
    pub row_offset: usize,
    /// When true, the focused row shows extra fields. Moving between log
    /// rows keeps this flag so each iterated row is expanded.
    pub expanded: bool,
    /// `0` is the log summary. Values above that index expanded detail rows,
    /// including collapsed JSON bodies.
    pub expand_cursor: usize,
    expand_line_offset: usize,
    json_open: HashSet<String>,
    pub searching: bool,
    pub query: String,
}

impl ConsoleState {
    pub fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        if !self.visible {
            self.fullscreen = false;
            self.searching = false;
        }
        self.visible
    }

    pub fn toggle_fullscreen(&mut self) {
        if self.visible {
            self.fullscreen = !self.fullscreen;
        }
    }

    pub fn start_search(&mut self) {
        self.searching = true;
        self.query.clear();
    }

    pub fn insert_search_char(&mut self, ch: char) {
        if self.searching && is_printable_char(ch) {
            self.query.push(ch);
        }
    }

    pub fn search_backspace(&mut self) {
        if self.searching {
            self.query.pop();
        }
    }

    pub fn confirm_search(&mut self) {
        self.searching = false;
    }

    /// Esc: leave search typing, or clear an applied query.
    pub fn escape_search(&mut self) -> bool {
        if self.searching {
            self.searching = false;
            self.query.clear();
            return true;
        }
        if !self.query.is_empty() {
            self.query.clear();
            return true;
        }
        false
    }

    #[must_use]
    pub fn filtered_indices(&self, entries: &[ConsoleEntry]) -> Vec<usize> {
        entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.matches(&self.query))
            .map(|(index, _)| index)
            .collect()
    }

    pub fn clamp_selection(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.selected = 0;
            self.row_offset = 0;
            self.reset_inspect();
            return;
        }
        if self.selected >= filtered_len {
            self.selected = filtered_len - 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize, filtered_len: usize) {
        let previous = self.selected;
        let Some(max) = filtered_len.checked_sub(1) else {
            self.selected = 0;
            self.reset_inspect();
            return;
        };
        self.selected = add_clamped(self.selected, delta, max);
        if self.selected != previous {
            self.reset_inspect();
        }
    }

    pub fn page_by(&mut self, direction: isize, page: usize, filtered_len: usize) {
        let step = isize::try_from(page.max(1)).unwrap_or(1);
        self.move_selection(direction.saturating_mul(step), filtered_len);
    }

    pub fn select_first(&mut self) {
        if self.selected != 0 {
            self.reset_inspect();
        }
        self.selected = 0;
    }

    pub fn select_last(&mut self, filtered_len: usize) {
        let next = filtered_len.saturating_sub(1);
        if self.selected != next {
            self.reset_inspect();
        }
        self.selected = next;
    }

    pub fn toggle_expanded(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.expanded = false;
            self.reset_inspect();
            return;
        }
        self.expanded = !self.expanded;
        self.reset_inspect();
    }

    /// Enter: expand/collapse the log, or toggle the focused JSON node.
    pub fn activate(&mut self, entries: &[ConsoleEntry], filtered_len: usize) {
        if self.expanded && self.expand_cursor > 0 && self.toggle_json_at_cursor(entries) {
            return;
        }
        self.toggle_expanded(filtered_len);
    }

    /// Right/`l`: step into details, or expand a collapsed JSON node.
    pub fn enter_detail(&mut self, entries: &[ConsoleEntry]) {
        if !self.expanded {
            return;
        }
        let Some(entry) = self.selected_entry(entries) else {
            return;
        };
        let rows = entry.detail_rows(&self.json_open);
        if rows.is_empty() {
            return;
        }
        if self.expand_cursor == 0 {
            self.expand_cursor = 1;
            return;
        }
        if let Some(row) = self.focused_json_row(entries)
            && row.expandable
            && !row.expanded
        {
            self.json_open.insert(row.path);
            return;
        }
        let max = rows.len();
        if self.expand_cursor < max {
            self.expand_cursor += 1;
        }
    }

    /// Left/`h`: collapse an open JSON node, or step back toward the summary.
    pub fn leave_detail(&mut self, entries: &[ConsoleEntry]) {
        if !self.expanded || self.expand_cursor == 0 {
            return;
        }
        if let Some(row) = self.focused_json_row(entries)
            && row.expandable
            && row.expanded
        {
            self.json_open.remove(&row.path);
            return;
        }
        self.expand_cursor = self.expand_cursor.saturating_sub(1);
    }

    /// `j`/`k` move between logs from the summary, or among detail rows once
    /// the cursor has entered the expanded block.
    pub fn move_cursor(&mut self, delta: isize, entries: &[ConsoleEntry], filtered_len: usize) {
        if self.expanded
            && self.expand_cursor > 0
            && let Some(entry) = self.selected_entry(entries)
        {
            let max = entry.detail_rows(&self.json_open).len();
            self.expand_cursor = add_clamped(self.expand_cursor, delta, max);
            return;
        }
        self.move_selection(delta, filtered_len);
    }

    fn reset_inspect(&mut self) {
        self.expand_cursor = 0;
        self.expand_line_offset = 0;
        self.json_open.clear();
    }

    fn focused_json_row(&self, entries: &[ConsoleEntry]) -> Option<JsonRow> {
        let entry = self.selected_entry(entries)?;
        let rows = entry.detail_rows(&self.json_open);
        let index = self.expand_cursor.checked_sub(1)?;
        match rows.get(index)? {
            DetailRow::Json(row) => Some(row.clone()),
            DetailRow::Field { .. } => None,
        }
    }

    fn toggle_json_at_cursor(&mut self, entries: &[ConsoleEntry]) -> bool {
        let Some(row) = self.focused_json_row(entries) else {
            return false;
        };
        if !row.expandable {
            return false;
        }
        if row.expanded {
            self.json_open.remove(&row.path);
        } else {
            self.json_open.insert(row.path);
        }
        true
    }

    /// Jump to the next (or previous) filtered row. With an active query the
    /// filtered list is already the match set, so this is next/previous hit.
    pub fn jump_match(&mut self, forward: bool, filtered_len: usize) {
        if filtered_len == 0 {
            return;
        }
        if forward {
            self.move_selection(1, filtered_len);
        } else {
            self.move_selection(-1, filtered_len);
        }
    }

    #[must_use]
    pub fn selected_entry<'entries>(
        &self,
        entries: &'entries [ConsoleEntry],
    ) -> Option<&'entries ConsoleEntry> {
        let indices = self.filtered_indices(entries);
        indices.get(self.selected).and_then(|i| entries.get(*i))
    }

    pub fn ensure_visible(&mut self, entries: &[ConsoleEntry], viewport: usize) {
        let indices = self.filtered_indices(entries);
        self.clamp_selection(indices.len());
        if indices.is_empty() || viewport == 0 {
            self.row_offset = 0;
            self.expand_line_offset = 0;
            return;
        }
        if self.selected < self.row_offset {
            self.row_offset = self.selected;
        }
        loop {
            let used = self.span_lines(entries, &indices, self.row_offset, self.selected);
            if used <= viewport || self.row_offset >= self.selected {
                break;
            }
            self.row_offset += 1;
        }
        self.clamp_expand_cursor(entries);
        self.ensure_expand_line_visible(entries, &indices, viewport);
    }

    fn clamp_expand_cursor(&mut self, entries: &[ConsoleEntry]) {
        if !self.expanded {
            self.expand_cursor = 0;
            return;
        }
        let max = self
            .selected_entry(entries)
            .map_or(0, |entry| entry.detail_rows(&self.json_open).len());
        if self.expand_cursor > max {
            self.expand_cursor = max;
        }
    }

    fn ensure_expand_line_visible(
        &mut self,
        entries: &[ConsoleEntry],
        indices: &[usize],
        viewport: usize,
    ) {
        if !self.expanded {
            self.expand_line_offset = 0;
            return;
        }
        let Some(&entry_index) = indices.get(self.selected) else {
            self.expand_line_offset = 0;
            return;
        };
        let Some(entry) = entries.get(entry_index) else {
            return;
        };
        let height = entry.expanded_height(&self.json_open);
        let above = if self.selected <= self.row_offset {
            0
        } else {
            self.span_lines(entries, indices, self.row_offset, self.selected - 1)
        };
        let available = viewport.saturating_sub(above).max(1);
        if self.expand_cursor < self.expand_line_offset {
            self.expand_line_offset = self.expand_cursor;
        }
        let last = self.expand_line_offset + available - 1;
        if self.expand_cursor > last {
            self.expand_line_offset = self.expand_cursor + 1 - available;
        }
        let max_off = height.saturating_sub(available);
        if self.expand_line_offset > max_off {
            self.expand_line_offset = max_off;
        }
    }

    fn span_lines(
        &self,
        entries: &[ConsoleEntry],
        indices: &[usize],
        from: usize,
        through: usize,
    ) -> usize {
        let mut lines = 0;
        for (pos, &entry_index) in indices.iter().enumerate() {
            if pos < from {
                continue;
            }
            if pos > through {
                break;
            }
            lines += self.row_height(entries.get(entry_index), pos);
        }
        lines
    }

    fn row_height(&self, entry: Option<&ConsoleEntry>, filtered_index: usize) -> usize {
        if self.expanded && filtered_index == self.selected {
            entry.map_or(1, |item| item.expanded_height(&self.json_open))
        } else {
            1
        }
    }

    /// Render header + rows clipped to `width` × `height` (inner pane).
    #[must_use]
    pub fn lines(
        &self,
        entries: &[ConsoleEntry],
        styles: &Styles,
        width: usize,
        height: usize,
        focused: bool,
    ) -> Vec<Line<'static>> {
        if width == 0 || height == 0 {
            return Vec::new();
        }
        let mut state = self.clone();
        let indices = state.filtered_indices(entries);
        state.clamp_selection(indices.len());
        let body_height = height.saturating_sub(1).max(1);
        state.ensure_visible(entries, body_height);

        let mut out = Vec::with_capacity(height);
        out.push(header_line(width, styles));

        if indices.is_empty() {
            let empty = if state.query.is_empty() {
                "No application logs"
            } else {
                "No matching logs"
            };
            out.push(Line::from(Span::styled(
                fit_cell(empty, width),
                styles.muted,
            )));
            return constrain_lines(out, width, height);
        }

        let mut used = 0usize;
        for (pos, &entry_index) in indices.iter().enumerate().skip(state.row_offset) {
            if used >= body_height {
                break;
            }
            let Some(entry) = entries.get(entry_index) else {
                continue;
            };
            let is_selected = pos == state.selected;
            let expand = state.expanded && is_selected;
            let remaining = body_height - used;
            let mut row_lines = entry_lines(
                entry,
                width,
                styles,
                is_selected,
                focused,
                expand.then_some((state.expand_cursor, &state.json_open)),
            );
            if expand && state.expand_line_offset > 0 {
                if state.expand_line_offset < row_lines.len() {
                    row_lines = row_lines.split_off(state.expand_line_offset);
                } else {
                    row_lines.clear();
                }
            }
            for line in row_lines.into_iter().take(remaining) {
                out.push(line);
                used += 1;
            }
        }
        constrain_lines(out, width, height)
    }

    #[must_use]
    pub fn title(&self) -> String {
        let mut title = String::from(" Console ");
        if self.fullscreen {
            title.push_str("FULL ");
        }
        if !self.query.is_empty() || self.searching {
            title.push('/');
            title.push_str(&self.query);
            if self.searching {
                title.push('_');
            }
            title.push(' ');
        }
        title
    }
}

fn header_line(width: usize, styles: &Styles) -> Line<'static> {
    let time = fit_cell("TIME", TIME_COL);
    let level = fit_cell("LEVEL", LEVEL_COL);
    let msg_w = message_width(width);
    let message = fit_cell("MESSAGE", msg_w);
    Line::from(vec![
        Span::styled(time, styles.muted.add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(COL_GAP)),
        Span::styled(level, styles.muted.add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(COL_GAP)),
        Span::styled(message, styles.muted.add_modifier(Modifier::BOLD)),
    ])
}

fn message_width(width: usize) -> usize {
    width
        .saturating_sub(TIME_COL)
        .saturating_sub(LEVEL_COL)
        .saturating_sub(COL_GAP * 2)
}

fn entry_lines(
    entry: &ConsoleEntry,
    width: usize,
    styles: &Styles,
    selected: bool,
    focused: bool,
    expand: Option<(usize, &HashSet<String>)>,
) -> Vec<Line<'static>> {
    let expand_cursor = expand.map_or(0, |(cursor, _)| cursor);
    let summary_focused = focused && selected && expand_cursor == 0;
    let summary_style = if summary_focused {
        styles.focus
    } else if selected {
        styles.text.add_modifier(Modifier::BOLD)
    } else {
        styles.text
    };
    let time_style = if summary_focused {
        styles.focus
    } else {
        styles.muted
    };
    let level_style = if summary_focused {
        styles.focus
    } else {
        level_style(entry.level, styles)
    };

    let msg_w = message_width(width);
    let mut lines = vec![Line::from(vec![
        Span::styled(fit_cell(&entry.time, TIME_COL), time_style),
        Span::raw(" ".repeat(COL_GAP)),
        Span::styled(fit_cell(entry.level.as_str(), LEVEL_COL), level_style),
        Span::raw(" ".repeat(COL_GAP)),
        Span::styled(fit_cell(&entry.message, msg_w), summary_style),
    ])];

    if let Some((expand_cursor, json_open)) = expand {
        for (index, row) in entry.detail_rows(json_open).into_iter().enumerate() {
            let cursor_here = focused && selected && expand_cursor == index + 1;
            let style = if cursor_here {
                styles.focus
            } else {
                styles.muted
            };
            lines.push(detail_line(&row, width, style));
        }
    }
    lines
}

fn detail_line(row: &DetailRow, width: usize, style: Style) -> Line<'static> {
    match row {
        DetailRow::Field { key, value } => {
            let indent = "  ";
            let key_cell = fit_cell(key, DETAIL_KEY_COL);
            let value_w = width
                .saturating_sub(indent.len())
                .saturating_sub(DETAIL_KEY_COL)
                .saturating_sub(COL_GAP);
            Line::from(vec![
                Span::styled(indent.to_string(), style),
                Span::styled(key_cell, style),
                Span::raw(" ".repeat(COL_GAP)),
                Span::styled(fit_cell(value, value_w), style),
            ])
        }
        DetailRow::Json(json) => {
            let indent = "  ".repeat(json.depth + 1);
            let chevron = if json.expandable {
                if json.expanded { "▾ " } else { "▸ " }
            } else {
                "  "
            };
            let prefix = format!("{indent}{chevron}");
            let mut text = json.label.clone();
            if !json.value.is_empty() {
                text.push(' ');
                text.push_str(&json.value);
            }
            let value_w = width.saturating_sub(prefix.len());
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(fit_cell(&text, value_w), style),
            ])
        }
    }
}

fn level_style(level: ConsoleLevel, styles: &Styles) -> Style {
    match level {
        ConsoleLevel::Error => styles.error,
        ConsoleLevel::Warn => styles.alert,
        ConsoleLevel::Info => styles.signal,
        ConsoleLevel::Debug | ConsoleLevel::Trace => styles.muted,
    }
}

fn add_clamped(value: usize, delta: isize, max: usize) -> usize {
    let next = isize::try_from(value).unwrap_or(0).saturating_add(delta);
    let max = isize::try_from(max).unwrap_or(0);
    usize::try_from(next.clamp(0, max)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use mtui_core::{DefaultTheme, Theme};

    use super::*;
    use crate::layout::{line_plain, line_width};

    fn styles() -> Styles {
        Styles::from_palette(DefaultTheme::new().palette())
    }

    fn entry(message: &str, level: ConsoleLevel) -> ConsoleEntry {
        ConsoleEntry {
            time: "2026-08-22 03:25:01.000".into(),
            level,
            message: message.into(),
            fields: vec![
                ("target".into(), "mtui_app::app".into()),
                ("endpoint".into(), "/interface".into()),
            ],
        }
    }

    #[test]
    fn docked_height_is_a_quarter_with_room_for_the_body() {
        assert_eq!(console_pane_height(24, false, false), 0);
        assert_eq!(console_pane_height(24, true, false), 6);
        assert_eq!(console_pane_height(40, true, false), 10);
        assert_eq!(console_pane_height(24, true, true), 15);
    }

    #[test]
    fn short_terminals_keep_a_body_slot() {
        let height = console_pane_height(10, true, false);
        assert!(height > 0);
        assert!(height + 2 + crate::chrome::tab_strip_height(10) <= 10);
    }

    #[test]
    fn columns_align_across_rows() {
        let styles = styles();
        let entries = vec![
            entry("outbound request", ConsoleLevel::Info),
            entry("failed", ConsoleLevel::Error),
        ];
        let state = ConsoleState::default();
        let lines = state.lines(&entries, &styles, 72, 8, true);
        let header = line_plain(&lines[0]);
        let row = line_plain(&lines[1]);
        assert_eq!(header.find("LEVEL"), row.find("INFO"));
        assert_eq!(
            header.find("MESSAGE"),
            Some(TIME_COL + COL_GAP + LEVEL_COL + COL_GAP)
        );
        assert!(row.contains("outbound request"));
        assert_eq!(line_width(&lines[0]), 72);
        assert_eq!(line_width(&lines[1]), 72);
    }

    #[test]
    fn search_is_case_insensitive() {
        let entries = vec![
            entry("Outbound Request", ConsoleLevel::Info),
            entry("other", ConsoleLevel::Debug),
        ];
        let mut state = ConsoleState {
            query: "outbound".into(),
            ..ConsoleState::default()
        };
        assert_eq!(state.filtered_indices(&entries), vec![0]);
        state.query = "ERROR".into();
        let err = vec![entry("x", ConsoleLevel::Error)];
        assert_eq!(state.filtered_indices(&err), vec![0]);
    }

    #[test]
    fn expanding_uses_a_distinct_detail_color_and_closes_the_previous_row() {
        let styles = styles();
        let entries = vec![
            entry("one", ConsoleLevel::Info),
            entry("two", ConsoleLevel::Warn),
        ];
        let mut state = ConsoleState {
            expanded: true,
            selected: 0,
            ..ConsoleState::default()
        };
        let first = state.lines(&entries, &styles, 80, 10, true);
        assert!(line_plain(&first[2]).contains("target"));
        state.selected = 1;
        let second = state.lines(&entries, &styles, 80, 10, true);
        let plain = second.iter().map(line_plain).collect::<Vec<_>>().join("\n");
        assert!(plain.contains("two"));
        assert_eq!(plain.matches("target").count(), 1);
        assert_eq!(second[2].spans[0].style, styles.focus);
        assert_eq!(second[3].spans[0].style, styles.muted);
        state.expand_cursor = 1;
        let inner = state.lines(&entries, &styles, 80, 10, true);
        assert_eq!(inner[3].spans[0].style, styles.focus);
    }

    #[test]
    fn iterating_keeps_expansion_on_the_focused_row() {
        let mut state = ConsoleState {
            expanded: true,
            ..ConsoleState::default()
        };
        state.move_selection(1, 3);
        assert!(state.expanded);
        assert_eq!(state.selected, 1);
        state.toggle_expanded(3);
        assert!(!state.expanded);
        state.toggle_expanded(3);
        assert!(state.expanded);
    }

    #[test]
    fn page_and_home_end_move_the_cursor() {
        let mut state = ConsoleState::default();
        state.page_by(1, 4, 20);
        assert_eq!(state.selected, 4);
        state.select_last(20);
        assert_eq!(state.selected, 19);
        state.select_first();
        assert_eq!(state.selected, 0);
        state.page_by(-1, 4, 20);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn copy_text_includes_fields() {
        let text = entry("outbound request", ConsoleLevel::Info).copy_text();
        assert!(text.contains("2026-08-22 03:25:01.000"));
        assert!(text.contains("INFO"));
        assert!(text.contains("outbound request"));
        assert!(text.contains("endpoint: /interface"));
    }

    #[test]
    fn json_body_starts_collapsed_and_expands_into_keys() {
        let styles = styles();
        let mut body = entry("response /interface/list/add", ConsoleLevel::Error);
        body.fields.push((
            "body".into(),
            r#"{"error":400,"message":"Bad Request","detail":"no such item"}"#.into(),
        ));
        let entries = vec![body];
        let mut state = ConsoleState {
            expanded: true,
            ..ConsoleState::default()
        };
        let collapsed = state.lines(&entries, &styles, 88, 12, true);
        let collapsed_plain = collapsed
            .iter()
            .map(line_plain)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(collapsed_plain.contains("▸"));
        assert!(collapsed_plain.contains("body"));
        assert!(!collapsed_plain.contains("no such item"));

        state.expand_cursor = 3; // target, endpoint, then body
        state.enter_detail(&entries);
        let expanded = state.lines(&entries, &styles, 88, 16, true);
        let expanded_plain = expanded
            .iter()
            .map(line_plain)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(expanded_plain.contains("▾"));
        assert!(expanded_plain.contains("error"));
        assert!(expanded_plain.contains("no such item"));
    }

    #[test]
    fn copy_text_pretty_prints_json_bodies() {
        let mut body = entry("response /interface/list/add", ConsoleLevel::Error);
        body.fields.push((
            "body".into(),
            r#"{"error":400,"message":"Bad Request"}"#.into(),
        ));
        let text = body.copy_text();
        assert!(text.contains("{\n"));
        assert!(text.contains("\"error\": 400"));
    }
}
