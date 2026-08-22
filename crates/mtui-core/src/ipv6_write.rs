//! Form schemas for the IPv6 nav group.
//!
//! Catalog wiring (do not register here):
//! - `ipv6-addresses` → `/rest/ipv6/address` (`IPV6_ADDRESS_FORM`)
//! - `ipv6-neighbors` → `/rest/ipv6/neighbor` (`IPV6_NEIGHBOR_FORM`)
//! - `ipv6-nd` → `/rest/ipv6/nd` (`IPV6_ND_FORM`)
//! - `ipv6-routes` → `/rest/ipv6/route` (`IPV6_ROUTE_FORM`)
//! - `ipv6-pool` → `/rest/ipv6/pool` (`IPV6_POOL_FORM`)
//! - `ipv6-settings` → `/rest/ipv6/settings` (`IPV6_SETTINGS_FORM`)
//! - `ipv6-firewall-filter` → `/rest/ipv6/firewall/filter` (`IPV6_FIREWALL_FILTER_FORM`)
//! - `ipv6-dhcp-client` → `/rest/ipv6/dhcp-client` (`IPV6_DHCP_CLIENT_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-dhcp-server` → `/rest/ipv6/dhcp-server` (`IPV6_DHCP_SERVER_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-nd-prefix` → `/rest/ipv6/nd/prefix` (`IPV6_ND_PREFIX_FORM`, `MEMBER_ACTIONS`)
//! - `ipv6-firewall-nat` → `/rest/ipv6/firewall/nat` (`IPV6_FIREWALL_NAT_FORM`, `FILTER_ACTIONS`)
//! - `ipv6-address-list` → `/rest/ipv6/firewall/address-list` (`IPV6_ADDRESS_LIST_FORM`, `MEMBER_ACTIONS`)
//!
//! Group id: `ipv6-group`.

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

const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IFACE_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "interface-lists",
    value_key: "name",
    multiple: false,
};
const LOOKUP_ROUTING_TABLE: FieldKind = FieldKind::Lookup {
    resource_id: "routing-tables",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IPV6_POOL: FieldKind = FieldKind::Lookup {
    resource_id: "ipv6-pool",
    value_key: "name",
    multiple: false,
};
const LOOKUP_IPV6_ADDRESS_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "ipv6-address-list",
    value_key: "list",
    multiple: false,
};

const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const CHAIN: FieldSpec = f!("chain", "Chain", FieldKind::Text);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);
const IN_INTERFACE: FieldSpec = f!("in-interface", "In interface", LOOKUP_IFACE);
const OUT_INTERFACE: FieldSpec = f!("out-interface", "Out interface", LOOKUP_IFACE);
const IN_INTERFACE_LIST: FieldSpec =
    f!("in-interface-list", "In interface list", LOOKUP_IFACE_LIST);
const OUT_INTERFACE_LIST: FieldSpec = f!(
    "out-interface-list",
    "Out interface list",
    LOOKUP_IFACE_LIST
);
const ROUTING_TABLE: FieldSpec = f!("routing-table", "Routing table", LOOKUP_ROUTING_TABLE);
const NAT_CHAIN: FieldSpec = f!(
    "chain",
    "Chain",
    FieldKind::Enum {
        values: &["srcnat", "dstnat"],
    }
);
const ADDRESS_POOL: FieldSpec = f!("address-pool", "Address pool", LOOKUP_IPV6_POOL);
const SRC_ADDRESS: FieldSpec = f!("src-address", "Src address", FieldKind::Text);
const DST_ADDRESS: FieldSpec = f!("dst-address", "Dst address", FieldKind::Text);
const SRC_ADDRESS_LIST: FieldSpec = f!(
    "src-address-list",
    "Src address list",
    LOOKUP_IPV6_ADDRESS_LIST
);
const DST_ADDRESS_LIST: FieldSpec = f!(
    "dst-address-list",
    "Dst address list",
    LOOKUP_IPV6_ADDRESS_LIST
);
const ADDRESS_LIST_NAME: FieldSpec = f!("list", "List", LOOKUP_IPV6_ADDRESS_LIST);
const PREFIX: FieldSpec = f!("prefix", "Prefix", FieldKind::Text);

pub static IPV6_ADDRESS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                INTERFACE,
                f!("advertise", "Advertise", FieldKind::Toggle),
                f!("eui-64", "EUI-64", FieldKind::Toggle),
                f!("no-dad", "No DAD", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("actual-interface", "Actual interface", FieldKind::Readonly),
                f!("from-pool", "From pool", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
                f!("slave", "Slave", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS, INTERFACE],
    }],
};

pub static IPV6_NEIGHBOR_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                INTERFACE,
                f!("mac-address", "MAC address", FieldKind::Text),
                COMMENT,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("origin", "Origin", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ADDRESS,
            INTERFACE,
            f!("mac-address", "MAC address", FieldKind::Text),
        ],
    }],
};

pub static IPV6_ND_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            INTERFACE,
            f!("ra-interval", "RA interval", FieldKind::Text),
            f!("ra-delay", "RA delay", FieldKind::Text),
            f!("mtu", "MTU", FieldKind::Text),
            f!("advertise-mac-address", "Advertise MAC", FieldKind::Toggle),
            f!("advertise-dns", "Advertise DNS", FieldKind::Toggle),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[INTERFACE],
    }],
};

pub static IPV6_ROUTE_FORM: FormSchema = FormSchema {
    title_key: "dst-address",
    subtitle_keys: &["gateway"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("dst-address", "Dst address", FieldKind::Text),
                f!("gateway", "Gateway", FieldKind::Text),
                f!("distance", "Distance", FieldKind::Text),
                ROUTING_TABLE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("active", "Active", FieldKind::Readonly),
                f!("static", "Static", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("dst-address", "Dst address", FieldKind::Text),
            f!("gateway", "Gateway", FieldKind::Text),
        ],
    }],
};

pub static IPV6_POOL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["prefix"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("prefix", "Prefix", FieldKind::Text),
            f!("prefix-length", "Prefix length", FieldKind::Number),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, f!("prefix", "Prefix", FieldKind::Text)],
    }],
};

pub static IPV6_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "forward",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("forward", "Forward", FieldKind::Toggle),
            f!("accept-redirects", "Accept redirects", FieldKind::Text),
            f!("max-neighbor-entries", "Max neighbors", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};

pub static IPV6_FIREWALL_FILTER_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                CHAIN,
                ACTION,
                f!("src-address", "Src address", FieldKind::Text),
                f!("dst-address", "Dst address", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
                IN_INTERFACE,
                OUT_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE_LIST,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[CHAIN, ACTION],
    }],
};

pub static IPV6_DHCP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["status"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                INTERFACE,
                f!("pool-name", "Pool name", FieldKind::Text),
                f!("request", "Request", FieldKind::Text),
                f!("add-default-route", "Default route", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("prefix", "Prefix", FieldKind::Readonly),
                f!("expires-after", "Expires after", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[INTERFACE],
    }],
};

pub static IPV6_DHCP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            INTERFACE,
            ADDRESS_POOL,
            f!("lease-time", "Lease time", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE],
    }],
};

pub static IPV6_ND_PREFIX_FORM: FormSchema = FormSchema {
    title_key: "prefix",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            PREFIX,
            INTERFACE,
            f!("advertise", "Advertise", FieldKind::Toggle),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[PREFIX, INTERFACE],
    }],
};

pub static IPV6_FIREWALL_NAT_FORM: FormSchema = FormSchema {
    title_key: "chain",
    subtitle_keys: &["action"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAT_CHAIN,
                ACTION,
                f!("protocol", "Protocol", FieldKind::Text),
                SRC_ADDRESS,
                SRC_ADDRESS_LIST,
                DST_ADDRESS,
                DST_ADDRESS_LIST,
                f!("dst-port", "Dst port", FieldKind::Text),
                f!("to-addresses", "To addresses", FieldKind::Text),
                f!("to-ports", "To ports", FieldKind::Text),
                IN_INTERFACE,
                IN_INTERFACE_LIST,
                OUT_INTERFACE,
                OUT_INTERFACE_LIST,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("packets", "Packets", FieldKind::Readonly),
                f!("bytes", "Bytes", FieldKind::Readonly),
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAT_CHAIN, ACTION],
    }],
};

pub static IPV6_ADDRESS_LIST_FORM: FormSchema = FormSchema {
    title_key: "list",
    subtitle_keys: &["address"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS_LIST_NAME,
                ADDRESS,
                f!("timeout", "Timeout", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("creation-time", "Creation time", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[ADDRESS_LIST_NAME, ADDRESS],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn assert_lookup(
        schema: &FormSchema,
        key: &str,
        resource_id: &'static str,
        value_key: &'static str,
    ) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Lookup {
                resource_id,
                value_key,
                multiple: false,
            })
        );
    }

    #[test]
    fn ipv6_address_matches_webfig() {
        assert_eq!(
            IPV6_ADDRESS_FORM.writable_keys(),
            [
                "address",
                "interface",
                "advertise",
                "eui-64",
                "no-dad",
                "comment",
                "disabled",
            ]
        );
        assert_eq!(create_keys(&IPV6_ADDRESS_FORM), ["address", "interface"]);
        assert_lookup(&IPV6_ADDRESS_FORM, "interface", "interfaces", "name");
        assert_lookup(&IPV6_NEIGHBOR_FORM, "interface", "interfaces", "name");
        assert_lookup(&IPV6_ND_FORM, "interface", "interfaces", "name");
        assert!(
            IPV6_ADDRESS_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        assert!(!IPV6_ADDRESS_FORM.writable_keys().contains(&"from-pool"));
    }

    #[test]
    fn ipv6_neighbor_optional_edit_and_create() {
        assert_eq!(
            IPV6_NEIGHBOR_FORM.writable_keys(),
            ["address", "interface", "mac-address", "comment"]
        );
        assert_eq!(
            create_keys(&IPV6_NEIGHBOR_FORM),
            ["address", "interface", "mac-address"]
        );
        assert!(!IPV6_NEIGHBOR_FORM.writable_keys().contains(&"origin"));
    }

    #[test]
    fn ipv6_nd_create_is_interface_only() {
        assert_eq!(create_keys(&IPV6_ND_FORM), ["interface"]);
        assert!(IPV6_ND_FORM.writable_keys().contains(&"advertise-dns"));
        assert!(IPV6_ND_FORM.writable_keys().contains(&"ra-interval"));
    }

    #[test]
    fn ipv6_route_status_is_readonly() {
        assert_eq!(create_keys(&IPV6_ROUTE_FORM), ["dst-address", "gateway"]);
        assert!(!IPV6_ROUTE_FORM.writable_keys().contains(&"active"));
        assert!(IPV6_ROUTE_FORM.writable_keys().contains(&"routing-table"));
        assert_lookup(&IPV6_ROUTE_FORM, "routing-table", "routing-tables", "name");
    }

    #[test]
    fn ipv6_pool_and_settings() {
        assert_eq!(create_keys(&IPV6_POOL_FORM), ["name", "prefix"]);
        assert!(IPV6_POOL_FORM.writable_keys().contains(&"prefix-length"));
        assert!(IPV6_SETTINGS_FORM.create_sections.is_empty());
        assert_eq!(
            IPV6_SETTINGS_FORM.writable_keys(),
            ["forward", "accept-redirects", "max-neighbor-entries"]
        );
    }

    #[test]
    fn ipv6_firewall_filter_like_ipv4() {
        assert_eq!(create_keys(&IPV6_FIREWALL_FILTER_FORM), ["chain", "action"]);
        assert!(
            !IPV6_FIREWALL_FILTER_FORM
                .writable_keys()
                .contains(&"packets")
        );
        assert!(IPV6_FIREWALL_FILTER_FORM.known_keys().contains(&"invalid"));
        assert!(
            IPV6_FIREWALL_FILTER_FORM
                .writable_keys()
                .contains(&"in-interface-list")
        );
        assert!(
            IPV6_FIREWALL_FILTER_FORM
                .writable_keys()
                .contains(&"out-interface-list")
        );
        assert_lookup(
            &IPV6_FIREWALL_FILTER_FORM,
            "in-interface",
            "interfaces",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_FILTER_FORM,
            "out-interface",
            "interfaces",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_FILTER_FORM,
            "in-interface-list",
            "interface-lists",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_FILTER_FORM,
            "out-interface-list",
            "interface-lists",
            "name",
        );
        let general = IPV6_FIREWALL_FILTER_FORM
            .sections
            .iter()
            .find(|section| section.id == "general")
            .expect("general");
        assert!(
            general
                .fields
                .iter()
                .any(|field| field.key == "in-interface-list")
        );
        assert!(
            general
                .fields
                .iter()
                .any(|field| field.key == "out-interface-list")
        );
        assert!(
            IPV6_FIREWALL_FILTER_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn ipv6_operator_create_keys_and_lookups() {
        assert_eq!(create_keys(&IPV6_DHCP_CLIENT_FORM), ["interface"]);
        assert_eq!(create_keys(&IPV6_DHCP_SERVER_FORM), ["name", "interface"]);
        assert_eq!(create_keys(&IPV6_ND_PREFIX_FORM), ["prefix", "interface"]);
        assert_eq!(create_keys(&IPV6_FIREWALL_NAT_FORM), ["chain", "action"]);
        assert_eq!(create_keys(&IPV6_ADDRESS_LIST_FORM), ["list", "address"]);

        assert_lookup(&IPV6_DHCP_CLIENT_FORM, "interface", "interfaces", "name");
        assert_lookup(&IPV6_DHCP_SERVER_FORM, "interface", "interfaces", "name");
        assert_lookup(&IPV6_DHCP_SERVER_FORM, "address-pool", "ipv6-pool", "name");
        assert_lookup(&IPV6_ND_PREFIX_FORM, "interface", "interfaces", "name");
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "in-interface",
            "interfaces",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "out-interface",
            "interfaces",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "in-interface-list",
            "interface-lists",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "out-interface-list",
            "interface-lists",
            "name",
        );
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "src-address-list",
            "ipv6-address-list",
            "list",
        );
        assert_lookup(
            &IPV6_FIREWALL_NAT_FORM,
            "dst-address-list",
            "ipv6-address-list",
            "list",
        );
        assert_lookup(&IPV6_ADDRESS_LIST_FORM, "list", "ipv6-address-list", "list");
        assert_eq!(
            IPV6_FIREWALL_NAT_FORM
                .field("chain")
                .map(|field| field.kind),
            Some(FieldKind::Enum {
                values: &["srcnat", "dstnat"],
            })
        );
        assert!(
            IPV6_FIREWALL_NAT_FORM
                .writable_keys()
                .contains(&"to-addresses")
        );
        assert!(!IPV6_DHCP_CLIENT_FORM.writable_keys().contains(&"status"));
        assert!(!IPV6_DHCP_CLIENT_FORM.writable_keys().contains(&"prefix"));
        assert!(
            !IPV6_DHCP_CLIENT_FORM
                .writable_keys()
                .contains(&"expires-after")
        );
        assert!(!IPV6_ADDRESS_LIST_FORM.writable_keys().contains(&"dynamic"));
    }

    #[test]
    fn unknown_ipv6_keys_land_on_status_extras() {
        let mut row = HashMap::new();
        row.insert("address".into(), "2001:db8::1/64".into());
        row.insert("link-local".into(), "true".into());
        let extras = extra_status_fields(&IPV6_ADDRESS_FORM, &row);
        assert_eq!(extras, vec![("link-local".into(), "true".into())]);
    }

    #[test]
    fn patch_body_skips_readonly_route_flags() {
        let mut original = HashMap::new();
        original.insert("dst-address".into(), "2001:db8::/32".into());
        original.insert("gateway".into(), "fe80::1".into());
        original.insert("active".into(), "true".into());
        let mut current = original.clone();
        current.insert("gateway".into(), "fe80::2".into());
        current.insert("active".into(), "false".into());
        let body = patch_body(&IPV6_ROUTE_FORM, &original, &current, "********");
        assert_eq!(body.get("gateway").map(String::as_str), Some("fe80::2"));
        assert!(!body.contains_key("active"));
    }
}
