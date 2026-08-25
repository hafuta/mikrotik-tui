//! Shared domain types, resource catalog, and pluggable theme infrastructure.
//!
//! This crate has no UI or networking dependencies. Themes expose semantic
//! palettes; concrete styling belongs in `mtui-ui`.

mod about;
mod access;
mod actions;
mod bridge_write;
mod capabilities;
mod container_write;
mod features;
mod forms;
mod hotspot_write;
mod interface_write;
mod ip_write;
mod ipsec_write;
mod ipv6_write;
mod ppp_write;
mod queue_write;
mod radius_write;
mod resources;
mod routeros_version;
mod routing_write;
mod safe_mode;
mod switch_write;
mod system_write;
mod theme;
mod tools_write;
mod wireguard_write;

pub use about::{AboutCopy, ScreenGuide, WHEN_YOU_NEED_IT, about_copy, screen_guide};
pub use access::{
    POLICY_POLICY, POLICY_READ, POLICY_REBOOT, POLICY_SNIFF, POLICY_TEST, POLICY_WRITE,
    SessionAccess, is_permission_trap, parse_policy_list, permission_denied_copy, required_policy,
    trap_permission_copy,
};
pub use actions::{
    ACTION_ADD, ACTION_BACKUP_LOAD, ACTION_BACKUP_SAVE, ACTION_COPY, ACTION_EDIT, ACTION_REBOOT,
    ACTION_REMOVE, ACTION_RESET, ACTION_SHUTDOWN, ACTION_TOGGLE, ACTION_TORCH, ActionCommand,
    ActionKind, ActionSpec, ActionWhen, CERTIFICATE_ACTIONS, ETHERNET_ACTIONS, FILE_ACTIONS,
    INTERFACE_LIST_ACTIONS, NEIGHBOR_ACTIONS, NeighborConnectTarget, RESOURCE_LIFECYCLE_ACTIONS,
    action_label, is_backup_file, neighbor_connect_target, resolve_actions, truthy,
};
pub use capabilities::{
    BULK_SELECT_RESOURCES, CONTAINER_PACKAGES, MISSING_PATH_REASON, WIFI_PACKAGES,
    WIRELESS_PACKAGES, cli_path_available, inspect_parent_key, installed_package_names,
    is_missing_command_prefix, menu_path_segments, merge_unavailable_menus, required_packages,
    supports_bulk_select, unavailable_from_menu_tree, unavailable_menus,
    unavailable_menus_for_device,
};
pub use features::interfaces::actions::INTERFACE_CREATE_TARGETS;
pub use features::interfaces::edit_resource_for_interface_type;
pub use forms::{
    BoxedFieldPredicate, EnumChoice, FieldKind, FieldPredicate, FieldRule, FieldSpec, FormSchema,
    FormSection, ScalarKind, accepts_constrained_number_char, accepts_number_char,
    default_writable_value, evaluate_field_rules, extra_status_fields, field_enabled,
    field_visible, form_mutation_body, join_ros_list, patch_body, prepare_lookup_options,
    preview_changes, split_ros_list, validate_form_values, with_leading_all, with_leading_none,
};
pub use resources::{
    ALL_RESOURCES, CatalogError, ColumnSpec, DASHBOARD_ID, FetchKind, NAVIGATION, NavGroup,
    NavItem, ResourceSpec, navigation_tree, resource_by_id, validate_active_catalog,
};
pub use routeros_version::{
    MIN_ROUTEROS_VERSION, RouterOsVersion, parse_routeros_version, routeros_meets_minimum,
    unsupported_routeros_copy,
};
pub use safe_mode::{
    SAFE_MODE_HISTORY_LIMIT, SAFE_MODE_HISTORY_WARN, SafeModeStatus, floating_undo_count,
    safe_mode_overflow_warning,
};
pub use system_write::{
    AT_CHAT_PROMPT, CERT_EXPORT_PROMPT, CERT_IMPORT_PROMPT, CERT_SIGN_PROMPT, CERTIFICATE_FORM,
    EXPORT_CONFIG_PROMPT, FORMAT_DISK_PROMPT, IMPORT_CONFIG_PROMPT, INSTALL_PACKAGE_PROMPT,
    LICENSE_IMPORT_PROMPT, RESET_CONFIG_PROMPT,
};
pub use theme::{
    ColorRgb, DefaultTheme, Palette, Theme, ThemeError, ThemeId, ThemeRegistry, ThemeSet,
};
pub use tools_write::{SMS_PROMPT, WOL_PROMPT};
