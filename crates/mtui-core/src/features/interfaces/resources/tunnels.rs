//! Tunnel interface resource descriptors.

use std::time::Duration;

use crate::resources::{FetchKind, ResourceSpec};

const REFRESH: Duration = Duration::from_secs(10);

pub const EOIP: ResourceSpec = ResourceSpec {
    id: "eoip",
    group: "interfaces-group",
    label: "EoIP Tunnel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/eoip",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("remote-address", "Remote Address", 18),
        col!("tunnel-id", "Tunnel ID", 10),
        col!("actual-mtu", "Actual MTU", 10),
        col!("running", "Running", 7),
        col!("disabled", "Disabled", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: REFRESH,
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::EOIP_FORM),
};

pub const IPIP: ResourceSpec = ResourceSpec {
    id: "ipip",
    group: "interfaces-group",
    label: "IP Tunnel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/ipip",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("local-address", "Local Address", 18),
        col!("remote-address", "Remote Address", 18),
        col!("actual-mtu", "Actual MTU", 10),
        col!("running", "Running", 7),
        col!("disabled", "Disabled", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: REFRESH,
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::IPIP_FORM),
};

pub const GRE: ResourceSpec = ResourceSpec {
    id: "gre",
    group: "interfaces-group",
    label: "GRE Tunnel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/gre",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("local-address", "Local Address", 18),
        col!("remote-address", "Remote Address", 18),
        col!("actual-mtu", "Actual MTU", 10),
        col!("running", "Running", 7),
        col!("disabled", "Disabled", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: REFRESH,
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::GRE_FORM),
};

pub const SIX_TO_FOUR: ResourceSpec = ResourceSpec {
    id: "6to4",
    group: "interfaces-group",
    label: "6to4 Tunnel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/6to4",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("local-address", "Local Address", 18),
        col!("remote-address", "Remote Address", 18),
        col!("actual-mtu", "Actual MTU", 10),
        col!("running", "Running", 7),
        col!("disabled", "Disabled", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: REFRESH,
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::tunnels::SIX_TO_FOUR_FORM),
};

pub const GRE6: ResourceSpec = ResourceSpec {
    id: "gre6",
    group: "interfaces-group",
    label: "GRE6 Tunnel",
    fetch: FetchKind::List {
        endpoint: "/rest/interface/gre6",
    },
    columns: &[
        col!("name", "Name", 18),
        col!("local-address", "Local Address", 22),
        col!("remote-address", "Remote Address", 22),
        col!("actual-mtu", "Actual MTU", 10),
        col!("running", "Running", 7),
        col!("disabled", "Disabled", 8),
        col!("comment", "Comment", 28),
    ],
    refresh: REFRESH,
    actions: crate::features::interfaces::actions::VIRTUAL_IFACE_ACTIONS,
    form: Some(&crate::features::interfaces::forms::tunnels::GRE6_FORM),
};

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(spec: &ResourceSpec) -> &'static str {
        match spec.fetch {
            FetchKind::List { endpoint } => endpoint,
            FetchKind::System { .. } | FetchKind::Local => {
                panic!("tunnel must be a list resource")
            }
        }
    }

    #[test]
    fn routeros_7215_tunnel_endpoints_are_exact_and_have_no_sit() {
        let resources = [&EOIP, &IPIP, &GRE, &SIX_TO_FOUR, &GRE6];
        assert_eq!(
            resources
                .iter()
                .map(|spec| endpoint(spec))
                .collect::<Vec<_>>(),
            [
                "/rest/interface/eoip",
                "/rest/interface/ipip",
                "/rest/interface/gre",
                "/rest/interface/6to4",
                "/rest/interface/gre6",
            ]
        );
        assert!(
            resources
                .iter()
                .all(|spec| spec.id != "sit" && endpoint(spec) != "/rest/interface/sit")
        );
    }

    #[test]
    fn each_tunnel_uses_its_distinct_form_contract() {
        assert!(std::ptr::eq(
            SIX_TO_FOUR.form.unwrap(),
            &raw const crate::features::interfaces::forms::tunnels::SIX_TO_FOUR_FORM
        ));
        assert!(std::ptr::eq(
            GRE6.form.unwrap(),
            &raw const crate::features::interfaces::forms::tunnels::GRE6_FORM
        ));
        assert!(SIX_TO_FOUR.form.unwrap().field("allow-fast-path").is_none());
        assert!(GRE6.form.unwrap().field("dont-fragment").is_none());
    }
}
