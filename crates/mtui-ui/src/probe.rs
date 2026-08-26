//! Ping and traceroute overlays: filters plus a bounded result table.

use std::collections::HashMap;
use std::collections::VecDeque;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::login::is_printable_char;
use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::styles::Styles;

const SAMPLE_CAP: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeKind {
    Ping,
    Traceroute,
    BandwidthTest,
    FloodPing,
    MacScan,
    IpScan,
    Profiler,
    WifiScan,
}

impl ProbeKind {
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::Traceroute => "Traceroute",
            Self::BandwidthTest => "Bandwidth test",
            Self::FloodPing => "Flood ping",
            Self::MacScan => "MAC scan",
            Self::IpScan => "IP scan",
            Self::Profiler => "Profiler",
            Self::WifiScan => "WiFi scan",
        }
    }

    #[must_use]
    pub fn command(self) -> &'static str {
        match self {
            Self::Ping => "ping",
            Self::Traceroute => "traceroute",
            Self::BandwidthTest => "bandwidth-test",
            Self::FloodPing => "flood-ping",
            Self::MacScan => "mac-scan",
            Self::IpScan => "ip-scan",
            Self::Profiler => "profile",
            Self::WifiScan => "scan",
        }
    }

    #[must_use]
    pub fn endpoint(self) -> &'static str {
        match self {
            Self::WifiScan => "/interface/wifi",
            _ => "/tool",
        }
    }

    #[must_use]
    pub fn requires_address(self) -> bool {
        matches!(
            self,
            Self::Ping | Self::Traceroute | Self::BandwidthTest | Self::FloodPing | Self::IpScan
        )
    }

    #[must_use]
    pub fn requires_interface(self) -> bool {
        matches!(self, Self::MacScan | Self::WifiScan)
    }

    #[must_use]
    pub fn default_count(self) -> &'static str {
        match self {
            Self::Ping => "4",
            Self::Traceroute => "8",
            Self::BandwidthTest => "10",
            Self::FloodPing => "100",
            Self::Profiler => "5",
            Self::MacScan | Self::IpScan | Self::WifiScan => "",
        }
    }

    fn fields(self) -> &'static [ProbeField] {
        match self {
            Self::Ping | Self::FloodPing => {
                &[ProbeField::Address, ProbeField::Count, ProbeField::Src]
            }
            Self::Traceroute => &[
                ProbeField::Address,
                ProbeField::Count,
                ProbeField::Src,
                ProbeField::Protocol,
            ],
            Self::BandwidthTest => &[ProbeField::Address, ProbeField::Count, ProbeField::Protocol],
            Self::MacScan | Self::WifiScan => &[ProbeField::Src],
            Self::IpScan => &[ProbeField::Address, ProbeField::Src],
            Self::Profiler => &[ProbeField::Count],
        }
    }

    fn preferred_columns(self) -> &'static [&'static str] {
        match self {
            Self::Ping => &["seq", "host", "time", "ttl", "size", "status"],
            Self::Traceroute => &["hop", "address", "status", "time", "loss"],
            Self::BandwidthTest => &[
                "status",
                "tx-current",
                "rx-current",
                "tx-10-second-average",
                "rx-10-second-average",
            ],
            Self::FloodPing => &["sent", "received", "min-rtt", "avg-rtt", "max-rtt"],
            Self::MacScan => &["address", "mac-address", "age"],
            Self::IpScan => &["address", "mac-address", "time"],
            Self::Profiler => &["name", "usage", "load"],
            Self::WifiScan => &["ssid", "bssid", "channel", "signal", "security"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeField {
    Address,
    Count,
    Src,
    Protocol,
}

impl ProbeField {
    fn label(self) -> &'static str {
        match self {
            Self::Address => "addr",
            Self::Count => "count",
            Self::Src => "src",
            Self::Protocol => "proto",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeState {
    pub kind: ProbeKind,
    pub address: String,
    pub count: String,
    pub src: String,
    pub protocol: String,
    pub focus: ProbeField,
    pub running: bool,
    pub samples: VecDeque<HashMap<String, String>>,
    pub offset: usize,
    pub error: Option<String>,
    pub generation: u64,
}

impl ProbeState {
    #[must_use]
    pub fn new(kind: ProbeKind, generation: u64) -> Self {
        Self {
            kind,
            address: String::new(),
            count: kind.default_count().to_string(),
            src: String::new(),
            protocol: match kind {
                ProbeKind::Traceroute => "icmp".into(),
                ProbeKind::BandwidthTest => "tcp".into(),
                _ => String::new(),
            },
            focus: kind
                .fields()
                .first()
                .copied()
                .unwrap_or(ProbeField::Address),
            running: false,
            samples: VecDeque::new(),
            offset: 0,
            error: None,
            generation,
        }
    }

    pub fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            ProbeField::Address => &mut self.address,
            ProbeField::Count => &mut self.count,
            ProbeField::Src => &mut self.src,
            ProbeField::Protocol => &mut self.protocol,
        }
    }

    pub fn cycle_focus(&mut self) {
        let fields = self.kind.fields();
        let idx = fields
            .iter()
            .position(|field| *field == self.focus)
            .unwrap_or(0);
        self.focus = fields[(idx + 1) % fields.len()];
    }

    pub fn insert_char(&mut self, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        self.focused_mut().push(ch);
    }

    pub fn backspace(&mut self) {
        self.focused_mut().pop();
    }

    pub fn push_samples(&mut self, rows: Vec<HashMap<String, String>>) {
        for row in rows {
            if self.samples.len() == SAMPLE_CAP {
                self.samples.pop_front();
            }
            self.samples.push_back(row);
        }
        let max = self.samples.len().saturating_sub(1);
        self.offset = self.offset.min(max);
    }
}

pub fn render_probe(frame: &mut Frame<'_>, area: Rect, probe: &ProbeState, styles: &Styles) {
    dim_canvas(frame, area, styles);
    let rect = compact_modal_rect(
        area,
        area.width.saturating_sub(4).clamp(40, 96),
        area.height.saturating_sub(2).clamp(10, 30),
    );
    frame.render_widget(Clear, rect);
    let live = if probe.running { "RUNNING" } else { "READY" };
    let block = Block::default()
        .title(Span::styled(
            format!(" {} · {live} ", probe.kind.title()),
            styles.title,
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    let mut spans = Vec::new();
    for (i, field) in probe.kind.fields().iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", styles.muted));
        }
        let focused = probe.focus == *field;
        let value = match field {
            ProbeField::Address => probe.address.as_str(),
            ProbeField::Count => probe.count.as_str(),
            ProbeField::Src => probe.src.as_str(),
            ProbeField::Protocol => probe.protocol.as_str(),
        };
        let style = if focused { styles.focus } else { styles.muted };
        spans.push(Span::styled(format!("{} ", field.label()), style));
        spans.push(Span::styled(
            if value.is_empty() { "—" } else { value }.to_string(),
            if focused { styles.focus } else { styles.text },
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), chunks[0]);

    let columns = result_columns(probe);
    let mut lines = vec![Line::from(Span::styled(
        format_header(&columns),
        styles.muted,
    ))];
    let start = probe.offset.min(probe.samples.len());
    for sample in probe.samples.iter().skip(start) {
        lines.push(Line::from(Span::styled(
            format_row(sample, &columns),
            styles.text,
        )));
    }
    if probe.samples.is_empty() {
        let msg = probe
            .error
            .as_deref()
            .unwrap_or("enter to start · no samples yet");
        lines.push(Line::from(Span::styled(msg, styles.muted)));
    } else if let Some(err) = probe.error.as_deref() {
        lines.push(Line::from(Span::styled(err, styles.muted)));
    }
    frame.render_widget(Paragraph::new(lines), chunks[1]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "tab field   enter start   esc close",
            styles.muted,
        ))),
        chunks[2],
    );
}

fn result_columns(probe: &ProbeState) -> Vec<String> {
    let preferred = probe.kind.preferred_columns();
    if let Some(sample) = probe.samples.front() {
        let mut cols: Vec<String> = preferred
            .iter()
            .filter(|key| sample.contains_key(**key))
            .map(|key| (*key).to_string())
            .collect();
        if cols.is_empty() {
            cols = sample
                .keys()
                .filter(|key| key.as_str() != ".id")
                .cloned()
                .collect();
            cols.sort();
        }
        cols
    } else {
        preferred.iter().map(|key| (*key).to_string()).collect()
    }
}

fn format_header(columns: &[String]) -> String {
    columns
        .iter()
        .map(|key| format!("{key:<14}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_row(sample: &HashMap<String, String>, columns: &[String]) -> String {
    columns
        .iter()
        .map(|key| {
            let value = sample.get(key).map_or("", String::as_str);
            format!("{value:<14}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::styles::Styles;

    #[test]
    fn bounds_sample_buffer() {
        let mut probe = ProbeState::new(ProbeKind::Ping, 1);
        let rows: Vec<_> = (0..100)
            .map(|i| HashMap::from([("seq".into(), format!("{i}"))]))
            .collect();
        probe.push_samples(rows);
        assert_eq!(probe.samples.len(), SAMPLE_CAP);
    }

    #[test]
    fn ignores_control_runes() {
        let mut probe = ProbeState::new(ProbeKind::Ping, 1);
        probe.insert_char('\0');
        probe.insert_char('8');
        assert_eq!(probe.address, "8");
    }

    #[test]
    fn ping_defaults_count_to_four() {
        let probe = ProbeState::new(ProbeKind::Ping, 1);
        assert_eq!(probe.count, "4");
        let trace = ProbeState::new(ProbeKind::Traceroute, 1);
        assert_eq!(trace.count, "8");
        assert_eq!(trace.protocol, "icmp");
    }

    #[test]
    fn narrow_empty_probe_does_not_panic() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(20, 8);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let probe = ProbeState::new(ProbeKind::Ping, 1);
        terminal
            .draw(|frame| render_probe(frame, frame.area(), &probe, &styles))
            .expect("draw");
    }
}
