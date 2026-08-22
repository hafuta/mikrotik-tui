//! Compact action menu for the selected resource row.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::login::is_printable_char;
use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::styles::Styles;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMenuItem {
    pub id: String,
    pub label: String,
    pub keys: String,
    pub danger: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMenuState {
    pub items: Vec<ActionMenuItem>,
    pub query: String,
    pub selected: usize,
}

impl ActionMenuState {
    #[must_use]
    pub fn new(items: Vec<ActionMenuItem>) -> Self {
        Self {
            items,
            query: String::new(),
            selected: 0,
        }
    }

    #[must_use]
    pub fn filtered(&self) -> Vec<&ActionMenuItem> {
        let q = self.query.to_ascii_lowercase();
        self.items
            .iter()
            .filter(|item| {
                q.is_empty() || item.label.to_ascii_lowercase().contains(&q) || item.id.contains(&q)
            })
            .collect()
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.filtered().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let cur = isize::try_from(self.selected).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        self.selected = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
    }

    pub fn insert_char(&mut self, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        self.query.push(ch);
        self.selected = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.selected = 0;
    }

    #[must_use]
    pub fn confirm(&self) -> Option<String> {
        self.filtered()
            .get(self.selected)
            .map(|item| item.id.clone())
    }
}

pub fn render_action_menu(
    frame: &mut Frame<'_>,
    area: Rect,
    menu: &ActionMenuState,
    styles: &Styles,
) {
    dim_canvas(frame, area, styles);
    let matches = menu.filtered();
    let rows = u16::try_from(matches.len().clamp(3, 10)).unwrap_or(3);
    let height = rows.saturating_add(5);
    let rect = compact_modal_rect(area, 48, height);
    frame.render_widget(Clear, rect);
    let block = Block::default()
        .title(Span::styled(" Actions ", styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines = vec![Line::from(Span::styled(
        format!("/{}", menu.query),
        styles.focus,
    ))];
    if matches.is_empty() {
        lines.push(Line::from(Span::styled(
            "no matching actions",
            styles.muted,
        )));
    }
    for (idx, item) in matches.iter().enumerate() {
        let focused = idx == menu.selected;
        let style = if item.danger && focused {
            styles.alert
        } else if focused {
            styles.focus
        } else if item.danger {
            styles.alert
        } else {
            styles.text
        };
        let mark = if focused { ">" } else { " " };
        lines.push(Line::from(Span::styled(
            format!("{mark} {:<16} {}", item.label, item.keys),
            style,
        )));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_and_confirm() {
        let mut menu = ActionMenuState::new(vec![
            ActionMenuItem {
                id: "edit".into(),
                label: "Edit".into(),
                keys: "e".into(),
                danger: false,
            },
            ActionMenuItem {
                id: "remove".into(),
                label: "Remove".into(),
                keys: "x".into(),
                danger: true,
            },
        ]);
        menu.insert_char('r');
        assert_eq!(menu.confirm().as_deref(), Some("remove"));
        menu.insert_char('\0');
        assert_eq!(menu.query, "r");
    }
}
