//! In-terminal directory browser. Listing happens outside this crate.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::layout::clip_line;
use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::styles::Styles;

/// One directory or file row the picker can show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePickerEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

/// Keyboard-driven local file browser state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePickerState {
    pub dir: String,
    pub entries: Vec<FilePickerEntry>,
    pub selected: usize,
    pub offset: usize,
    pub loading: bool,
    pub refreshing: bool,
    pub error: Option<String>,
    pub generation: u64,
}

impl FilePickerState {
    #[must_use]
    pub fn loading(dir: impl Into<String>, generation: u64) -> Self {
        Self {
            dir: dir.into(),
            entries: Vec::new(),
            selected: 0,
            offset: 0,
            loading: true,
            refreshing: false,
            error: None,
            generation,
        }
    }

    pub fn begin_list(&mut self, dir: impl Into<String>, generation: u64) {
        self.dir = dir.into();
        self.generation = generation;
        self.error = None;
        if self.entries.is_empty() {
            self.loading = true;
            self.refreshing = false;
        } else {
            self.loading = false;
            self.refreshing = true;
        }
    }

    pub fn apply_listing(
        &mut self,
        generation: u64,
        dir: String,
        entries: Vec<FilePickerEntry>,
        error: Option<String>,
    ) -> bool {
        if generation != self.generation {
            return false;
        }
        self.dir = dir;
        self.entries = entries;
        self.error = error;
        self.loading = false;
        self.refreshing = false;
        self.selected = 0;
        self.offset = 0;
        true
    }

    #[must_use]
    pub fn selected_entry(&self) -> Option<&FilePickerEntry> {
        self.entries.get(self.selected)
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.entries.len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let cur = isize::try_from(self.selected).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        self.selected = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
        self.ensure_visible(8);
    }

    pub fn jump_home(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    pub fn jump_end(&mut self) {
        self.selected = self.entries.len().saturating_sub(1);
        self.ensure_visible(8);
    }

    pub fn ensure_visible(&mut self, visible: usize) {
        let visible = visible.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset.saturating_add(visible) {
            self.offset = self.selected.saturating_add(1).saturating_sub(visible);
        }
    }
}

/// Centered directory list. Dims the canvas; never paints a shared background.
pub fn render_file_picker(
    frame: &mut Frame<'_>,
    area: Rect,
    picker: &FilePickerState,
    styles: &Styles,
) {
    dim_canvas(frame, area, styles);
    let width = area.width.saturating_sub(6).clamp(28, 72);
    let height = area.height.saturating_sub(4).clamp(8, 22);
    let rect = compact_modal_rect(area, width, height);
    frame.render_widget(Clear, rect);

    let title = if picker.refreshing {
        " Browse CA file · listing… "
    } else {
        " Browse CA file "
    };
    let block = Block::default()
        .title(Span::styled(title, styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    let dir_label = if picker.dir.is_empty() {
        "Drives"
    } else {
        picker.dir.as_str()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            clip_line(dir_label, usize::from(chunks[0].width.max(1))),
            styles.muted,
        ))),
        chunks[0],
    );

    let list_width = usize::from(chunks[1].width.max(1));
    let list_height = usize::from(chunks[1].height.max(1));
    frame.render_widget(
        Paragraph::new(file_picker_lines(picker, list_width, list_height, styles)),
        chunks[1],
    );
    if chunks[2].height > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "↑↓ move   enter open   ← up   esc",
                styles.muted,
            ))),
            chunks[2],
        );
    }
}

fn file_picker_lines(
    picker: &FilePickerState,
    width: usize,
    height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if height == 0 {
        return lines;
    }
    if picker.loading && picker.entries.is_empty() {
        lines.push(Line::from(Span::styled("loading…", styles.muted)));
        return lines;
    }
    if let Some(err) = &picker.error
        && picker.entries.is_empty()
    {
        lines.push(Line::from(Span::styled(
            clip_line(err, width),
            styles.alert,
        )));
        return lines;
    }
    if picker.entries.is_empty() {
        lines.push(Line::from(Span::styled("empty folder", styles.muted)));
        return lines;
    }
    let start = picker.selected.saturating_sub(height.saturating_sub(1));
    for (idx, entry) in picker.entries.iter().enumerate().skip(start) {
        if lines.len() >= height {
            break;
        }
        let caret = if idx == picker.selected { ">" } else { " " };
        let kind = if entry.is_dir { "/" } else { "" };
        let mark = if !entry.is_dir && looks_like_cert(&entry.name) {
            "  cert"
        } else {
            ""
        };
        let body = format!("{caret} {name}{kind}{mark}", name = entry.name);
        let style = if idx == picker.selected {
            styles.focus
        } else if entry.is_dir {
            styles.text
        } else {
            styles.muted
        };
        lines.push(Line::from(Span::styled(clip_line(&body, width), style)));
    }
    lines
}

fn looks_like_cert(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "pem" | "crt" | "cer" | "der" | "ca"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn sample() -> FilePickerState {
        let mut picker = FilePickerState::loading("/certs", 1);
        assert!(picker.apply_listing(
            1,
            "/certs".into(),
            vec![
                FilePickerEntry {
                    name: "ca".into(),
                    path: "/certs/ca".into(),
                    is_dir: true,
                },
                FilePickerEntry {
                    name: "router.pem".into(),
                    path: "/certs/router.pem".into(),
                    is_dir: false,
                },
            ],
            None,
        ));
        picker
    }

    #[test]
    fn move_selection_clamps() {
        let mut picker = sample();
        picker.move_selection(-3);
        assert_eq!(picker.selected, 0);
        picker.move_selection(8);
        assert_eq!(picker.selected, 1);
        picker.jump_home();
        assert_eq!(picker.selected, 0);
        picker.jump_end();
        assert_eq!(picker.selected, 1);
    }

    #[test]
    fn stale_listing_is_ignored() {
        let mut picker = sample();
        assert!(!picker.apply_listing(99, "/other".into(), Vec::new(), None));
        assert_eq!(picker.dir, "/certs");
        assert_eq!(picker.entries.len(), 2);
    }

    #[test]
    fn begin_list_keeps_rows_while_refreshing() {
        let mut picker = sample();
        picker.begin_list("/certs/ca", 2);
        assert!(!picker.loading);
        assert!(picker.refreshing);
        assert_eq!(picker.entries.len(), 2);
    }

    #[test]
    fn render_keeps_header_and_hint_on_a_small_canvas() {
        let backend = TestBackend::new(60, 18);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let picker = sample();
        terminal
            .draw(|frame| {
                render_file_picker(frame, frame.area(), &picker, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut found_title = false;
        let mut found_file = false;
        let mut found_hint = false;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains("Browse CA file") {
                found_title = true;
            }
            if row.contains("router.pem") {
                found_file = true;
            }
            if row.contains("enter open") {
                found_hint = true;
            }
        }
        assert!(found_title);
        assert!(found_file);
        assert!(found_hint);
    }
}
