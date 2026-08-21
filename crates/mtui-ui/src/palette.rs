//! Command palette: ranked path search, compact overlay view.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::styles::Styles;

/// Visible match rows inside the palette (Go `paletteVisibleRows`).
pub const PALETTE_VISIBLE_ROWS: usize = 8;

/// A palette entry. Actions are identified by [`Command::id`] so the UI crate
/// stays free of application side effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub description: String,
    pub path: String,
}

impl Command {
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            path: String::new(),
        }
    }

    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

/// Compact, filterable command palette (parity with the Go overlay).
#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub commands: Vec<Command>,
    pub query: String,
    pub visible: bool,
    pub width: u16,
    selected: usize,
}

impl CommandPalette {
    #[must_use]
    pub fn new(commands: Vec<Command>) -> Self {
        Self {
            commands,
            query: String::new(),
            visible: false,
            width: 56,
            selected: 0,
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    #[must_use]
    pub fn matches(&self) -> Vec<&Command> {
        let query = self.query.trim().to_lowercase();
        if query.is_empty() {
            return self.commands.iter().collect();
        }
        let mut ranked: Vec<(usize, u16, &Command)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(order, command)| {
                let score = command_match_score(command, &query);
                (score > 0).then_some((order, score, command))
            })
            .collect();
        ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
        ranked.into_iter().map(|(_, _, command)| command).collect()
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Command> {
        let matches = self.matches();
        if matches.is_empty() {
            return None;
        }
        Some(matches[self.selected.min(matches.len() - 1)])
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.matches().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let next = self.selected.saturating_add_signed(delta);
        self.selected = next.min(len - 1);
    }

    pub fn insert_char(&mut self, ch: char) {
        if !is_printable(ch) {
            return;
        }
        self.query.push(ch);
        self.selected = 0;
        self.reconcile();
    }

    pub fn backspace(&mut self) {
        if self.query.pop().is_some() {
            self.selected = 0;
            self.reconcile();
        }
    }

    /// Close the palette and return the selected command id, if any.
    #[must_use]
    pub fn confirm(&mut self) -> Option<String> {
        let id = self.selected().map(|command| command.id.clone())?;
        self.visible = false;
        Some(id)
    }

    #[must_use]
    pub fn render_lines(&self, styles: &Styles) -> Vec<Line<'static>> {
        self.render_lines_at(self.clamped_width(), styles)
    }

    #[must_use]
    pub fn render_lines_at(&self, width: u16, styles: &Styles) -> Vec<Line<'static>> {
        if !self.visible {
            return Vec::new();
        }
        let width = width.clamp(24, 72);
        let matches = self.matches();
        let (start, end) = self.visible_range(matches.len());
        let mut title = vec![Span::styled("Command palette", styles.focus)];
        if matches.len() > PALETTE_VISIBLE_ROWS {
            title.push(Span::styled(
                format!("  {}", format_range(start + 1, end, matches.len())),
                styles.muted,
            ));
        }
        let mut lines = vec![
            Line::from(title),
            Line::from(Span::styled(format!("> {}▏", self.query), styles.text)),
        ];
        let cut = usize::from(width).saturating_sub(4).max(1);
        for (index, command) in matches.iter().enumerate().take(end).skip(start) {
            lines.push(render_match(
                command,
                index == self.selected,
                &self.query,
                cut,
                styles,
            ));
        }
        if matches.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No matching commands",
                styles.muted,
            )));
        }
        lines.push(Line::from(Span::styled(
            "↑↓ choose  enter open  esc close",
            styles.muted,
        )));
        lines
    }

    /// Paint the palette as a compact rounded modal over a dimmed canvas.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, styles: &Styles) {
        if !self.visible {
            return;
        }
        dim_canvas(frame, area, styles);
        let width = area.width.saturating_sub(4).min(64).clamp(24, 72);
        let lines = self.render_lines_at(width, styles);
        let content_h = u16::try_from(lines.len()).unwrap_or(u16::MAX);
        let height = content_h.saturating_add(4);
        let rect = compact_modal_rect(area, width, height);
        frame.render_widget(Clear, rect);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(styles.border)
            .style(styles.text)
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(rect);
        frame.render_widget(block, rect);
        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn clamped_width(&self) -> u16 {
        self.width.clamp(24, 72)
    }

    fn visible_range(&self, total: usize) -> (usize, usize) {
        if total == 0 {
            return (0, 0);
        }
        let start = if self.selected >= PALETTE_VISIBLE_ROWS {
            self.selected + 1 - PALETTE_VISIBLE_ROWS
        } else {
            0
        };
        (start, total.min(start + PALETTE_VISIBLE_ROWS))
    }

    fn reconcile(&mut self) {
        let n = self.matches().len();
        self.selected = if n == 0 { 0 } else { self.selected.min(n - 1) };
    }
}

fn command_match_score(command: &Command, query: &str) -> u16 {
    let title = command.title.to_lowercase();
    let description = command.description.to_lowercase();
    let mut path = command.path.trim().to_lowercase();
    if path.is_empty() {
        path.clone_from(&title);
    }
    let slashed = format!("/{query}");
    if path == query || path == slashed {
        300
    } else if path.starts_with(query) || path.starts_with(&slashed) {
        200
    } else if path.contains(&slashed) {
        150
    } else if path.contains(query) {
        100
    } else if title.contains(query) || description.contains(query) {
        50
    } else {
        0
    }
}

/// Highlight the first case-insensitive occurrence of `query` in `text`.
#[must_use]
pub fn highlight_match(text: &str, query: &str, base: Style, matched: Style) -> Vec<Span<'static>> {
    if query.is_empty() || text.is_empty() {
        return vec![Span::styled(text.to_string(), base)];
    }
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let Some(start) = lower_text.find(&lower_query) else {
        return vec![Span::styled(text.to_string(), base)];
    };
    let end = start + lower_query.len();
    if end > text.len() || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return vec![Span::styled(text.to_string(), base)];
    }
    let mut spans = Vec::with_capacity(3);
    if start > 0 {
        spans.push(Span::styled(text[..start].to_string(), base));
    }
    spans.push(Span::styled(text[start..end].to_string(), matched));
    if end < text.len() {
        spans.push(Span::styled(text[end..].to_string(), base));
    }
    spans
}

fn render_match(
    command: &Command,
    selected: bool,
    query: &str,
    width: usize,
    styles: &Styles,
) -> Line<'static> {
    let query = query.trim().to_lowercase();
    let base = if selected { styles.focus } else { styles.text };
    let matched = if selected {
        styles.signal.add_modifier(Modifier::BOLD)
    } else {
        styles.signal
    };
    let mut spans = Vec::new();
    if selected {
        spans.push(Span::styled("› ", styles.focus));
    } else {
        spans.push(Span::raw("  "));
    }
    spans.extend(highlight_match(&command.title, &query, base, matched));
    if !command.description.is_empty() {
        spans.push(Span::styled(" — ", base));
        spans.extend(highlight_match(
            &command.description,
            &query,
            styles.muted,
            matched,
        ));
    }
    Line::from(cut_spans(spans, width.max(1)))
}

fn cut_spans(spans: Vec<Span<'static>>, width: usize) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut used = 0usize;
    for span in spans {
        let span_width = span.content.width();
        if used + span_width <= width {
            used += span_width;
            out.push(span);
            continue;
        }
        let remaining = width.saturating_sub(used);
        if remaining == 0 {
            break;
        }
        out.push(Span::styled(
            cut_to_width(&span.content, remaining),
            span.style,
        ));
        break;
    }
    out
}

fn cut_to_width(value: &str, width: usize) -> String {
    if value.width() <= width {
        return value.to_string();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for ch in value.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

fn format_range(start: usize, end: usize, total: usize) -> String {
    format!("{start}\u{2013}{end}/{total}")
}

fn is_printable(ch: char) -> bool {
    !ch.is_control()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};

    use crate::layout::{line_plain, line_width, lines_plain};

    fn fixture() -> CommandPalette {
        let mut palette = CommandPalette::new(vec![
            Command::new("arp", "/ip/arp")
                .with_description("ARP")
                .with_path("/ip/arp"),
            Command::new("address", "/ip/address")
                .with_description("Addresses")
                .with_path("/ip/address"),
            Command::new("filter", "/ip/firewall/filter")
                .with_description("Firewall")
                .with_path("/ip/firewall/filter"),
            Command::new("iface", "/interface")
                .with_description("Interface")
                .with_path("/interface"),
            Command::new("leases", "/ip/dhcp-server/lease")
                .with_description("Leases")
                .with_path("/ip/dhcp-server/lease"),
            Command::new("refresh", "Refresh").with_description("reload the current resource"),
        ]);
        palette.visible = true;
        palette
    }

    fn ids(palette: &CommandPalette) -> Vec<&str> {
        palette.matches().iter().map(|c| c.id.as_str()).collect()
    }

    fn styles() -> Styles {
        let theme = DefaultTheme::new();
        Styles::from_palette(theme.palette())
    }

    #[test]
    fn matches_router_os_paths() {
        let mut palette = fixture();
        palette.query = "ip".into();
        assert_eq!(ids(&palette), ["arp", "address", "filter", "leases"]);

        palette.query = "/IP/firewall".into();
        assert_eq!(ids(&palette), ["filter"]);

        palette.query = "leases".into();
        assert_eq!(ids(&palette), ["leases"]);

        palette.query = "nat".into();
        assert!(ids(&palette).is_empty());
    }

    #[test]
    fn highlights_matched_path() {
        let base = Style::default();
        let matched = Style::default().add_modifier(Modifier::BOLD);
        let spans = highlight_match("/ip/firewall/filter", "ip", base, matched);
        assert_eq!(spans[0].content.as_ref(), "/");
        assert_eq!(spans[1].content.as_ref(), "ip");
        assert_eq!(spans[1].style, matched);
        assert_eq!(spans[2].content.as_ref(), "/firewall/filter");

        let mut palette = fixture();
        palette.query = "ip".into();
        let styles = styles();
        let lines = palette.render_lines(&styles);
        let view = lines_plain(&lines);
        assert!(
            view.contains("/ip/arp"),
            "palette view missing path: {view}"
        );
        let arp = lines
            .iter()
            .find(|line| line_plain(line).contains("/ip/arp"))
            .expect("arp row");
        assert!(
            arp.spans
                .iter()
                .any(|span| span.content.as_ref() == "ip" && span.style.fg == styles.signal.fg),
            "matched path was not highlighted: {arp:?}"
        );
    }

    #[test]
    fn enter_returns_selected_command() {
        let mut palette = CommandPalette::new(vec![
            Command::new("filter", "/ip/firewall/filter").with_path("/ip/firewall/filter"),
        ]);
        palette.visible = true;
        palette.query = "firewall".into();
        let id = palette.confirm();
        assert_eq!(id.as_deref(), Some("filter"));
        assert!(!palette.visible);
    }

    #[test]
    fn ignores_non_printable_input() {
        let mut palette = fixture();
        palette.insert_char('\0');
        palette.insert_char('\u{1}');
        assert_eq!(palette.query, "");
        palette.insert_char('I');
        palette.insert_char('P');
        assert_eq!(palette.query, "IP");
    }

    #[test]
    fn scrolls_visible_matches() {
        let commands: Vec<_> = (0..12)
            .map(|index| {
                let path = format!("/ip/item-{index:02}");
                Command::new(format!("item-{index:02}"), path.clone()).with_path(path)
            })
            .collect();
        let mut palette = CommandPalette::new(commands);
        palette.visible = true;
        palette.query = "ip".into();
        for _ in 0..8 {
            palette.move_selection(1);
        }
        let view = lines_plain(&palette.render_lines(&styles()));
        assert!(
            !view.contains("/ip/item-00") && view.contains("/ip/item-08"),
            "scrolled palette = {view}"
        );
    }

    #[test]
    fn empty_query_keeps_catalog_order() {
        let palette = fixture();
        assert_eq!(
            ids(&palette),
            ["arp", "address", "filter", "iface", "leases", "refresh"]
        );
    }

    #[test]
    fn lines_stay_within_width() {
        let mut palette = fixture();
        palette.query = "ip".into();
        palette.width = 32;
        for width in [32_u16, 80, 128] {
            let lines = palette.render_lines_at(width, &styles());
            let cap = usize::from(width.clamp(24, 72));
            for line in &lines {
                let w = line_width(line);
                assert!(w <= cap, "line width {w} > {cap}: {line:?}");
            }
            let again = palette.render_lines_at(width, &styles());
            assert_eq!(lines_plain(&lines), lines_plain(&again));
        }
    }
}
