//! Responsive layout breakpoints and clipping.

use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Narrow,
    Medium,
    Wide,
}

impl Breakpoint {
    #[must_use]
    pub fn from_width(width: u16) -> Self {
        if width < 72 {
            Self::Narrow
        } else if width < 112 {
            Self::Medium
        } else {
            Self::Wide
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LayoutMetrics {
    pub width: u16,
    pub height: u16,
    pub breakpoint: Breakpoint,
    pub nav_width: u16,
    pub inspector_width: u16,
}

impl LayoutMetrics {
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        let breakpoint = Breakpoint::from_width(width);
        let (nav_width, inspector_width) = match breakpoint {
            Breakpoint::Narrow => (0, 0),
            Breakpoint::Medium => (28, 0),
            Breakpoint::Wide => (30, 36),
        };
        Self {
            width,
            height,
            breakpoint,
            nav_width,
            inspector_width,
        }
    }
}

#[must_use]
pub fn clip_line(input: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if input.width() <= width {
        return input.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in input.chars() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    out
}

/// Pad or ellipsize `value` to `width` runes, matching the Go `fitCell` helper.
#[must_use]
pub fn fit_cell(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let replaced = value.replace('\n', " ");
    let runes: Vec<char> = replaced.chars().collect();
    if runes.len() > width {
        if width == 1 {
            return "…".to_string();
        }
        let mut out: String = runes.into_iter().take(width - 1).collect();
        out.push('…');
        return out;
    }
    let mut out: String = runes.iter().collect();
    out.push_str(&" ".repeat(width - runes.len()));
    out
}

/// Visual width of a ratatui line (span contents, no ANSI).
#[must_use]
pub fn line_width(line: &Line<'_>) -> usize {
    line.spans
        .iter()
        .map(|span| span.content.as_ref().width())
        .sum()
}

/// Concatenate span contents without styles.
#[must_use]
pub fn line_plain(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

/// Join line contents with newlines.
#[must_use]
pub fn lines_plain(lines: &[Line<'_>]) -> String {
    lines.iter().map(line_plain).collect::<Vec<_>>().join("\n")
}

fn clip_to_width(input: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if input.width() <= width {
        return input.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in input.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out
}

/// Truncate or pad a line to an exact visual width.
#[must_use]
pub fn fit_line(line: Line<'static>, width: usize) -> Line<'static> {
    if width == 0 {
        return Line::default();
    }
    let mut used = 0;
    let mut spans = Vec::new();
    for span in line.spans {
        if used >= width {
            break;
        }
        let content = span.content.as_ref();
        let w = content.width();
        if used + w <= width {
            used += w;
            spans.push(span);
            continue;
        }
        let truncated = clip_to_width(content, width - used);
        if !truncated.is_empty() {
            used += truncated.width();
            spans.push(Span::styled(truncated, span.style));
        }
        break;
    }
    if used < width {
        let pad = " ".repeat(width - used);
        let pad_style = spans
            .iter()
            .find_map(|span| span.style.bg)
            .map(|bg| ratatui::style::Style::default().bg(bg));
        spans.push(match pad_style {
            Some(style) => Span::styled(pad, style),
            None => Span::raw(pad),
        });
    }
    Line::from(spans)
}

/// Clip/pad lines to a fixed `width` × `height` canvas.
#[must_use]
pub fn constrain_lines(
    mut lines: Vec<Line<'static>>,
    width: usize,
    height: usize,
) -> Vec<Line<'static>> {
    let width = width.max(1);
    let height = height.max(1);
    if lines.len() > height {
        lines.truncate(height);
    }
    while lines.len() < height {
        lines.push(Line::from(" ".repeat(width)));
    }
    lines
        .into_iter()
        .map(|line| fit_line(line, width))
        .collect()
}

/// Join two column canvases with `gap` spaces, top-aligned.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn join_horizontal(
    left: Vec<Line<'static>>,
    right: Vec<Line<'static>>,
    gap: usize,
) -> Vec<Line<'static>> {
    let height = left.len().max(right.len());
    let left_width = left.first().map_or(0, line_width);
    let right_width = right.first().map_or(0, line_width);
    let mut out = Vec::with_capacity(height);
    for row in 0..height {
        let left_line = left
            .get(row)
            .cloned()
            .unwrap_or_else(|| Line::from(" ".repeat(left_width)));
        let right_line = right
            .get(row)
            .cloned()
            .unwrap_or_else(|| Line::from(" ".repeat(right_width)));
        let mut spans = fit_line(left_line, left_width.max(1)).spans;
        if gap > 0 {
            spans.push(Span::raw(" ".repeat(gap)));
        }
        spans.extend(fit_line(right_line, right_width.max(1)).spans);
        out.push(Line::from(spans));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints() {
        assert_eq!(Breakpoint::from_width(71), Breakpoint::Narrow);
        assert_eq!(Breakpoint::from_width(72), Breakpoint::Medium);
        assert_eq!(Breakpoint::from_width(112), Breakpoint::Wide);
    }

    #[test]
    fn medium_nav_fits_reset_configuration_label() {
        let metrics = LayoutMetrics::new(80, 24);
        assert_eq!(metrics.breakpoint, Breakpoint::Medium);
        let inner = usize::from(metrics.nav_width.saturating_sub(4));
        assert!(
            inner >= "  Reset Configuration".chars().count(),
            "nav inner {inner} clips Reset Configuration"
        );
    }
}
