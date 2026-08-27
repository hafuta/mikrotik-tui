//! Feature-owned operator guides for RADIUS screens.

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
        "radius",
        "RADIUS clients: where to send AAA for login, PPP, Hotspot, DHCP, wireless, and \
         similar services.",
        "Add your RADIUS server and enable the matching service. Incoming RADIUS (if used) \
         is a related system setting.",
        "address, protocol, secret, service, timeout, src-address, disabled."
    ),
    guide!(
        "radius-incoming",
        "RADIUS incoming (`/radius incoming`) — accept incoming RADIUS on a port.",
        "Needed for some disconnect/CoA setups. Not User Manager.",
        "accept, port."
    ),
];
