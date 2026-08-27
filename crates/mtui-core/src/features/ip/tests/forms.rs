use crate::features::ip::forms::*;
use crate::features::ip::guides::GUIDES;
use crate::features::ip::resources::RESOURCES;
use crate::features::ip::rules::form_field_state;
use crate::forms::{FieldKind, FormSchema};
use std::collections::HashMap;

const FORMS: &[&FormSchema] = &[
    &ARP_FORM,
    &ADDRESS_FORM,
    &DHCP_SERVER_FORM,
    &DHCP_NETWORK_FORM,
    &DHCP_LEASE_FORM,
    &FIREWALL_FILTER_FORM,
    &DHCP_CLIENT_FORM,
    &DNS_FORM,
    &DNS_STATIC_FORM,
    &ROUTE_FORM,
    &POOL_FORM,
    &SERVICE_FORM,
    &IP_SETTINGS_FORM,
    &FIREWALL_NAT_FORM,
    &FIREWALL_MANGLE_FORM,
    &ADDRESS_LIST_FORM,
    &DHCP_RELAY_FORM,
    &DHCP_OPTION_FORM,
    &DHCP_OPTION_SET_FORM,
    &FIREWALL_RAW_FORM,
    &LAYER7_FORM,
    &SERVICE_PORT_FORM,
    &CLOUD_FORM,
    &KID_CONTROL_FORM,
    &KID_CONTROL_DEVICE_FORM,
    &SOCKS_FORM,
    &SMB_FORM,
    &SMB_SHARE_FORM,
    &SMB_USER_FORM,
    &UPNP_FORM,
    &UPNP_INTERFACE_FORM,
    &DHCP_ALERT_FORM,
    &CONNECTION_TRACKING_FORM,
    &NEIGHBOR_DISCOVERY_FORM,
    &IP_SSH_FORM,
    &TRAFFIC_FLOW_FORM,
    &TRAFFIC_FLOW_TARGET_FORM,
    &TRAFFIC_FLOW_IPFIX_FORM,
    &IGMP_PROXY_FORM,
    &IGMP_PROXY_INTERFACE_FORM,
    &IGMP_PROXY_MFC_FORM,
    &IPSEC_PEER_FORM,
    &IPSEC_IDENTITY_FORM,
    &IPSEC_POLICY_FORM,
    &IPSEC_PROPOSAL_FORM,
    &IPSEC_PROFILE_FORM,
    &IPSEC_SETTINGS_FORM,
    &IPSEC_MODE_CONFIG_FORM,
    &IPSEC_KEY_RSA_FORM,
    &IPSEC_KEY_PSK_FORM,
    &IPSEC_KEY_QKD_FORM,
    &HOTSPOT_FORM,
    &HOTSPOT_PROFILE_FORM,
    &HOTSPOT_USER_FORM,
    &HOTSPOT_HOST_FORM,
    &HOTSPOT_IP_BINDING_FORM,
    &HOTSPOT_WALLED_GARDEN_FORM,
    &HOTSPOT_WALLED_GARDEN_IP_FORM,
    &PROXY_FORM,
    &PROXY_ACCESS_FORM,
    &PROXY_CACHE_FORM,
    &PROXY_DIRECT_FORM,
];

fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
    schema.create_keys()
}

#[test]
fn create_keys_match_writable_and_omit_status() {
    for form in FORMS {
        assert_eq!(
            create_keys(form),
            form.writable_keys(),
            "{}",
            form.title_key
        );
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| !section.hidden_on_create())
        );
        assert!(
            form.sections_for(true)
                .iter()
                .all(|section| section.id != "status")
        );
    }
}

#[test]
fn disabled_uses_enabled_inverted_toggle() {
    let mut n = 0;
    for form in FORMS {
        if let Some(field) = form.field("disabled") {
            assert_eq!(field.label, "Enabled", "{}", form.title_key);
            assert_eq!(field.kind, FieldKind::InvertedToggle, "{}", form.title_key);
            n += 1;
        }
    }
    assert!(n >= 35, "expected every disabled key converted, got {n}");
}

#[test]
fn catalog_and_guides_cover_the_ip_group() {
    assert_eq!(RESOURCES.len(), 67);
    assert_eq!(GUIDES.len(), 67);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
    }
    assert!(form_field_state("arp", "disabled", &HashMap::new()).is_none());
}

#[test]
fn firewall_and_ipsec_policy_use_match_sections() {
    for form in [
        &FIREWALL_FILTER_FORM,
        &FIREWALL_NAT_FORM,
        &FIREWALL_MANGLE_FORM,
        &FIREWALL_RAW_FORM,
        &IPSEC_POLICY_FORM,
    ] {
        let match_tab = form
            .sections
            .iter()
            .find(|section| section.id == "match")
            .expect("match");
        assert_eq!(match_tab.label, "Match");
        assert!(!match_tab.read_only);
    }
    let peer_adv = IPSEC_PEER_FORM
        .sections
        .iter()
        .find(|section| section.id == "advanced")
        .expect("peer advanced");
    assert_eq!(peer_adv.label, "Advanced");
}

#[test]
fn smb_user_password_stays_secret() {
    assert_eq!(
        SMB_USER_FORM.field("password").map(|field| field.kind),
        Some(FieldKind::Secret)
    );
}

mod core {
    use super::*;

    use crate::forms::{extra_status_fields, field_visible, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema.create_keys()
    }

    fn status_readonly(schema: &FormSchema) {
        let status = schema
            .sections
            .iter()
            .find(|section| section.id == "status")
            .expect("status tab");
        assert!(status.read_only);
        for field in status.fields {
            assert!(!field.kind.writable(), "{}", field.key);
            assert!(!schema.writable_keys().contains(&field.key));
        }
    }

    #[test]
    fn create_sheets_match_writable_fields() {
        assert_eq!(create_keys(&ARP_FORM), ARP_FORM.writable_keys());
        assert_eq!(create_keys(&ADDRESS_FORM), ADDRESS_FORM.writable_keys());
        assert_eq!(
            create_keys(&DHCP_SERVER_FORM),
            DHCP_SERVER_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_NETWORK_FORM),
            DHCP_NETWORK_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_LEASE_FORM),
            DHCP_LEASE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&FIREWALL_FILTER_FORM),
            FIREWALL_FILTER_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_CLIENT_FORM),
            DHCP_CLIENT_FORM.writable_keys()
        );
        assert!(DNS_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&DNS_STATIC_FORM),
            DNS_STATIC_FORM.writable_keys()
        );
        assert_eq!(create_keys(&ROUTE_FORM), ROUTE_FORM.writable_keys());
        assert_eq!(create_keys(&POOL_FORM), POOL_FORM.writable_keys());
        assert!(SERVICE_FORM.create_sections.is_empty());
        assert!(TRAFFIC_FLOW_FORM.create_sections.is_empty());
        assert!(TRAFFIC_FLOW_IPFIX_FORM.create_sections.is_empty());
        assert!(IGMP_PROXY_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&TRAFFIC_FLOW_TARGET_FORM),
            TRAFFIC_FLOW_TARGET_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&IGMP_PROXY_INTERFACE_FORM),
            IGMP_PROXY_INTERFACE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&IGMP_PROXY_MFC_FORM),
            IGMP_PROXY_MFC_FORM.writable_keys()
        );
        assert!(IP_SETTINGS_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&FIREWALL_NAT_FORM),
            FIREWALL_NAT_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&FIREWALL_MANGLE_FORM),
            FIREWALL_MANGLE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&ADDRESS_LIST_FORM),
            ADDRESS_LIST_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_RELAY_FORM),
            DHCP_RELAY_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_OPTION_FORM),
            DHCP_OPTION_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&DHCP_OPTION_SET_FORM),
            DHCP_OPTION_SET_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&FIREWALL_RAW_FORM),
            FIREWALL_RAW_FORM.writable_keys()
        );
        assert_eq!(create_keys(&LAYER7_FORM), LAYER7_FORM.writable_keys());
        assert!(SERVICE_PORT_FORM.create_sections.is_empty());
    }

    #[test]
    fn firewall_status_is_readonly() {
        status_readonly(&FIREWALL_FILTER_FORM);
        status_readonly(&FIREWALL_NAT_FORM);
        status_readonly(&FIREWALL_MANGLE_FORM);
        status_readonly(&FIREWALL_RAW_FORM);
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"packets"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"bytes"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"dynamic"));
        assert!(!FIREWALL_FILTER_FORM.writable_keys().contains(&"invalid"));
    }

    #[test]
    fn patch_body_skips_status_and_unchanged() {
        let mut original = HashMap::new();
        original.insert("address".into(), "192.168.1.2/24".into());
        original.insert("interface".into(), "ether1".into());
        original.insert("comment".into(), "lan".into());
        original.insert("disabled".into(), "false".into());
        original.insert("network".into(), "192.168.1.0".into());
        original.insert("dynamic".into(), "false".into());
        let mut current = original.clone();
        current.insert("comment".into(), "office".into());
        current.insert("network".into(), "10.0.0.0".into());
        let body = patch_body(&ADDRESS_FORM, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("office"));
        assert!(!body.contains_key("network"));
        assert!(!body.contains_key("dynamic"));
        assert!(!body.contains_key("address"));
    }

    #[test]
    fn service_name_is_readonly() {
        // Disabling both api and api-ssl drops classic API access to this app.
        assert!(!SERVICE_FORM.writable_keys().contains(&"name"));
        assert!(SERVICE_FORM.writable_keys().contains(&"disabled"));
        status_readonly(&SERVICE_FORM);
    }

    fn field_kind(schema: &FormSchema, key: &str) -> FieldKind {
        schema
            .sections
            .iter()
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
            .unwrap_or_else(|| panic!("missing field {key}"))
            .kind
    }

    fn create_field_kind(schema: &FormSchema, key: &str) -> FieldKind {
        schema
            .sections_for(true)
            .iter()
            .flat_map(|section| section.fields)
            .find(|field| field.key == key)
            .unwrap_or_else(|| panic!("missing create field {key}"))
            .kind
    }

    fn lookup(resource_id: &'static str, value_key: &'static str) -> FieldKind {
        FieldKind::Lookup {
            resource_id,
            value_key,
            multiple: false,
        }
    }

    fn lookup_multi(resource_id: &'static str, value_key: &'static str) -> FieldKind {
        FieldKind::Lookup {
            resource_id,
            value_key,
            multiple: true,
        }
    }

    #[test]
    fn lookup_fields_use_named_resources() {
        let interfaces = lookup("interfaces", "name");
        let pools = lookup("pools", "name");
        let address_list_names = lookup("address-list", "list");
        let interface_lists = lookup("interface-lists", "name");

        for schema in [
            &ARP_FORM,
            &ADDRESS_FORM,
            &DHCP_CLIENT_FORM,
            &DHCP_SERVER_FORM,
            &DHCP_RELAY_FORM,
        ] {
            assert_eq!(field_kind(schema, "interface"), interfaces);
            assert_eq!(create_field_kind(schema, "interface"), interfaces);
        }

        assert_eq!(field_kind(&DHCP_SERVER_FORM, "address-pool"), pools);
        assert_eq!(create_field_kind(&DHCP_SERVER_FORM, "address-pool"), pools);

        assert_eq!(
            field_kind(&DHCP_LEASE_FORM, "server"),
            lookup("dhcp-servers", "name")
        );

        assert_eq!(field_kind(&POOL_FORM, "next-pool"), pools);
        assert_eq!(
            field_kind(&ROUTE_FORM, "routing-table"),
            lookup("routing-tables", "name")
        );
        assert_eq!(
            field_kind(&SERVICE_FORM, "certificate"),
            lookup("certificates", "name")
        );

        assert_eq!(field_kind(&ADDRESS_LIST_FORM, "list"), address_list_names);
        assert_eq!(
            create_field_kind(&ADDRESS_LIST_FORM, "list"),
            address_list_names
        );

        for schema in [
            &FIREWALL_FILTER_FORM,
            &FIREWALL_NAT_FORM,
            &FIREWALL_MANGLE_FORM,
            &FIREWALL_RAW_FORM,
        ] {
            assert_eq!(field_kind(schema, "in-interface"), interfaces);
            assert_eq!(field_kind(schema, "out-interface"), interfaces);
            assert_eq!(field_kind(schema, "in-interface-list"), interface_lists);
            assert_eq!(field_kind(schema, "out-interface-list"), interface_lists);
            assert_eq!(field_kind(schema, "src-address-list"), address_list_names);
            assert_eq!(field_kind(schema, "dst-address-list"), address_list_names);
            assert!(create_keys(schema).contains(&"in-interface"));
            assert!(create_keys(schema).contains(&"src-address-list"));
        }

        assert_eq!(
            field_kind(&DHCP_OPTION_SET_FORM, "options"),
            lookup_multi("dhcp-options", "name")
        );
        assert_eq!(field_kind(&DHCP_OPTION_FORM, "code"), FieldKind::Number);
        assert_eq!(
            create_field_kind(&DHCP_OPTION_FORM, "code"),
            FieldKind::Number
        );
        assert!(!SERVICE_PORT_FORM.writable_keys().contains(&"name"));
        assert!(SERVICE_PORT_FORM.writable_keys().contains(&"disabled"));
    }

    fn no_advanced(schema: &FormSchema) -> bool {
        schema
            .sections
            .iter()
            .all(|section| section.id != "advanced")
    }

    fn assert_enum(schema: &FormSchema, key: &str, values: &'static [&'static str]) {
        assert_eq!(
            schema.field(key).map(|field| field.kind),
            Some(FieldKind::Enum { values })
        );
    }

    fn assert_label(schema: &FormSchema, key: &str, label: &str) {
        assert_eq!(schema.field(key).map(|field| field.label), Some(label));
    }

    #[test]
    fn traffic_flow_forms_use_webfig_field_kinds() {
        assert!(TRAFFIC_FLOW_FORM.create_sections.is_empty());
        assert!(no_advanced(&TRAFFIC_FLOW_FORM));
        assert_eq!(
            TRAFFIC_FLOW_FORM.writable_keys(),
            [
                "enabled",
                "interfaces",
                "cache-entries",
                "active-flow-timeout",
                "inactive-flow-timeout",
                "packet-sampling",
                "sampling-interval",
                "sampling-space",
            ]
        );
        assert_eq!(field_kind(&TRAFFIC_FLOW_FORM, "enabled"), FieldKind::Toggle);
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "interfaces"),
            FieldKind::Repeat
        );
        assert_enum(&TRAFFIC_FLOW_FORM, "cache-entries", CACHE_ENTRIES);
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "active-flow-timeout"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "inactive-flow-timeout"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "packet-sampling"),
            FieldKind::Toggle
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "sampling-interval"),
            FieldKind::Number
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_FORM, "sampling-space"),
            FieldKind::Number
        );
        assert_label(&TRAFFIC_FLOW_FORM, "cache-entries", "Cache Entries");
        assert_label(
            &TRAFFIC_FLOW_FORM,
            "active-flow-timeout",
            "Active Flow Timeout",
        );
        assert_label(&TRAFFIC_FLOW_FORM, "packet-sampling", "Packet Sampling");
    }

    #[test]
    fn traffic_flow_target_and_ipfix_use_webfig_field_kinds() {
        assert_eq!(
            create_keys(&TRAFFIC_FLOW_TARGET_FORM),
            TRAFFIC_FLOW_TARGET_FORM.writable_keys()
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "src-address"),
            FieldKind::Ip
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "dst-address"),
            FieldKind::Ip
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "port"),
            FieldKind::Number
        );
        assert_enum(&TRAFFIC_FLOW_TARGET_FORM, "version", TRAFFIC_FLOW_VERSIONS);
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "v9-template-refresh"),
            FieldKind::Number
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "v9-template-timeout"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_TARGET_FORM, "disabled"),
            FieldKind::InvertedToggle
        );
        assert_label(&TRAFFIC_FLOW_TARGET_FORM, "src-address", "Src. Address");
        assert_label(
            &TRAFFIC_FLOW_TARGET_FORM,
            "v9-template-refresh",
            "v9 Template Refresh",
        );

        assert!(TRAFFIC_FLOW_IPFIX_FORM.create_sections.is_empty());
        assert!(no_advanced(&TRAFFIC_FLOW_IPFIX_FORM));
        for field in IPFIX_GENERAL {
            assert_eq!(field.kind, FieldKind::Toggle, "{}", field.key);
            assert_ne!(field.kind, FieldKind::Text, "{}", field.key);
            assert_ne!(field.kind, FieldKind::Number, "{}", field.key);
        }
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_IPFIX_FORM, "src-port"),
            FieldKind::Toggle
        );
        assert_eq!(
            field_kind(&TRAFFIC_FLOW_IPFIX_FORM, "bytes"),
            FieldKind::Toggle
        );
        assert_label(
            &TRAFFIC_FLOW_IPFIX_FORM,
            "ip-total-length",
            "IP Total Length",
        );
    }

    #[test]
    fn igmp_proxy_forms_use_webfig_field_kinds() {
        assert!(IGMP_PROXY_FORM.create_sections.is_empty());
        assert!(no_advanced(&IGMP_PROXY_FORM));
        assert_eq!(
            IGMP_PROXY_FORM.writable_keys(),
            [
                "query-interval",
                "query-response-interval",
                "last-member-query-interval",
                "robustness",
                "quick-leave",
            ]
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_FORM, "query-interval"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_FORM, "query-response-interval"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_FORM, "last-member-query-interval"),
            FieldKind::Time
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_FORM, "robustness"),
            FieldKind::Number
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_FORM, "quick-leave"),
            FieldKind::Toggle
        );
        assert_label(&IGMP_PROXY_FORM, "quick-leave", "Quick Leave");
        assert_label(
            &IGMP_PROXY_FORM,
            "last-member-query-interval",
            "Last Member Query Interval",
        );

        assert_eq!(
            create_keys(&IGMP_PROXY_INTERFACE_FORM),
            IGMP_PROXY_INTERFACE_FORM.writable_keys()
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_INTERFACE_FORM, "interface"),
            lookup("interfaces", "name")
        );
        assert_eq!(
            create_field_kind(&IGMP_PROXY_INTERFACE_FORM, "interface"),
            lookup("interfaces", "name")
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_INTERFACE_FORM, "upstream"),
            FieldKind::Toggle
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_INTERFACE_FORM, "threshold"),
            FieldKind::Number
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_INTERFACE_FORM, "alternative-subnets"),
            FieldKind::Repeat
        );
        assert_label(
            &IGMP_PROXY_INTERFACE_FORM,
            "alternative-subnets",
            "Alternative Subnets",
        );
        status_readonly(&IGMP_PROXY_INTERFACE_FORM);

        assert_eq!(
            create_keys(&IGMP_PROXY_MFC_FORM),
            IGMP_PROXY_MFC_FORM.writable_keys()
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_MFC_FORM, "upstream-interface"),
            lookup("interfaces", "name")
        );
        assert_eq!(
            field_kind(&IGMP_PROXY_MFC_FORM, "downstream-interfaces"),
            lookup_multi("interfaces", "name")
        );
        assert_eq!(field_kind(&IGMP_PROXY_MFC_FORM, "group"), FieldKind::Ip);
        status_readonly(&IGMP_PROXY_MFC_FORM);
        assert!(!IGMP_PROXY_MFC_FORM.writable_keys().contains(&"packets"));
        assert!(!IGMP_PROXY_MFC_FORM.writable_keys().contains(&"bytes"));
    }

    #[test]
    fn traffic_flow_and_igmp_patch_skips_status_and_unchanged() {
        let mut original = HashMap::new();
        original.insert("interface".into(), "ether2".into());
        original.insert("upstream".into(), "true".into());
        original.insert("threshold".into(), "1".into());
        original.insert("alternative-subnets".into(), "192.168.50.0/24".into());
        original.insert("querier".into(), "yes".into());
        original.insert("rx-bytes".into(), "100".into());
        let mut current = original.clone();
        current.insert("threshold".into(), "2".into());
        current.insert("rx-bytes".into(), "999".into());
        let body = patch_body(&IGMP_PROXY_INTERFACE_FORM, &original, &current, "********");
        assert_eq!(body.get("threshold").map(String::as_str), Some("2"));
        assert!(!body.contains_key("querier"));
        assert!(!body.contains_key("rx-bytes"));
        assert!(!body.contains_key("interface"));

        let mut original = HashMap::new();
        original.insert("enabled".into(), "false".into());
        original.insert("interfaces".into(), "all".into());
        original.insert("cache-entries".into(), "4k".into());
        original.insert("packet-sampling".into(), "false".into());
        let mut current = original.clone();
        current.insert("enabled".into(), "true".into());
        current.insert("interfaces".into(), "ether1,ether2".into());
        let body = patch_body(&TRAFFIC_FLOW_FORM, &original, &current, "********");
        assert_eq!(body.get("enabled").map(String::as_str), Some("true"));
        assert_eq!(
            body.get("interfaces").map(String::as_str),
            Some("ether1,ether2")
        );
        assert!(!body.contains_key("cache-entries"));
        assert!(!body.contains_key("packet-sampling"));
    }

    #[test]
    fn smb_shares_and_users_match_webfig_kinds() {
        assert_eq!(
            SMB_SHARE_FORM.writable_keys(),
            [
                "name",
                "directory",
                "comment",
                "valid-users",
                "invalid-users",
                "read-only",
                "require-encryption",
                "disabled",
            ]
        );
        assert_eq!(
            SMB_USER_FORM.writable_keys(),
            ["name", "password", "comment", "read-only", "disabled"]
        );
        assert_eq!(
            field_kind(&SMB_SHARE_FORM, "directory"),
            lookup("files", "name")
        );
        assert_eq!(
            create_field_kind(&SMB_SHARE_FORM, "directory"),
            lookup("files", "name")
        );
        assert_eq!(
            field_kind(&SMB_SHARE_FORM, "valid-users"),
            lookup_multi("smb-users", "name")
        );
        assert_eq!(
            field_kind(&SMB_SHARE_FORM, "invalid-users"),
            lookup_multi("smb-users", "name")
        );
        assert_eq!(field_kind(&SMB_SHARE_FORM, "read-only"), FieldKind::Toggle);
        assert_eq!(
            field_kind(&SMB_SHARE_FORM, "require-encryption"),
            FieldKind::Toggle
        );
        assert_eq!(
            field_kind(&SMB_SHARE_FORM, "disabled"),
            FieldKind::InvertedToggle
        );
        assert_eq!(field_kind(&SMB_USER_FORM, "password"), FieldKind::Secret);
        assert_eq!(
            create_field_kind(&SMB_USER_FORM, "password"),
            FieldKind::Secret
        );
        assert_eq!(field_kind(&SMB_USER_FORM, "read-only"), FieldKind::Toggle);
        assert_eq!(
            SMB_SHARE_FORM.field("valid-users").map(|f| f.label),
            Some("Valid Users")
        );
        assert_eq!(
            SMB_SHARE_FORM.field("invalid-users").map(|f| f.label),
            Some("Invalid Users")
        );
        assert_eq!(
            SMB_SHARE_FORM.field("require-encryption").map(|f| f.label),
            Some("Require Encryption")
        );
        assert_eq!(
            SMB_SHARE_FORM.field("read-only").map(|f| f.label),
            Some("Read Only")
        );
        assert_eq!(
            SMB_USER_FORM.field("read-only").map(|f| f.label),
            Some("Read Only")
        );
        status_readonly(&SMB_SHARE_FORM);
        status_readonly(&SMB_USER_FORM);
        assert!(!SMB_SHARE_FORM.writable_keys().contains(&"default"));
        assert!(!SMB_USER_FORM.writable_keys().contains(&"dynamic"));
        assert!(
            SMB_SHARE_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        assert!(
            SMB_USER_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );

        let mut original = HashMap::new();
        original.insert("name".into(), "mtuser".into());
        original.insert("password".into(), "********".into());
        original.insert("read-only".into(), "true".into());
        original.insert("default".into(), "false".into());
        let mut current = original.clone();
        current.insert("read-only".into(), "false".into());
        current.insert("password".into(), "********".into());
        current.insert("default".into(), "true".into());
        let body = patch_body(&SMB_USER_FORM, &original, &current, "********");
        assert_eq!(body.get("read-only").map(String::as_str), Some("false"));
        assert!(!body.contains_key("password"));
        assert!(!body.contains_key("default"));
    }

    #[test]
    fn smb_optional_and_unknown_fields_stay_out_of_patch() {
        let mut original = HashMap::new();
        original.insert("name".into(), "backup".into());
        original.insert("directory".into(), "backup".into());
        original.insert("valid-users".into(), "mtuser".into());
        original.insert("comment".into(), String::new());
        original.insert("dynamic".into(), "false".into());
        let mut current = original.clone();
        current.insert("valid-users".into(), "mtuser,guest".into());
        current.insert("unexpected-flag".into(), "yes".into());
        current.insert("dynamic".into(), "true".into());
        let body = patch_body(&SMB_SHARE_FORM, &original, &current, "********");
        assert_eq!(
            body.get("valid-users").map(String::as_str),
            Some("mtuser,guest")
        );
        assert!(!body.contains_key("directory"));
        assert!(!body.contains_key("dynamic"));
        assert!(!body.contains_key("unexpected-flag"));

        let mut row = HashMap::new();
        row.insert("name".into(), "pub".into());
        row.insert("directory".into(), "/pub".into());
        row.insert("unexpected-flag".into(), "yes".into());
        let extras = extra_status_fields(&SMB_SHARE_FORM, &row);
        assert_eq!(extras, vec![("unexpected-flag".into(), "yes".into())]);

        let empty = HashMap::new();
        for key in SMB_SHARE_FORM.known_keys() {
            assert!(
                field_visible("smb-shares", key, &empty),
                "{key} should stay visible"
            );
        }
        for key in SMB_USER_FORM.known_keys() {
            assert!(
                field_visible("smb-users", key, &empty),
                "{key} should stay visible"
            );
        }
    }
}

mod ipsec {
    use super::*;

    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema.create_keys()
    }

    fn status_readonly(schema: &FormSchema) {
        let status = schema
            .sections
            .iter()
            .find(|section| section.id == "status")
            .expect("status tab");
        assert!(status.read_only);
        for field in status.fields {
            assert!(!field.kind.writable(), "{}", field.key);
            assert!(!schema.writable_keys().contains(&field.key));
        }
    }

    #[test]
    fn create_peer_requires_name_and_address() {
        assert_eq!(
            create_keys(&IPSEC_PEER_FORM),
            IPSEC_PEER_FORM.writable_keys()
        );
        assert!(IPSEC_PEER_FORM.writable_keys().contains(&"exchange-mode"));
        assert!(IPSEC_PEER_FORM.writable_keys().contains(&"local-address"));
        assert!(!IPSEC_PEER_FORM.writable_keys().contains(&"dynamic"));
        status_readonly(&IPSEC_PEER_FORM);
    }

    #[test]
    fn identity_secret_is_masked_and_my_id_is_not() {
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("secret").map(|f| f.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("my-id").map(|f| f.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("remote-id").map(|f| f.kind),
            Some(FieldKind::Text)
        );
        assert!(IPSEC_IDENTITY_FORM.writable_keys().contains(&"peer"));
        assert!(IPSEC_IDENTITY_FORM.writable_keys().contains(&"auth-method"));
        assert!(
            IPSEC_IDENTITY_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        status_readonly(&IPSEC_IDENTITY_FORM);
    }

    #[test]
    fn identity_patch_omits_masked_secret() {
        let mut original = HashMap::new();
        original.insert("peer".into(), "office".into());
        original.insert("secret".into(), "********".into());
        original.insert("my-id".into(), "fqdn:router.example".into());
        original.insert("comment".into(), "psk".into());
        let mut current = original.clone();
        current.insert("comment".into(), "updated".into());
        current.insert("secret".into(), "********".into());
        let body = patch_body(&IPSEC_IDENTITY_FORM, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("updated"));
        assert!(!body.contains_key("secret"));
        assert!(!body.contains_key("peer"));
        assert!(!body.contains_key("my-id"));
    }

    #[test]
    fn identity_patch_sends_changed_secret() {
        let mut original = HashMap::new();
        original.insert("secret".into(), "********".into());
        let mut current = original.clone();
        current.insert("secret".into(), "new-preshared-key".into());
        let body = patch_body(&IPSEC_IDENTITY_FORM, &original, &current, "********");
        assert_eq!(
            body.get("secret").map(String::as_str),
            Some("new-preshared-key")
        );
    }

    #[test]
    fn policy_status_is_readonly() {
        status_readonly(&IPSEC_POLICY_FORM);
        assert!(IPSEC_POLICY_FORM.writable_keys().contains(&"src-address"));
        assert!(IPSEC_POLICY_FORM.writable_keys().contains(&"proposal"));
        assert!(!IPSEC_POLICY_FORM.writable_keys().contains(&"ph2-state"));
        assert_eq!(
            create_keys(&IPSEC_POLICY_FORM),
            IPSEC_POLICY_FORM.writable_keys()
        );
    }

    #[test]
    fn proposal_and_profile_writable_keys() {
        for key in [
            "name",
            "auth-algorithms",
            "enc-algorithms",
            "pfs-group",
            "lifetime",
            "disabled",
        ] {
            assert!(IPSEC_PROPOSAL_FORM.writable_keys().contains(&key), "{key}");
        }
        for key in [
            "name",
            "hash-algorithm",
            "enc-algorithm",
            "dh-group",
            "proposal-check",
            "lifetime",
            "nat-traversal",
            "dpd-interval",
            "dpd-maximum-failures",
        ] {
            assert!(IPSEC_PROFILE_FORM.writable_keys().contains(&key), "{key}");
        }
        assert_eq!(
            create_keys(&IPSEC_PROPOSAL_FORM),
            IPSEC_PROPOSAL_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&IPSEC_PROFILE_FORM),
            IPSEC_PROFILE_FORM.writable_keys()
        );
    }

    #[test]
    fn settings_unknown_keys_land_on_status_extras() {
        assert!(IPSEC_SETTINGS_FORM.create_sections.is_empty());
        assert!(IPSEC_SETTINGS_FORM.writable_keys().contains(&"accounting"));
        assert!(
            IPSEC_SETTINGS_FORM
                .writable_keys()
                .contains(&"interim-update")
        );
        let mut row = HashMap::new();
        row.insert("accounting".into(), "true".into());
        row.insert("unexpected-flag".into(), "yes".into());
        let extras = extra_status_fields(&IPSEC_SETTINGS_FORM, &row);
        assert_eq!(extras, vec![("unexpected-flag".into(), "yes".into())]);
    }

    fn assert_lookup(form: &FormSchema, key: &str, resource_id: &'static str) {
        assert_eq!(
            form.field(key).map(|field| field.kind),
            Some(FieldKind::Lookup {
                resource_id,
                value_key: "name",
                multiple: false,
            }),
            "{key}"
        );
    }

    #[test]
    fn lookup_fields_point_at_named_resources() {
        assert_lookup(&IPSEC_IDENTITY_FORM, "peer", "ipsec-peers");
        assert_lookup(&IPSEC_PEER_FORM, "profile", "ipsec-profiles");
        assert_lookup(&IPSEC_POLICY_FORM, "proposal", "ipsec-proposals");
        assert_lookup(&IPSEC_IDENTITY_FORM, "certificate", "certificates");
        assert_lookup(&IPSEC_IDENTITY_FORM, "remote-certificate", "certificates");
        assert_lookup(&IPSEC_POLICY_FORM, "peer", "ipsec-peers");
    }

    #[test]
    fn non_resource_fields_stay_plain_text_or_secret() {
        assert_eq!(
            IPSEC_PEER_FORM.field("address").map(|field| field.kind),
            Some(FieldKind::Ip)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("secret").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("my-id").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM
                .field("auth-method")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_PROPOSAL_FORM
                .field("auth-algorithms")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_PROPOSAL_FORM
                .field("enc-algorithms")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }

    #[test]
    fn rsa_keys_are_named_with_readonly_size() {
        assert_eq!(
            create_keys(&IPSEC_KEY_RSA_FORM),
            IPSEC_KEY_RSA_FORM.writable_keys()
        );
        assert!(IPSEC_KEY_RSA_FORM.writable_keys().contains(&"name"));
        assert!(!IPSEC_KEY_RSA_FORM.writable_keys().contains(&"key-size"));
        status_readonly(&IPSEC_KEY_RSA_FORM);
    }

    #[test]
    fn psk_keys_use_peer_lookup_and_secret() {
        assert_eq!(
            create_keys(&IPSEC_KEY_PSK_FORM),
            IPSEC_KEY_PSK_FORM.writable_keys()
        );
        assert_lookup(&IPSEC_KEY_PSK_FORM, "peer", "ipsec-peers");
        assert_eq!(
            IPSEC_KEY_PSK_FORM.field("key").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(IPSEC_KEY_PSK_FORM.secret_keys(), ["key"]);
        assert!(
            IPSEC_KEY_PSK_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn qkd_is_singleton_edit_with_certificate_lookup() {
        assert!(IPSEC_KEY_QKD_FORM.create_sections.is_empty());
        assert_lookup(&IPSEC_KEY_QKD_FORM, "certificate", "certificates");
        assert!(IPSEC_KEY_QKD_FORM.writable_keys().contains(&"address"));
        assert!(IPSEC_KEY_QKD_FORM.writable_keys().contains(&"kme-id"));
        assert!(!IPSEC_KEY_QKD_FORM.writable_keys().contains(&"cache-state"));
        status_readonly(&IPSEC_KEY_QKD_FORM);
        assert!(
            IPSEC_KEY_QKD_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }
}

mod hotspot {
    use super::*;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema.create_keys()
    }

    #[test]
    fn hotspot_create_matches_writable_sheet() {
        assert_eq!(create_keys(&HOTSPOT_FORM), HOTSPOT_FORM.writable_keys());
        assert_eq!(
            create_keys(&HOTSPOT_PROFILE_FORM),
            HOTSPOT_PROFILE_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&HOTSPOT_USER_FORM),
            HOTSPOT_USER_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&HOTSPOT_HOST_FORM),
            HOTSPOT_HOST_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&HOTSPOT_IP_BINDING_FORM),
            HOTSPOT_IP_BINDING_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&HOTSPOT_WALLED_GARDEN_FORM),
            HOTSPOT_WALLED_GARDEN_FORM.writable_keys()
        );
        assert_eq!(
            create_keys(&HOTSPOT_WALLED_GARDEN_IP_FORM),
            HOTSPOT_WALLED_GARDEN_IP_FORM.writable_keys()
        );
        assert!(PROXY_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&PROXY_ACCESS_FORM),
            PROXY_ACCESS_FORM.writable_keys()
        );
        assert!(HOTSPOT_USER_FORM.writable_keys().contains(&"password"));
        assert!(!HOTSPOT_HOST_FORM.writable_keys().contains(&"authorized"));
    }
}
