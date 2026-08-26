//! Feature-owned operator guides for PPP screens.

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
        "ppp-secrets",
        "PPP user database: usernames used by PPPoE, PPTP, L2TP, SSTP, and similar servers. \
         Passwords are secrets and stay masked in this client.",
        "Add a secret per customer or incoming VPN user. Profiles supply the bulk of IP, \
         rate-limit, and DNS settings.",
        "name, password, service, profile, local/remote-address, routes, disabled."
    ),
    guide!(
        "ppp-profiles",
        "Shared PPP session settings: addresses, DNS, rate-limits, bridges, and encryption \
         defaults applied to secrets and clients.",
        "Create profiles for staff vs customers vs VPN instead of repeating options on every \
         secret.",
        "local/remote-address, dns-server, rate-limit, session-timeout, incoming/outgoing \
         filters, use-encryption, bridge."
    ),
    guide!(
        "ppp-active",
        "Currently connected PPP sessions (PPPoE, L2TP, …). This is runtime state, not a \
         configuration list.",
        "Inspect who is online, addresses, and uptime. Disconnect is a session action; it \
         does not delete the secret.",
        "name, service, caller-id, address, uptime, encoding. Typically no property sheet."
    ),
    guide!(
        "ppp-aaa",
        "PPP authentication, authorization, and accounting: whether to use local secrets, \
         RADIUS, or both.",
        "Point incoming PPP at RADIUS when User Manager or an external AAA server owns the \
         users.",
        "use-radius, accounting, interim-update, and related AAA toggles."
    ),
    guide!(
        "ppp-client",
        "Generic PPP client (serial/async or similar). Most WAN links use PPPoE Client \
         instead.",
        "Use it for analog/serial PPP or uncommon client modes, not typical Ethernet PPPoE.",
        "port, user, password, profile, add-default-route, dial-on-demand."
    ),
    guide!(
        "pppoe-clients",
        "PPPoE client interfaces: the usual ISP last-mile session over Ethernet.",
        "Create one per WAN when the ISP authenticates with PPPoE. Needs user/password and \
         the Ethernet (or VLAN) facing the ISP.",
        "interface, user, password, add-default-route, use-peer-dns, profile, ac-name/service."
    ),
    guide!(
        "pppoe-servers",
        "PPPoE server instances that accept customer sessions on an interface.",
        "Use it when this router is the ISP concentrator. Secrets/RADIUS decide who may \
         connect.",
        "interface, service-name, default-profile, max-mtu/mru, authentication methods."
    ),
    guide!(
        "pppoe-server-ifaces",
        "Per-interface PPPoE server bindings (which ports run the server).",
        "Attach the server to the customer-facing Ethernet or VLAN.",
        "interface and the server/service it belongs to."
    ),
    guide!(
        "pptp-client",
        "PPTP VPN client. The protocol is obsolete and insecure by modern standards.",
        "Only for legacy peers that cannot do L2TP, SSTP, WireGuard, or IPsec.",
        "connect-to, user, password, profile, add-default-route."
    ),
    guide!(
        "pptp-server-ifaces",
        "Interfaces where the PPTP server listens.",
        "Legacy PPTP access concentrator bindings.",
        "interface assignment for the PPTP server."
    ),
    guide!(
        "pptp-server",
        "Global PPTP server settings (enable, default profile, authentication).",
        "Leave disabled unless you must terminate PPTP. Prefer WireGuard, IPsec, or L2TP.",
        "enabled, default-profile, authentication, keepalive-timeout."
    ),
    guide!(
        "l2tp-client",
        "L2TP client, often combined with IPsec (L2TP/IPsec) for site or road-warrior VPNs.",
        "Use it to join a remote L2TP concentrator. Pair with IPsec when the peer requires it.",
        "connect-to, user, password, profile, use-ipsec/ipsec-secret, add-default-route."
    ),
    guide!(
        "l2tp-server-ifaces",
        "Interfaces associated with the L2TP server.",
        "Bind L2TP service to specific interfaces when not listening globally.",
        "interface membership for the L2TP server."
    ),
    guide!(
        "l2tp-server",
        "Global L2TP server: enable incoming L2TP (optionally with IPsec).",
        "Use it to terminate remote L2TP clients. Secrets or RADIUS authenticate users.",
        "enabled, default-profile, use-ipsec, ipsec-secret, authentication, keepalive."
    ),
    guide!(
        "sstp-client",
        "SSTP client: PPP inside TLS, typically to a Windows or RouterOS SSTP server.",
        "Use it when the path must look like HTTPS (TCP 443) to pass strict firewalls.",
        "connect-to, user, password, proxy, certificates, add-default-route."
    ),
    guide!(
        "sstp-server-ifaces",
        "Interfaces for the SSTP server.",
        "Bind SSTP where TLS should terminate.",
        "interface assignment for SSTP."
    ),
    guide!(
        "sstp-server",
        "Global SSTP server (TLS-wrapped PPP).",
        "Terminate SSTP clients with a certificate. Often used instead of PPTP through NAT.",
        "enabled, certificate, default-profile, authentication, port (usually 443)."
    ),
    guide!(
        "ovpn-client",
        "OpenVPN client. RouterOS supports a subset of OpenVPN features (mode, auth, ciphers).",
        "Use it to connect to an OpenVPN server. WireGuard is usually simpler if both ends \
         can run it.",
        "connect-to, port, mode, user/password or certificates, profile, cipher, auth."
    ),
    guide!(
        "ovpn-server-ifaces",
        "Interfaces for the OpenVPN server.",
        "Bind OpenVPN to a local interface when required.",
        "interface assignment for OpenVPN."
    ),
    guide!(
        "ovpn-server",
        "Global OpenVPN server settings.",
        "Terminate OpenVPN clients. Check mode (ip/ethernet) and certificates against the \
         client config.",
        "enabled, port, mode, certificate, default-profile, auth, cipher."
    ),
];
