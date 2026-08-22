//! Application log console: aligned columns, vim search, expand, copy text.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

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
    let chrome = 2; // header and footer bands
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
            out.push_str(value);
        }
        out
    }

    fn expanded_height(&self) -> usize {
        1 + self.fields.len().max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConsoleState {
    pub visible: bool,
    pub fullscreen: bool,
    pub selected: usize,
    pub row_offset: usize,
    /// When true, the focused row shows extra fields. Moving the cursor keeps
    /// this flag so each iterated row is expanded.
    pub expanded: bool,
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
            return;
        }
        if self.selected >= filtered_len {
            self.selected = filtered_len - 1;
        }
    }

    pub fn move_selection(&mut self, delta: isize, filtered_len: usize) {
        let Some(max) = filtered_len.checked_sub(1) else {
            self.selected = 0;
            return;
        };
        self.selected = add_clamped(self.selected, delta, max);
    }

    pub fn page_by(&mut self, direction: isize, page: usize, filtered_len: usize) {
        let step = isize::try_from(page.max(1)).unwrap_or(1);
        self.move_selection(direction.saturating_mul(step), filtered_len);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self, filtered_len: usize) {
        self.selected = filtered_len.saturating_sub(1);
    }

    pub fn toggle_expanded(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.expanded = false;
            return;
        }
        self.expanded = !self.expanded;
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
            return;
        }
        if self.selected < self.row_offset {
            self.row_offset = self.selected;
            return;
        }
        loop {
            let used = self.span_lines(entries, &indices, self.row_offset, self.selected);
            if used <= viewport || self.row_offset >= self.selected {
                break;
            }
            self.row_offset += 1;
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
            entry.map_or(1, ConsoleEntry::expanded_height)
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
            let row_lines = entry_lines(entry, width, styles, is_selected, expand, focused);
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
    expanded: bool,
    focused: bool,
) -> Vec<Line<'static>> {
    let summary_style = if focused && selected {
        styles.focus
    } else if selected {
        styles.text.add_modifier(Modifier::BOLD)
    } else {
        styles.text
    };
    let time_style = if focused && selected {
        styles.focus
    } else {
        styles.muted
    };
    let level_style = if focused && selected {
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

    if expanded {
        let detail_style = if focused && selected {
            styles.alert
        } else {
            styles.muted
        };
        let details = if entry.fields.is_empty() {
            vec![("target".into(), String::from("(none)"))]
        } else {
            entry.fields.clone()
        };
        for (key, value) in details {
            let indent = "  ";
            let key_cell = fit_cell(&key, DETAIL_KEY_COL);
            let value_w = width
                .saturating_sub(indent.len())
                .saturating_sub(DETAIL_KEY_COL)
                .saturating_sub(COL_GAP);
            lines.push(Line::from(vec![
                Span::styled(indent.to_string(), detail_style),
                Span::styled(key_cell, detail_style),
                Span::raw(" ".repeat(COL_GAP)),
                Span::styled(fit_cell(&value, value_w), detail_style),
            ]));
        }
    }
    lines
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
                ("endpoint".into(), "/rest/interface".into()),
            ],
        }
    }

    #[test]
    fn docked_height_is_a_quarter_with_room_for_the_body() {
        assert_eq!(console_pane_height(24, false, false), 0);
        assert_eq!(console_pane_height(24, true, false), 6);
        assert_eq!(console_pane_height(40, true, false), 10);
        assert_eq!(console_pane_height(24, true, true), 22);
    }

    #[test]
    fn short_terminals_keep_a_body_slot() {
        let height = console_pane_height(10, true, false);
        assert!(height > 0);
        assert!(height + 2 + 3 <= 10);
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
        assert_eq!(second[3].spans[1].style, styles.alert);
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
        assert!(text.contains("endpoint: /rest/interface"));
    }
}
