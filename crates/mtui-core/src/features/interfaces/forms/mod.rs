//! Feature-owned form schemas for the complete Interfaces navigation group.

use crate::forms::{EnumChoice, FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const L2MTU: FieldSpec = f!("l2mtu", "L2 MTU", FieldKind::Number);
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const SLAVE: FieldSpec = f!("slave", "Slave", FieldKind::Readonly);
const IFACE_TYPE: FieldSpec = f!("type", "Type", FieldKind::Readonly);
const DEFAULT_NAME: FieldSpec = f!("default-name", "Default Name", FieldKind::Readonly);
const ARP_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "Enabled",
        value: "enabled",
    },
    EnumChoice {
        label: "Disabled",
        value: "disabled",
    },
    EnumChoice {
        label: "Proxy ARP",
        value: "proxy-arp",
    },
    EnumChoice {
        label: "Reply Only",
        value: "reply-only",
    },
    EnumChoice {
        label: "Local Proxy ARP",
        value: "local-proxy-arp",
    },
];
const ARP: FieldSpec = f!(
    "arp",
    "ARP",
    FieldKind::LabeledEnum {
        choices: ARP_CHOICES
    }
);

const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACES: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: true,
};
const LOOKUP_IFACE_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACE_LISTS: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: true,
};
const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};
const LOOKUP_MACSEC_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "macsec-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_LTE_APN: FieldKind = FieldKind::Lookup {
    resource_id: "lte-apn",
    value_key: "name",
    multiple: true,
};

const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);

pub(crate) mod base;
pub(crate) mod cellular;
pub(crate) mod tunnels;
pub(crate) mod virtuals;
pub(crate) mod wifi;

pub(crate) use base::{ETHERNET_FORM, INTERFACES_FORM, VLAN_FORM};
pub(crate) use cellular::{LTE_APN_FORM, LTE_FORM};
pub(crate) use tunnels::{BONDING_FORM, EOIP_FORM, GRE_FORM, IPIP_FORM, VRRP_FORM, VXLAN_FORM};
pub(crate) use virtuals::{
    DETECT_INTERNET_FORM, LIST_FORM, MACSEC_FORM, MACSEC_PROFILE_FORM, MACVLAN_FORM, MEMBER_FORM,
    VETH_FORM, VRF_FORM,
};
pub(crate) use wifi::{
    WIFI_CAP_FORM, WIFI_CAPSMAN_FORM, WIFI_CHANNEL_FORM, WIFI_CONFIGURATION_FORM,
    WIFI_DATAPATH_FORM, WIFI_FORM, WIFI_PROVISIONING_FORM, WIFI_SECURITY_FORM,
    WIRELESS_ACCESS_LIST_FORM, WIRELESS_FORM, WIRELESS_SECURITY_FORM,
};
