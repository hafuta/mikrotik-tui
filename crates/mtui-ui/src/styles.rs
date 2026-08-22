//! Theme-driven ratatui styles.
//!
//! Shared component styles are foreground-only. Backgrounds are applied at
//! the paint boundary to a known rectangle (see [`crate::paint`]).

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
    pub data: Style,
    pub muted: Style,
    pub quiet: Style,
    pub hidden: Style,
    pub focus: Style,
    pub key: Style,
    pub signal: Style,
    pub alert: Style,
    pub error: Style,
    pub border: Style,
    pub title: Style,
    pub void: Color,
    pub band: Color,
    pub inset: Color,
    pub selection: Color,
}

impl Styles {
    #[must_use]
    pub fn from_palette(p: &Palette) -> Self {
        Self {
            base: Style::default().fg(rgb_color(p.text)),
            panel: Style::default().fg(rgb_color(p.text)),
            text: Style::default().fg(rgb_color(p.text)),
            data: Style::default().fg(rgb_color(p.data)),
            muted: Style::default().fg(rgb_color(p.muted)),
            quiet: Style::default().fg(rgb_color(p.muted.blend(p.void, 0.32))),
            hidden: Style::default().fg(rgb_color(p.muted.blend(p.void, 0.55))),
            focus: Style::default()
                .fg(rgb_color(p.focus))
                .add_modifier(Modifier::BOLD),
            key: Style::default().fg(rgb_color(p.focus)),
            signal: Style::default().fg(rgb_color(p.signal)),
            alert: Style::default().fg(rgb_color(p.alert)),
            error: Style::default().fg(rgb_color(p.error)),
            border: Style::default().fg(rgb_color(p.border)),
            title: Style::default()
                .fg(rgb_color(p.text))
                .add_modifier(Modifier::BOLD),
            void: rgb_color(p.void),
            band: rgb_color(p.band),
            inset: rgb_color(p.inset),
            selection: rgb_color(p.selection),
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
        assert!(styles.text.bg.is_none());
        assert!(styles.muted.bg.is_none());
        assert!(styles.hidden.bg.is_none());
        assert_ne!(styles.quiet.fg, styles.muted.fg);
        assert_ne!(styles.quiet.fg, styles.text.fg);
        assert_ne!(styles.hidden.fg, styles.quiet.fg);
        assert_eq!(styles.band, rgb_color(theme.palette().band));
        assert_eq!(styles.selection, rgb_color(theme.palette().selection));
    }
}
