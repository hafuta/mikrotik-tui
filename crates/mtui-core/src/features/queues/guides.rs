//! Feature-owned operator guides for Queues screens.

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
];
