//! Feature-owned catalog entries for the `Container` navigation group.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        crate::resources::ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    CONTAINERS,
    CONTAINER_CONFIG,
    CONTAINER_ENVS,
    CONTAINER_MOUNTS,
    APPS,
];

const CONTAINERS: ResourceSpec = ResourceSpec {
    id: "containers",
    group: "container-group",
    cli_path: None,
    label: "Containers",
    fetch: FetchKind::List {
        endpoint: "/container",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("tag", "Tag", 22),
        col!("interface", "Interface", 14),
        col!("status", "Status", 14),
        col!("arch", "Arch", 8),
        col!("memory-current", "Memory", 12),
        col!("cpu-usage", "CPU", 8),
        col!("root-dir", "Root dir", 24),
        col!("start-on-boot", "Boot", 6),
        col!("logging", "Log", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::CONTAINER_ACTIONS,
    form: Some(&crate::features::container::forms::CONTAINER_FORM),
};

const CONTAINER_CONFIG: ResourceSpec = ResourceSpec {
    id: "container-config",
    group: "container-group",
    cli_path: None,
    label: "Config",
    fetch: FetchKind::System {
        endpoint: "/container/config",
    },
    columns: &[
        col!("registry-url", "Registry", 28),
        col!("tmpdir", "Tmpdir", 20),
        col!("username", "User", 12),
        col!("memory-current", "Memory", 12),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::container::forms::CONTAINER_CONFIG_FORM),
};

const CONTAINER_ENVS: ResourceSpec = ResourceSpec {
    id: "container-envs",
    group: "container-group",
    cli_path: None,
    label: "Envs",
    fetch: FetchKind::List {
        endpoint: "/container/envs",
    },
    columns: &[
        col!("list", "List", 16),
        col!("key", "Key", 20),
        col!("value", "Value", 24),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::container::forms::CONTAINER_ENV_FORM),
};

const CONTAINER_MOUNTS: ResourceSpec = ResourceSpec {
    id: "container-mounts",
    group: "container-group",
    cli_path: None,
    label: "Mounts",
    fetch: FetchKind::List {
        endpoint: "/container/mounts",
    },
    columns: &[
        col!("list", "List", 16),
        col!("src", "Src", 24),
        col!("dst", "Dst", 24),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::container::forms::CONTAINER_MOUNT_FORM),
};

const APPS: ResourceSpec = ResourceSpec {
    id: "apps",
    group: "container-group",
    cli_path: None,
    label: "Apps",
    fetch: FetchKind::List { endpoint: "/app" },
    columns: &[
        col!("name", "Name", 18),
        col!("status", "Status", 16),
        col!("running", "Run", 5),
        col!("network", "Network", 12),
        col!("ui-url", "UI URL", 28),
        col!("ip-address", "IP", 18),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::APP_ACTIONS,
    form: Some(&crate::features::container::forms::APP_FORM),
};
