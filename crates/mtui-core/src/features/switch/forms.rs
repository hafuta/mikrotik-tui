//! Feature-owned form schemas for the Switch navigation group.

use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_SWITCH: FieldKind = FieldKind::Lookup {
    resource_id: "switch",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SWITCH_PORT: FieldKind = FieldKind::Lookup {
    resource_id: "switch-port",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SWITCH_PORTS: FieldKind = FieldKind::Lookup {
    resource_id: "switch-port",
    value_key: "name",
    multiple: true,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const NAME_RO: FieldSpec = f!("name", "Name", FieldKind::Readonly);
const SWITCH: FieldSpec = f!("switch", "Switch", LOOKUP_SWITCH);
const SWITCH_RO: FieldSpec = f!("switch", "Switch", FieldKind::Readonly);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const PORTS: FieldSpec = f!("ports", "Ports", LOOKUP_SWITCH_PORTS);
const VLAN_ID: FieldSpec = f!("vlan-id", "VLAN ID", FieldKind::Number);
const L3HW: FieldSpec = f!("l3-hw-offloading", "L3 HW Offloading", FieldKind::Toggle);

pub static SWITCH_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("mirror-source", "Mirror Source", LOOKUP_SWITCH_PORT),
                f!("mirror-target", "Mirror Target", LOOKUP_SWITCH_PORT),
                f!(
                    "mirror-egress-target",
                    "Mirror Egress Target",
                    LOOKUP_SWITCH_PORT
                ),
                f!("cpu-flow-control", "CPU Flow Control", FieldKind::Toggle),
                L3HW,
                f!("switch-all-ports", "Switch All Ports", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("type", "Type", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static SWITCH_PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["switch"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME_RO,
                SWITCH_RO,
                f!("vlan-mode", "VLAN Mode", FieldKind::Text),
                f!("vlan-header", "VLAN Header", FieldKind::Text),
                f!("default-vlan-id", "Default VLAN ID", FieldKind::Number),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("ingress-rate", "Ingress Rate", FieldKind::Text),
                f!("egress-rate", "Egress Rate", FieldKind::Text),
                f!("storm-rate", "Storm Rate", FieldKind::Text),
                L3HW,
                f!("mirror-ingress", "Mirror Ingress", FieldKind::Toggle),
                f!("mirror-egress", "Mirror Egress", FieldKind::Toggle),
            ],
        },
    ],
    create_sections: &[],
};

pub static SWITCH_VLAN_FORM: FormSchema = FormSchema {
    title_key: "switch",
    subtitle_keys: &["vlan-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            SWITCH,
            VLAN_ID,
            PORTS,
            f!(
                "independent-learning",
                "Independent Learning",
                FieldKind::Toggle
            ),
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static SWITCH_RULE_FORM: FormSchema = FormSchema {
    title_key: "switch",
    subtitle_keys: &["ports"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[SWITCH, PORTS, COMMENT, ENABLED],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                f!("mac-protocol", "MAC Protocol", FieldKind::Text),
                f!("src-mac-address", "Src. MAC Address", FieldKind::Text),
                f!("dst-mac-address", "Dst. MAC Address", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
                f!("src-address", "Src. Address", FieldKind::Text),
                f!("dst-address", "Dst. Address", FieldKind::Text),
                f!("src-port", "Src. Port", FieldKind::Text),
                f!("dst-port", "Dst. Port", FieldKind::Text),
                VLAN_ID,
                f!("new-dst-ports", "New Dst. Ports", LOOKUP_SWITCH_PORTS),
                f!("redirect-to-cpu", "Redirect To CPU", FieldKind::Toggle),
                f!("mirror", "Mirror", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("invalid", "Invalid", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

pub static SWITCH_PORT_ISOLATION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["switch"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME_RO,
            SWITCH_RO,
            f!(
                "forwarding-override",
                "Forwarding Override",
                LOOKUP_SWITCH_PORTS
            ),
        ],
    }],
    create_sections: &[],
};

pub static SWITCH_L3HW_FORM: FormSchema = FormSchema {
    title_key: "autorestart",
    subtitle_keys: &[],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("autorestart", "Autorestart", FieldKind::Toggle),
                f!("fasttrack-hw", "FastTrack HW", FieldKind::Toggle),
                f!("ipv6-hw", "IPv6 HW", FieldKind::Toggle),
                f!(
                    "icmp-reply-on-error",
                    "ICMP Reply On Error",
                    FieldKind::Toggle
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!(
                "hw-supports-fasttrack",
                "HW Supports FastTrack",
                FieldKind::Readonly
            )],
        },
    ],
    create_sections: &[],
};
