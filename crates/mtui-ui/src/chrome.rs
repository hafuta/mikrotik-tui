//! Header, tab bar, status, and footer chrome.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::layout::{clip_line, fit_line, line_width};
use crate::paint::fill_rect;
use crate::styles::Styles;

/// Hide the activity pulse until a request has lasted this long.
pub const ACTIVITY_SHOW_AFTER: Duration = Duration::from_millis(280);

const ACTIVITY_SLOT: usize = 2;
const CHROME_GUTTER: usize = 2;

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
/// rewrites the status or metrics text. `trailing` stays on the right of the
/// metrics rail so a long Safe Mode label is not clipped first.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn session_header(
    product: &str,
    identity: &str,
    metrics: &[Signal],
    trailing: &[Signal],
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
        join_header_rails(left_line, metrics, trailing, inner, styles)
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

fn join_header_rails(
    left: Line<'static>,
    metrics: &[Signal],
    trailing: &[Signal],
    inner: usize,
    styles: &Styles,
) -> Line<'static> {
    let left_w = line_width(&left);
    if left_w + 2 >= inner || (metrics.is_empty() && trailing.is_empty()) {
        return fit_line(left, inner);
    }
    let rest = inner.saturating_sub(left_w);
    let trailing_need = rail_width(trailing, styles).min(rest.saturating_sub(1));
    let trailing_line = signal_rail(trailing, trailing_need, styles);
    let trailing_w = line_width(&trailing_line);
    let sep = usize::from(trailing_w > 0 && !metrics.is_empty()) * 3;
    let metrics_budget = rest
        .saturating_sub(trailing_w)
        .saturating_sub(sep)
        .saturating_sub(1);
    let metrics_line = signal_rail(metrics, metrics_budget, styles);
    let metrics_w = line_width(&metrics_line);
    let used = metrics_w.saturating_add(sep).saturating_add(trailing_w);
    let pad = rest.saturating_sub(used).max(1);
    let mut spans = left.spans;
    spans.push(Span::raw(" ".repeat(pad)));
    spans.extend(metrics_line.spans);
    if metrics_w > 0 && trailing_w > 0 {
        spans.push(Span::styled(" · ", styles.muted));
    }
    spans.extend(trailing_line.spans);
    fit_line(Line::from(spans), inner)
}

/// Compact status rail matching the Go `SignalRail` header.
#[must_use]
pub fn signal_rail(signals: &[Signal], width: usize, styles: &Styles) -> Line<'static> {
    if signals.is_empty() || width == 0 {
        return Line::default();
    }
    fit_line(Line::from(signal_spans(signals, styles)), width)
}

fn signal_spans(signals: &[Signal], styles: &Styles) -> Vec<Span<'static>> {
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
    spans
}

fn rail_width(signals: &[Signal], styles: &Styles) -> usize {
    if signals.is_empty() {
        return 0;
    }
    line_width(&Line::from(signal_spans(signals, styles)))
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

const TAB_TITLE_MAX: usize = 18;
const TAB_MIN_WIDTH: usize = 12;
const TAB_GAP: usize = 1;
const TAB_SIDE_PAD: usize = 1;
const PLUS_SLOT: usize = 2;
const LIVE_MARK: &str = " ●";
const TAB_CAP_TOP: char = '▄';
const TAB_CAP_BOT: char = '▀';

/// Rows reserved for the session tab strip. Shrinks on short terminals.
pub const TAB_STRIP_HEIGHT: u16 = 3;

/// Rows reserved for the session header and footer bands.
pub const CHROME_BAND_HEIGHT: u16 = 3;

/// Height of the session strip for this terminal.
#[must_use]
pub fn tab_strip_height(terminal_height: u16) -> u16 {
    if terminal_height < 12 {
        1
    } else {
        TAB_STRIP_HEIGHT
    }
}

/// Height of the session header and footer for this terminal.
#[must_use]
pub fn chrome_band_height(terminal_height: u16) -> u16 {
    if terminal_height < 16 {
        1
    } else {
        CHROME_BAND_HEIGHT
    }
}

/// Place `content` on the middle row of a band so it stays vertically centered.
#[must_use]
pub fn center_in_band(content: &Line<'static>, height: u16, width: usize) -> Vec<Line<'static>> {
    let height = usize::from(height.max(1));
    let mid = height.saturating_sub(1) / 2;
    let mut lines = Vec::with_capacity(height);
    for index in 0..height {
        if index == mid {
            lines.push(fit_line(content.clone(), width));
        } else {
            lines.push(fit_line(Line::default(), width));
        }
    }
    lines
}

struct TabSlot {
    x: usize,
    width: usize,
    index: usize,
    title: String,
    connected: bool,
    active: bool,
}

fn live_width(connected: bool) -> usize {
    if connected {
        UnicodeWidthStr::width(LIVE_MARK)
    } else {
        0
    }
}

fn tab_content_width(index: usize, title: &str, connected: bool) -> usize {
    index.to_string().width() + 1 + title.width() + live_width(connected)
}

fn tab_outer_width(index: usize, title: &str, connected: bool) -> usize {
    tab_content_width(index, title, connected)
        .saturating_add(TAB_SIDE_PAD.saturating_mul(2))
        .max(TAB_MIN_WIDTH)
}

fn clip_title_to_outer(index: usize, title: &str, connected: bool, max_outer: usize) -> String {
    let mut title = clip_line(title.trim(), TAB_TITLE_MAX);
    while title.width() > 0 && tab_outer_width(index, &title, connected) > max_outer {
        title = clip_line(&title, title.width().saturating_sub(1));
    }
    title
}

fn layout_step(count: usize) -> usize {
    if count == 0 {
        0
    } else {
        count.saturating_sub(1).saturating_mul(TAB_GAP)
    }
}

fn layout_tabs(tabs: &[TabLabel], active: u64, width: usize) -> Vec<TabSlot> {
    if width == 0 || tabs.is_empty() {
        return Vec::new();
    }
    let budget = width.saturating_sub(PLUS_SLOT).max(3);
    let mut titles: Vec<String> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| clip_title_to_outer(i + 1, &tab.title, tab.connected, budget))
        .collect();
    loop {
        let total: usize = titles
            .iter()
            .enumerate()
            .map(|(i, title)| tab_outer_width(i + 1, title, tabs[i].connected))
            .sum::<usize>()
            .saturating_add(layout_step(titles.len()));
        if total <= budget || titles.iter().all(|title| title.width() <= 1) {
            break;
        }
        let longest = titles
            .iter()
            .enumerate()
            .max_by_key(|(_, title)| title.width())
            .map(|(i, _)| i);
        let Some(i) = longest else {
            break;
        };
        titles[i] = clip_line(&titles[i], titles[i].width().saturating_sub(1));
    }
    let mut slots = Vec::with_capacity(tabs.len());
    let mut x = 0;
    for (i, tab) in tabs.iter().enumerate() {
        let index = i + 1;
        let mut title = titles[i].clone();
        let mut outer = tab_outer_width(index, &title, tab.connected);
        if x > 0 {
            x += TAB_GAP;
        }
        if x + outer > budget {
            if tab.id == active && x < budget {
                title = clip_title_to_outer(index, &title, tab.connected, budget.saturating_sub(x));
                outer = tab_outer_width(index, &title, tab.connected).min(budget.saturating_sub(x));
            } else if tab.id != active {
                x = x.saturating_sub(TAB_GAP);
                continue;
            } else {
                break;
            }
        }
        if outer < 3 {
            continue;
        }
        slots.push(TabSlot {
            x,
            width: outer,
            index,
            title,
            connected: tab.connected,
            active: tab.id == active,
        });
        x += outer;
    }
    slots
}

fn slot_fill(slot: &TabSlot, styles: &Styles) -> ratatui::style::Color {
    if slot.active {
        styles.selection
    } else {
        styles.band
    }
}

fn pad_bg(width: usize, bg: ratatui::style::Color) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    vec![Span::styled(" ".repeat(width), Style::default().bg(bg))]
}

fn pad_void(width: usize, styles: &Styles) -> Vec<Span<'static>> {
    pad_bg(width, styles.void)
}

fn slot_cap(slot: &TabSlot, styles: &Styles, cap: char) -> Vec<Span<'static>> {
    vec![Span::styled(
        cap.to_string().repeat(slot.width),
        Style::default().fg(slot_fill(slot, styles)).bg(styles.void),
    )]
}

fn slot_label(slot: &TabSlot, styles: &Styles) -> Vec<Span<'static>> {
    let bg = slot_fill(slot, styles);
    let title_style = if slot.active {
        styles.focus
    } else if slot.connected {
        styles.text
    } else {
        styles.muted
    };
    let index_style = if slot.active {
        styles.key
    } else {
        styles.quiet
    };
    let mut inner = vec![
        Span::styled(slot.index.to_string(), index_style.bg(bg)),
        Span::styled(" ", Style::default().bg(bg)),
        Span::styled(slot.title.clone(), title_style.bg(bg)),
    ];
    if slot.connected {
        inner.push(Span::styled(LIVE_MARK, styles.signal.bg(bg)));
    }
    let used = line_width(&Line::from(inner.clone()));
    let extra = slot.width.saturating_sub(used);
    let left = extra / 2;
    let right = extra.saturating_sub(left);
    let mut spans = pad_bg(left, bg);
    spans.extend(inner);
    spans.extend(pad_bg(right, bg));
    spans
}

fn gap_spans(styles: &Styles) -> Vec<Span<'static>> {
    pad_void(TAB_GAP, styles)
}

/// Session tiles: filled, vertically centered labels, side padding, one-cell gaps.
#[must_use]
pub fn tab_bar(
    tabs: &[TabLabel],
    active: u64,
    width: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let slots = layout_tabs(tabs, active, width);
    let caps = |slots: &[TabSlot], cap: char| {
        let mut spans = Vec::new();
        for (i, slot) in slots.iter().enumerate() {
            if i > 0 {
                spans.extend(gap_spans(styles));
            }
            spans.extend(slot_cap(slot, styles, cap));
        }
        spans
    };
    let mut top = caps(&slots, TAB_CAP_TOP);
    let mut bot = caps(&slots, TAB_CAP_BOT);
    let mut mid = Vec::new();
    for (i, slot) in slots.iter().enumerate() {
        if i > 0 {
            mid.extend(gap_spans(styles));
        }
        mid.extend(slot_label(slot, styles));
    }
    let used = slots.last().map_or(0, |slot| slot.x + slot.width);
    let rest = width.saturating_sub(used);
    if rest > 0 {
        top.extend(pad_void(rest, styles));
        bot.extend(pad_void(rest, styles));
        let mut rest_mid = pad_void(rest, styles);
        if rest >= PLUS_SLOT {
            rest_mid = vec![
                Span::styled(" ", Style::default().bg(styles.void)),
                Span::styled("+", styles.key.bg(styles.void)),
            ];
            if rest > PLUS_SLOT {
                rest_mid.push(Span::styled(
                    " ".repeat(rest.saturating_sub(PLUS_SLOT)),
                    Style::default().bg(styles.void),
                ));
            }
        }
        mid.extend(rest_mid);
    }
    vec![
        fit_line(Line::from(top), width),
        fit_line(Line::from(mid), width),
        fit_line(Line::from(bot), width),
    ]
}

/// Paint session tiles into `area`. Each fill is isolated to that tab's rect.
pub fn render_tab_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    tabs: &[TabLabel],
    active: u64,
    styles: &Styles,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    fill_rect(frame, area, styles.void);
    let width = usize::from(area.width);
    let slots = layout_tabs(tabs, active, width);
    let fill_y = if area.height >= 3 {
        area.y.saturating_add(area.height.saturating_sub(1) / 2)
    } else {
        area.y
    };
    let fill_h = if area.height >= 3 { 1 } else { area.height };
    for slot in &slots {
        let Ok(x) = u16::try_from(slot.x) else {
            continue;
        };
        let Ok(tab_w) = u16::try_from(slot.width) else {
            continue;
        };
        if x >= area.width {
            continue;
        }
        let width = tab_w.min(area.width.saturating_sub(x));
        fill_rect(
            frame,
            Rect {
                x: area.x.saturating_add(x),
                y: fill_y,
                width,
                height: fill_h,
            },
            slot_fill(slot, styles),
        );
    }
    let lines = tab_bar(tabs, active, width, styles);
    let shown = match area.height {
        1 => lines.get(1).into_iter().collect::<Vec<_>>(),
        2 => lines.iter().take(2).collect(),
        _ => lines.iter().take(3).collect(),
    };
    for (i, line) in shown.into_iter().enumerate() {
        let Ok(row) = u16::try_from(i) else {
            break;
        };
        if row >= area.height {
            break;
        }
        frame.render_widget(
            Paragraph::new(line.clone()),
            Rect {
                x: area.x,
                y: area.y.saturating_add(row),
                width: area.width,
                height: 1,
            },
        );
    }
}

/// Hints on the left, optional status clipped to the right.
///
/// Two-cell gutters match [`session_header`] so the footer lines up with the
/// identity and metrics row.
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
    let inner = width.saturating_sub(CHROME_GUTTER.saturating_mul(2));
    let hints_line = footer_hints(hints, styles);
    let content = if inner == 0 {
        Line::default()
    } else {
        let status = status.trim();
        if status.is_empty() {
            fit_line(hints_line, inner)
        } else {
            let hint_w = line_width(&hints_line);
            if hint_w + 2 >= inner {
                fit_line(hints_line, inner)
            } else {
                let rest = inner.saturating_sub(hint_w);
                let clipped = clip_line(status, rest.saturating_sub(1));
                let pad = rest.saturating_sub(clipped.width()).max(1);
                let mut spans = hints_line.spans;
                spans.push(Span::raw(" ".repeat(pad)));
                spans.push(Span::styled(clipped, styles.muted));
                fit_line(Line::from(spans), inner)
            }
        }
    };
    let mut spans = vec![Span::raw(" ".repeat(CHROME_GUTTER))];
    spans.extend(content.spans);
    spans.push(Span::raw(" ".repeat(CHROME_GUTTER)));
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
            "routeros-tui",
            "CCR2004 · 192.0.2.1",
            &[
                Signal::new("CPU", "18%", SignalLevel::Good),
                Signal::new("MEM", "41%", SignalLevel::Good),
                Signal::new("WAN", "84.2 Mb/s", SignalLevel::Good),
            ],
            &[],
            80,
            &styles,
            false,
        );
        let plain = line_plain(&line);
        assert!(plain.contains("routeros-tui"));
        assert!(plain.contains("CCR2004 · 192.0.2.1"));
        assert!(plain.contains("CPU 18%"));
        assert!(plain.contains("WAN 84.2 Mb/s"));
        assert_eq!(line_width(&line), 80);
        let product = plain.find("routeros-tui").expect("product");
        let wan = plain.find("WAN").expect("wan");
        assert!(product < wan);
        assert!(!plain.contains('●'));
    }

    #[test]
    fn session_header_keeps_trailing_safe_mode_on_the_right() {
        let styles = styles();
        let line = session_header(
            "routeros-tui",
            "CCR2004 · 192.0.2.1",
            &[
                Signal::new("CPU", "18%", SignalLevel::Good),
                Signal::new("WAN", "84.2 Mb/s", SignalLevel::Good),
            ],
            &[Signal::new(
                "SAFE",
                "ON - changes unroll if this tab drops",
                SignalLevel::Warning,
            )],
            120,
            &styles,
            false,
        );
        let plain = line_plain(&line);
        let wan = plain.find("WAN").expect("wan");
        let safe = plain.find("SAFE ON -").expect("safe");
        assert!(wan < safe, "{plain}");
        assert!(
            plain.contains("changes unroll if this tab drops"),
            "{plain}"
        );
        assert_eq!(line_width(&line), 120);
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
            "routeros-tui",
            "CCR2004 · 192.0.2.1",
            &metrics,
            &[],
            80,
            &styles,
            false,
        );
        let busy = session_header(
            "routeros-tui",
            "CCR2004 · 192.0.2.1",
            &metrics,
            &[],
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
        assert!(plain.starts_with("  enter edit"), "{plain:?}");
        assert!(plain.contains("resource loaded interfaces"), "{plain:?}");
        assert!(plain.ends_with("  "), "{plain:?}");
        assert_eq!(line_width(&line), 60);
    }

    #[test]
    fn footer_bar_keeps_side_gutters_on_a_narrow_row() {
        let styles = styles();
        let line = footer_bar("on", &[("q", "quit")], 16, &styles);
        let plain = line_plain(&line);
        assert!(plain.starts_with("  "), "{plain:?}");
        assert!(plain.ends_with("  "), "{plain:?}");
        assert_eq!(line_width(&line), 16);
    }

    fn tab_lines(tabs: &[TabLabel], active: u64, width: usize) -> (Vec<Line<'static>>, String) {
        let lines = tab_bar(tabs, active, width, &styles());
        let plain = lines.iter().map(line_plain).collect::<Vec<_>>().join("\n");
        (lines, plain)
    }

    #[test]
    fn tab_bar_respects_width() {
        let tabs = [
            TabLabel::new(10, "edge-office", true),
            TabLabel::new(20, "core", true),
        ];
        let (lines, _) = tab_lines(&tabs, 10, 40);
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert_eq!(line_width(line), 40);
        }
    }

    #[test]
    fn tab_bar_marks_active_tab() {
        let tabs = [
            TabLabel::new(10, "Login", false),
            TabLabel::new(20, "core", true),
        ];
        let (lines, plain) = tab_lines(&tabs, 20, 48);
        assert!(!plain.contains("Sessions:"), "{plain}");
        assert!(!plain.contains('┌'), "{plain}");
        assert!(!plain.contains('─'), "{plain}");
        assert!(plain.contains("core"), "{plain}");
        assert!(plain.contains("Login"), "{plain}");
        assert!(plain.contains('●'), "{plain}");
        let active = lines[1]
            .spans
            .iter()
            .find(|span| span.content == "core")
            .expect("active title");
        assert!(active.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn tab_bar_clips_narrow_width_without_panic() {
        let tabs = [TabLabel::new(1, "very-long-profile-name", true)];
        let empty = tab_bar(&tabs, 1, 0, &styles());
        assert!(empty.is_empty());
        let tiny = tab_bar(&tabs, 1, 3, &styles());
        for line in &tiny {
            assert_eq!(line_width(line), 3);
        }
        let none = tab_bar(&[], 99, 8, &styles());
        assert_eq!(none.len(), 3);
        assert_eq!(line_width(&none[0]), 8);
    }

    #[test]
    fn tab_bar_shows_both_tabs_when_width_allows() {
        let tabs = [
            TabLabel::new(1, "alpha", true),
            TabLabel::new(2, "beta", false),
        ];
        let (lines, plain) = tab_lines(&tabs, 1, 48);
        assert!(plain.contains("alpha"), "{plain}");
        assert!(plain.contains("beta"), "{plain}");
        assert!(plain.contains('+'), "{plain}");
        let slots = layout_tabs(&tabs, 1, 48);
        assert_eq!(slots.len(), 2);
        assert!(slots[0].width >= TAB_MIN_WIDTH, "{}", slots[0].width);
        assert_eq!(slots[1].x, slots[0].width + TAB_GAP);
        let mid = line_plain(&lines[1]);
        let alpha_at = mid.find("alpha").expect("alpha");
        assert!(alpha_at > slots[0].x, "title should be centered: {mid}");
        assert!(plain.contains('●'), "{plain}");
        let slot: String = mid.chars().skip(slots[0].x).take(slots[0].width).collect();
        assert!(slot.starts_with(' '), "{slot:?}");
        assert!(slot.ends_with(' '), "{slot:?}");
        assert!(slot.contains('●'), "{slot:?}");
        assert!(!line_plain(&lines[0]).contains("alpha"));
        assert!(!line_plain(&lines[2]).contains("alpha"));
        assert!(line_plain(&lines[0]).contains(TAB_CAP_TOP), "{plain}");
        assert!(line_plain(&lines[2]).contains(TAB_CAP_BOT), "{plain}");
    }

    #[test]
    fn live_tab_keeps_side_pad_and_dot() {
        let tabs = [TabLabel::new(1, "very-long-profile-name", true)];
        let (lines, _) = tab_lines(&tabs, 1, 80);
        let slots = layout_tabs(&tabs, 1, 80);
        assert_eq!(slots.len(), 1);
        let mid = line_plain(&lines[1]);
        let slot: String = mid.chars().skip(slots[0].x).take(slots[0].width).collect();
        assert!(slot.starts_with(' '), "{slot:?}");
        assert!(slot.ends_with(' '), "{slot:?}");
        assert!(slot.contains('●'), "{slot:?}");
        assert!(slot.contains("very-long-profile"), "{slot:?}");
    }

    #[test]
    fn tab_fill_stays_inside_each_tab() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let styles = styles();
        let tabs = [
            TabLabel::new(1, "alpha", true),
            TabLabel::new(2, "beta", false),
        ];
        let backend = TestBackend::new(36, 3);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_tab_bar(frame, frame.area(), &tabs, 1, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let slots = layout_tabs(&tabs, 1, 36);
        assert_eq!(slots.len(), 2);
        for y in 0..3_u16 {
            for x in 0..36_u16 {
                let cell = &buf[(x, y)];
                let in_active = x >= u16::try_from(slots[0].x).unwrap()
                    && x < u16::try_from(slots[0].x + slots[0].width).unwrap();
                let in_idle = x >= u16::try_from(slots[1].x).unwrap()
                    && x < u16::try_from(slots[1].x + slots[1].width).unwrap();
                if y == 1 && in_active {
                    assert_eq!(
                        cell.bg, styles.selection,
                        "active fill missing at ({x},{y})"
                    );
                } else if y == 1 && in_idle {
                    assert_eq!(cell.bg, styles.band, "idle fill missing at ({x},{y})");
                } else {
                    assert_eq!(cell.bg, styles.void, "fill escaped to ({x},{y})");
                }
            }
        }
    }
}
