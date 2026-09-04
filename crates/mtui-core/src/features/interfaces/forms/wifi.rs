//! Feature-owned 7.21.5 form schemas for wifiwave2 and legacy wireless.

#![allow(clippy::wildcard_imports)]

use super::*;
use crate::form_fields::{
    KIND_NV2_TDMA_PERIOD, KIND_TLS_MODE, KIND_WDS_MODE, KIND_WIFI_COUNTRY,
    KIND_WIFI_COUNTRY_OPTIONAL, KIND_WIRELESS_BAND, KIND_WIRELESS_CHANNEL_WIDTH,
    KIND_WIRELESS_VLAN_MODE, LOOKUP_WIFI_AAA, LOOKUP_WIFI_INTERWORKING, LOOKUP_WIFI_STEERING,
};
use crate::forms::{EnumChoice, FieldRule, ScalarKind};

const LOOKUP_WIFI: FieldKind = FieldKind::Lookup {
    resource_id: "wifi",
    value_key: "name",
    multiple: false,
};
const LOOKUP_WIFI_CONFIG: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-configuration",
    value_key: "name",
    multiple: false,
};
const LOOKUP_WIFI_CONFIGS: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-configuration",
    value_key: "name",
    multiple: true,
};
const LOOKUP_WIFI_CHANNEL: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-channel",
    value_key: "name",
    multiple: false,
};
const LOOKUP_WIFI_SECURITY: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-security",
    value_key: "name",
    multiple: false,
};
const LOOKUP_WIFI_DATAPATH: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-datapath",
    value_key: "name",
    multiple: false,
};
const LOOKUP_BRIDGE: FieldKind = FieldKind::Lookup {
    resource_id: "bridges",
    value_key: "name",
    multiple: false,
};
const LOOKUP_CERT: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};
const LOOKUP_WIRELESS_SECURITY: FieldKind = FieldKind::Lookup {
    resource_id: "wireless-security-profiles",
    value_key: "name",
    multiple: false,
};

pub(crate) const FIELD_RULES: &[FieldRule] = &[];

const OPTIONAL_MTU: FieldSpec = f!(
    "mtu",
    "MTU",
    FieldKind::Optional {
        kind: ScalarKind::Number {
            min: Some(32),
            max: Some(65_535)
        },
        unset: "auto",
        unset_label: "Auto"
    }
);
const OPTIONAL_TEXT: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Text,
    unset: "",
    unset_label: "none",
};
const OPTIONAL_NUM: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Number {
        min: None,
        max: None,
    },
    unset: "",
    unset_label: "none",
};

const MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "ap",
        value: "ap",
    },
    EnumChoice {
        label: "station",
        value: "station",
    },
    EnumChoice {
        label: "station bridge",
        value: "station-bridge",
    },
    EnumChoice {
        label: "station pseudobridge",
        value: "station-pseudobridge",
    },
];

const BAND_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "5GHz A",
        value: "5ghz-a",
    },
    EnumChoice {
        label: "5GHz A/N",
        value: "5ghz-a/n",
    },
    EnumChoice {
        label: "5GHz AC",
        value: "5ghz-ac",
    },
    EnumChoice {
        label: "5GHz AX",
        value: "5ghz-ax",
    },
    EnumChoice {
        label: "5GHz BE",
        value: "5ghz-be",
    },
    EnumChoice {
        label: "2GHz G",
        value: "2ghz-g",
    },
    EnumChoice {
        label: "2GHz N",
        value: "2ghz-n",
    },
    EnumChoice {
        label: "2GHz AX",
        value: "2ghz-ax",
    },
    EnumChoice {
        label: "2GHz BE",
        value: "2ghz-be",
    },
];

const WIDTH_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "20MHz",
        value: "20mhz",
    },
    EnumChoice {
        label: "20/40MHz",
        value: "20/40mhz",
    },
    EnumChoice {
        label: "20/40MHz Ce",
        value: "20/40mhz-ce",
    },
    EnumChoice {
        label: "20/40MHz eC",
        value: "20/40mhz-ec",
    },
    EnumChoice {
        label: "20/40/80MHz",
        value: "20/40/80mhz",
    },
    EnumChoice {
        label: "20/40/80+80MHz",
        value: "20/40/80+80mhz",
    },
    EnumChoice {
        label: "20/40/80/160MHz",
        value: "20/40/80/160mhz",
    },
];

const SKIP_DFS_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "disabled",
        value: "disabled",
    },
    EnumChoice {
        label: "all",
        value: "all",
    },
    EnumChoice {
        label: "10min CAC",
        value: "10min-cac",
    },
];

const MANAGER_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "",
        value: "",
    },
    EnumChoice {
        label: "local",
        value: "local",
    },
    EnumChoice {
        label: "capsman",
        value: "capsman",
    },
    EnumChoice {
        label: "capsman or local",
        value: "capsman-or-local",
    },
];

const MULTICAST_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "disabled",
        value: "disabled",
    },
    EnumChoice {
        label: "enabled",
        value: "enabled",
    },
];

const QOS_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "priority",
        value: "priority",
    },
    EnumChoice {
        label: "dscp high 3 bits",
        value: "dscp-high-3-bits",
    },
];

const HW_PROTECT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "rts-cts",
        value: "rts-cts",
    },
    EnumChoice {
        label: "cts-to-self",
        value: "cts-to-self",
    },
];

const MGMT_PROTECT_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "disabled",
        value: "disabled",
    },
    EnumChoice {
        label: "allowed",
        value: "allowed",
    },
    EnumChoice {
        label: "required",
        value: "required",
    },
];

const WPS_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "disable",
        value: "disable",
    },
    EnumChoice {
        label: "push button",
        value: "push-button",
    },
];

const SAE_PWE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "hunting and pecking",
        value: "hunting-and-pecking",
    },
    EnumChoice {
        label: "hash to element",
        value: "hash-to-element",
    },
    EnumChoice {
        label: "both",
        value: "both",
    },
];

const TRAFFIC_PROCESSING_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "on-cap",
        value: "on-cap",
    },
    EnumChoice {
        label: "on-capsman",
        value: "on-capsman",
    },
    EnumChoice {
        label: "on-capsman-secure",
        value: "on-capsman-secure",
    },
];

const PROVISION_ACTION_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "create enabled",
        value: "create-enabled",
    },
    EnumChoice {
        label: "create disabled",
        value: "create-disabled",
    },
    EnumChoice {
        label: "create dynamic enabled",
        value: "create-dynamic-enabled",
    },
];

const UPGRADE_POLICY_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "suggest same version",
        value: "suggest-same-version",
    },
    EnumChoice {
        label: "require same version",
        value: "require-same-version",
    },
];

const INSTALLATION_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "indoor",
        value: "indoor",
    },
    EnumChoice {
        label: "outdoor",
        value: "outdoor",
    },
];

const LINK_STATUS: &[FieldSpec] = &[
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
    f!("state", "State", FieldKind::Readonly),
    f!("current-channel", "Current Channel", FieldKind::Readonly),
    f!("tx-power", "Tx Power", FieldKind::Readonly),
];

const TRAFFIC_FIELDS: &[FieldSpec] = &[
    f!("tx-byte", "Tx Bytes", FieldKind::Readonly),
    f!("rx-byte", "Rx Bytes", FieldKind::Readonly),
    f!("tx-packet", "Tx Packets", FieldKind::Readonly),
    f!("rx-packet", "Rx Packets", FieldKind::Readonly),
    f!("fp-tx-byte", "FP Tx Bytes", FieldKind::Readonly),
    f!("fp-rx-byte", "FP Rx Bytes", FieldKind::Readonly),
    f!("tx-drop", "Tx Drops", FieldKind::Readonly),
    f!("rx-drop", "Rx Drops", FieldKind::Readonly),
    f!("tx-error", "Tx Errors", FieldKind::Readonly),
    f!("rx-error", "Rx Errors", FieldKind::Readonly),
];

const WIFI_GENERAL: &[FieldSpec] = &[
    ENABLED,
    COMMENT,
    NAME,
    DEFAULT_NAME,
    IFACE_TYPE,
    OPTIONAL_MTU,
    f!("actual-mtu", "Actual MTU", FieldKind::Readonly),
    f!(
        "l2mtu",
        "L2MTU",
        FieldKind::Optional {
            kind: ScalarKind::Number {
                min: Some(32),
                max: Some(65_535)
            },
            unset: "auto",
            unset_label: "Auto"
        }
    ),
    f!("vrf", "VRF", FieldKind::Readonly),
    f!("mac-address", "MAC Address", FieldKind::Mac),
    f!(
        "arp",
        "ARP",
        FieldKind::Optional {
            kind: ScalarKind::Enum {
                choices: ARP_CHOICES
            },
            unset: "",
            unset_label: "none"
        }
    ),
    f!(
        "arp-timeout",
        "Arp Timeout",
        FieldKind::Optional {
            kind: ScalarKind::Time,
            unset: "auto",
            unset_label: "Auto"
        }
    ),
    f!("cap", "CAP", FieldKind::Readonly),
    f!("master-interface", "Master", LOOKUP_WIFI),
    f!(
        "configuration.mode",
        "Mode",
        FieldKind::LabeledEnum {
            choices: MODE_CHOICES
        }
    ),
];

const WIFI_CONFIGURATION_FIELDS: &[FieldSpec] = &[
    f!("configuration", "Configuration", LOOKUP_WIFI_CONFIG),
    f!("configuration.ssid", "SSID", OPTIONAL_TEXT),
    f!(
        "configuration.country",
        "Country",
        KIND_WIFI_COUNTRY_OPTIONAL
    ),
    f!("configuration.chains", "Chains", FieldKind::Repeat),
    f!("configuration.tx-chains", "Tx Chains", FieldKind::Repeat),
    f!("configuration.tx-power", "Max Tx Power", OPTIONAL_NUM),
    f!("configuration.antenna-gain", "Antenna Gain", OPTIONAL_NUM),
    f!("configuration.distance", "Distance", OPTIONAL_TEXT),
    f!(
        "configuration.installation",
        "Installation",
        FieldKind::LabeledEnum {
            choices: INSTALLATION_CHOICES
        }
    ),
    f!("configuration.hide-ssid", "Hide SSID", FieldKind::Toggle),
    f!(
        "configuration.manager",
        "Manager",
        FieldKind::LabeledEnum {
            choices: MANAGER_CHOICES
        }
    ),
    f!(
        "configuration.multicast-enhance",
        "Multicast Enhance",
        FieldKind::LabeledEnum {
            choices: MULTICAST_CHOICES
        }
    ),
    f!(
        "configuration.qos-classifier",
        "QoS Classifier",
        FieldKind::LabeledEnum {
            choices: QOS_CHOICES
        }
    ),
    f!(
        "configuration.station-roaming",
        "Station Roaming",
        FieldKind::Toggle
    ),
    f!(
        "configuration.hw-protection-mode",
        "Hw.Protection Mode",
        FieldKind::LabeledEnum {
            choices: HW_PROTECT_CHOICES
        }
    ),
];

const WIFI_CHANNEL_FIELDS: &[FieldSpec] = &[
    f!("channel", "Channel", LOOKUP_WIFI_CHANNEL),
    f!(
        "channel.band",
        "Band",
        FieldKind::LabeledEnum {
            choices: BAND_CHOICES
        }
    ),
    f!(
        "channel.width",
        "Channel Width",
        FieldKind::LabeledEnum {
            choices: WIDTH_CHOICES
        }
    ),
    f!("channel.frequency", "Frequency", FieldKind::Repeat),
    f!(
        "channel.secondary-frequency",
        "Secondary Frequency",
        FieldKind::Repeat
    ),
    f!(
        "channel.skip-dfs-channels",
        "Skip DFS Channels",
        FieldKind::LabeledEnum {
            choices: SKIP_DFS_CHOICES
        }
    ),
    f!(
        "channel.deprioritize-unii-3-4",
        "Deprioritize UNII-3-4",
        FieldKind::Toggle
    ),
];

const WIFI_SECURITY_FIELDS: &[FieldSpec] = &[
    f!("security", "Security", LOOKUP_WIFI_SECURITY),
    f!(
        "security.authentication-types",
        "Authentication Types",
        FieldKind::Repeat
    ),
    f!("security.encryption", "Encryption", FieldKind::Repeat),
    f!("security.passphrase", "Passphrase", FieldKind::Secret),
    f!("security.disable-pmkid", "Disable PMKID", FieldKind::Toggle),
    f!(
        "security.management-protection",
        "Management Protection",
        FieldKind::LabeledEnum {
            choices: MGMT_PROTECT_CHOICES
        }
    ),
    f!(
        "security.wps",
        "WPS",
        FieldKind::LabeledEnum {
            choices: WPS_CHOICES
        }
    ),
    f!(
        "security.sae-pwe",
        "SAE PWE",
        FieldKind::LabeledEnum {
            choices: SAE_PWE_CHOICES
        }
    ),
    f!(
        "security.owe-transition-interface",
        "OWE Transition Interface",
        LOOKUP_WIFI
    ),
];

const WIFI_EAP_FIELDS: &[FieldSpec] = &[
    f!("security.eap-methods", "Methods", FieldKind::Repeat),
    f!("security.tls-certificate", "TLS Certificate", LOOKUP_CERT),
    f!(
        "security.eap-anonymous-identity",
        "Anonymous Identity",
        FieldKind::Text
    ),
    f!("security.eap-password", "Password", FieldKind::Secret),
    f!("security.eap-accounting", "Accounting", FieldKind::Toggle),
];

const WIFI_FT_FIELDS: &[FieldSpec] = &[
    f!("security.ft", "FT Enabled", FieldKind::Toggle),
    f!("security.ft-over-ds", "FT Over DS", FieldKind::Toggle),
    f!(
        "security.ft-preserve-vlanid",
        "FT Preserve VLAN ID",
        FieldKind::Toggle
    ),
];

const WIFI_AAA_FIELDS: &[FieldSpec] = &[
    f!("aaa", "AAA", LOOKUP_WIFI_AAA),
    f!("aaa.nas-identifier", "NAS Identifier", OPTIONAL_TEXT),
];

const WIFI_DATAPATH_FIELDS: &[FieldSpec] = &[
    f!("datapath", "Datapath", LOOKUP_WIFI_DATAPATH),
    f!("datapath.bridge", "Bridge", LOOKUP_BRIDGE),
    f!(
        "datapath.client-isolation",
        "Client Isolation",
        FieldKind::Toggle
    ),
    f!(
        "datapath.traffic-processing",
        "Traffic Processing",
        FieldKind::LabeledEnum {
            choices: TRAFFIC_PROCESSING_CHOICES
        }
    ),
    f!("datapath.vlan-id", "VLAN ID", OPTIONAL_NUM),
    f!(
        "datapath.interface-list",
        "Interface List",
        LOOKUP_IFACE_LIST
    ),
];

const WIFI_INTERWORKING_FIELDS: &[FieldSpec] = &[
    f!("interworking", "Interworking", LOOKUP_WIFI_INTERWORKING),
    f!("interworking.internet", "Internet", FieldKind::Toggle),
    f!("interworking.hessid", "HESSID", FieldKind::Mac),
    f!("interworking.hotspot20", "Hotspot 2.0", FieldKind::Toggle),
];

const WIFI_STEERING_FIELDS: &[FieldSpec] = &[
    f!("steering", "Steering", LOOKUP_WIFI_STEERING),
    f!("steering.rrm", "RRM", FieldKind::Toggle),
    f!("steering.wnm", "WNM", FieldKind::Toggle),
];

const WIFI_WRITABLE: &[FormSection] = &[
    FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: WIFI_GENERAL,
    },
    FormSection {
        id: "configuration",
        label: "Configuration",
        read_only: false,
        fields: WIFI_CONFIGURATION_FIELDS,
    },
    FormSection {
        id: "channel",
        label: "Channel",
        read_only: false,
        fields: WIFI_CHANNEL_FIELDS,
    },
    FormSection {
        id: "security",
        label: "Security",
        read_only: false,
        fields: WIFI_SECURITY_FIELDS,
    },
    FormSection {
        id: "eap",
        label: "EAP",
        read_only: false,
        fields: WIFI_EAP_FIELDS,
    },
    FormSection {
        id: "ft",
        label: "FT",
        read_only: false,
        fields: WIFI_FT_FIELDS,
    },
    FormSection {
        id: "aaa",
        label: "AAA",
        read_only: false,
        fields: WIFI_AAA_FIELDS,
    },
    FormSection {
        id: "datapath",
        label: "Datapath",
        read_only: false,
        fields: WIFI_DATAPATH_FIELDS,
    },
    FormSection {
        id: "interworking",
        label: "Interworking",
        read_only: false,
        fields: WIFI_INTERWORKING_FIELDS,
    },
    FormSection {
        id: "steering",
        label: "Steering",
        read_only: false,
        fields: WIFI_STEERING_FIELDS,
    },
];

pub static WIFI_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["configuration.ssid", "master-interface"],
    sections: &[
        WIFI_WRITABLE[0],
        WIFI_WRITABLE[1],
        WIFI_WRITABLE[2],
        WIFI_WRITABLE[3],
        WIFI_WRITABLE[4],
        WIFI_WRITABLE[5],
        WIFI_WRITABLE[6],
        WIFI_WRITABLE[7],
        WIFI_WRITABLE[8],
        WIFI_WRITABLE[9],
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: LINK_STATUS,
        },
        FormSection {
            id: "traffic",
            label: "Traffic",
            read_only: true,
            fields: TRAFFIC_FIELDS,
        },
    ],
    create_sections: WIFI_WRITABLE,
};

const CHANNEL_PROFILE_FIELDS: &[FieldSpec] = &[
    NAME,
    f!(
        "band",
        "Band",
        FieldKind::LabeledEnum {
            choices: BAND_CHOICES
        }
    ),
    f!(
        "width",
        "Channel Width",
        FieldKind::LabeledEnum {
            choices: WIDTH_CHOICES
        }
    ),
    f!("frequency", "Frequency", FieldKind::Repeat),
    f!(
        "secondary-frequency",
        "Secondary Frequency",
        FieldKind::Repeat
    ),
    f!(
        "skip-dfs-channels",
        "Skip DFS Channels",
        FieldKind::LabeledEnum {
            choices: SKIP_DFS_CHOICES
        }
    ),
    f!(
        "deprioritize-unii-3-4",
        "Deprioritize UNII-3-4",
        FieldKind::Toggle
    ),
    ENABLED,
    COMMENT,
];

pub static WIFI_CHANNEL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["band"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: CHANNEL_PROFILE_FIELDS,
    }],
    create_sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: CHANNEL_PROFILE_FIELDS,
    }],
};

const SECURITY_PROFILE_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "security",
        label: "Security",
        read_only: false,
        fields: &[
            NAME,
            f!(
                "authentication-types",
                "Authentication Types",
                FieldKind::Repeat
            ),
            f!("encryption", "Encryption", FieldKind::Repeat),
            f!("passphrase", "Passphrase", FieldKind::Secret),
            f!("disable-pmkid", "Disable PMKID", FieldKind::Toggle),
            f!(
                "management-protection",
                "Management Protection",
                FieldKind::LabeledEnum {
                    choices: MGMT_PROTECT_CHOICES
                }
            ),
            f!(
                "wps",
                "WPS",
                FieldKind::LabeledEnum {
                    choices: WPS_CHOICES
                }
            ),
            ENABLED,
            COMMENT,
        ],
    },
    FormSection {
        id: "eap",
        label: "EAP",
        read_only: false,
        fields: &[
            f!("eap-methods", "Methods", FieldKind::Repeat),
            f!("tls-certificate", "TLS Certificate", LOOKUP_CERT),
            f!("eap-password", "Password", FieldKind::Secret),
        ],
    },
    FormSection {
        id: "ft",
        label: "FT",
        read_only: false,
        fields: &[
            f!("ft", "FT Enabled", FieldKind::Toggle),
            f!("ft-over-ds", "FT Over DS", FieldKind::Toggle),
        ],
    },
];

pub static WIFI_SECURITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["authentication-types"],
    sections: SECURITY_PROFILE_SECTIONS,
    create_sections: SECURITY_PROFILE_SECTIONS,
};

const DATAPATH_FIELDS: &[FieldSpec] = &[
    NAME,
    f!("bridge", "Bridge", LOOKUP_BRIDGE),
    f!("client-isolation", "Client Isolation", FieldKind::Toggle),
    f!(
        "traffic-processing",
        "Traffic Processing",
        FieldKind::LabeledEnum {
            choices: TRAFFIC_PROCESSING_CHOICES
        }
    ),
    f!("vlan-id", "VLAN ID", OPTIONAL_NUM),
    f!("interface-list", "Interface List", LOOKUP_IFACE_LIST),
    ENABLED,
    COMMENT,
];

pub static WIFI_DATAPATH_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["bridge"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: DATAPATH_FIELDS,
    }],
    create_sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: DATAPATH_FIELDS,
    }],
};

const CONFIG_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "configuration",
        label: "Configuration",
        read_only: false,
        fields: &[
            NAME,
            f!("ssid", "SSID", FieldKind::Text),
            f!("country", "Country", KIND_WIFI_COUNTRY),
            f!("hide-ssid", "Hide SSID", FieldKind::Toggle),
            f!(
                "mode",
                "Mode",
                FieldKind::LabeledEnum {
                    choices: MODE_CHOICES
                }
            ),
            ENABLED,
            COMMENT,
        ],
    },
    FormSection {
        id: "channel",
        label: "Channel",
        read_only: false,
        fields: &[
            f!("channel", "Channel", LOOKUP_WIFI_CHANNEL),
            f!(
                "channel.band",
                "Band",
                FieldKind::LabeledEnum {
                    choices: BAND_CHOICES
                }
            ),
        ],
    },
    FormSection {
        id: "security",
        label: "Security",
        read_only: false,
        fields: &[f!("security", "Security", LOOKUP_WIFI_SECURITY)],
    },
    FormSection {
        id: "datapath",
        label: "Datapath",
        read_only: false,
        fields: &[f!("datapath", "Datapath", LOOKUP_WIFI_DATAPATH)],
    },
];

pub static WIFI_CONFIGURATION_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ssid"],
    sections: CONFIG_SECTIONS,
    create_sections: CONFIG_SECTIONS,
};

const PROVISIONING_FIELDS: &[FieldSpec] = &[
    f!(
        "radio-mac",
        "Radio MAC",
        FieldKind::Optional {
            kind: ScalarKind::Mac,
            unset: "",
            unset_label: "any"
        }
    ),
    f!("identity-regexp", "Identity Regexp", OPTIONAL_TEXT),
    f!("common-name-regexp", "Common Name Regexp", OPTIONAL_TEXT),
    f!("address-ranges", "Address Ranges", FieldKind::Repeat),
    f!("supported-bands", "Supported Bands", FieldKind::Repeat),
    f!(
        "action",
        "Action",
        FieldKind::LabeledEnum {
            choices: PROVISION_ACTION_CHOICES
        }
    ),
    f!(
        "master-configuration",
        "Master Configuration",
        LOOKUP_WIFI_CONFIG
    ),
    f!(
        "slave-configurations",
        "Slave Configurations",
        LOOKUP_WIFI_CONFIGS
    ),
    f!("name-format", "Name Format", OPTIONAL_TEXT),
    f!("slave-name-format", "Slave Name Format", OPTIONAL_TEXT),
    ENABLED,
    COMMENT,
];

pub static WIFI_PROVISIONING_FORM: FormSchema = FormSchema {
    title_key: "action",
    subtitle_keys: &["master-configuration"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: PROVISIONING_FIELDS,
    }],
    create_sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: PROVISIONING_FIELDS,
    }],
};

const CAP_ENABLED: FieldSpec = f!("enabled", "Enabled", FieldKind::Toggle);

pub static WIFI_CAP_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: &[
            CAP_ENABLED,
            f!(
                "discovery-interfaces",
                "Discovery Interfaces",
                LOOKUP_IFACES
            ),
            f!("certificate", "Certificate", LOOKUP_CERT),
            f!("caps-man-addresses", "CAPsMAN Addresses", FieldKind::Repeat),
            f!("caps-man-names", "CAPsMAN Names", FieldKind::Repeat),
            f!("lock-to-caps-man", "Lock To CAPsMAN", FieldKind::Toggle),
            f!("slaves-static", "Slaves Static", FieldKind::Toggle),
            f!("slaves-datapath", "Slaves Datapath", LOOKUP_WIFI_DATAPATH),
            f!(
                "requested-certificate",
                "Requested Certificate",
                FieldKind::Readonly
            ),
            f!(
                "current-caps-man-address",
                "Current CAPsMAN address",
                FieldKind::Readonly
            ),
            f!(
                "current-caps-man-identity",
                "Current CAPsMAN Identity",
                FieldKind::Readonly
            ),
        ],
    }],
    create_sections: &[],
};

pub static WIFI_CAPSMAN_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: &[
            CAP_ENABLED,
            f!("interfaces", "Interfaces", LOOKUP_IFACES),
            f!("ca-certificate", "CA Certificate", LOOKUP_CERT),
            f!("certificate", "Certificate", LOOKUP_CERT),
            f!(
                "require-peer-certificate",
                "Require Peer Certificate",
                FieldKind::Toggle
            ),
            f!("package-path", "Package Path", OPTIONAL_TEXT),
            f!(
                "upgrade-policy",
                "Upgrade Policy",
                FieldKind::LabeledEnum {
                    choices: UPGRADE_POLICY_CHOICES
                }
            ),
            f!(
                "generated-ca-certificate",
                "Generated CA Certificate",
                FieldKind::Readonly
            ),
            f!(
                "generated-certificate",
                "Generated Certificate",
                FieldKind::Readonly
            ),
        ],
    }],
    create_sections: &[],
};

const WIRELESS_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "ap-bridge",
        value: "ap-bridge",
    },
    EnumChoice {
        label: "bridge",
        value: "bridge",
    },
    EnumChoice {
        label: "station",
        value: "station",
    },
    EnumChoice {
        label: "station-bridge",
        value: "station-bridge",
    },
    EnumChoice {
        label: "station-pseudobridge",
        value: "station-pseudobridge",
    },
    EnumChoice {
        label: "station-wds",
        value: "station-wds",
    },
    EnumChoice {
        label: "wds-slave",
        value: "wds-slave",
    },
    EnumChoice {
        label: "alignment-only",
        value: "alignment-only",
    },
];

const WIRELESS_SECTIONS: &[FormSection] = &[
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
                    min: Some(32),
                    max: Some(2290)
                }
            ),
            f!("actual-mtu", "Actual MTU", FieldKind::Readonly),
            f!(
                "l2mtu",
                "L2 MTU",
                FieldKind::ConstrainedNumber {
                    min: Some(32),
                    max: Some(2290)
                }
            ),
            f!("vrf", "VRF", FieldKind::Readonly),
            f!("mac-address", "MAC Address", FieldKind::Mac),
            ARP,
            f!(
                "arp-timeout",
                "ARP Timeout",
                FieldKind::Optional {
                    kind: ScalarKind::Time,
                    unset: "auto",
                    unset_label: "Auto"
                }
            ),
        ],
    },
    FormSection {
        id: "wireless",
        label: "Wireless",
        read_only: false,
        fields: &[
            f!(
                "mode",
                "Mode",
                FieldKind::LabeledEnum {
                    choices: WIRELESS_MODE_CHOICES
                }
            ),
            f!("band", "Band", KIND_WIRELESS_BAND),
            f!(
                "channel-width",
                "Channel Width",
                KIND_WIRELESS_CHANNEL_WIDTH
            ),
            f!("frequency", "Frequency", FieldKind::Text),
            f!("ssid", "SSID", FieldKind::Text),
            f!("radio-name", "Radio Name", FieldKind::Text),
            f!("scan-list", "Scan List", FieldKind::Repeat),
            f!(
                "security-profile",
                "Security Profile",
                LOOKUP_WIRELESS_SECURITY
            ),
            f!("hide-ssid", "Hide SSID", FieldKind::Toggle),
            f!("master-interface", "Master Interface", LOOKUP_IFACE),
        ],
    },
    FormSection {
        id: "ht",
        label: "HT",
        read_only: false,
        fields: &[
            f!("ht-txchains", "Tx Chains", FieldKind::Repeat),
            f!("ht-rxchains", "Rx Chains", FieldKind::Repeat),
        ],
    },
    FormSection {
        id: "wds",
        label: "WDS",
        read_only: false,
        fields: &[
            f!("wds-mode", "WDS Mode", KIND_WDS_MODE),
            f!("wds-default-bridge", "WDS Default Bridge", LOOKUP_BRIDGE),
        ],
    },
    FormSection {
        id: "nstreme",
        label: "Nstreme",
        read_only: false,
        fields: &[f!("enable-nstreme", "Enable Nstreme", FieldKind::Toggle)],
    },
    FormSection {
        id: "nv2",
        label: "NV2",
        read_only: false,
        fields: &[
            f!(
                "nv2-tdma-period-size",
                "TDMA Period Size",
                KIND_NV2_TDMA_PERIOD
            ),
            f!("nv2-preshared-key", "Preshared Key", FieldKind::Secret),
        ],
    },
    FormSection {
        id: "status",
        label: "Status",
        read_only: true,
        fields: LINK_STATUS,
    },
];

pub static WIRELESS_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["ssid", "mode"],
    sections: WIRELESS_SECTIONS,
    create_sections: &[
        WIRELESS_SECTIONS[0],
        WIRELESS_SECTIONS[1],
        WIRELESS_SECTIONS[2],
        WIRELESS_SECTIONS[3],
        WIRELESS_SECTIONS[4],
        WIRELESS_SECTIONS[5],
    ],
};

const WIRELESS_SECURITY_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "none",
        value: "none",
    },
    EnumChoice {
        label: "static keys optional",
        value: "static-keys-optional",
    },
    EnumChoice {
        label: "static keys required",
        value: "static-keys-required",
    },
    EnumChoice {
        label: "dynamic keys",
        value: "dynamic-keys",
    },
];

const WIRELESS_SECURITY_SECTIONS: &[FormSection] = &[
    FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!(
                "mode",
                "Mode",
                FieldKind::LabeledEnum {
                    choices: WIRELESS_SECURITY_MODE_CHOICES
                }
            ),
            f!(
                "authentication-types",
                "Authentication Types",
                FieldKind::Repeat
            ),
            f!("unicast-ciphers", "Unicast Ciphers", FieldKind::Repeat),
            f!("group-ciphers", "Group Ciphers", FieldKind::Repeat),
            f!(
                "wpa-pre-shared-key",
                "WPA Pre-Shared Key",
                FieldKind::Secret
            ),
            f!(
                "wpa2-pre-shared-key",
                "WPA2 Pre-Shared Key",
                FieldKind::Secret
            ),
            f!(
                "supplicant-identity",
                "Supplicant Identity",
                FieldKind::Text
            ),
            f!(
                "management-protection",
                "Management Protection",
                FieldKind::LabeledEnum {
                    choices: MGMT_PROTECT_CHOICES
                }
            ),
            f!("disable-pmkid", "Disable PMKID", FieldKind::Toggle),
        ],
    },
    FormSection {
        id: "radius",
        label: "RADIUS",
        read_only: false,
        fields: &[
            f!(
                "radius-mac-authentication",
                "MAC Authentication",
                FieldKind::Toggle
            ),
            f!("radius-mac-accounting", "MAC Accounting", FieldKind::Toggle),
            f!("radius-eap-accounting", "EAP Accounting", FieldKind::Toggle),
        ],
    },
    FormSection {
        id: "eap",
        label: "EAP",
        read_only: false,
        fields: &[
            f!("eap-methods", "Methods", FieldKind::Repeat),
            f!("tls-mode", "TLS Mode", KIND_TLS_MODE),
            f!("tls-certificate", "TLS Certificate", LOOKUP_CERT),
        ],
    },
    FormSection {
        id: "static-keys",
        label: "Static Keys",
        read_only: false,
        fields: &[
            f!("static-key-0", "Key 0", FieldKind::Secret),
            f!("static-key-1", "Key 1", FieldKind::Secret),
            f!("static-key-2", "Key 2", FieldKind::Secret),
            f!("static-key-3", "Key 3", FieldKind::Secret),
            f!(
                "static-sta-private-key",
                "St. Private Key",
                FieldKind::Secret
            ),
        ],
    },
];

pub static WIRELESS_SECURITY_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["mode"],
    sections: WIRELESS_SECURITY_SECTIONS,
    create_sections: WIRELESS_SECURITY_SECTIONS,
};

const ACCESS_LIST_FIELDS: &[FieldSpec] = &[
    f!(
        "mac-address",
        "MAC Address",
        FieldKind::Optional {
            kind: ScalarKind::Mac,
            unset: "",
            unset_label: "any"
        }
    ),
    INTERFACE,
    f!("signal-range", "Signal Strength Range", FieldKind::Text),
    f!("authentication", "Authentication", FieldKind::Toggle),
    f!("forwarding", "Forwarding", FieldKind::Toggle),
    f!("vlan-mode", "VLAN Mode", KIND_WIRELESS_VLAN_MODE),
    f!(
        "vlan-id",
        "VLAN ID",
        FieldKind::ConstrainedNumber {
            min: Some(1),
            max: Some(4095)
        }
    ),
    f!(
        "private-pre-shared-key",
        "Private Pre Shared Key",
        FieldKind::Secret
    ),
    f!(
        "management-protection-key",
        "Management Protection Key",
        FieldKind::Secret
    ),
    ENABLED,
    COMMENT,
];

pub static WIRELESS_ACCESS_LIST_FORM: FormSchema = FormSchema {
    title_key: "mac-address",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: ACCESS_LIST_FIELDS,
    }],
    create_sections: &[FormSection {
        id: "form",
        label: "",
        read_only: false,
        fields: ACCESS_LIST_FIELDS,
    }],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_interface_keeps_webfig_section_order() {
        assert_eq!(
            WIFI_FORM
                .sections
                .iter()
                .map(|section| section.id)
                .collect::<Vec<_>>(),
            [
                "general",
                "configuration",
                "channel",
                "security",
                "eap",
                "ft",
                "aaa",
                "datapath",
                "interworking",
                "steering",
                "status",
                "traffic"
            ]
        );
        assert_eq!(
            WIFI_FORM.field("master-interface").unwrap().kind,
            LOOKUP_WIFI
        );
        assert!(matches!(
            WIFI_FORM.field("configuration.mode").unwrap().kind,
            FieldKind::LabeledEnum { .. }
        ));
    }

    #[test]
    fn cap_enabled_is_not_disabled_polarity() {
        assert_eq!(
            WIFI_CAP_FORM.field("enabled").unwrap().kind,
            FieldKind::Toggle
        );
        assert!(WIFI_CAP_FORM.field("disabled").is_none());
        assert_eq!(
            WIFI_CAPSMAN_FORM.field("enabled").unwrap().kind,
            FieldKind::Toggle
        );
    }

    #[test]
    fn wireless_keeps_requested_screens() {
        assert!(WIRELESS_FORM.field("security-profile").is_some());
        assert_eq!(
            WIRELESS_SECURITY_FORM
                .sections
                .iter()
                .map(|section| section.id)
                .collect::<Vec<_>>(),
            ["general", "radius", "eap", "static-keys"]
        );
        assert_eq!(
            WIRELESS_ACCESS_LIST_FORM.field("mac-address").unwrap().kind,
            FieldKind::Optional {
                kind: ScalarKind::Mac,
                unset: "",
                unset_label: "any"
            }
        );
    }
}
