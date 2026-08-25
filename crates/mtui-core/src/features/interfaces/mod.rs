//! Interfaces feature boundary: catalog, forms, actions, guides, and gates.

pub(crate) mod actions;
pub(crate) mod forms;
pub(crate) mod guides;
pub(crate) mod resources;
pub(crate) mod rules;

/// Map a `/interface` `type` value to the feature-owned editor resource.
#[must_use]
pub fn edit_resource_for_interface_type(iface_type: &str) -> Option<&'static str> {
    Some(match iface_type {
        "ether" => "ethernet",
        "vlan" => "vlan",
        "eoip" => "eoip",
        "ipip" => "ipip",
        "gre" | "gre-tunnel" => "gre",
        "gre6" | "gre6-tunnel" => "gre6",
        "6to4" => "6to4",
        "vxlan" => "vxlan",
        "vrrp" => "vrrp",
        "bond" => "bonding",
        "lte" => "lte",
        "wlan" => "wireless",
        "wifi" | "wifiwave2" => "wifi",
        "macvlan" => "macvlan",
        "veth" => "veth",
        "macsec" => "macsec",
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
