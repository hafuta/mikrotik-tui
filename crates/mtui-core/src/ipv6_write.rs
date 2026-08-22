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

const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const INTERFACE: FieldSpec = f!("interface", "Interface", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const CHAIN: FieldSpec = f!("chain", "Chain", FieldKind::Text);
const ACTION: FieldSpec = f!("action", "Action", FieldKind::Text);

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
                f!("routing-table", "Routing table", FieldKind::Text),
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
                f!("in-interface", "In interface", FieldKind::Text),
                f!("out-interface", "Out interface", FieldKind::Text),
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
