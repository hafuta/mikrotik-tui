//! Filterable / sortable table state.

use std::collections::{HashMap, HashSet};

use mtui_core::ColumnSpec;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::layout::{clip_line, constrain_lines};
use crate::styles::Styles;

pub type Row = HashMap<String, String>;

const DEFAULT_VIEWPORT_WIDTH: usize = 80;
const DEFAULT_VIEWPORT_HEIGHT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct TableState {
    pub columns: Vec<ColumnSpec>,
    pub rows: Vec<Row>,
    pub filter: String,
    pub sort_col: Option<usize>,
    pub sort_dir: SortDir,
    pub selected: usize,
    pub row_offset: usize,
    pub col_offset: usize,
    viewport_width: usize,
    viewport_height: usize,
    filtered: Vec<usize>,
    checked: HashSet<String>,
}

impl TableState {
    #[must_use]
    pub fn new(columns: &[ColumnSpec]) -> Self {
        Self {
            columns: columns.to_vec(),
            rows: Vec::new(),
            filter: String::new(),
            sort_col: None,
            sort_dir: SortDir::Asc,
            selected: 0,
            row_offset: 0,
            col_offset: 0,
            viewport_width: DEFAULT_VIEWPORT_WIDTH,
            viewport_height: DEFAULT_VIEWPORT_HEIGHT,
            filtered: Vec::new(),
            checked: HashSet::new(),
        }
    }

    pub fn set_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        self.recompute();
        self.prune_checked();
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.recompute();
    }

    pub fn cycle_sort(&mut self) {
        if self.columns.is_empty() {
            return;
        }
        match self.sort_col {
            None => {
                self.sort_col = Some(0);
                self.sort_dir = SortDir::Asc;
            }
            Some(_) if self.sort_dir == SortDir::Asc => self.sort_dir = SortDir::Desc,
            Some(i) if i + 1 < self.columns.len() => {
                self.sort_col = Some(i + 1);
                self.sort_dir = SortDir::Asc;
            }
            _ => {
                self.sort_col = None;
            }
        }
        self.recompute();
    }

    /// Move the selection by `delta` rows, clamped to the filtered list.
    pub fn move_selection(&mut self, delta: isize) {
        let Some(max) = self.filtered.len().checked_sub(1) else {
            return;
        };
        self.selected = add_clamped(self.selected, delta, max);
        self.reconcile_offsets();
    }

    pub fn page_by(&mut self, direction: isize) {
        let page = isize::try_from(self.page_size()).unwrap_or(1);
        self.move_selection(direction.saturating_mul(page));
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
        self.reconcile_offsets();
    }

    pub fn select_last(&mut self) {
        self.selected = self.filtered.len().saturating_sub(1);
        self.reconcile_offsets();
    }

    /// Number of filtered rows (the scrollable list, not the on-screen window).
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.filtered.len()
    }

    /// Rows that fit in the current viewport, excluding the header.
    #[must_use]
    pub fn page_size(&self) -> usize {
        self.body_height(self.viewport_height).max(1)
    }

    #[must_use]
    pub fn selected_row(&self) -> Option<&Row> {
        self.filtered
            .get(self.selected)
            .and_then(|i| self.rows.get(*i))
    }

    /// Toggle the current row's bulk-select mark (keyed by `.id`).
    pub fn toggle_checked(&mut self) {
        let Some(id) = self.selected_row().and_then(|row| row.get(".id")).cloned() else {
            return;
        };
        if !self.checked.remove(&id) {
            self.checked.insert(id);
        }
    }

    /// Mark every filtered row that has an `.id`.
    pub fn check_all_filtered(&mut self) {
        let ids: Vec<String> = self
            .visible_rows()
            .into_iter()
            .filter_map(|row| row.get(".id").filter(|id| !id.is_empty()).cloned())
            .collect();
        self.checked.extend(ids);
    }

    pub fn clear_checked(&mut self) {
        self.checked.clear();
    }

    #[must_use]
    pub fn checked_count(&self) -> usize {
        self.checked.len()
    }

    #[must_use]
    pub fn is_row_checked(&self, row: &Row) -> bool {
        row.get(".id")
            .is_some_and(|id| !id.is_empty() && self.checked.contains(id))
    }

    /// Checked `.id` values in table order.
    #[must_use]
    pub fn checked_ids(&self) -> Vec<String> {
        self.rows
            .iter()
            .filter_map(|row| {
                let id = row.get(".id")?;
                self.checked.contains(id).then(|| id.clone())
            })
            .collect()
    }

    fn prune_checked(&mut self) {
        self.checked.retain(|id| {
            self.rows
                .iter()
                .any(|row| row.get(".id").map(String::as_str) == Some(id.as_str()))
        });
    }

    /// Restore selection to the filtered row whose `.id` matches `id`.
    pub fn select_id(&mut self, id: &str) -> bool {
        let Some(pos) = self.filtered.iter().position(|&i| {
            self.rows
                .get(i)
                .and_then(|row| row.get(".id"))
                .map(String::as_str)
                == Some(id)
        }) else {
            return false;
        };
        self.selected = pos;
        self.reconcile_offsets();
        true
    }

    #[must_use]
    pub fn visible_rows(&self) -> Vec<&Row> {
        self.filtered
            .iter()
            .filter_map(|i| self.rows.get(*i))
            .collect()
    }

    /// Scroll the horizontal column window by `delta` columns, clamped so the
    /// remaining columns still fill the current viewport width when possible.
    pub fn scroll_columns(&mut self, delta: isize) {
        let max = self.max_col_offset(self.content_width());
        self.col_offset = add_clamped(self.col_offset, delta, max);
    }

    /// Whether `scroll_columns(delta)` would change the column window.
    #[must_use]
    pub fn can_scroll_columns(&self, delta: isize) -> bool {
        let max = self.max_col_offset(self.content_width());
        add_clamped(self.col_offset, delta, max) != self.col_offset
    }

    pub fn scroll_columns_home(&mut self) {
        self.col_offset = 0;
    }

    pub fn scroll_columns_end(&mut self) {
        self.col_offset = self.max_col_offset(self.content_width());
    }

    /// Remember the pane size and keep selection plus offsets inside it.
    pub fn sync_viewport(&mut self, visible_width: usize, visible_height: usize) {
        self.viewport_width = visible_width.max(1);
        self.viewport_height = visible_height.max(1);
        self.reconcile_offsets();
    }

    /// Recompute `row_offset` so the selected row stays within a
    /// `visible_height`-row window (classic scrolling-list behavior).
    pub fn ensure_selection_visible(&mut self, visible_height: usize) {
        if visible_height == 0 || self.filtered.is_empty() {
            self.row_offset = 0;
            return;
        }
        if self.selected < self.row_offset {
            self.row_offset = self.selected;
        } else if self.selected >= self.row_offset + visible_height {
            self.row_offset = self.selected + 1 - visible_height;
        }
    }

    /// Indices into `rows` visible within a `height`-row window, honoring
    /// `row_offset` and clamping when the pane grows.
    #[must_use]
    pub fn visible_window(&self, height: usize) -> &[usize] {
        let start = self.clamped_row_offset(height);
        let end = start.saturating_add(height).min(self.filtered.len());
        &self.filtered[start..end]
    }

    /// Build the styled header line, honoring column offset and pane width.
    #[must_use]
    pub fn header_line(&self, styles: &Styles, width: usize) -> Line<'static> {
        packed_row_line(
            &self.columns,
            self.effective_col_offset(width),
            width,
            |idx, col| {
                let mut title = col.title.to_string();
                if self.sort_col == Some(idx) {
                    title.push_str(match self.sort_dir {
                        SortDir::Asc => " ↑",
                        SortDir::Desc => " ↓",
                    });
                }
                (title, styles.muted)
            },
        )
    }

    /// Build a styled line for `row`, honoring column offset and pane width.
    /// The selected row is a fully painted selection rectangle (no box borders).
    #[must_use]
    pub fn row_line(
        &self,
        row: &Row,
        selected: bool,
        styles: &Styles,
        width: usize,
    ) -> Line<'static> {
        let line = packed_row_line(
            &self.columns,
            self.effective_col_offset(width),
            width,
            |_, col| {
                let text = row.get(col.key).cloned().unwrap_or_default();
                let style = cell_style(col.key, &text, selected, styles);
                (text, style)
            },
        );
        if selected {
            crate::layout::fit_line(crate::paint::line_on_bg(line, styles.selection), width)
        } else if self.is_row_checked(row) {
            Line::from(
                line.spans
                    .into_iter()
                    .map(|span| Span::styled(span.content, styles.focus))
                    .collect::<Vec<_>>(),
            )
        } else {
            line
        }
    }

    /// Render header + visible rows into a `width` × `height` canvas.
    #[must_use]
    pub fn lines(
        &self,
        styles: &Styles,
        width: usize,
        height: usize,
        empty_hint: &str,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut body_h = height;
        if !self.columns.is_empty() && height > 0 {
            lines.push(self.header_line(styles, width));
            body_h = height.saturating_sub(1);
        }
        let start = self.clamped_row_offset(body_h);
        let window = self.visible_window(body_h);
        if window.is_empty() {
            if body_h > 0 {
                lines.push(Line::from(Span::styled(
                    empty_hint.to_string(),
                    styles.muted,
                )));
            }
        } else {
            for (i, &row_idx) in window.iter().enumerate() {
                let Some(row) = self.rows.get(row_idx) else {
                    continue;
                };
                let selected = start + i == self.selected;
                lines.push(self.row_line(row, selected, styles, width));
            }
        }
        constrain_lines(lines, width, height)
    }

    #[must_use]
    pub fn cell(&self, row: &Row, col: &ColumnSpec, width: usize) -> String {
        let value = row.get(col.key).map_or("", String::as_str);
        clip_line(value, width)
    }

    fn body_height(&self, inner_height: usize) -> usize {
        if self.columns.is_empty() {
            inner_height
        } else {
            inner_height.saturating_sub(1)
        }
    }

    fn clamped_row_offset(&self, visible_height: usize) -> usize {
        let max_start = self.filtered.len().saturating_sub(visible_height);
        self.row_offset.min(max_start)
    }

    fn effective_col_offset(&self, width: usize) -> usize {
        self.col_offset.min(self.max_col_offset(width))
    }

    fn max_col_offset(&self, visible_width: usize) -> usize {
        if self.columns.is_empty() {
            return 0;
        }
        for i in 0..self.columns.len() {
            if self.content_width_from(i) <= visible_width {
                return i;
            }
        }
        self.columns.len() - 1
    }

    fn content_width_from(&self, start: usize) -> usize {
        let cols = self.columns.get(start..).unwrap_or(&[]);
        if cols.is_empty() {
            return 0;
        }
        let cells: usize = cols.iter().map(|col| usize::from(col.width).max(1)).sum();
        cells.saturating_add(cols.len().saturating_sub(1).saturating_mul(2))
    }

    fn reconcile_offsets(&mut self) {
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
        let body_h = self.body_height(self.viewport_height);
        self.ensure_selection_visible(body_h);
        self.row_offset = self.clamped_row_offset(body_h);
        self.col_offset = self
            .col_offset
            .min(self.max_col_offset(self.content_width()));
    }

    fn content_width(&self) -> usize {
        self.viewport_width
    }

    fn recompute(&mut self) {
        let filter = self.filter.to_ascii_lowercase();
        let mut indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| {
                if filter.is_empty() {
                    return true;
                }
                row.values()
                    .any(|v| v.to_ascii_lowercase().contains(&filter))
            })
            .map(|(i, _)| i)
            .collect();

        if let Some(col_idx) = self.sort_col
            && let Some(col) = self.columns.get(col_idx)
        {
            let key = col.key;
            let dir = self.sort_dir;
            indices.sort_by(|&a, &b| {
                let av = self.rows[a].get(key).map_or("", String::as_str);
                let bv = self.rows[b].get(key).map_or("", String::as_str);
                match dir {
                    SortDir::Asc => av.cmp(bv),
                    SortDir::Desc => bv.cmp(av),
                }
            });
        }

        self.filtered = indices;
        self.reconcile_offsets();
    }
}

fn add_clamped(base: usize, delta: isize, max: usize) -> usize {
    let base_i = isize::try_from(base).unwrap_or(isize::MAX);
    let max_i = isize::try_from(max).unwrap_or(isize::MAX);
    let next = base_i.saturating_add(delta).clamp(0, max_i);
    usize::try_from(next).unwrap_or(0)
}

fn packed_row_line<F>(
    columns: &[ColumnSpec],
    col_offset: usize,
    max_width: usize,
    mut cell_text: F,
) -> Line<'static>
where
    F: FnMut(usize, &ColumnSpec) -> (String, Style),
{
    if max_width == 0 {
        return Line::default();
    }
    let mut spans = Vec::new();
    let mut used = 0;
    for (idx, col) in columns.iter().enumerate().skip(col_offset) {
        let sep_w = 2;
        let sep = if spans.is_empty() { 0 } else { sep_w };
        if used + sep >= max_width {
            break;
        }
        let available = max_width - used - sep;
        let col_w = usize::from(col.width).max(1);
        let cell_w = col_w.min(available);
        let (raw, style) = cell_text(idx, col);
        let text = clip_line(&raw, cell_w);
        let padded = format!("{text:<cell_w$}");
        if sep > 0 {
            spans.push(Span::styled(" ".repeat(sep), style));
            used += sep;
        }
        spans.push(Span::styled(padded, style));
        used += cell_w;
        if cell_w < col_w {
            break;
        }
    }
    Line::from(spans)
}

fn cell_style(key: &str, value: &str, selected: bool, styles: &Styles) -> Style {
    let lower = value.trim().to_ascii_lowercase();
    let positive = matches!(
        lower.as_str(),
        "true" | "yes" | "1" | "running" | "up" | "enabled"
    );
    let negative = matches!(
        lower.as_str(),
        "false" | "no" | "0" | "down" | "disabled" | "offline"
    );
    if matches!(key, "running" | "status") || key.ends_with("-status") {
        if positive {
            return styles.signal;
        }
        if negative {
            return styles.muted;
        }
    }
    if matches!(key, "disabled" | "invalid") {
        if positive {
            return styles.alert;
        }
        return styles.muted;
    }
    if selected && is_live_rate_key(key) {
        return styles.alert;
    }
    if is_live_rate_key(key) {
        return styles.data;
    }
    styles.text
}

fn is_live_rate_key(key: &str) -> bool {
    matches!(key, "rx" | "tx" | "rx-byte" | "tx-byte" | "rate")
        || (key.contains("rx") && (key.contains("byte") || key.contains("rate")))
        || (key.contains("tx") && (key.contains("byte") || key.contains("rate")))
}

#[cfg(test)]
mod scroll_tests {
    use super::*;
    use crate::layout::lines_plain;
    use mtui_core::{DefaultTheme, Theme};

    fn columns() -> Vec<ColumnSpec> {
        vec![
            ColumnSpec {
                key: "name",
                title: "Name",
                width: 10,
            },
            ColumnSpec {
                key: "type",
                title: "Type",
                width: 8,
            },
            ColumnSpec {
                key: "extra",
                title: "Extra",
                width: 6,
            },
        ]
    }

    fn rows(count: usize) -> Vec<Row> {
        (0..count)
            .map(|i| {
                let mut row = HashMap::new();
                row.insert("name".into(), format!("row-{i:02}"));
                row.insert("type".into(), "ether".into());
                row.insert("extra".into(), format!("{i}"));
                row
            })
            .collect()
    }

    fn styles() -> Styles {
        let theme = DefaultTheme::new();
        crate::styles::Styles::from_palette(theme.palette())
    }

    #[test]
    fn scroll_columns_clamps_to_bounds() {
        let mut state = TableState::new(&columns());
        state.sync_viewport(18, 8);
        state.scroll_columns(-3);
        assert_eq!(state.col_offset, 0);
        assert!(!state.can_scroll_columns(-1));
        assert!(state.can_scroll_columns(1));
        state.scroll_columns(10);
        assert_eq!(state.col_offset, 1);
        assert!(!state.can_scroll_columns(1));
        assert!(state.can_scroll_columns(-1));
    }

    #[test]
    fn ensure_selection_visible_scrolls_row_window() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(4));
        state.selected = 3;
        state.ensure_selection_visible(2);
        assert_eq!(state.row_offset, 2);
        assert_eq!(state.visible_window(2), &[2, 3]);
    }

    #[test]
    fn move_selection_clamps_instead_of_wrapping() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(3));
        state.move_selection(-4);
        assert_eq!(state.selected, 0);
        state.move_selection(10);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn growing_the_pane_collapses_offsets() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(8));
        state.sync_viewport(18, 3);
        state.select_last();
        state.scroll_columns(1);
        assert!(state.row_offset > 0, "row offset {}", state.row_offset);
        assert_eq!(state.col_offset, 1);

        state.sync_viewport(100, 20);
        assert_eq!(state.row_offset, 0);
        assert_eq!(state.col_offset, 0);
        assert_eq!(state.selected, 7);
    }

    #[test]
    fn shrinking_the_pane_keeps_the_selection_in_view() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(10));
        state.sync_viewport(80, 20);
        state.selected = 9;
        state.sync_viewport(80, 4);
        assert_eq!(state.selected, 9);
        assert_eq!(state.row_offset, 7);
        assert_eq!(state.visible_window(3), &[7, 8, 9]);
    }

    #[test]
    fn lines_only_include_the_visible_window() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(6));
        state.sync_viewport(24, 4);
        state.select_last();
        let view = lines_plain(&state.lines(&styles(), 24, 4, "empty"));
        assert!(view.contains("row-05"), "missing last row: {view}");
        assert!(
            !view.contains("row-00"),
            "scrolled table still showed the first row: {view}"
        );
        assert!(view.contains("Name"), "missing header: {view}");
    }

    #[test]
    fn narrow_width_packs_only_columns_that_fit() {
        let mut state = TableState::new(&columns());
        state.set_rows(rows(1));
        state.sync_viewport(10, 4);
        let header = lines_plain(&[state.header_line(&styles(), 10)]);
        assert!(header.contains("Name"), "missing first column: {header}");
        assert!(
            !header.contains("Extra"),
            "narrow header still showed Extra: {header}"
        );
        state.scroll_columns(1);
        let header = lines_plain(&[state.header_line(&styles(), 10)]);
        assert!(header.contains("Type"), "did not pan to Type: {header}");
        assert!(!header.contains("Name"), "Name stayed after pan: {header}");
    }

    #[test]
    fn toggle_checked_tracks_row_ids() {
        let mut state = TableState::new(&columns());
        let mut first = rows(1).remove(0);
        first.insert(".id".into(), "*1".into());
        let mut second = HashMap::new();
        second.insert("name".into(), "row-01".into());
        second.insert(".id".into(), "*2".into());
        state.set_rows(vec![first, second]);
        state.toggle_checked();
        assert_eq!(state.checked_ids(), vec!["*1".to_string()]);
        state.move_selection(1);
        state.toggle_checked();
        assert_eq!(state.checked_count(), 2);
        state.check_all_filtered();
        assert_eq!(state.checked_count(), 2);
        state.clear_checked();
        assert_eq!(state.checked_count(), 0);
    }

    #[test]
    fn empty_table_toggle_checked_is_a_noop() {
        let mut state = TableState::new(&columns());
        state.toggle_checked();
        assert_eq!(state.checked_count(), 0);
        state.check_all_filtered();
        assert_eq!(state.checked_count(), 0);
    }

    #[test]
    fn check_all_filtered_skips_hidden_rows_and_prunes_stale_ids() {
        let mut state = TableState::new(&columns());
        let rows: Vec<Row> = (0..3)
            .map(|i| {
                let mut row = HashMap::new();
                row.insert("name".into(), format!("row-{i:02}"));
                row.insert(".id".into(), format!("*{i}"));
                row
            })
            .collect();
        state.set_rows(rows);
        state.set_filter("row-01".into());
        state.check_all_filtered();
        assert_eq!(state.checked_ids(), vec!["*1".to_string()]);
        state.set_rows({
            let mut kept = HashMap::new();
            kept.insert("name".into(), "row-02".into());
            kept.insert(".id".into(), "*2".into());
            vec![kept]
        });
        assert_eq!(state.checked_count(), 0);
    }

    #[test]
    fn checked_unselected_row_uses_focus_style_at_fixed_width() {
        let mut state = TableState::new(&columns());
        let mut first = HashMap::new();
        first.insert("name".into(), "ether1".into());
        first.insert(".id".into(), "*1".into());
        let mut second = HashMap::new();
        second.insert("name".into(), "ether2".into());
        second.insert(".id".into(), "*2".into());
        state.set_rows(vec![first, second]);
        state.toggle_checked();
        state.move_selection(1);
        let styles = styles();
        let line = state.row_line(&state.rows[0], false, &styles, 24);
        let plain = crate::layout::line_plain(&line);
        assert!(plain.contains("ether1"), "{plain}");
        assert_eq!(crate::layout::line_width(&line), 24);
        assert!(
            line.spans
                .iter()
                .any(|span| span.content.contains("ether1") && span.style.fg == styles.focus.fg),
            "checked row should use focus fg: {line:?}"
        );
        assert!(
            line.spans.iter().all(|span| span.style.bg.is_none()),
            "checked mark must not paint an unbounded background: {line:?}"
        );
    }

    #[test]
    fn header_shows_sort_indicator() {
        let mut state = TableState::new(&columns());
        state.cycle_sort();
        let header = lines_plain(&[state.header_line(&styles(), 40)]);
        assert!(header.contains("↑"), "missing sort marker: {header}");
    }

    #[test]
    fn selected_row_marks_cursor_and_colors_status() {
        let mut state = TableState::new(&[
            ColumnSpec {
                key: "name",
                title: "Name",
                width: 8,
            },
            ColumnSpec {
                key: "running",
                title: "Run",
                width: 8,
            },
            ColumnSpec {
                key: "rx-byte",
                title: "RX",
                width: 10,
            },
        ]);
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        row.insert("running".into(), "true".into());
        row.insert("rx-byte".into(), "84.2 Mb/s".into());
        state.set_rows(vec![row]);
        let styles = styles();
        let line = state.row_line(state.selected_row().expect("row"), true, &styles, 40);
        let plain = crate::layout::line_plain(&line);
        assert!(plain.contains("ether1"), "{plain}");
        assert!(!plain.contains('›'));
        assert_eq!(crate::layout::line_width(&line), 40);
        assert!(
            line.spans
                .iter()
                .any(|span| { span.content.contains("true") && span.style.fg == styles.signal.fg }),
            "running cell should be signal: {line:?}"
        );
        assert!(
            line.spans
                .iter()
                .any(|span| { span.content.contains("84.2") && span.style.fg == styles.alert.fg }),
            "selected rx should be alert: {line:?}"
        );
        assert!(
            line.spans
                .iter()
                .all(|span| span.style.bg == Some(styles.selection)),
            "selected row must be a bounded fill: {line:?}"
        );
    }
}
