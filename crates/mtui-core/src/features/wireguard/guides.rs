//! Feature-owned operator guides for `WireGuard` screens.

use crate::about::ScreenGuide;

macro_rules! guide {
    ($id:literal, $summary:literal, $when:literal, $fields:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: None,
            },
        )
    };
    ($id:literal, $summary:literal, $when:literal, $fields:literal, $docs:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: Some($docs),
            },
        )
    };
}

pub(crate) static GUIDES: &[(&str, ScreenGuide)] = &[
    guide!(
        "wireguard",
        "WireGuard VPN interfaces: a simple, fast UDP tunnel using modern public-key \
         crypto. Each interface has a key pair and listen-port; peers are a separate table.",
        "Use it for site-to-site or road-warrior VPNs when both ends speak WireGuard. It is \
         not IPsec or OpenVPN. Private keys never leave this device.",
        "listen-port, private-key/public-key, mtu (often 1420), vrf (applies to the UDP \
         socket, not the wg interface itself).",
        "https://manual.mikrotik.com/docs/virtual-private-networks/wireguard/"
    ),
    guide!(
        "wireguard-peers",
        "Peers allowed to use a WireGuard interface. Identity is the remote public key; \
         allowed-address is the traffic that may traverse the tunnel for that peer.",
        "Add one peer per remote device or site. Endpoint is needed when this side must \
         initiate; omit it on a responder that only answers.",
        "interface, public-key, allowed-address, endpoint-address/port, preshared-key, \
         persistent-keepalive, responder. Client-* fields feed QR/export for mobile apps.",
        "https://manual.mikrotik.com/docs/virtual-private-networks/wireguard/"
    ),
];
