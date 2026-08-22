//! Centered modal overlay helpers.
//!
//! Overlays dim the already-drawn canvas with faint muted foreground (never a
//! background fill), then punch a compact bordered dialog through the center.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::styles::Styles;

/// Visual treatment for a generic modal chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalKind {
    Default,
    Alert,
}

/// Semantic weight for a footer action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalButtonKind {
    Primary,
    Secondary,
}

/// A labeled footer action with its keyboard shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalButton {
    pub label: &'static str,
    pub keys: &'static str,
    pub kind: ModalButtonKind,
}

/// Optional inset panel inside a modal (fingerprints, codes, quotes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalPanel<'a> {
    pub label: &'a str,
    pub value: &'a str,
}

/// Content-driven dialog. Callers supply copy and actions; layout, dimming,
/// and button chrome stay here so every overlay looks consistent.
#[derive(Debug, Clone)]
pub struct Modal<'a> {
    pub title: &'a str,
    pub kind: ModalKind,
    pub kicker: Option<&'a str>,
    pub body: &'a str,
    pub panel: Option<ModalPanel<'a>>,
    pub accent_heading: Option<&'a str>,
    pub hint: Option<&'a str>,
    pub buttons: &'a [ModalButton],
    pub scroll: u16,
}

impl<'a> Modal<'a> {
    #[must_use]
    pub fn new(title: &'a str, body: &'a str) -> Self {
        Self {
            title,
            kind: ModalKind::Default,
            kicker: None,
            body,
            panel: None,
            accent_heading: None,
            hint: None,
            buttons: &[],
            scroll: 0,
        }
    }

    #[must_use]
    pub fn alert(mut self) -> Self {
        self.kind = ModalKind::Alert;
        self
    }

    #[must_use]
    pub fn kicker(mut self, kicker: &'a str) -> Self {
        self.kicker = Some(kicker);
        self
    }

    #[must_use]
    pub fn panel(mut self, panel: ModalPanel<'a>) -> Self {
        self.panel = Some(panel);
        self
    }

    #[must_use]
    pub fn accent_heading(mut self, heading: &'a str) -> Self {
        self.accent_heading = Some(heading);
        self
    }

    #[must_use]
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    #[must_use]
    pub fn buttons(mut self, buttons: &'a [ModalButton]) -> Self {
        self.buttons = buttons;
        self
    }

    #[must_use]
    pub fn scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }
}

/// Compute a centered modal rect inside `area` from percentage sizes.
#[must_use]
pub fn modal_rect(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let width = area.width.saturating_mul(width_pct) / 100;
    let height = area.height.saturating_mul(height_pct) / 100;
    compact_modal_rect(area, width.max(20), height.max(5))
}

/// Center a fixed-size dialog, clamped so it stays inside `area` with a 1-cell
/// outer margin when the terminal allows it.
#[must_use]
pub fn compact_modal_rect(area: Rect, width: u16, height: u16) -> Rect {
    let max_w = area.width.saturating_sub(2).max(1).min(area.width.max(1));
    let max_h = area.height.saturating_sub(2).max(1).min(area.height.max(1));
    let width = width.max(1).min(max_w);
    let height = height.max(1).min(max_h);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// Replace cell styling in `area` with faint muted foreground, keeping glyphs.
/// Does not paint a background, so the dim cannot bleed outside the canvas.
pub fn dim_canvas(frame: &mut Frame<'_>, area: Rect, styles: &Styles) {
    let buf = frame.buffer_mut();
    let area = area.intersection(*buf.area());
    let style = styles.muted.add_modifier(Modifier::DIM);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_style(style);
        }
    }
}

/// Format a SHA-256 hex fingerprint as grouped uppercase quartets.
#[must_use]
pub fn format_fingerprint(pin: &str) -> String {
    let compact: String = pin
        .chars()
        .filter(|ch| *ch != ':' && !ch.is_whitespace())
        .map(|ch| ch.to_ascii_uppercase())
        .collect();
    let mut parts = Vec::new();
    let mut rest = compact.as_str();
    while !rest.is_empty() {
        let (head, tail) = rest.split_at(rest.len().min(4));
        parts.push(head);
        rest = tail;
    }
    parts.join(" ")
}

/// Dim the canvas and draw a generic content-sized modal.
///
/// Body, kicker, and panel scroll. Hint and buttons stay pinned to the bottom
/// of the dialog so they cannot scroll off the viewport. The dialog hugs its
/// content; scroll stops when the last body line reaches the bottom of the
/// viewport.
pub fn render_modal(frame: &mut Frame<'_>, area: Rect, modal: &Modal<'_>, styles: &Styles) {
    dim_canvas(frame, area, styles);

    let prepared = prepare_modal(area, modal, styles);
    let rect = compact_modal_rect(area, prepared.width, prepared.height);

    frame.render_widget(Clear, rect);
    let border = match modal.kind {
        ModalKind::Default => styles.border,
        ModalKind::Alert => styles.alert,
    };
    let title = format!(" {} ", modal.title.trim());
    let block = Block::default()
        .title(Span::styled(title, styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border)
        .style(styles.text)
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let (body_area, footer_area) = split_modal_inner(inner, prepared.footer_h);
    if let Some(body_area) = body_area {
        let visible = usize::from(body_area.height);
        let start = clamp_scroll(modal.scroll, prepared.body_lines.len(), visible);
        let view: Vec<Line<'static>> = prepared
            .body_lines
            .into_iter()
            .skip(start)
            .take(visible)
            .collect();
        frame.render_widget(
            Paragraph::new(view)
                .style(styles.text)
                .wrap(Wrap { trim: false }),
            body_area,
        );
    }
    if let Some(footer_area) = footer_area {
        frame.render_widget(
            Paragraph::new(prepared.footer_lines)
                .style(styles.text)
                .wrap(Wrap { trim: false }),
            footer_area,
        );
    }
}

/// Largest `scroll` that still keeps the last body line in view.
#[must_use]
pub fn modal_max_scroll(area: Rect, modal: &Modal<'_>, styles: &Styles) -> u16 {
    let prepared = prepare_modal(area, modal, styles);
    let rect = compact_modal_rect(area, prepared.width, prepared.height);
    let inner = Rect {
        x: 0,
        y: 0,
        width: rect.width.saturating_sub(6),
        height: rect.height.saturating_sub(4),
    };
    let (body_area, _) = split_modal_inner(inner, prepared.footer_h);
    let visible = body_area.map_or(0, |rect| usize::from(rect.height));
    u16::try_from(prepared.body_lines.len().saturating_sub(visible)).unwrap_or(u16::MAX)
}

/// Convenience wrapper for a title + plain body (help text, simple notices).
pub fn render_modal_frame(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    body: &str,
    styles: &Styles,
    scroll: u16,
) {
    let modal = Modal::new(title.trim(), body).scroll(scroll);
    render_modal(frame, area, &modal, styles);
}

struct PreparedModal {
    width: u16,
    height: u16,
    footer_h: u16,
    body_lines: Vec<Line<'static>>,
    footer_lines: Vec<Line<'static>>,
}

fn prepare_modal(area: Rect, modal: &Modal<'_>, styles: &Styles) -> PreparedModal {
    let width = modal_outer_width(area);
    let inner_width = usize::from(width.saturating_sub(6).max(8));
    let body_lines = modal_body_lines(modal, inner_width, styles);
    let footer_lines = modal_footer_lines(modal, inner_width, styles);
    let body_h = u16::try_from(body_lines.len()).unwrap_or(u16::MAX);
    let footer_h = u16::try_from(footer_lines.len()).unwrap_or(u16::MAX);
    let height = body_h.saturating_add(footer_h).saturating_add(4).max(5);
    PreparedModal {
        width,
        height,
        footer_h,
        body_lines,
        footer_lines,
    }
}

fn clamp_scroll(scroll: u16, line_count: usize, visible: usize) -> usize {
    usize::from(scroll).min(line_count.saturating_sub(visible))
}

fn modal_outer_width(area: Rect) -> u16 {
    area.width.saturating_sub(4).min(64).clamp(28, 72)
}

fn split_modal_inner(inner: Rect, footer_h: u16) -> (Option<Rect>, Option<Rect>) {
    if inner.height == 0 {
        return (None, None);
    }
    if footer_h == 0 {
        return (Some(inner), None);
    }
    let footer_h = footer_h.min(inner.height);
    if footer_h == inner.height {
        return (None, Some(inner));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_h)])
        .split(inner);
    (Some(chunks[0]), Some(chunks[1]))
}

fn modal_body_lines(modal: &Modal<'_>, width: usize, styles: &Styles) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(kicker) = modal.kicker {
        let style = match modal.kind {
            ModalKind::Alert => styles.alert.add_modifier(Modifier::BOLD),
            ModalKind::Default => styles.focus,
        };
        lines.extend(wrap_styled(kicker, width, style));
        lines.push(Line::default());
    }
    if !modal.body.trim().is_empty() {
        for paragraph in modal.body.split("\n\n") {
            for (index, chunk) in paragraph.split('\n').enumerate() {
                if index > 0 && chunk.is_empty() {
                    lines.push(Line::default());
                    continue;
                }
                let style = if modal.accent_heading == Some(chunk) {
                    styles.signal.add_modifier(Modifier::BOLD)
                } else {
                    styles.text
                };
                lines.extend(wrap_styled(chunk, width, style));
            }
            lines.push(Line::default());
        }
        if lines.last().is_some_and(|line| line.spans.is_empty()) {
            lines.pop();
        }
    }
    if let Some(panel) = modal.panel {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.push(Line::from(Span::styled(
            panel.label.to_ascii_uppercase(),
            styles.muted,
        )));
        lines.extend(wrap_styled(panel.value, width, styles.signal));
    }
    lines
}

fn modal_footer_lines(modal: &Modal<'_>, width: usize, styles: &Styles) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(hint) = modal.hint {
        lines.extend(wrap_styled(hint, width, styles.muted));
    }
    if !modal.buttons.is_empty() {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.extend(button_lines(modal.buttons, width, styles));
    }
    lines
}

fn button_lines(buttons: &[ModalButton], width: usize, styles: &Styles) -> Vec<Line<'static>> {
    let mut spans = Vec::new();
    for (index, button) in buttons.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("   ", styles.muted));
        }
        spans.extend(button_spans(*button, styles));
    }
    let line = Line::from(spans);
    if crate::layout::line_width(&line) <= width {
        return vec![line];
    }
    buttons
        .iter()
        .map(|button| Line::from(button_spans(*button, styles)))
        .collect()
}

fn button_spans(button: ModalButton, styles: &Styles) -> Vec<Span<'static>> {
    let (bracket, label) = match button.kind {
        ModalButtonKind::Primary => (styles.focus, styles.focus.add_modifier(Modifier::BOLD)),
        ModalButtonKind::Secondary => (styles.border, styles.text),
    };
    vec![
        Span::styled("[ ", bracket),
        Span::styled(button.label.to_string(), label),
        Span::styled(" ]", bracket),
        Span::styled(format!(" {}", button.keys), styles.muted),
    ]
}

fn wrap_styled(text: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    wrap_words(text, width.max(1))
        .into_iter()
        .map(|part| Line::from(Span::styled(part, style)))
        .collect()
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if word.width() > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let mut chunk = String::new();
            for ch in word.chars() {
                let next = format!("{chunk}{ch}");
                if next.width() > width && !chunk.is_empty() {
                    lines.push(std::mem::take(&mut chunk));
                    chunk.push(ch);
                } else {
                    chunk = next;
                }
            }
            current = chunk;
            continue;
        }
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if candidate.width() > width {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::widgets::Paragraph;

    use crate::styles::Styles;

    #[test]
    fn modal_is_centered() {
        let area = Rect::new(0, 0, 100, 40);
        let m = modal_rect(area, 50, 50);
        assert!(m.x > 0);
        assert!(m.y > 0);
        assert!(m.x + m.width <= area.width);
        assert!(m.y + m.height <= area.height);
    }

    #[test]
    fn compact_modal_is_centered_and_clamped() {
        let area = Rect::new(0, 0, 80, 24);
        let m = compact_modal_rect(area, 56, 14);
        assert_eq!(m.width, 56);
        assert_eq!(m.x, (80 - 56) / 2);
        assert_eq!(m.y, (24 - 14) / 2);
        let tiny = compact_modal_rect(Rect::new(0, 0, 10, 6), 64, 20);
        assert!(tiny.width <= 10);
        assert!(tiny.height <= 6);
        assert!(tiny.x + tiny.width <= 10);
        assert!(tiny.y + tiny.height <= 6);
    }

    #[test]
    fn dim_canvas_uses_foreground_only() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("hello world"), frame.area());
                dim_canvas(frame, frame.area(), &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let cell = &buf[(0, 0)];
        assert_eq!(cell.bg, Color::Reset);
        assert_eq!(cell.fg, styles.muted.fg.unwrap_or(Color::Reset));
        assert!(cell.modifier.contains(Modifier::DIM));
        assert_eq!(cell.symbol(), "h");
    }

    #[test]
    fn format_fingerprint_groups_uppercase_quartets() {
        let pin = "ab".repeat(32);
        let formatted = format_fingerprint(&pin);
        assert!(formatted.starts_with("ABAB ABAB"));
        assert!(!formatted.contains(':'));
        assert_eq!(formatted.chars().filter(|ch| *ch == ' ').count(), 15);
    }

    #[test]
    fn generic_modal_renders_labeled_buttons() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let fingerprint = format_fingerprint(&"12".repeat(32));
        let buttons = [
            ModalButton {
                label: "Trust",
                keys: "y / enter",
                kind: ModalButtonKind::Primary,
            },
            ModalButton {
                label: "Cancel",
                keys: "n / esc",
                kind: ModalButtonKind::Secondary,
            },
        ];
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("backdrop"), frame.area());
                let modal = Modal::new("Certificate", "Verify this SHA-256 fingerprint.")
                    .alert()
                    .kicker("Unrecognized router certificate")
                    .panel(ModalPanel {
                        label: "SHA-256",
                        value: &fingerprint,
                    })
                    .hint("Credentials are not sent until you approve.")
                    .buttons(&buttons);
                render_modal(frame, frame.area(), &modal, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(rendered.contains("Certificate"));
        assert!(rendered.contains("Unrecognized router certificate"));
        assert!(rendered.contains("[ Trust ]"));
        assert!(rendered.contains("y / enter"));
        assert!(rendered.contains("[ Cancel ]"));
        assert!(rendered.contains("n / esc"));
        assert!(rendered.contains("1212 1212"));
    }

    #[test]
    fn modal_hint_stays_pinned_when_body_scrolls() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 16);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let body = (0..40)
            .map(|i| format!("line-{i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        terminal
            .draw(|frame| {
                let modal = Modal::new("About MACsec", &body)
                    .hint("esc close · j/k scroll")
                    .scroll(30);
                render_modal(frame, frame.area(), &modal, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        let mut last_content_row = 0;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if row.contains("esc close") {
                last_content_row = y;
            }
            rendered.push_str(&row);
            rendered.push('\n');
        }
        assert!(
            rendered.contains("esc close · j/k scroll"),
            "pinned hint missing after scroll: {rendered}"
        );
        assert!(
            !rendered.contains("line-00"),
            "scrolled body should have left the first line: {rendered}"
        );
        assert!(
            last_content_row > buf.area.height / 2,
            "hint should sit in the lower half of the canvas, got row {last_content_row}"
        );
    }

    #[test]
    fn modal_scroll_stops_at_last_body_line() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let area = Rect::new(0, 0, 80, 24);
        let short = Modal::new("About", "one\ntwo\nthree").hint("esc close · j/k scroll");
        assert_eq!(modal_max_scroll(area, &short, &styles), 0);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| {
                render_modal(frame, frame.area(), &short.clone().scroll(40), &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        assert!(
            rendered.contains("one"),
            "fitting copy must stay at the top when scroll is overshot: {rendered}"
        );
        assert!(
            rendered.contains("three"),
            "last line must remain visible: {rendered}"
        );
    }

    #[test]
    fn wrap_words_respects_width() {
        let lines = wrap_words("verify this fingerprint through a trusted channel", 18);
        assert!(lines.iter().all(|line| line.width() <= 18));
        assert!(lines.len() > 1);
    }

    #[test]
    fn accent_heading_uses_signal_color_only_on_that_line() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let body = "Summary of the feature.\n\nWhen you need it\nUse this for the real job.\n\nNotable fields\nname, comment.";
        terminal
            .draw(|frame| {
                let modal = Modal::new("About", body).accent_heading("When you need it");
                render_modal(frame, frame.area(), &modal, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut saw_heading = false;
        let mut saw_body = false;
        for y in 0..buf.area.height {
            let mut row = String::new();
            for x in 0..buf.area.width {
                row.push_str(buf[(x, y)].symbol());
            }
            if let Some(idx) = row.find("When you need it") {
                saw_heading = true;
                let x = u16::try_from(idx).expect("column");
                assert_eq!(buf[(x, y)].fg, styles.signal.fg.expect("signal fg"));
            }
            if let Some(idx) = row.find("Use this for the real job") {
                saw_body = true;
                let x = u16::try_from(idx).expect("column");
                assert_eq!(buf[(x, y)].fg, styles.text.fg.expect("text fg"));
            }
        }
        assert!(saw_heading && saw_body);
    }
}
