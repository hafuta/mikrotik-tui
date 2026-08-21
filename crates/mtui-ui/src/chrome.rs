//! Header, status, and footer chrome.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::layout::fit_line;
use crate::styles::Styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalLevel {
    Idle,
    Good,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub label: String,
    pub value: String,
    pub level: SignalLevel,
}

impl Signal {
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>, level: SignalLevel) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            level,
        }
    }
}

#[must_use]
pub fn header_line(title: &str, subtitle: &str, styles: &Styles) -> Line<'static> {
    let mut spans = vec![Span::styled(format!(" {title} "), styles.title)];
    if !subtitle.is_empty() {
        spans.push(Span::styled(subtitle.to_string(), styles.muted));
    }
    Line::from(spans)
}

/// Compact status rail matching the Go `SignalRail` header.
#[must_use]
pub fn signal_rail(signals: &[Signal], width: usize, styles: &Styles) -> Line<'static> {
    if signals.is_empty() || width == 0 {
        return Line::default();
    }
    let mut spans = Vec::new();
    for (i, signal) in signals.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", styles.muted));
        }
        let text = format!("{} {}", signal.label, signal.value)
            .trim()
            .to_string();
        spans.push(Span::styled(text, style_for_level(signal.level, styles)));
    }
    fit_line(Line::from(spans), width)
}

fn style_for_level(level: SignalLevel, styles: &Styles) -> Style {
    match level {
        SignalLevel::Good => styles.signal,
        SignalLevel::Warning => styles.alert,
        SignalLevel::Error => styles.error,
        SignalLevel::Idle => styles.muted,
    }
}

#[must_use]
pub fn status_line(message: &str, styles: &Styles) -> Line<'static> {
    Line::from(Span::styled(format!(" {message}"), styles.muted))
}

#[must_use]
pub fn footer_hints(hints: &[(&str, &str)], styles: &Styles) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", styles.muted));
        }
        spans.push(Span::styled(key.to_string(), styles.focus));
        spans.push(Span::styled(format!(" {label}"), styles.muted));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use mtui_core::{DefaultTheme, Theme};

    use super::*;
    use crate::layout::{line_plain, line_width};

    fn styles() -> Styles {
        let theme = DefaultTheme::new();
        Styles::from_palette(theme.palette())
    }

    #[test]
    fn signal_rail_joins_with_separators_and_respects_width() {
        let styles = styles();
        let line = signal_rail(
            &[
                Signal::new("ROUTERDECK", "hEX S", SignalLevel::Good),
                Signal::new("https://router.test", "", SignalLevel::Good),
                Signal::new("user", "reader", SignalLevel::Idle),
                Signal::new("session", "1m0s", SignalLevel::Idle),
            ],
            80,
            &styles,
        );
        let plain = line_plain(&line);
        assert!(plain.contains("ROUTERDECK hEX S"));
        assert!(plain.contains("https://router.test"));
        assert!(plain.contains("user reader"));
        assert!(plain.contains("session 1m0s"));
        assert!(plain.contains(" │ "));
        assert_eq!(line_width(&line), 80);
    }

    #[test]
    fn signal_rail_is_empty_when_there_is_no_room() {
        let styles = styles();
        let line = signal_rail(
            &[Signal::new("router", "online", SignalLevel::Good)],
            0,
            &styles,
        );
        assert!(line_plain(&line).is_empty());
    }
}
