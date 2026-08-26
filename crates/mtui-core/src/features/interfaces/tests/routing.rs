use crate::features::interfaces::edit_resource_for_interface_type;

#[test]
fn maps_wireguard_ppp_and_bridge_interface_types() {
    assert_eq!(edit_resource_for_interface_type("wg"), Some("wireguard"));
    assert_eq!(edit_resource_for_interface_type("bridge"), Some("bridges"));
    assert_eq!(
        edit_resource_for_interface_type("pppoe-out"),
        Some("pppoe-clients")
    );
    assert_eq!(
        edit_resource_for_interface_type("pptp-out"),
        Some("pptp-client")
    );
    assert_eq!(
        edit_resource_for_interface_type("l2tp-out"),
        Some("l2tp-client")
    );
    assert_eq!(
        edit_resource_for_interface_type("sstp-out"),
        Some("sstp-client")
    );
    assert_eq!(
        edit_resource_for_interface_type("ovpn-out"),
        Some("ovpn-client")
    );
}
