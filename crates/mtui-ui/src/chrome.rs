//! Header, tab bar, status, and footer chrome.

use std::time::{Duration, Instant};

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::layout::{clip_line, fit_line, line_width};
use crate::styles::Styles;

/// Hide the activity pulse until a request has lasted this long.
pub const ACTIVITY_SHOW_AFTER: Duration = Duration::from_millis(280);

const ACTIVITY_SLOT: usize = 2;

/// True when a busy operation has lasted long enough to be worth showing.
#[must_use]
pub fn activity_shown(busy_since: Option<Instant>, now: Instant) -> bool {
    busy_since.is_some_and(|started| now.saturating_duration_since(started) >= ACTIVITY_SHOW_AFTER)
}

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
    let mut spans = vec![Span::styled(format!("  {title}"), styles.title)];
    if !subtitle.is_empty() {
        spans.push(Span::styled(format!("  {subtitle}"), styles.muted));
    }
    Line::from(spans)
}

/// Product title, identity, and right-aligned live metrics (Deck mock header).
///
/// The last two columns are a reserved activity pulse so busy state never
/// rewrites the status or metrics text.
#[must_use]
pub fn session_header(
    product: &str,
    identity: &str,
    metrics: &[Signal],
    width: usize,
    styles: &Styles,
    activity: bool,
) -> Line<'static> {
    if width == 0 {
        return Line::default();
    }
    let inner = width.saturating_sub(ACTIVITY_SLOT.min(width));
    let mut left = vec![Span::styled(format!("  {product}"), styles.title)];
    if !identity.is_empty() {
        left.push(Span::styled(format!("  {identity}"), styles.muted));
    }
    let left_line = Line::from(left);
    let content = if inner == 0 {
        Line::default()
    } else {
        let left_w = line_width(&left_line);
        if metrics.is_empty() || left_w + 2 >= inner {
            fit_line(left_line, inner)
        } else {
            let rest = inner.saturating_sub(left_w);
            let rail = signal_rail(metrics, rest.saturating_sub(1), styles);
            let rail_w = line_width(&rail);
            let pad = rest.saturating_sub(rail_w).max(1);
            let mut spans = left_line.spans;
            spans.push(Span::raw(" ".repeat(pad)));
            spans.extend(rail.spans);
            fit_line(Line::from(spans), inner)
        }
    };
    if width < ACTIVITY_SLOT {
        return content;
    }
    let mut spans = content.spans;
    if activity {
        spans.push(Span::styled(" ●", styles.signal));
    } else {
        spans.push(Span::raw("  "));
    }
    fit_line(Line::from(spans), width)
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
            spans.push(Span::styled(" · ", styles.muted));
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
        spans.push(Span::styled(key.to_string(), styles.key));
        spans.push(Span::styled(format!(" {label}"), styles.muted));
    }
    Line::from(spans)
}

/// One session tab. `id` is opaque and is not interpreted by the widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabLabel {
    pub id: u64,
    pub title: String,
    pub connected: bool,
}

impl TabLabel {
    #[must_use]
    pub fn new(id: u64, title: impl Into<String>, connected: bool) -> Self {
        Self {
            id,
            title: title.into(),
            connected,
        }
    }
}

/// Numbered session tabs clipped to `width`.
///
/// Tabs render as `1:title  2:title`. The active tab is `[n]:title` and bold
/// so it is not color-only. Disconnected tabs use muted foreground.
#[must_use]
pub fn tab_bar(tabs: &[TabLabel], active: u64, width: usize, styles: &Styles) -> Line<'static> {
    if width == 0 {
        return Line::default();
    }
    if tabs.is_empty() {
        return fit_line(Line::default(), width);
    }
    let mut spans = Vec::with_capacity(tabs.len().saturating_mul(2));
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let n = i + 1;
        let is_active = tab.id == active;
        let label = if is_active {
            format!("[{n}]:{}", tab.title)
        } else {
            format!("{n}:{}", tab.title)
        };
        let style = if is_active {
            styles.focus
        } else if tab.connected {
            styles.text
        } else {
            styles.muted
        };
        spans.push(Span::styled(label, style));
    }
    fit_line(Line::from(spans), width)
}

/// Hints on the left, optional status clipped to the right.
#[must_use]
pub fn footer_bar(
    status: &str,
    hints: &[(&str, &str)],
    width: usize,
    styles: &Styles,
) -> Line<'static> {
    if width == 0 {
        return Line::default();
    }
    let hints_line = footer_hints(hints, styles);
    let status = status.trim();
    if status.is_empty() {
        return fit_line(hints_line, width);
    }
    let hint_w = line_width(&hints_line);
    if hint_w + 2 >= width {
        return fit_line(hints_line, width);
    }
    let rest = width.saturating_sub(hint_w);
    let clipped = clip_line(status, rest.saturating_sub(1));
    let pad = rest.saturating_sub(clipped.width()).max(1);
    let mut spans = hints_line.spans;
    spans.push(Span::raw(" ".repeat(pad)));
    spans.push(Span::styled(clipped, styles.muted));
    fit_line(Line::from(spans), width)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use mtui_core::{DefaultTheme, Theme};
    use ratatui::style::Modifier;

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
        assert!(plain.contains(" · "));
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

    #[test]
    fn session_header_puts_metrics_on_the_right() {
        let styles = styles();
        let line = session_header(
            "mikrotik-tui",
            "CCR2004 · 192.0.2.1",
            &[
                Signal::new("CPU", "18%", SignalLevel::Good),
                Signal::new("MEM", "41%", SignalLevel::Good),
                Signal::new("WAN", "84.2 Mb/s", SignalLevel::Good),
            ],
            80,
            &styles,
            false,
        );
        let plain = line_plain(&line);
        assert!(plain.contains("mikrotik-tui"));
        assert!(plain.contains("CCR2004 · 192.0.2.1"));
        assert!(plain.contains("CPU 18%"));
        assert!(plain.contains("WAN 84.2 Mb/s"));
        assert_eq!(line_width(&line), 80);
        let product = plain.find("mikrotik-tui").expect("product");
        let wan = plain.find("WAN").expect("wan");
        assert!(product < wan);
        assert!(!plain.contains('●'));
    }

    #[test]
    fn session_header_reserves_activity_slot_without_shifting_metrics() {
        let styles = styles();
        let metrics = [
            Signal::new("CPU", "18%", SignalLevel::Good),
            Signal::new("MEM", "41%", SignalLevel::Good),
            Signal::new("WAN", "84.2 Mb/s", SignalLevel::Good),
        ];
        let idle = session_header(
            "mikrotik-tui",
            "CCR2004 · 192.0.2.1",
            &metrics,
            80,
            &styles,
            false,
        );
        let busy = session_header(
            "mikrotik-tui",
            "CCR2004 · 192.0.2.1",
            &metrics,
            80,
            &styles,
            true,
        );
        let idle_plain = line_plain(&idle);
        let busy_plain = line_plain(&busy);
        let idle_wan = idle_plain.find("WAN").expect("idle wan");
        let busy_wan = busy_plain.find("WAN").expect("busy wan");
        assert_eq!(idle_wan, busy_wan);
        assert!(busy_plain.contains('●'));
        assert!(!idle_plain.contains('●'));
        assert_eq!(line_width(&idle), 80);
        assert_eq!(line_width(&busy), 80);
    }

    #[test]
    fn activity_pulse_waits_out_fast_requests() {
        let started = Instant::now();
        assert!(!activity_shown(None, started));
        assert!(!activity_shown(Some(started), started));
        assert!(!activity_shown(
            Some(started),
            started + Duration::from_millis(100)
        ));
        assert!(activity_shown(Some(started), started + ACTIVITY_SHOW_AFTER));
    }

    #[test]
    fn footer_bar_keeps_hints_left_and_status_right() {
        let styles = styles();
        let line = footer_bar(
            "resource loaded interfaces",
            &[("enter", "edit"), ("q", "quit")],
            60,
            &styles,
        );
        let plain = line_plain(&line);
        assert!(plain.starts_with("enter edit"));
        assert!(plain.contains("resource loaded interfaces"));
        assert_eq!(line_width(&line), 60);
    }

    #[test]
    fn tab_bar_respects_width() {
        let styles = styles();
        let tabs = [
            TabLabel::new(10, "edge-office", true),
            TabLabel::new(20, "core", true),
        ];
        let line = tab_bar(&tabs, 10, 40, &styles);
        assert_eq!(line_width(&line), 40);
        for span in &line.spans {
            assert!(span.style.bg.is_none());
        }
    }

    #[test]
    fn tab_bar_marks_active_tab() {
        let styles = styles();
        let tabs = [
            TabLabel::new(10, "Login", false),
            TabLabel::new(20, "core", true),
        ];
        let line = tab_bar(&tabs, 20, 48, &styles);
        let plain = line_plain(&line);
        assert!(plain.contains("1:Login"));
        assert!(plain.contains("[2]:core"));
        assert!(!plain.contains("[1]:"));
        let active = line
            .spans
            .iter()
            .find(|span| span.content.contains("[2]:core"))
            .expect("active span");
        assert!(active.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn tab_bar_clips_narrow_width_without_panic() {
        let styles = styles();
        let tabs = [TabLabel::new(1, "very-long-profile-name", true)];
        let empty = tab_bar(&tabs, 1, 0, &styles);
        assert!(line_plain(&empty).is_empty());
        let tiny = tab_bar(&tabs, 1, 3, &styles);
        assert_eq!(line_width(&tiny), 3);
        let none = tab_bar(&[], 99, 8, &styles);
        assert_eq!(line_width(&none), 8);
    }

    #[test]
    fn tab_bar_shows_both_tabs_when_width_allows() {
        let styles = styles();
        let tabs = [
            TabLabel::new(1, "alpha", true),
            TabLabel::new(2, "beta", false),
        ];
        let line = tab_bar(&tabs, 1, 32, &styles);
        let plain = line_plain(&line);
        assert!(plain.contains("[1]:alpha"));
        assert!(plain.contains("2:beta"));
    }
}
