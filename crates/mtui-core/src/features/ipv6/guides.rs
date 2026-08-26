//! Feature-owned operator guides for `IPv6` screens.

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
];
