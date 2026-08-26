//! Feature-owned operator guides for Switch screens.

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
];
