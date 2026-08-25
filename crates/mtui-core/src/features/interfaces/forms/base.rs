//! Feature-owned 7.21.5 form schemas for Interface, Ethernet, and VLAN.

#![allow(clippy::wildcard_imports)]

use super::*;
use crate::forms::{BoxedFieldPredicate, EnumChoice, FieldPredicate, FieldRule, ScalarKind};

const OPTIONAL_ARP_TIMEOUT: FieldSpec = f!(
    "arp-timeout",
    "ARP Timeout",
    FieldKind::Optional {
        kind: ScalarKind::Time,
        unset: "auto",
        unset_label: "Auto"
    }
);
const ACTUAL_MTU: FieldSpec = f!("actual-mtu", "Actual MTU", FieldKind::Readonly);
const READONLY_L2_MTU: FieldSpec = f!("l2mtu", "L2 MTU", FieldKind::Readonly);
const READONLY_VRF: FieldSpec = f!("vrf", "VRF", FieldKind::Readonly);
const READONLY_MAC: FieldSpec = f!("mac-address", "MAC Address", FieldKind::Readonly);
const MTU_64_65535: FieldSpec = f!(
    "mtu",
    "MTU",
    FieldKind::ConstrainedNumber {
        min: Some(64),
        max: Some(65_535)
    }
);

const LOOP_PROTECT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Default",
        value: "default",
    },
    EnumChoice {
        label: "On",
        value: "on",
    },
    EnumChoice {
        label: "Off",
        value: "off",
    },
];

const LOOP_PROTECT_SECTION: FormSection = FormSection {
    id: "loop-protect",
    label: "Loop Protect",
    read_only: false,
    fields: &[
        f!(
            "loop-protect",
            "Loop Protect",
            FieldKind::LabeledEnum {
                choices: LOOP_PROTECT_CHOICES
            }
        ),
        f!(
            "loop-protect-send-interval",
            "Send Interval",
            FieldKind::Time
        ),
        f!("loop-protect-disable-time", "Disable Time", FieldKind::Time),
        f!("loop-protect-status", "Status", FieldKind::Readonly),
    ],
};

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

const STATUS_SECTION: FormSection = FormSection {
    id: "status",
    label: "Status",
    read_only: true,
    fields: LINK_STATUS_FIELDS,
};
const TRAFFIC_SECTION: FormSection = FormSection {
    id: "traffic",
    label: "Traffic",
    read_only: true,
    fields: TRAFFIC_FIELDS,
};

const FLOW_CONTROL_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "off",
        value: "off",
    },
    EnumChoice {
        label: "on",
        value: "on",
    },
    EnumChoice {
        label: "auto",
        value: "auto",
    },
];

const COMBO_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "copper",
        value: "copper",
    },
    EnumChoice {
        label: "sfp",
        value: "sfp",
    },
];

const POE_OUT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "off",
        value: "off",
    },
    EnumChoice {
        label: "auto on",
        value: "auto-on",
    },
    EnumChoice {
        label: "forced on",
        value: "forced-on",
    },
];

const POE_VOLTAGE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "low",
        value: "low",
    },
    EnumChoice {
        label: "high",
        value: "high",
    },
];

const FEC_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "off",
        value: "off",
    },
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "fec74",
        value: "fec74",
    },
    EnumChoice {
        label: "fec91",
        value: "fec91",
    },
];

const RATE_SELECT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "low",
        value: "low",
    },
    EnumChoice {
        label: "high",
        value: "high",
    },
];

const SPEED_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "10M baseT half",
        value: "10M-baseT-half",
    },
    EnumChoice {
        label: "10M baseT full",
        value: "10M-baseT-full",
    },
    EnumChoice {
        label: "100M baseT half",
        value: "100M-baseT-half",
    },
    EnumChoice {
        label: "100M baseT full",
        value: "100M-baseT-full",
    },
    EnumChoice {
        label: "1G baseT half",
        value: "1G-baseT-half",
    },
    EnumChoice {
        label: "1G baseT full",
        value: "1G-baseT-full",
    },
    EnumChoice {
        label: "1G baseX",
        value: "1G-baseX",
    },
    EnumChoice {
        label: "2.5G baseT",
        value: "2.5G-baseT",
    },
    EnumChoice {
        label: "2.5G baseX",
        value: "2.5G-baseX",
    },
    EnumChoice {
        label: "5G baseT",
        value: "5G-baseT",
    },
    EnumChoice {
        label: "10G baseT",
        value: "10G-baseT",
    },
    EnumChoice {
        label: "10G baseSR-LR",
        value: "10G-baseSR-LR",
    },
    EnumChoice {
        label: "10G baseCR",
        value: "10G-baseCR",
    },
    EnumChoice {
        label: "25G baseSR-LR",
        value: "25G-baseSR-LR",
    },
    EnumChoice {
        label: "25G baseCR",
        value: "25G-baseCR",
    },
    EnumChoice {
        label: "40G baseSR4-LR4",
        value: "40G-baseSR4-LR4",
    },
    EnumChoice {
        label: "40G baseCR4",
        value: "40G-baseCR4",
    },
    EnumChoice {
        label: "50G baseSR2-LR2",
        value: "50G-baseSR2-LR2",
    },
    EnumChoice {
        label: "50G baseCR2",
        value: "50G-baseCR2",
    },
    EnumChoice {
        label: "100G baseSR4-LR4",
        value: "100G-baseSR4-LR4",
    },
    EnumChoice {
        label: "100G baseCR4",
        value: "100G-baseCR4",
    },
];

const AUTO_NEG: FieldPredicate = FieldPredicate::Truthy("auto-negotiation");
const MANUAL_SPEED: FieldPredicate = FieldPredicate::Not(BoxedFieldPredicate(&AUTO_NEG));
const PING_ENABLED: FieldPredicate = FieldPredicate::Truthy("power-cycle-ping-enabled");
/// Ethernet PoE-out capability bit (`caps & 524288`) plus `poe-*` print attrs.
const ETHERNET_POE_CAP: u64 = 524_288;
const HAS_POE: FieldPredicate = FieldPredicate::Any(&[
    FieldPredicate::HasBits {
        key: "caps",
        mask: ETHERNET_POE_CAP,
    },
    FieldPredicate::HasKeyPrefix("poe-"),
]);
/// SFP tab follows `/interface/ethernet` print: any `sfp-*` attribute.
/// Interface name (`sfp1`) and a boolean `sfp` flag are not gates.
const HAS_SFP: FieldPredicate = FieldPredicate::HasKeyPrefix("sfp-");
/// VLAN L3 HW controls follow `/interface/vlan` print: any `l3-*` attribute.
const HAS_L3: FieldPredicate = FieldPredicate::HasKeyPrefix("l3-");
const POE_PING: FieldPredicate = FieldPredicate::All(&[HAS_POE, PING_ENABLED]);

pub(crate) const FIELD_RULES: &[FieldRule] = &[
    FieldRule {
        resource_id: "ethernet",
        field_key: "advertise",
        visible: AUTO_NEG,
        enabled: AUTO_NEG,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "speed",
        visible: MANUAL_SPEED,
        enabled: MANUAL_SPEED,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "power-cycle-ping-address",
        visible: POE_PING,
        enabled: POE_PING,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "power-cycle-ping-timeout",
        visible: POE_PING,
        enabled: POE_PING,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-out",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-voltage",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-priority",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "power-cycle-ping-enabled",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "power-cycle-interval",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-out-status",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-out-current",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-out-voltage",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "poe-out-power",
        visible: HAS_POE,
        enabled: HAS_POE,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-rate-select",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-ignore-rx-los",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-shutdown-temperature",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "fec-mode",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-module-present",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-rx-loss",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-tx-fault",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-vendor-name",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-vendor-part-number",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-vendor-serial",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-temperature",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-supply-voltage",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-tx-power",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "ethernet",
        field_key: "sfp-rx-power",
        visible: HAS_SFP,
        enabled: HAS_SFP,
    },
    FieldRule {
        resource_id: "vlan",
        field_key: "l3-hw-offloading",
        visible: HAS_L3,
        enabled: HAS_L3,
    },
    FieldRule {
        resource_id: "vlan",
        field_key: "hw-offloaded",
        visible: HAS_L3,
        enabled: HAS_L3,
    },
];

const INTERFACE_GENERAL: &[FieldSpec] = &[
    ENABLED,
    COMMENT,
    NAME,
    DEFAULT_NAME,
    IFACE_TYPE,
    ACTUAL_MTU,
    READONLY_L2_MTU,
    READONLY_VRF,
];

pub static INTERFACES_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: INTERFACE_GENERAL,
        },
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[],
};

const ETHERNET_GENERAL: &[FieldSpec] = &[
    ENABLED,
    COMMENT,
    NAME,
    DEFAULT_NAME,
    IFACE_TYPE,
    MTU_64_65535,
    ACTUAL_MTU,
    L2MTU,
    f!("max-l2mtu", "Max L2 MTU", FieldKind::Readonly),
    READONLY_VRF,
    f!("mac-address", "MAC Address", FieldKind::Mac),
    ARP,
    OPTIONAL_ARP_TIMEOUT,
    f!(
        "combo-mode",
        "Combo Mode",
        FieldKind::LabeledEnum {
            choices: COMBO_MODE_CHOICES
        }
    ),
    f!(
        "passthrough-interface",
        "Passthrough Interface",
        LOOKUP_IFACE
    ),
];

const POE_FIELDS: &[FieldSpec] = &[
    f!(
        "poe-out",
        "PoE Out",
        FieldKind::LabeledEnum {
            choices: POE_OUT_CHOICES
        }
    ),
    f!(
        "poe-voltage",
        "PoE Voltage",
        FieldKind::LabeledEnum {
            choices: POE_VOLTAGE_CHOICES
        }
    ),
    f!(
        "poe-priority",
        "PoE Priority",
        FieldKind::ConstrainedNumber {
            min: Some(0),
            max: Some(100)
        }
    ),
    f!(
        "power-cycle-ping-enabled",
        "Power Cycle Ping Enabled",
        FieldKind::Toggle
    ),
    f!(
        "power-cycle-ping-address",
        "Power Cycle Ping Address",
        FieldKind::Raw
    ),
    f!(
        "power-cycle-ping-timeout",
        "Power Cycle Ping Timeout",
        FieldKind::Time
    ),
    f!(
        "power-cycle-interval",
        "Power Cycle Interval",
        FieldKind::Optional {
            kind: ScalarKind::Time,
            unset: "",
            unset_label: "none"
        }
    ),
    f!("poe-out-status", "PoE Out Status", FieldKind::Readonly),
    f!("poe-out-current", "PoE Out Current", FieldKind::Readonly),
    f!("poe-out-voltage", "PoE Out Voltage", FieldKind::Readonly),
    f!("poe-out-power", "PoE Out Power", FieldKind::Readonly),
];

const SFP_FIELDS: &[FieldSpec] = &[
    f!(
        "sfp-rate-select",
        "Rate Select",
        FieldKind::LabeledEnum {
            choices: RATE_SELECT_CHOICES
        }
    ),
    f!("sfp-ignore-rx-los", "Ignore Rx LOS", FieldKind::Toggle),
    f!(
        "sfp-shutdown-temperature",
        "SFP Shutdown Temperature",
        FieldKind::Number
    ),
    f!(
        "fec-mode",
        "FEC Mode",
        FieldKind::LabeledEnum {
            choices: FEC_MODE_CHOICES
        }
    ),
    f!("sfp-module-present", "Module Present", FieldKind::Readonly),
    f!("sfp-rx-loss", "Rx Loss", FieldKind::Readonly),
    f!("sfp-tx-fault", "Tx Fault", FieldKind::Readonly),
    f!("sfp-vendor-name", "Vendor Name", FieldKind::Readonly),
    f!(
        "sfp-vendor-part-number",
        "Vendor Part Number",
        FieldKind::Readonly
    ),
    f!("sfp-vendor-serial", "Vendor Serial", FieldKind::Readonly),
    f!("sfp-temperature", "Temperature", FieldKind::Readonly),
    f!("sfp-supply-voltage", "Supply Voltage", FieldKind::Readonly),
    f!("sfp-tx-power", "Tx Power", FieldKind::Readonly),
    f!("sfp-rx-power", "Rx Power", FieldKind::Readonly),
];

const ETHERNET_PHY: &[FieldSpec] = &[
    f!(
        "tx-flow-control",
        "Tx Flow Control",
        FieldKind::LabeledEnum {
            choices: FLOW_CONTROL_CHOICES
        }
    ),
    f!(
        "rx-flow-control",
        "Rx Flow Control",
        FieldKind::LabeledEnum {
            choices: FLOW_CONTROL_CHOICES
        }
    ),
    f!("auto-negotiation", "Auto Negotiation", FieldKind::Toggle),
    f!("advertise", "Advertise", FieldKind::Repeat),
    f!(
        "speed",
        "Speed",
        FieldKind::LabeledEnum {
            choices: SPEED_CHOICES
        }
    ),
];

const ETHERNET_STATUS: &[FieldSpec] = &[
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
    f!("orig-mac-address", "Orig. MAC Address", FieldKind::Readonly),
    f!("switch", "Switch", FieldKind::Readonly),
    f!("rate", "Rate", FieldKind::Readonly),
    f!("full-duplex", "Full Duplex", FieldKind::Readonly),
    f!("fec", "FEC", FieldKind::Readonly),
    f!("advertising", "Advertising", FieldKind::Readonly),
    f!(
        "link-partner-advertising",
        "Link Partner Advertising",
        FieldKind::Readonly
    ),
    RUNNING,
    SLAVE,
];

pub static ETHERNET_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["default-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: ETHERNET_GENERAL,
        },
        FormSection {
            id: "poe",
            label: "PoE",
            read_only: false,
            fields: POE_FIELDS,
        },
        FormSection {
            id: "sfp",
            label: "SFP",
            read_only: false,
            fields: SFP_FIELDS,
        },
        FormSection {
            id: "ethernet",
            label: "Ethernet",
            read_only: false,
            fields: ETHERNET_PHY,
        },
        LOOP_PROTECT_SECTION,
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: ETHERNET_STATUS,
        },
        TRAFFIC_SECTION,
    ],
    create_sections: &[],
};

const VLAN_GENERAL: &[FieldSpec] = &[
    ENABLED,
    COMMENT,
    NAME,
    DEFAULT_NAME,
    IFACE_TYPE,
    MTU_64_65535,
    ACTUAL_MTU,
    READONLY_L2_MTU,
    READONLY_VRF,
    READONLY_MAC,
    ARP,
    OPTIONAL_ARP_TIMEOUT,
    f!(
        "vlan-id",
        "VLAN ID",
        FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(4094)
        }
    ),
    INTERFACE,
    f!("use-service-tag", "Use Service Tag", FieldKind::Toggle),
    f!("mvrp", "MVRP", FieldKind::Toggle),
    f!("l3-hw-offloading", "L3 Hw Offloading", FieldKind::Toggle),
    f!("hw-offloaded", "Hw. Offloaded", FieldKind::Readonly),
];

pub static VLAN_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["vlan-id", "interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: VLAN_GENERAL,
        },
        LOOP_PROTECT_SECTION,
        STATUS_SECTION,
        TRAFFIC_SECTION,
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: VLAN_GENERAL,
    }],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn section_ids(schema: &FormSchema) -> Vec<&'static str> {
        schema.sections.iter().map(|section| section.id).collect()
    }

    #[test]
    fn aggregate_interface_is_nonaddable_and_has_no_writable_mac() {
        assert_eq!(
            section_ids(&INTERFACES_FORM),
            ["general", "status", "traffic"]
        );
        assert!(INTERFACES_FORM.create_sections.is_empty());
        assert!(INTERFACES_FORM.field("mac-address").is_none());
        assert!(INTERFACES_FORM.field("mtu").is_none());
        assert_eq!(
            INTERFACES_FORM.field("disabled").unwrap().kind,
            FieldKind::InvertedToggle
        );
        assert_eq!(
            INTERFACES_FORM.field("l2mtu").unwrap().kind,
            FieldKind::Readonly
        );
    }

    #[test]
    fn ethernet_matches_webfig_sections_and_status_only_duplex() {
        assert_eq!(
            section_ids(&ETHERNET_FORM),
            [
                "general",
                "poe",
                "sfp",
                "ethernet",
                "loop-protect",
                "status",
                "traffic"
            ]
        );
        assert!(ETHERNET_FORM.create_sections.is_empty());
        assert_eq!(
            ETHERNET_FORM.field("full-duplex").unwrap().kind,
            FieldKind::Readonly
        );
        assert_eq!(
            ETHERNET_FORM.field("advertise").unwrap().kind,
            FieldKind::Repeat
        );
        assert!(matches!(
            ETHERNET_FORM.field("speed").unwrap().kind,
            FieldKind::LabeledEnum { .. }
        ));
        assert_eq!(
            ETHERNET_FORM.field("mac-address").unwrap().kind,
            FieldKind::Mac
        );
    }

    #[test]
    fn vlan_matches_live_webfig_order() {
        assert_eq!(
            section_ids(&VLAN_FORM),
            ["general", "loop-protect", "status", "traffic"]
        );
        assert_eq!(VLAN_FORM.field("l2mtu").unwrap().kind, FieldKind::Readonly);
        assert_eq!(
            VLAN_FORM.field("mac-address").unwrap().kind,
            FieldKind::Readonly
        );
        assert!(matches!(
            VLAN_FORM.field("arp-timeout").unwrap().kind,
            FieldKind::Optional { .. }
        ));
        assert!(VLAN_FORM.field("mvrp").is_some());
        assert!(VLAN_FORM.field("l3-hw-offloading").is_some());
        assert!(VLAN_FORM.field("use-service-tag").is_some());
    }

    #[test]
    fn ethernet_poe_and_sfp_follow_port_capabilities() {
        use crate::forms::field_visible;
        use std::collections::HashMap;

        let copper = HashMap::from([("name".to_string(), "ether1".to_string())]);
        for field in POE_FIELDS.iter().chain(SFP_FIELDS) {
            assert!(
                !field_visible("ethernet", field.key, &copper),
                "{} should stay hidden on a copper port without capability attrs",
                field.key
            );
        }

        let poe = HashMap::from([("caps".to_string(), "524288".to_string())]);
        assert!(field_visible("ethernet", "poe-out", &poe));
        assert!(!field_visible("ethernet", "sfp-rate-select", &poe));
        assert!(!field_visible("ethernet", "power-cycle-ping-address", &poe));

        let poe_hex = HashMap::from([("caps".to_string(), "0x80000".to_string())]);
        assert!(field_visible("ethernet", "poe-out", &poe_hex));

        let poe_attr = HashMap::from([("poe-out".to_string(), "auto-on".to_string())]);
        assert!(field_visible("ethernet", "poe-priority", &poe_attr));
        assert!(!field_visible("ethernet", "sfp-rate-select", &poe_attr));

        let sfp_flag = HashMap::from([("sfp".to_string(), "true".to_string())]);
        assert!(
            !field_visible("ethernet", "sfp-rate-select", &sfp_flag),
            "a boolean sfp flag is not the print attribute"
        );

        let sfp_temp = HashMap::from([("sfp-shutdown-temperature".to_string(), "95C".to_string())]);
        assert!(field_visible("ethernet", "sfp-rate-select", &sfp_temp));
        assert!(field_visible("ethernet", "sfp-ignore-rx-los", &sfp_temp));
        assert!(!field_visible("ethernet", "poe-out", &sfp_temp));

        let sfp_name = HashMap::from([
            ("name".to_string(), "sfp1".to_string()),
            ("default-name".to_string(), "sfp1".to_string()),
        ]);
        assert!(
            !field_visible("ethernet", "sfp-ignore-rx-los", &sfp_name),
            "SFP is detected from sfp-* print attrs, not the name"
        );

        let sfp_attr = HashMap::from([("sfp-module-present".to_string(), "false".to_string())]);
        assert!(field_visible("ethernet", "sfp-ignore-rx-los", &sfp_attr));
    }

    #[test]
    fn vlan_l3_hw_follows_vlan_print_attributes() {
        use crate::forms::field_visible;
        use std::collections::HashMap;

        let create = HashMap::from([("name".to_string(), "vlan10".to_string())]);
        assert!(!field_visible("vlan", "l3-hw-offloading", &create));
        assert!(!field_visible("vlan", "hw-offloaded", &create));

        let l3 = HashMap::from([("l3-hw-offloading".to_string(), "yes".to_string())]);
        assert!(field_visible("vlan", "l3-hw-offloading", &l3));
        assert!(field_visible("vlan", "hw-offloaded", &l3));
        assert!(field_visible("vlan", "name", &l3));
    }
}
