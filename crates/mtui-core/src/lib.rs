//! Shared domain types, resource catalog, and pluggable theme infrastructure.
//!
//! This crate has no UI or networking dependencies. Themes expose semantic
//! palettes; concrete styling belongs in `mtui-ui`.

mod about;
mod actions;
mod bridge_write;
mod forms;
mod interface_write;
mod ip_write;
mod ipv6_write;
mod ppp_write;
mod queue_write;
mod radius_write;
mod resources;
mod routing_write;
mod switch_write;
mod system_write;
mod theme;
mod tools_write;
mod wireguard_write;

pub use about::{AboutCopy, ScreenGuide, WHEN_YOU_NEED_IT, about_copy, screen_guide};
pub use actions::{
    ACTION_ADD, ACTION_BACKUP_LOAD, ACTION_BACKUP_SAVE, ACTION_COPY, ACTION_EDIT, ACTION_REBOOT,
    ACTION_REMOVE, ACTION_RESET, ACTION_SHUTDOWN, ACTION_TOGGLE, ACTION_TORCH, ActionCommand,
    ActionKind, ActionSpec, ActionWhen, ETHERNET_ACTIONS, FILE_ACTIONS, INTERFACE_CREATE_TARGETS,
    INTERFACE_LIST_ACTIONS, RESOURCE_LIFECYCLE_ACTIONS, action_label, is_backup_file,
    resolve_actions, truthy,
};
pub use forms::{FieldKind, FieldSpec, FormSchema, FormSection, extra_status_fields, patch_body};
pub use resources::{
    ALL_RESOURCES, ColumnSpec, DASHBOARD_ID, FetchKind, NAVIGATION, NavGroup, NavItem,
    ResourceSpec, navigation_tree, resource_by_id,
};
pub use theme::{
    ColorRgb, DefaultTheme, Palette, Theme, ThemeError, ThemeId, ThemeRegistry, ThemeSet,
};
