//! On-demand screen guides for `RouterOS` menus.
//!
//! Copy is original wording aligned with
//! <https://manual.mikrotik.com/docs/>. Property tables are not reproduced.
//! Every catalog id must have an entry; the CLI reference URL is derived from
//! the resource path so field names stay tied to what `RouterOS` exposes.

use crate::resources::{DASHBOARD_ID, resource_by_id};

const CLI_DOCS: &str = "https://manual.mikrotik.com/docs/cli-reference";
const MANUAL: &str = "https://manual.mikrotik.com/docs";

/// Section heading for the operator-facing “do I need this?” copy.
pub const WHEN_YOU_NEED_IT: &str = "When you need it";

/// Curated explanation for one navigation screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenGuide {
    pub summary: &'static str,
    pub use_when: &'static str,
    pub fields: &'static str,
    /// Conceptual manual page when one exists; CLI reference is always added.
    pub docs_url: Option<&'static str>,
}

/// Title, path kicker, and wrapped body for the about overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutCopy {
    pub title: String,
    pub kicker: String,
    pub body: String,
}

/// Guide for the open screen (`dashboard` or a resource id).
#[must_use]
pub fn screen_guide(id: &str) -> Option<&'static ScreenGuide> {
    GUIDES
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, guide)| guide)
}

/// Formatted overlay copy, or `None` when the id is unknown.
#[must_use]
pub fn about_copy(id: &str) -> Option<AboutCopy> {
    let guide = screen_guide(id)?;
    let (title, kicker, cli_url) = if id == DASHBOARD_ID {
        (
            "About Dashboard".to_string(),
            "overview".to_string(),
            format!("{MANUAL}/introduction/"),
        )
    } else {
        let spec = resource_by_id(id)?;
        (
            format!("About {}", spec.label),
            spec.cli_path().to_string(),
            format!("{CLI_DOCS}{}/", spec.cli_path()),
        )
    };

    let mut body = String::new();
    body.push_str(guide.summary);
    body.push_str("\n\n");
    body.push_str(WHEN_YOU_NEED_IT);
    body.push('\n');
    body.push_str(guide.use_when);
    if !guide.fields.is_empty() {
        body.push_str("\n\nNotable fields\n");
        body.push_str(guide.fields);
    }
    body.push_str("\n\nOfficial documentation\n");
    if let Some(url) = guide.docs_url
        && url != cli_url
    {
        body.push_str(url);
        body.push('\n');
    }
    body.push_str(&cli_url);
    Some(AboutCopy {
        title,
        kicker,
        body,
    })
}

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

static GUIDES: &[(&str, ScreenGuide)] = &[
    guide!(
        "dashboard",
        "Live CPU, memory, WAN throughput, and firewall-hit overview for the connected \
         router. This is an mtui dashboard, not a RouterOS menu.",
        "Use it as a first look at whether the device is busy or the WAN is moving traffic. \
         Open a specific menu when you need to change configuration.",
        "Charts are sampled locally. Firewall rows are hit counters from filter rules, not \
         a rule editor.",
        "https://manual.mikrotik.com/docs/introduction/"
    ),
    guide!(
        "interfaces",
        "All RouterOS interfaces in one table: Ethernet, VLANs, tunnels, wireless, bridges, \
         and dynamic entries. Each row is a L2/L3 attachment point with MTU, MAC, and counters.",
        "Start here to see what exists, whether a link is running, or to torch/reset counters. \
         Type-specific menus (Ethernet, VLAN, WireGuard, …) edit the extra properties of that type.",
        "name, type, mtu/l2mtu/actual-mtu, mac-address, running/slave/disabled, and RX/TX \
         counters. Fast-path (fp-*) counters appear when the packet path bypasses the CPU."
    ),
    guide!(
        "interface-lists",
        "Named sets of interfaces used by firewall, neighbor discovery, detect-internet, \
         and similar consumers. Built-in sets include all, none, static, and dynamic; \
         you can add your own names.",
        "Define the set here, then attach interfaces on List members. Use include/exclude \
         to nest other lists instead of repeating the same members. Changing a list updates \
         every rule that refers to it.",
        "name is the set. include and exclude nest other lists. Lists have no disabled flag. \
         Membership joins live on List members — adding a bridge is not the same as adding \
         its ports.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/47579180/Interface+Lists"
    ),
    guide!(
        "ethernet",
        "Physical Ethernet (and SFP) ports: negotiation, advertised speeds, switch chip, \
         and loop-protect. These are the parent interfaces for VLANs, MACsec, and MACVLAN.",
        "Use it to rename ports, lock speed/duplex, or inspect the switch the port belongs to. \
         Do not expect virtual interfaces here.",
        "auto-negotiation, advertise, speed, full-duplex, arp, orig-mac-address, switch, \
         loop-protect. default-name is the factory name."
    ),
    guide!(
        "interface-list-members",
        "Joins that attach an interface to a list. This table is the membership, not the \
         named set itself (that lives on Lists).",
        "Assign LAN/WAN or a custom list so firewall and discovery can match a group of \
         ports. Dynamic members created by include/exclude on Lists are not shown here.",
        "list is the target set; interface is the member. disabled parks a join without \
         deleting it. Built-in lists still accept members."
    ),
    guide!(
        "eoip",
        "Ethernet over IP: a Layer-2 tunnel that carries Ethernet frames inside IP so two \
         sites can share a broadcast domain. It is a MikroTik protocol, not GRE.",
        "Use it to bridge remote LANs or run protocols that need Ethernet, not just IP. \
         Pair local-address/remote-address and a matching tunnel-id on both ends.",
        "tunnel-id, local-address, remote-address, ipsec-secret when encrypting, keepalive, \
         and the usual interface MTU/MAC fields."
    ),
    guide!(
        "ipip",
        "IP-in-IP tunnel: encapsulates IPv4 (or IPv6) packets inside another IP packet. \
         It does not carry Ethernet frames.",
        "Use it for simple point-to-point routed tunnels when you do not need L2 bridging. \
         EoIP or GRE are better if you need Ethernet or extra GRE features.",
        "local-address, remote-address, clamp-tcp-mss, ipsec-secret, keepalive."
    ),
    guide!(
        "gre",
        "GRE tunnel for encapsulating packets between two IP endpoints. RouterOS can add \
         IPsec to the same tunnel.",
        "Use GRE when you need a standard tunnel that other vendors also speak, or when \
         you want IPsec without a separate IPsec policy set.",
        "local-address, remote-address, allow-fast-path, ipsec-secret, keepalive, mtu."
    ),
    guide!(
        "vlan",
        "IEEE 802.1Q (and Q-in-Q) VLAN interfaces on a parent port. This is a router VLAN \
         subinterface, not the bridge VLAN table used for switching.",
        "Use it when the router itself needs an IP on a tagged VLAN. For hardware switching \
         between ports, use Bridge VLANs (or Switch VLAN) instead.",
        "vlan-id and parent interface define the tag. use-service-tag selects 802.1ad. MTU \
         is usually inherited from the parent."
    ),
    guide!(
        "vxlan",
        "VXLAN overlay: Ethernet segments identified by a VNI, carried over UDP/IP between \
         VTEPs. Used to stretch L2 across an L3 underlay.",
        "Use it in datacenter or campus overlays, not as a replacement for a simple VLAN on \
         one switch. Pair vni with local-address or multicast group.",
        "vni, local-address, group, port (default 4789), vteps, learning, and optional bridge \
         binding for hardware offload on supported chips."
    ),
    guide!(
        "vrrp",
        "Virtual Router Redundancy Protocol: two or more routers share a virtual IP (and MAC) \
         so LAN hosts keep a gateway if the master fails.",
        "Use it for first-hop redundancy on a LAN. Both routers need a matching VRID and \
         authentication; priority elects the master.",
        "interface, vrid, priority, version (v2/v3), address, preemption-mode, authentication."
    ),
    guide!(
        "bonding",
        "Link aggregation: several Ethernet (or similar) ports act as one logical interface \
         for capacity or failover (802.3ad, active-backup, balance-*, …).",
        "Use it when the switch or peer supports the same mode. Members must match speed and \
         usually sit on the same switch chip for offload.",
        "mode, slaves/members, transmit-hash-policy, lacp-rate, min-links, mtu."
    ),
    guide!(
        "lte",
        "LTE/5G modems: APN, band, and connection state for the cellular interface.",
        "Use it on devices with a modem (or USB LTE) to bring up a WAN over mobile data.",
        "apn, network-mode, band, imei/iccid where present, running and disabled."
    ),
    guide!(
        "wifi",
        "wifiwave2 / wifi interfaces (RouterOS 7 radio stack): SSIDs, datapath, and radio \
         settings for ax/ac Wave2 hardware.",
        "Use this on modern wifi packages. Legacy `/interface wireless` is a different driver \
         and menu (Wireless).",
        "configuration, datapath, channel, ssid, security, running/disabled. CAPsMAN-managed \
         radios may be mostly read-only here."
    ),
    guide!(
        "wireless",
        "Legacy wireless (wireless package): older 802.11 radios, station/AP modes, and \
         WDS. Distinct from the wifiwave2 wifi menu.",
        "Use it only on hardware that still runs the old wireless driver. New ax devices \
         belong under WiFi.",
        "mode, ssid, frequency, band, security-profile, running/disabled."
    ),
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
    guide!(
        "macvlan",
        "Virtual interfaces with their own MAC on a parent that already has a MAC. Unlike \
         VLAN, there is no 802.1Q tag — the distinction is the MAC address.",
        "Use it to get extra MACs/IPs or multiple PPPoE sessions from one Ethernet. Not for \
         bridging or stacking a VLAN on top of MACVLAN.",
        "interface (parent), mac-address, mode private (no MACVLAN-to-MACVLAN on the same \
         parent) or bridge (they can talk). Parent must not already be bridged or bonded.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/217874440/MACVLAN"
    ),
    guide!(
        "macsec",
        "MACsec (IEEE 802.1AE) encrypts Ethernet frames on a point-to-point link so traffic \
         on the wire is confidential, intact, and authentic. RouterOS uses GCM-AES-128 and \
         protects all LAN traffic on that hop, including DHCP, ARP, and LLDP.",
        "Use it when two devices share a dedicated Ethernet (or similar) hop and you need \
         Layer-2 encryption rather than IPsec. Matching pre-shared CAK and CKN are required \
         on both ends (no Dot1x keying yet). Hardware offload exists only on some products. \
         Skip this menu if you are not encrypting a single Ethernet hop.",
        "interface is the parent Ethernet (one MACsec per Ethernet). cak (16-byte) and ckn \
         (32-byte) must match the peer; omit them to let RouterOS generate values, then copy \
         those to the other side. profile selects key-server priority. status is read-only \
         (negotiating, open-encrypted, …). MTU defaults to 1468; L2MTU is derived (parent \
         minus 32 bytes) and is not settable.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/macsec/"
    ),
    guide!(
        "macsec-profiles",
        "MACsec profiles used to elect the MKA key server on a point-to-point link. The key \
         server creates the Secure Association Key (SAK) that actually encrypts frames.",
        "Change this only when you need a specific side to be key server. Most setups keep \
         the default profile.",
        "name and server-priority (0–255, lower wins). Equal priority falls back to the \
         lowest MAC address.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/macsec/"
    ),
    guide!(
        "vrf",
        "Virtual Routing and Forwarding: isolated routing tables so the same prefixes can \
         exist in more than one tenant or WAN context.",
        "Use it for overlapping VPNs, multi-tenant CE, or keeping management routing apart \
         from data. Interfaces are assigned to a VRF; connected routes follow.",
        "name, interfaces, and comments. Routes themselves still live under IP/IPv6 Routes \
         with a routing-table/VRF mark."
    ),
    guide!(
        "detect-internet",
        "Singleton that classifies interfaces as internet, WAN, or LAN using reachability \
         tests, then can populate interface lists.",
        "Useful on simple CPE so WAN/LAN lists stay correct. Turn it off if you assign WAN \
         membership yourself and do not want surprise list changes.",
        "detect-interface-list, internet/wan/lan-interface-list, and related state."
    ),
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
    guide!(
        "bridges",
        "Software (and hardware-offloaded) bridges that join Ethernet-like ports into one \
         L2 domain. Transparent: hosts on different ports look like one LAN.",
        "Use a bridge for switching, VLAN filtering, or STP. For one routed port, an IP on \
         Ethernet is enough — you do not need a bridge.",
        "name, vlan-filtering, protocol-mode (none/stp/rstp/mstp), admin-mac/auto-mac, \
         igmp-snooping, dhcp-snooping.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-ports",
        "Member ports of a bridge: PVID, frame types, trusted, edge, and STP costs.",
        "Add every interface that should share the bridge forwarding domain. Horizon splits \
         ports so they cannot flood to each other (split horizon).",
        "bridge, interface, pvid, frame-types, ingress-filtering, trusted, edge, \
         point-to-point, hw (offload).",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-hosts",
        "Learned or static MAC table of the bridge (which port a MAC was seen on).",
        "Inspect flooding/learning issues. Static entries pin a MAC to a port.",
        "mac-address, on-interface, vid, local/dynamic/authorized flags.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-vlans",
        "Bridge VLAN table: which ports are tagged or untagged for each VLAN ID when \
         vlan-filtering is on. This is switching, not `/interface vlan`.",
        "Required for proper trunk/access ports. Interface lists can be used as tagged or \
         untagged from RouterOS 7.17.",
        "bridge, vlan-ids, tagged, untagged, current-tagged/untagged (read-only).",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-mdb",
        "Multicast database for IGMP/MLD snooping: which ports should receive a group.",
        "Inspect or pin multicast forwarding when igmp-snooping is enabled.",
        "bridge, group, ports, vid, dynamic.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-msti",
        "Multiple Spanning Tree Instances: separate STP topologies per VLAN group (MSTP).",
        "Only when protocol-mode is mstp and you need VLANs to follow different trees.",
        "bridge, identifier, vlan-mapping, priority.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-filter",
        "Bridge firewall (L2): match MAC/VLAN and drop, accept, or mark frames in the bridge.",
        "Filter at Layer 2 before routing. IP firewall still applies after a frame is routed.",
        "chain, mac-protocol, src/dst-mac, in/out-interface, vlan-id, action.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-nat",
        "Bridge NAT: rewrite MAC addresses as frames pass the bridge.",
        "Rare; used for MAC mapping rather than IP NAT.",
        "chain, src/dst-mac, action (src-nat/dst-nat), to-src-mac/to-dst-mac.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-settings",
        "Global bridge settings (one object): allow-fast-path, use-ip-firewall, and related \
         toggles that apply to all bridges.",
        "Change this when you need IP firewall to see bridged traffic, or to disable fast-path \
         for debugging. Most devices keep defaults.",
        "use-ip-firewall, use-ip-firewall-for-vlan/pppoe, allow-fast-path, \
         forward-reserved-addresses.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/"
    ),
    guide!(
        "bridge-port-controller",
        "CRS/switch port-controller (802.1BR) coordinator settings.",
        "Only on supported switch hardware using a port extender architecture. Ignore on \
         ordinary routers.",
        "enabled and controller identity fields."
    ),
    guide!(
        "bridge-port-controller-device",
        "Discovered or configured port-extender devices attached to the controller.",
        "Inventory of extenders in a 802.1BR setup.",
        "name, identity, and status of each extender."
    ),
    guide!(
        "bridge-port-controller-port",
        "Ports on the controller side of a port-extender topology.",
        "Map extended ports. Skip unless you run PE hardware.",
        "port, device, and cascade relations."
    ),
    guide!(
        "bridge-port-extender",
        "Port extender (PE) role settings when this device is the satellite, not the \
         controller.",
        "Configure only on extender hardware that joins a controller.",
        "controller address and extender identity."
    ),
    guide!(
        "switch",
        "Switch chips in the device (name, type, mirroring, CPU flow control).",
        "Use it to see which ASIC exists and to set chip-wide options. Per-port settings are \
         under Switch Port.",
        "name, type, mirror-source/target, l3-hw-offloading related flags."
    ),
    guide!(
        "switch-port",
        "Per-port switch-chip settings: VLAN mode, default VLAN id, storm rate, and PHY.",
        "Use alongside Bridge when the chip offloads forwarding. VLAN mode must match how \
         you tag on that port.",
        "vlan-mode, vlan-header, default-vlan-id, rx/tx-storm, limit, invalid-vlan-action."
    ),
    guide!(
        "switch-vlan",
        "Switch-chip VLAN table (older / independent of bridge VLAN filtering).",
        "On chips that still use `/interface ethernet switch vlan` instead of (or as well as) \
         bridge vlan-filtering.",
        "switch, ports, vlan-id, independent-learning."
    ),
    guide!(
        "switch-host",
        "Switch host (MAC) table learned on the chip.",
        "See which port the ASIC thinks a MAC is on. Useful when hardware offload is on.",
        "switch, mac-address, ports, vid, dynamic/invalid."
    ),
    guide!(
        "switch-rule",
        "Switch ACL/rules: hardware match-action on supported chips.",
        "Offload simple drops/redirects. Complex policy still belongs in IP or bridge filter.",
        "switch, ports, src/dst-mac/ip, protocol, action (copy-to-cpu, drop, redirect, …)."
    ),
    guide!(
        "switch-port-isolation",
        "Port isolation maps: which switch ports may forward to which others.",
        "Private VLAN-style isolation without a full bridge filter.",
        "switch, port, forwarding-override / isolated port lists."
    ),
    guide!(
        "switch-l3hw",
        "Layer-3 hardware offload (L3HW) on CRS3xx/5xx and similar: routing in the ASIC.",
        "Enable only on supported switches after reading the L3HW caveats (fasttrack, \
         connection tracking, some features disabled).",
        "l3hw settings and per-switch offload flags."
    ),
    guide!(
        "arp",
        "IPv4 ARP table: IP to MAC bindings on local subnets, including static entries.",
        "Inspect neighbors, pin a static ARP, or spot duplicates. IPv6 neighbors are a \
         different menu.",
        "address, mac-address, interface, published, complete, dhcp, dynamic."
    ),
    guide!(
        "addresses",
        "IPv4 addresses assigned to interfaces (connected networks).",
        "Give the router an address on each L3 interface. Network/broadcast are derived from \
         the prefix.",
        "address (CIDR), interface, network, disabled, comment."
    ),
    guide!(
        "dhcp-servers",
        "IPv4 DHCP servers that lease addresses from pools on an interface.",
        "Run a server on LAN. Needs a pool, network, and usually an address on the interface.",
        "interface, address-pool, lease-time, relay, disabled."
    ),
    guide!(
        "dhcp-networks",
        "DHCP network options handed to clients: gateway, DNS, domain, NTP.",
        "One entry per served prefix. Mismatch with the pool/server is a common outage cause.",
        "address, gateway, dns-server, domain, ntp-server, wins-server, dhcp-option."
    ),
    guide!(
        "dhcp-leases",
        "Active and static IPv4 DHCP leases (who got which address).",
        "See clients, make a lease static, or find a MAC. Dynamic leases disappear when they \
         expire or the server forgets them.",
        "address, mac-address, client-id, server, status, expires-after, host-name."
    ),
    guide!(
        "dhcp-relay",
        "Forwards DHCP between a client LAN and a DHCP server on another network.",
        "Use when the DHCP server is not on this broadcast domain. The relay interface is \
         the client side; dhcp-server is the real server address.",
        "name, interface, dhcp-server, local-address, disabled."
    ),
    guide!(
        "dhcp-options",
        "Named DHCP option codes and values that networks or option sets can attach.",
        "Define vendor or extra options once, then reference them from a network or set.",
        "name, code, value."
    ),
    guide!(
        "dhcp-option-sets",
        "Named groups of DHCP options applied together.",
        "Attach a set to a DHCP network instead of listing every option on that network.",
        "name, options (list of option names)."
    ),
    guide!(
        "firewall-filter",
        "IPv4 filter: accept, drop, reject, fasttrack, jump — the main packet policy table.",
        "Control what the router forwards or accepts. Chains input/forward/output are the \
         usual starting points.",
        "chain, action, src/dst-address, protocol, ports, in/out-interface(-list), \
         connection-state, log, packets/bytes."
    ),
    guide!(
        "firewall-nat",
        "IPv4 NAT: srcnat (masquerade/src-nat) and dstnat (port forward).",
        "Hide LAN behind WAN (masquerade) or publish an internal service (dst-nat). Filter \
         still needs to allow the traffic.",
        "chain, action, to-addresses/to-ports, src/dst-address, protocol, in/out-interface."
    ),
    guide!(
        "firewall-mangle",
        "IPv4 mangle: mark connections, packets, or routing for policy routing and QoS.",
        "Use marks that queues and routes consume. Do not put simple allow/deny here — that \
         is filter.",
        "chain, action (mark-connection/packet/routing), new-*-mark, passthrough, matchers."
    ),
    guide!(
        "firewall-raw",
        "IPv4 raw table: prerouting/output before connection tracking.",
        "Drop or notrack early, or exempt traffic from conntrack. Most policy still belongs \
         in filter.",
        "chain (prerouting/output), action, address and interface matchers, packets/bytes."
    ),
    guide!(
        "firewall-connections",
        "IPv4 connection tracking table: live conntrack entries the firewall is following.",
        "Inspect who is talking through the router. Remove drops that tracked entry so the \
         next packet is treated as a new connection; it does not delete a filter rule.",
        "src/dst-address, protocol, ports, tcp-state, timeout, orig-rate/repl-rate, \
         connection-mark. Reply addresses and other keys show in the inspector.",
        "https://manual.mikrotik.com/docs/firewall-and-quality-of-service/connection-tracking/"
    ),
    guide!(
        "address-list",
        "Named IPv4 address lists referenced by firewall matchers (and some other menus).",
        "Group IPs/prefixes for allowlists, blocks, or PCC. Timeouts make dynamic entries.",
        "list, address, timeout, dynamic, creation-time."
    ),
    guide!(
        "firewall-layer7",
        "Named regular expressions used as layer7-protocol matchers in firewall rules.",
        "Match application payloads when ports are not enough. Regex is costly; keep it rare.",
        "name, regexp."
    ),
    guide!(
        "firewall-service-port",
        "Helper services (ftp, h323, sip, …) the firewall can inspect or disable.",
        "Turn a helper off if it breaks NAT or is unused. These rows are built-in; you do \
         not add new names.",
        "name, ports, disabled."
    ),
    guide!(
        "ipsec-peers",
        "IKE peers: remote address, profile, and how this router starts or answers Phase 1.",
        "Add one peer per remote VPN endpoint. Profiles hold the crypto; identities hold \
         how you prove who you are.",
        "name, address, profile, exchange-mode, port, passive, send-initial-contact.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-identities",
        "Authentication for an IPsec peer: pre-shared key, certificates, or EAP, plus local \
         and remote IDs.",
        "Pair an identity with a peer. my-id and remote-id are identifiers, not secrets; \
         secret is the PSK.",
        "peer, auth-method, secret, my-id, remote-id, certificate, generate-policy.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-policies",
        "Which traffic is protected (or bypassed) after IKE: selectors, tunnel vs transport, \
         proposal, and peer.",
        "Match src/dst prefixes to encrypt, or use generate-policy on identities for road \
         warriors. Check ph2-state when a tunnel is up but traffic is not.",
        "src/dst-address, ports, protocol, action, level, proposal, peer, tunnel, ph2-state.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-proposals",
        "Phase 2 transform sets: integrity, encryption, PFS group, and lifetime.",
        "Policies point at a proposal. Keep algorithms the two ends share; weaker suites \
         are for interoperability only.",
        "name, auth-algorithms, enc-algorithms, pfs-group, lifetime.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-profiles",
        "Phase 1 crypto: hash, encryption, DH group, NAT traversal, and DPD.",
        "Peers reference a profile. Change DPD when a remote goes silent; NAT-T when a \
         peer is behind NAT.",
        "name, hash-algorithm, enc-algorithm, dh-group, proposal-check, lifetime, \
         nat-traversal, dpd-interval.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-installed-sa",
        "Runtime security associations currently installed (SPI, algorithms, byte counters). \
         This is live state, not a config list.",
        "Confirm a tunnel actually negotiated. Removing a selected SA flushes that \
         association so IKE can rebuild it.",
        "src/dst-address, spi, auth-algorithm, enc-algorithm, state, current-bytes. Key \
         material is not shown.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-settings",
        "Global IPsec knobs: accounting, RADIUS for XAuth, and how identities are matched.",
        "Leave defaults unless you account IPsec sessions or run XAuth against RADIUS.",
        "accounting, interim-update, xauth-use-radius, uniq-id-accounting, identities-matching.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "neighbors",
        "Neighbor discovery (MNDP/CDP/LLDP): other MikroTik (and some vendor) devices seen \
         on the wire.",
        "Find adjacent routers and their addresses/identity. Read-only discovery, not a config \
         list you add to.",
        "identity, address, mac-address, interface, platform, version, unpacked."
    ),
    guide!(
        "dhcp-clients",
        "IPv4 DHCP clients: this router obtaining an address (typical WAN).",
        "Use on the ISP port when the WAN is DHCP. add-default-route and use-peer-dns are \
         the usual toggles.",
        "interface, status, address, gateway, dhcp-server, add-default-route, use-peer-dns."
    ),
    guide!(
        "dns",
        "DNS resolver settings: cache, allow-remote-requests, upstream servers, DoH.",
        "Turn on allow-remote-requests to serve LAN clients. Point at upstreams or use \
         DoH where you want encrypted DNS.",
        "servers, allow-remote-requests, cache-size, use-doh-server, verify-doh-cert."
    ),
    guide!(
        "dns-static",
        "Static DNS names in the local resolver (A/AAAA/CNAME/MX/…).",
        "Override or invent names for LAN hosts without running a separate DNS server.",
        "name, address, type, cname, ttl, regexp, comment."
    ),
    guide!(
        "routes",
        "IPv4 routing table: connected, static, and dynamic routes (OSPF/BGP/…).",
        "Add static defaults or more-specifics. Check distance, scope, and routing-table \
         (VRF) when a route is ignored.",
        "dst-address, gateway, distance, routing-table, suppress-hw-offload, active/dynamic."
    ),
    guide!(
        "pools",
        "IPv4 address pools used by DHCP, PPP, and Hotspot.",
        "Define ranges you can hand out. Servers point at a pool by name.",
        "name, ranges, next-pool."
    ),
    guide!(
        "ip-services",
        "Management services and their ports/addresses: www, www-ssl, api, api-ssl, ssh, \
         telnet, ftp, winbox.",
        "Disable what you do not use; restrict available-from. REST uses www-ssl.",
        "name, port, address, certificate (for TLS), disabled."
    ),
    guide!(
        "ip-settings",
        "Global IPv4 stack: rp-filter, tcp-syncookies, forwarding, ICMP, neighbor limits.",
        "Tune the IP stack. rp-filter and forwarding are the usual security-related knobs.",
        "ip-forward, rp-filter, tcp-syncookies, icmp-rate-limit, max-neighbor-entries."
    ),
    guide!(
        "ipv6-addresses",
        "IPv6 addresses on interfaces (including link-local and SLAAC/DHCPv6 results).",
        "Assign GUAs or ULAs. Advertise prefixes via ND if this router is the LAN gateway.",
        "address, interface, advertise, eui-64, from-pool, actual-interface."
    ),
    guide!(
        "ipv6-neighbors",
        "IPv6 Neighbor Discovery cache (like ARP for IPv6).",
        "Inspect who is on-link. Static entries are uncommon.",
        "address, mac-address, interface, status (reachable/stale/…)."
    ),
    guide!(
        "ipv6-nd",
        "Per-interface IPv6 Neighbor Discovery: RA, managed flags, MTU, DNS in RA.",
        "This is how LAN hosts learn the default gateway and prefix. Disable RA if another \
         router should advertise.",
        "interface, ra-interval, advertise-dns, mtu, hop-limit, managed-address-configuration."
    ),
    guide!(
        "ipv6-nd-prefix",
        "Prefixes advertised in IPv6 Router Advertisements on an interface.",
        "Publish which prefix LAN hosts may use. Distinct from the ND interface settings.",
        "prefix, interface, advertise, disabled."
    ),
    guide!(
        "ipv6-routes",
        "IPv6 routing table (static and dynamic).",
        "Same idea as IPv4 routes: dst-prefix, gateway, distance, VRF/table.",
        "dst-address, gateway, distance, routing-table, active/dynamic."
    ),
    guide!(
        "ipv6-pool",
        "IPv6 prefix pools for PD or assignment.",
        "Hand prefixes to DHCPv6 or PPP. prefix-length is the size delegated.",
        "name, prefix, prefix-length."
    ),
    guide!(
        "ipv6-dhcp-client",
        "DHCPv6 client: request a prefix or address on an interface.",
        "Typical WAN PD: request prefix, store it in a pool, then advertise on LAN via ND.",
        "interface, pool-name, request, add-default-route, status, prefix, expires-after."
    ),
    guide!(
        "ipv6-dhcp-server",
        "DHCPv6 server that leases prefixes or addresses from an IPv6 pool.",
        "Use on LAN when hosts need stateful DHCPv6 rather than SLAAC-only.",
        "name, interface, address-pool, lease-time, disabled."
    ),
    guide!(
        "ipv6-settings",
        "Global IPv6 stack: forwarding, accept-redirects, neighbor limits.",
        "Disable forward to make the box a host. Most routers keep forward on.",
        "forward, accept-redirects, max-neighbor-entries, disable-ipv6."
    ),
    guide!(
        "ipv6-firewall-filter",
        "IPv6 filter table (separate from IPv4 filter).",
        "IPv6 is not covered by IPv4 rules. Build input/forward policy here too.",
        "chain, action, src/dst-address, protocol, in/out-interface, packets/bytes."
    ),
    guide!(
        "ipv6-firewall-nat",
        "IPv6 NAT table (srcnat/dstnat). Less common than IPv4 NAT but the same idea.",
        "NPTv6 or port mapping when you must rewrite IPv6. Filter still has to allow traffic.",
        "chain, action, to-addresses/to-ports, address and interface matchers."
    ),
    guide!(
        "ipv6-address-list",
        "Named IPv6 address lists for firewall matchers.",
        "Group prefixes the same way IPv4 address lists group IPv4.",
        "list, address, timeout, dynamic."
    ),
    guide!(
        "routing-tables",
        "Named routing tables (including main and VRF FIBs).",
        "Extra tables are for policy routing. fib marks whether the table installs in the FIB.",
        "name, fib, dynamic."
    ),
    guide!(
        "routing-rules",
        "Policy routing rules: select a table from src/dst/routing-mark before looking up \
         the main table.",
        "Use with mangle routing marks or multi-WAN. Order matters.",
        "src-address, dst-address, routing-mark, action, table, disabled."
    ),
    guide!(
        "ospf-instances",
        "OSPF routing instances (v2/v3): router-id and how default routes are originated.",
        "Need dynamic IGP inside an AS. Areas and interface templates are sibling menus.",
        "name, version, router-id, originate-default, disabled."
    ),
    guide!(
        "ospf-areas",
        "OSPF areas belonging to an instance: area-id and type (backbone, stub, NSSA, …).",
        "Split a large domain. Area 0.0.0.0 is backbone. Attach networks via interface \
         templates.",
        "name, instance, area-id, type, disabled."
    ),
    guide!(
        "ospf-interface-templates",
        "OSPF interface templates (RouterOS v7): which interfaces sit in which area.",
        "Bind instance+area to one or more interfaces. There is no separate \
         /routing/ospf/interface menu in v7.",
        "instance, area, interfaces, type, disabled."
    ),
    guide!(
        "bgp-connections",
        "BGP connections (RouterOS v7 style): remote address/AS and local role.",
        "Peering with ISPs or other ASes. Templates and address-families may exist beyond \
         this table.",
        "name, remote.address, remote.as, local.role, disabled."
    ),
    guide!(
        "bgp-templates",
        "Reusable BGP session defaults (AS, router-id, address-families) for connections.",
        "Put common peering options on a template, then point connections at it.",
        "name, as, router-id, address-families, output.network, disabled."
    ),
    guide!(
        "queue-simple",
        "Simple queues: easy per-target (IP/interface) rate limits using HTB.",
        "Cap a customer or a WAN. For hierarchical sharing use Queue Tree plus packet marks.",
        "name, target, max-limit, burst, packet-marks, parent, disabled."
    ),
    guide!(
        "queue-tree",
        "HTB queue tree: hierarchical classes usually keyed by packet-mark.",
        "Build QoS after mangle marks traffic. Needs a parent (often global-in/out or an \
         interface).",
        "name, parent, packet-mark, max-limit, limit-at, priority, bucket-size."
    ),
    guide!(
        "queue-type",
        "Queue type definitions: pfifo, sfq, pcq, fq-codel, cake, and so on.",
        "Reuse a type from simple/tree queues. PCQ is common for per-peer fairness.",
        "name, kind, and kind-specific options (pcq-rate, fq-codel-limit, …)."
    ),
    guide!(
        "queue-interface",
        "Default queue type attached to an interface’s software queue.",
        "Change only if you know you need a specific qdisc on that port.",
        "interface, queue (type name)."
    ),
    guide!(
        "files",
        "Router filesystem: backups, scripts, images, and uploaded files.",
        "Save a named backup or load a `.backup` file from the action menu (that replaces the \
         running configuration and reboots). Upload a local UTF-8 file (`u`, 1 MiB REST cap), \
         download the selected file (`w`), or fetch a URL onto the router with /tool/fetch (`f`). \
         Larger or binary files should be fetched by URL. Removing a file here deletes it on the \
         router.",
        "name, type, size, creation-time. Contents are not shown in the table."
    ),
    guide!(
        "netwatch",
        "Host monitoring: ICMP/TCP/HTTP probes that can run on-up/on-down scripts.",
        "Watch a gateway or extra WAN and trigger failover scripts. Not a replacement for \
         routing protocols.",
        "host, type, interval, timeout, status, up/down-script, disabled."
    ),
    guide!(
        "email",
        "SMTP client used by `/tool e-mail` and some scripts.",
        "Configure if you want the router to send alerts. Password is a secret.",
        "server, port, from, user, password, tls (yes/starttls/no)."
    ),
    guide!(
        "ping",
        "One-shot ICMP (or similar) reachability check from the router to an address.",
        "Confirm a host is reachable from this router, not from your workstation. Default \
         count is 4 so the REST command finishes within the client timeout.",
        "address (required), count, src-address. Results appear in the Ping overlay; this \
         screen is not a live poll of /tool/ping."
    ),
    guide!(
        "traceroute",
        "Hop-by-hop path discovery from the router toward an address.",
        "See where packets leave this device on the way to a destination. Default hop count \
         is 8 so the REST command stays within the client timeout.",
        "address (required), src-address, protocol (icmp by default), count/max hops. Open \
         the overlay with Enter; t is reserved for interface torch elsewhere."
    ),
    guide!(
        "radius",
        "RADIUS clients: where to send AAA for login, PPP, Hotspot, DHCP, wireless, and \
         similar services.",
        "Add your RADIUS server and enable the matching service. Incoming RADIUS (if used) \
         is a related system setting.",
        "address, protocol, secret, service, timeout, src-address, disabled."
    ),
    guide!(
        "users",
        "RouterOS login accounts (full, write, read, group-based).",
        "Create operators. Prefer groups over sharing admin. Passwords are secrets.",
        "name, group, address (allowed source), last-logged-in, disabled."
    ),
    guide!(
        "user-groups",
        "Permission groups for local users (and some AAA mappings).",
        "Define what a role may read or write instead of using the built-in full user for \
         everything.",
        "name, policy flags, skin."
    ),
    guide!(
        "routerboard",
        "Hardware identity: model, serial, firmware, and factory settings (RouterBOOT).",
        "Read-only inventory. Firmware upgrades are a different, careful operation.",
        "model, serial-number, firmware-type, current/upgrade-firmware, board-name."
    ),
    guide!(
        "ntp",
        "NTP client/server: how the clock is synchronized.",
        "Point at reliable NTP. Many features (certs, logs) need a sane clock.",
        "enabled, servers, mode, freq-error (status)."
    ),
    guide!(
        "clock",
        "Local date, time, and time zone.",
        "Set zone even when NTP is on, so logs print local time.",
        "time, date, time-zone-name, gmt-offset."
    ),
    guide!(
        "identity",
        "System identity string shown in neighbors, WinBox, and prompts.",
        "Set a unique name per device. It is not a DNS name unless you also create DNS.",
        "name."
    ),
    guide!(
        "resources",
        "CPU, memory, HDD, uptime, version — `/system resource`.",
        "Capacity and version checks. Reboot or shut down the router from this screen; those \
         commands are system-wide, not operations on the resource table itself.",
        "uptime, version, cpu, cpu-load, free/total-memory, architecture."
    ),
    guide!(
        "health",
        "Hardware sensors: voltage, temperature, fans where the board has them.",
        "Spot PSU or thermal issues. Not every model exports health.",
        "name, value, type — varies by hardware."
    ),
    guide!(
        "packages",
        "Installed RouterOS packages (wireless, extra, …) and their versions.",
        "See what is enabled. Installing packages is a reboot-class change; do it with a \
         plan.",
        "name, version, build-time, scheduled, disabled."
    ),
    guide!(
        "scheduler",
        "Scheduled scripts: run a `/system script` at intervals or calendar times.",
        "Automate backups or housekeeping. The script body lives under Scripts.",
        "name, start-date/time, interval, on-event, disabled."
    ),
    guide!(
        "scripts",
        "Stored RouterOS scripts (the source you schedule or run on events).",
        "Keep automation here rather than one-off terminal history. Policy/permissions \
         apply when they run.",
        "name, owner, policy, source (long), dont-require-permissions."
    ),
    guide!(
        "logging",
        "Log rules: which topics go to memory, disk, email, or remote syslog.",
        "Tune noise vs audit. The Logs screen is the memory/file tail, not this config.",
        "topics, action, prefix, disabled."
    ),
    guide!(
        "snmp",
        "SNMP agent: enable, contact, location, trap targets.",
        "For NMS polling. Use v3 where you can; communities are secrets of a sort.",
        "enabled, contact, location, trap-version, trap-community, src-address."
    ),
    guide!(
        "snmp-communities",
        "SNMPv1/v2c communities (and v3 users depending on version).",
        "Restrict addresses; do not use public/private on the internet.",
        "name, addresses, read-access, write-access, security."
    ),
    guide!(
        "certificates",
        "Local certificate store: CA, device certs, CSRs for www-ssl, SSTP, IPsec, OpenVPN. \
         Create an empty request, sign against a CA (or the same name for a root), import a \
         file already on the router, or export PEM/PKCS12.",
        "Needed for HTTPS REST/WinBox TLS and several VPN types. Keys and passphrases stay \
         secret. Sign with g, import with p, export with w.",
        "name, common-name, key-usage, ca, file-name, type, passphrase, export-passphrase."
    ),
    guide!(
        "watchdog",
        "Hardware/software watchdog: reboot if the system stops pinging a target or hangs.",
        "Safety net on remote sites. A bad watch-address can reboot-loop the box.",
        "watch-address, ping-timeout, ping-start-after, no-ping-delay, automatic-supout."
    ),
    guide!(
        "note",
        "Administrative note shown on login (banner-like text).",
        "Leave a contact or change warning for the next operator.",
        "note text, show-at-login."
    ),
    guide!(
        "logs",
        "Live log tail from `/log` (topics + message). This client keeps a bounded local \
         buffer; it does not delete logs on the router when you clear the view.",
        "Debug events and errors. Configure what is recorded under Logging.",
        "time, topics, message. Space pauses the view; severity filter is local."
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ALL_RESOURCES;

    #[test]
    fn every_resource_and_dashboard_has_a_guide() {
        let mut ids: Vec<&str> = GUIDES.iter().map(|(id, _)| *id).collect();
        let original = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), original, "duplicate screen guide ids");
        assert!(screen_guide(DASHBOARD_ID).is_some());
        for spec in ALL_RESOURCES {
            assert!(
                screen_guide(spec.id).is_some(),
                "missing screen guide for {}",
                spec.id
            );
        }
    }

    #[test]
    fn macsec_guide_tracks_the_manual() {
        let guide = screen_guide("macsec").expect("macsec");
        let copy = about_copy("macsec").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["802.1AE", "GCM-AES-128", "CAK", "CKN", "Dot1x", "Ethernet"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(copy.body.contains("/interface/macsec"));
        assert_eq!(
            guide.docs_url,
            Some("https://manual.mikrotik.com/docs/bridging-and-switching/macsec/")
        );
        assert!(copy.kicker.contains("/interface/macsec"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
    }

    #[test]
    fn cli_docs_url_follows_the_resource_path() {
        let copy = about_copy("vlan").expect("vlan");
        assert!(
            copy.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/interface/vlan/")
        );
        assert_eq!(copy.title, "About VLAN");
    }

    #[test]
    fn interface_list_guides_cross_link_definitions_and_members() {
        let lists = about_copy("interface-lists").expect("lists");
        let members = about_copy("interface-list-members").expect("members");
        assert_eq!(lists.title, "About Lists");
        assert_eq!(members.title, "About List members");
        assert!(lists.body.contains("List members"));
        assert!(members.body.contains("Lists"));
        assert!(lists.body.to_ascii_lowercase().contains("include"));
        assert!(lists.body.to_ascii_lowercase().contains("exclude"));
        assert!(members.body.contains("join") || members.body.contains("Joins"));
    }
}
