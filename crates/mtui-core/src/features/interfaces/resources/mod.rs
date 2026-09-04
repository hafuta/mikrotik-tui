//! Feature-owned catalog entries for the complete Interfaces navigation group.

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        crate::resources::ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

mod base;
mod cellular;
mod tunnels;
mod virtuals;
mod wifi;
mod wireless;

use crate::resources::ResourceSpec;

pub(crate) static RESOURCES: &[ResourceSpec] = &[
    base::INTERFACES,
    base::INTERFACE_LISTS,
    base::INTERFACE_LIST_MEMBERS,
    base::ETHERNET,
    tunnels::EOIP,
    tunnels::IPIP,
    tunnels::GRE,
    tunnels::SIX_TO_FOUR,
    tunnels::GRE6,
    virtuals::VLAN,
    virtuals::VXLAN,
    virtuals::VRRP,
    virtuals::BONDING,
    cellular::LTE,
    cellular::LTE_APN,
    wifi::WIFI,
    wifi::WIFI_SECURITY,
    wifi::WIFI_CHANNEL,
    wifi::WIFI_DATAPATH,
    wifi::WIFI_CONFIGURATION,
    wifi::WIFI_PROVISIONING,
    wifi::WIFI_CAP,
    wifi::WIFI_CAPSMAN,
    wifi::WIFI_REGISTRATION_TABLE,
    wireless::WIRELESS,
    wireless::WIRELESS_SECURITY_PROFILES,
    wireless::WIRELESS_ACCESS_LIST,
    wireless::WIRELESS_REGISTRATION_TABLE,
    virtuals::MACVLAN,
    virtuals::VETH,
    virtuals::MACSEC,
    virtuals::MACSEC_PROFILES,
    virtuals::VRF,
    virtuals::DETECT_INTERNET,
];
