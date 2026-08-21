//! Firewall hit heatmap table (Go `FirewallHitChart`).

use mtui_core::Palette;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::charts::BrailleSparkline;
use crate::layout::{constrain_lines, fit_cell, fit_line};
use crate::styles::{Styles, rgb_color};

/// Maximum rule rows a dashboard pane shows.
pub const MAX_FIREWALL_RULES: usize = 10;

/// Read-only rule counters and recent packet deltas.
#[derive(Debug, Clone, PartialEq)]
pub struct FirewallRuleMetric {
    pub id: String,
    pub label: String,
    pub action: String,
    pub packets: u64,
    pub bytes: u64,
    pub recent_packets: u64,
    pub recent_bytes: u64,
    pub history: Vec<f64>,
}

/// Ranked, scrollable firewall hit histories.
#[derive(Debug, Clone)]
pub struct FirewallHitChart<'a> {
    pub rules: &'a [FirewallRuleMetric],
    pub width: usize,
    pub height: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy)]
struct FirewallColumns {
    heat: usize,
    rule: usize,
    action: usize,
    spark: usize,
    now: usize,
    total: usize,
    compact: bool,
}

impl FirewallHitChart<'_> {
    #[must_use]
    pub fn visible_rows(&self) -> usize {
        MAX_FIREWALL_RULES.min(self.height.saturating_sub(1))
    }

    #[must_use]
    pub fn max_offset(&self) -> usize {
        self.rules.len().saturating_sub(self.visible_rows())
    }

    #[must_use]
    pub fn lines(&self, styles: &Styles, palette: &Palette) -> Vec<Line<'static>> {
        let width = self.width.max(1);
        let height = self.height.max(1);
        if self.rules.is_empty() {
            return empty_state(
                "No firewall counters",
                "Filter rules will appear after telemetry loads",
                width,
                height,
                styles,
            );
        }
        let ranked = ranked_firewall_rules(self.rules);
        let visible = self.visible_rows();
        let offset = self.offset.min(ranked.len().saturating_sub(visible));
        let end = ranked.len().min(offset + visible);
        let window = &ranked[offset..end];
        let mut peak = 1.0_f64;
        for rule in &ranked {
            peak = peak.max(latest_hit(rule));
        }
        let cols = firewall_column_layout(width, &ranked);
        let mut lines = Vec::with_capacity(height);
        lines.push(header(&cols, width, offset, visible, ranked.len(), styles));
        for rule in window {
            lines.push(row(rule, &cols, peak, styles, palette));
        }
        constrain_lines(lines, width, height)
    }
}

fn header(
    cols: &FirewallColumns,
    width: usize,
    offset: usize,
    visible: usize,
    total: usize,
    styles: &Styles,
) -> Line<'static> {
    let mut cells = vec![fit_cell("HEAT", cols.heat), fit_cell("RULE", cols.rule)];
    if !cols.compact {
        cells.push(fit_cell("ACTION", cols.action));
    }
    let history = if total > visible {
        let start = offset + 1;
        let end = total.min(offset + visible);
        format!("HIT HISTORY {start}-{end}/{total}")
    } else {
        "HIT HISTORY".to_string()
    };
    cells.push(fit_cell(&history, cols.spark));
    cells.push(fit_cell("NOW", cols.now));
    if !cols.compact {
        cells.push(fit_cell("TOTAL", cols.total));
    }
    fit_line(
        Line::from(Span::styled(cells.join(" "), styles.muted)),
        width,
    )
}

fn row(
    rule: &FirewallRuleMetric,
    cols: &FirewallColumns,
    peak: f64,
    styles: &Styles,
    palette: &Palette,
) -> Line<'static> {
    let current = latest_hit(rule);
    let style = firewall_heat_style(current, peak, rule.packets, palette);
    let state = heat_label(current, peak, rule.packets);
    let spark = BrailleSparkline {
        samples: &rule.history,
        width: cols.spark,
        height: 1,
        min: 0.0,
        max: max_history(&rule.history),
        style,
    }
    .lines();
    let mut now = format!("+{}", format_count(rule.recent_packets));
    if !cols.compact {
        now.push_str(" pkt");
    }
    let mut spans = vec![
        Span::styled(fit_cell(state, cols.heat), style),
        Span::raw(" "),
        Span::styled(fit_cell(&rule.label, cols.rule), styles.text),
    ];
    if !cols.compact {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            fit_cell(&rule.action, cols.action),
            styles.muted,
        ));
    }
    spans.push(Span::raw(" "));
    if let Some(spark_line) = spark.into_iter().next() {
        spans.extend(spark_line.spans);
    }
    spans.push(Span::raw(" "));
    spans.push(Span::styled(fit_cell(&now, cols.now), style));
    if !cols.compact {
        let total = format!(
            "{} / {}",
            format_count(rule.packets),
            format_metric_bytes(rule.bytes)
        );
        spans.push(Span::raw(" "));
        spans.push(Span::styled(fit_cell(&total, cols.total), styles.muted));
    }
    Line::from(spans)
}

fn empty_state(
    title: &str,
    hint: &str,
    width: usize,
    height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let title = center_text(title, width);
    let hint = center_text(hint, width);
    constrain_lines(
        vec![
            Line::from(Span::styled(
                title,
                styles.text.add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(hint, styles.muted)),
        ],
        width,
        height,
    )
}

fn center_text(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let runes: Vec<char> = value.chars().collect();
    if runes.len() >= width {
        return runes.into_iter().take(width).collect();
    }
    let pad = (width - runes.len()) / 2;
    format!(
        "{}{value}{}",
        " ".repeat(pad),
        " ".repeat(width - pad - runes.len())
    )
}

fn ranked_firewall_rules(rules: &[FirewallRuleMetric]) -> Vec<FirewallRuleMetric> {
    let mut ranked = rules.to_vec();
    ranked.sort_by(|left, right| {
        latest_hit(right)
            .total_cmp(&latest_hit(left))
            .then_with(|| right.packets.cmp(&left.packets))
            .then_with(|| left.id.cmp(&right.id))
    });
    ranked
}

fn firewall_column_layout(width: usize, rules: &[FirewallRuleMetric]) -> FirewallColumns {
    if width < 80 {
        let heat = 5;
        let now = 6;
        let gaps = 3;
        let flex = width.saturating_sub(heat + now + gaps).max(8);
        let rule = (flex / 2).clamp(8, flex.saturating_sub(4).max(8));
        let spark = flex.saturating_sub(rule).max(4);
        return FirewallColumns {
            heat,
            rule,
            action: 0,
            spark,
            now,
            total: 0,
            compact: true,
        };
    }
    let heat = 7;
    let now = 10;
    let total = 14;
    let mut action = "ACTION".len();
    let mut longest_rule = "RULE".len();
    for rule in rules {
        action = action.max(rule.action.chars().count());
        longest_rule = longest_rule.max(rule.label.chars().count());
    }
    action = action.clamp(6, 12);
    let gaps = 5;
    let flex = width
        .saturating_sub(heat + action + now + total + gaps)
        .max(16);
    let min_spark = (flex / 4).max(8);
    let mut rule = longest_rule.min(flex.saturating_sub(min_spark)).max(12);
    if rule > flex.saturating_sub(min_spark) {
        rule = flex.saturating_sub(min_spark).max(8);
    }
    let spark = flex.saturating_sub(rule).max(min_spark);
    FirewallColumns {
        heat,
        rule,
        action,
        spark,
        now,
        total,
        compact: false,
    }
}

fn latest_hit(rule: &FirewallRuleMetric) -> f64 {
    rule.history.last().copied().unwrap_or(0.0)
}

fn max_history(history: &[f64]) -> f64 {
    history.iter().copied().fold(1.0_f64, f64::max)
}

fn heat_label(current: f64, peak: f64, total: u64) -> &'static str {
    if total == 0 {
        "○ DEAD"
    } else if current == 0.0 {
        "· COLD"
    } else if current / peak >= 0.66 {
        "● HOT"
    } else if current / peak >= 0.25 {
        "◉ WARM"
    } else {
        "• HIT"
    }
}

fn firewall_heat_style(current: f64, peak: f64, total: u64, palette: &Palette) -> Style {
    if total == 0 {
        return Style::default().fg(rgb_color(palette.muted));
    }
    let mut ratio = current / peak.max(1.0);
    if current == 0.0 {
        ratio = 0.15;
    }
    let color = palette.muted.blend(palette.alert, ratio);
    let mut style = Style::default().fg(rgb_color(color));
    if ratio >= 0.66 {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

fn format_count(value: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    if value >= 1_000_000_000 {
        format!("{:.1}G", value as f64 / 1_000_000_000.0)
    } else if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

fn format_metric_bytes(value: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    if value >= 1 << 30 {
        format!("{:.1}G", value as f64 / f64::from(1_u32 << 30))
    } else if value >= 1 << 20 {
        format!("{:.1}M", value as f64 / f64::from(1_u32 << 20))
    } else if value >= 1 << 10 {
        format!("{:.1}K", value as f64 / f64::from(1_u32 << 10))
    } else {
        format!("{value}B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};

    fn styles() -> (Styles, Palette) {
        let theme = DefaultTheme::new();
        (Styles::from_palette(theme.palette()), *theme.palette())
    }

    fn rule(
        id: &str,
        label: &str,
        action: &str,
        packets: u64,
        bytes: u64,
        recent: u64,
        history: &[f64],
    ) -> FirewallRuleMetric {
        FirewallRuleMetric {
            id: id.into(),
            label: label.into(),
            action: action.into(),
            packets,
            bytes,
            recent_packets: recent,
            recent_bytes: 0,
            history: history.to_vec(),
        }
    }

    fn skip_visual(value: &str, columns: usize) -> &str {
        value
            .char_indices()
            .nth(columns)
            .map_or("", |(idx, _)| &value[idx..])
    }

    #[test]
    fn shows_hot_and_dead_rules_without_background() {
        let (styles, palette) = styles();
        let rules = [
            rule(
                "*1",
                "established",
                "accept",
                1200,
                900_000,
                200,
                &[10.0, 30.0, 200.0],
            ),
            rule("*2", "dns", "accept", 50, 5000, 2, &[1.0, 0.0, 2.0]),
            rule("*3", "unused legacy", "drop", 0, 0, 0, &[0.0, 0.0, 0.0]),
            rule("*4", "scanner", "drop", 800, 64_000, 40, &[5.0, 20.0, 40.0]),
        ];
        let lines = FirewallHitChart {
            rules: &rules,
            width: 80,
            height: 5,
            offset: 0,
        }
        .lines(&styles, &palette);
        assert_eq!(lines.len(), 5);
        let plain = crate::layout::lines_plain(&lines);
        assert!(plain.contains("HOT"), "{plain}");
        assert!(plain.contains("DEAD"), "{plain}");
        assert!(plain.contains("unused legacy"), "{plain}");
        for line in &lines {
            assert!(crate::layout::line_width(line) <= 80);
            for span in &line.spans {
                assert!(span.style.bg.is_none());
            }
        }
    }

    #[test]
    fn aligns_columns_and_uses_available_width() {
        let (styles, palette) = styles();
        let rules = [
            rule(
                "*1",
                "special dummy rule to prevent accidental lockout",
                "passthrough",
                737_300_000,
                4_200_000_000,
                227,
                &[10.0, 227.0],
            ),
            rule(
                "*2",
                "Allow Related, Established",
                "accept",
                20,
                2000,
                2,
                &[2.0],
            ),
        ];
        let cols = firewall_column_layout(120, &rules);
        assert!(cols.rule >= 30, "{cols:?}");
        assert!(cols.action >= "passthrough".len(), "{cols:?}");
        let lines = FirewallHitChart {
            rules: &rules,
            width: 120,
            height: 4,
            offset: 0,
        }
        .lines(&styles, &palette);
        let header = crate::layout::line_plain(&lines[0]);
        let row = crate::layout::line_plain(&lines[1]);
        let now_at = header.find("NOW").expect("NOW");
        let total_at = header.find("TOTAL").expect("TOTAL");
        let now_rest = skip_visual(&row, header[..now_at].chars().count());
        let total_rest = skip_visual(&row, header[..total_at].chars().count());
        assert!(now_rest.contains("+227 pkt"), "header={header} row={row}");
        assert!(total_rest.contains("737.3M"), "header={header} row={row}");
        assert!(!row.contains("passthr…"), "{row}");
        assert!(row.contains("passthrough"), "{row}");
        assert!(!row.contains("special dummy rule to…"), "{row}");
    }

    #[test]
    fn scrolls_past_visible_window() {
        let (styles, palette) = styles();
        let rules: Vec<_> = (0..12)
            .map(|index| {
                let n = u32::try_from(12 - index).unwrap_or(1);
                rule(
                    &format!("*{}", index + 1),
                    &format!("rule-{:02}", index + 1),
                    "accept",
                    u64::from(n) * 10,
                    0,
                    u64::from(n),
                    &[f64::from(n)],
                )
            })
            .collect();
        let chart = FirewallHitChart {
            rules: &rules,
            width: 90,
            height: 4,
            offset: 0,
        };
        let first = crate::layout::lines_plain(&chart.lines(&styles, &palette));
        assert!(first.contains("rule-01"), "{first}");
        assert!(!first.contains("rule-12"), "{first}");
        let scrolled = FirewallHitChart {
            offset: chart.max_offset(),
            ..chart
        };
        let last = crate::layout::lines_plain(&scrolled.lines(&styles, &palette));
        assert!(last.contains("rule-12"), "{last}");
        assert!(!last.contains("rule-01"), "{last}");
    }
}
