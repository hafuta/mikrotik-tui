//! Pluggable theme infrastructure.
//!
//! Semantic tokens are stable. Concrete color values live in named themes
//! (starting with [`DefaultTheme`]). The UI never hard-codes product colors;
//! it resolves styles from the active theme's [`Palette`].

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

/// Stable identifier for a theme (e.g. `"default"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThemeId(String);

impl ThemeId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ThemeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ThemeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// 24-bit RGB color used by theme palettes (UI maps this to the terminal crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse `#RRGGBB` (case-insensitive).
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Mix `self` toward `to` by `ratio` (0..=1), truncating like integer blending.
    #[must_use]
    pub fn blend(self, to: Self, ratio: f64) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let mix = |from: u8, toward: u8| -> u8 {
            let from = i32::from(from);
            let toward = i32::from(toward);
            #[allow(clippy::cast_possible_truncation)]
            let stepped = (f64::from(toward - from) * ratio) as i32;
            let mixed = from.saturating_add(stepped);
            u8::try_from(mixed.clamp(0, 255)).unwrap_or(0)
        };
        Self {
            r: mix(self.r, to.r),
            g: mix(self.g, to.g),
            b: mix(self.b, to.b),
        }
    }
}

/// Semantic color roles shared by every theme.
///
/// Components must depend on these names, never on ad-hoc literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub void: ColorRgb,
    pub panel: ColorRgb,
    pub band: ColorRgb,
    pub inset: ColorRgb,
    pub selection: ColorRgb,
    pub data: ColorRgb,
    pub text: ColorRgb,
    pub focus: ColorRgb,
    pub signal: ColorRgb,
    pub alert: ColorRgb,
    pub muted: ColorRgb,
    pub border: ColorRgb,
    pub error: ColorRgb,
}

/// A named visual theme. New themes implement this trait and register in
/// [`ThemeRegistry`].
pub trait Theme: Send + Sync {
    fn id(&self) -> &ThemeId;
    fn name(&self) -> &'static str;
    fn palette(&self) -> &Palette;
}

/// Errors when selecting or looking up themes.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ThemeError {
    #[error("theme not found: {0}")]
    NotFound(String),
}

/// Built-in dark control-deck palette (canonical product look).
#[derive(Debug, Clone)]
pub struct DefaultTheme {
    id: ThemeId,
    palette: Palette,
}

impl DefaultTheme {
    pub const ID: &'static str = "default";

    #[must_use]
    pub fn new() -> Self {
        Self {
            id: ThemeId::new(Self::ID),
            palette: Palette {
                void: ColorRgb::from_hex("#0C1118").expect("valid hex"),
                panel: ColorRgb::from_hex("#0C1118").expect("valid hex"),
                band: ColorRgb::from_hex("#151920").expect("valid hex"),
                inset: ColorRgb::from_hex("#12171E").expect("valid hex"),
                selection: ColorRgb::from_hex("#1A293D").expect("valid hex"),
                data: ColorRgb::from_hex("#C9D4E2").expect("valid hex"),
                text: ColorRgb::from_hex("#E8EEF6").expect("valid hex"),
                focus: ColorRgb::from_hex("#62A8FF").expect("valid hex"),
                signal: ColorRgb::from_hex("#55D6BE").expect("valid hex"),
                alert: ColorRgb::from_hex("#FFB454").expect("valid hex"),
                muted: ColorRgb::from_hex("#8B97A8").expect("valid hex"),
                border: ColorRgb::from_hex("#1F242A").expect("valid hex"),
                error: ColorRgb::from_hex("#FF6B7A").expect("valid hex"),
            },
        }
    }
}

impl Default for DefaultTheme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme for DefaultTheme {
    fn id(&self) -> &ThemeId {
        &self.id
    }

    fn name(&self) -> &'static str {
        "Default"
    }

    fn palette(&self) -> &Palette {
        &self.palette
    }
}

/// Registry of available themes plus the currently active selection.
#[derive(Clone)]
pub struct ThemeRegistry {
    themes: HashMap<ThemeId, Arc<dyn Theme>>,
    active: ThemeId,
}

impl ThemeRegistry {
    /// Create a registry containing only the default theme, already selected.
    #[must_use]
    pub fn with_default() -> Self {
        let default = Arc::new(DefaultTheme::new());
        let id = default.id().clone();
        let mut themes = HashMap::new();
        themes.insert(id.clone(), default as Arc<dyn Theme>);
        Self { themes, active: id }
    }

    /// Register an additional theme. Replaces any theme with the same id.
    pub fn register(&mut self, theme: Arc<dyn Theme>) {
        let id = theme.id().clone();
        self.themes.insert(id, theme);
    }

    /// Select an already-registered theme by id.
    pub fn set_active(&mut self, id: impl Into<ThemeId>) -> Result<(), ThemeError> {
        let id = id.into();
        if !self.themes.contains_key(&id) {
            return Err(ThemeError::NotFound(id.to_string()));
        }
        self.active = id;
        Ok(())
    }

    #[must_use]
    pub fn active_id(&self) -> &ThemeId {
        &self.active
    }

    #[must_use]
    pub fn active(&self) -> Arc<dyn Theme> {
        self.themes
            .get(&self.active)
            .cloned()
            .expect("active theme always registered")
    }

    #[must_use]
    pub fn get(&self, id: &ThemeId) -> Option<Arc<dyn Theme>> {
        self.themes.get(id).cloned()
    }

    #[must_use]
    pub fn ids(&self) -> Vec<ThemeId> {
        let mut ids: Vec<_> = self.themes.keys().cloned().collect();
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        ids
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::with_default()
    }
}

/// Convenience snapshot of the active theme for rendering.
#[derive(Debug, Clone)]
pub struct ThemeSet {
    pub id: ThemeId,
    pub name: String,
    pub palette: Palette,
}

impl ThemeSet {
    #[must_use]
    pub fn from_theme(theme: &dyn Theme) -> Self {
        Self {
            id: theme.id().clone(),
            name: theme.name().to_string(),
            palette: *theme.palette(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_active() {
        let registry = ThemeRegistry::with_default();
        assert_eq!(registry.active_id().as_str(), "default");
        assert_eq!(registry.active().name(), "Default");
        assert_eq!(
            registry.active().palette().focus,
            ColorRgb::from_hex("#62A8FF").unwrap()
        );
    }

    #[test]
    fn unknown_theme_errors() {
        let mut registry = ThemeRegistry::with_default();
        assert!(matches!(
            registry.set_active("neon"),
            Err(ThemeError::NotFound(_))
        ));
    }

    #[test]
    fn register_and_switch() {
        #[derive(Debug)]
        struct Alt {
            id: ThemeId,
            palette: Palette,
        }
        impl Theme for Alt {
            fn id(&self) -> &ThemeId {
                &self.id
            }
            fn name(&self) -> &'static str {
                "Alt"
            }
            fn palette(&self) -> &Palette {
                &self.palette
            }
        }

        let mut registry = ThemeRegistry::with_default();
        let mut palette = *registry.active().palette();
        palette.focus = ColorRgb::new(255, 0, 0);
        registry.register(Arc::new(Alt {
            id: ThemeId::new("alt"),
            palette,
        }));
        registry.set_active("alt").unwrap();
        assert_eq!(registry.active().palette().focus, ColorRgb::new(255, 0, 0));
    }
}
