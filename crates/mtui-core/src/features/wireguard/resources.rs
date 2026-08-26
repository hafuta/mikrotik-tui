//! Feature-owned catalog entries for the `WireGuard` navigation group.

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

pub const WIREGUARD: ResourceSpec = ResourceSpec {
    id: "wireguard",
    group: "wireguard-group",
    cli_path: None,
    label: "WireGuard",
    fetch: FetchKind::List {
        endpoint: "/interface/wireguard",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("listen-port", "Listen", 8),
        col!("public-key", "Public key", 44),
        col!("private-key", "Private key", 12),
        col!("mtu", "MTU", 7),
        col!("vrf", "VRF", 12),
        col!("running", "Run", 5),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::wireguard::forms::WIREGUARD_FORM),
};

pub const WIREGUARD_PEERS: ResourceSpec = ResourceSpec {
    id: "wireguard-peers",
    group: "wireguard-group",
    cli_path: None,
    label: "WireGuard Peers",
    fetch: FetchKind::List {
        endpoint: "/interface/wireguard/peers",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("interface", "Interface", 16),
        col!("public-key", "Public key", 44),
        col!("endpoint-address", "Endpoint", 22),
        col!("endpoint-port", "Port", 8),
        col!("allowed-address", "Allowed", 28),
        col!("persistent-keepalive", "Keepalive", 10),
        col!("responder", "Responder", 10),
        col!("current-endpoint-address", "Current", 22),
        col!("current-endpoint-port", "Cur port", 9),
        col!("last-handshake", "Handshake", 14),
        col!("rx", "RX", 12),
        col!("tx", "TX", 12),
        col!("disabled", "Off", 5),
        col!("comment", "Comment", 28),
    ],
    refresh: Duration::from_secs(5),
    actions: crate::actions::MEMBER_ACTIONS,
    form: Some(&crate::features::wireguard::forms::WIREGUARD_PEER_FORM),
};

pub(crate) static RESOURCES: &[ResourceSpec] = &[WIREGUARD, WIREGUARD_PEERS];
