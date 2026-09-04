//! Feature-owned catalog entries for the `Routing` navigation group.

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
    ROUTING_TABLES,
    ROUTING_RULES,
    OSPF_INSTANCES,
    OSPF_AREAS,
    OSPF_INTERFACE_TEMPLATES,
    OSPF_INTERFACES,
    BGP_CONNECTIONS,
    BGP_SESSIONS,
    BGP_TEMPLATES,
    RIP_INSTANCES,
    RIP_INTERFACE_TEMPLATES,
    BFD,
    ROUTING_FILTERS,
    ROUTING_ID,
    OSPF_NEIGHBORS,
    OSPF_LSA,
    BGP_ADVERTISEMENTS,
];

const ROUTING_TABLES: ResourceSpec = ResourceSpec {
    id: "routing-tables",
    group: "routing-group",
    cli_path: None,
    label: "Tables",
    fetch: FetchKind::List {
        endpoint: "/routing/table",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("fib", "FIB", 5),
        col!("dynamic", "Dyn", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::routing::forms::ROUTING_TABLE_FORM),
};

const ROUTING_RULES: ResourceSpec = ResourceSpec {
    id: "routing-rules",
    group: "routing-group",
    cli_path: None,
    label: "Rules",
    fetch: FetchKind::List {
        endpoint: "/routing/rule",
    },
    columns: &[
        col!("src-address", "Source", 20),
        col!("dst-address", "Destination", 20),
        col!("routing-mark", "Mark", 14),
        col!("action", "Action", 12),
        col!("table", "Table", 14),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::ROUTING_RULE_FORM),
};

const OSPF_INSTANCES: ResourceSpec = ResourceSpec {
    id: "ospf-instances",
    group: "routing-group",
    cli_path: None,
    label: "OSPF",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/instance",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("version", "Ver", 5),
        col!("router-id", "Router ID", 16),
        col!("originate-default", "Default", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::OSPF_INSTANCE_FORM),
};

const OSPF_AREAS: ResourceSpec = ResourceSpec {
    id: "ospf-areas",
    group: "routing-group",
    cli_path: None,
    label: "OSPF Areas",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/area",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("instance", "Instance", 16),
        col!("area-id", "Area ID", 16),
        col!("type", "Type", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::OSPF_AREA_FORM),
};

const OSPF_INTERFACE_TEMPLATES: ResourceSpec = ResourceSpec {
    id: "ospf-interface-templates",
    group: "routing-group",
    cli_path: None,
    label: "OSPF Interface Templates",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/interface-template",
    },
    columns: &[
        col!("instance", "Instance", 16),
        col!("area", "Area", 16),
        col!("interfaces", "Interfaces", 24),
        col!("type", "Type", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::OSPF_INTERFACE_TEMPLATE_FORM),
};

const OSPF_INTERFACES: ResourceSpec = ResourceSpec {
    id: "ospf-interfaces",
    group: "routing-group",
    cli_path: None,
    label: "OSPF Interface",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/interface",
    },
    columns: &[
        col!("address", "Address", 22),
        col!("area", "Area", 12),
        col!("state", "State", 10),
        col!("network-type", "Type", 12),
        col!("cost", "Cost", 6),
        col!("dr", "DR", 16),
        col!("bdr", "BDR", 16),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::routing::forms::OSPF_INTERFACE_FORM),
};

const BGP_CONNECTIONS: ResourceSpec = ResourceSpec {
    id: "bgp-connections",
    group: "routing-group",
    cli_path: None,
    label: "BGP",
    fetch: FetchKind::List {
        endpoint: "/routing/bgp/connection",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("remote.address", "Remote", 18),
        col!("remote.as", "Remote AS", 10),
        col!("local.role", "Role", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(10),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::BGP_CONNECTION_FORM),
};

const BGP_SESSIONS: ResourceSpec = ResourceSpec {
    id: "bgp-sessions",
    group: "routing-group",
    cli_path: None,
    label: "BGP Sessions",
    fetch: FetchKind::List {
        endpoint: "/routing/bgp/session",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("remote.address", "Remote", 18),
        col!("remote.as", "Remote AS", 10),
        col!("established", "Est", 5),
        col!("uptime", "Uptime", 12),
        col!("prefix-count", "Prefixes", 10),
        col!("ebgp", "eBGP", 6),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::routing::forms::BGP_SESSION_FORM),
};

const BGP_TEMPLATES: ResourceSpec = ResourceSpec {
    id: "bgp-templates",
    group: "routing-group",
    cli_path: None,
    label: "BGP Templates",
    fetch: FetchKind::List {
        endpoint: "/routing/bgp/template",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("as", "AS", 10),
        col!("router-id", "Router ID", 16),
        col!("address-families", "Families", 16),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::BGP_TEMPLATE_FORM),
};

const RIP_INSTANCES: ResourceSpec = ResourceSpec {
    id: "rip-instances",
    group: "routing-group",
    cli_path: None,
    label: "RIP",
    fetch: FetchKind::List {
        endpoint: "/routing/rip/instance",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("vrf", "VRF", 12),
        col!("originate-default", "Default", 8),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::routing::forms::RIP_INSTANCE_FORM),
};

const RIP_INTERFACE_TEMPLATES: ResourceSpec = ResourceSpec {
    id: "rip-interface-templates",
    group: "routing-group",
    cli_path: None,
    label: "RIP Interfaces",
    fetch: FetchKind::List {
        endpoint: "/routing/rip/interface-template",
    },
    columns: &[
        col!("instance", "Instance", 16),
        col!("interfaces", "Interfaces", 24),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::RIP_INTERFACE_TEMPLATE_FORM),
};

const BFD: ResourceSpec = ResourceSpec {
    id: "bfd",
    group: "routing-group",
    cli_path: None,
    label: "BFD",
    fetch: FetchKind::List {
        endpoint: "/routing/bfd/configuration",
    },
    columns: &[
        col!("interfaces", "Interfaces", 24),
        col!("addresses", "Addresses", 20),
        col!("min-tx-interval", "TX", 10),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::routing::forms::BFD_CONFIGURATION_FORM),
};

const ROUTING_FILTERS: ResourceSpec = ResourceSpec {
    id: "routing-filters",
    group: "routing-group",
    cli_path: None,
    label: "Filters",
    fetch: FetchKind::List {
        endpoint: "/routing/filter/rule",
    },
    columns: &[
        col!("chain", "Chain", 16),
        col!("rule", "Rule", 40),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::FILTER_ACTIONS,
    form: Some(&crate::features::routing::forms::ROUTING_FILTER_FORM),
};

const ROUTING_ID: ResourceSpec = ResourceSpec {
    id: "routing-id",
    group: "routing-group",
    cli_path: None,
    label: "Router ID",
    fetch: FetchKind::List {
        endpoint: "/routing/id",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("id", "ID", 16),
        col!("select", "Select", 16),
        col!("disabled", "Off", 5),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::routing::forms::ROUTING_ID_FORM),
};

const OSPF_NEIGHBORS: ResourceSpec = ResourceSpec {
    id: "ospf-neighbors",
    group: "routing-group",
    cli_path: None,
    label: "OSPF Neighbors",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/neighbor",
    },
    columns: &[
        col!("instance", "Instance", 16),
        col!("router-id", "Router ID", 16),
        col!("address", "Address", 18),
        col!("state", "State", 12),
        col!("adjacency", "Adjacency", 12),
    ],
    refresh: Duration::from_secs(5),
    actions: &[],
    form: None,
};

const OSPF_LSA: ResourceSpec = ResourceSpec {
    id: "ospf-lsa",
    group: "routing-group",
    cli_path: None,
    label: "OSPF LSA",
    fetch: FetchKind::List {
        endpoint: "/routing/ospf/lsa",
    },
    columns: &[
        col!("type", "Type", 12),
        col!("id", "ID", 16),
        col!("originator", "Originator", 16),
        col!("area", "Area", 12),
        col!("sequence", "Seq", 10),
    ],
    refresh: Duration::from_secs(10),
    actions: &[],
    form: None,
};

const BGP_ADVERTISEMENTS: ResourceSpec = ResourceSpec {
    id: "bgp-advertisements",
    group: "routing-group",
    cli_path: None,
    label: "BGP Advertisements",
    fetch: FetchKind::List {
        endpoint: "/routing/bgp/advertisements",
    },
    columns: &[
        col!("prefix", "Prefix", 24),
        col!("nexthop", "Nexthop", 18),
        col!("peer", "Peer", 16),
        col!("as-path", "AS path", 24),
    ],
    refresh: Duration::from_secs(5),
    actions: &[],
    form: None,
};
