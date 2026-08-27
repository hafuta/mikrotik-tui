//! Terminal color-depth detection and RGB → xterm-256 mapping.
//!
//! Truecolor (`Color::Rgb`) is the default. Apple Terminal.app only gained
//! reliable 24-bit color in macOS 26 Tahoe (Terminal 2.15 / bundle version
//! 465). Older builds mis-parse `38;2;R;G;B` sequences as SGR attributes and
//! wash the screen green. Those builds get indexed-256 colors instead.
//!
//! Detection is capability-based, not “every `Apple_Terminal`”:
//! 1. `ROUTEROS_TUI_COLOR` (`auto` / `truecolor` / `256`)
//! 2. `COLORTERM=truecolor|24bit`
//! 3. `TERM` ending in `-direct` or containing `truecolor`
//! 4. `TERM_PROGRAM=Apple_Terminal` below the Tahoe cutoff → 256
//! 5. otherwise truecolor

use std::env;

use mtui_core::ColorRgb;
use ratatui::style::Color;

/// First Terminal.app bundle version that supports 24-bit color (macOS 26).
const APPLE_TERMINAL_TRUECOLOR_BUILD: u32 = 465;

/// First marketing version (`2.15`) that supports 24-bit color.
const APPLE_TERMINAL_TRUECOLOR_MAJOR: u32 = 2;
const APPLE_TERMINAL_TRUECOLOR_MINOR: u32 = 15;

/// How many distinct colors the terminal can take from our palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    /// 24-bit `Color::Rgb`.
    TrueColor,
    /// xterm 256-color `Color::Indexed`.
    Ansi256,
}

impl ColorDepth {
    /// Probe the process environment.
    #[must_use]
    pub fn detect() -> Self {
        Self::from_vars(env::vars())
    }

    /// Probe an arbitrary key/value source (tests inject a map).
    #[must_use]
    pub fn from_vars<I, K, V>(vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut colorterm = None;
        let mut term = None;
        let mut program = None;
        let mut program_version = None;
        let mut override_color = None;
        for (key, value) in vars {
            match key.as_ref() {
                "ROUTEROS_TUI_COLOR" => override_color = Some(value.as_ref().to_string()),
                "COLORTERM" => colorterm = Some(value.as_ref().to_string()),
                "TERM" => term = Some(value.as_ref().to_string()),
                "TERM_PROGRAM" => program = Some(value.as_ref().to_string()),
                "TERM_PROGRAM_VERSION" => program_version = Some(value.as_ref().to_string()),
                _ => {}
            }
        }
        from_captured(
            override_color.as_deref(),
            colorterm.as_deref(),
            term.as_deref(),
            program.as_deref(),
            program_version.as_deref(),
        )
    }
}

fn from_captured(
    override_color: Option<&str>,
    colorterm: Option<&str>,
    term: Option<&str>,
    program: Option<&str>,
    program_version: Option<&str>,
) -> ColorDepth {
    if let Some(forced) = override_color.and_then(parse_color_override) {
        return forced;
    }
    if colorterm.is_some_and(advertises_truecolor) {
        return ColorDepth::TrueColor;
    }
    if term.is_some_and(term_advertises_truecolor) {
        return ColorDepth::TrueColor;
    }
    if is_apple_terminal(program.unwrap_or(""))
        && !apple_terminal_has_truecolor(program_version.unwrap_or(""))
    {
        return ColorDepth::Ansi256;
    }
    ColorDepth::TrueColor
}

fn parse_color_override(raw: &str) -> Option<ColorDepth> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "256" | "ansi256" | "indexed" => Some(ColorDepth::Ansi256),
        "truecolor" | "true-color" | "24bit" | "24-bit" | "rgb" => Some(ColorDepth::TrueColor),
        _ => None,
    }
}

fn advertises_truecolor(colorterm: &str) -> bool {
    let value = colorterm.trim().to_ascii_lowercase();
    value.contains("truecolor") || value.contains("24bit")
}

fn term_advertises_truecolor(term: &str) -> bool {
    let value = term.trim().to_ascii_lowercase();
    value.ends_with("-direct") || value.contains("truecolor")
}

fn is_apple_terminal(program: &str) -> bool {
    let normalized = program.trim().replace(' ', "_").to_ascii_lowercase();
    normalized == "apple_terminal"
}

/// `TERM_PROGRAM_VERSION` is either a bundle version (`455`, `470.2`) or a
/// marketing version (`2.14`, `2.15.1`).
fn apple_terminal_has_truecolor(version: &str) -> bool {
    let token = version.trim().split([' ', '(']).next().unwrap_or("").trim();
    let mut parts = token.split('.');
    let Some(major) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
        return false;
    };
    if major >= 100 {
        return major >= APPLE_TERMINAL_TRUECOLOR_BUILD;
    }
    let minor = parts
        .next()
        .and_then(|part| part.parse::<u32>().ok())
        .unwrap_or(0);
    major > APPLE_TERMINAL_TRUECOLOR_MAJOR
        || (major == APPLE_TERMINAL_TRUECOLOR_MAJOR && minor >= APPLE_TERMINAL_TRUECOLOR_MINOR)
}

/// Map a theme color to a ratatui `Color` for this terminal.
#[must_use]
pub fn rgb_color_for(c: ColorRgb, depth: ColorDepth) -> Color {
    match depth {
        ColorDepth::TrueColor => Color::Rgb(c.r, c.g, c.b),
        ColorDepth::Ansi256 => Color::Indexed(rgb_to_xterm256(c.r, c.g, c.b)),
    }
}

/// Nearest xterm-256 index (6×6×6 cube or 24-step gray ramp).
#[must_use]
pub fn rgb_to_xterm256(r: u8, g: u8, b: u8) -> u8 {
    let cube_r = cube_component(r);
    let cube_g = cube_component(g);
    let cube_b = cube_component(b);
    let cube_index = 16 + 36 * cube_r + 6 * cube_g + cube_b;
    let cube_dist = color_distance(
        r,
        g,
        b,
        cube_level(cube_r),
        cube_level(cube_g),
        cube_level(cube_b),
    );

    let gray_n = gray_index(r, g, b);
    let gray_v = gray_level(gray_n);
    let gray_index = 232 + gray_n;
    let gray_dist = color_distance(r, g, b, gray_v, gray_v, gray_v);

    if gray_dist < cube_dist {
        gray_index
    } else {
        cube_index
    }
}

fn cube_component(value: u8) -> u8 {
    if value < 48 {
        0
    } else if value < 115 {
        1
    } else {
        let stepped = (u16::from(value) - 35) / 40;
        u8::try_from(stepped.min(5)).unwrap_or(5)
    }
}

fn cube_level(index: u8) -> u8 {
    if index == 0 { 0 } else { 40 * index + 55 }
}

fn gray_index(r: u8, g: u8, b: u8) -> u8 {
    let avg = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
    u8::try_from(((avg.saturating_sub(8)) / 10).min(23)).unwrap_or(23)
}

fn gray_level(index: u8) -> u8 {
    8 + 10 * index
}

fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> i32 {
    let dr = i32::from(r1) - i32::from(r2);
    let dg = i32::from(g1) - i32::from(g2);
    let db = i32::from(b1) - i32::from(b2);
    dr * dr + dg * dg + db * db
}

#[cfg(test)]
mod tests {
    use super::*;

    fn depth(pairs: &[(&str, &str)]) -> ColorDepth {
        ColorDepth::from_vars(pairs.iter().map(|(k, v)| (*k, *v)))
    }

    #[test]
    fn default_is_truecolor() {
        assert_eq!(depth(&[]), ColorDepth::TrueColor);
    }

    #[test]
    fn colorterm_wins_over_old_apple_terminal() {
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("TERM_PROGRAM_VERSION", "440"),
                ("COLORTERM", "truecolor"),
            ]),
            ColorDepth::TrueColor
        );
    }

    #[test]
    fn old_apple_terminal_falls_back_to_256() {
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("TERM_PROGRAM_VERSION", "455"),
                ("TERM", "xterm-256color"),
            ]),
            ColorDepth::Ansi256
        );
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("TERM_PROGRAM_VERSION", "2.14"),
            ]),
            ColorDepth::Ansi256
        );
    }

    #[test]
    fn tahoe_apple_terminal_keeps_truecolor() {
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("TERM_PROGRAM_VERSION", "465"),
            ]),
            ColorDepth::TrueColor
        );
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple_Terminal"),
                ("TERM_PROGRAM_VERSION", "470.2"),
            ]),
            ColorDepth::TrueColor
        );
        assert_eq!(
            depth(&[
                ("TERM_PROGRAM", "Apple Terminal"),
                ("TERM_PROGRAM_VERSION", "2.15.1"),
            ]),
            ColorDepth::TrueColor
        );
    }

    #[test]
    fn unknown_apple_terminal_version_is_conservative() {
        assert_eq!(
            depth(&[("TERM_PROGRAM", "Apple_Terminal")]),
            ColorDepth::Ansi256
        );
    }

    #[test]
    fn iterm_is_truecolor_without_colorterm() {
        assert_eq!(
            depth(&[("TERM_PROGRAM", "iTerm.app"), ("TERM", "xterm-256color")]),
            ColorDepth::TrueColor
        );
    }

    #[test]
    fn override_forces_256() {
        assert_eq!(
            depth(&[
                ("ROUTEROS_TUI_COLOR", "256"),
                ("COLORTERM", "truecolor"),
                ("TERM_PROGRAM", "iTerm.app"),
            ]),
            ColorDepth::Ansi256
        );
    }

    #[test]
    fn term_direct_is_truecolor() {
        assert_eq!(depth(&[("TERM", "xterm-direct")]), ColorDepth::TrueColor);
    }

    #[test]
    fn red_maps_to_xterm_cube() {
        assert_eq!(rgb_to_xterm256(255, 0, 0), 196);
    }

    #[test]
    fn rgb_color_for_respects_depth() {
        let rgb = ColorRgb {
            r: 12,
            g: 17,
            b: 24,
        };
        assert_eq!(
            rgb_color_for(rgb, ColorDepth::TrueColor),
            Color::Rgb(12, 17, 24)
        );
        assert!(matches!(
            rgb_color_for(rgb, ColorDepth::Ansi256),
            Color::Indexed(_)
        ));
    }
}
