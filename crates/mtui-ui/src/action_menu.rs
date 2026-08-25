//! Compact action menu for the selected resource row.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::layout::clip_line;
use crate::login::is_printable_char;
use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::scroll::ScrollView;
use crate::styles::Styles;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionMenuItem {
    pub id: String,
    pub label: String,
    pub keys: String,
    pub danger: bool,
    pub note: String,
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
    title: &str,
) {
    dim_canvas(frame, area, styles);
    let matches = menu.filtered();
    let chrome = 7_u16;
    let max_list = usize::from(area.height.saturating_sub(chrome)).clamp(4, 16);
    let list_rows = matches.len().clamp(1, max_list);
    let height = u16::try_from(list_rows.saturating_add(5)).unwrap_or(9);
    let rect = compact_modal_rect(area, 48, height);
    frame.render_widget(Clear, rect);

    let view = ScrollView::around_focus(menu.selected, list_rows, matches.len());
    let mut title_spans = vec![Span::styled(format!(" {title} "), styles.title)];
    let range = view.range_label();
    if !range.is_empty() {
        title_spans.push(Span::styled(format!("{range} "), styles.muted));
    }
    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    let list_width = usize::from(chunks[1].width.max(1));

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            clip_line(
                &format!("/{}", menu.query),
                usize::from(chunks[0].width.max(1)),
            ),
            styles.focus,
        ))),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(action_menu_list_lines(
            menu, &matches, view, list_width, styles,
        )),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            clip_line(
                "type to filter   ↑↓   enter   esc",
                usize::from(chunks[2].width.max(1)),
            ),
            styles.muted,
        ))),
        chunks[2],
    );
}

fn action_menu_list_lines(
    menu: &ActionMenuState,
    matches: &[&ActionMenuItem],
    view: ScrollView,
    width: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if matches.is_empty() {
        lines.push(Line::from(Span::styled(
            "no matching actions",
            styles.muted,
        )));
        return lines;
    }
    for (idx, item) in matches
        .iter()
        .enumerate()
        .skip(view.offset)
        .take(view.visible)
    {
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
        let suffix = if item.note.is_empty() {
            item.keys.clone()
        } else {
            format!("{}  {}", item.keys, item.note)
        };
        let row_style = if !item.note.is_empty() && !focused {
            styles.muted
        } else {
            style
        };
        let gutter = view.gutter(idx.saturating_sub(view.offset));
        let gutter_style = if gutter == '▐' {
            styles.key
        } else {
            styles.quiet
        };
        let body_w = width.saturating_sub(1);
        let body = clip_line(&format!("{mark} {:<16} {suffix}", item.label), body_w);
        let pad =
            " ".repeat(body_w.saturating_sub(unicode_width::UnicodeWidthStr::width(body.as_str())));
        lines.push(Line::from(vec![
            Span::styled(format!("{body}{pad}"), row_style),
            Span::styled(gutter.to_string(), gutter_style),
        ]));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn item(id: &str, label: &str) -> ActionMenuItem {
        ActionMenuItem {
            id: id.into(),
            label: label.into(),
            keys: String::new(),
            danger: false,
            note: String::new(),
        }
    }

    #[test]
    fn filter_and_confirm() {
        let mut menu = ActionMenuState::new(vec![
            ActionMenuItem {
                id: "edit".into(),
                label: "Edit".into(),
                keys: "e".into(),
                danger: false,
                note: String::new(),
            },
            ActionMenuItem {
                id: "remove".into(),
                label: "Remove".into(),
                keys: "x".into(),
                danger: true,
                note: String::new(),
            },
        ]);
        menu.insert_char('r');
        assert_eq!(menu.confirm().as_deref(), Some("remove"));
        menu.insert_char('\0');
        assert_eq!(menu.query, "r");
    }

    #[test]
    fn long_type_picker_scrolls_and_pins_the_hint() {
        let items: Vec<ActionMenuItem> = (0..17)
            .map(|i| item(&format!("t{i}"), &format!("Type {i:02}")))
            .collect();
        let mut menu = ActionMenuState::new(items);
        menu.selected = 16;
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(56, 16);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_action_menu(frame, frame.area(), &menu, &styles, "New interface");
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("New interface"), "{rendered}");
        assert!(
            rendered.contains("Type 16"),
            "focused tail missing: {rendered}"
        );
        assert!(
            !rendered.contains("Type 00"),
            "scrolled list still showed the first row: {rendered}"
        );
        assert!(
            rendered.contains("type to filter"),
            "hint must stay pinned: {rendered}"
        );
        assert!(
            rendered.contains("17/17") || rendered.contains("/17"),
            "missing range chrome: {rendered}"
        );
        assert!(
            rendered.contains('▐') || rendered.contains('│'),
            "missing scroll gutter: {rendered}"
        );
    }
}
