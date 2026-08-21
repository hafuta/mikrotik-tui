//! Shared domain types, resource catalog, and pluggable theme infrastructure.
//!
//! This crate has no UI or networking dependencies. Themes expose semantic
//! palettes; concrete styling belongs in `mtui-ui`.

mod resources;
mod theme;

pub use resources::{
    ALL_RESOURCES, ColumnSpec, DASHBOARD_ID, FetchKind, NAVIGATION, NavGroup, NavItem,
    ResourceSpec, navigation_tree, resource_by_id,
};
pub use theme::{
    ColorRgb, DefaultTheme, Palette, Theme, ThemeError, ThemeId, ThemeRegistry, ThemeSet,
};
