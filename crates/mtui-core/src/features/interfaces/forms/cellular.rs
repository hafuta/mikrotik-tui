//! Feature-owned 7.21.5 form schemas for LTE and LTE APN.

#![allow(clippy::wildcard_imports)]

use super::*;
use crate::forms::{BoxedFieldPredicate, EnumChoice, FieldPredicate, FieldRule, ScalarKind};

const OPTIONAL_TEXT: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Text,
    unset: "",
    unset_label: "none",
};

const IP_TYPE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Auto",
        value: "auto",
    },
    EnumChoice {
        label: "IPv4",
        value: "ipv4",
    },
    EnumChoice {
        label: "IPv6",
        value: "ipv6",
    },
];

const AUTH_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "CHAP",
        value: "chap",
    },
    EnumChoice {
        label: "PAP",
        value: "pap",
    },
];

const AUTH_SET: FieldPredicate = FieldPredicate::Any(&[
    FieldPredicate::Equals {
        key: "authentication",
        value: "pap",
    },
    FieldPredicate::Equals {
        key: "authentication",
        value: "chap",
    },
]);
const DEFAULT_ROUTE: FieldPredicate = FieldPredicate::Truthy("add-default-route");
const PASSTHROUGH_NONE: FieldPredicate = FieldPredicate::Equals {
    key: "passthrough-interface",
    value: "none",
};
const PASSTHROUGH: FieldPredicate = FieldPredicate::All(&[
    FieldPredicate::NonEmpty("passthrough-interface"),
    FieldPredicate::Not(BoxedFieldPredicate(&PASSTHROUGH_NONE)),
]);

pub(crate) const FIELD_RULES: &[FieldRule] = &[
    FieldRule {
        resource_id: "lte-apn",
        field_key: "user",
        visible: AUTH_SET,
        enabled: AUTH_SET,
    },
    FieldRule {
        resource_id: "lte-apn",
        field_key: "password",
        visible: AUTH_SET,
        enabled: AUTH_SET,
    },
    FieldRule {
        resource_id: "lte-apn",
        field_key: "default-route-distance",
        visible: DEFAULT_ROUTE,
        enabled: DEFAULT_ROUTE,
    },
    FieldRule {
        resource_id: "lte-apn",
        field_key: "passthrough-mac",
        visible: PASSTHROUGH,
        enabled: PASSTHROUGH,
    },
    FieldRule {
        resource_id: "lte-apn",
        field_key: "passthrough-subnet-size",
        visible: PASSTHROUGH,
        enabled: PASSTHROUGH,
    },
];

const LTE_GENERAL: &[FieldSpec] = &[
    ENABLED,
    COMMENT,
    NAME,
    DEFAULT_NAME,
    IFACE_TYPE,
    f!(
        "mtu",
        "MTU",
        FieldKind::Optional {
            kind: ScalarKind::Number {
                min: Some(64),
                max: Some(65_535)
            },
            unset: "auto",
            unset_label: "Auto"
        }
    ),
    f!("actual-mtu", "Actual MTU", FieldKind::Readonly),
    f!("l2mtu", "L2 MTU", FieldKind::Readonly),
    f!("vrf", "VRF", FieldKind::Readonly),
    f!("network-mode", "Network Mode", FieldKind::Repeat),
    f!("band", "LTE Bands", FieldKind::Repeat),
    f!("nr-band", "NR Bands", FieldKind::Repeat),
    f!("pin", "PIN", FieldKind::Secret),
    f!("operator", "Operator", OPTIONAL_TEXT),
    f!("modem-init", "Modem Init", OPTIONAL_TEXT),
    f!("apn-profiles", "APN Profile", LOOKUP_LTE_APN),
    f!("allow-roaming", "Allow Roaming", FieldKind::Toggle),
    f!("manufacturer", "Manufacturer", FieldKind::Readonly),
    f!("model", "Model", FieldKind::Readonly),
    f!("revision", "Revision", FieldKind::Readonly),
    f!("serial-number", "Serial Number", FieldKind::Readonly),
];

const LTE_STATUS: &[FieldSpec] = &[
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
    f!(
        "registration-status",
        "Registration Status",
        FieldKind::Readonly
    ),
    f!("pin-status", "PIN Status", FieldKind::Readonly),
    f!("functionality", "Functionality", FieldKind::Readonly),
];

const LTE_CELLULAR: &[FieldSpec] = &[
    f!("current-operator", "Current Operator", FieldKind::Readonly),
    f!("lac", "LAC", FieldKind::Readonly),
    f!("current-cellid", "Current Cell ID", FieldKind::Readonly),
    f!("enb-id", "eNB ID", FieldKind::Readonly),
    f!("sector-id", "Sector ID", FieldKind::Readonly),
    f!("phy-cellid", "PHY Cell ID", FieldKind::Readonly),
    f!("roaming", "Roaming", FieldKind::Readonly),
    f!(
        "access-technology",
        "Access Technology",
        FieldKind::Readonly
    ),
    f!("data-class", "Data Class", FieldKind::Readonly),
    f!("imei", "IMEI", FieldKind::Readonly),
    f!("imsi", "IMSI", FieldKind::Readonly),
    f!("iccid", "ICCID", FieldKind::Readonly),
    f!("earfcn", "EARFCN", FieldKind::Readonly),
    f!("primary-band", "Primary Band", FieldKind::Readonly),
    f!("session-uptime", "Session Uptime", FieldKind::Readonly),
    f!("rssi", "RSSI", FieldKind::Readonly),
    f!("rsrp", "RSRP", FieldKind::Readonly),
    f!("sinr", "SINR", FieldKind::Readonly),
    f!("rsrq", "RSRQ", FieldKind::Readonly),
];

const LTE_CAPABILITIES: &[FieldSpec] = &[
    f!(
        "modem-bus-location",
        "Modem Bus Location",
        FieldKind::Readonly
    ),
    f!(
        "apn-address-family",
        "APN Address Family",
        FieldKind::Readonly
    ),
    f!("rat-modes", "RAT Modes", FieldKind::Readonly),
    f!("lte-bands", "LTE Bands", FieldKind::Readonly),
    f!("nr-bands", "NR Bands", FieldKind::Readonly),
    f!("max-apn-count", "Max APN Count", FieldKind::Readonly),
    f!("passthrough", "Passthrough", FieldKind::Readonly),
    f!("firmware-update", "Firmware Update", FieldKind::Readonly),
    f!("at-chat", "AT Chat", FieldKind::Readonly),
    f!("cell-scan", "Cell Scan", FieldKind::Readonly),
    f!("esim-detected", "eSIM Detected", FieldKind::Readonly),
];

const TRAFFIC_FIELDS: &[FieldSpec] = &[
    f!("tx-byte", "Tx Bytes", FieldKind::Readonly),
    f!("rx-byte", "Rx Bytes", FieldKind::Readonly),
    f!("tx-packet", "Tx Packets", FieldKind::Readonly),
    f!("rx-packet", "Rx Packets", FieldKind::Readonly),
    f!("tx-drop", "Tx Drops", FieldKind::Readonly),
    f!("rx-drop", "Rx Drops", FieldKind::Readonly),
    f!("tx-error", "Tx Errors", FieldKind::Readonly),
    f!("rx-error", "Rx Errors", FieldKind::Readonly),
];

pub static LTE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["default-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: LTE_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: LTE_STATUS,
        },
        FormSection {
            id: "cellular",
            label: "Cellular",
            read_only: true,
            fields: LTE_CELLULAR,
        },
        FormSection {
            id: "capabilities",
            label: "Capabilities",
            read_only: true,
            fields: LTE_CAPABILITIES,
        },
        FormSection {
            id: "traffic",
            label: "Traffic",
            read_only: true,
            fields: TRAFFIC_FIELDS,
        },
    ],
    create_sections: &[],
};

const LTE_APN_FIELDS: &[FieldSpec] = &[
    NAME,
    f!("apn", "APN", FieldKind::Text),
    f!(
        "ip-type",
        "IP Type",
        FieldKind::LabeledEnum {
            choices: IP_TYPE_CHOICES
        }
    ),
    f!("use-peer-dns", "Use Peer DNS", FieldKind::Toggle),
    f!("use-network-apn", "Use Network APN", FieldKind::Toggle),
    f!("add-default-route", "Add Default Route", FieldKind::Toggle),
    f!(
        "default-route-distance",
        "Default Route Distance",
        FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(255)
        }
    ),
    f!("ipv6-interface", "IPv6 Interface", LOOKUP_IFACE),
    f!(
        "authentication",
        "Authentication",
        FieldKind::LabeledEnum {
            choices: AUTH_CHOICES
        }
    ),
    f!("user", "User", FieldKind::Text),
    f!("password", "Password", FieldKind::Secret),
    f!(
        "passthrough-interface",
        "Passthrough Interface",
        LOOKUP_IFACE
    ),
    f!(
        "passthrough-mac",
        "Passthr. MAC Address",
        FieldKind::Optional {
            kind: ScalarKind::Mac,
            unset: "",
            unset_label: "auto"
        }
    ),
    f!(
        "passthrough-subnet-size",
        "Passthr. Subnet Size",
        FieldKind::Optional {
            kind: ScalarKind::Number {
                min: Some(16),
                max: Some(32)
            },
            unset: "auto",
            unset_label: "Auto"
        }
    ),
    COMMENT,
];

pub static LTE_APN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["apn"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: LTE_APN_FIELDS,
    }],
    create_sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: LTE_APN_FIELDS,
    }],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    #[test]
    fn lte_uses_exact_webfig_sections() {
        assert_eq!(
            LTE_FORM
                .sections
                .iter()
                .map(|section| section.id)
                .collect::<Vec<_>>(),
            ["general", "status", "cellular", "capabilities", "traffic"]
        );
        assert!(LTE_FORM.create_sections.is_empty());
        assert_eq!(LTE_FORM.field("apn-profiles").unwrap().label, "APN Profile");
        assert_eq!(LTE_FORM.field("pin").unwrap().kind, FieldKind::Secret);
        assert!(LTE_FORM.field("sms-protocol").is_none());
    }

    #[test]
    fn lte_apn_uses_passthrough_subnet_size_and_labeled_ip_type() {
        assert_eq!(
            keys(&LTE_APN_FORM)[..6],
            [
                "name",
                "apn",
                "ip-type",
                "use-peer-dns",
                "use-network-apn",
                "add-default-route"
            ]
        );
        assert!(LTE_APN_FORM.field("passthrough-subnet-selection").is_none());
        assert!(LTE_APN_FORM.field("passthrough-subnet-size").is_some());
        assert_eq!(
            LTE_APN_FORM
                .field("ip-type")
                .unwrap()
                .kind
                .display_value("auto"),
            "Auto"
        );
        assert_eq!(
            LTE_APN_FORM.field("password").unwrap().kind,
            FieldKind::Secret
        );
    }
}
