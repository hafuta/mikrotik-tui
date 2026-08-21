//! Theme-driven ratatui styles (foreground / borders only).

use mtui_core::{ColorRgb, Palette};
use ratatui::style::{Color, Modifier, Style};

#[must_use]
pub fn rgb_color(c: ColorRgb) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

/// Semantic styles derived from a theme palette.
#[derive(Debug, Clone, Copy)]
pub struct Styles {
    pub base: Style,
    pub panel: Style,
    pub text: Style,
    pub muted: Style,
    pub focus: Style,
    pub signal: Style,
    pub alert: Style,
    pub error: Style,
    pub border: Style,
    pub title: Style,
}

impl Styles {
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            base: Style::default().fg(rgb_color(p.text)),
            panel: Style::default().fg(rgb_color(p.text)),
            text: Style::default().fg(rgb_color(p.text)),
            muted: Style::default().fg(rgb_color(p.muted)),
            focus: Style::default()
                .fg(rgb_color(p.focus))
                .add_modifier(Modifier::BOLD),
            signal: Style::default().fg(rgb_color(p.signal)),
            alert: Style::default().fg(rgb_color(p.alert)),
            error: Style::default().fg(rgb_color(p.error)),
            border: Style::default().fg(rgb_color(p.border)),
            title: Style::default()
                .fg(rgb_color(p.focus))
                .add_modifier(Modifier::BOLD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};

    #[test]
    fn styles_from_default_theme() {
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        assert_eq!(styles.focus.fg, Some(rgb_color(theme.palette().focus)));
    }
}
