//! Feature-owned form schemas for virtual and miscellaneous Interfaces.

#![allow(clippy::wildcard_imports)]

use super::*;
use crate::forms::{EnumChoice, FieldRule, ScalarKind};

const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const ACTUAL_MTU: FieldSpec = f!("actual-mtu", "Actual MTU", FieldKind::Readonly);
const READONLY_L2_MTU: FieldSpec = f!("l2mtu", "L2 MTU", FieldKind::Readonly);
const READONLY_VRF: FieldSpec = f!("vrf", "VRF", FieldKind::Readonly);
const READONLY_MAC: FieldSpec = f!("mac-address", "MAC Address", FieldKind::Readonly);
const OPTIONAL_MAC: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Mac,
    unset: "",
    unset_label: "auto",
};

const MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "private",
        value: "private",
    },
    EnumChoice {
        label: "bridge",
        value: "bridge",
    },
];

const DETECT_STATE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "no link",
        value: "no-link",
    },
    EnumChoice {
        label: "unknown",
        value: "unknown",
    },
    EnumChoice {
        label: "lan",
        value: "lan",
    },
    EnumChoice {
        label: "wan",
        value: "wan",
    },
    EnumChoice {
        label: "internet",
        value: "internet",
    },
    EnumChoice {
        label: "slave",
        value: "slave",
    },
];

const LINK_STATUS_FIELDS: &[FieldSpec] = &[
    f!(
        "last-link-down-time",
        "Last Link Down Time",
        FieldKind::Readonly
    ),
    f!(
        "last-link-up-time",
        "Last Link Up Time",
        FieldKind::Readonly
    ),
    f!("link-downs", "Link Downs", FieldKind::Readonly),
];

const TRAFFIC_FIELDS: &[FieldSpec] = &[
    f!("tx-byte", "Tx Bytes", FieldKind::Readonly),
    f!("rx-byte", "Rx Bytes", FieldKind::Readonly),
    f!("tx-packet", "Tx Packets", FieldKind::Readonly),
    f!("rx-packet", "Rx Packets", FieldKind::Readonly),
    f!("fp-tx-byte", "FP Tx Bytes", FieldKind::Readonly),
    f!("fp-rx-byte", "FP Rx Bytes", FieldKind::Readonly),
    f!("fp-tx-packet", "FP Tx Packets", FieldKind::Readonly),
    f!("fp-rx-packet", "FP Rx Packets", FieldKind::Readonly),
    f!("tx-drop", "Tx Drops", FieldKind::Readonly),
    f!("rx-drop", "Rx Drops", FieldKind::Readonly),
    f!("tx-queue-drop", "Tx Queue Drops", FieldKind::Readonly),
    f!("tx-error", "Tx Errors", FieldKind::Readonly),
    f!("rx-error", "Rx Errors", FieldKind::Readonly),
];

/// Family-owned field predicates for later aggregation by `interfaces::rules`.
///
/// 7.21.5 does not condition VETH's static controls on DHCP, so this
/// family deliberately contributes no visibility rule.
pub(crate) const FIELD_RULES: &[FieldRule] = &[];

/// Capability predicates which cannot yet be attached to `ResourceSpec`.
#[allow(dead_code)]
pub(crate) const SYSCAP_GATES: &[(&str, u32)] = &[("veth", 2)];

/// Architecture exclusions which cannot yet be attached to `ResourceSpec`.
#[allow(dead_code)]
pub(crate) const HIDDEN_ARCHITECTURES: &[(&str, &[&str])] =
    &[("macsec", &["smips"]), ("macsec-profiles", &["smips"])];

/// The VRF row selector combines both source maps in the official schema.
#[allow(dead_code)]
pub(crate) const VRF_INTERFACE_SELECTOR_SOURCES: &[&str] = &["interfaces", "interface-lists"];

const LIST_SECTIONS: &[FormSection] = &[FormSection {
    id: "form",
    label: "",
    read_only: false,
    fields: &[
        COMMENT,
        NAME,
        f!("include", "Include", LOOKUP_IFACE_LISTS),
        f!("exclude", "Exclude", LOOKUP_IFACE_LISTS),
    ],
}];

pub static LIST_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: LIST_SECTIONS,
    create_sections: LIST_SECTIONS,
};

const MEMBER_SECTIONS: &[FormSection] = &[FormSection {
    id: "form",
    label: "",
    read_only: false,
    fields: &[
        ENABLED,
        COMMENT,
        f!("list", "List", LOOKUP_IFACE_LIST),
        INTERFACE,
    ],
}];

pub static MEMBER_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["list"],
    sections: MEMBER_SECTIONS,
    create_sections: MEMBER_SECTIONS,
};

const MACVLAN_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            COMMENT,
            NAME,
            DEFAULT_NAME,
            IFACE_TYPE,
            f!(
                "mtu",
                "MTU",
                FieldKind::ConstrainedNumber {
                    min: Some(64),
                    max: Some(65_535)
                }
            ),
            ACTUAL_MTU,
            READONLY_L2_MTU,
            READONLY_VRF,
            f!("mac-address", "MAC Address", OPTIONAL_MAC),
            INTERFACE,
            f!(
                "mode",
                "Mode",
                FieldKind::LabeledEnum {
                    choices: MODE_CHOICES
                }
            ),
        ],
    },
    FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: LINK_STATUS_FIELDS,
    },
    FormSection {
        id: "traffic",
        label: "Traffic",
        read_only: true,
        fields: TRAFFIC_FIELDS,
    },
];

pub static MACVLAN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "mode"],
    sections: MACVLAN_SECTIONS,
    create_sections: MACVLAN_SECTIONS,
};

const VETH_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            COMMENT,
            NAME,
            DEFAULT_NAME,
            IFACE_TYPE,
            ACTUAL_MTU,
            READONLY_L2_MTU,
            READONLY_VRF,
            f!("mac-address", "MAC Address", OPTIONAL_MAC),
            f!(
                "container-mac-address",
                "Container MAC Address",
                OPTIONAL_MAC
            ),
            f!("address", "Address", FieldKind::Repeat),
            f!("gateway", "Gateway", FieldKind::Ip),
            f!("gateway6", "IPv6 Gateway", FieldKind::Ipv6),
            f!("dhcp", "DHCP", FieldKind::Toggle),
            f!("dhcp-address", "DHCP Address", FieldKind::Readonly),
        ],
    },
    FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: LINK_STATUS_FIELDS,
    },
    FormSection {
        id: "traffic",
        label: "Traffic",
        read_only: true,
        fields: TRAFFIC_FIELDS,
    },
];

pub static VETH_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["address"],
    sections: VETH_SECTIONS,
    create_sections: VETH_SECTIONS,
};

const MACSEC_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            COMMENT,
            NAME,
            DEFAULT_NAME,
            IFACE_TYPE,
            f!(
                "mtu",
                "MTU",
                FieldKind::ConstrainedNumber {
                    min: Some(64),
                    max: Some(65_535)
                }
            ),
            ACTUAL_MTU,
            READONLY_L2_MTU,
            READONLY_VRF,
            READONLY_MAC,
            INTERFACE,
            f!("cak", "CAK", FieldKind::Secret),
            f!("ckn", "CKN", FieldKind::Raw),
            f!("profile", "Profile", LOOKUP_MACSEC_PROFILE),
            f!("status", "Status", FieldKind::Readonly),
        ],
    },
    FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: LINK_STATUS_FIELDS,
    },
    FormSection {
        id: "traffic",
        label: "Traffic",
        read_only: true,
        fields: TRAFFIC_FIELDS,
    },
];

pub static MACSEC_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "status"],
    sections: MACSEC_SECTIONS,
    create_sections: MACSEC_SECTIONS,
};

const MACSEC_PROFILE_SECTIONS: &[FormSection] = &[FormSection {
    id: "form",
    label: "",
    read_only: false,
    fields: &[
        NAME,
        f!(
            "server-priority",
            "Server Priority",
            FieldKind::ConstrainedNumber {
                min: Some(0),
                max: Some(255)
            }
        ),
        f!("default", "Default", FieldKind::Readonly),
    ],
}];

/// Create default for `/interface/macsec/profile`.
#[allow(dead_code)]
pub(crate) const MACSEC_PROFILE_DEFAULTS: &[(&str, &str)] = &[("server-priority", "10")];

pub static MACSEC_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["server-priority"],
    sections: MACSEC_PROFILE_SECTIONS,
    create_sections: MACSEC_PROFILE_SECTIONS,
};

const VRF_SECTIONS: &[FormSection] = &[FormSection {
    id: "form",
    label: "",
    read_only: false,
    fields: &[
        NAME,
        f!("interfaces", "Interfaces", FieldKind::Repeat),
        ENABLED,
        f!("builtin", "Builtin", FieldKind::Readonly),
        COMMENT,
    ],
}];

pub static VRF_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interfaces"],
    sections: VRF_SECTIONS,
    create_sections: VRF_SECTIONS,
};

const DETECT_INTERNET_SECTIONS: &[FormSection] = &[FormSection {
    id: "form",
    label: "",
    read_only: false,
    fields: &[
        f!(
            "detect-interface-list",
            "Detect Interface List",
            LOOKUP_IFACE_LIST
        ),
        f!(
            "lan-interface-list",
            "LAN Interface List",
            LOOKUP_IFACE_LIST
        ),
        f!(
            "wan-interface-list",
            "WAN Interface List",
            LOOKUP_IFACE_LIST
        ),
        f!(
            "internet-interface-list",
            "Internet Interface List",
            LOOKUP_IFACE_LIST
        ),
    ],
}];

pub static DETECT_INTERNET_FORM: FormSchema = FormSchema {
    title_key: "detect-interface-list",
    subtitle_keys: &[],
    sections: DETECT_INTERNET_SECTIONS,
    create_sections: &[],
};

#[allow(dead_code)]
pub static DETECT_INTERNET_STATE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["state"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: true,
        fields: &[
            f!("name", "Interface", FieldKind::Readonly),
            f!(
                "state",
                "State",
                FieldKind::LabeledEnum {
                    choices: DETECT_STATE_CHOICES
                }
            ),
        ],
    }],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(schema: &FormSchema, section: &str) -> Vec<&'static str> {
        schema
            .sections
            .iter()
            .find(|candidate| candidate.id == section)
            .expect("section")
            .fields
            .iter()
            .map(|field| field.key)
            .collect()
    }

    #[test]
    fn unsectioned_forms_keep_webfig_order() {
        assert_eq!(
            keys(&LIST_FORM, "form"),
            ["comment", "name", "include", "exclude"]
        );
        assert_eq!(
            keys(&MEMBER_FORM, "form"),
            ["disabled", "comment", "list", "interface"]
        );
        assert_eq!(
            keys(&VRF_FORM, "form"),
            ["name", "interfaces", "disabled", "builtin", "comment"]
        );
    }

    #[test]
    fn virtual_interfaces_have_exact_tabs() {
        for schema in [&MACVLAN_FORM, &VETH_FORM, &MACSEC_FORM] {
            assert_eq!(
                schema
                    .sections
                    .iter()
                    .map(|section| section.id)
                    .collect::<Vec<_>>(),
                ["general", "status", "traffic"]
            );
            assert_eq!(schema.sections, schema.create_sections);
        }
        assert!(MACVLAN_FORM.field("arp").is_none());
        assert!(MACVLAN_FORM.field("loop-protect").is_none());
        assert_eq!(VETH_FORM.field("gateway").unwrap().kind, FieldKind::Ip);
        assert_eq!(VETH_FORM.field("gateway6").unwrap().kind, FieldKind::Ipv6);
        assert_eq!(MACSEC_FORM.field("cak").unwrap().kind, FieldKind::Secret);
        assert_eq!(MACSEC_FORM.field("ckn").unwrap().kind, FieldKind::Raw);
    }

    #[test]
    fn profile_and_detect_internet_use_exact_contracts() {
        assert_eq!(
            MACSEC_PROFILE_FORM.field("server-priority").unwrap().kind,
            FieldKind::ConstrainedNumber {
                min: Some(0),
                max: Some(255)
            }
        );
        assert!(DETECT_INTERNET_FORM.field("request-interval").is_none());
        assert!(DETECT_INTERNET_FORM.field("state").is_none());
        assert_eq!(keys(&DETECT_INTERNET_STATE_FORM, "form"), ["name", "state"]);
        assert_eq!(
            DETECT_INTERNET_STATE_FORM
                .field("state")
                .unwrap()
                .kind
                .display_value("no-link"),
            "no link"
        );
    }

    #[test]
    fn gates_are_family_owned() {
        assert_eq!(FIELD_RULES, []);
        assert_eq!(SYSCAP_GATES, [("veth", 2)]);
        assert_eq!(
            HIDDEN_ARCHITECTURES,
            [
                ("macsec", &["smips"][..]),
                ("macsec-profiles", &["smips"][..])
            ]
        );
        assert_eq!(
            VRF_INTERFACE_SELECTOR_SOURCES,
            ["interfaces", "interface-lists"]
        );
    }
}
