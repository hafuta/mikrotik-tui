//! Feature-owned operator guides for IP screens.

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
        "Find adjacent routers and their addresses/identity. Connect opens a new device tab \
         using the neighbor address (identity or MAC for the tab name). Discovery is not a \
         config list you add to.",
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
        "Disable what you do not use; restrict available-from. This client uses api-ssl or api.",
        "name, port, address, certificate (for TLS), disabled."
    ),
    guide!(
        "ip-settings",
        "Global IPv4 stack: rp-filter, tcp-syncookies, forwarding, ICMP, neighbor limits.",
        "Tune the IP stack. rp-filter and forwarding are the usual security-related knobs.",
        "ip-forward, rp-filter, tcp-syncookies, icmp-rate-limit, max-neighbor-entries."
    ),
    guide!(
        "ipsec-mode-config",
        "IPsec mode-config (address pool and split-include handed to road warriors).",
        "Use with IKEv2/XAuth clients that expect an internal IP from this router.",
        "name, address-pool, address-prefix-length, split-include, system-dns."
    ),
    guide!(
        "ipsec-key-rsa",
        "RSA keys under `/ip/ipsec/key/rsa` (print lives here, not on `/ip/ipsec/key`).",
        "Named RSA keys used by IPsec identities. `/ip/ipsec/key` itself only exports.",
        "name, key-size.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-key-psk",
        "Peer-bound pre-shared keys under `/ip/ipsec/key/psk`.",
        "Use when an identity authenticates with a PSK stored next to a peer and id, not \
         a generated RSA key.",
        "peer, id, key.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/103841835/IPsec"
    ),
    guide!(
        "ipsec-key-qkd",
        "Quantum key-distribution client settings (`/ip/ipsec/key/qkd`). One object, not a list.",
        "Point the router at a KME when an IPsec profile uses post-quantum PPK via QKD. \
         Missing on builds without that command.",
        "address, cache-size, certificate, key-size, kme-id, peer-sae-id, cache-state.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/341770268/QKD"
    ),
    guide!(
        "cloud",
        "MikroTik Cloud DDNS and time (`/ip cloud`).",
        "Enable DDNS if you want a stable name for a changing WAN IP. Not a graphing UI.",
        "ddns-enabled, update-time, public-address, dns-name, status."
    ),
    guide!(
        "kid-control",
        "Kid Control profiles: time and rate limits for named users.",
        "Limit a household profile; devices are a child table.",
        "name, mon-fri, sat, sun, rate-limit, disabled."
    ),
    guide!(
        "kid-control-devices",
        "Devices assigned to a Kid Control user (usually by MAC).",
        "Map phones/PCs onto a profile. Look up the user name.",
        "name, mac-address, user, disabled."
    ),
    guide!(
        "socks",
        "SOCKS proxy listener (`/ip socks`).",
        "Enable only if you intentionally run a SOCKS service on this router.",
        "enabled, port, connection-idle-timeout."
    ),
    guide!(
        "smb",
        "SMB/CIFS service (`/ip smb`).",
        "Turn the file-sharing listener on for this router. Share paths and accounts are on \
         SMB Shares and SMB Users. Guest access is the default guest user on SMB Users on current \
         RouterOS, not a service-wide allow-guests switch.",
        "enabled, domain, allow-guests, comment.",
        "https://manual.mikrotik.com/docs/storage/smb/"
    ),
    guide!(
        "smb-shares",
        "SMB share folders (`/ip smb shares`): names and directories clients can mount.",
        "Point a share at a router directory (created if missing). Limit access with Valid Users \
         and Invalid Users. Require Encryption is the usual choice for macOS clients.",
        "name, directory, valid-users, invalid-users, read-only, require-encryption, disabled.",
        "https://manual.mikrotik.com/docs/storage/smb/"
    ),
    guide!(
        "smb-users",
        "SMB accounts (`/ip smb users`) that may open shares on this router.",
        "Create a login and password here, then allow or deny that name on SMB Shares. The \
         default guest user can be disabled instead of using a service-wide guest switch.",
        "name, password, read-only, disabled, comment.",
        "https://manual.mikrotik.com/docs/storage/smb/"
    ),
    guide!(
        "upnp",
        "UPnP global switches (`/ip upnp`).",
        "Let LAN clients open NAT mappings. Keep dummy-rule/WAN-disable in mind.",
        "enabled, allow-disable-external-interface, show-dummy-rule."
    ),
    guide!(
        "upnp-interfaces",
        "Which interfaces are internal vs external for UPnP.",
        "Mark WAN as external and LAN as internal. Forced external IP is optional.",
        "interface, type, forced-external-ip, disabled."
    ),
    guide!(
        "dns-cache",
        "Resolver cache entries (`/ip dns cache`) plus flush.",
        "Inspect what the router cached. Flush is a table action, not a graph.",
        "name, type, data, ttl. Flush clears the cache; remove drops one entry."
    ),
    guide!(
        "dhcp-alerts",
        "DHCP server alerts when a foreign DHCP server is seen on an interface.",
        "Watch for rogue DHCP on a LAN you serve.",
        "interface, valid-server, alert-timeout, disabled."
    ),
    guide!(
        "connection-tracking",
        "Connection-tracking timeouts and table size (`/ip firewall connection tracking`).",
        "Tune timeouts; the connections table is a separate screen.",
        "enabled, tcp-established-timeout, udp-timeout, icmp-timeout, total-entries."
    ),
    guide!(
        "neighbor-discovery",
        "MNDP/LLDP/CDP discovery settings (`/ip neighbor discovery-settings`).",
        "Choose which interface list advertises this router. The neighbor table is separate.",
        "discover-interface-list, protocol, lldp-med-net-policy-vlan, mode."
    ),
    guide!(
        "ip-ssh",
        "SSH server crypto settings (`/ip ssh`), not user accounts.",
        "Strong-crypto and host-key size. User SSH keys live under System.",
        "strong-crypto, host-key-size, always-allow-password-login, forwarding-enabled."
    ),
    guide!(
        "traffic-flow",
        "Traffic Flow settings (`/ip traffic-flow`): export CPU-processed flows as NetFlow or \
         IPFIX. Hardware-offloaded bridge traffic is not counted.",
        "Point collectors at this router for ISP or campus accounting. Enable the service, pick \
         interfaces (all or a list), then add Traffic Flow Targets. Sampling Interval and \
         Sampling Space appear only when Packet Sampling is on.",
        "Enabled, Interfaces, Cache Entries, Active Flow Timeout, Inactive Flow Timeout, Packet \
         Sampling, Sampling Interval, Sampling Space.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/traffic-flow/"
    ),
    guide!(
        "traffic-flow-targets",
        "Collectors that receive Traffic Flow exports (`/ip traffic-flow target`).",
        "Add each NetFlow or IPFIX collector by Dst. Address, Port, and Version. v9 Template \
         Refresh and v9 Template Timeout appear only for version 9 or ipfix.",
        "Src. Address, Dst. Address, Port, Version, v9 Template Refresh, v9 Template Timeout, \
         Disabled.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/traffic-flow/"
    ),
    guide!(
        "traffic-flow-ipfix",
        "Which IPFIX information elements this router includes in exported records \
         (`/ip traffic-flow ipfix`).",
        "Tune the template after Version is ipfix on a target. Each row is a yes or no include \
         flag, not the packet field itself.",
        "Bytes, addresses, ports, NAT, TCP, ICMP, interfaces, protocol, ToS, TTL, and related \
         include flags.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/traffic-flow/"
    ),
    guide!(
        "igmp-proxy",
        "IGMP Proxy global timers (`/routing igmp-proxy`). Forwards IGMP and multicast when PIM \
         is more than the topology needs. Exactly one upstream interface belongs on IGMP Proxy \
         Interfaces.",
        "IPTV or multicast handoff from a provider toward LAN subscribers. Bridge IGMP snooping \
         is a different menu.",
        "Query Interval, Query Response Interval, Last Member Query Interval, Robustness, Quick \
         Leave.",
        "https://manual.mikrotik.com/docs/user-guides/routing-and-networking-protocols/multicast/"
    ),
    guide!(
        "igmp-proxy-interfaces",
        "Interfaces that participate in IGMP Proxy (`/routing igmp-proxy interface`). Traffic on \
         other interfaces is ignored. Alternative Subnets applies on the upstream interface.",
        "Mark one uplink as Upstream toward the multicast source and add downstream subscriber \
         interfaces. Both sides need IP addresses.",
        "Interface, Upstream, Threshold, Alternative Subnets, Disabled, plus Status (Querier, \
         Source IP Address, RX/TX counters).",
        "https://manual.mikrotik.com/docs/user-guides/routing-and-networking-protocols/multicast/"
    ),
    guide!(
        "igmp-proxy-mfc",
        "Multicast forwarding cache for IGMP Proxy (`/routing igmp-proxy mfc`). Dynamic entries \
         show what is flowing; a static rule for a group replaces dynamic rules for that group.",
        "Inspect streams, or pin a group to an upstream interface and downstream list when the \
         proxy interfaces are already set.",
        "Group, Source, Upstream Interface, Downstream Interfaces, plus Status (Active Downstream \
         Interfaces, Bytes, Packets, Wrong Packets).",
        "https://manual.mikrotik.com/docs/user-guides/routing-and-networking-protocols/multicast/"
    ),
    guide!(
        "proxy",
        "HTTP proxy singleton (`/ip proxy`).",
        "Enable a cache/proxy on this router. Child lists cover access, cache, and direct.",
        "enabled, port, parent-proxy, max-cache-size, cache-administrator."
    ),
    guide!(
        "proxy-access",
        "Proxy access list (allow/deny clients and destinations).",
        "Restrict who may use the proxy. Enable/disable via the usual toggle.",
        "src-address, dst-address, dst-host, action, disabled."
    ),
    guide!(
        "proxy-cache",
        "Which responses the HTTP proxy may cache.",
        "Do not treat this as a content browser — it is the cache rule list.",
        "dst-host, method, action, disabled."
    ),
    guide!(
        "proxy-direct",
        "Destinations the proxy should bypass (go direct).",
        "Skip parent-proxy or caching for local/internal hosts.",
        "dst-host, dst-address, action, disabled."
    ),
    guide!(
        "hotspot",
        "Hotspot servers bound to an interface (`/ip hotspot`).",
        "Captive portal on a LAN. Profiles, users, hosts, and walled garden are siblings.",
        "name, interface, address-pool, profile, disabled."
    ),
    guide!(
        "hotspot-profiles",
        "Hotspot profiles: portal address, DNS name, HTML, login methods.",
        "Point servers at a profile. RADIUS is optional.",
        "name, hotspot-address, dns-name, html-directory, login-by, use-radius."
    ),
    guide!(
        "hotspot-users",
        "Hotspot user accounts (local, not RADIUS users).",
        "Create trial/staff logins. Passwords are secrets. Look up profile and server.",
        "name, password, profile, server, disabled."
    ),
    guide!(
        "hotspot-cookies",
        "Remembered Hotspot cookies. Remove to force a new login.",
        "Clear a remembered MAC/user pair. Not an editor for cookie policy.",
        "user, mac-address, expires-in."
    ),
    guide!(
        "hotspot-hosts",
        "Live Hotspot hosts with authenticate and bypass actions.",
        "See who is in the portal. Authenticate or bypass a row when the API exposes those commands.",
        "mac-address, address, server, authorized, bypassed, uptime."
    ),
    guide!(
        "hotspot-ip-bindings",
        "Static Hotspot IP bindings (bypassed, blocked, regular).",
        "Pin a MAC to an address or always-bypass a kiosk.",
        "mac-address, address, to-address, type, server, disabled."
    ),
    guide!(
        "hotspot-walled-garden",
        "HTTP walled garden (host/port exceptions before login).",
        "Allow a payment or landing page through the portal.",
        "dst-host, dst-port, action, server, disabled."
    ),
    guide!(
        "hotspot-walled-garden-ip",
        "IP-based walled garden exceptions.",
        "Same idea as walled garden, matching destination addresses.",
        "dst-address, action, server, disabled."
    ),
];
