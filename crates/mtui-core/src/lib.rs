//! Shared domain types, resource catalog, and pluggable theme infrastructure.
//!
//! This crate has no UI or networking dependencies. Themes expose semantic
//! palettes; concrete styling belongs in `mtui-ui`.

mod actions;
mod forms;
mod interface_write;
mod resources;
mod theme;

pub use actions::{
    ACTION_ADD, ACTION_COPY, ACTION_EDIT, ACTION_REMOVE, ACTION_RESET, ACTION_TOGGLE, ACTION_TORCH,
    ActionCommand, ActionKind, ActionSpec, ActionWhen, ETHERNET_ACTIONS, INTERFACE_CREATE_TARGETS,
    INTERFACE_LIST_ACTIONS, action_label, resolve_actions, truthy,
};
pub use forms::{FieldKind, FieldSpec, FormSchema, FormSection, extra_status_fields, patch_body};
pub use resources::{
    ALL_RESOURCES, ColumnSpec, DASHBOARD_ID, FetchKind, NAVIGATION, NavGroup, NavItem,
    ResourceSpec, navigation_tree, resource_by_id,
};
pub use theme::{
    ColorRgb, DefaultTheme, Palette, Theme, ThemeError, ThemeId, ThemeRegistry, ThemeSet,
};
