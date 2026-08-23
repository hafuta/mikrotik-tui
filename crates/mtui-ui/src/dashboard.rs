//! Fixed-slot dashboard geometry and section assembly.

use std::time::Duration;

use mtui_core::Palette;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::charts::{
    BrailleSparkline, TrafficChart, TrafficSample, format_bytes, format_rate, percent_bar,
    percent_meter,
};
use crate::firewall::{FirewallHitChart, FirewallRuleMetric, MAX_FIREWALL_RULES};
use crate::layout::{constrain_lines, fit_cell, fit_line, join_horizontal};
use crate::styles::Styles;

/// Heights reserved from terminal size (and core count for CPU rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardGeometry {
    pub stacked: bool,
    pub compact: bool,
    pub cpu_height: usize,
    pub memory_height: usize,
    pub wan_height: usize,
    pub firewall_height: usize,
    pub metric_width_left: usize,
    pub metric_width_right: usize,
}

impl DashboardGeometry {
    /// Allocate section slots. Loading/empty/populated must share this geometry
    /// for a given `width`, `height`, and `cpu_core_count`.
    #[must_use]
    pub fn new(width: usize, height: usize, cpu_core_count: usize) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let mut geometry = Self {
            stacked: width < 72,
            compact: height < 10,
            cpu_height: 0,
            memory_height: 0,
            wan_height: 0,
            firewall_height: 0,
            metric_width_left: 0,
            metric_width_right: 0,
        };
        if geometry.compact {
            return geometry;
        }
        let mut cpu_rows = if cpu_core_count == 0 {
            4
        } else {
            cpu_core_count
        };
        cpu_rows = cpu_rows.clamp(1, 8);
        if geometry.stacked {
            let memory_rows = cpu_rows.min(2);
            let budget = sub_budget(height, 5 + cpu_rows + memory_rows);
            let (wan, firewall) = split_dashboard_budget(budget);
            geometry.cpu_height = cpu_rows;
            geometry.memory_height = memory_rows;
            geometry.wan_height = wan;
            geometry.firewall_height = firewall;
            return geometry;
        }
        let left = width.saturating_sub(2) / 2;
        geometry.metric_width_left = left;
        geometry.metric_width_right = width.saturating_sub(left + 2);
        geometry.cpu_height = cpu_rows;
        geometry.memory_height = cpu_rows;
        let budget = sub_budget(height, 4 + cpu_rows);
        let (wan, firewall) = split_dashboard_budget(budget);
        geometry.wan_height = wan;
        geometry.firewall_height = firewall;
        geometry
    }
}

fn sub_budget(height: usize, used: usize) -> isize {
    let height = isize::try_from(height).unwrap_or(isize::MAX);
    let used = isize::try_from(used).unwrap_or(isize::MAX);
    height.saturating_sub(used)
}

fn split_dashboard_budget(budget: isize) -> (usize, usize) {
    let budget = budget.max(2);
    let mut min_wan = (budget * 3 / 5).max(4);
    if min_wan > budget - 2 {
        min_wan = (budget - 2).max(1);
    }
    let firewall = (budget - min_wan)
        .max(2)
        .min(1 + isize::try_from(MAX_FIREWALL_RULES).unwrap_or(10));
    let wan = (budget - firewall).max(1);
    (
        usize::try_from(wan).unwrap_or(1),
        usize::try_from(firewall).unwrap_or(2),
    )
}

/// Snapshot of one CPU core for dashboard rendering.
#[derive(Debug, Clone, Copy)]
pub struct CpuCoreView<'a> {
    pub name: &'a str,
    pub load: f64,
    pub samples: &'a [f64],
}

/// Read-only dashboard inputs. Geometry does not depend on sample *values*.
#[derive(Debug, Clone, Copy)]
pub struct DashboardView<'a> {
    pub cpu_cores: &'a [CpuCoreView<'a>],
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_samples: &'a [f64],
    pub wan_interface: &'a str,
    pub traffic_has_base: bool,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub traffic_samples: &'a [TrafficSample],
    pub firewall_rules: &'a [FirewallRuleMetric],
    pub firewall_offset: usize,
}

/// Render the dashboard canvas into a fixed `width` × `height` slot.
#[must_use]
pub fn dashboard_content(
    width: usize,
    height: usize,
    view: &DashboardView<'_>,
    styles: &Styles,
    palette: &Palette,
) -> Vec<Line<'static>> {
    let width = width.max(1);
    let height = height.max(1);
    let geometry = DashboardGeometry::new(width, height, view.cpu_cores.len());
    if geometry.compact {
        return constrain_lines(compact_dashboard(width, view, styles), width, height);
    }
    let metrics = if geometry.stacked {
        let mut lines = dashboard_section(
            "CPU CORES",
            cpu_dashboard_view(width, geometry.cpu_height, view, styles),
            width,
            geometry.cpu_height,
            styles,
        );
        lines.extend(dashboard_section(
            "MEMORY",
            memory_dashboard_view(width, geometry.memory_height, view, styles),
            width,
            geometry.memory_height,
            styles,
        ));
        lines
    } else {
        join_horizontal(
            dashboard_section(
                "CPU CORES",
                cpu_dashboard_view(
                    geometry.metric_width_left,
                    geometry.cpu_height,
                    view,
                    styles,
                ),
                geometry.metric_width_left,
                geometry.cpu_height,
                styles,
            ),
            dashboard_section(
                "MEMORY",
                memory_dashboard_view(
                    geometry.metric_width_right,
                    geometry.memory_height,
                    view,
                    styles,
                ),
                geometry.metric_width_right,
                geometry.memory_height,
                styles,
            ),
            2,
        )
    };
    let mut out = metrics;
    out.push(fit_line(Line::default(), width));
    out.extend(dashboard_section(
        "WAN THROUGHPUT",
        wan_dashboard_view(width, geometry.wan_height, view, styles),
        width,
        geometry.wan_height,
        styles,
    ));
    out.extend(dashboard_section(
        "FIREWALL HIT HEAT",
        FirewallHitChart {
            rules: view.firewall_rules,
            width,
            height: geometry.firewall_height,
            offset: view.firewall_offset,
        }
        .lines(styles, palette),
        width,
        geometry.firewall_height,
        styles,
    ));
    constrain_lines(out, width, height)
}

fn dashboard_section(
    title: &str,
    content: Vec<Line<'static>>,
    width: usize,
    content_height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let fill = width.saturating_sub(title.len() + 1);
    let heading = Line::from(vec![
        Span::styled(title.to_string(), styles.focus),
        Span::raw(" "),
        Span::styled("─".repeat(fill), styles.muted),
    ]);
    let mut lines = vec![fit_line(heading, width)];
    lines.extend(constrain_lines(content, width, content_height.max(1)));
    lines
}

fn compact_dashboard(
    width: usize,
    view: &DashboardView<'_>,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let cpu = if view.cpu_cores.is_empty() {
        "CPU collecting".to_string()
    } else {
        let total: f64 = view.cpu_cores.iter().map(|core| core.load).sum();
        format!(
            "CPU {:.0}% avg",
            total / f64::from(u32::try_from(view.cpu_cores.len()).unwrap_or(1))
        )
    };
    let memory = if view.memory_total_bytes == 0 {
        "memory collecting".to_string()
    } else {
        format!(
            "memory {:.0}%",
            used_percent(view.memory_used_bytes, view.memory_total_bytes)
        )
    };
    let wan = if view.traffic_has_base {
        format!(
            "WAN ↓ {}  ↑ {}",
            format_rate(view.rx_rate),
            format_rate(view.tx_rate)
        )
    } else {
        "WAN collecting".to_string()
    };
    let firewall = format!("firewall {} enabled rules", view.firewall_rules.len());
    let mut lines = dashboard_section(
        "ROUTER TELEMETRY",
        vec![Line::from(Span::styled(
            format!("{cpu}  ·  {memory}"),
            styles.text,
        ))],
        width,
        1,
        styles,
    );
    lines.extend(dashboard_section(
        "THROUGHPUT",
        vec![Line::from(Span::styled(wan, styles.text))],
        width,
        1,
        styles,
    ));
    lines.extend(dashboard_section(
        "FIREWALL",
        vec![Line::from(Span::styled(firewall, styles.text))],
        width,
        1,
        styles,
    ));
    lines
}

fn cpu_dashboard_view(
    width: usize,
    height: usize,
    view: &DashboardView<'_>,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let height = height.max(1);
    if view.cpu_cores.is_empty() {
        let mut lines = vec![Line::from(Span::styled(
            "Collecting per-core load…",
            styles.muted,
        ))];
        for _ in 1..height {
            lines.extend(
                BrailleSparkline {
                    samples: &[],
                    width,
                    height: 1,
                    min: 0.0,
                    max: 100.0,
                    style: styles.muted,
                }
                .lines(),
            );
        }
        return constrain_lines(lines, width, height);
    }
    let mut lines = Vec::with_capacity(height);
    for core in view.cpu_cores {
        let (style, state) = cpu_state(core.load, styles);
        let label_width = (width / 5).clamp(4, 8);
        let value_width = 10;
        let spark_width = width.saturating_sub(label_width + value_width + 2).max(4);
        let bar = percent_meter(core.load, spark_width, style, styles.quiet);
        let value = format!("{load:3.0}% {state:<4}", load = core.load, state = state);
        let mut spans = vec![
            Span::styled(fit_cell(core.name, label_width), styles.text),
            Span::raw(" "),
        ];
        spans.extend(bar.spans);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(value, style));
        lines.push(Line::from(spans));
        if lines.len() >= height {
            break;
        }
    }
    while lines.len() < height {
        lines.push(Line::default());
    }
    constrain_lines(lines, width, height)
}

fn cpu_state(load: f64, styles: &Styles) -> (ratatui::style::Style, &'static str) {
    if load >= 85.0 {
        (styles.error, "HIGH")
    } else if load >= 60.0 {
        (styles.alert, "BUSY")
    } else {
        (styles.signal, "OK")
    }
}

fn memory_dashboard_view(
    width: usize,
    height: usize,
    view: &DashboardView<'_>,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let height = height.max(1);
    if view.memory_total_bytes == 0 {
        let mut lines = vec![Line::from(Span::styled(
            "Collecting memory pressure…",
            styles.muted,
        ))];
        for _ in 1..height {
            lines.extend(
                BrailleSparkline {
                    samples: &[],
                    width,
                    height: 1,
                    min: 0.0,
                    max: 100.0,
                    style: styles.muted,
                }
                .lines(),
            );
        }
        return constrain_lines(lines, width, height);
    }
    let percent = used_percent(view.memory_used_bytes, view.memory_total_bytes);
    let (base, state) = if percent >= 90.0 {
        (styles.error, "CRITICAL")
    } else if percent >= 75.0 {
        (styles.alert, "PRESSURE")
    } else {
        (styles.signal, "HEALTHY")
    };
    let summary = Line::from(vec![
        Span::styled(
            format!("{percent:.1}% {state}"),
            base.add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{} / {}",
                format_bytes(view.memory_used_bytes),
                format_bytes(view.memory_total_bytes)
            ),
            styles.muted,
        ),
    ]);
    if height <= 1 {
        return vec![summary];
    }
    let mut lines = vec![summary];
    if height > 1 {
        lines.push(percent_bar(percent, width, base, styles.quiet));
    }
    while lines.len() < height {
        lines.push(Line::default());
    }
    constrain_lines(lines, width, height)
}

#[allow(clippy::cast_precision_loss)]
fn used_percent(used: u64, total: u64) -> f64 {
    used as f64 * 100.0 / total as f64
}

fn wan_dashboard_view(
    width: usize,
    height: usize,
    view: &DashboardView<'_>,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let height = height.max(1);
    let identity = if view.traffic_has_base || !view.wan_interface.is_empty() {
        let name = if view.wan_interface.trim().is_empty() {
            "—"
        } else {
            view.wan_interface
        };
        Line::from(vec![
            Span::styled(name.to_string(), styles.text.add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("● LIVE", styles.signal),
            Span::raw("  "),
            Span::styled(
                format!("↓ {}", format_rate(view.rx_rate)),
                styles.signal.add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("↑ {}", format_rate(view.tx_rate)), styles.focus),
        ])
    } else {
        Line::from(Span::styled("Detecting WAN interface…", styles.muted))
    };
    if height <= 1 {
        return vec![identity];
    }
    let mut lines = vec![identity];
    lines.extend(
        TrafficChart {
            samples: view.traffic_samples,
            width,
            height: height.saturating_sub(1),
            sample_interval: Duration::from_secs(2),
        }
        .lines(styles),
    );
    constrain_lines(lines, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};

    fn styles() -> (Styles, Palette) {
        let theme = DefaultTheme::new();
        (Styles::from_palette(theme.palette()), *theme.palette())
    }

    fn empty_view<'a>(cores: &'a [CpuCoreView<'a>]) -> DashboardView<'a> {
        DashboardView {
            cpu_cores: cores,
            memory_used_bytes: 0,
            memory_total_bytes: 0,
            memory_samples: &[],
            wan_interface: "",
            traffic_has_base: false,
            rx_rate: 0.0,
            tx_rate: 0.0,
            traffic_samples: &[],
            firewall_rules: &[],
            firewall_offset: 0,
        }
    }

    #[test]
    fn loading_and_populated_share_canvas_size() {
        let (styles, palette) = styles();
        let cores = [
            CpuCoreView {
                name: "cpu0",
                load: 10.0,
                samples: &[10.0],
            },
            CpuCoreView {
                name: "cpu1",
                load: 20.0,
                samples: &[20.0],
            },
            CpuCoreView {
                name: "cpu2",
                load: 30.0,
                samples: &[30.0],
            },
            CpuCoreView {
                name: "cpu3",
                load: 40.0,
                samples: &[40.0],
            },
        ];
        let samples = [
            TrafficSample {
                rx: 1000.0,
                tx: 500.0,
            },
            TrafficSample {
                rx: 2000.0,
                tx: 800.0,
            },
        ];
        let rules = [FirewallRuleMetric {
            id: "*1".into(),
            label: "accept".into(),
            action: "accept".into(),
            packets: 10,
            bytes: 1000,
            recent_packets: 0,
            recent_bytes: 0,
            history: vec![0.0],
        }];
        for (width, height) in [(44, 8), (60, 16), (90, 20)] {
            let loading = dashboard_content(width, height, &empty_view(&[]), &styles, &palette);
            let loaded = dashboard_content(
                width,
                height,
                &DashboardView {
                    cpu_cores: &cores,
                    memory_used_bytes: 600,
                    memory_total_bytes: 1000,
                    memory_samples: &[60.0],
                    wan_interface: "ether1",
                    traffic_has_base: true,
                    rx_rate: 2000.0,
                    tx_rate: 800.0,
                    traffic_samples: &samples,
                    firewall_rules: &rules,
                    firewall_offset: 0,
                },
                &styles,
                &palette,
            );
            assert_eq!(loading.len(), height, "{width}x{height} loading");
            assert_eq!(loaded.len(), height, "{width}x{height} loaded");
            for line in loading.iter().chain(loaded.iter()) {
                assert!(crate::layout::line_width(line) <= width);
            }
        }
    }

    #[test]
    fn prefers_wan_height_and_caps_firewall() {
        let geometry = DashboardGeometry::new(100, 24, 4);
        assert_eq!(geometry.cpu_height, 4, "{geometry:?}");
        assert!(
            geometry.wan_height > geometry.cpu_height,
            "WAN chart should receive leftover height: {geometry:?}"
        );
        assert!(geometry.firewall_height <= 11, "{geometry:?}");
        assert!(!geometry.stacked);
        assert!(!geometry.compact);
    }

    #[test]
    fn cpu_core_bars_sit_on_adjacent_rows() {
        let (styles, palette) = styles();
        let cores = [
            CpuCoreView {
                name: "cpu0",
                load: 80.0,
                samples: &[80.0],
            },
            CpuCoreView {
                name: "cpu1",
                load: 80.0,
                samples: &[80.0],
            },
            CpuCoreView {
                name: "cpu2",
                load: 80.0,
                samples: &[80.0],
            },
            CpuCoreView {
                name: "cpu3",
                load: 80.0,
                samples: &[80.0],
            },
        ];
        let lines = dashboard_content(
            100,
            24,
            &DashboardView {
                cpu_cores: &cores,
                memory_used_bytes: 600,
                memory_total_bytes: 1000,
                memory_samples: &[60.0],
                wan_interface: "ether1",
                traffic_has_base: false,
                rx_rate: 0.0,
                tx_rate: 0.0,
                traffic_samples: &[],
                firewall_rules: &[],
                firewall_offset: 0,
            },
            &styles,
            &palette,
        );
        let cpu = lines
            .iter()
            .map(crate::layout::line_plain)
            .map(|line| line.chars().take(12).collect::<String>())
            .collect::<Vec<_>>();
        let start = cpu
            .iter()
            .position(|line| line.contains("cpu0"))
            .expect("cpu0");
        assert!(cpu[start].contains("cpu0"));
        assert!(cpu[start + 1].contains("cpu1"), "{cpu:?}");
        assert!(cpu[start + 2].contains("cpu2"), "{cpu:?}");
        assert!(cpu[start + 3].contains("cpu3"), "{cpu:?}");
    }

    #[test]
    fn stacks_cpu_and_memory_when_narrow() {
        let geometry = DashboardGeometry::new(60, 20, 4);
        assert!(geometry.stacked);
        assert_eq!(geometry.memory_height, 2);
    }
}
