//! Feature-owned catalog entries for the Files navigation group.

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

pub(crate) static RESOURCES: &[ResourceSpec] = &[FILES];

const FILES: ResourceSpec = ResourceSpec {
    id: "files",
    group: "files-group",
    cli_path: Some("/file"),
    label: "Files",
    fetch: FetchKind::List { endpoint: "/file" },
    columns: &[
        col!("name", "Name", 40),
        col!("type", "Type", 12),
        col!("size", "Size", 12),
        col!("creation-time", "Created", 20),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::FILE_ACTIONS,
    form: None,
};
