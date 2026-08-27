//! Feature-owned catalog entries for the `Tools` navigation group.

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
    NETWATCH,
    EMAIL,
    ROMON,
    ROMON_PORTS,
    GRAPHING,
    GRAPHING_INTERFACE,
    GRAPHING_QUEUE,
    GRAPHING_RESOURCE,
    PING,
    TRACEROUTE,
    SNIFFER,
    BANDWIDTH_TEST,
    FLOOD_PING,
    MAC_SCAN,
    IP_SCAN,
    PROFILER,
    WOL,
    SMS,
];

const NETWATCH: ResourceSpec = ResourceSpec {
    id: "netwatch",
    group: "tools-group",
    cli_path: None,
    label: "Netwatch",
    fetch: FetchKind::List {
        endpoint: "/tool/netwatch",
    },
    columns: &[
        col!("host", "Host", 22),
        col!("type", "Type", 10),
        col!("interval", "Interval", 10),
        col!("status", "Status", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::tools::forms::NETWATCH_FORM),
};

const EMAIL: ResourceSpec = ResourceSpec {
    id: "email",
    group: "tools-group",
    cli_path: None,
    label: "Email",
    fetch: FetchKind::System {
        endpoint: "/tool/e-mail",
    },
    columns: &[
        col!("server", "Server", 22),
        col!("from", "From", 24),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("tls", "TLS", 10),
        col!("port", "Port", 6),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::tools::forms::EMAIL_FORM),
};

const ROMON: ResourceSpec = ResourceSpec {
    id: "romon",
    group: "tools-group",
    cli_path: None,
    label: "RoMON",
    fetch: FetchKind::System {
        endpoint: "/tool/romon",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("id", "ID", 18),
        col!("secrets", "Secrets", 10),
        col!("current-id", "Current ID", 18),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::tools::forms::ROMON_FORM),
};

const ROMON_PORTS: ResourceSpec = ResourceSpec {
    id: "romon-ports",
    group: "tools-group",
    cli_path: None,
    label: "RoMON Ports",
    fetch: FetchKind::List {
        endpoint: "/tool/romon/port",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("forbid", "Forbid", 8),
        col!("cost", "Cost", 8),
        col!("secrets", "Secrets", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::tools::forms::ROMON_PORT_FORM),
};

const GRAPHING: ResourceSpec = ResourceSpec {
    id: "graphing",
    group: "tools-group",
    cli_path: None,
    label: "Graphing",
    fetch: FetchKind::System {
        endpoint: "/tool/graphing",
    },
    columns: &[
        col!("store-every", "Store Every", 12),
        col!("page-refresh", "Page Refresh", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::tools::forms::GRAPHING_FORM),
};

const GRAPHING_INTERFACE: ResourceSpec = ResourceSpec {
    id: "graphing-interface",
    group: "tools-group",
    cli_path: None,
    label: "Graphing Interface",
    fetch: FetchKind::List {
        endpoint: "/tool/graphing/interface",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("allow-address", "Allow Address", 18),
        col!("store-on-disk", "Store On Disk", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::tools::forms::GRAPHING_INTERFACE_FORM),
};

const GRAPHING_QUEUE: ResourceSpec = ResourceSpec {
    id: "graphing-queue",
    group: "tools-group",
    cli_path: None,
    label: "Graphing Queue",
    fetch: FetchKind::List {
        endpoint: "/tool/graphing/queue",
    },
    columns: &[
        col!("simple-queue", "Simple Queue", 16),
        col!("allow-address", "Allow Address", 18),
        col!("allow-target", "Allow Target", 12),
        col!("store-on-disk", "Store On Disk", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::tools::forms::GRAPHING_QUEUE_FORM),
};

const GRAPHING_RESOURCE: ResourceSpec = ResourceSpec {
    id: "graphing-resource",
    group: "tools-group",
    cli_path: None,
    label: "Graphing Resource",
    fetch: FetchKind::List {
        endpoint: "/tool/graphing/resource",
    },
    columns: &[
        col!("allow-address", "Allow Address", 18),
        col!("store-on-disk", "Store On Disk", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::tools::forms::GRAPHING_RESOURCE_FORM),
};

const PING: ResourceSpec = ResourceSpec {
    id: "ping",
    group: "tools-group",
    cli_path: Some("/tool/ping"),
    label: "Ping",
    fetch: FetchKind::Local,
    columns: &[
        col!("seq", "Seq", 6),
        col!("host", "Host", 22),
        col!("time", "Time", 12),
        col!("ttl", "TTL", 6),
        col!("status", "Status", 12),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::PING_ACTIONS,
    form: None,
};

const TRACEROUTE: ResourceSpec = ResourceSpec {
    id: "traceroute",
    group: "tools-group",
    cli_path: Some("/tool/traceroute"),
    label: "Traceroute",
    fetch: FetchKind::Local,
    columns: &[
        col!("hop", "Hop", 6),
        col!("address", "Address", 22),
        col!("status", "Status", 12),
        col!("time", "Time", 12),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::TRACEROUTE_ACTIONS,
    form: None,
};

const SNIFFER: ResourceSpec = ResourceSpec {
    id: "sniffer",
    group: "tools-group",
    cli_path: None,
    label: "Packet Sniffer",
    fetch: FetchKind::System {
        endpoint: "/tool/sniffer",
    },
    columns: &[
        col!("interface", "Interface", 16),
        col!("file-name", "File", 24),
        col!("running", "Run", 5),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::SNIFFER_ACTIONS,
    form: Some(&crate::features::tools::forms::SNIFFER_FORM),
};

const BANDWIDTH_TEST: ResourceSpec = ResourceSpec {
    id: "bandwidth-test",
    group: "tools-group",
    cli_path: Some("/tool/bandwidth-test"),
    label: "Bandwidth Test",
    fetch: FetchKind::Local,
    columns: &[
        col!("address", "Address", 22),
        col!("tx-current", "TX", 12),
        col!("rx-current", "RX", 12),
        col!("status", "Status", 12),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::BANDWIDTH_ACTIONS,
    form: None,
};

const FLOOD_PING: ResourceSpec = ResourceSpec {
    id: "flood-ping",
    group: "tools-group",
    cli_path: Some("/tool/flood-ping"),
    label: "Flood Ping",
    fetch: FetchKind::Local,
    columns: &[
        col!("address", "Address", 22),
        col!("sent", "Sent", 8),
        col!("received", "Received", 10),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::FLOOD_PING_ACTIONS,
    form: None,
};

const MAC_SCAN: ResourceSpec = ResourceSpec {
    id: "mac-scan",
    group: "tools-group",
    cli_path: Some("/tool/mac-scan"),
    label: "MAC Scan",
    fetch: FetchKind::Local,
    columns: &[
        col!("address", "Address", 18),
        col!("mac-address", "MAC", 18),
        col!("age", "Age", 8),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::MAC_SCAN_ACTIONS,
    form: None,
};

const IP_SCAN: ResourceSpec = ResourceSpec {
    id: "ip-scan",
    group: "tools-group",
    cli_path: Some("/tool/ip-scan"),
    label: "IP Scan",
    fetch: FetchKind::Local,
    columns: &[
        col!("address", "Address", 18),
        col!("mac-address", "MAC", 18),
        col!("time", "Time", 8),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::IP_SCAN_ACTIONS,
    form: None,
};

const PROFILER: ResourceSpec = ResourceSpec {
    id: "profiler",
    group: "tools-group",
    cli_path: Some("/tool/profile"),
    label: "Profiler",
    fetch: FetchKind::Local,
    columns: &[
        col!("name", "Name", 24),
        col!("usage", "Usage", 10),
        col!("load", "Load", 8),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::PROFILER_ACTIONS,
    form: None,
};

const WOL: ResourceSpec = ResourceSpec {
    id: "wol",
    group: "tools-group",
    cli_path: Some("/tool/wol"),
    label: "Wake on LAN",
    fetch: FetchKind::Local,
    columns: &[col!("interface", "Interface", 16), col!("mac", "MAC", 18)],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::WOL_ACTIONS,
    form: None,
};

const SMS: ResourceSpec = ResourceSpec {
    id: "sms",
    group: "tools-group",
    cli_path: Some("/tool/sms"),
    label: "SMS",
    fetch: FetchKind::Local,
    columns: &[
        col!("phone-number", "Phone", 16),
        col!("message", "Message", 36),
    ],
    refresh: Duration::from_secs(3600),
    actions: crate::actions::SMS_ACTIONS,
    form: None,
};
