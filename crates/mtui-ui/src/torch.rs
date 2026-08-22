//! Live torch overlay: filters plus a bounded sample table.

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
pub enum TorchField {
    Src,
    Dst,
    Protocol,
    Port,
}

impl TorchField {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Src => Self::Dst,
            Self::Dst => Self::Protocol,
            Self::Protocol => Self::Port,
            Self::Port => Self::Src,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorchState {
    pub interface: String,
    pub record_id: String,
    pub src: String,
    pub dst: String,
    pub protocol: String,
    pub port: String,
    pub focus: TorchField,
    pub running: bool,
    pub samples: VecDeque<HashMap<String, String>>,
    pub offset: usize,
    pub error: Option<String>,
    pub generation: u64,
}

impl TorchState {
    #[must_use]
    pub fn new(
        interface: impl Into<String>,
        record_id: impl Into<String>,
        generation: u64,
    ) -> Self {
        Self {
            interface: interface.into(),
            record_id: record_id.into(),
            src: String::new(),
            dst: String::new(),
            protocol: String::new(),
            port: String::new(),
            focus: TorchField::Src,
            running: false,
            samples: VecDeque::new(),
            offset: 0,
            error: None,
            generation,
        }
    }

    pub fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            TorchField::Src => &mut self.src,
            TorchField::Dst => &mut self.dst,
            TorchField::Protocol => &mut self.protocol,
            TorchField::Port => &mut self.port,
        }
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

pub fn render_torch(frame: &mut Frame<'_>, area: Rect, torch: &TorchState, styles: &Styles) {
    dim_canvas(frame, area, styles);
    let rect = compact_modal_rect(
        area,
        area.width.saturating_sub(4).clamp(52, 96),
        area.height.saturating_sub(2).clamp(14, 30),
    );
    frame.render_widget(Clear, rect);
    let live = if torch.running { "LIVE" } else { "PAUSED" };
    let block = Block::default()
        .title(Span::styled(
            format!(" Torch · {} · {live} ", torch.interface),
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
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(inner);

    let filters = [
        ("src", torch.src.as_str(), torch.focus == TorchField::Src),
        ("dst", torch.dst.as_str(), torch.focus == TorchField::Dst),
        (
            "proto",
            torch.protocol.as_str(),
            torch.focus == TorchField::Protocol,
        ),
        ("port", torch.port.as_str(), torch.focus == TorchField::Port),
    ];
    let mut spans = Vec::new();
    for (i, (label, value, focused)) in filters.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", styles.muted));
        }
        let style = if *focused { styles.focus } else { styles.muted };
        spans.push(Span::styled(format!("{label} "), style));
        spans.push(Span::styled(
            if value.is_empty() { "—" } else { *value }.to_string(),
            if *focused { styles.focus } else { styles.text },
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), chunks[0]);

    let mut lines = vec![Line::from(Span::styled(
        format!("{:<18} {:<18} {:<8} {:<8}", "src", "dst", "proto", "bytes"),
        styles.muted,
    ))];
    let start = torch.offset.min(torch.samples.len());
    for sample in torch.samples.iter().skip(start) {
        lines.push(Line::from(Span::styled(
            format!(
                "{:<18} {:<18} {:<8} {:<8}",
                sample
                    .get("src-address")
                    .or_else(|| sample.get("src"))
                    .map_or("", String::as_str),
                sample
                    .get("dst-address")
                    .or_else(|| sample.get("dst"))
                    .map_or("", String::as_str),
                sample
                    .get("ip-protocol")
                    .or_else(|| sample.get("protocol"))
                    .map_or("", String::as_str),
                sample
                    .get("tx")
                    .or_else(|| sample.get("bytes"))
                    .map_or("", String::as_str),
            ),
            styles.text,
        )));
    }
    if torch.samples.is_empty() {
        let msg = torch
            .error
            .as_deref()
            .unwrap_or("space to start · no samples yet");
        lines.push(Line::from(Span::styled(msg, styles.muted)));
    }
    frame.render_widget(Paragraph::new(lines), chunks[1]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "tab filter   space start/stop   esc close",
            styles.muted,
        ))),
        chunks[2],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_sample_buffer() {
        let mut torch = TorchState::new("ether1", "*1", 1);
        let rows: Vec<_> = (0..100)
            .map(|i| HashMap::from([("src".into(), format!("{i}"))]))
            .collect();
        torch.push_samples(rows);
        assert_eq!(torch.samples.len(), SAMPLE_CAP);
    }

    #[test]
    fn ignores_control_runes() {
        let mut torch = TorchState::new("ether1", "*1", 1);
        torch.insert_char('\0');
        torch.insert_char('a');
        assert_eq!(torch.src, "a");
    }
}
