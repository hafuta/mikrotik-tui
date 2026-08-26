//! Feature-owned catalog entries for the complete PPP navigation group.

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

pub const PPP_SECRETS: ResourceSpec = ResourceSpec {
    id: "ppp-secrets",
    group: "ppp-group",
    cli_path: None,
    label: "Secrets",
    fetch: FetchKind::List {
        endpoint: "/ppp/secret",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("service", "Service", 10),
        col!("profile", "Profile", 18),
        col!("caller-id", "Caller", 18),
        col!("local-address", "Local", 18),
        col!("remote-address", "Remote", 18),
        col!("remote-ipv6-prefix", "IPv6 prefix", 22),
        col!("password", "Password", 10),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPP_SECRET_FORM),
};

pub const PPP_PROFILES: ResourceSpec = ResourceSpec {
    id: "ppp-profiles",
    group: "ppp-group",
    cli_path: None,
    label: "Profiles",
    fetch: FetchKind::List {
        endpoint: "/ppp/profile",
    },
    columns: &[
        col!("name", "Name", 20),
        col!("local-address", "Local", 18),
        col!("remote-address", "Remote", 18),
        col!("dns-server", "DNS", 24),
        col!("rate-limit", "Rate limit", 18),
        col!("only-one", "Only one", 9),
        col!("use-encryption", "Encrypt", 9),
        col!("use-compression", "Compress", 9),
        col!("change-tcp-mss", "MSS", 8),
        col!("bridge", "Bridge", 16),
        col!("interface-list", "List", 16),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::LIST_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPP_PROFILE_FORM),
};

pub const PPP_ACTIVE: ResourceSpec = ResourceSpec {
    id: "ppp-active",
    group: "ppp-group",
    cli_path: None,
    label: "Active",
    fetch: FetchKind::List {
        endpoint: "/ppp/active",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("service", "Service", 10),
        col!("caller-id", "Caller", 18),
        col!("address", "Address", 18),
        col!("uptime", "Uptime", 12),
        col!("encoding", "Encoding", 16),
        col!("session-id", "Session", 12),
        col!("limit-bytes-in", "Limit in", 12),
        col!("limit-bytes-out", "Limit out", 12),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const PPP_AAA: ResourceSpec = ResourceSpec {
    id: "ppp-aaa",
    group: "ppp-group",
    cli_path: None,
    label: "AAA",
    fetch: FetchKind::System {
        endpoint: "/ppp/aaa",
    },
    columns: &[
        col!("use-radius", "RADIUS", 8),
        col!("accounting", "Accounting", 11),
        col!("interim-update", "Interim", 10),
        col!("enable-ipv6-accounting", "IPv6 acct", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPP_AAA_FORM),
};

pub const PPP_CLIENT: ResourceSpec = ResourceSpec {
    id: "ppp-client",
    group: "ppp-group",
    cli_path: None,
    label: "PPP Client",
    fetch: FetchKind::List {
        endpoint: "/interface/ppp-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("port", "Port", 14),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("profile", "Profile", 16),
        col!("phone", "Phone", 16),
        col!("add-default-route", "Default", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPP_CLIENT_FORM),
};

pub const PPPOE_CLIENTS: ResourceSpec = ResourceSpec {
    id: "pppoe-clients",
    group: "ppp-group",
    cli_path: None,
    label: "PPPoE Clients",
    fetch: FetchKind::List {
        endpoint: "/interface/pppoe-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("service-name", "Service", 16),
        col!("ac-name", "AC name", 16),
        col!("profile", "Profile", 16),
        col!("add-default-route", "Default", 8),
        col!("use-peer-dns", "Peer DNS", 9),
        col!("status", "Status", 12),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPPOE_CLIENT_FORM),
};

pub const PPPOE_SERVERS: ResourceSpec = ResourceSpec {
    id: "pppoe-servers",
    group: "ppp-group",
    cli_path: None,
    label: "PPPoE Servers",
    fetch: FetchKind::List {
        endpoint: "/interface/pppoe-server/server",
    },
    columns: &[
        col!("service-name", "Service", 16),
        col!("interface", "Interface", 16),
        col!("default-profile", "Profile", 16),
        col!("authentication", "Auth", 16),
        col!("max-mtu", "Max MTU", 8),
        col!("max-mru", "Max MRU", 8),
        col!("one-session-per-host", "One sess", 9),
        col!("max-sessions", "Max sess", 9),
        col!("keepalive-timeout", "Keepalive", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(15),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPPOE_SERVER_FORM),
};

pub const PPPOE_SERVER_IFACES: ResourceSpec = ResourceSpec {
    id: "pppoe-server-ifaces",
    group: "ppp-group",
    cli_path: None,
    label: "PPPoE Sessions",
    fetch: FetchKind::List {
        endpoint: "/interface/pppoe-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("user", "User", 18),
        col!("service-name", "Service", 16),
        col!("interface", "Interface", 16),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const PPTP_CLIENT: ResourceSpec = ResourceSpec {
    id: "pptp-client",
    group: "ppp-group",
    cli_path: None,
    label: "PPTP Clients",
    fetch: FetchKind::List {
        endpoint: "/interface/pptp-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("connect-to", "Connect to", 22),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("profile", "Profile", 16),
        col!("add-default-route", "Default", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPTP_CLIENT_FORM),
};

pub const PPTP_SERVER_IFACES: ResourceSpec = ResourceSpec {
    id: "pptp-server-ifaces",
    group: "ppp-group",
    cli_path: None,
    label: "PPTP Sessions",
    fetch: FetchKind::List {
        endpoint: "/interface/pptp-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("user", "User", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const PPTP_SERVER: ResourceSpec = ResourceSpec {
    id: "pptp-server",
    group: "ppp-group",
    cli_path: None,
    label: "PPTP Server",
    fetch: FetchKind::System {
        endpoint: "/interface/pptp-server/server",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("default-profile", "Profile", 16),
        col!("authentication", "Auth", 16),
        col!("keepalive-timeout", "Keepalive", 12),
        col!("max-mtu", "Max MTU", 8),
        col!("max-mru", "Max MRU", 8),
        col!("mrru", "MRRU", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ppp::forms::PPTP_SERVER_FORM),
};

pub const L2TP_CLIENT: ResourceSpec = ResourceSpec {
    id: "l2tp-client",
    group: "ppp-group",
    cli_path: None,
    label: "L2TP Clients",
    fetch: FetchKind::List {
        endpoint: "/interface/l2tp-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("connect-to", "Connect to", 22),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("profile", "Profile", 16),
        col!("use-ipsec", "IPsec", 8),
        col!("ipsec-secret", "IPsec secret", 14),
        col!("add-default-route", "Default", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::L2TP_CLIENT_FORM),
};

pub const L2TP_SERVER_IFACES: ResourceSpec = ResourceSpec {
    id: "l2tp-server-ifaces",
    group: "ppp-group",
    cli_path: None,
    label: "L2TP Sessions",
    fetch: FetchKind::List {
        endpoint: "/interface/l2tp-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("user", "User", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const L2TP_SERVER: ResourceSpec = ResourceSpec {
    id: "l2tp-server",
    group: "ppp-group",
    cli_path: None,
    label: "L2TP Server",
    fetch: FetchKind::System {
        endpoint: "/interface/l2tp-server/server",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("default-profile", "Profile", 16),
        col!("authentication", "Auth", 16),
        col!("use-ipsec", "IPsec", 8),
        col!("ipsec-secret", "IPsec secret", 14),
        col!("keepalive-timeout", "Keepalive", 12),
        col!("max-mtu", "Max MTU", 8),
        col!("max-mru", "Max MRU", 8),
        col!("allow-fast-path", "Fast path", 10),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ppp::forms::L2TP_SERVER_FORM),
};

pub const SSTP_CLIENT: ResourceSpec = ResourceSpec {
    id: "sstp-client",
    group: "ppp-group",
    cli_path: None,
    label: "SSTP Clients",
    fetch: FetchKind::List {
        endpoint: "/interface/sstp-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("connect-to", "Connect to", 22),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("profile", "Profile", 16),
        col!("certificate", "Certificate", 18),
        col!("verify-server-certificate", "Verify", 8),
        col!("add-default-route", "Default", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::SSTP_CLIENT_FORM),
};

pub const SSTP_SERVER_IFACES: ResourceSpec = ResourceSpec {
    id: "sstp-server-ifaces",
    group: "ppp-group",
    cli_path: None,
    label: "SSTP Sessions",
    fetch: FetchKind::List {
        endpoint: "/interface/sstp-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("user", "User", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const SSTP_SERVER: ResourceSpec = ResourceSpec {
    id: "sstp-server",
    group: "ppp-group",
    cli_path: None,
    label: "SSTP Server",
    fetch: FetchKind::System {
        endpoint: "/interface/sstp-server/server",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("certificate", "Certificate", 18),
        col!("default-profile", "Profile", 16),
        col!("authentication", "Auth", 16),
        col!("port", "Port", 8),
        col!("verify-client-certificate", "Verify", 8),
        col!("keepalive-timeout", "Keepalive", 12),
        col!("max-mtu", "Max MTU", 8),
        col!("max-mru", "Max MRU", 8),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ppp::forms::SSTP_SERVER_FORM),
};

pub const OVPN_CLIENT: ResourceSpec = ResourceSpec {
    id: "ovpn-client",
    group: "ppp-group",
    cli_path: None,
    label: "OpenVPN Clients",
    fetch: FetchKind::List {
        endpoint: "/interface/ovpn-client",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("connect-to", "Connect to", 22),
        col!("port", "Port", 8),
        col!("mode", "Mode", 10),
        col!("user", "User", 18),
        col!("password", "Password", 10),
        col!("profile", "Profile", 16),
        col!("certificate", "Certificate", 18),
        col!("cipher", "Cipher", 14),
        col!("auth", "Auth", 10),
        col!("add-default-route", "Default", 8),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::ppp::forms::OVPN_CLIENT_FORM),
};

pub const OVPN_SERVER_IFACES: ResourceSpec = ResourceSpec {
    id: "ovpn-server-ifaces",
    group: "ppp-group",
    cli_path: None,
    label: "OpenVPN Sessions",
    fetch: FetchKind::List {
        endpoint: "/interface/ovpn-server",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("user", "User", 18),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::DISCONNECT_ACTIONS,
    form: None,
};

pub const OVPN_SERVER: ResourceSpec = ResourceSpec {
    id: "ovpn-server",
    group: "ppp-group",
    cli_path: None,
    label: "OpenVPN Server",
    fetch: FetchKind::System {
        endpoint: "/interface/ovpn-server/server",
    },
    columns: &[
        col!("enabled", "Enabled", 8),
        col!("port", "Port", 8),
        col!("mode", "Mode", 10),
        col!("netmask", "Netmask", 8),
        col!("certificate", "Certificate", 18),
        col!("default-profile", "Profile", 16),
        col!("auth", "Auth", 16),
        col!("cipher", "Cipher", 16),
        col!("require-client-certificate", "Client cert", 12),
    ],
    refresh: Duration::from_secs(30),
    actions: crate::actions::SINGLETON_EDIT_ACTIONS,
    form: Some(&crate::features::ppp::forms::OVPN_SERVER_FORM),
};

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    PPP_SECRETS,
    PPP_PROFILES,
    PPP_ACTIVE,
    PPP_AAA,
    PPP_CLIENT,
    PPPOE_CLIENTS,
    PPPOE_SERVERS,
    PPPOE_SERVER_IFACES,
    PPTP_CLIENT,
    PPTP_SERVER_IFACES,
    PPTP_SERVER,
    L2TP_CLIENT,
    L2TP_SERVER_IFACES,
    L2TP_SERVER,
    SSTP_CLIENT,
    SSTP_SERVER_IFACES,
    SSTP_SERVER,
    OVPN_CLIENT,
    OVPN_SERVER_IFACES,
    OVPN_SERVER,
];
