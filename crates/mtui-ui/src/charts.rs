//! Braille charts for the live dashboard (Go `TrafficChart` / `BrailleSparkline`).

use std::time::Duration;

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::layout::fit_cell;
use crate::styles::Styles;

const BRAILLE_BASE: u32 = 0x2800;

/// Receive and transmit rates in bits per second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrafficSample {
    pub rx: f64,
    pub tx: f64,
}

/// Dual-series Braille traffic chart with a rate axis and time axis.
#[derive(Debug, Clone)]
pub struct TrafficChart<'a> {
    pub samples: &'a [TrafficSample],
    pub width: usize,
    pub height: usize,
    pub sample_interval: Duration,
}

impl TrafficChart<'_> {
    #[must_use]
    pub fn lines(&self, styles: &Styles) -> Vec<Line<'static>> {
        let width = self.width.max(1);
        let height = self.height.max(1);
        let label_width = (width / 3).clamp(5, 10);
        let plot_width = width.saturating_sub(label_width + 3).max(1);
        let plot_height = height.saturating_sub(1).max(1);
        let mut samples = self.samples;
        if samples.len() > plot_width {
            samples = &samples[samples.len() - plot_width..];
        }
        let mut peak = 1.0_f64;
        for sample in samples {
            peak = peak.max(sample.rx).max(sample.tx);
        }
        peak *= 1.08;
        let peak_label = if self.samples.is_empty() {
            "—".to_string()
        } else {
            format_traffic_rate(peak)
        };

        let logical_width = plot_width.saturating_mul(2);
        let logical_height = plot_height.saturating_mul(4);
        let mut cells = vec![vec![TrafficCell::default(); plot_width]; plot_height];
        plot_series(
            &mut cells,
            samples,
            logical_width,
            logical_height,
            peak,
            true,
        );
        plot_series(
            &mut cells,
            samples,
            logical_width,
            logical_height,
            peak,
            false,
        );

        let mut lines = Vec::with_capacity(height);
        for (row, row_cells) in cells.iter().enumerate() {
            let mut plot = Vec::with_capacity(plot_width);
            for cell in row_cells {
                let bits = cell.rx | cell.tx;
                if bits == 0 {
                    plot.push(Span::raw(" "));
                    continue;
                }
                let ch = char::from_u32(BRAILLE_BASE + u32::from(bits)).unwrap_or(' ');
                let style = match (cell.rx != 0, cell.tx != 0) {
                    (true, true) => styles.alert,
                    (true, false) => styles.signal,
                    (false, true) => styles.focus,
                    (false, false) => styles.muted,
                };
                plot.push(Span::styled(ch.to_string(), style));
            }
            let label = if row == 0 {
                peak_label.as_str()
            } else if row + 1 == cells.len() {
                "0 bps"
            } else {
                ""
            };
            let mut spans = vec![
                Span::styled(fit_cell(label, label_width), styles.muted),
                Span::raw(" "),
                Span::styled("│", styles.muted),
                Span::raw(" "),
            ];
            spans.extend(plot);
            lines.push(Line::from(spans));
        }

        let interval = if self.sample_interval.is_zero() {
            Duration::from_secs(2)
        } else {
            self.sample_interval
        };
        let window =
            interval.saturating_mul(u32::try_from(plot_width.saturating_sub(1)).unwrap_or(0));
        let axis = traffic_time_axis(plot_width, window);
        let mut axis_spans = vec![Span::raw(" ".repeat(label_width + 1))];
        axis_spans.push(Span::styled(format!("└ {axis}"), styles.muted));
        lines.push(Line::from(axis_spans));
        lines
    }
}

/// Right-aligned single-series Braille sparkline.
#[derive(Debug, Clone)]
pub struct BrailleSparkline<'a> {
    pub samples: &'a [f64],
    pub width: usize,
    pub height: usize,
    pub min: f64,
    pub max: f64,
    pub style: Style,
}

impl BrailleSparkline<'_> {
    #[must_use]
    pub fn lines(&self) -> Vec<Line<'static>> {
        let width = self.width.max(1);
        let height = self.height.max(1);
        if self.samples.is_empty() {
            return vec![Line::from(" ".repeat(width)); height];
        }
        let mut samples = self.samples;
        if samples.len() > width {
            samples = &samples[samples.len() - width..];
        }
        let (minimum, maximum) = if self.max <= self.min {
            let mut maximum = 1.0_f64;
            for value in samples {
                maximum = maximum.max(*value);
            }
            (0.0, maximum)
        } else {
            (self.min, self.max)
        };
        let logical_width = width.saturating_mul(2);
        let logical_height = height.saturating_mul(4);
        let mut cells = vec![vec![0_u8; width]; height];
        let start_x = logical_width
            .saturating_sub(1)
            .saturating_sub(samples.len().saturating_sub(1).saturating_mul(2));
        let point = |index: usize| -> (i32, i32) {
            let x = (start_x + index.saturating_mul(2)).min(logical_width.saturating_sub(1));
            let span = (maximum - minimum).max(f64::EPSILON);
            let ratio = ((samples[index] - minimum) / span).clamp(0.0, 1.0);
            let y = plot_y(ratio, logical_height);
            (i32_coord(x), i32_coord(y))
        };
        let (mut x, mut y) = point(0);
        set_spark_dot(&mut cells, x, y);
        for index in 1..samples.len() {
            let (next_x, next_y) = point(index);
            draw_line(&mut cells, x, y, next_x, next_y, |row, col, bit| {
                row[col] |= bit;
            });
            x = next_x;
            y = next_y;
        }
        cells
            .into_iter()
            .map(|row| {
                let mut out = String::with_capacity(width);
                for bits in row {
                    if bits == 0 {
                        out.push(' ');
                    } else {
                        out.push(char::from_u32(BRAILLE_BASE + u32::from(bits)).unwrap_or(' '));
                    }
                }
                Line::from(Span::styled(out, self.style))
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TrafficCell {
    rx: u8,
    tx: u8,
}

fn plot_series(
    cells: &mut [Vec<TrafficCell>],
    samples: &[TrafficSample],
    width: usize,
    height: usize,
    peak: f64,
    receive: bool,
) {
    if samples.is_empty() || width == 0 || height == 0 {
        return;
    }
    let start_x = width
        .saturating_sub(1)
        .saturating_sub(samples.len().saturating_sub(1).saturating_mul(2));
    let value_at = |index: usize| {
        if receive {
            samples[index].rx
        } else {
            samples[index].tx
        }
    };
    let point = |index: usize| -> (i32, i32) {
        let x = (start_x + index.saturating_mul(2)).min(width.saturating_sub(1));
        let ratio = (value_at(index) / peak).clamp(0.0, 1.0);
        (i32_coord(x), i32_coord(plot_y(ratio, height)))
    };
    let (mut previous_x, mut previous_y) = point(0);
    set_traffic_dot(cells, previous_x, previous_y, receive);
    for index in 1..samples.len() {
        let (current_x, current_y) = point(index);
        draw_line(
            cells,
            previous_x,
            previous_y,
            current_x,
            current_y,
            |row, col, bit| {
                if receive {
                    row[col].rx |= bit;
                } else {
                    row[col].tx |= bit;
                }
            },
        );
        previous_x = current_x;
        previous_y = current_y;
    }
}

fn plot_y(ratio: f64, height: usize) -> usize {
    let last = height.saturating_sub(1);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let offset = (ratio * last as f64) as usize;
    last.saturating_sub(offset.min(last))
}

fn i32_coord(value: usize) -> i32 {
    i32::try_from(value.min(i32::MAX as usize)).unwrap_or(i32::MAX)
}

fn draw_line<T>(
    cells: &mut [Vec<T>],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    mut set_bit: impl FnMut(&mut Vec<T>, usize, u8),
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        set_dot(cells, x0, y0, &mut set_bit);
        if x0 == x1 && y0 == y1 {
            return;
        }
        let double = 2 * err;
        if double >= dy {
            err += dy;
            x0 += step_x;
        }
        if double <= dx {
            err += dx;
            y0 += step_y;
        }
    }
}

fn set_traffic_dot(cells: &mut [Vec<TrafficCell>], x: i32, y: i32, receive: bool) {
    set_dot(cells, x, y, |row, col, bit| {
        if receive {
            row[col].rx |= bit;
        } else {
            row[col].tx |= bit;
        }
    });
}

fn set_spark_dot(cells: &mut [Vec<u8>], x: i32, y: i32) {
    set_dot(cells, x, y, |row, col, bit| {
        row[col] |= bit;
    });
}

fn set_dot<T>(cells: &mut [Vec<T>], x: i32, y: i32, set_bit: impl FnOnce(&mut Vec<T>, usize, u8)) {
    if x < 0 || y < 0 || cells.is_empty() || cells[0].is_empty() {
        return;
    }
    let Ok(x) = usize::try_from(x) else {
        return;
    };
    let Ok(y) = usize::try_from(y) else {
        return;
    };
    let cell_x = x / 2;
    let cell_y = y / 4;
    if cell_y >= cells.len() || cell_x >= cells[cell_y].len() {
        return;
    }
    set_bit(&mut cells[cell_y], cell_x, braille_bit(x % 2, y % 4));
}

fn braille_bit(x: usize, y: usize) -> u8 {
    const BITS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    BITS[y.min(3)][x.min(1)]
}

fn traffic_time_axis(width: usize, window: Duration) -> String {
    if width == 0 {
        return String::new();
    }
    let left = format!("-{}", format_traffic_window(window));
    let right = "now";
    if width <= left.len() + right.len() + 1 {
        return fit_cell(right, width);
    }
    format!(
        "{left}{}{right}",
        "─".repeat(width - left.len() - right.len())
    )
}

fn format_traffic_window(window: Duration) -> String {
    let secs = window.as_secs();
    if secs < 60 {
        return format!("{secs}s");
    }
    let minutes = secs / 60;
    let seconds = secs % 60;
    if seconds == 0 {
        format!("{minutes}m")
    } else {
        format!("{minutes}m{seconds:02}s")
    }
}

/// Format a bits-per-second rate for the traffic chart axis (1 decimal Gbps).
#[must_use]
pub fn format_traffic_rate(bits_per_second: f64) -> String {
    if bits_per_second >= 1_000_000_000.0 {
        format!("{:.1} Gb/s", bits_per_second / 1_000_000_000.0)
    } else if bits_per_second >= 1_000_000.0 {
        format!("{:.1} Mb/s", bits_per_second / 1_000_000.0)
    } else if bits_per_second >= 1_000.0 {
        format!("{:.1} Kb/s", bits_per_second / 1_000.0)
    } else {
        format!("{bits_per_second:.0} b/s")
    }
}

/// Header-rate formatting (2 decimal Gb/s).
#[must_use]
pub fn format_rate(bits_per_second: f64) -> String {
    if bits_per_second >= 1_000_000_000.0 {
        format!("{:.2} Gb/s", bits_per_second / 1_000_000_000.0)
    } else if bits_per_second >= 1_000_000.0 {
        format!("{:.1} Mb/s", bits_per_second / 1_000_000.0)
    } else if bits_per_second >= 1_000.0 {
        format!("{:.1} Kb/s", bits_per_second / 1_000.0)
    } else {
        format!("{bits_per_second:.0} b/s")
    }
}

/// Binary byte totals for memory and firewall totals.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;
    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};

    fn styles() -> Styles {
        let theme = DefaultTheme::new();
        Styles::from_palette(theme.palette())
    }

    fn assert_bounded(lines: &[Line<'_>], width: usize, height: usize) {
        assert_eq!(lines.len(), height);
        for line in lines {
            assert_eq!(
                crate::layout::line_width(line),
                width,
                "{}",
                crate::layout::line_plain(line)
            );
            for span in &line.spans {
                assert!(span.style.bg.is_none());
            }
        }
    }

    fn has_braille(text: &str) -> bool {
        text.chars()
            .any(|ch| ('\u{2800}'..='\u{28FF}').contains(&ch))
    }

    #[test]
    fn traffic_chart_is_deterministic_and_bounded() {
        let styles = styles();
        let samples = [
            TrafficSample {
                rx: 1_000_000.0,
                tx: 500_000.0,
            },
            TrafficSample {
                rx: 8_000_000.0,
                tx: 2_000_000.0,
            },
            TrafficSample {
                rx: 3_000_000.0,
                tx: 7_000_000.0,
            },
            TrafficSample {
                rx: 10_000_000.0,
                tx: 4_000_000.0,
            },
        ];
        let chart = TrafficChart {
            samples: &samples,
            width: 30,
            height: 6,
            sample_interval: Duration::ZERO,
        };
        let first = chart.lines(&styles);
        let second = chart.lines(&styles);
        assert_eq!(
            crate::layout::lines_plain(&first),
            crate::layout::lines_plain(&second)
        );
        assert_bounded(&first, 30, 6);
        let plain = crate::layout::lines_plain(&first);
        assert!(has_braille(&plain), "{plain}");
        assert!(plain.contains("Mb/s"), "{plain}");
        assert!(plain.contains("0 bps"), "{plain}");
        assert!(plain.contains("-32s"), "{plain}");
        assert!(plain.contains("now"), "{plain}");
    }

    #[test]
    fn traffic_chart_right_aligns_sparse_recent_samples() {
        let styles = styles();
        let samples = [
            TrafficSample {
                rx: 1_000_000.0,
                tx: 0.0,
            },
            TrafficSample {
                rx: 8_000_000.0,
                tx: 0.0,
            },
        ];
        let lines = TrafficChart {
            samples: &samples,
            width: 40,
            height: 5,
            sample_interval: Duration::from_secs(2),
        }
        .lines(&styles);
        for line in lines.iter().take(4) {
            let plain = crate::layout::line_plain(line);
            let Some(separator) = plain.find("│ ") else {
                panic!("plot row has no axis: {plain}");
            };
            let plot: Vec<char> = plain[separator + "│ ".len()..].chars().collect();
            let keep = plot.len().saturating_sub(4);
            for ch in plot.iter().take(keep) {
                assert!(
                    !('\u{2800}'..='\u{28FF}').contains(ch),
                    "sparse samples were stretched across history: {plain}"
                );
            }
        }
    }

    #[test]
    fn braille_sparkline_is_bounded_and_right_aligned() {
        let spark = BrailleSparkline {
            samples: &[10.0, 40.0, 20.0, 90.0],
            width: 20,
            height: 2,
            min: 0.0,
            max: 100.0,
            style: styles().signal,
        }
        .lines();
        assert_bounded(&spark, 20, 2);
        let plain = crate::layout::lines_plain(&spark);
        let first_braille = plain
            .chars()
            .position(|ch| ('\u{2800}'..='\u{28FF}').contains(&ch));
        assert!(first_braille.is_some_and(|idx| idx >= 12), "{plain}");
    }
}
