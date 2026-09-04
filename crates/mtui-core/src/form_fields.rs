//! Shared property-sheet kinds used by more than one feature form.
//!
//! Feature modules still own labels and keys. Value lists and `Lookup`
//! targets live here so `protocol`, `chain`, and interface pickers do not
//! each invent a slightly different combo.

use crate::forms::{EnumChoice, FieldKind, FieldSpec, ScalarKind};

pub const LOOKUP_INTERFACES: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_INTERFACES_MULTI: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: true,
};
pub const LOOKUP_INTERFACE_LISTS: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_INTERFACE_LISTS_MULTI: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: true,
};
pub const LOOKUP_POOLS: FieldKind = FieldKind::Lookup {
    resource_id: "pools",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_FILES: FieldKind = FieldKind::Lookup {
    resource_id: "files",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_CERTIFICATES: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_ROUTING_TABLES: FieldKind = FieldKind::Lookup {
    resource_id: "routing-tables",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_ADDRESS_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "address-list",
    value_key: "list",
    multiple: false,
};
pub const LOOKUP_PORTS: FieldKind = FieldKind::Lookup {
    resource_id: "ports",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_KID_CONTROL: FieldKind = FieldKind::Lookup {
    resource_id: "kid-control",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_DHCP_SERVERS: FieldKind = FieldKind::Lookup {
    resource_id: "dhcp-servers",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_IPV6_DHCP_SERVER: FieldKind = FieldKind::Lookup {
    resource_id: "ipv6-dhcp-server",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_VRRP: FieldKind = FieldKind::Lookup {
    resource_id: "vrrp",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_WIFI_AAA: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-aaa",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_WIFI_INTERWORKING: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-interworking",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_WIFI_STEERING: FieldKind = FieldKind::Lookup {
    resource_id: "wifi-steering",
    value_key: "name",
    multiple: false,
};
pub const LOOKUP_IPSEC_POLICY_GROUPS: FieldKind = FieldKind::Lookup {
    resource_id: "ipsec-policy-groups",
    value_key: "name",
    multiple: false,
};

pub const IP_PROTOCOLS: &[&str] = &[
    "tcp",
    "udp",
    "icmp",
    "icmpv6",
    "igmp",
    "gre",
    "esp",
    "ah",
    "sctp",
    "ospf",
    "ipip",
    "vrrp",
    "l2tp",
    "ipsec-esp",
    "ipsec-ah",
];
pub const MAC_PROTOCOLS: &[&str] = &[
    "ip",
    "ipv6",
    "arp",
    "vlan",
    "pppoe",
    "pppoe-discovery",
    "mpls-unicast",
    "mpls-multicast",
    "802.2",
    "rarp",
    "ipx",
];
pub const FILTER_CHAINS: &[&str] = &["input", "forward", "output"];
pub const MANGLE_CHAINS: &[&str] = &["prerouting", "input", "forward", "output", "postrouting"];
pub const RAW_CHAINS: &[&str] = &["prerouting", "output"];
pub const NAT_CHAINS: &[&str] = &["srcnat", "dstnat"];
pub const BRIDGE_FILTER_CHAINS: &[&str] = &["input", "forward", "output"];
pub const BRIDGE_NAT_CHAINS: &[&str] = &["srcnat", "dstnat"];

pub const FILTER_ACTIONS: &[&str] = &[
    "accept",
    "drop",
    "reject",
    "jump",
    "return",
    "log",
    "passthrough",
    "add-src-to-address-list",
    "add-dst-to-address-list",
    "fasttrack-connection",
    "tarpit",
];
pub const NAT_ACTIONS: &[&str] = &[
    "accept",
    "dst-nat",
    "src-nat",
    "masquerade",
    "redirect",
    "netmap",
    "same",
    "jump",
    "return",
    "log",
    "passthrough",
    "add-src-to-address-list",
    "add-dst-to-address-list",
];
pub const MANGLE_ACTIONS: &[&str] = &[
    "accept",
    "drop",
    "jump",
    "return",
    "log",
    "passthrough",
    "mark-connection",
    "mark-packet",
    "mark-routing",
    "change-mss",
    "change-ttl",
    "change-dscp",
    "set-priority",
    "sniff-tzsp",
    "sniff-pc",
    "strip-ipv4-options",
    "clear-df",
    "add-src-to-address-list",
    "add-dst-to-address-list",
];
pub const RAW_ACTIONS: &[&str] = &[
    "accept",
    "drop",
    "notrack",
    "jump",
    "return",
    "log",
    "passthrough",
    "add-src-to-address-list",
    "add-dst-to-address-list",
];
pub const BRIDGE_FILTER_ACTIONS: &[&str] = &[
    "accept",
    "passthrough",
    "drop",
    "jump",
    "return",
    "log",
    "mark-packet",
    "set-priority",
];
pub const BRIDGE_NAT_ACTIONS: &[&str] = &[
    "accept",
    "passthrough",
    "drop",
    "jump",
    "return",
    "log",
    "mark-packet",
    "set-priority",
    "src-nat",
    "dst-nat",
    "redirect",
    "arp-reply",
];

pub const KIND_IP_PROTOCOL: FieldKind = FieldKind::Enum {
    values: IP_PROTOCOLS,
};
pub const KIND_MAC_PROTOCOL: FieldKind = FieldKind::Enum {
    values: MAC_PROTOCOLS,
};
pub const KIND_FILTER_CHAIN: FieldKind = FieldKind::Enum {
    values: FILTER_CHAINS,
};
pub const KIND_MANGLE_CHAIN: FieldKind = FieldKind::Enum {
    values: MANGLE_CHAINS,
};
pub const KIND_RAW_CHAIN: FieldKind = FieldKind::Enum { values: RAW_CHAINS };
pub const KIND_NAT_CHAIN: FieldKind = FieldKind::Enum { values: NAT_CHAINS };
pub const KIND_FILTER_ACTION: FieldKind = FieldKind::Enum {
    values: FILTER_ACTIONS,
};
pub const KIND_NAT_ACTION: FieldKind = FieldKind::Enum {
    values: NAT_ACTIONS,
};
pub const KIND_MANGLE_ACTION: FieldKind = FieldKind::Enum {
    values: MANGLE_ACTIONS,
};
pub const KIND_RAW_ACTION: FieldKind = FieldKind::Enum {
    values: RAW_ACTIONS,
};
pub const KIND_BRIDGE_FILTER_CHAIN: FieldKind = FieldKind::Enum {
    values: BRIDGE_FILTER_CHAINS,
};
pub const KIND_BRIDGE_NAT_CHAIN: FieldKind = FieldKind::Enum {
    values: BRIDGE_NAT_CHAINS,
};
pub const KIND_BRIDGE_FILTER_ACTION: FieldKind = FieldKind::Enum {
    values: BRIDGE_FILTER_ACTIONS,
};
pub const KIND_BRIDGE_NAT_ACTION: FieldKind = FieldKind::Enum {
    values: BRIDGE_NAT_ACTIONS,
};

pub const RP_FILTER: &[&str] = &["no", "loose", "strict"];
pub const CONNTRACK_ENABLED: &[EnumChoice] = &[
    EnumChoice {
        label: "yes",
        value: "yes",
    },
    EnumChoice {
        label: "no",
        value: "no",
    },
    EnumChoice {
        label: "auto",
        value: "auto",
    },
];
pub const NEIGHBOR_MODE: &[&str] = &["rx-only", "tx-only", "tx-and-rx"];
pub const SSH_FORWARDING: &[&str] = &["no", "local", "remote", "both"];
pub const SSH_HOST_KEY_SIZE: &[&str] = &["1024", "1536", "2048", "4096", "8192"];
pub const UPNP_INTERFACE_TYPE: &[&str] = &["external", "internal"];
pub const DNS_STATIC_TYPE: &[&str] = &[
    "A", "AAAA", "CNAME", "FWD", "MX", "NS", "NXDOMAIN", "SRV", "TXT",
];
pub const IPSEC_EXCHANGE_MODE: &[&str] = &["main", "aggressive", "base", "ike2"];
pub const IPSEC_AUTH_METHOD: &[&str] = &[
    "pre-shared-key",
    "digital-signature",
    "eap",
    "eap-radius",
    "pre-shared-key-xauth",
    "rsa-key",
    "rsa-signature-hybrid",
];
pub const IPSEC_GENERATE_POLICY: &[&str] = &["no", "port-override", "port-strict"];
pub const IPSEC_POLICY_ACTION: &[&str] = &["encrypt", "none", "discard"];
pub const IPSEC_POLICY_LEVEL: &[&str] = &["require", "unique", "use"];
pub const IPSEC_PROTOCOLS: &[&str] = &["esp", "ah"];
pub const IPSEC_PFS_GROUP: &[&str] = &[
    "none", "modp1024", "modp1536", "modp2048", "modp3072", "modp4096", "modp6144", "modp8192",
    "ecp256", "ecp384", "ecp521",
];
pub const IPSEC_HASH: &[&str] = &["md5", "sha1", "sha256", "sha512"];
pub const IPSEC_PROPOSAL_CHECK: &[&str] = &["claim", "exact", "obey", "strict"];
pub const IPSEC_IDENTITIES_MATCHING: &[&str] = &["type", "remote-id"];
pub const HOTSPOT_BINDING_TYPE: &[&str] = &["regular", "bypassed", "blocked"];
pub const ALLOW_DENY: &[&str] = &["allow", "deny"];
pub const PROXY_ACCESS_ACTION: &[&str] = &["allow", "deny", "redirect"];
pub const HTTP_METHODS: &[&str] = &["get", "post", "head", "put", "delete", "connect", "options"];
pub const IPV6_ACCEPT_REDIRECTS: &[EnumChoice] = &[
    EnumChoice {
        label: "no",
        value: "no",
    },
    EnumChoice {
        label: "yes if forwarding disabled",
        value: "yes-if-forwarding-disabled",
    },
];
pub const ADVERTISE_DNS: &[&str] = &["no", "yes", "self"];
pub const NTP_CLIENT_MODE: &[&str] = &["unicast", "broadcast", "multicast", "manycast"];
pub const SNMP_SECURITY: &[&str] = &["none", "authorized", "private"];
pub const PACKAGE_CHANNEL: &[&str] = &["long-term", "stable", "testing", "development"];
pub const ORIGINATE_DEFAULT: &[&str] = &["never", "if-installed", "always"];
pub const USE_IPSEC_REQUIRE: &[&str] = &["no", "yes", "require"];
pub const WDS_MODE: &[&str] = &[
    "disabled",
    "static",
    "dynamic",
    "static-mesh",
    "dynamic-mesh",
];
pub const TLS_MODE: &[&str] = &[
    "verify-certificate",
    "dont-verify-certificate",
    "no-certificates",
    "verify-certificate-with-crl",
];
pub const WIRELESS_VLAN_MODE: &[&str] = &["default", "no-tag", "use-tag", "use-service-tag"];
pub const WIRELESS_BAND: &[&str] = &[
    "2ghz-b",
    "2ghz-b/g",
    "2ghz-b/g/n",
    "2ghz-onlyg",
    "2ghz-onlyn",
    "5ghz-a",
    "5ghz-a/n",
    "5ghz-a/n/ac",
    "5ghz-onlyn",
    "5ghz-onlyac",
    "5ghz-n/ac",
];
pub const WIRELESS_CHANNEL_WIDTH: &[&str] = &[
    "20mhz",
    "10mhz",
    "5mhz",
    "20/40mhz-Ce",
    "20/40mhz-eC",
    "20/40mhz-XX",
    "20/40/80mhz-Ceee",
    "20/40/80mhz-eCee",
    "20/40/80mhz-eeCe",
    "20/40/80mhz-eeeC",
    "20/40/80mhz-XXXX",
];
pub const NV2_TDMA_PERIOD: &[EnumChoice] = &[
    EnumChoice {
        label: "auto",
        value: "auto",
    },
    EnumChoice {
        label: "1",
        value: "1",
    },
    EnumChoice {
        label: "2",
        value: "2",
    },
    EnumChoice {
        label: "3",
        value: "3",
    },
    EnumChoice {
        label: "4",
        value: "4",
    },
    EnumChoice {
        label: "5",
        value: "5",
    },
    EnumChoice {
        label: "6",
        value: "6",
    },
    EnumChoice {
        label: "7",
        value: "7",
    },
    EnumChoice {
        label: "8",
        value: "8",
    },
    EnumChoice {
        label: "9",
        value: "9",
    },
    EnumChoice {
        label: "10",
        value: "10",
    },
];

pub const KIND_RP_FILTER: FieldKind = FieldKind::Enum { values: RP_FILTER };
pub const KIND_CONNTRACK_ENABLED: FieldKind = FieldKind::LabeledEnum {
    choices: CONNTRACK_ENABLED,
};
pub const KIND_NEIGHBOR_MODE: FieldKind = FieldKind::Enum {
    values: NEIGHBOR_MODE,
};
pub const KIND_SSH_FORWARDING: FieldKind = FieldKind::Enum {
    values: SSH_FORWARDING,
};
pub const KIND_SSH_HOST_KEY_SIZE: FieldKind = FieldKind::Enum {
    values: SSH_HOST_KEY_SIZE,
};
pub const KIND_UPNP_TYPE: FieldKind = FieldKind::Enum {
    values: UPNP_INTERFACE_TYPE,
};
pub const KIND_DNS_STATIC_TYPE: FieldKind = FieldKind::Enum {
    values: DNS_STATIC_TYPE,
};
pub const KIND_IPSEC_EXCHANGE_MODE: FieldKind = FieldKind::Enum {
    values: IPSEC_EXCHANGE_MODE,
};
pub const KIND_IPSEC_AUTH_METHOD: FieldKind = FieldKind::Enum {
    values: IPSEC_AUTH_METHOD,
};
pub const KIND_IPSEC_GENERATE_POLICY: FieldKind = FieldKind::Enum {
    values: IPSEC_GENERATE_POLICY,
};
pub const KIND_IPSEC_POLICY_ACTION: FieldKind = FieldKind::Enum {
    values: IPSEC_POLICY_ACTION,
};
pub const KIND_IPSEC_POLICY_LEVEL: FieldKind = FieldKind::Enum {
    values: IPSEC_POLICY_LEVEL,
};
pub const KIND_IPSEC_PROTOCOLS: FieldKind = FieldKind::Enum {
    values: IPSEC_PROTOCOLS,
};
pub const KIND_IPSEC_PFS_GROUP: FieldKind = FieldKind::Enum {
    values: IPSEC_PFS_GROUP,
};
pub const KIND_IPSEC_HASH: FieldKind = FieldKind::Enum { values: IPSEC_HASH };
pub const KIND_IPSEC_PROPOSAL_CHECK: FieldKind = FieldKind::Enum {
    values: IPSEC_PROPOSAL_CHECK,
};
pub const KIND_IPSEC_IDENTITIES_MATCHING: FieldKind = FieldKind::Enum {
    values: IPSEC_IDENTITIES_MATCHING,
};
pub const KIND_HOTSPOT_BINDING_TYPE: FieldKind = FieldKind::Enum {
    values: HOTSPOT_BINDING_TYPE,
};
pub const KIND_ALLOW_DENY: FieldKind = FieldKind::Enum { values: ALLOW_DENY };
pub const KIND_PROXY_ACCESS_ACTION: FieldKind = FieldKind::Enum {
    values: PROXY_ACCESS_ACTION,
};
pub const KIND_HTTP_METHOD: FieldKind = FieldKind::Enum {
    values: HTTP_METHODS,
};
pub const KIND_IPV6_ACCEPT_REDIRECTS: FieldKind = FieldKind::LabeledEnum {
    choices: IPV6_ACCEPT_REDIRECTS,
};
pub const KIND_ADVERTISE_DNS: FieldKind = FieldKind::Enum {
    values: ADVERTISE_DNS,
};
pub const KIND_NTP_CLIENT_MODE: FieldKind = FieldKind::Enum {
    values: NTP_CLIENT_MODE,
};
pub const KIND_SNMP_SECURITY: FieldKind = FieldKind::Enum {
    values: SNMP_SECURITY,
};
pub const KIND_PACKAGE_CHANNEL: FieldKind = FieldKind::Enum {
    values: PACKAGE_CHANNEL,
};
pub const KIND_ORIGINATE_DEFAULT: FieldKind = FieldKind::Enum {
    values: ORIGINATE_DEFAULT,
};
pub const KIND_USE_IPSEC_REQUIRE: FieldKind = FieldKind::Enum {
    values: USE_IPSEC_REQUIRE,
};
pub const KIND_WDS_MODE: FieldKind = FieldKind::Enum { values: WDS_MODE };
pub const KIND_TLS_MODE: FieldKind = FieldKind::Enum { values: TLS_MODE };
pub const KIND_WIRELESS_VLAN_MODE: FieldKind = FieldKind::Enum {
    values: WIRELESS_VLAN_MODE,
};
pub const KIND_WIRELESS_BAND: FieldKind = FieldKind::Enum {
    values: WIRELESS_BAND,
};
pub const KIND_WIRELESS_CHANNEL_WIDTH: FieldKind = FieldKind::Enum {
    values: WIRELESS_CHANNEL_WIDTH,
};
pub const KIND_NV2_TDMA_PERIOD: FieldKind = FieldKind::LabeledEnum {
    choices: NV2_TDMA_PERIOD,
};

pub const FIELD_PROTOCOL: FieldSpec = spec("protocol", "Protocol", KIND_IP_PROTOCOL);
pub const FIELD_FILTER_CHAIN: FieldSpec = spec("chain", "Chain", KIND_FILTER_CHAIN);
pub const FIELD_MANGLE_CHAIN: FieldSpec = spec("chain", "Chain", KIND_MANGLE_CHAIN);
pub const FIELD_RAW_CHAIN: FieldSpec = spec("chain", "Chain", KIND_RAW_CHAIN);
pub const FIELD_NAT_CHAIN: FieldSpec = spec("chain", "Chain", KIND_NAT_CHAIN);
pub const FIELD_FILTER_ACTION: FieldSpec = spec("action", "Action", KIND_FILTER_ACTION);
pub const FIELD_NAT_ACTION: FieldSpec = spec("action", "Action", KIND_NAT_ACTION);
pub const FIELD_MANGLE_ACTION: FieldSpec = spec("action", "Action", KIND_MANGLE_ACTION);
pub const FIELD_RAW_ACTION: FieldSpec = spec("action", "Action", KIND_RAW_ACTION);
pub const FIELD_BRIDGE_FILTER_CHAIN: FieldSpec = spec("chain", "Chain", KIND_BRIDGE_FILTER_CHAIN);
pub const FIELD_BRIDGE_NAT_CHAIN: FieldSpec = spec("chain", "Chain", KIND_BRIDGE_NAT_CHAIN);
pub const FIELD_BRIDGE_FILTER_ACTION: FieldSpec =
    spec("action", "Action", KIND_BRIDGE_FILTER_ACTION);
pub const FIELD_BRIDGE_NAT_ACTION: FieldSpec = spec("action", "Action", KIND_BRIDGE_NAT_ACTION);
pub const FIELD_MAC_PROTOCOL: FieldSpec = spec("mac-protocol", "MAC protocol", KIND_MAC_PROTOCOL);
pub const FIELD_IP_PROTOCOL: FieldSpec = spec("ip-protocol", "IP protocol", KIND_IP_PROTOCOL);
pub const FIELD_ALLOW_DENY: FieldSpec = spec("action", "Action", KIND_ALLOW_DENY);
pub const FIELD_PROXY_ACCESS_ACTION: FieldSpec = spec("action", "Action", KIND_PROXY_ACCESS_ACTION);

/// IEEE bridge priority (4096-step hex). Port STP uses [`BRIDGE_PORT_PRIORITY`].
pub const BRIDGE_PRIORITY: &[EnumChoice] = &[
    choice("0x0000", "0x0000"),
    choice("0x1000", "0x1000"),
    choice("0x2000", "0x2000"),
    choice("0x3000", "0x3000"),
    choice("0x4000", "0x4000"),
    choice("0x5000", "0x5000"),
    choice("0x6000", "0x6000"),
    choice("0x7000", "0x7000"),
    choice("0x8000", "0x8000"),
    choice("0x9000", "0x9000"),
    choice("0xa000", "0xa000"),
    choice("0xb000", "0xb000"),
    choice("0xc000", "0xc000"),
    choice("0xd000", "0xd000"),
    choice("0xe000", "0xe000"),
    choice("0xf000", "0xf000"),
];
pub const BRIDGE_PORT_PRIORITY: &[EnumChoice] = &[
    choice("0x00", "0x00"),
    choice("0x10", "0x10"),
    choice("0x20", "0x20"),
    choice("0x30", "0x30"),
    choice("0x40", "0x40"),
    choice("0x50", "0x50"),
    choice("0x60", "0x60"),
    choice("0x70", "0x70"),
    choice("0x80", "0x80"),
    choice("0x90", "0x90"),
    choice("0xa0", "0xa0"),
    choice("0xb0", "0xb0"),
    choice("0xc0", "0xc0"),
    choice("0xd0", "0xd0"),
    choice("0xe0", "0xe0"),
    choice("0xf0", "0xf0"),
];
pub const KIND_BRIDGE_PRIORITY: FieldKind = FieldKind::LabeledEnum {
    choices: BRIDGE_PRIORITY,
};
pub const KIND_BRIDGE_PORT_PRIORITY: FieldKind = FieldKind::LabeledEnum {
    choices: BRIDGE_PORT_PRIORITY,
};

pub const STOP_SIGNAL: &[EnumChoice] = &[
    choice("1-SIGHUP", "1"),
    choice("2-SIGINT", "2"),
    choice("3-SIGQUIT", "3"),
    choice("9-SIGKILL", "9"),
    choice("15-SIGTERM", "15"),
];
pub const KIND_STOP_SIGNAL: FieldKind = FieldKind::LabeledEnum {
    choices: STOP_SIGNAL,
};

/// Observed `RouterOS` wifi/wireless country combo. Keep printed unknowns.
pub const WIFI_COUNTRY: &[EnumChoice] = &[
    choice("etsi", "etsi"),
    choice("united states", "united states"),
    choice("no_country_set", "no_country_set"),
    choice("superchannel", "superchannel"),
    choice("austria", "austria"),
    choice("belgium", "belgium"),
    choice("bulgaria", "bulgaria"),
    choice("canada", "canada"),
    choice("croatia", "croatia"),
    choice("cyprus", "cyprus"),
    choice("czech republic", "czech republic"),
    choice("denmark", "denmark"),
    choice("estonia", "estonia"),
    choice("finland", "finland"),
    choice("france", "france"),
    choice("germany", "germany"),
    choice("greece", "greece"),
    choice("hungary", "hungary"),
    choice("iceland", "iceland"),
    choice("ireland", "ireland"),
    choice("israel", "israel"),
    choice("italy", "italy"),
    choice("japan", "japan"),
    choice("latvia", "latvia"),
    choice("liechtenstein", "liechtenstein"),
    choice("lithuania", "lithuania"),
    choice("luxembourg", "luxembourg"),
    choice("malta", "malta"),
    choice("netherlands", "netherlands"),
    choice("norway", "norway"),
    choice("poland", "poland"),
    choice("portugal", "portugal"),
    choice("romania", "romania"),
    choice("slovakia", "slovakia"),
    choice("slovenia", "slovenia"),
    choice("spain", "spain"),
    choice("sweden", "sweden"),
    choice("switzerland", "switzerland"),
    choice("turkey", "turkey"),
    choice("united kingdom", "united kingdom"),
    choice("australia", "australia"),
    choice("brazil", "brazil"),
    choice("china", "china"),
    choice("india", "india"),
    choice("indonesia", "indonesia"),
    choice("korea republic", "korea republic"),
    choice("mexico", "mexico"),
    choice("new zealand", "new zealand"),
    choice("russia", "russia"),
    choice("singapore", "singapore"),
    choice("south africa", "south africa"),
    choice("taiwan", "taiwan"),
    choice("thailand", "thailand"),
    choice("ukraine", "ukraine"),
];
pub const KIND_WIFI_COUNTRY: FieldKind = FieldKind::LabeledEnum {
    choices: WIFI_COUNTRY,
};
pub const KIND_WIFI_COUNTRY_OPTIONAL: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Enum {
        choices: WIFI_COUNTRY,
    },
    unset: "",
    unset_label: "none",
};

/// Common `RouterOS` timezone names plus `manual`. Keep printed unknowns.
pub const TIME_ZONE_NAME: &[&str] = &[
    "manual",
    "UTC",
    "Europe/London",
    "Europe/Berlin",
    "Europe/Paris",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Amsterdam",
    "Europe/Brussels",
    "Europe/Vienna",
    "Europe/Warsaw",
    "Europe/Prague",
    "Europe/Budapest",
    "Europe/Athens",
    "Europe/Bucharest",
    "Europe/Helsinki",
    "Europe/Stockholm",
    "Europe/Oslo",
    "Europe/Copenhagen",
    "Europe/Riga",
    "Europe/Vilnius",
    "Europe/Tallinn",
    "Europe/Kiev",
    "Europe/Moscow",
    "Europe/Istanbul",
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "America/Toronto",
    "America/Sao_Paulo",
    "America/Mexico_City",
    "Asia/Jerusalem",
    "Asia/Dubai",
    "Asia/Kolkata",
    "Asia/Bangkok",
    "Asia/Singapore",
    "Asia/Hong_Kong",
    "Asia/Shanghai",
    "Asia/Tokyo",
    "Asia/Seoul",
    "Australia/Sydney",
    "Australia/Melbourne",
    "Pacific/Auckland",
    "Africa/Johannesburg",
    "Africa/Cairo",
];
pub const KIND_TIME_ZONE_NAME: FieldKind = FieldKind::Enum {
    values: TIME_ZONE_NAME,
};

const fn choice(label: &'static str, value: &'static str) -> EnumChoice {
    EnumChoice { label, value }
}

const fn spec(key: &'static str, label: &'static str, kind: FieldKind) -> FieldSpec {
    FieldSpec { key, label, kind }
}
