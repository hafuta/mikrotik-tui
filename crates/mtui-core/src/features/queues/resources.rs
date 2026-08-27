//! Feature-owned catalog entries for the Queues navigation group.

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

pub(crate) static RESOURCES: &[ResourceSpec] =
    &[QUEUE_SIMPLE, QUEUE_TREE, QUEUE_TYPE, QUEUE_INTERFACE];

const QUEUE_SIMPLE: ResourceSpec = ResourceSpec {
    id: "queue-simple",
    group: "queue-group",
    cli_path: None,
    label: "Simple",
    fetch: FetchKind::List {
        endpoint: "/queue/simple",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("target", "Target", 24),
        col!("max-limit", "Max limit", 18),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::queues::forms::QUEUE_SIMPLE_FORM),
};

const QUEUE_TREE: ResourceSpec = ResourceSpec {
    id: "queue-tree",
    group: "queue-group",
    cli_path: None,
    label: "Tree",
    fetch: FetchKind::List {
        endpoint: "/queue/tree",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("parent", "Parent", 16),
        col!("packet-mark", "Mark", 16),
        col!("max-limit", "Max limit", 14),
        col!("priority", "Prio", 6),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::queues::forms::QUEUE_TREE_FORM),
};

const QUEUE_TYPE: ResourceSpec = ResourceSpec {
    id: "queue-type",
    group: "queue-group",
    cli_path: None,
    label: "Queue Type",
    fetch: FetchKind::List {
        endpoint: "/queue/type",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("kind", "Kind", 12),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::queues::forms::QUEUE_TYPE_FORM),
};

const QUEUE_INTERFACE: ResourceSpec = ResourceSpec {
    id: "queue-interface",
    group: "queue-group",
    cli_path: None,
    label: "Interface",
    fetch: FetchKind::List {
        endpoint: "/queue/interface",
    },
    columns: &[
        col!("interface", "Interface", 18),
        col!("queue", "Queue", 18),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::HARDWARE_EDIT_ACTIONS,
    form: Some(&crate::features::queues::forms::QUEUE_INTERFACE_FORM),
};
