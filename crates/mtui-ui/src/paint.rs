//! Bounded background fills.
//!
//! Shared theme styles stay foreground-only. A background is applied only to
//! an explicit rectangle (header band, footer band, selected row). Tests
//! prove the fill cannot escape that rectangle.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;

/// Paint every cell in `area` with `bg`. No-op on an empty rect.
pub fn fill_rect(frame: &mut Frame<'_>, area: Rect, bg: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    frame.render_widget(Block::default().style(Style::default().bg(bg)), area);
}

/// Copy `bg` onto every span. Does not pad; pair with [`crate::layout::fit_line`].
#[must_use]
pub fn line_on_bg(line: Line<'static>, bg: Color) -> Line<'static> {
    Line::from(
        line.spans
            .into_iter()
            .map(|span| Span::styled(span.content, span.style.bg(bg)))
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    use super::{fill_rect, line_on_bg};
    use crate::layout::{fit_line, line_width};

    const FILL: Color = Color::Rgb(26, 41, 61);
    const VOID: Color = Color::Rgb(12, 17, 24);

    #[test]
    fn fill_rect_does_not_escape_its_area() {
        let backend = TestBackend::new(12, 6);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let painted = Rect::new(2, 1, 5, 3);
        terminal
            .draw(|frame| {
                fill_rect(frame, frame.area(), VOID);
                fill_rect(frame, painted, FILL);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                let inside = x >= painted.x
                    && x < painted.x + painted.width
                    && y >= painted.y
                    && y < painted.y + painted.height;
                if inside {
                    assert_eq!(cell.bg, FILL, "missing fill at ({x},{y})");
                } else {
                    assert_eq!(cell.bg, VOID, "fill escaped to ({x},{y})");
                }
            }
        }
    }

    #[test]
    fn selected_row_fill_stays_on_that_row() {
        let backend = TestBackend::new(16, 5);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let body = Rect::new(0, 1, 16, 3);
        let row = Rect::new(0, 2, 16, 1);
        terminal
            .draw(|frame| {
                fill_rect(frame, frame.area(), VOID);
                fill_rect(frame, body, VOID);
                let line = fit_line(
                    line_on_bg(Line::from(Span::styled("ether1", Style::default())), FILL),
                    usize::from(row.width),
                );
                frame.render_widget(Paragraph::new(line), row);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        for x in 0..16 {
            assert_eq!(buf[(x, 2)].bg, FILL, "row gap at x={x}");
        }
        for y in [0_u16, 1, 3, 4] {
            for x in 0..16 {
                assert_eq!(buf[(x, y)].bg, VOID, "row fill escaped to ({x},{y})");
            }
        }
    }

    #[test]
    fn line_on_bg_keeps_width_when_fitted() {
        let line = fit_line(line_on_bg(Line::from("abc"), FILL), 10);
        assert_eq!(line_width(&line), 10);
        assert!(line.spans.iter().all(|span| span.style.bg == Some(FILL)));
    }
}
