//! Feature-owned catalog entries for the RADIUS navigation group.

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

pub(crate) static RESOURCES: &[ResourceSpec] = &[RADIUS, RADIUS_INCOMING];

const RADIUS: ResourceSpec = ResourceSpec {
    id: "radius",
    group: "radius-group",
    cli_path: None,
    label: "RADIUS",
    fetch: FetchKind::List {
        endpoint: "/radius",
    },
    columns: &[
        col!("address", "Address", 18),
        col!("protocol", "Proto", 8),
        col!("secret", "Secret", 10),
        col!("service", "Service", 16),
        col!("timeout", "Timeout", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::radius::forms::RADIUS_FORM),
};

const RADIUS_INCOMING: ResourceSpec = ResourceSpec {
    id: "radius-incoming",
    group: "radius-group",
    cli_path: None,
    label: "Incoming",
    fetch: FetchKind::System {
        endpoint: "/radius/incoming",
    },
    columns: &[col!("accept", "Accept", 8), col!("port", "Port", 6)],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::radius::forms::RADIUS_INCOMING_FORM),
};
