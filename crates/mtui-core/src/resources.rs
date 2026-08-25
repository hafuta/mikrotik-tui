//! Descriptor-driven `RouterOS` resource catalog and navigation tree.

use std::time::Duration;

use crate::actions::ActionSpec;
use crate::forms::FormSchema;

/// Dashboard nav / content id (not a list resource).
pub const DASHBOARD_ID: &str = "dashboard";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchKind {
    /// List-like collection.
    List { endpoint: &'static str },
    /// Singleton system resource.
    System { endpoint: &'static str },
    /// Overlay-driven screen; never polled.
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSpec {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub fetch: FetchKind,
    pub columns: &'static [ColumnSpec],
    pub refresh: Duration,
    pub actions: &'static [ActionSpec],
    pub form: Option<&'static FormSchema>,
}

impl ResourceSpec {
    #[must_use]
    pub fn endpoint(&self) -> &'static str {
        match self.fetch {
            FetchKind::List { endpoint } | FetchKind::System { endpoint } => endpoint,
            FetchKind::Local => "",
        }
    }

    #[must_use]
    pub fn cli_path(&self) -> &str {
        match self.fetch {
            FetchKind::Local => match self.id {
                "ping" => "/tool/ping",
                "traceroute" => "/tool/traceroute",
                "bandwidth-test" => "/tool/bandwidth-test",
                "flood-ping" => "/tool/flood-ping",
                "mac-scan" => "/tool/mac-scan",
                "ip-scan" => "/tool/ip-scan",
                "profiler" => "/tool/profile",
                "wol" => "/tool/wol",
                "sms" => "/tool/sms",
                "reset-configuration" => "/system/reset-configuration",
                other => other,
            },
            FetchKind::List { endpoint } | FetchKind::System { endpoint } => {
                endpoint.trim_start_matches("/rest")
            }
        }
    }

    #[must_use]
    pub fn is_singleton(&self) -> bool {
        matches!(self.fetch, FetchKind::System { .. })
    }

    #[must_use]
    #[allow(clippy::implicit_hasher)]
    pub fn resolved_actions(
        &self,
        row: Option<&std::collections::HashMap<String, String>>,
    ) -> Vec<&ActionSpec> {
        crate::actions::resolve_actions(self.actions, self.is_singleton(), row)
    }
}

macro_rules! col {
    ($key:literal, $title:literal, $width:expr) => {
        ColumnSpec {
            key: $key,
            title: $title,
            width: $width,
        }
    };
}

pub static ALL_RESOURCES: &[ResourceSpec] = &[
    ResourceSpec {
        id: "interfaces",
        group: "interfaces-group",
        label: "Interface",
        fetch: FetchKind::List {
            endpoint: "/rest/interface",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("type", "Type", 14),
            col!("mtu", "MTU", 7),
            col!("actual-mtu", "Actual MTU", 11),
            col!("l2mtu", "L2 MTU", 8),
            col!("max-l2mtu", "Max L2 MTU", 11),
            col!("mac-address", "MAC address", 18),
            col!("tx-byte", "TX", 12),
            col!("rx-byte", "RX", 12),
            col!("tx-packet", "TX pkt", 10),
            col!("rx-packet", "RX pkt", 10),
            col!("fp-tx-byte", "FP TX", 12),
            col!("fp-rx-byte", "FP RX", 12),
            col!("fp-tx-packet", "FP TX pkt", 10),
            col!("fp-rx-packet", "FP RX pkt", 10),
            col!("last-link-up-time", "Last link up", 20),
            col!("last-link-down-time", "Last link down", 20),
            col!("link-downs", "Link downs", 11),
            col!("tx-drop", "TX drop", 9),
            col!("rx-drop", "RX drop", 9),
            col!("tx-queue-drop", "TX q-drop", 10),
            col!("rx-error", "RX error", 9),
            col!("tx-error", "TX error", 9),
            col!("running", "Run", 5),
            col!("slave", "Slave", 6),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::INTERFACE_LIST_ACTIONS,
        form: Some(&crate::interface_write::INTERFACES_FORM),
    },
    ResourceSpec {
        id: "interface-lists",
        group: "interfaces-group",
        label: "Lists",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/list",
        },
        columns: &[
            col!("name", "Name", 20),
            col!("include", "Include", 24),
            col!("exclude", "Exclude", 24),
            col!("builtin", "Builtin", 8),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::INTERFACE_LIST_DEF_ACTIONS,
        form: Some(&crate::interface_write::LIST_FORM),
    },
    ResourceSpec {
        id: "interface-list-members",
        group: "interfaces-group",
        label: "List members",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/list/member",
        },
        columns: &[
            col!("list", "List", 16),
            col!("interface", "Interface", 18),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::INTERFACE_LIST_MEMBER_ACTIONS,
        form: Some(&crate::interface_write::MEMBER_FORM),
    },
    ResourceSpec {
        id: "ethernet",
        group: "interfaces-group",
        label: "Ethernet",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("default-name", "Default name", 16),
            col!("mtu", "MTU", 7),
            col!("l2mtu", "L2 MTU", 8),
            col!("mac-address", "MAC address", 18),
            col!("orig-mac-address", "Orig MAC", 18),
            col!("arp", "ARP", 16),
            col!("auto-negotiation", "Auto-neg", 9),
            col!("advertise", "Advertise", 28),
            col!("speed", "Speed", 16),
            col!("full-duplex", "Duplex", 8),
            col!("switch", "Switch", 12),
            col!("loop-protect", "Loop protect", 13),
            col!("loop-protect-status", "Loop status", 12),
            col!("running", "Run", 5),
            col!("slave", "Slave", 6),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::ETHERNET_ACTIONS,
        form: Some(&crate::interface_write::ETHERNET_FORM),
    },
    ResourceSpec {
        id: "eoip",
        group: "interfaces-group",
        label: "EoIP Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/eoip",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("tunnel-id", "Tunnel ID", 10),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("mac-address", "MAC address", 18),
            col!("arp", "ARP", 16),
            col!("keepalive", "Keepalive", 12),
            col!("allow-fast-path", "Fast path", 10),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::EOIP_FORM),
    },
    ResourceSpec {
        id: "ipip",
        group: "interfaces-group",
        label: "IP Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ipip",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("clamp-tcp-mss", "Clamp MSS", 10),
            col!("dscp", "DSCP", 8),
            col!("allow-fast-path", "Fast path", 10),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::IPIP_FORM),
    },
    ResourceSpec {
        id: "gre",
        group: "interfaces-group",
        label: "GRE Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/gre",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("keepalive", "Keepalive", 12),
            col!("dscp", "DSCP", 8),
            col!("clamp-tcp-mss", "Clamp MSS", 10),
            col!("allow-fast-path", "Fast path", 10),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::GRE_FORM),
    },
    ResourceSpec {
        id: "6to4",
        group: "interfaces-group",
        label: "6to4 Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/6to4",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::IPIP_FORM),
    },
    ResourceSpec {
        id: "sit",
        group: "interfaces-group",
        label: "SIT Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/sit",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::IPIP_FORM),
    },
    ResourceSpec {
        id: "gre6",
        group: "interfaces-group",
        label: "GRE6 Tunnel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/gre6",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("mtu", "MTU", 7),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::GRE_FORM),
    },
    ResourceSpec {
        id: "vlan",
        group: "interfaces-group",
        label: "VLAN",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/vlan",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("vlan-id", "VLAN ID", 8),
            col!("interface", "Interface", 16),
            col!("mtu", "MTU", 7),
            col!("l2mtu", "L2 MTU", 8),
            col!("mac-address", "MAC address", 18),
            col!("arp", "ARP", 16),
            col!("use-service-tag", "Service tag", 12),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::VLAN_FORM),
    },
    ResourceSpec {
        id: "vxlan",
        group: "interfaces-group",
        label: "VXLAN",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/vxlan",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("vni", "VNI", 8),
            col!("port", "Port", 8),
            col!("group", "Group", 16),
            col!("local", "Local", 18),
            col!("interface", "Interface", 16),
            col!("vrf", "VRF", 12),
            col!("mtu", "MTU", 7),
            col!("mac-address", "MAC address", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::VXLAN_FORM),
    },
    ResourceSpec {
        id: "vrrp",
        group: "interfaces-group",
        label: "VRRP",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/vrrp",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("vrid", "VRID", 6),
            col!("priority", "Priority", 9),
            col!("interval", "Interval", 10),
            col!("version", "Version", 8),
            col!("v3-protocol", "V3 proto", 10),
            col!("preemption-mode", "Preempt", 8),
            col!("mac-address", "MAC address", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::VRRP_FORM),
    },
    ResourceSpec {
        id: "bonding",
        group: "interfaces-group",
        label: "Bonding",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bonding",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("slaves", "Slaves", 28),
            col!("mode", "Mode", 16),
            col!("link-monitoring", "Monitor", 16),
            col!("transmit-hash-policy", "Hash", 16),
            col!("primary", "Primary", 16),
            col!("mtu", "MTU", 7),
            col!("mac-address", "MAC address", 18),
            col!("arp", "ARP", 16),
            col!("min-links", "Min links", 10),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::BONDING_FORM),
    },
    ResourceSpec {
        id: "lte",
        group: "interfaces-group",
        label: "LTE",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/lte",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("default-name", "Default name", 16),
            col!("mtu", "MTU", 7),
            col!("mac-address", "MAC address", 18),
            col!("network-mode", "Network", 14),
            col!("apn-profiles", "APN Profiles", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::LTE_ACTIONS,
        form: Some(&crate::interface_write::LTE_FORM),
    },
    ResourceSpec {
        id: "lte-apn",
        group: "interfaces-group",
        label: "LTE APN",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/lte/apn",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("apn", "APN", 18),
            col!("authentication", "Authentication", 14),
            col!("ip-type", "IP Type", 12),
            col!("use-network-apn", "Network APN", 12),
            col!("add-default-route", "Default", 8),
            col!("password", "Password", 10),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::LTE_APN_FORM),
    },
    ResourceSpec {
        id: "wifi",
        group: "interfaces-group",
        label: "WiFi",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("default-name", "Default name", 16),
            col!("configuration", "Configuration", 20),
            col!("master-interface", "Master", 16),
            col!("mac-address", "MAC address", 18),
            col!("radio-mac", "Radio MAC", 18),
            col!("current-channel", "Channel", 16),
            col!("ssid", "SSID", 20),
            col!("mtu", "MTU", 7),
            col!("l2mtu", "L2 MTU", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::RADIO_ACTIONS,
        form: Some(&crate::interface_write::WIFI_FORM),
    },
    ResourceSpec {
        id: "wifi-security",
        group: "interfaces-group",
        label: "WiFi Security",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi/security",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("authentication-types", "Auth", 18),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::WIFI_SECURITY_FORM),
    },
    ResourceSpec {
        id: "wifi-channel",
        group: "interfaces-group",
        label: "WiFi Channel",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi/channel",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("band", "Band", 12),
            col!("frequency", "Frequency", 14),
            col!("width", "Width", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::WIFI_CHANNEL_FORM),
    },
    ResourceSpec {
        id: "wifi-datapath",
        group: "interfaces-group",
        label: "WiFi Datapath",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi/datapath",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("bridge", "Bridge", 16),
            col!("vlan-id", "VLAN", 6),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::WIFI_DATAPATH_FORM),
    },
    ResourceSpec {
        id: "wifi-configuration",
        group: "interfaces-group",
        label: "WiFi Configuration",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi/configuration",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("ssid", "SSID", 20),
            col!("country", "Country", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::WIFI_CONFIGURATION_FORM),
    },
    ResourceSpec {
        id: "wifi-provisioning",
        group: "interfaces-group",
        label: "WiFi Provisioning",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wifi/provisioning",
        },
        columns: &[
            col!("action", "Action", 14),
            col!("supported-bands", "Bands", 16),
            col!("master-configuration", "Master", 18),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::interface_write::WIFI_PROVISIONING_FORM),
    },
    ResourceSpec {
        id: "wifi-cap",
        group: "interfaces-group",
        label: "WiFi CAP",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/wifi/cap",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("caps-man-addresses", "CAPsMAN", 24),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::interface_write::WIFI_CAP_FORM),
    },
    ResourceSpec {
        id: "wifi-capsman",
        group: "interfaces-group",
        label: "CAPsMAN",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/wifi/capsman",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("ca-certificate", "CA", 18),
            col!("certificate", "Certificate", 18),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::interface_write::WIFI_CAPSMAN_FORM),
    },
    ResourceSpec {
        id: "wireless",
        group: "interfaces-group",
        label: "Wireless",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireless",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("default-name", "Default name", 16),
            col!("ssid", "SSID", 20),
            col!("mode", "Mode", 14),
            col!("band", "Band", 14),
            col!("frequency", "Frequency", 12),
            col!("mac-address", "MAC address", 18),
            col!("mtu", "MTU", 7),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::RADIO_ACTIONS,
        form: Some(&crate::interface_write::WIRELESS_FORM),
    },
    ResourceSpec {
        id: "wireless-security-profiles",
        group: "interfaces-group",
        label: "Security Profiles",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireless/security-profiles",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("mode", "Mode", 12),
            col!("authentication-types", "Auth", 18),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::WIRELESS_SECURITY_FORM),
    },
    ResourceSpec {
        id: "wireless-access-list",
        group: "interfaces-group",
        label: "Access List",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireless/access-list",
        },
        columns: &[
            col!("mac-address", "MAC", 18),
            col!("interface", "Interface", 16),
            col!("authentication", "Auth", 6),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::interface_write::WIRELESS_ACCESS_LIST_FORM),
    },
    ResourceSpec {
        id: "wireless-registration-table",
        group: "interfaces-group",
        label: "Registration",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireless/registration-table",
        },
        columns: &[
            col!("mac-address", "MAC", 18),
            col!("interface", "Interface", 16),
            col!("ap", "AP", 5),
            col!("signal-strength", "Signal", 10),
            col!("uptime", "Uptime", 10),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "wireguard",
        group: "wireguard-group",
        label: "WireGuard",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireguard",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("listen-port", "Listen", 8),
            col!("public-key", "Public key", 44),
            col!("private-key", "Private key", 12),
            col!("mtu", "MTU", 7),
            col!("vrf", "VRF", 12),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::wireguard_write::WIREGUARD_FORM),
    },
    ResourceSpec {
        id: "wireguard-peers",
        group: "wireguard-group",
        label: "WireGuard Peers",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/wireguard/peers",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("public-key", "Public key", 44),
            col!("endpoint-address", "Endpoint", 22),
            col!("endpoint-port", "Port", 8),
            col!("allowed-address", "Allowed", 28),
            col!("persistent-keepalive", "Keepalive", 10),
            col!("responder", "Responder", 10),
            col!("current-endpoint-address", "Current", 22),
            col!("current-endpoint-port", "Cur port", 9),
            col!("last-handshake", "Handshake", 14),
            col!("rx", "RX", 12),
            col!("tx", "TX", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::wireguard_write::WIREGUARD_PEER_FORM),
    },
    ResourceSpec {
        id: "macvlan",
        group: "interfaces-group",
        label: "MACVLAN",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/macvlan",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("mode", "Mode", 12),
            col!("mac-address", "MAC address", 18),
            col!("mtu", "MTU", 7),
            col!("arp", "ARP", 16),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::MACVLAN_FORM),
    },
    ResourceSpec {
        id: "macsec",
        group: "interfaces-group",
        label: "MACsec",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/macsec",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("profile", "Profile", 14),
            col!("mtu", "MTU", 7),
            col!("status", "Status", 16),
            col!("ckn", "CKN", 24),
            col!("cak", "CAK", 10),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::interface_write::MACSEC_FORM),
    },
    ResourceSpec {
        id: "macsec-profiles",
        group: "interfaces-group",
        label: "MACsec Profile",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/macsec/profile",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("server-priority", "Server priority", 16),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::interface_write::MACSEC_PROFILE_FORM),
    },
    ResourceSpec {
        id: "vrf",
        group: "interfaces-group",
        label: "VRF",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/vrf",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interfaces", "Interfaces", 36),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::VRF_ACTIONS,
        form: Some(&crate::interface_write::VRF_FORM),
    },
    ResourceSpec {
        id: "detect-internet",
        group: "interfaces-group",
        label: "Detect Internet",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/detect-internet",
        },
        columns: &[
            col!("detect-interface-list", "Detect", 16),
            col!("lan-interface-list", "LAN", 16),
            col!("wan-interface-list", "WAN", 16),
            col!("internet-interface-list", "Internet", 16),
            col!("state", "State", 12),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::interface_write::DETECT_INTERNET_FORM),
    },
    ResourceSpec {
        id: "ppp-secrets",
        group: "ppp-group",
        label: "Secrets",
        fetch: FetchKind::List {
            endpoint: "/rest/ppp/secret",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("service", "Service", 10),
            col!("profile", "Profile", 18),
            col!("caller-id", "Caller", 18),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("remote-ipv6-prefix", "IPv6 prefix", 22),
            col!("password", "Password", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ppp_write::PPP_SECRET_FORM),
    },
    ResourceSpec {
        id: "ppp-profiles",
        group: "ppp-group",
        label: "Profiles",
        fetch: FetchKind::List {
            endpoint: "/rest/ppp/profile",
        },
        columns: &[
            col!("name", "Name", 20),
            col!("local-address", "Local", 18),
            col!("remote-address", "Remote", 18),
            col!("dns-server", "DNS", 24),
            col!("rate-limit", "Rate limit", 18),
            col!("only-one", "Only one", 9),
            col!("use-encryption", "Encrypt", 9),
            col!("use-compression", "Compress", 9),
            col!("change-tcp-mss", "MSS", 8),
            col!("bridge", "Bridge", 16),
            col!("interface-list", "List", 16),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ppp_write::PPP_PROFILE_FORM),
    },
    ResourceSpec {
        id: "ppp-active",
        group: "ppp-group",
        label: "Active",
        fetch: FetchKind::List {
            endpoint: "/rest/ppp/active",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("service", "Service", 10),
            col!("caller-id", "Caller", 18),
            col!("address", "Address", 18),
            col!("uptime", "Uptime", 12),
            col!("encoding", "Encoding", 16),
            col!("session-id", "Session", 12),
            col!("limit-bytes-in", "Limit in", 12),
            col!("limit-bytes-out", "Limit out", 12),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ppp-aaa",
        group: "ppp-group",
        label: "AAA",
        fetch: FetchKind::System {
            endpoint: "/rest/ppp/aaa",
        },
        columns: &[
            col!("use-radius", "RADIUS", 8),
            col!("accounting", "Accounting", 11),
            col!("interim-update", "Interim", 10),
            col!("enable-ipv6-accounting", "IPv6 acct", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ppp_write::PPP_AAA_FORM),
    },
    ResourceSpec {
        id: "ppp-client",
        group: "ppp-group",
        label: "PPP Client",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ppp-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("port", "Port", 14),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("profile", "Profile", 16),
            col!("phone", "Phone", 16),
            col!("add-default-route", "Default", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::PPP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "pppoe-clients",
        group: "ppp-group",
        label: "PPPoE Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pppoe-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("service-name", "Service", 16),
            col!("ac-name", "AC name", 16),
            col!("profile", "Profile", 16),
            col!("add-default-route", "Default", 8),
            col!("use-peer-dns", "Peer DNS", 9),
            col!("status", "Status", 12),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::PPPOE_CLIENT_FORM),
    },
    ResourceSpec {
        id: "pppoe-servers",
        group: "ppp-group",
        label: "PPPoE Servers",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pppoe-server/server",
        },
        columns: &[
            col!("service-name", "Service", 16),
            col!("interface", "Interface", 16),
            col!("default-profile", "Profile", 16),
            col!("authentication", "Auth", 16),
            col!("max-mtu", "Max MTU", 8),
            col!("max-mru", "Max MRU", 8),
            col!("one-session-per-host", "One sess", 9),
            col!("max-sessions", "Max sess", 9),
            col!("keepalive-timeout", "Keepalive", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ppp_write::PPPOE_SERVER_FORM),
    },
    ResourceSpec {
        id: "pppoe-server-ifaces",
        group: "ppp-group",
        label: "PPPoE Sessions",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pppoe-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("user", "User", 18),
            col!("service-name", "Service", 16),
            col!("interface", "Interface", 16),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "pptp-client",
        group: "ppp-group",
        label: "PPTP Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pptp-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("connect-to", "Connect to", 22),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("profile", "Profile", 16),
            col!("add-default-route", "Default", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::PPTP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "pptp-server-ifaces",
        group: "ppp-group",
        label: "PPTP Sessions",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/pptp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("user", "User", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "pptp-server",
        group: "ppp-group",
        label: "PPTP Server",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/pptp-server/server",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("default-profile", "Profile", 16),
            col!("authentication", "Auth", 16),
            col!("keepalive-timeout", "Keepalive", 12),
            col!("max-mtu", "Max MTU", 8),
            col!("max-mru", "Max MRU", 8),
            col!("mrru", "MRRU", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ppp_write::PPTP_SERVER_FORM),
    },
    ResourceSpec {
        id: "l2tp-client",
        group: "ppp-group",
        label: "L2TP Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/l2tp-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("connect-to", "Connect to", 22),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("profile", "Profile", 16),
            col!("use-ipsec", "IPsec", 8),
            col!("ipsec-secret", "IPsec secret", 14),
            col!("add-default-route", "Default", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::L2TP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "l2tp-server-ifaces",
        group: "ppp-group",
        label: "L2TP Sessions",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/l2tp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("user", "User", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "l2tp-server",
        group: "ppp-group",
        label: "L2TP Server",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/l2tp-server/server",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("default-profile", "Profile", 16),
            col!("authentication", "Auth", 16),
            col!("use-ipsec", "IPsec", 8),
            col!("ipsec-secret", "IPsec secret", 14),
            col!("keepalive-timeout", "Keepalive", 12),
            col!("max-mtu", "Max MTU", 8),
            col!("max-mru", "Max MRU", 8),
            col!("allow-fast-path", "Fast path", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ppp_write::L2TP_SERVER_FORM),
    },
    ResourceSpec {
        id: "sstp-client",
        group: "ppp-group",
        label: "SSTP Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/sstp-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("connect-to", "Connect to", 22),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("profile", "Profile", 16),
            col!("certificate", "Certificate", 18),
            col!("verify-server-certificate", "Verify", 8),
            col!("add-default-route", "Default", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::SSTP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "sstp-server-ifaces",
        group: "ppp-group",
        label: "SSTP Sessions",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/sstp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("user", "User", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "sstp-server",
        group: "ppp-group",
        label: "SSTP Server",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/sstp-server/server",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("certificate", "Certificate", 18),
            col!("default-profile", "Profile", 16),
            col!("authentication", "Auth", 16),
            col!("port", "Port", 8),
            col!("verify-client-certificate", "Verify", 8),
            col!("keepalive-timeout", "Keepalive", 12),
            col!("max-mtu", "Max MTU", 8),
            col!("max-mru", "Max MRU", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ppp_write::SSTP_SERVER_FORM),
    },
    ResourceSpec {
        id: "ovpn-client",
        group: "ppp-group",
        label: "OpenVPN Clients",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ovpn-client",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("connect-to", "Connect to", 22),
            col!("port", "Port", 8),
            col!("mode", "Mode", 10),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("profile", "Profile", 16),
            col!("certificate", "Certificate", 18),
            col!("cipher", "Cipher", 14),
            col!("auth", "Auth", 10),
            col!("add-default-route", "Default", 8),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::ppp_write::OVPN_CLIENT_FORM),
    },
    ResourceSpec {
        id: "ovpn-server-ifaces",
        group: "ppp-group",
        label: "OpenVPN Sessions",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ovpn-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("user", "User", 18),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ovpn-server",
        group: "ppp-group",
        label: "OpenVPN Server",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/ovpn-server/server",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("port", "Port", 8),
            col!("mode", "Mode", 10),
            col!("netmask", "Netmask", 8),
            col!("certificate", "Certificate", 18),
            col!("default-profile", "Profile", 16),
            col!("auth", "Auth", 16),
            col!("cipher", "Cipher", 16),
            col!("require-client-certificate", "Client cert", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ppp_write::OVPN_SERVER_FORM),
    },
    ResourceSpec {
        id: "bridges",
        group: "bridge-group",
        label: "Bridge",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("protocol-mode", "Protocol", 12),
            col!("vlan-filtering", "VLAN", 6),
            col!("pvid", "PVID", 7),
            col!("igmp-snooping", "IGMP", 6),
            col!("dhcp-snooping", "DHCP snoop", 11),
            col!("arp", "ARP", 16),
            col!("mac-address", "MAC address", 18),
            col!("mtu", "MTU", 7),
            col!("fast-forward", "Fast fwd", 9),
            col!("frame-types", "Frames", 18),
            col!("ingress-filtering", "Ingress", 8),
            col!("priority", "Priority", 10),
            col!("region-name", "Region", 16),
            col!("running", "Run", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::VIRTUAL_IFACE_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_FORM),
    },
    ResourceSpec {
        id: "bridge-ports",
        group: "bridge-group",
        label: "Ports",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/port",
        },
        columns: &[
            col!("interface", "Interface", 18),
            col!("bridge", "Bridge", 18),
            col!("pvid", "PVID", 7),
            col!("hw", "HW", 4),
            col!("role", "Role", 16),
            col!("edge", "Edge", 12),
            col!("frame-types", "Frames", 18),
            col!("ingress-filtering", "Ingress", 8),
            col!("trusted", "Trusted", 8),
            col!("horizon", "Horizon", 8),
            col!("path-cost", "Cost", 8),
            col!("priority", "Priority", 9),
            col!("bpdu-guard", "BPDU", 6),
            col!("restricted-role", "Root guard", 11),
            col!("learn", "Learn", 8),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_PORT_FORM),
    },
    ResourceSpec {
        id: "bridge-hosts",
        group: "bridge-group",
        label: "Hosts",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/host",
        },
        columns: &[
            col!("mac-address", "MAC address", 18),
            col!("vid", "VID", 6),
            col!("on-interface", "On interface", 18),
            col!("bridge", "Bridge", 16),
            col!("dynamic", "Dyn", 5),
            col!("local", "Local", 6),
            col!("external", "Ext", 5),
            col!("invalid", "Bad", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::HOST_TABLE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "bridge-vlans",
        group: "bridge-group",
        label: "VLANs",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/vlan",
        },
        columns: &[
            col!("bridge", "Bridge", 16),
            col!("vlan-ids", "VLAN IDs", 14),
            col!("tagged", "Tagged", 24),
            col!("untagged", "Untagged", 24),
            col!("current-tagged", "Current tagged", 24),
            col!("current-untagged", "Current untagged", 24),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_VLAN_FORM),
    },
    ResourceSpec {
        id: "bridge-mdb",
        group: "bridge-group",
        label: "MDB",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/mdb",
        },
        columns: &[
            col!("group", "Group", 22),
            col!("vid", "VID", 6),
            col!("on-ports", "On ports", 28),
            col!("bridge", "Bridge", 16),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_MDB_FORM),
    },
    ResourceSpec {
        id: "bridge-msti",
        group: "bridge-group",
        label: "MSTIs",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/msti",
        },
        columns: &[
            col!("bridge", "Bridge", 16),
            col!("identifier", "MSTI", 6),
            col!("vlan-mapping", "VLANs", 28),
            col!("priority", "Priority", 10),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_MSTI_FORM),
    },
    ResourceSpec {
        id: "bridge-filter",
        group: "bridge-group",
        label: "Filter",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/filter",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("mac-protocol", "MAC proto", 12),
            col!("src-mac-address", "Src MAC", 20),
            col!("dst-mac-address", "Dst MAC", 20),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("ip-protocol", "IP proto", 10),
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("src-port", "Src port", 10),
            col!("dst-port", "Dst port", 10),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_FILTER_FORM),
    },
    ResourceSpec {
        id: "bridge-nat",
        group: "bridge-group",
        label: "NAT",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/nat",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 14),
            col!("mac-protocol", "MAC proto", 12),
            col!("src-mac-address", "Src MAC", 20),
            col!("dst-mac-address", "Dst MAC", 20),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("to-src-mac-address", "To src MAC", 20),
            col!("to-dst-mac-address", "To dst MAC", 20),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_NAT_FORM),
    },
    ResourceSpec {
        id: "bridge-settings",
        group: "bridge-group",
        label: "Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/bridge/settings",
        },
        columns: &[
            col!("use-ip-firewall", "IP firewall", 12),
            col!("use-ip-firewall-for-vlan", "VLAN FW", 8),
            col!("use-ip-firewall-for-pppoe", "PPPoE FW", 9),
            col!("allow-fast-path", "Fast path", 10),
            col!("bridge-fast-path-active", "FP active", 10),
            col!("bridge-fast-path-packets", "FP pkts", 12),
            col!("bridge-fast-path-bytes", "FP bytes", 12),
            col!("bridge-fast-forward-packets", "FF pkts", 12),
            col!("bridge-fast-forward-bytes", "FF bytes", 12),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_SETTINGS_FORM),
    },
    ResourceSpec {
        id: "bridge-port-controller",
        group: "bridge-group",
        label: "Port Controller",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/bridge/port-controller",
        },
        columns: &[
            col!("bridge", "Bridge", 16),
            col!("switch", "Switch", 12),
            col!("cascade-ports", "Cascade", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_PORT_CONTROLLER_FORM),
    },
    ResourceSpec {
        id: "bridge-port-controller-device",
        group: "bridge-group",
        label: "Controller Devices",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/port-controller/device",
        },
        columns: &[
            col!("name", "Name", 16),
            col!("pe-mac", "PE MAC", 18),
            col!("descr", "Description", 36),
            col!("control-ports", "Control ports", 28),
            col!("status", "Status", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_PORT_CONTROLLER_DEVICE_FORM),
    },
    ResourceSpec {
        id: "bridge-port-controller-port",
        group: "bridge-group",
        label: "Controller Ports",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/bridge/port-controller/port",
        },
        columns: &[
            col!("name", "Name", 20),
            col!("device", "Device", 16),
            col!("status", "Status", 12),
            col!("port-status", "Port status", 12),
            col!("rate", "Rate", 10),
            col!("pcid", "PCID", 8),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_PORT_CONTROLLER_PORT_FORM),
    },
    ResourceSpec {
        id: "bridge-port-extender",
        group: "bridge-group",
        label: "Port Extender",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/bridge/port-extender",
        },
        columns: &[
            col!("switch", "Switch", 12),
            col!("control-ports", "Control", 28),
            col!("excluded-ports", "Excluded", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::bridge_write::BRIDGE_PORT_EXTENDER_FORM),
    },
    ResourceSpec {
        id: "switch",
        group: "switch-group",
        label: "Switch",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch",
        },
        columns: &[
            col!("name", "Name", 12),
            col!("type", "Type", 18),
            col!("mirror-source", "Mirror src", 16),
            col!("mirror-target", "Mirror dst", 16),
            col!("mirror-egress-target", "Egress dst", 16),
            col!("cpu-flow-control", "CPU FC", 8),
            col!("l3-hw-offloading", "L3HW", 6),
            col!("switch-all-ports", "All ports", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::HARDWARE_EDIT_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_FORM),
    },
    ResourceSpec {
        id: "switch-port",
        group: "switch-group",
        label: "Ports",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch/port",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("switch", "Switch", 12),
            col!("vlan-mode", "VLAN mode", 10),
            col!("vlan-header", "VLAN hdr", 14),
            col!("default-vlan-id", "Default VID", 12),
            col!("ingress-rate", "Ingress", 12),
            col!("egress-rate", "Egress", 12),
            col!("storm-rate", "Storm", 8),
            col!("l3-hw-offloading", "L3HW", 6),
            col!("mirror-ingress", "Mir in", 7),
            col!("mirror-egress", "Mir out", 8),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::HARDWARE_EDIT_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_PORT_FORM),
    },
    ResourceSpec {
        id: "switch-vlan",
        group: "switch-group",
        label: "VLANs",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch/vlan",
        },
        columns: &[
            col!("switch", "Switch", 12),
            col!("vlan-id", "VLAN ID", 8),
            col!("ports", "Ports", 36),
            col!("independent-learning", "IVL", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_VLAN_FORM),
    },
    ResourceSpec {
        id: "switch-host",
        group: "switch-group",
        label: "Hosts",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch/host",
        },
        columns: &[
            col!("switch", "Switch", 12),
            col!("mac-address", "MAC address", 18),
            col!("ports", "Ports", 24),
            col!("vlan-id", "VLAN ID", 8),
            col!("drop", "Drop", 6),
            col!("mirror", "Mirror", 7),
            col!("copy-to-cpu", "Copy CPU", 9),
            col!("redirect-to-cpu", "Redir CPU", 10),
            col!("dynamic", "Dyn", 5),
            col!("invalid", "Bad", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::HOST_TABLE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "switch-rule",
        group: "switch-group",
        label: "Rules",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch/rule",
        },
        columns: &[
            col!("switch", "Switch", 12),
            col!("ports", "Ports", 24),
            col!("mac-protocol", "MAC proto", 12),
            col!("src-mac-address", "Src MAC", 20),
            col!("dst-mac-address", "Dst MAC", 20),
            col!("protocol", "IP proto", 10),
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("src-port", "Src port", 10),
            col!("dst-port", "Dst port", 10),
            col!("vlan-id", "VLAN ID", 8),
            col!("new-dst-ports", "New dst", 20),
            col!("redirect-to-cpu", "Redir CPU", 10),
            col!("mirror", "Mirror", 7),
            col!("invalid", "Bad", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_RULE_FORM),
    },
    ResourceSpec {
        id: "switch-port-isolation",
        group: "switch-group",
        label: "Port Isolation",
        fetch: FetchKind::List {
            endpoint: "/rest/interface/ethernet/switch/port-isolation",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("switch", "Switch", 12),
            col!("forwarding-override", "Forward to", 36),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::HARDWARE_EDIT_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_PORT_ISOLATION_FORM),
    },
    ResourceSpec {
        id: "switch-l3hw",
        group: "switch-group",
        label: "L3HW Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/interface/ethernet/switch/l3hw-settings",
        },
        columns: &[
            col!("autorestart", "Autorestart", 12),
            col!("fasttrack-hw", "FastTrack HW", 13),
            col!("ipv6-hw", "IPv6 HW", 8),
            col!("icmp-reply-on-error", "ICMP error", 11),
            col!("hw-supports-fasttrack", "FT support", 11),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::switch_write::SWITCH_L3HW_FORM),
    },
    ResourceSpec {
        id: "arp",
        group: "ip-group",
        label: "ARP",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/arp",
        },
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC address", 18),
            col!("interface", "Interface", 16),
            col!("status", "Status", 12),
            col!("dynamic", "Dyn", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::ARP_ACTIONS,
        form: Some(&crate::ip_write::ARP_FORM),
    },
    ResourceSpec {
        id: "addresses",
        group: "ip-group",
        label: "Addresses",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/address",
        },
        columns: &[
            col!("address", "Address", 20),
            col!("network", "Network", 18),
            col!("interface", "Interface", 16),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::ADDRESS_FORM),
    },
    ResourceSpec {
        id: "dhcp-servers",
        group: "ip-group",
        label: "DHCP",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("address-pool", "Pool", 18),
            col!("lease-time", "Lease time", 12),
            col!("status", "Status", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::DHCP_SERVER_FORM),
    },
    ResourceSpec {
        id: "dhcp-networks",
        group: "ip-group",
        label: "Networks",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/network",
        },
        columns: &[
            col!("address", "Network", 20),
            col!("gateway", "Gateway", 18),
            col!("dns-server", "DNS", 24),
            col!("domain", "Domain", 18),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::DHCP_NETWORK_FORM),
    },
    ResourceSpec {
        id: "dhcp-leases",
        group: "ip-group",
        label: "Leases",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/lease",
        },
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC address", 18),
            col!("host-name", "Hostname", 20),
            col!("status", "Status", 10),
            col!("expires-after", "Expires", 12),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::LEASE_ACTIONS,
        form: Some(&crate::ip_write::DHCP_LEASE_FORM),
    },
    ResourceSpec {
        id: "dhcp-relay",
        group: "ip-group",
        label: "DHCP Relay",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-relay",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("dhcp-server", "DHCP server", 24),
            col!("local-address", "Local", 18),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::DHCP_RELAY_FORM),
    },
    ResourceSpec {
        id: "dhcp-options",
        group: "ip-group",
        label: "DHCP Options",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/option",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("code", "Code", 6),
            col!("value", "Value", 28),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::DHCP_OPTION_FORM),
    },
    ResourceSpec {
        id: "dhcp-option-sets",
        group: "ip-group",
        label: "DHCP Option Sets",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/option/sets",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("options", "Options", 36),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::DHCP_OPTION_SET_FORM),
    },
    ResourceSpec {
        id: "firewall-filter",
        group: "ip-group",
        label: "Firewall",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/filter",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 20),
            col!("src-port", "Src port", 10),
            col!("dst-address", "Destination", 20),
            col!("dst-port", "Dst port", 10),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("dynamic", "Dyn", 5),
            col!("invalid", "Bad", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ip_write::FIREWALL_FILTER_FORM),
    },
    ResourceSpec {
        id: "users",
        group: "system-group",
        label: "Users",
        fetch: FetchKind::List {
            endpoint: "/rest/user",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("group", "Group", 14),
            col!("last-logged-in", "Last login", 22),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::system_write::USER_FORM),
    },
    ResourceSpec {
        id: "routerboard",
        group: "system-group",
        label: "RouterBOARD",
        fetch: FetchKind::System {
            endpoint: "/rest/system/routerboard",
        },
        columns: &[
            col!("model", "Model", 18),
            col!("serial-number", "Serial", 18),
            col!("current-firmware", "Current", 12),
            col!("upgrade-firmware", "Upgrade", 12),
        ],
        refresh: Duration::from_secs(60),
        actions: crate::actions::ROUTERBOARD_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ntp",
        group: "system-group",
        label: "NTP Client",
        fetch: FetchKind::System {
            endpoint: "/rest/system/ntp/client",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("mode", "Mode", 12),
            col!("servers", "Servers", 28),
            col!("status", "Status", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::NTP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "ntp-server",
        group: "system-group",
        label: "NTP Server",
        fetch: FetchKind::System {
            endpoint: "/rest/system/ntp/server",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("broadcast", "Broadcast", 10),
            col!("multicast", "Multicast", 10),
            col!("manycast", "Manycast", 10),
            col!("vrf", "VRF", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::NTP_SERVER_FORM),
    },
    ResourceSpec {
        id: "ntp-keys",
        group: "system-group",
        label: "NTP Keys",
        fetch: FetchKind::List {
            endpoint: "/rest/system/ntp/key",
        },
        columns: &[col!("key-id", "Key ID", 10)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::system_write::NTP_KEY_FORM),
    },
    ResourceSpec {
        id: "clock",
        group: "system-group",
        label: "Clock",
        fetch: FetchKind::System {
            endpoint: "/rest/system/clock",
        },
        columns: &[
            col!("time", "Time", 12),
            col!("date", "Date", 14),
            col!("time-zone-name", "Time zone", 22),
            col!("gmt-offset", "Offset", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::CLOCK_FORM),
    },
    ResourceSpec {
        id: "neighbors",
        group: "ip-group",
        label: "Neighbors",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/neighbor",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("address", "Address", 18),
            col!("mac-address", "MAC address", 18),
            col!("identity", "Identity", 20),
            col!("platform", "Platform", 14),
            col!("version", "Version", 14),
            col!("interface-name", "If name", 16),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::NEIGHBOR_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "dhcp-clients",
        group: "ip-group",
        label: "DHCP Client",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-client",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("status", "Status", 12),
            col!("address", "Address", 20),
            col!("gateway", "Gateway", 18),
            col!("dhcp-server", "Server", 18),
            col!("add-default-route", "Default", 8),
            col!("use-peer-dns", "Peer DNS", 9),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::DHCP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "dns",
        group: "ip-group",
        label: "DNS",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/dns",
        },
        columns: &[
            col!("servers", "Servers", 28),
            col!("allow-remote-requests", "Remote", 8),
            col!("cache-size", "Cache", 10),
            col!("cache-max-ttl", "Max TTL", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::DNS_FORM),
    },
    ResourceSpec {
        id: "dns-static",
        group: "ip-group",
        label: "Static DNS",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dns/static",
        },
        columns: &[
            col!("name", "Name", 24),
            col!("address", "Address", 18),
            col!("type", "Type", 8),
            col!("ttl", "TTL", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::DNS_STATIC_FORM),
    },
    ResourceSpec {
        id: "routes",
        group: "ip-group",
        label: "Routes",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/route",
        },
        columns: &[
            col!("dst-address", "Dst", 20),
            col!("gateway", "Gateway", 18),
            col!("distance", "Dist", 6),
            col!("routing-table", "Table", 12),
            col!("active", "Act", 5),
            col!("static", "Static", 7),
            col!("dynamic", "Dyn", 5),
            col!("unreachable", "Unreach", 8),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::ROUTE_FORM),
    },
    ResourceSpec {
        id: "pools",
        group: "ip-group",
        label: "Pool",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/pool",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("ranges", "Ranges", 36),
            col!("next-pool", "Next", 18),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::POOL_FORM),
    },
    ResourceSpec {
        id: "ip-services",
        group: "ip-group",
        label: "Services",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/service",
        },
        columns: &[
            col!("name", "Name", 14),
            col!("port", "Port", 6),
            col!("address", "Address", 20),
            col!("certificate", "Certificate", 18),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::TOGGLE_EDIT_ACTIONS,
        form: Some(&crate::ip_write::SERVICE_FORM),
    },
    ResourceSpec {
        id: "ip-settings",
        group: "ip-group",
        label: "Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/settings",
        },
        columns: &[
            col!("ip-forward", "Forward", 8),
            col!("rp-filter", "RP filter", 12),
            col!("tcp-syncookies", "Syncookies", 11),
            col!("accept-redirects", "Accept redir", 13),
            col!("send-redirects", "Send redir", 11),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::IP_SETTINGS_FORM),
    },
    ResourceSpec {
        id: "firewall-nat",
        group: "ip-group",
        label: "NAT",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/nat",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 14),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("dst-port", "Dst port", 10),
            col!("to-addresses", "To addr", 20),
            col!("to-ports", "To ports", 10),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("dynamic", "Dyn", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ip_write::FIREWALL_NAT_FORM),
    },
    ResourceSpec {
        id: "firewall-mangle",
        group: "ip-group",
        label: "Mangle",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/mangle",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 14),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("new-routing-mark", "Mark", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ip_write::FIREWALL_MANGLE_FORM),
    },
    ResourceSpec {
        id: "firewall-raw",
        group: "ip-group",
        label: "Raw",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/raw",
        },
        columns: &[
            col!("chain", "Chain", 12),
            col!("action", "Action", 14),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("dynamic", "Dyn", 5),
            col!("invalid", "Bad", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ip_write::FIREWALL_RAW_FORM),
    },
    ResourceSpec {
        id: "firewall-connections",
        group: "ip-group",
        label: "Connections",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/connection",
        },
        columns: &[
            col!("src-address", "Source", 22),
            col!("dst-address", "Destination", 22),
            col!("protocol", "Protocol", 9),
            col!("src-port", "Src port", 10),
            col!("dst-port", "Dst port", 10),
            col!("tcp-state", "TCP state", 12),
            col!("timeout", "Timeout", 10),
            col!("orig-rate", "Orig rate", 12),
            col!("repl-rate", "Repl rate", 12),
            col!("connection-mark", "Mark", 16),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "address-list",
        group: "ip-group",
        label: "Address List",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/address-list",
        },
        columns: &[
            col!("list", "List", 16),
            col!("address", "Address", 20),
            col!("timeout", "Timeout", 12),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::ADDRESS_LIST_FORM),
    },
    ResourceSpec {
        id: "firewall-layer7",
        group: "ip-group",
        label: "Layer7 Protocol",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/layer7-protocol",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("regexp", "Regexp", 36),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::LAYER7_FORM),
    },
    ResourceSpec {
        id: "firewall-service-port",
        group: "ip-group",
        label: "Service Port",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/firewall/service-port",
        },
        columns: &[
            col!("name", "Name", 14),
            col!("ports", "Ports", 20),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::TOGGLE_EDIT_ACTIONS,
        form: Some(&crate::ip_write::SERVICE_PORT_FORM),
    },
    ResourceSpec {
        id: "ipsec-peers",
        group: "ip-group",
        label: "Peers",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/peer",
        },
        columns: &[
            col!("name", "Name", 16),
            col!("address", "Address", 20),
            col!("profile", "Profile", 14),
            col!("exchange-mode", "Exchange", 12),
            col!("passive", "Passive", 8),
            col!("send-initial-contact", "Init contact", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_PEER_FORM),
    },
    ResourceSpec {
        id: "ipsec-identities",
        group: "ip-group",
        label: "Identities",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/identity",
        },
        columns: &[
            col!("peer", "Peer", 16),
            col!("auth-method", "Auth", 16),
            col!("my-id", "My ID", 18),
            col!("remote-id", "Remote ID", 18),
            col!("generate-policy", "Gen policy", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_IDENTITY_FORM),
    },
    ResourceSpec {
        id: "ipsec-policies",
        group: "ip-group",
        label: "Policies",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/policy",
        },
        columns: &[
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("src-port", "Src port", 10),
            col!("dst-port", "Dst port", 10),
            col!("protocol", "Protocol", 9),
            col!("action", "Action", 10),
            col!("level", "Level", 10),
            col!("ipsec-protocols", "Protocols", 12),
            col!("proposal", "Proposal", 14),
            col!("peer", "Peer", 16),
            col!("tunnel", "Tunnel", 7),
            col!("sa-src-address", "SA src", 18),
            col!("sa-dst-address", "SA dst", 18),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_POLICY_FORM),
    },
    ResourceSpec {
        id: "ipsec-proposals",
        group: "ip-group",
        label: "Proposals",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/proposal",
        },
        columns: &[
            col!("name", "Name", 16),
            col!("auth-algorithms", "Auth", 18),
            col!("enc-algorithms", "Enc", 22),
            col!("pfs-group", "PFS", 10),
            col!("lifetime", "Lifetime", 12),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_PROPOSAL_FORM),
    },
    ResourceSpec {
        id: "ipsec-profiles",
        group: "ip-group",
        label: "Profiles",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/profile",
        },
        columns: &[
            col!("name", "Name", 16),
            col!("hash-algorithm", "Hash", 12),
            col!("enc-algorithm", "Enc", 16),
            col!("dh-group", "DH", 12),
            col!("proposal-check", "Check", 12),
            col!("lifetime", "Lifetime", 12),
            col!("nat-traversal", "NAT-T", 6),
            col!("dpd-interval", "DPD", 10),
            col!("dpd-maximum-failures", "DPD max", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_PROFILE_FORM),
    },
    ResourceSpec {
        id: "ipsec-installed-sa",
        group: "ip-group",
        label: "Installed SAs",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/installed-sa",
        },
        columns: &[
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("spi", "SPI", 12),
            col!("auth-algorithm", "Auth", 12),
            col!("enc-algorithm", "Enc", 14),
            col!("state", "State", 10),
            col!("current-bytes", "Bytes", 12),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::IPSEC_SA_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ipsec-settings",
        group: "ip-group",
        label: "IPsec Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/ipsec/settings",
        },
        columns: &[
            col!("accounting", "Accounting", 11),
            col!("interim-update", "Interim", 12),
            col!("xauth-use-radius", "XAuth RADIUS", 13),
            col!("uniq-id-accounting", "Uniq-id acct", 13),
            col!("identities-matching", "ID match", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_SETTINGS_FORM),
    },
    ResourceSpec {
        id: "ipsec-mode-config",
        group: "ip-group",
        label: "Mode Config",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/mode-config",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("address-pool", "Pool", 16),
            col!("address-prefix-length", "Prefix", 8),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_MODE_CONFIG_FORM),
    },
    ResourceSpec {
        id: "ipsec-key",
        group: "ip-group",
        label: "IPsec Keys",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/ipsec/key",
        },
        columns: &[col!("name", "Name", 18), col!("key-size", "Size", 8)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ipsec_write::IPSEC_KEY_FORM),
    },
    ResourceSpec {
        id: "cloud",
        group: "ip-group",
        label: "Cloud",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/cloud",
        },
        columns: &[
            col!("ddns-enabled", "DDNS", 6),
            col!("dns-name", "DNS name", 28),
            col!("public-address", "Public", 18),
            col!("status", "Status", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::CLOUD_FORM),
    },
    ResourceSpec {
        id: "kid-control",
        group: "ip-group",
        label: "Kid Control",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/kid-control",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("mon-fri", "Mon-Fri", 16),
            col!("rate-limit", "Rate", 12),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::KID_CONTROL_FORM),
    },
    ResourceSpec {
        id: "kid-control-devices",
        group: "ip-group",
        label: "Kid Devices",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/kid-control/device",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("mac-address", "MAC", 18),
            col!("user", "User", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::KID_CONTROL_DEVICE_FORM),
    },
    ResourceSpec {
        id: "socks",
        group: "ip-group",
        label: "SOCKS",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/socks",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("port", "Port", 6),
            col!("connection-idle-timeout", "Idle", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::SOCKS_FORM),
    },
    ResourceSpec {
        id: "smb",
        group: "ip-group",
        label: "SMB",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/smb",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("domain", "Domain", 16),
            col!("allow-guests", "Guests", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::SMB_FORM),
    },
    ResourceSpec {
        id: "smb-shares",
        group: "ip-group",
        label: "SMB Shares",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/smb/shares",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("directory", "Directory", 24),
            col!("valid-users", "Valid Users", 16),
            col!("invalid-users", "Invalid Users", 16),
            col!("read-only", "Read Only", 10),
            col!("require-encryption", "Require Encryption", 18),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::SMB_SHARE_FORM),
    },
    ResourceSpec {
        id: "smb-users",
        group: "ip-group",
        label: "SMB Users",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/smb/users",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("password", "Password", 10),
            col!("read-only", "Read Only", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::SMB_USER_FORM),
    },
    ResourceSpec {
        id: "upnp",
        group: "ip-group",
        label: "UPnP",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/upnp",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("allow-disable-external-interface", "Allow WAN", 10),
            col!("show-dummy-rule", "Dummy", 7),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::UPNP_FORM),
    },
    ResourceSpec {
        id: "upnp-interfaces",
        group: "ip-group",
        label: "UPnP Interfaces",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/upnp/interfaces",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("type", "Type", 10),
            col!("forced-external-ip", "External IP", 18),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::UPNP_INTERFACE_FORM),
    },
    ResourceSpec {
        id: "dns-cache",
        group: "ip-group",
        label: "DNS Cache",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dns/cache",
        },
        columns: &[
            col!("name", "Name", 28),
            col!("type", "Type", 8),
            col!("data", "Data", 28),
            col!("ttl", "TTL", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::DNS_CACHE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "dhcp-alerts",
        group: "ip-group",
        label: "DHCP Alert",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/dhcp-server/alert",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("valid-server", "Valid server", 20),
            col!("alert-timeout", "Timeout", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::DHCP_ALERT_FORM),
    },
    ResourceSpec {
        id: "connection-tracking",
        group: "ip-group",
        label: "Conntrack",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/firewall/connection/tracking",
        },
        columns: &[
            col!("enabled", "Enabled", 10),
            col!("tcp-established-timeout", "TCP est.", 12),
            col!("udp-timeout", "UDP", 10),
            col!("total-entries", "Entries", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::CONNECTION_TRACKING_FORM),
    },
    ResourceSpec {
        id: "neighbor-discovery",
        group: "ip-group",
        label: "Discovery Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/neighbor/discovery-settings",
        },
        columns: &[
            col!("discover-interface-list", "Discover", 18),
            col!("protocol", "Protocol", 12),
            col!("mode", "Mode", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::NEIGHBOR_DISCOVERY_FORM),
    },
    ResourceSpec {
        id: "ip-ssh",
        group: "ip-group",
        label: "SSH",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/ssh",
        },
        columns: &[
            col!("strong-crypto", "Strong", 8),
            col!("host-key-size", "Key size", 10),
            col!("always-allow-password-login", "Password", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::IP_SSH_FORM),
    },
    ResourceSpec {
        id: "traffic-flow",
        group: "ip-group",
        label: "Traffic Flow",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/traffic-flow",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("interfaces", "Interfaces", 20),
            col!("cache-entries", "Cache", 8),
            col!("packet-sampling", "Sampling", 9),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::TRAFFIC_FLOW_FORM),
    },
    ResourceSpec {
        id: "traffic-flow-targets",
        group: "ip-group",
        label: "Traffic Flow Targets",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/traffic-flow/target",
        },
        columns: &[
            col!("src-address", "Src. Address", 16),
            col!("dst-address", "Dst. Address", 16),
            col!("port", "Port", 6),
            col!("version", "Version", 8),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::TRAFFIC_FLOW_TARGET_FORM),
    },
    ResourceSpec {
        id: "traffic-flow-ipfix",
        group: "ip-group",
        label: "Traffic Flow IPFIX",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/traffic-flow/ipfix",
        },
        columns: &[
            col!("bytes", "Bytes", 7),
            col!("src-address", "Src", 6),
            col!("dst-address", "Dst", 6),
            col!("protocol", "Proto", 6),
            col!("nat-events", "NAT", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::TRAFFIC_FLOW_IPFIX_FORM),
    },
    ResourceSpec {
        id: "igmp-proxy",
        group: "ip-group",
        label: "IGMP Proxy",
        fetch: FetchKind::System {
            endpoint: "/rest/routing/igmp-proxy",
        },
        columns: &[
            col!("query-interval", "Query", 10),
            col!("query-response-interval", "Response", 10),
            col!("quick-leave", "Quick leave", 11),
            col!("robustness", "Robust", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ip_write::IGMP_PROXY_FORM),
    },
    ResourceSpec {
        id: "igmp-proxy-interfaces",
        group: "ip-group",
        label: "IGMP Proxy Interfaces",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/igmp-proxy/interface",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("upstream", "Up", 5),
            col!("threshold", "TTL", 6),
            col!("querier", "Querier", 8),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ip_write::IGMP_PROXY_INTERFACE_FORM),
    },
    ResourceSpec {
        id: "igmp-proxy-mfc",
        group: "ip-group",
        label: "IGMP Proxy MFC",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/igmp-proxy/mfc",
        },
        columns: &[
            col!("group", "Group", 16),
            col!("source", "Source", 16),
            col!("upstream-interface", "Upstream", 14),
            col!("downstream-interfaces", "Downstream", 20),
            col!("packets", "Packets", 10),
            col!("bytes", "Bytes", 12),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ip_write::IGMP_PROXY_MFC_FORM),
    },
    ResourceSpec {
        id: "proxy",
        group: "ip-group",
        label: "Proxy",
        fetch: FetchKind::System {
            endpoint: "/rest/ip/proxy",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("port", "Port", 6),
            col!("parent-proxy", "Parent", 18),
            col!("max-cache-size", "Cache", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::hotspot_write::PROXY_FORM),
    },
    ResourceSpec {
        id: "proxy-access",
        group: "ip-group",
        label: "Proxy Access",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/proxy/access",
        },
        columns: &[
            col!("src-address", "Source", 18),
            col!("dst-host", "Host", 22),
            col!("action", "Action", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::hotspot_write::PROXY_ACCESS_FORM),
    },
    ResourceSpec {
        id: "proxy-cache",
        group: "ip-group",
        label: "Proxy Cache",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/proxy/cache",
        },
        columns: &[
            col!("dst-host", "Host", 22),
            col!("method", "Method", 10),
            col!("action", "Action", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::PROXY_CACHE_FORM),
    },
    ResourceSpec {
        id: "proxy-direct",
        group: "ip-group",
        label: "Proxy Direct",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/proxy/direct",
        },
        columns: &[
            col!("dst-host", "Host", 22),
            col!("dst-address", "Address", 18),
            col!("action", "Action", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::PROXY_DIRECT_FORM),
    },
    ResourceSpec {
        id: "hotspot",
        group: "ip-group",
        label: "Hotspot",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("profile", "Profile", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_FORM),
    },
    ResourceSpec {
        id: "hotspot-profiles",
        group: "ip-group",
        label: "Hotspot Profiles",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/profile",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("hotspot-address", "Address", 18),
            col!("dns-name", "DNS", 20),
            col!("html-directory", "HTML", 16),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_PROFILE_FORM),
    },
    ResourceSpec {
        id: "hotspot-users",
        group: "ip-group",
        label: "Hotspot Users",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/user",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("profile", "Profile", 16),
            col!("server", "Server", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_USER_FORM),
    },
    ResourceSpec {
        id: "hotspot-cookies",
        group: "ip-group",
        label: "Hotspot Cookies",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/cookie",
        },
        columns: &[
            col!("user", "User", 16),
            col!("mac-address", "MAC", 18),
            col!("expires-in", "Expires", 12),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "hotspot-hosts",
        group: "ip-group",
        label: "Hotspot Hosts",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/host",
        },
        columns: &[
            col!("mac-address", "MAC", 18),
            col!("address", "Address", 16),
            col!("server", "Server", 16),
            col!("authorized", "Auth", 6),
            col!("bypassed", "Bypass", 7),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::HOTSPOT_HOST_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_HOST_FORM),
    },
    ResourceSpec {
        id: "hotspot-ip-bindings",
        group: "ip-group",
        label: "IP Bindings",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/ip-binding",
        },
        columns: &[
            col!("mac-address", "MAC", 18),
            col!("address", "Address", 16),
            col!("to-address", "To", 16),
            col!("type", "Type", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_IP_BINDING_FORM),
    },
    ResourceSpec {
        id: "hotspot-walled-garden",
        group: "ip-group",
        label: "Walled Garden",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/walled-garden",
        },
        columns: &[
            col!("dst-host", "Host", 24),
            col!("dst-port", "Port", 8),
            col!("action", "Action", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_WALLED_GARDEN_FORM),
    },
    ResourceSpec {
        id: "hotspot-walled-garden-ip",
        group: "ip-group",
        label: "Walled Garden IP",
        fetch: FetchKind::List {
            endpoint: "/rest/ip/hotspot/walled-garden-ip",
        },
        columns: &[
            col!("dst-address", "Address", 20),
            col!("action", "Action", 10),
            col!("server", "Server", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::hotspot_write::HOTSPOT_WALLED_GARDEN_IP_FORM),
    },
    ResourceSpec {
        id: "user-groups",
        group: "system-group",
        label: "User Groups",
        fetch: FetchKind::List {
            endpoint: "/rest/user/group",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("policy", "Policy", 36),
            col!("skin", "Skin", 12),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::system_write::USER_GROUP_FORM),
    },
    ResourceSpec {
        id: "identity",
        group: "system-group",
        label: "Identity",
        fetch: FetchKind::System {
            endpoint: "/rest/system/identity",
        },
        columns: &[col!("name", "Name", 28)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::IDENTITY_FORM),
    },
    ResourceSpec {
        id: "resources",
        group: "system-group",
        label: "Resources",
        fetch: FetchKind::System {
            endpoint: "/rest/system/resource",
        },
        columns: &[
            col!("uptime", "Uptime", 12),
            col!("version", "Version", 16),
            col!("build-time", "Build", 20),
            col!("cpu-load", "CPU", 6),
            col!("free-memory", "Free mem", 12),
            col!("total-memory", "Total mem", 12),
            col!("cpu-count", "CPUs", 6),
            col!("board-name", "Board", 18),
            col!("architecture-name", "Arch", 10),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::RESOURCE_LIFECYCLE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "health",
        group: "system-group",
        label: "Health",
        fetch: FetchKind::List {
            endpoint: "/rest/system/health",
        },
        columns: &[
            col!("name", "Name", 20),
            col!("value", "Value", 12),
            col!("type", "Type", 12),
        ],
        refresh: Duration::from_secs(10),
        actions: &[],
        form: None,
    },
    ResourceSpec {
        id: "packages",
        group: "system-group",
        label: "Packages",
        fetch: FetchKind::List {
            endpoint: "/rest/system/package",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("version", "Version", 14),
            col!("build-time", "Build", 20),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::PACKAGE_ACTIONS,
        form: Some(&crate::system_write::PACKAGE_FORM),
    },
    ResourceSpec {
        id: "package-update",
        group: "system-group",
        label: "Package Update",
        fetch: FetchKind::System {
            endpoint: "/rest/system/package/update",
        },
        columns: &[
            col!("channel", "Channel", 12),
            col!("installed-version", "Installed", 14),
            col!("latest-version", "Latest", 14),
            col!("status", "Status", 16),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::PACKAGE_UPDATE_ACTIONS,
        form: Some(&crate::system_write::PACKAGE_UPDATE_FORM),
    },
    ResourceSpec {
        id: "reset-configuration",
        group: "system-group",
        label: "Reset Configuration",
        fetch: FetchKind::Local,
        columns: &[
            col!("keep-users", "Keep users", 12),
            col!("no-defaults", "No defaults", 12),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::RESET_CONFIG_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ssh-keys",
        group: "system-group",
        label: "SSH Keys",
        fetch: FetchKind::List {
            endpoint: "/rest/user/ssh-keys",
        },
        columns: &[col!("user", "User", 16), col!("key-owner", "Owner", 20)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::system_write::SSH_KEY_FORM),
    },
    ResourceSpec {
        id: "history",
        group: "system-group",
        label: "History",
        fetch: FetchKind::List {
            endpoint: "/rest/system/history",
        },
        columns: &[
            col!("floating-undo", "F", 3),
            col!("time", "Time", 20),
            col!("action", "Action", 12),
            col!("by", "By", 14),
            col!("policy", "Policy", 20),
        ],
        refresh: Duration::from_secs(10),
        actions: &[],
        form: None,
    },
    ResourceSpec {
        id: "scheduler",
        group: "system-group",
        label: "Scheduler",
        fetch: FetchKind::List {
            endpoint: "/rest/system/scheduler",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("start-date", "Start date", 12),
            col!("start-time", "Start time", 12),
            col!("interval", "Interval", 12),
            col!("on-event", "On event", 24),
            col!("next-run", "Next", 16),
            col!("run-count", "Runs", 8),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SCHEDULER_ACTIONS,
        form: Some(&crate::system_write::SCHEDULER_FORM),
    },
    ResourceSpec {
        id: "scripts",
        group: "system-group",
        label: "Scripts",
        fetch: FetchKind::List {
            endpoint: "/rest/system/script",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("owner", "Owner", 14),
            col!("policy", "Policy", 28),
            col!("dont-require-permissions", "No perms", 9),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SCRIPT_ACTIONS,
        form: Some(&crate::system_write::SCRIPT_FORM),
    },
    ResourceSpec {
        id: "logging",
        group: "system-group",
        label: "Logging",
        fetch: FetchKind::List {
            endpoint: "/rest/system/logging",
        },
        columns: &[
            col!("topics", "Topics", 24),
            col!("action", "Action", 12),
            col!("prefix", "Prefix", 14),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::system_write::LOGGING_FORM),
    },
    ResourceSpec {
        id: "logging-actions",
        group: "system-group",
        label: "Logging Actions",
        fetch: FetchKind::List {
            endpoint: "/rest/system/logging/action",
        },
        columns: &[
            col!("name", "Name", 14),
            col!("target", "Type", 10),
            col!("remote", "Remote", 16),
            col!("remote-port", "Port", 6),
            col!("remote-protocol", "Proto", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::system_write::LOGGING_ACTION_FORM),
    },
    ResourceSpec {
        id: "snmp",
        group: "system-group",
        label: "SNMP",
        fetch: FetchKind::System {
            endpoint: "/rest/snmp",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("contact", "Contact", 20),
            col!("location", "Location", 20),
            col!("engine-id", "Engine", 18),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::SNMP_FORM),
    },
    ResourceSpec {
        id: "snmp-communities",
        group: "system-group",
        label: "SNMP Communities",
        fetch: FetchKind::List {
            endpoint: "/rest/snmp/community",
        },
        columns: &[
            col!("name", "Name", 16),
            col!("addresses", "Addresses", 24),
            col!("security", "Security", 12),
            col!("authentication-password", "Auth", 10),
            col!("encryption-password", "Encrypt", 10),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::system_write::SNMP_COMMUNITY_FORM),
    },
    ResourceSpec {
        id: "certificates",
        group: "system-group",
        label: "Certificates",
        fetch: FetchKind::List {
            endpoint: "/rest/certificate",
        },
        columns: &[
            col!("name", "Name", 20),
            col!("common-name", "CN", 24),
            col!("key-usage", "Usage", 24),
            col!("trusted", "Trust", 6),
            col!("invalid-after", "Expires", 20),
            col!("fingerprint", "Fingerprint", 20),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::CERTIFICATE_ACTIONS,
        form: Some(&crate::system_write::CERTIFICATE_FORM),
    },
    ResourceSpec {
        id: "watchdog",
        group: "system-group",
        label: "Watchdog",
        fetch: FetchKind::System {
            endpoint: "/rest/system/watchdog",
        },
        columns: &[
            col!("watch-address", "Watch", 18),
            col!("watch-interval", "Interval", 10),
            col!("automatic-supout", "Supout", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::WATCHDOG_FORM),
    },
    ResourceSpec {
        id: "note",
        group: "system-group",
        label: "Note",
        fetch: FetchKind::System {
            endpoint: "/rest/system/note",
        },
        columns: &[col!("show-at-login", "Login", 8), col!("note", "Note", 48)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::system_write::NOTE_FORM),
    },
    ResourceSpec {
        id: "ipv6-addresses",
        group: "ipv6-group",
        label: "Addresses",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/address",
        },
        columns: &[
            col!("address", "Address", 28),
            col!("interface", "Interface", 16),
            col!("advertise", "Adv", 5),
            col!("eui-64", "EUI-64", 7),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_ADDRESS_FORM),
    },
    ResourceSpec {
        id: "ipv6-neighbors",
        group: "ipv6-group",
        label: "Neighbors",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/neighbor",
        },
        columns: &[
            col!("address", "Address", 28),
            col!("interface", "Interface", 16),
            col!("mac-address", "MAC address", 18),
            col!("status", "Status", 12),
            col!("origin", "Origin", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_NEIGHBOR_FORM),
    },
    ResourceSpec {
        id: "ipv6-nd",
        group: "ipv6-group",
        label: "ND",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/nd",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("ra-interval", "RA interval", 12),
            col!("advertise-mac-address", "Adv MAC", 8),
            col!("advertise-dns", "Adv DNS", 8),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_ND_FORM),
    },
    ResourceSpec {
        id: "ipv6-nd-prefix",
        group: "ipv6-group",
        label: "ND Prefix",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/nd/prefix",
        },
        columns: &[
            col!("prefix", "Prefix", 28),
            col!("interface", "Interface", 16),
            col!("advertise", "Adv", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_ND_PREFIX_FORM),
    },
    ResourceSpec {
        id: "ipv6-routes",
        group: "ipv6-group",
        label: "Routes",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/route",
        },
        columns: &[
            col!("dst-address", "Dst", 28),
            col!("gateway", "Gateway", 28),
            col!("distance", "Dist", 6),
            col!("routing-table", "Table", 12),
            col!("active", "Act", 5),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_ROUTE_FORM),
    },
    ResourceSpec {
        id: "ipv6-pool",
        group: "ipv6-group",
        label: "Pool",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/pool",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("prefix", "Prefix", 28),
            col!("prefix-length", "Len", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_POOL_FORM),
    },
    ResourceSpec {
        id: "ipv6-dhcp-client",
        group: "ipv6-group",
        label: "DHCP Client",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/dhcp-client",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("status", "Status", 12),
            col!("prefix", "Prefix", 28),
            col!("pool-name", "Pool", 16),
            col!("expires-after", "Expires", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_DHCP_CLIENT_FORM),
    },
    ResourceSpec {
        id: "ipv6-dhcp-server",
        group: "ipv6-group",
        label: "DHCP Server",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/dhcp-server",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("address-pool", "Pool", 18),
            col!("lease-time", "Lease time", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_DHCP_SERVER_FORM),
    },
    ResourceSpec {
        id: "ipv6-settings",
        group: "ipv6-group",
        label: "Settings",
        fetch: FetchKind::System {
            endpoint: "/rest/ipv6/settings",
        },
        columns: &[
            col!("forward", "Forward", 8),
            col!("accept-redirects", "Redirects", 10),
            col!("max-neighbor-entries", "ND max", 8),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_SETTINGS_FORM),
    },
    ResourceSpec {
        id: "ipv6-firewall-filter",
        group: "ipv6-group",
        label: "Firewall",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/filter",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("src-address", "Source", 24),
            col!("dst-address", "Destination", 24),
            col!("protocol", "Protocol", 9),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("packets", "Packets", 12),
            col!("bytes", "Bytes", 14),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_FIREWALL_FILTER_FORM),
    },
    ResourceSpec {
        id: "ipv6-firewall-nat",
        group: "ipv6-group",
        label: "NAT",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/nat",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 14),
            col!("protocol", "Protocol", 9),
            col!("src-address", "Source", 24),
            col!("dst-address", "Destination", 24),
            col!("to-addresses", "To addr", 24),
            col!("in-interface", "In interface", 16),
            col!("out-interface", "Out interface", 16),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_FIREWALL_NAT_FORM),
    },
    ResourceSpec {
        id: "ipv6-address-list",
        group: "ipv6-group",
        label: "Address List",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/address-list",
        },
        columns: &[
            col!("list", "List", 16),
            col!("address", "Address", 28),
            col!("timeout", "Timeout", 12),
            col!("dynamic", "Dyn", 5),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_ADDRESS_LIST_FORM),
    },
    ResourceSpec {
        id: "ipv6-dhcp-relay",
        group: "ipv6-group",
        label: "DHCP Relay",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/dhcp-relay",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("interface", "Interface", 16),
            col!("dhcp-server", "Server", 24),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_DHCP_RELAY_FORM),
    },
    ResourceSpec {
        id: "ipv6-dhcp-bindings",
        group: "ipv6-group",
        label: "DHCP Bindings",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/dhcp-server/binding",
        },
        columns: &[
            col!("address", "Address", 28),
            col!("duid", "DUID", 20),
            col!("server", "Server", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::LEASE_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_DHCP_BINDING_FORM),
    },
    ResourceSpec {
        id: "ipv6-firewall-mangle",
        group: "ipv6-group",
        label: "Mangle",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/mangle",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("src-address", "Source", 24),
            col!("dst-address", "Destination", 24),
            col!("packets", "Packets", 12),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_FIREWALL_MANGLE_FORM),
    },
    ResourceSpec {
        id: "ipv6-firewall-raw",
        group: "ipv6-group",
        label: "Raw",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/raw",
        },
        columns: &[
            col!("chain", "Chain", 10),
            col!("action", "Action", 12),
            col!("src-address", "Source", 24),
            col!("dst-address", "Destination", 24),
            col!("packets", "Packets", 12),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::ipv6_write::IPV6_FIREWALL_RAW_FORM),
    },
    ResourceSpec {
        id: "ipv6-firewall-connections",
        group: "ipv6-group",
        label: "Connections",
        fetch: FetchKind::List {
            endpoint: "/rest/ipv6/firewall/connection",
        },
        columns: &[
            col!("src-address", "Source", 28),
            col!("dst-address", "Destination", 28),
            col!("protocol", "Protocol", 9),
            col!("src-port", "Src port", 10),
            col!("dst-port", "Dst port", 10),
            col!("tcp-state", "TCP state", 12),
            col!("timeout", "Timeout", 10),
            col!("orig-rate", "Orig rate", 12),
            col!("repl-rate", "Repl rate", 12),
            col!("connection-mark", "Mark", 16),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::DISCONNECT_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "routing-tables",
        group: "routing-group",
        label: "Tables",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/table",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("fib", "FIB", 5),
            col!("dynamic", "Dyn", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::routing_write::ROUTING_TABLE_FORM),
    },
    ResourceSpec {
        id: "routing-rules",
        group: "routing-group",
        label: "Rules",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/rule",
        },
        columns: &[
            col!("src-address", "Source", 20),
            col!("dst-address", "Destination", 20),
            col!("routing-mark", "Mark", 14),
            col!("action", "Action", 12),
            col!("table", "Table", 14),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::ROUTING_RULE_FORM),
    },
    ResourceSpec {
        id: "ospf-instances",
        group: "routing-group",
        label: "OSPF",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/instance",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("version", "Ver", 5),
            col!("router-id", "Router ID", 16),
            col!("originate-default", "Default", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::OSPF_INSTANCE_FORM),
    },
    ResourceSpec {
        id: "ospf-areas",
        group: "routing-group",
        label: "OSPF Areas",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/area",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("instance", "Instance", 16),
            col!("area-id", "Area ID", 16),
            col!("type", "Type", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::OSPF_AREA_FORM),
    },
    ResourceSpec {
        id: "ospf-interface-templates",
        group: "routing-group",
        label: "OSPF Interface Templates",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/interface-template",
        },
        columns: &[
            col!("instance", "Instance", 16),
            col!("area", "Area", 16),
            col!("interfaces", "Interfaces", 24),
            col!("type", "Type", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::OSPF_INTERFACE_TEMPLATE_FORM),
    },
    ResourceSpec {
        id: "ospf-interfaces",
        group: "routing-group",
        label: "OSPF Interface",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/interface",
        },
        columns: &[
            col!("address", "Address", 22),
            col!("area", "Area", 12),
            col!("state", "State", 10),
            col!("network-type", "Type", 12),
            col!("cost", "Cost", 6),
            col!("dr", "DR", 16),
            col!("bdr", "BDR", 16),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::routing_write::OSPF_INTERFACE_FORM),
    },
    ResourceSpec {
        id: "bgp-connections",
        group: "routing-group",
        label: "BGP",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/bgp/connection",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("remote.address", "Remote", 18),
            col!("remote.as", "Remote AS", 10),
            col!("local.role", "Role", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::BGP_CONNECTION_FORM),
    },
    ResourceSpec {
        id: "bgp-templates",
        group: "routing-group",
        label: "BGP Templates",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/bgp/template",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("as", "AS", 10),
            col!("router-id", "Router ID", 16),
            col!("address-families", "Families", 16),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::BGP_TEMPLATE_FORM),
    },
    ResourceSpec {
        id: "rip-instances",
        group: "routing-group",
        label: "RIP",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/rip/instance",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("vrf", "VRF", 12),
            col!("originate-default", "Default", 8),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::routing_write::RIP_INSTANCE_FORM),
    },
    ResourceSpec {
        id: "rip-interface-templates",
        group: "routing-group",
        label: "RIP Interfaces",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/rip/interface-template",
        },
        columns: &[
            col!("instance", "Instance", 16),
            col!("interfaces", "Interfaces", 24),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::RIP_INTERFACE_TEMPLATE_FORM),
    },
    ResourceSpec {
        id: "bfd",
        group: "routing-group",
        label: "BFD",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/bfd/configuration",
        },
        columns: &[
            col!("interfaces", "Interfaces", 24),
            col!("addresses", "Addresses", 20),
            col!("min-tx-interval", "TX", 10),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::routing_write::BFD_CONFIGURATION_FORM),
    },
    ResourceSpec {
        id: "routing-filters",
        group: "routing-group",
        label: "Filters",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/filter/rule",
        },
        columns: &[
            col!("chain", "Chain", 16),
            col!("rule", "Rule", 40),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::FILTER_ACTIONS,
        form: Some(&crate::routing_write::ROUTING_FILTER_FORM),
    },
    ResourceSpec {
        id: "routing-id",
        group: "routing-group",
        label: "Router ID",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/id",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("id", "ID", 16),
            col!("select", "Select", 16),
            col!("disabled", "Off", 5),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::routing_write::ROUTING_ID_FORM),
    },
    ResourceSpec {
        id: "ospf-neighbors",
        group: "routing-group",
        label: "OSPF Neighbors",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/neighbor",
        },
        columns: &[
            col!("instance", "Instance", 16),
            col!("router-id", "Router ID", 16),
            col!("address", "Address", 18),
            col!("state", "State", 12),
            col!("adjacency", "Adjacency", 12),
        ],
        refresh: Duration::from_secs(5),
        actions: &[],
        form: None,
    },
    ResourceSpec {
        id: "ospf-lsa",
        group: "routing-group",
        label: "OSPF LSA",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/ospf/lsa",
        },
        columns: &[
            col!("type", "Type", 12),
            col!("id", "ID", 16),
            col!("originator", "Originator", 16),
            col!("area", "Area", 12),
            col!("sequence", "Seq", 10),
        ],
        refresh: Duration::from_secs(10),
        actions: &[],
        form: None,
    },
    ResourceSpec {
        id: "bgp-advertisements",
        group: "routing-group",
        label: "BGP Advertisements",
        fetch: FetchKind::List {
            endpoint: "/rest/routing/bgp/advertisements",
        },
        columns: &[
            col!("prefix", "Prefix", 24),
            col!("nexthop", "Nexthop", 18),
            col!("peer", "Peer", 16),
            col!("as-path", "AS path", 24),
        ],
        refresh: Duration::from_secs(5),
        actions: &[],
        form: None,
    },
    ResourceSpec {
        id: "queue-simple",
        group: "queue-group",
        label: "Simple",
        fetch: FetchKind::List {
            endpoint: "/rest/queue/simple",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("target", "Target", 24),
            col!("max-limit", "Max limit", 18),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::queue_write::QUEUE_SIMPLE_FORM),
    },
    ResourceSpec {
        id: "queue-tree",
        group: "queue-group",
        label: "Tree",
        fetch: FetchKind::List {
            endpoint: "/rest/queue/tree",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("parent", "Parent", 16),
            col!("packet-mark", "Mark", 16),
            col!("max-limit", "Max limit", 14),
            col!("priority", "Prio", 6),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::queue_write::QUEUE_TREE_FORM),
    },
    ResourceSpec {
        id: "queue-type",
        group: "queue-group",
        label: "Queue Type",
        fetch: FetchKind::List {
            endpoint: "/rest/queue/type",
        },
        columns: &[
            col!("name", "Name", 18),
            col!("kind", "Kind", 12),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::LIST_ACTIONS,
        form: Some(&crate::queue_write::QUEUE_TYPE_FORM),
    },
    ResourceSpec {
        id: "queue-interface",
        group: "queue-group",
        label: "Interface",
        fetch: FetchKind::List {
            endpoint: "/rest/queue/interface",
        },
        columns: &[
            col!("interface", "Interface", 18),
            col!("queue", "Queue", 18),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::HARDWARE_EDIT_ACTIONS,
        form: Some(&crate::queue_write::QUEUE_INTERFACE_FORM),
    },
    ResourceSpec {
        id: "files",
        group: "files-group",
        label: "Files",
        fetch: FetchKind::List {
            endpoint: "/rest/file",
        },
        columns: &[
            col!("name", "Name", 40),
            col!("type", "Type", 12),
            col!("size", "Size", 12),
            col!("creation-time", "Created", 20),
        ],
        refresh: Duration::from_secs(10),
        actions: crate::actions::FILE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "netwatch",
        group: "tools-group",
        label: "Netwatch",
        fetch: FetchKind::List {
            endpoint: "/rest/tool/netwatch",
        },
        columns: &[
            col!("host", "Host", 22),
            col!("type", "Type", 10),
            col!("interval", "Interval", 10),
            col!("status", "Status", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::tools_write::NETWATCH_FORM),
    },
    ResourceSpec {
        id: "email",
        group: "tools-group",
        label: "Email",
        fetch: FetchKind::System {
            endpoint: "/rest/tool/email",
        },
        columns: &[
            col!("server", "Server", 22),
            col!("from", "From", 24),
            col!("user", "User", 18),
            col!("password", "Password", 10),
            col!("tls", "TLS", 10),
            col!("port", "Port", 6),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::tools_write::EMAIL_FORM),
    },
    ResourceSpec {
        id: "romon",
        group: "tools-group",
        label: "RoMON",
        fetch: FetchKind::System {
            endpoint: "/rest/tool/romon",
        },
        columns: &[
            col!("enabled", "Enabled", 8),
            col!("id", "ID", 18),
            col!("secrets", "Secrets", 10),
            col!("current-id", "Current ID", 18),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::tools_write::ROMON_FORM),
    },
    ResourceSpec {
        id: "romon-ports",
        group: "tools-group",
        label: "RoMON Ports",
        fetch: FetchKind::List {
            endpoint: "/rest/tool/romon/port",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("forbid", "Forbid", 8),
            col!("cost", "Cost", 8),
            col!("secrets", "Secrets", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::tools_write::ROMON_PORT_FORM),
    },
    ResourceSpec {
        id: "graphing",
        group: "tools-group",
        label: "Graphing",
        fetch: FetchKind::System {
            endpoint: "/rest/tool/graphing",
        },
        columns: &[
            col!("store-every", "Store Every", 12),
            col!("page-refresh", "Page Refresh", 12),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::tools_write::GRAPHING_FORM),
    },
    ResourceSpec {
        id: "graphing-interface",
        group: "tools-group",
        label: "Graphing Interface",
        fetch: FetchKind::List {
            endpoint: "/rest/tool/graphing/interface",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("allow-address", "Allow Address", 18),
            col!("store-on-disk", "Store On Disk", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::tools_write::GRAPHING_INTERFACE_FORM),
    },
    ResourceSpec {
        id: "graphing-queue",
        group: "tools-group",
        label: "Graphing Queue",
        fetch: FetchKind::List {
            endpoint: "/rest/tool/graphing/queue",
        },
        columns: &[
            col!("simple-queue", "Simple Queue", 16),
            col!("allow-address", "Allow Address", 18),
            col!("allow-target", "Allow Target", 12),
            col!("store-on-disk", "Store On Disk", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::tools_write::GRAPHING_QUEUE_FORM),
    },
    ResourceSpec {
        id: "graphing-resource",
        group: "tools-group",
        label: "Graphing Resource",
        fetch: FetchKind::List {
            endpoint: "/rest/tool/graphing/resource",
        },
        columns: &[
            col!("allow-address", "Allow Address", 18),
            col!("store-on-disk", "Store On Disk", 12),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(30),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::tools_write::GRAPHING_RESOURCE_FORM),
    },
    ResourceSpec {
        id: "ping",
        group: "tools-group",
        label: "Ping",
        fetch: FetchKind::Local,
        columns: &[
            col!("seq", "Seq", 6),
            col!("host", "Host", 22),
            col!("time", "Time", 12),
            col!("ttl", "TTL", 6),
            col!("status", "Status", 12),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::PING_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "traceroute",
        group: "tools-group",
        label: "Traceroute",
        fetch: FetchKind::Local,
        columns: &[
            col!("hop", "Hop", 6),
            col!("address", "Address", 22),
            col!("status", "Status", 12),
            col!("time", "Time", 12),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::TRACEROUTE_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "sniffer",
        group: "tools-group",
        label: "Packet Sniffer",
        fetch: FetchKind::System {
            endpoint: "/rest/tool/sniffer",
        },
        columns: &[
            col!("interface", "Interface", 16),
            col!("file-name", "File", 24),
            col!("running", "Run", 5),
        ],
        refresh: Duration::from_secs(5),
        actions: crate::actions::SNIFFER_ACTIONS,
        form: Some(&crate::tools_write::SNIFFER_FORM),
    },
    ResourceSpec {
        id: "bandwidth-test",
        group: "tools-group",
        label: "Bandwidth Test",
        fetch: FetchKind::Local,
        columns: &[
            col!("address", "Address", 22),
            col!("tx-current", "TX", 12),
            col!("rx-current", "RX", 12),
            col!("status", "Status", 12),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::BANDWIDTH_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "flood-ping",
        group: "tools-group",
        label: "Flood Ping",
        fetch: FetchKind::Local,
        columns: &[
            col!("address", "Address", 22),
            col!("sent", "Sent", 8),
            col!("received", "Received", 10),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::FLOOD_PING_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "mac-scan",
        group: "tools-group",
        label: "MAC Scan",
        fetch: FetchKind::Local,
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC", 18),
            col!("age", "Age", 8),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::MAC_SCAN_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "ip-scan",
        group: "tools-group",
        label: "IP Scan",
        fetch: FetchKind::Local,
        columns: &[
            col!("address", "Address", 18),
            col!("mac-address", "MAC", 18),
            col!("time", "Time", 8),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::IP_SCAN_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "profiler",
        group: "tools-group",
        label: "Profiler",
        fetch: FetchKind::Local,
        columns: &[
            col!("name", "Name", 24),
            col!("usage", "Usage", 10),
            col!("load", "Load", 8),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::PROFILER_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "wol",
        group: "tools-group",
        label: "Wake on LAN",
        fetch: FetchKind::Local,
        columns: &[col!("interface", "Interface", 16), col!("mac", "MAC", 18)],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::WOL_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "sms",
        group: "tools-group",
        label: "SMS",
        fetch: FetchKind::Local,
        columns: &[
            col!("phone-number", "Phone", 16),
            col!("message", "Message", 36),
        ],
        refresh: Duration::from_secs(3600),
        actions: crate::actions::SMS_ACTIONS,
        form: None,
    },
    ResourceSpec {
        id: "radius",
        group: "radius-group",
        label: "RADIUS",
        fetch: FetchKind::List {
            endpoint: "/rest/radius",
        },
        columns: &[
            col!("address", "Address", 18),
            col!("protocol", "Proto", 8),
            col!("secret", "Secret", 10),
            col!("service", "Service", 16),
            col!("timeout", "Timeout", 10),
            col!("disabled", "Off", 5),
            col!("comment", "Comment", 28),
        ],
        refresh: Duration::from_secs(15),
        actions: crate::actions::MEMBER_ACTIONS,
        form: Some(&crate::radius_write::RADIUS_FORM),
    },
    ResourceSpec {
        id: "radius-incoming",
        group: "radius-group",
        label: "Incoming",
        fetch: FetchKind::System {
            endpoint: "/rest/radius/incoming",
        },
        columns: &[col!("accept", "Accept", 8), col!("port", "Port", 6)],
        refresh: Duration::from_secs(30),
        actions: crate::actions::SINGLETON_EDIT_ACTIONS,
        form: Some(&crate::radius_write::RADIUS_INCOMING_FORM),
    },
    ResourceSpec {
        id: "logs",
        group: "system-group",
        label: "Logs",
        fetch: FetchKind::List {
            endpoint: "/rest/log",
        },
        columns: &[
            col!("time", "Time", 19),
            col!("topics", "Topics", 24),
            col!("message", "Message", 72),
        ],
        refresh: Duration::from_secs(1),
        actions: &[],
        form: None,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItem {
    pub id: String,
    pub label: String,
    pub children: Vec<NavItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavGroup {
    pub id: &'static str,
    pub label: &'static str,
}

/// Top-level navigation tree (dashboard + resource groups).
pub static NAVIGATION: &[NavGroup] = &[
    NavGroup {
        id: "interfaces-group",
        label: "Interfaces",
    },
    NavGroup {
        id: "wireguard-group",
        label: "WireGuard",
    },
    NavGroup {
        id: "ppp-group",
        label: "PPP",
    },
    NavGroup {
        id: "bridge-group",
        label: "Bridge",
    },
    NavGroup {
        id: "switch-group",
        label: "Switch",
    },
    NavGroup {
        id: "ip-group",
        label: "IP",
    },
    NavGroup {
        id: "ipv6-group",
        label: "IPv6",
    },
    NavGroup {
        id: "routing-group",
        label: "Routing",
    },
    NavGroup {
        id: "queue-group",
        label: "Queues",
    },
    NavGroup {
        id: "files-group",
        label: "Files",
    },
    NavGroup {
        id: "tools-group",
        label: "Tools",
    },
    NavGroup {
        id: "radius-group",
        label: "RADIUS",
    },
    NavGroup {
        id: "system-group",
        label: "System",
    },
];

#[must_use]
pub fn resource_by_id(id: &str) -> Option<&'static ResourceSpec> {
    ALL_RESOURCES.iter().find(|spec| spec.id == id)
}

#[must_use]
pub fn navigation_tree() -> Vec<NavItem> {
    let mut items = vec![NavItem {
        id: DASHBOARD_ID.to_string(),
        label: "Dashboard".to_string(),
        children: Vec::new(),
    }];
    for group in NAVIGATION {
        let children = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group.id)
            .map(|spec| NavItem {
                id: spec.id.to_string(),
                label: spec.label.to_string(),
                children: Vec::new(),
            })
            .collect();
        items.push(NavItem {
            id: group.id.to_string(),
            label: group.label.to_string(),
            children,
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_ids() {
        let mut ids: Vec<_> = ALL_RESOURCES.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL_RESOURCES.len());
    }

    #[test]
    fn logging_actions_is_list_not_singleton() {
        let spec = resource_by_id("logging-actions").expect("logging-actions");
        assert!(!spec.is_singleton());
        assert_eq!(spec.endpoint(), "/rest/system/logging/action");
        assert_eq!(spec.cli_path(), "/system/logging/action");
        assert_eq!(spec.group, "system-group");
        assert_eq!(spec.label, "Logging Actions");
        assert!(spec.form.is_some());
        let action_ids: Vec<_> = spec.actions.iter().map(|action| action.id).collect();
        assert_eq!(
            action_ids,
            crate::actions::LIST_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert!(matches!(
            spec.fetch,
            FetchKind::List {
                endpoint: "/rest/system/logging/action"
            }
        ));
        assert_eq!(
            column_keys("logging-actions"),
            ["name", "target", "remote", "remote-port", "remote-protocol"]
        );
        let logging = resource_by_id("logging").expect("logging");
        assert_eq!(logging.endpoint(), "/rest/system/logging");
        let logging_ids: Vec<_> = logging.actions.iter().map(|action| action.id).collect();
        assert_ne!(logging_ids, action_ids);
    }

    #[test]
    fn smb_shares_and_users_are_lists() {
        let shares = resource_by_id("smb-shares").expect("smb-shares");
        let users = resource_by_id("smb-users").expect("smb-users");
        let service = resource_by_id("smb").expect("smb");
        assert!(service.is_singleton());
        assert_eq!(service.endpoint(), "/rest/ip/smb");
        assert!(!shares.is_singleton());
        assert!(!users.is_singleton());
        assert_eq!(shares.endpoint(), "/rest/ip/smb/shares");
        assert_eq!(users.endpoint(), "/rest/ip/smb/users");
        assert_eq!(shares.cli_path(), "/ip/smb/shares");
        assert_eq!(users.cli_path(), "/ip/smb/users");
        assert_eq!(shares.group, "ip-group");
        assert_eq!(users.group, "ip-group");
        assert_eq!(shares.label, "SMB Shares");
        assert_eq!(users.label, "SMB Users");
        assert!(shares.form.is_some());
        assert!(users.form.is_some());
        assert_eq!(
            column_keys("smb-shares"),
            [
                "name",
                "directory",
                "valid-users",
                "invalid-users",
                "read-only",
                "require-encryption",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("smb-users"),
            ["name", "password", "read-only", "disabled", "comment"]
        );
        let share_actions: Vec<_> = shares.actions.iter().map(|action| action.id).collect();
        assert_eq!(
            share_actions,
            crate::actions::MEMBER_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        let user_actions: Vec<_> = users.actions.iter().map(|action| action.id).collect();
        assert_eq!(user_actions, share_actions);
    }

    #[test]
    fn ntp_server_is_system_singleton() {
        let spec = resource_by_id("ntp-server").expect("ntp-server");
        assert!(spec.is_singleton());
        assert_eq!(spec.endpoint(), "/rest/system/ntp/server");
        assert_eq!(spec.cli_path(), "/system/ntp/server");
        assert!(spec.form.is_some());
        assert_eq!(spec.group, "system-group");
        assert_eq!(spec.label, "NTP Server");
        assert!(matches!(
            spec.fetch,
            FetchKind::System {
                endpoint: "/rest/system/ntp/server"
            }
        ));
        let ntp = resource_by_id("ntp").expect("ntp");
        assert_eq!(ntp.label, "NTP Client");
        assert_eq!(ntp.endpoint(), "/rest/system/ntp/client");
        let keys = resource_by_id("ntp-keys").expect("ntp-keys");
        assert!(!keys.is_singleton());
        assert_eq!(keys.endpoint(), "/rest/system/ntp/key");
        assert_eq!(column_keys("ntp-keys"), ["key-id"]);
        assert!(keys.form.is_some());
    }

    #[test]
    fn ospf_interface_runtime_is_not_the_template_menu() {
        let live = resource_by_id("ospf-interfaces").expect("ospf-interfaces");
        let templates =
            resource_by_id("ospf-interface-templates").expect("ospf-interface-templates");
        assert!(!live.is_singleton());
        assert_eq!(live.endpoint(), "/rest/routing/ospf/interface");
        assert_eq!(live.cli_path(), "/routing/ospf/interface");
        assert_eq!(live.group, "routing-group");
        assert_eq!(live.label, "OSPF Interface");
        assert_eq!(templates.label, "OSPF Interface Templates");
        assert_eq!(
            templates.endpoint(),
            "/rest/routing/ospf/interface-template"
        );
        assert_ne!(live.endpoint(), templates.endpoint());
        assert_eq!(
            live.actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["edit"]
        );
        assert!(live.form.is_some());
        assert!(
            live.form
                .is_some_and(|form| form.writable_keys().is_empty())
        );
        assert!(
            templates
                .form
                .is_some_and(|form| !form.writable_keys().is_empty())
        );
        assert_eq!(
            column_keys("ospf-interfaces"),
            [
                "address",
                "area",
                "state",
                "network-type",
                "cost",
                "dr",
                "bdr",
            ]
        );
        assert!(!column_keys("ospf-interface-templates").contains(&"cost"));
        assert!(!column_keys("ospf-interface-templates").contains(&"state"));
        assert!(!column_keys("ospf-interfaces").contains(&"interfaces"));
        assert!(!column_keys("ospf-interfaces").contains(&"disabled"));
        let neighbors = resource_by_id("ospf-neighbors").expect("ospf-neighbors");
        assert!(neighbors.form.is_none());
        assert!(neighbors.actions.is_empty());
    }

    #[test]
    fn traffic_flow_and_igmp_proxy_are_ip_group_screens() {
        let flow = resource_by_id("traffic-flow").expect("traffic-flow");
        assert!(flow.is_singleton());
        assert_eq!(flow.endpoint(), "/rest/ip/traffic-flow");
        assert_eq!(flow.cli_path(), "/ip/traffic-flow");
        assert_eq!(flow.group, "ip-group");
        assert!(flow.form.is_some());
        assert_eq!(
            column_keys("traffic-flow"),
            ["enabled", "interfaces", "cache-entries", "packet-sampling"]
        );

        let targets = resource_by_id("traffic-flow-targets").expect("traffic-flow-targets");
        assert!(!targets.is_singleton());
        assert_eq!(targets.endpoint(), "/rest/ip/traffic-flow/target");
        assert_eq!(
            targets
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            crate::actions::MEMBER_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            column_keys("traffic-flow-targets"),
            ["src-address", "dst-address", "port", "version", "disabled"]
        );

        let ipfix = resource_by_id("traffic-flow-ipfix").expect("traffic-flow-ipfix");
        assert!(ipfix.is_singleton());
        assert_eq!(ipfix.endpoint(), "/rest/ip/traffic-flow/ipfix");

        let proxy = resource_by_id("igmp-proxy").expect("igmp-proxy");
        assert!(proxy.is_singleton());
        assert_eq!(proxy.endpoint(), "/rest/routing/igmp-proxy");
        assert_eq!(proxy.cli_path(), "/routing/igmp-proxy");
        assert_eq!(proxy.group, "ip-group");

        let ifaces = resource_by_id("igmp-proxy-interfaces").expect("igmp-proxy-interfaces");
        assert!(!ifaces.is_singleton());
        assert_eq!(ifaces.endpoint(), "/rest/routing/igmp-proxy/interface");
        assert!(ifaces.form.is_some());

        let mfc = resource_by_id("igmp-proxy-mfc").expect("igmp-proxy-mfc");
        assert_eq!(mfc.endpoint(), "/rest/routing/igmp-proxy/mfc");
        assert_eq!(
            mfc.actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            crate::actions::LIST_ACTIONS
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>()
        );
        assert_unique_endpoints("ip-group");
    }

    #[test]
    fn ping_and_traceroute_are_local_fetch() {
        let ping = resource_by_id("ping").expect("ping");
        let traceroute = resource_by_id("traceroute").expect("traceroute");
        assert!(matches!(ping.fetch, FetchKind::Local));
        assert!(matches!(traceroute.fetch, FetchKind::Local));
        assert!(ping.form.is_none());
        assert!(traceroute.form.is_none());
        assert_eq!(ping.cli_path(), "/tool/ping");
        assert_eq!(traceroute.cli_path(), "/tool/traceroute");
    }

    #[test]
    fn navigation_includes_dashboard_and_logs() {
        let tree = navigation_tree();
        assert_eq!(tree[0].id, DASHBOARD_ID);
        assert!(resource_by_id("logs").is_some());
        assert!(resource_by_id("firewall-filter").is_some());
    }

    #[test]
    fn interface_tables_expose_webfig_columns() {
        assert_eq!(
            column_keys("interfaces"),
            [
                "name",
                "type",
                "mtu",
                "actual-mtu",
                "l2mtu",
                "max-l2mtu",
                "mac-address",
                "tx-byte",
                "rx-byte",
                "tx-packet",
                "rx-packet",
                "fp-tx-byte",
                "fp-rx-byte",
                "fp-tx-packet",
                "fp-rx-packet",
                "last-link-up-time",
                "last-link-down-time",
                "link-downs",
                "tx-drop",
                "rx-drop",
                "tx-queue-drop",
                "rx-error",
                "tx-error",
                "running",
                "slave",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("interface-lists"),
            ["name", "include", "exclude", "builtin", "comment"]
        );
        assert_eq!(
            column_keys("interface-list-members"),
            ["list", "interface", "dynamic", "disabled", "comment"]
        );
        assert_eq!(
            column_keys("ethernet"),
            [
                "name",
                "default-name",
                "mtu",
                "l2mtu",
                "mac-address",
                "orig-mac-address",
                "arp",
                "auto-negotiation",
                "advertise",
                "speed",
                "full-duplex",
                "switch",
                "loop-protect",
                "loop-protect-status",
                "running",
                "slave",
                "disabled",
                "comment",
            ]
        );
    }

    #[test]
    fn interface_group_covers_webfig_screens() {
        let ids: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == "interfaces-group")
            .map(|spec| spec.id)
            .collect();
        assert_eq!(
            ids,
            [
                "interfaces",
                "interface-lists",
                "interface-list-members",
                "ethernet",
                "eoip",
                "ipip",
                "gre",
                "6to4",
                "sit",
                "gre6",
                "vlan",
                "vxlan",
                "vrrp",
                "bonding",
                "lte",
                "lte-apn",
                "wifi",
                "wifi-security",
                "wifi-channel",
                "wifi-datapath",
                "wifi-configuration",
                "wifi-provisioning",
                "wifi-cap",
                "wifi-capsman",
                "wireless",
                "wireless-security-profiles",
                "wireless-access-list",
                "wireless-registration-table",
                "macvlan",
                "macsec",
                "macsec-profiles",
                "vrf",
                "detect-internet",
            ]
        );
        let endpoints: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == "interfaces-group")
            .map(ResourceSpec::endpoint)
            .collect();
        let mut unique = endpoints.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), endpoints.len());
        assert!(resource_by_id("detect-internet").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("vlan").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            column_keys("macsec"),
            [
                "name",
                "interface",
                "profile",
                "mtu",
                "status",
                "ckn",
                "cak",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(column_keys("macsec-profiles"), ["name", "server-priority"]);
        assert_eq!(
            resource_by_id("macsec").map(ResourceSpec::endpoint),
            Some("/rest/interface/macsec")
        );
        assert_eq!(
            resource_by_id("macsec-profiles").map(ResourceSpec::endpoint),
            Some("/rest/interface/macsec/profile")
        );
        assert_eq!(
            resource_by_id("lte-apn").map(|spec| spec.label),
            Some("LTE APN")
        );
        assert_eq!(
            resource_by_id("lte-apn").map(ResourceSpec::endpoint),
            Some("/rest/interface/lte/apn")
        );
        assert_eq!(
            resource_by_id("lte-apn").map(ResourceSpec::cli_path),
            Some("/interface/lte/apn")
        );
        assert_eq!(
            column_keys("lte"),
            [
                "name",
                "default-name",
                "mtu",
                "mac-address",
                "network-mode",
                "apn-profiles",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("lte-apn"),
            [
                "name",
                "apn",
                "authentication",
                "ip-type",
                "use-network-apn",
                "add-default-route",
                "password",
                "comment",
            ]
        );
        assert!(resource_by_id("lte-apn").is_some_and(|spec| spec.form.is_some()));
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "interfaces-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn interface_lists_and_members_are_distinct() {
        assert_eq!(
            resource_by_id("interface-lists").map(|spec| spec.label),
            Some("Lists")
        );
        assert_eq!(
            resource_by_id("interface-list-members").map(|spec| spec.label),
            Some("List members")
        );
        assert_eq!(
            resource_by_id("interface-lists").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("New list"))
        );
        assert_eq!(
            resource_by_id("interface-list-members").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("New list member"))
        );
        assert_eq!(
            resource_by_id("dhcp-networks").map(|spec| spec
                .actions
                .iter()
                .find(|a| a.id == "add")
                .map(|a| a.label)),
            Some(Some("Add"))
        );
    }

    #[test]
    fn wireguard_is_its_own_nav_group() {
        assert_eq!(
            resource_by_id("wireguard").map(|spec| spec.group),
            Some("wireguard-group")
        );
        assert_eq!(
            resource_by_id("wireguard-peers").map(|spec| spec.group),
            Some("wireguard-group")
        );
        let tree = navigation_tree();
        let group = tree
            .iter()
            .find(|item| item.id == "wireguard-group")
            .expect("wireguard nav group");
        let child_ids: Vec<_> = group.children.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(child_ids, ["wireguard", "wireguard-peers"]);
        assert!(
            tree.iter()
                .find(|item| item.id == "interfaces-group")
                .expect("interfaces nav group")
                .children
                .iter()
                .all(|item| item.id != "wireguard" && item.id != "wireguard-peers")
        );
    }

    #[test]
    fn wireguard_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("wireguard-group"),
            ["wireguard", "wireguard-peers"]
        );
        assert_unique_endpoints("wireguard-group");
        assert_eq!(
            column_keys("wireguard"),
            [
                "name",
                "listen-port",
                "public-key",
                "private-key",
                "mtu",
                "vrf",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert_eq!(
            column_keys("wireguard-peers"),
            [
                "name",
                "interface",
                "public-key",
                "endpoint-address",
                "endpoint-port",
                "allowed-address",
                "persistent-keepalive",
                "responder",
                "current-endpoint-address",
                "current-endpoint-port",
                "last-handshake",
                "rx",
                "tx",
                "disabled",
                "comment",
            ]
        );
        assert!(resource_by_id("wireguard").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("wireguard-peers").is_some_and(|spec| spec.form.is_some()));
        let wg_actions: Vec<_> = resource_by_id("wireguard")
            .expect("wireguard")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            wg_actions,
            [
                "add",
                "edit",
                "toggle-disabled",
                "copy",
                "remove",
                "reset-counters"
            ]
        );
        let peer_actions: Vec<_> = resource_by_id("wireguard-peers")
            .expect("wireguard-peers")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            peer_actions,
            ["add", "edit", "toggle-disabled", "copy", "remove"]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "wireguard-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn ppp_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("ppp-group"),
            [
                "ppp-secrets",
                "ppp-profiles",
                "ppp-active",
                "ppp-aaa",
                "ppp-client",
                "pppoe-clients",
                "pppoe-servers",
                "pppoe-server-ifaces",
                "pptp-client",
                "pptp-server-ifaces",
                "pptp-server",
                "l2tp-client",
                "l2tp-server-ifaces",
                "l2tp-server",
                "sstp-client",
                "sstp-server-ifaces",
                "sstp-server",
                "ovpn-client",
                "ovpn-server-ifaces",
                "ovpn-server",
            ]
        );
        assert_unique_endpoints("ppp-group");
        assert!(resource_by_id("ppp-aaa").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("l2tp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("sstp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("ovpn-server").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("pptp-server").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("ppp-secrets").is_some_and(ResourceSpec::is_singleton));
        assert!(column_keys("ppp-secrets").contains(&"password"));
        assert!(column_keys("l2tp-client").contains(&"ipsec-secret"));
        assert!(resource_by_id("ppp-secrets").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("ppp-aaa").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("ppp-active").is_some_and(|spec| spec.form.is_none()));
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "ppp-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn bridge_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("bridge-group"),
            [
                "bridges",
                "bridge-ports",
                "bridge-hosts",
                "bridge-vlans",
                "bridge-mdb",
                "bridge-msti",
                "bridge-filter",
                "bridge-nat",
                "bridge-settings",
                "bridge-port-controller",
                "bridge-port-controller-device",
                "bridge-port-controller-port",
                "bridge-port-extender",
            ]
        );
        assert_unique_endpoints("bridge-group");
        assert!(resource_by_id("bridge-settings").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("bridge-port-controller").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("bridge-port-extender").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("bridge-hosts").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            column_keys("bridges"),
            [
                "name",
                "protocol-mode",
                "vlan-filtering",
                "pvid",
                "igmp-snooping",
                "dhcp-snooping",
                "arp",
                "mac-address",
                "mtu",
                "fast-forward",
                "frame-types",
                "ingress-filtering",
                "priority",
                "region-name",
                "running",
                "disabled",
                "comment",
            ]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "bridge-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    fn switch_group_covers_webfig_screens() {
        assert_eq!(
            group_ids("switch-group"),
            [
                "switch",
                "switch-port",
                "switch-vlan",
                "switch-host",
                "switch-rule",
                "switch-port-isolation",
                "switch-l3hw",
            ]
        );
        assert_unique_endpoints("switch-group");
        assert!(resource_by_id("switch-l3hw").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("switch-rule").is_some_and(ResourceSpec::is_singleton));
        let tree = navigation_tree();
        let group = tree
            .iter()
            .find(|item| item.id == "switch-group")
            .expect("switch nav group");
        let child_ids: Vec<_> = group.children.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(
            child_ids,
            [
                "switch",
                "switch-port",
                "switch-vlan",
                "switch-host",
                "switch-rule",
                "switch-port-isolation",
                "switch-l3hw",
            ]
        );
        assert!(
            ALL_RESOURCES
                .iter()
                .filter(|spec| spec.group == "switch-group")
                .all(|spec| !spec.columns.is_empty())
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn ip_group_covers_webfig_operator_screens() {
        assert_eq!(
            group_ids("ip-group"),
            [
                "arp",
                "addresses",
                "dhcp-servers",
                "dhcp-networks",
                "dhcp-leases",
                "dhcp-relay",
                "dhcp-options",
                "dhcp-option-sets",
                "firewall-filter",
                "neighbors",
                "dhcp-clients",
                "dns",
                "dns-static",
                "routes",
                "pools",
                "ip-services",
                "ip-settings",
                "firewall-nat",
                "firewall-mangle",
                "firewall-raw",
                "firewall-connections",
                "address-list",
                "firewall-layer7",
                "firewall-service-port",
                "ipsec-peers",
                "ipsec-identities",
                "ipsec-policies",
                "ipsec-proposals",
                "ipsec-profiles",
                "ipsec-installed-sa",
                "ipsec-settings",
                "ipsec-mode-config",
                "ipsec-key",
                "cloud",
                "kid-control",
                "kid-control-devices",
                "socks",
                "smb",
                "smb-shares",
                "smb-users",
                "upnp",
                "upnp-interfaces",
                "dns-cache",
                "dhcp-alerts",
                "connection-tracking",
                "neighbor-discovery",
                "ip-ssh",
                "traffic-flow",
                "traffic-flow-targets",
                "traffic-flow-ipfix",
                "igmp-proxy",
                "igmp-proxy-interfaces",
                "igmp-proxy-mfc",
                "proxy",
                "proxy-access",
                "proxy-cache",
                "proxy-direct",
                "hotspot",
                "hotspot-profiles",
                "hotspot-users",
                "hotspot-cookies",
                "hotspot-hosts",
                "hotspot-ip-bindings",
                "hotspot-walled-garden",
                "hotspot-walled-garden-ip",
            ]
        );
        assert_unique_endpoints("ip-group");
        assert!(resource_by_id("dns").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("ipsec-settings").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("routes").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("neighbors").is_some_and(|spec| spec.form.is_none()));
        assert!(resource_by_id("ipsec-installed-sa").is_some_and(|spec| spec.form.is_none()));
        assert!(
            !column_keys("ipsec-installed-sa")
                .iter()
                .any(|key| { key.contains("key") || *key == "secret" || key.contains("auth-key") })
        );
        let connections = resource_by_id("firewall-connections").expect("firewall-connections");
        assert_eq!(connections.endpoint(), "/rest/ip/firewall/connection");
        assert!(connections.form.is_none());
        let connection_actions: Vec<_> =
            connections.actions.iter().map(|action| action.id).collect();
        assert_eq!(connection_actions, ["remove"]);
        assert_eq!(
            column_keys("firewall-connections"),
            [
                "src-address",
                "dst-address",
                "protocol",
                "src-port",
                "dst-port",
                "tcp-state",
                "timeout",
                "orig-rate",
                "repl-rate",
                "connection-mark",
            ]
        );
    }

    #[test]
    fn new_webfig_groups_exist() {
        assert_eq!(
            group_ids("ipv6-group"),
            [
                "ipv6-addresses",
                "ipv6-neighbors",
                "ipv6-nd",
                "ipv6-nd-prefix",
                "ipv6-routes",
                "ipv6-pool",
                "ipv6-dhcp-client",
                "ipv6-dhcp-server",
                "ipv6-settings",
                "ipv6-firewall-filter",
                "ipv6-firewall-nat",
                "ipv6-address-list",
                "ipv6-dhcp-relay",
                "ipv6-dhcp-bindings",
                "ipv6-firewall-mangle",
                "ipv6-firewall-raw",
                "ipv6-firewall-connections",
            ]
        );
        assert_eq!(
            group_ids("routing-group"),
            [
                "routing-tables",
                "routing-rules",
                "ospf-instances",
                "ospf-areas",
                "ospf-interface-templates",
                "ospf-interfaces",
                "bgp-connections",
                "bgp-templates",
                "rip-instances",
                "rip-interface-templates",
                "bfd",
                "routing-filters",
                "routing-id",
                "ospf-neighbors",
                "ospf-lsa",
                "bgp-advertisements",
            ]
        );
        assert_eq!(
            group_ids("queue-group"),
            [
                "queue-simple",
                "queue-tree",
                "queue-type",
                "queue-interface",
            ]
        );
        assert_eq!(group_ids("files-group"), ["files"]);
        assert_eq!(
            group_ids("tools-group"),
            [
                "netwatch",
                "email",
                "romon",
                "romon-ports",
                "graphing",
                "graphing-interface",
                "graphing-queue",
                "graphing-resource",
                "ping",
                "traceroute",
                "sniffer",
                "bandwidth-test",
                "flood-ping",
                "mac-scan",
                "ip-scan",
                "profiler",
                "wol",
                "sms",
            ]
        );
        assert_eq!(group_ids("radius-group"), ["radius", "radius-incoming"]);
        for group in [
            "ipv6-group",
            "routing-group",
            "queue-group",
            "files-group",
            "tools-group",
            "radius-group",
        ] {
            assert_unique_endpoints(group);
        }
        let tree = navigation_tree();
        let labels: Vec<_> = tree.iter().map(|item| item.id.as_str()).collect();
        assert!(labels.contains(&"ipv6-group"));
        assert!(labels.contains(&"radius-group"));
        assert_eq!(labels.last().copied(), Some("system-group"));
    }

    #[test]
    fn ipv6_firewall_connections_mirror_ipv4() {
        let ipv6_connections =
            resource_by_id("ipv6-firewall-connections").expect("ipv6-firewall-connections");
        assert_eq!(
            ipv6_connections.endpoint(),
            "/rest/ipv6/firewall/connection"
        );
        assert!(ipv6_connections.form.is_none());
        let ipv6_connection_actions: Vec<_> = ipv6_connections
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(ipv6_connection_actions, ["remove"]);
        assert_eq!(
            column_keys("ipv6-firewall-connections"),
            [
                "src-address",
                "dst-address",
                "protocol",
                "src-port",
                "dst-port",
                "tcp-state",
                "timeout",
                "orig-rate",
                "repl-rate",
                "connection-mark",
            ]
        );
        assert_eq!(
            column_keys("ipv6-firewall-connections"),
            column_keys("firewall-connections")
        );
    }

    #[test]
    fn romon_and_graphing_cover_webfig_tools() {
        assert!(resource_by_id("romon").is_some_and(ResourceSpec::is_singleton));
        assert!(resource_by_id("graphing").is_some_and(ResourceSpec::is_singleton));
        assert!(!resource_by_id("romon-ports").is_some_and(ResourceSpec::is_singleton));
        assert_eq!(
            resource_by_id("romon").expect("romon").endpoint(),
            "/rest/tool/romon"
        );
        assert_eq!(
            resource_by_id("romon-ports").expect("ports").endpoint(),
            "/rest/tool/romon/port"
        );
        assert_eq!(
            resource_by_id("graphing").expect("graphing").endpoint(),
            "/rest/tool/graphing"
        );
        assert_eq!(
            resource_by_id("graphing-interface").expect("gi").endpoint(),
            "/rest/tool/graphing/interface"
        );
        assert_eq!(
            resource_by_id("graphing-queue").expect("gq").endpoint(),
            "/rest/tool/graphing/queue"
        );
        assert_eq!(
            resource_by_id("graphing-resource").expect("gr").endpoint(),
            "/rest/tool/graphing/resource"
        );
        assert!(resource_by_id("romon").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("romon-ports").is_some_and(|spec| spec.form.is_some()));
        assert!(resource_by_id("graphing").is_some_and(|spec| spec.form.is_some()));
        assert!(column_keys("romon").contains(&"secrets"));
        assert!(column_keys("romon-ports").contains(&"forbid"));
        assert_eq!(column_keys("graphing"), ["store-every", "page-refresh"]);
        let port_actions: Vec<_> = resource_by_id("romon-ports")
            .expect("ports")
            .actions
            .iter()
            .map(|action| action.id)
            .collect();
        assert_eq!(
            port_actions,
            ["add", "edit", "toggle-disabled", "copy", "remove"]
        );
    }

    #[test]
    fn neighbors_connect_is_overlay_without_a_form() {
        let neighbors = resource_by_id("neighbors").expect("neighbors");
        assert!(neighbors.form.is_none());
        assert_eq!(neighbors.endpoint(), "/rest/ip/neighbor");
        assert_eq!(
            neighbors
                .actions
                .iter()
                .map(|action| action.id)
                .collect::<Vec<_>>(),
            ["connect", "remove"]
        );
        assert_eq!(
            neighbors.actions[0].kind,
            crate::actions::ActionKind::Overlay {
                id: "connect-neighbor"
            }
        );
    }

    #[test]
    fn mutations_require_forms_except_remove_only_rows() {
        use crate::actions::ActionKind;

        for spec in ALL_RESOURCES {
            if spec.actions.is_empty() {
                continue;
            }
            let ids: Vec<_> = spec.actions.iter().map(|action| action.id).collect();
            let overlay_only = spec
                .actions
                .iter()
                .all(|action| matches!(action.kind, crate::actions::ActionKind::Overlay { .. }));
            if ids == ["remove"] {
                assert!(spec.form.is_none(), "{} should be remove-only", spec.id);
                continue;
            }
            if spec.id == "files" {
                assert!(
                    spec.form.is_none(),
                    "files uses transfer prompts, not a sheet"
                );
                continue;
            }
            if overlay_only {
                assert!(
                    spec.form.is_none(),
                    "{} is overlay-only and should not have a form",
                    spec.id
                );
                continue;
            }
            let needs_sheet = spec
                .actions
                .iter()
                .any(|action| matches!(action.kind, ActionKind::Edit | ActionKind::Create));
            if needs_sheet {
                assert!(spec.form.is_some(), "{} needs a property sheet", spec.id);
            }
        }
        assert!(resource_by_id("logs").is_some_and(|spec| spec.actions.is_empty()));
        assert!(
            resource_by_id("routerboard")
                .is_some_and(|spec| spec.actions.iter().any(|action| action.id == "upgrade"))
        );
        assert!(
            resource_by_id("resources").is_some_and(|spec| spec.form.is_none()
                && spec.actions.iter().any(|action| action.id == "reboot"))
        );
        assert!(
            resource_by_id("files").is_some_and(|spec| spec.form.is_none()
                && spec.actions.iter().any(|action| action.id == "backup-save")
                && spec.actions.iter().any(|action| action.id == "fetch"))
        );
    }

    fn group_ids(group: &str) -> Vec<&'static str> {
        ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group)
            .map(|spec| spec.id)
            .collect()
    }

    fn assert_unique_endpoints(group: &str) {
        let endpoints: Vec<_> = ALL_RESOURCES
            .iter()
            .filter(|spec| spec.group == group)
            .filter(|spec| !matches!(spec.fetch, FetchKind::Local))
            .map(ResourceSpec::endpoint)
            .collect();
        let mut unique = endpoints.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), endpoints.len());
    }

    fn column_keys(id: &str) -> Vec<&'static str> {
        resource_by_id(id)
            .expect("catalog resource")
            .columns
            .iter()
            .map(|col| col.key)
            .collect()
    }
}
