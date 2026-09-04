//! Feature-owned operator guides for `Routing` screens.

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
        "Peering with ISPs or other ASes. Live established state is on BGP Sessions, not \
         this table.",
        "name, remote.address, remote.as, local.role, disabled."
    ),
    guide!(
        "bgp-sessions",
        "Live BGP sessions: established flag, uptime, prefix-count, and last notification.",
        "Incident view for a peer that will not come up. Configure remote address and role \
         on BGP connections. Monitor-only; there is no Add.",
        "name, remote.address, remote.as, established, uptime, prefix-count, ebgp, \
         last-started, last-stopped.",
        "https://manual.mikrotik.com/docs/cli-reference/routing/bgp/session/"
    ),
    guide!(
        "bgp-templates",
        "Reusable BGP session defaults (AS, router-id, address-families) for connections.",
        "Put common peering options on a template, then point connections at it.",
        "name, as, router-id, address-families, output.network, disabled."
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
];
