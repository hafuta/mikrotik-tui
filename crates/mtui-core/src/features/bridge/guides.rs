//! Feature-owned operator guides for Bridge screens.

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
];
