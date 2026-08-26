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
    crate::features::interfaces::guides::GUIDES
        .iter()
        .chain(crate::features::wireguard::guides::GUIDES)
        .chain(crate::features::ppp::guides::GUIDES)
        .chain(crate::features::bridge::guides::GUIDES)
        .chain(GUIDES)
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
        "containers",
        "Linux containers on RouterOS v7. Images come from a registry (remote-image) or a \
         tar already on Files. Adding a row starts download or extract; it does not start \
         the container. Status, arch, OS, and tag are what the device stored after extract.",
        "Needs the container extra package (arm, arm64, x86, CHR). Device-mode container=yes \
         needs a reset or mode button, or a cold power-off on x86, within the timeout. DNS \
         must be set on IP DNS or on the container. EN7562CT boards only run arm32v5 images; \
         the registry rejects other architectures. This client does not filter image names.",
        "name, interface (VETH), remote-image or file, root-dir, envlist, mountlists, \
         start-on-boot, logging, memory limits, healthcheck (7.23+), status/arch/tag.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-config",
        "Global container settings: registry URL, extract directory, layer store, and \
         registry username/password.",
        "Set registry-url and tmpdir on disk before a remote-image add. Password is stored \
         on the router; this client masks it in the sheet.",
        "registry-url, tmpdir, layer-dir, username, password, memory-high/max, swap-max, \
         assumed-registry-url, memory-current.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-envs",
        "Named environment lists. Each row is a list name plus one key and value. A \
         container points at a list with envlist.",
        "Group variables per app. RouterOS does not mark env values as secrets.",
        "list, key, value.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-mounts",
        "Named bind-mount lists. Each row is list, host src, and path inside the container. \
         Containers reference lists with mountlists.",
        "Point src at a disk path that already exists on the router.",
        "list, src, dst.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "apps",
        "MikroTik app catalog on top of containers. YAML plus NAT and veth that RouterOS \
         applies. arm64 and x86 only; EN7562CT is not supported for Apps even when \
         containers are.",
        "Use it when you want a packaged app instead of a raw container row. The device \
         fetches the catalog; this client lists /app.",
        "name, network (internal/lan/default), YAML, environment/mounts/redirects, status, \
         UI URL, IP.",
        "https://manual.mikrotik.com/docs/containers/apps/"
    ),
    guide!(
        "switch",
        "Switch chips in the device (name, type, mirroring, CPU flow control).",
        "Use it to see which ASIC exists and to set chip-wide options. Per-port settings are \
         under Switch Port.",
        "name, type, and only the chip attributes present on print (mirror-source/target, \
         mirror-egress-target, cpu-flow-control, switch-all-ports, l3-hw-offloading)."
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
        "Bind instance and area to one or more interfaces. Live cost and adjacency state \
         are on OSPF Interface, not on this template.",
        "instance, area, interfaces, type, disabled."
    ),
    guide!(
        "ospf-interfaces",
        "Live OSPF interfaces after templates match: address, area, state, cost, and DR/BDR.",
        "Watch interface state and metric. Change cost or network type on OSPF Interface \
         Templates. Monitor-only; there is no Add.",
        "address, area, state, network-type, cost, dr, bdr."
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
         running configuration and reboots). Pull a file onto the router with /tool/fetch (`f`). \
         Removing a file here deletes it on the router. Local contents upload/download is not \
         available over the classic API.",
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
        "romon",
        "Router Management Overlay Network: an independent L2 overlay used to reach neighbors \
         when IP routing is down. Packets use EtherType 0x88bf and are not shown in sniffer or torch.",
        "Enable it when you need out-of-band management through a neighbor. Secrets authenticate \
         frames with MD5 hashing; they are not encryption. Use SSH or a secure WinBox session on \
         top. A zero ID lets the router pick current-id from a port MAC.",
        "enabled, id, secrets (list), current-id (runtime).",
        "https://manual.mikrotik.com/docs/management-tools/romon/"
    ),
    guide!(
        "romon-ports",
        "Which interfaces (or interface lists) take part in RoMON, with a cost and optional \
         per-port secrets. A default all entry is present on typical routers.",
        "Restrict RoMON to backbone ports, raise cost on slower links, or forbid a WAN. \
         Port secrets override the global list when they are set.",
        "interface (all or a name), forbid, cost, secrets, disabled, comment.",
        "https://manual.mikrotik.com/docs/management-tools/romon/"
    ),
    guide!(
        "graphing",
        "Built-in RouterOS graphs for CPU/memory/disk, interface traffic, and simple queues. \
         Collection is configured here; the pictures are served at /graphs/ on the router HTTP(S) \
         service, not as local dashboard sparks.",
        "Turn graphing on when you want history on the box. Avoid store-on-disk on devices with \
         tiny flash. Page refresh may be seconds or never.",
        "store-every (5min, hour, 24hours), page-refresh.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-interface",
        "Which interfaces graphing samples for traffic charts, and which addresses may view them.",
        "Add all or a named interface. Tighten allow-address on untrusted networks.",
        "interface, allow-address, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-queue",
        "Which simple queues graphing samples, plus whether the queue target-address may view charts.",
        "Use it for per-queue history. If a queue target is 0.0.0.0/0, allow-target can open graphs \
         more widely than allow-address.",
        "simple-queue, allow-address, allow-target, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-resource",
        "CPU, memory, and disk usage graphs. There is no per-interface selector on this list.",
        "Add an entry so resource charts exist, then restrict allow-address if the graphs page \
         should not be public.",
        "allow-address, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "ping",
        "One-shot ICMP (or similar) reachability check from the router to an address.",
        "Confirm a host is reachable from this router, not from your workstation. Replies stream \
         until the count finishes or you close the overlay.",
        "address (required), count, src-address. Results appear in the Ping overlay; this \
         screen is not a live poll of /tool/ping."
    ),
    guide!(
        "traceroute",
        "Hop-by-hop path discovery from the router toward an address.",
        "See where packets leave this device on the way to a destination. Hop replies stream \
         until the probe finishes or you close the overlay.",
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
        "name, group, address, inactivity-policy, inactivity-timeout, last-logged-in, disabled."
    ),
    guide!(
        "special-login",
        "Serial-port proxy logins: an SSH/Telnet user is bound to a `/port` instead of the RouterOS CLI.",
        "Use it so a dedicated account drops straight onto a serial device. Disable the matching `/system/console` binding first or the port stays owned by the local console.",
        "user, port, disabled.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
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
        "model, serial-number, firmware-type, current/upgrade-firmware, board-name. Upgrade and USB power reset are actions.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-settings",
        "RouterBOOT settings: boot device, OS, frequencies, and protected RouterBOOT.",
        "Change boot order or silent-boot on hardware that exposes `/system/routerboard/settings`. Missing on CHR.",
        "boot-os, boot-device, boot-protocol, cpu-frequency, protected-routerboot, silent-boot, auto-upgrade.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-mode-button",
        "Mode-button script: hold time and the script to run.",
        "Wire a physical mode button to a `/system script` on boards that have `/system/routerboard/mode-button`.",
        "enabled, hold-time, on-event.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-reset-button",
        "Reset-button script: hold time and the script to run.",
        "Same idea as the mode button, on boards that expose `/system/routerboard/reset-button`.",
        "enabled, hold-time, on-event.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "ntp",
        "NTP client: how this router synchronizes its clock from NTP servers.",
        "Point the client at reliable NTP sources. Certificates, logs, and many services need a \
         sane clock.",
        "enabled, servers, mode, status.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "ntp-server",
        "NTP server: this router as an NTP source for LAN clients.",
        "Enable the server so clients can unicast to the router. Broadcast needs \
         broadcast-addresses. Set local-clock-stratum when use-local-clock is on.",
        "enabled, broadcast, multicast, manycast, broadcast-addresses, vrf, use-local-clock, \
         local-clock-stratum, auth-key.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "ntp-keys",
        "NTP symmetric keys: numeric key ids and their secret values.",
        "Create a key here, then pick its id as Auth. Key on NTP Server (or leave none).",
        "key-id, key-val.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "clock",
        "Local date, time, and time zone.",
        "Set zone even when NTP is on, so logs print local time.",
        "time, date, time-zone-name, gmt-offset."
    ),
    guide!(
        "license",
        "RouterOS license status for this device: Software ID and nlevel on RouterBOARD or x86, \
         System ID and CHR level on Cloud Hosted Router.",
        "Check the level before an upgrade or a CHR move. Apply a key or import a file already on \
         the router; this client never prints or logs a license key. Output-key is not offered.",
        "software-id, nlevel, features, expires-in on hardware. system-id, level, next-renewal-at, \
         deadline-at, limited-upgrades on CHR.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328149/RouterOS+license+keys"
    ),
    guide!(
        "disks",
        "Attached storage: USB, NAND, RAID, tmpfs, and network-backed slots under `/disk`.",
        "Inspect size and filesystem before containers or extra logging. Format and eject ask for \
         confirmation. RAID type and role are sheet fields with a save preview; they are not silent \
         extra commands.",
        "slot, type, mount-filesystem, RAID type/role/master, size, free, fs, state. Format needs \
         a file-system type.",
        "https://manual.mikrotik.com/docs/hardware/disks/"
    ),
    guide!(
        "device-mode",
        "RouterOS v7 device-mode: which features (container, scheduler, traffic-gen, fetch, and \
         others) this box is allowed to run. Home, basic, advanced, and ROSE presets each leave \
         some flags off until you enable them.",
        "Read the flags before blaming a missing menu. Saving here sends `/system/device-mode \
         update`, not a silent PATCH. RouterOS then waits for a reset or mode button press, or a \
         cold power-off, within the activation timeout (default 5 minutes). The device reboots when \
         the change is confirmed. If you do nothing, the update is canceled.",
        "mode, per-feature yes/no flags, flagged, flagging-enabled, allowed-versions, attempt-count.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/93749258/Device-mode"
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
        "Log rules: which topics go to an action (memory, disk, echo, email, or remote syslog).",
        "Tune noise vs audit. Each rule picks an action from Logging Actions. The Logs screen \
         is the memory/file tail, not this config.",
        "topics, action, prefix, disabled."
    ),
    guide!(
        "logging-actions",
        "Log destinations: memory, disk, console echo, remote syslog, email, or a script. \
         Built-in names (memory, disk, echo, remote, email) exist on typical routers.",
        "Configure the destination here, then point a Logging rule at it. An action is unused \
         until a rule uses its name. Fields follow Type: memory, disk, echo, remote, email, or \
         script. Remote syslog adds address, port, protocol (udp, tcp, or tls), format, and VRF. \
         Check Certificate appears only for TLS. Syslog Facility and Syslog Severity appear only \
         for BSD syslog; CEF Event Delimiter only for CEF.",
        "name, Type, then fields for the selected Type (memory lines, disk file, remote syslog, \
         email, or script).",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/log/"
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
        "Needed for api-ssl/WinBox TLS and several VPN types. Keys and passphrases stay \
         secret. Sign with g, import with p, export with w.",
        "name, common-name, key-usage, ca, file-name, type, passphrase, export-passphrase."
    ),
    guide!(
        "watchdog",
        "Hardware/software watchdog: reboot if the system stops pinging a target or hangs.",
        "Safety net on remote sites. A bad watch-address can reboot-loop the box. Fields edit in place; Ctrl+s patches.",
        "watch-address, watchdog-timer, watch-interval, no-ping-delay, ping-timeout, automatic-supout."
    ),
    guide!(
        "system-console",
        "Serial console bindings (`/system/console`): attach a local terminal to a `/port`.",
        "Not the in-app log pane. Disabling the last serial console can lock you out of that port.",
        "port, term, channel, disabled. Runtime used/free/wedged stay on Status.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
    ),
    guide!(
        "leds",
        "Per-LED bindings (`/system/led`): type, interface or modem, and which LEDs light.",
        "Map board LEDs to link or modem activity. LED Settings is the sibling singleton for all-off.",
        "type, interface, modem, leds, disabled.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/8978532/LEDs"
    ),
    guide!(
        "led-settings",
        "Board-wide LED settings (`/system/led/settings`).",
        "Turn every LED off immediately, after an hour, or never. Separate from per-LED bindings.",
        "all-leds-off.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/8978532/LEDs"
    ),
    guide!(
        "ports",
        "Serial port hardware (`/port`): baud, parity, and flow control.",
        "Console and Special Login look up these names. This is not the interactive serial terminal.",
        "name, baud-rate, data-bits, parity, stop-bits, flow-control.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
    ),
    guide!(
        "note",
        "Administrative note shown on login (banner-like text).",
        "Leave a contact or change warning for the next operator.",
        "note text, show-at-login."
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
    guide!(
        "package-update",
        "Package update channel and check/install (`/system package update`).",
        "Check for a new RouterOS build, then install (reboot-class). Distinct from RouterBOARD firmware.",
        "channel, installed-version, latest-version, status. Check and Install are actions."
    ),
    guide!(
        "reset-configuration",
        "Factory-style `/system reset-configuration` with flags on the page, then a confirm.",
        "Set keep-users, no-defaults, skip-backup, caps-mode, and run-after-reset here. Ctrl+s asks before POST. Destructive; never Safe Mode.",
        "keep-users, no-defaults, skip-backup, caps-mode, run-after-reset."
    ),
    guide!(
        "reboot",
        "Reboot this router (`/system reboot`).",
        "Opens a confirm as soon as you select the System item. Esc cancels without POST.",
        "No fields. Same warning as Resources used to show when Safe Mode is on."
    ),
    guide!(
        "shutdown",
        "Power off this router (`/system shutdown`).",
        "Opens a confirm as soon as you select the System item. Esc cancels without POST.",
        "No fields. Same warning as Resources used to show when Safe Mode is on."
    ),
    guide!(
        "ssh-keys",
        "User SSH public keys (`/user ssh-keys`).",
        "Install keys so operators can log in without a password. Private keys stay off this table unless the API exposes them.",
        "user, key-owner."
    ),
    guide!(
        "history",
        "Configuration history (`/system history`). Undo a selected row after a confirm prompt.",
        "See who changed what locally. Undo runs `/system history undo` for that row. It is not Safe Mode unroll; take or release Safe Mode with F4. Rows tagged F (floating-undo) are Safe Mode work that unrolls if that session dies.",
        "floating-undo, time, action, by, policy. Undo is a row action."
    ),
    guide!(
        "ipv6-dhcp-relay",
        "DHCPv6 relay (`/ipv6 dhcp-relay`).",
        "Forward DHCPv6 from a LAN to an off-box server.",
        "name, interface, dhcp-server, disabled."
    ),
    guide!(
        "ipv6-dhcp-bindings",
        "DHCPv6 server bindings (static and dynamic leases).",
        "Reserve a prefix/address for a DUID. Release/make-static follow IPv4 leases.",
        "address, duid, server, disabled."
    ),
    guide!(
        "ipv6-firewall-mangle",
        "IPv6 mangle table with move and reset-counters like IPv4.",
        "Mark or change IPv6 packets. Short New sheet: chain + action.",
        "chain, action, src-address, dst-address, protocol, in/out-interface, packets."
    ),
    guide!(
        "ipv6-firewall-raw",
        "IPv6 raw table (prerouting/output before conntrack).",
        "Drop or notrack early. Same filter actions as IPv4 raw.",
        "chain, action, src-address, dst-address, in/out-interface, packets."
    ),
    guide!(
        "ipv6-firewall-connections",
        "IPv6 connection tracking table: live conntrack entries the IPv6 firewall is following.",
        "Inspect who is talking through the router on IPv6. Remove drops that tracked entry so the \
         next packet is treated as a new connection; it does not delete a filter rule. IPv4 \
         connections are a separate table.",
        "src/dst-address, protocol, ports, tcp-state, timeout, orig-rate/repl-rate, \
         connection-mark. Reply addresses and other keys show in the inspector.",
        "https://manual.mikrotik.com/docs/firewall-and-quality-of-service/connection-tracking/"
    ),
    guide!(
        "rip-instances",
        "RIP instance (`/routing rip instance`) for RIP v1/v2 on ROS 7.",
        "Only if you still speak RIP with a neighbor. Prefer OSPF/BGP otherwise.",
        "name, vrf, originate-default, disabled."
    ),
    guide!(
        "rip-interface-templates",
        "RIP interface templates (which interfaces run RIP).",
        "Attach an instance to interfaces. Look up the instance name.",
        "instance, interfaces, disabled."
    ),
    guide!(
        "bfd",
        "BFD sessions/configuration (`/routing bfd configuration`).",
        "Faster neighbor failure detection for OSPF/BGP. Easy to flap a link if timers are too tight.",
        "interfaces, addresses, min-tx-interval, min-rx-interval, multiplier."
    ),
    guide!(
        "routing-filters",
        "ROS 7 routing filters (`/routing filter rule`) — the large chain/rule language.",
        "Control what OSPF/BGP accept or advertise. The rule body is a script-like filter.",
        "chain, rule, disabled, comment."
    ),
    guide!(
        "routing-id",
        "Routing IDs (`/routing id`) used by OSPF/BGP instances.",
        "Set a stable router-id selector instead of relying on a random address.",
        "name, id, select, disabled."
    ),
    guide!(
        "ospf-neighbors",
        "OSPF neighbor table. Monitor-only; no Add.",
        "See adjacency state. Configure instances/areas/templates elsewhere.",
        "instance, router-id, address, state, adjacency."
    ),
    guide!(
        "ospf-lsa",
        "OSPF LSA database. Monitor-only.",
        "Inspect what the area flooded. Not an editor.",
        "type, id, originator, area, sequence."
    ),
    guide!(
        "bgp-advertisements",
        "BGP advertisements table. Monitor-only.",
        "See what this router is announcing. VPN table is omitted unless the API lists it stably.",
        "prefix, nexthop, peer, as-path."
    ),
    guide!(
        "sniffer",
        "Packet sniffer start/stop, optional save-to-file on the router.",
        "Capture on-box; there is no live pcap UI. Start/stop are actions.",
        "interface, file-name, file-limit, filter-stream, filter-interface."
    ),
    guide!(
        "bandwidth-test",
        "Bandwidth-test overlay (client to a MikroTik bandwidth-test server).",
        "Measure throughput from this router. No graphs — streamed samples in the overlay.",
        "address, protocol, duration/count, direction/user if the server requires them."
    ),
    guide!(
        "flood-ping",
        "Flood-ping overlay for a burst of ICMP from the router.",
        "Stress a path briefly. Close the overlay to stop.",
        "address, count, src-address."
    ),
    guide!(
        "mac-scan",
        "MAC-scan overlay on a L2 interface.",
        "Discover neighbors by MAC on a LAN segment.",
        "interface (src), results: address/mac-address/age."
    ),
    guide!(
        "ip-scan",
        "IP-scan overlay for a range on an interface.",
        "Find which addresses answer on a subnet.",
        "address range, interface (src), mac-address, time."
    ),
    guide!(
        "profiler",
        "CPU profiler overlay (`/tool profile`).",
        "See which processes burn CPU. No WebFig-style graphs.",
        "samples of name, usage, load."
    ),
    guide!(
        "wol",
        "Wake-on-LAN one-shot (`/tool wol`).",
        "Send a magic packet out an interface to a MAC.",
        "interface, mac."
    ),
    guide!(
        "sms",
        "SMS send (`/tool sms send`) when the sms package exists.",
        "Send a short message via a modem channel. Skip if the package is absent.",
        "phone-number, message, channel."
    ),
    guide!(
        "radius-incoming",
        "RADIUS incoming (`/radius incoming`) — accept incoming RADIUS on a port.",
        "Needed for some disconnect/CoA setups. Not User Manager.",
        "accept, port."
    ),
    guide!(
        "logs",
        "Live log tail from `/log` (topics + message), newest first. This client keeps a \
         bounded local buffer; it does not delete logs on the router when you clear the view.",
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
        let mut ids: Vec<&str> = crate::features::interfaces::guides::GUIDES
            .iter()
            .chain(crate::features::wireguard::guides::GUIDES)
            .chain(crate::features::ppp::guides::GUIDES)
            .chain(crate::features::bridge::guides::GUIDES)
            .chain(GUIDES)
            .map(|(id, _)| *id)
            .collect();
        let original = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), original, "duplicate screen guide ids");
        assert!(screen_guide(DASHBOARD_ID).is_some());
        for spec in ALL_RESOURCES.iter() {
            assert!(
                screen_guide(spec.id).is_some(),
                "missing screen guide for {}",
                spec.id
            );
        }
    }

    #[test]
    fn neighbors_guide_mentions_connect_tab() {
        let guide = screen_guide("neighbors").expect("neighbors");
        assert!(guide.use_when.contains("Connect"), "{}", guide.use_when);
        assert!(guide.use_when.contains("device tab"), "{}", guide.use_when);
        let copy = about_copy("neighbors").expect("copy");
        assert!(copy.kicker.contains("/ip/neighbor"));
        assert!(!copy.body.contains('\u{2014}'));
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
    fn lte_apn_guide_points_at_the_cli_reference() {
        let guide = screen_guide("lte-apn").expect("lte-apn");
        let copy = about_copy("lte-apn").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["APN", "authentication", "use-network-apn"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.kicker.contains("/interface/lte/apn"));
        assert!(copy.body.contains("/interface/lte/apn"));
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
        assert!(!copy.body.contains('\u{2014}'));
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

    #[test]
    fn ipv6_firewall_connections_guide_mirrors_ipv4() {
        let guide = screen_guide("ipv6-firewall-connections").expect("guide");
        let copy = about_copy("ipv6-firewall-connections").expect("copy");
        assert_eq!(copy.title, "About Connections");
        assert!(copy.kicker.contains("/ipv6/firewall/connection"));
        assert!(copy.body.contains("/ipv6/firewall/connection"));
        assert!(guide.summary.to_ascii_lowercase().contains("ipv6"));
        assert!(guide.use_when.to_ascii_lowercase().contains("remove"));
        assert!(guide.fields.contains("src/dst-address"));
        assert_eq!(
            guide.docs_url,
            Some(
                "https://manual.mikrotik.com/docs/firewall-and-quality-of-service/connection-tracking/"
            )
        );
        assert!(
            !copy.body.contains('\u{2014}'),
            "about copy must not use em dashes"
        );
    }

    #[test]
    fn ospf_interface_guide_is_runtime_not_a_template() {
        let live = about_copy("ospf-interfaces").expect("ospf-interfaces");
        let templates = about_copy("ospf-interface-templates").expect("templates");
        assert_eq!(live.title, "About OSPF Interface");
        assert_eq!(templates.title, "About OSPF Interface Templates");
        assert!(live.kicker.contains("/routing/ospf/interface"));
        assert!(!live.kicker.contains("interface-template"));
        assert!(templates.kicker.contains("interface-template"));
        assert!(live.body.contains("Monitor-only"));
        assert!(live.body.contains("cost"));
        assert!(templates.body.contains("OSPF Interface"));
        assert!(!templates.body.contains("no separate"));
        assert!(
            live.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/routing/ospf/interface/")
        );
    }

    #[test]
    fn traffic_flow_and_igmp_guides_track_the_manual() {
        let flow = about_copy("traffic-flow").expect("traffic-flow");
        assert_eq!(flow.title, "About Traffic Flow");
        assert!(flow.kicker.contains("/ip/traffic-flow"));
        assert!(flow.body.contains("NetFlow") || flow.body.contains("IPFIX"));
        assert!(flow.body.contains("Packet Sampling"));
        assert!(!flow.body.contains('\u{2014}'));

        let targets = about_copy("traffic-flow-targets").expect("targets");
        assert!(targets.body.contains("Dst. Address"));
        assert!(targets.body.contains("ipfix"));

        let proxy = about_copy("igmp-proxy").expect("igmp-proxy");
        assert!(proxy.kicker.contains("/routing/igmp-proxy"));
        assert!(proxy.body.contains("upstream"));
        assert!(!proxy.body.contains('\u{2014}'));

        let ifaces = about_copy("igmp-proxy-interfaces").expect("ifaces");
        assert!(ifaces.body.contains("Upstream"));
        let mfc = about_copy("igmp-proxy-mfc").expect("mfc");
        assert!(mfc.body.contains("Group"));
    }

    #[test]
    fn romon_and_graphing_guides_track_the_manual() {
        let romon = about_copy("romon").expect("romon");
        let ports = about_copy("romon-ports").expect("ports");
        let graphing = about_copy("graphing").expect("graphing");
        assert_eq!(romon.title, "About RoMON");
        assert_eq!(ports.title, "About RoMON Ports");
        assert_eq!(graphing.title, "About Graphing");
        assert!(romon.kicker.contains("/tool/romon"));
        assert!(ports.kicker.contains("/tool/romon/port"));
        assert!(graphing.kicker.contains("/tool/graphing"));
        assert!(romon.body.to_ascii_lowercase().contains("secret"));
        assert!(ports.body.contains("forbid"));
        assert!(graphing.body.contains("/graphs/"));
        assert!(
            about_copy("graphing-interface")
                .expect("gi")
                .kicker
                .contains("/tool/graphing/interface")
        );
        assert!(
            about_copy("graphing-queue")
                .expect("gq")
                .kicker
                .contains("/tool/graphing/queue")
        );
        assert!(
            about_copy("graphing-resource")
                .expect("gr")
                .kicker
                .contains("/tool/graphing/resource")
        );
        for copy in [&romon, &ports, &graphing] {
            assert!(
                !copy.body.contains('\u{2014}'),
                "about copy must not use em dashes"
            );
        }
        assert_eq!(
            screen_guide("romon").expect("g").docs_url,
            Some("https://manual.mikrotik.com/docs/management-tools/romon/")
        );
        assert_eq!(
            screen_guide("graphing").expect("g").docs_url,
            Some(
                "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
            )
        );
    }

    #[test]
    fn history_guide_covers_undo_and_keeps_safe_mode_separate() {
        let copy = about_copy("history").expect("history");
        assert_eq!(copy.title, "About History");
        assert!(copy.kicker.contains("/system/history"));
        assert!(copy.body.contains("undo"));
        assert!(copy.body.contains("F4"));
        assert!(copy.body.contains("Safe Mode"));
        assert!(copy.body.contains("floating-undo"));
    }
}
