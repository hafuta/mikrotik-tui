use crate::features::ipv6::forms::*;
use crate::form_fields::{
    KIND_ADVERTISE_DNS, KIND_FILTER_ACTION, KIND_FILTER_CHAIN, KIND_IP_PROTOCOL,
    KIND_IPV6_ACCEPT_REDIRECTS, KIND_MANGLE_ACTION, KIND_MANGLE_CHAIN, KIND_NAT_ACTION,
    KIND_NAT_CHAIN, KIND_RAW_ACTION, KIND_RAW_CHAIN,
};
use crate::forms::{FieldKind, FormSchema, extra_status_fields, patch_body};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &IPV6_ADDRESS_FORM,
    &IPV6_NEIGHBOR_FORM,
    &IPV6_ND_FORM,
    &IPV6_ROUTE_FORM,
    &IPV6_POOL_FORM,
    &IPV6_SETTINGS_FORM,
    &IPV6_FIREWALL_FILTER_FORM,
    &IPV6_DHCP_CLIENT_FORM,
    &IPV6_DHCP_SERVER_FORM,
    &IPV6_ND_PREFIX_FORM,
    &IPV6_FIREWALL_NAT_FORM,
    &IPV6_ADDRESS_LIST_FORM,
    &IPV6_DHCP_RELAY_FORM,
    &IPV6_DHCP_BINDING_FORM,
    &IPV6_FIREWALL_MANGLE_FORM,
    &IPV6_FIREWALL_RAW_FORM,
];

const ENABLED_FORMS: &[&FormSchema] = &[
    &IPV6_ADDRESS_FORM,
    &IPV6_ND_FORM,
    &IPV6_ROUTE_FORM,
    &IPV6_FIREWALL_FILTER_FORM,
    &IPV6_DHCP_CLIENT_FORM,
    &IPV6_DHCP_SERVER_FORM,
    &IPV6_ND_PREFIX_FORM,
    &IPV6_FIREWALL_NAT_FORM,
    &IPV6_ADDRESS_LIST_FORM,
    &IPV6_DHCP_RELAY_FORM,
    &IPV6_DHCP_BINDING_FORM,
    &IPV6_FIREWALL_MANGLE_FORM,
    &IPV6_FIREWALL_RAW_FORM,
];

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
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
fn create_matches_writable_sheet_and_omits_status() {
    for form in FORMS {
        assert_eq!(create_keys(form), form.writable_keys());
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| section.id != "status" && !section.hidden_on_create())
        );
    }
    assert!(IPV6_SETTINGS_FORM.create_sections.is_empty());
}

#[test]
fn disabled_fields_are_enabled_inverted_toggles() {
    for form in ENABLED_FORMS {
        assert_eq!(
            form.field("disabled").map(|field| field.label),
            Some("Enabled")
        );
        assert_eq!(
            form.field("disabled").map(|field| field.kind),
            Some(FieldKind::InvertedToggle)
        );
    }
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
    assert_eq!(
        create_keys(&IPV6_ADDRESS_FORM),
        IPV6_ADDRESS_FORM.writable_keys()
    );
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
        IPV6_NEIGHBOR_FORM.writable_keys()
    );
    assert!(!IPV6_NEIGHBOR_FORM.writable_keys().contains(&"origin"));
}

#[test]
fn ipv6_nd_create_is_full_writable_sheet() {
    assert_eq!(create_keys(&IPV6_ND_FORM), IPV6_ND_FORM.writable_keys());
    assert!(IPV6_ND_FORM.writable_keys().contains(&"advertise-dns"));
    assert!(IPV6_ND_FORM.writable_keys().contains(&"ra-interval"));
    assert_eq!(
        IPV6_ND_FORM.field("advertise-dns").map(|field| field.kind),
        Some(KIND_ADVERTISE_DNS)
    );
}

#[test]
fn ipv6_route_status_is_readonly() {
    assert_eq!(
        create_keys(&IPV6_ROUTE_FORM),
        IPV6_ROUTE_FORM.writable_keys()
    );
    assert!(!IPV6_ROUTE_FORM.writable_keys().contains(&"active"));
    assert!(IPV6_ROUTE_FORM.writable_keys().contains(&"routing-table"));
    assert_lookup(&IPV6_ROUTE_FORM, "routing-table", "routing-tables", "name");
}

#[test]
fn ipv6_pool_and_settings() {
    assert_eq!(create_keys(&IPV6_POOL_FORM), IPV6_POOL_FORM.writable_keys());
    assert!(IPV6_POOL_FORM.writable_keys().contains(&"prefix-length"));
    assert!(IPV6_SETTINGS_FORM.create_sections.is_empty());
    assert_eq!(
        IPV6_SETTINGS_FORM.writable_keys(),
        ["forward", "accept-redirects", "max-neighbor-entries"]
    );
    assert_eq!(
        IPV6_SETTINGS_FORM
            .field("accept-redirects")
            .map(|field| field.kind),
        Some(KIND_IPV6_ACCEPT_REDIRECTS)
    );
}

#[test]
fn ipv6_firewall_filter_like_ipv4() {
    assert_eq!(
        create_keys(&IPV6_FIREWALL_FILTER_FORM),
        IPV6_FIREWALL_FILTER_FORM.writable_keys()
    );
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
    assert_eq!(
        IPV6_FIREWALL_FILTER_FORM
            .field("chain")
            .map(|field| field.kind),
        Some(KIND_FILTER_CHAIN)
    );
    assert_eq!(
        IPV6_FIREWALL_FILTER_FORM
            .field("action")
            .map(|field| field.kind),
        Some(KIND_FILTER_ACTION)
    );
    assert_eq!(
        IPV6_FIREWALL_FILTER_FORM
            .field("protocol")
            .map(|field| field.kind),
        Some(KIND_IP_PROTOCOL)
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
    assert_eq!(
        create_keys(&IPV6_DHCP_CLIENT_FORM),
        IPV6_DHCP_CLIENT_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&IPV6_DHCP_SERVER_FORM),
        IPV6_DHCP_SERVER_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&IPV6_ND_PREFIX_FORM),
        IPV6_ND_PREFIX_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&IPV6_FIREWALL_NAT_FORM),
        IPV6_FIREWALL_NAT_FORM.writable_keys()
    );
    assert_eq!(
        create_keys(&IPV6_ADDRESS_LIST_FORM),
        IPV6_ADDRESS_LIST_FORM.writable_keys()
    );

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
    assert_lookup(
        &IPV6_DHCP_BINDING_FORM,
        "server",
        "ipv6-dhcp-server",
        "name",
    );
    assert_eq!(
        IPV6_DHCP_RELAY_FORM
            .field("dhcp-server")
            .map(|field| field.kind),
        Some(FieldKind::Text)
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
fn ipv6_firewall_uses_shared_chain_action_kinds() {
    assert_eq!(
        IPV6_FIREWALL_NAT_FORM
            .field("chain")
            .map(|field| field.kind),
        Some(KIND_NAT_CHAIN)
    );
    assert_eq!(
        IPV6_FIREWALL_NAT_FORM
            .field("action")
            .map(|field| field.kind),
        Some(KIND_NAT_ACTION)
    );
    assert_eq!(
        IPV6_FIREWALL_NAT_FORM
            .field("protocol")
            .map(|field| field.kind),
        Some(KIND_IP_PROTOCOL)
    );
    assert_eq!(
        IPV6_FIREWALL_MANGLE_FORM
            .field("chain")
            .map(|field| field.kind),
        Some(KIND_MANGLE_CHAIN)
    );
    assert_eq!(
        IPV6_FIREWALL_MANGLE_FORM
            .field("action")
            .map(|field| field.kind),
        Some(KIND_MANGLE_ACTION)
    );
    assert_eq!(
        IPV6_FIREWALL_MANGLE_FORM
            .field("protocol")
            .map(|field| field.kind),
        Some(KIND_IP_PROTOCOL)
    );
    assert_eq!(
        IPV6_FIREWALL_RAW_FORM
            .field("chain")
            .map(|field| field.kind),
        Some(KIND_RAW_CHAIN)
    );
    assert_eq!(
        IPV6_FIREWALL_RAW_FORM
            .field("action")
            .map(|field| field.kind),
        Some(KIND_RAW_ACTION)
    );
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

#[test]
fn ipv6_firewall_connections_have_no_field_sheet() {
    assert!(
        crate::features::ipv6::resources::IPV6_FIREWALL_CONNECTIONS
            .form
            .is_none(),
        "conntrack is inspect/remove only; do not ship a Text sheet"
    );
    assert!(
        crate::features::ipv6::resources::IPV6_FIREWALL_CONNECTIONS
            .actions
            .iter()
            .all(|action| action.id == "remove")
    );
}
