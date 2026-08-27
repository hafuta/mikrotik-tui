//! Feature-owned operator guides for `Tools` screens.

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
        "Which interfaces (or interface lists) take part in `RoMON`, with a cost and optional \
         per-port secrets. A default all entry is present on typical routers.",
        "Restrict `RoMON` to backbone ports, raise cost on slower links, or forbid a WAN. \
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
];
