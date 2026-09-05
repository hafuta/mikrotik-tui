//! Feature-owned form schemas for the `Routing` navigation group.
//!
//! `RouterOS` 7 writes `OSPF` interface parameters on `/routing/ospf/interface-template`.
//! `/routing/ospf/interface` is the matched live object (cost, state, DR/BDR).
//! `/routing/bgp/session` is the live peer cache; connections and templates stay writable.

use crate::form_fields::{
    KIND_ORIGINATE_DEFAULT, LOOKUP_ADDRESS_LIST, LOOKUP_INTERFACES_MULTI, LOOKUP_ROUTING_TABLES,
    LOOKUP_VRF,
};
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

const LOOKUP_OSPF_INSTANCE: FieldKind = FieldKind::Lookup {
    resource_id: "ospf-instances",
    value_key: "name",
    multiple: false,
};
const LOOKUP_OSPF_AREA: FieldKind = FieldKind::Lookup {
    resource_id: "ospf-areas",
    value_key: "name",
    multiple: false,
};
const LOOKUP_RIP_INSTANCE: FieldKind = FieldKind::Lookup {
    resource_id: "rip-instances",
    value_key: "name",
    multiple: false,
};

const OSPF_AREA_TYPE_VALUES: &[&str] = &["backbone", "standard", "stub", "nssa"];
const OSPF_NETWORK_TYPE_VALUES: &[&str] = &[
    "broadcast",
    "nbma",
    "ptp",
    "ptmp",
    "ptp-unnumbered",
    "virtual-link",
];
const OSPF_VERSION_VALUES: &[&str] = &["2", "3"];
const ROUTING_RULE_ACTION_VALUES: &[&str] = &[
    "lookup",
    "lookup-only",
    "unreachable",
    "blackhole",
    "prohibit",
];
const BGP_LOCAL_ROLE_VALUES: &[&str] = &[
    "ibgp",
    "ibgp-rr",
    "ibgp-rrclient",
    "ebgp",
    "ebgp-customer",
    "ebgp-peer",
    "ebgp-provider",
    "ebgp-rs",
    "ebgp-rs-client",
];
const ROUTING_ID_SELECT_VALUES: &[&str] = &["any", "only-dynamic", "only-static"];

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const TABLE: FieldSpec = f!("table", "Table", LOOKUP_ROUTING_TABLES);
const OSPF_INSTANCE: FieldSpec = f!("instance", "Instance", LOOKUP_OSPF_INSTANCE);
const OSPF_AREA: FieldSpec = f!("area", "Area", LOOKUP_OSPF_AREA);
const OSPF_NETWORK_TYPE: FieldSpec = f!(
    "type",
    "Type",
    FieldKind::Enum {
        values: OSPF_NETWORK_TYPE_VALUES,
    }
);

pub static ROUTING_TABLE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("fib", "FIB", FieldKind::Flag), COMMENT],
    }],
    create_sections: &[],
};

pub static ROUTING_RULE_FORM: FormSchema = FormSchema {
    title_key: "action",
    subtitle_keys: &["table"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("src-address", "Src. Address", FieldKind::Ip),
            f!("dst-address", "Dst. Address", FieldKind::Ip),
            f!("routing-mark", "Routing Mark", FieldKind::Text),
            f!(
                "action",
                "Action",
                FieldKind::Enum {
                    values: ROUTING_RULE_ACTION_VALUES,
                }
            ),
            TABLE,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static OSPF_INSTANCE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["router-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!(
                "version",
                "Version",
                FieldKind::Enum {
                    values: OSPF_VERSION_VALUES,
                }
            ),
            f!("router-id", "Router ID", FieldKind::Ip),
            f!(
                "originate-default",
                "Originate Default",
                KIND_ORIGINATE_DEFAULT
            ),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static OSPF_AREA_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["area-id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            OSPF_INSTANCE,
            f!("area-id", "Area ID", FieldKind::Ip),
            f!(
                "type",
                "Type",
                FieldKind::Enum {
                    values: OSPF_AREA_TYPE_VALUES
                }
            ),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static OSPF_INTERFACE_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "instance",
    subtitle_keys: &["area"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            OSPF_INSTANCE,
            OSPF_AREA,
            f!("interfaces", "Interfaces", LOOKUP_INTERFACES_MULTI),
            OSPF_NETWORK_TYPE,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

/// Live `/routing/ospf/interface` rows. `RouterOS` marks this menu read-only.
pub static OSPF_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["state"],
    sections: &[FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: &[
            f!("address", "Address", FieldKind::Readonly),
            f!("area", "Area", FieldKind::Readonly),
            f!("state", "State", FieldKind::Readonly),
            f!("network-type", "Network Type", FieldKind::Readonly),
            f!("cost", "Cost", FieldKind::Readonly),
            f!("priority", "Priority", FieldKind::Readonly),
            f!("dr", "DR", FieldKind::Readonly),
            f!("bdr", "BDR", FieldKind::Readonly),
            f!("hello-interval", "Hello Interval", FieldKind::Readonly),
            f!("dead-interval", "Dead Interval", FieldKind::Readonly),
            f!(
                "retransmit-interval",
                "Retransmit Interval",
                FieldKind::Readonly
            ),
            f!("transmit-delay", "Transmit Delay", FieldKind::Readonly),
            f!("dynamic", "Dynamic", FieldKind::Readonly),
        ],
    }],
    create_sections: &[],
};

pub static BGP_CONNECTION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["remote.address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("remote.address", "Remote Address", FieldKind::Ip),
            f!("remote.as", "Remote AS", FieldKind::Number),
            f!(
                "local.role",
                "Local Role",
                FieldKind::Enum {
                    values: BGP_LOCAL_ROLE_VALUES,
                }
            ),
            f!("local.address", "Local Address", FieldKind::Ip),
            f!("connect", "Connect", FieldKind::Toggle),
            f!("listen", "Listen", FieldKind::Toggle),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

/// Live `/routing/bgp/session` rows. `RouterOS` marks this menu read-only.
pub static BGP_SESSION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["established", "remote.address"],
    sections: &[FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: &[
            f!("name", "Name", FieldKind::Readonly),
            f!("instance", "Instance", FieldKind::Readonly),
            f!("remote.address", "Remote Address", FieldKind::Readonly),
            f!("remote.as", "Remote AS", FieldKind::Readonly),
            f!("remote.id", "Remote ID", FieldKind::Readonly),
            f!("remote.afi", "Remote AFI", FieldKind::Readonly),
            f!("local.address", "Local Address", FieldKind::Readonly),
            f!("local.role", "Local Role", FieldKind::Readonly),
            f!("established", "Established", FieldKind::Readonly),
            f!("ebgp", "eBGP", FieldKind::Readonly),
            f!("ibgp", "iBGP", FieldKind::Readonly),
            f!("stopped", "Stopped", FieldKind::Readonly),
            f!("limit-exceeded", "Limit Exceeded", FieldKind::Readonly),
            f!("uptime", "Uptime", FieldKind::Readonly),
            f!("last-started", "Last Started", FieldKind::Readonly),
            f!("last-stopped", "Last Stopped", FieldKind::Readonly),
            f!("prefix-count", "Prefix Count", FieldKind::Readonly),
            f!("hold-time", "Hold Time", FieldKind::Readonly),
            f!("keepalive-time", "Keepalive Time", FieldKind::Readonly),
            f!("remote.messages", "Remote Messages", FieldKind::Readonly),
            f!("local.messages", "Local Messages", FieldKind::Readonly),
            f!(
                "input.last-notification",
                "Input Last Notification",
                FieldKind::Readonly
            ),
            f!(
                "output.last-notification",
                "Output Last Notification",
                FieldKind::Readonly
            ),
            f!("multihop", "Multihop", FieldKind::Readonly),
            f!("use-bfd", "Use BFD", FieldKind::Readonly),
        ],
    }],
    create_sections: &[],
};

pub static BGP_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["as"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("as", "AS", FieldKind::Number),
            f!("router-id", "Router ID", FieldKind::Ip),
            f!("address-families", "Address Families", FieldKind::Repeat),
            f!("output.network", "Output Network", LOOKUP_ADDRESS_LIST),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static RIP_INSTANCE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vrf"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("vrf", "VRF", LOOKUP_VRF),
            f!(
                "originate-default",
                "Originate Default",
                KIND_ORIGINATE_DEFAULT
            ),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static RIP_INTERFACE_TEMPLATE_FORM: FormSchema = FormSchema {
    title_key: "interfaces",
    subtitle_keys: &["instance"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("instance", "Instance", LOOKUP_RIP_INSTANCE),
            f!("interfaces", "Interfaces", LOOKUP_INTERFACES_MULTI),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static BFD_CONFIGURATION_FORM: FormSchema = FormSchema {
    title_key: "interfaces",
    subtitle_keys: &["addresses"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interfaces", "Interfaces", LOOKUP_INTERFACES_MULTI),
            f!("addresses", "Addresses", FieldKind::Repeat),
            f!("min-tx-interval", "Min TX Interval", FieldKind::Time),
            f!("min-rx-interval", "Min RX Interval", FieldKind::Time),
            f!("multiplier", "Multiplier", FieldKind::Number),
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static ROUTING_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["rule"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("chain", "Chain", FieldKind::Text),
            f!("rule", "Rule", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static ROUTING_ID_FORM: FormSchema = FormSchema {
    title_key: "id",
    subtitle_keys: &["name"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("id", "ID", FieldKind::Ip),
            f!(
                "select",
                "Select",
                FieldKind::Enum {
                    values: ROUTING_ID_SELECT_VALUES,
                }
            ),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};
