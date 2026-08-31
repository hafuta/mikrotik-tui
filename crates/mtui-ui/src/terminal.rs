//! Docked `RouterOS` SSH terminal pane (VT cell grid, no networking).

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::styles::Styles;

/// SSH link shown in the dock title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLink {
    Idle,
    Connecting,
    Live,
    Failed,
}

/// One screen cell after VT parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: Color,
    pub bold: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bold: false,
        }
    }
}

/// Docked PTY view. Secrets never belong here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalState {
    pub visible: bool,
    pub fullscreen: bool,
    pub host: String,
    pub port: u16,
    pub link: TerminalLink,
    pub error: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<TerminalCell>,
    pub cursor_col: u16,
    pub cursor_row: u16,
    /// Rows of VT scrollback currently in view (`0` follows the live screen).
    pub scroll_offset: usize,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            visible: false,
            fullscreen: false,
            host: String::new(),
            port: 22,
            link: TerminalLink::Idle,
            error: None,
            cols: 80,
            rows: 24,
            cells: Vec::new(),
            cursor_col: 0,
            cursor_row: 0,
            scroll_offset: 0,
        }
    }
}

impl TerminalState {
    #[must_use]
    pub fn title(&self) -> String {
        let mut title = String::from(" Terminal ");
        if !self.host.is_empty() {
            title.push_str("· ssh ");
            title.push_str(&self.host);
            title.push(':');
            title.push_str(&self.port.to_string());
            title.push(' ');
        }
        match self.link {
            TerminalLink::Connecting => title.push_str("connecting "),
            TerminalLink::Live => title.push_str("LIVE "),
            TerminalLink::Failed => title.push_str("down "),
            TerminalLink::Idle => {}
        }
        if self.fullscreen {
            title.push_str("FULL ");
        }
        if self.scroll_offset > 0 {
            title.push_str("SCROLL ");
        }
        title
    }

    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    pub fn resize_grid(&mut self, cols: u16, rows: u16) {
        let cols = cols.max(1);
        let rows = rows.max(1);
        self.cols = cols;
        self.rows = rows;
        self.cells.resize(
            usize::from(cols).saturating_mul(usize::from(rows)),
            TerminalCell::default(),
        );
    }
}

/// Paint the docked terminal. `area` is already the dock rectangle.
pub fn render_terminal(
    frame: &mut Frame<'_>,
    area: Rect,
    term: &TerminalState,
    focused: bool,
    styles: &Styles,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let border = if focused { styles.focus } else { styles.border };
    let block = Block::default()
        .title(term.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line<'static>> = Vec::with_capacity(usize::from(inner.height));
    let rows = usize::from(inner.height).min(usize::from(term.rows));
    let cols = usize::from(inner.width).min(usize::from(term.cols));
    for row in 0..rows {
        let mut spans = Vec::with_capacity(cols);
        for col in 0..cols {
            let idx = row
                .saturating_mul(usize::from(term.cols))
                .saturating_add(col);
            let cell = term.cells.get(idx).copied().unwrap_or_default();
            let mut style = Style::default().fg(if cell.fg == Color::Reset {
                styles.data.fg.unwrap_or(Color::Reset)
            } else {
                cell.fg
            });
            if cell.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if u16::try_from(row) == Ok(term.cursor_row)
                && u16::try_from(col) == Ok(term.cursor_col)
            {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let ch = if cell.ch == '\0' { ' ' } else { cell.ch };
            spans.push(Span::styled(ch.to_string(), style));
        }
        lines.push(Line::from(spans));
    }
    if let Some(error) = term.error.as_deref().filter(|msg| !msg.is_empty())
        && lines.is_empty()
    {
        lines.push(Line::from(Span::styled(error.to_string(), styles.alert)));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use super::{TerminalLink, TerminalState};

    #[test]
    fn title_includes_host_and_live() {
        let mut term = TerminalState {
            host: "192.168.88.1".into(),
            port: 22,
            link: TerminalLink::Live,
            ..TerminalState::default()
        };
        let title = term.title();
        assert!(title.contains("ssh 192.168.88.1:22"));
        assert!(title.contains("LIVE"));
        term.fullscreen = true;
        assert!(term.title().contains("FULL"));
    }
}
