//! Feature-owned operator guides for Interfaces screens.

use crate::about::ScreenGuide;

macro_rules! guide {
    ($id:literal, $summary:literal, $when:literal, $fields:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: None,
            },
        )
    };
    ($id:literal, $summary:literal, $when:literal, $fields:literal, $docs:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: Some($docs),
            },
        )
    };
}

pub(crate) static GUIDES: &[(&str, ScreenGuide)] = &[
    guide!(
        "interfaces",
        "All RouterOS interfaces in one table: Ethernet, VLANs, tunnels, wireless, bridges, \
         and dynamic entries. Each row is a L2/L3 attachment point with MTU, MAC, and counters.",
        "Start here to see what exists, whether a link is running, or to torch/reset counters. \
         Type-specific menus (Ethernet, VLAN, WireGuard, …) edit the extra properties of that type.",
        "name, type, mtu/l2mtu/actual-mtu, mac-address, running/slave/disabled, and RX/TX \
         counters. Fast-path (fp-*) counters appear when the packet path bypasses the CPU."
    ),
    guide!(
        "interface-lists",
        "Named sets of interfaces used by firewall, neighbor discovery, detect-internet, \
         and similar consumers. Built-in sets include all, none, static, and dynamic; \
         you can add your own names.",
        "Define the set here, then attach interfaces on List members. Use include/exclude \
         to nest other lists instead of repeating the same members. Changing a list updates \
         every rule that refers to it.",
        "name is the set. include and exclude nest other lists. Lists have no disabled flag. \
         Membership joins live on List members — adding a bridge is not the same as adding \
         its ports.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/47579180/Interface+Lists"
    ),
    guide!(
        "ethernet",
        "Physical Ethernet (and SFP) ports: negotiation, advertised speeds, switch chip, \
         and loop-protect. These are the parent interfaces for VLANs, MACsec, and MACVLAN.",
        "Use it to rename ports, lock speed/duplex, or inspect the switch the port belongs to. \
         Do not expect virtual interfaces here.",
        "auto-negotiation, advertise, speed, full-duplex, arp, orig-mac-address, switch, \
         loop-protect. default-name is the factory name."
    ),
    guide!(
        "interface-list-members",
        "Joins that attach an interface to a list. This table is the membership, not the \
         named set itself (that lives on Lists).",
        "Assign LAN/WAN or a custom list so firewall and discovery can match a group of \
         ports. Dynamic members created by include/exclude on Lists are not shown here.",
        "list is the target set; interface is the member. disabled parks a join without \
         deleting it. Built-in lists still accept members."
    ),
    guide!(
        "eoip",
        "Ethernet over IP: a Layer-2 tunnel that carries Ethernet frames inside IP so two \
         sites can share a broadcast domain. It is a MikroTik protocol, not GRE.",
        "Use it to bridge remote LANs or run protocols that need Ethernet, not just IP. \
         Pair local-address/remote-address and a matching tunnel-id on both ends.",
        "tunnel-id, local-address, remote-address, ipsec-secret when encrypting, keepalive, \
         and the usual interface MTU/MAC fields."
    ),
    guide!(
        "ipip",
        "IP-in-IP tunnel: encapsulates IPv4 (or IPv6) packets inside another IP packet. \
         It does not carry Ethernet frames.",
        "Use it for simple point-to-point routed tunnels when you do not need L2 bridging. \
         EoIP or GRE are better if you need Ethernet or extra GRE features.",
        "local-address, remote-address, clamp-tcp-mss, ipsec-secret, keepalive."
    ),
    guide!(
        "gre",
        "GRE tunnel for encapsulating packets between two IP endpoints. RouterOS can add \
         IPsec to the same tunnel.",
        "Use GRE when you need a standard tunnel that other vendors also speak, or when \
         you want IPsec without a separate IPsec policy set.",
        "local-address, remote-address, allow-fast-path, ipsec-secret, keepalive, mtu."
    ),
    guide!(
        "vlan",
        "IEEE 802.1Q (and Q-in-Q) VLAN interfaces on a parent port. This is a router VLAN \
         subinterface, not the bridge VLAN table used for switching.",
        "Use it when the router itself needs an IP on a tagged VLAN. For hardware switching \
         between ports, use Bridge VLANs (or Switch VLAN) instead.",
        "vlan-id and parent interface define the tag. use-service-tag selects 802.1ad. MTU \
         is usually inherited from the parent."
    ),
    guide!(
        "vxlan",
        "VXLAN overlay: Ethernet segments identified by a VNI, carried over UDP/IP between \
         VTEPs. Used to stretch L2 across an L3 underlay.",
        "Use it in datacenter or campus overlays, not as a replacement for a simple VLAN on \
         one switch. Pair vni with local-address or multicast group.",
        "vni, local-address, group, port (default 4789), vteps, learning, and optional bridge \
         binding for hardware offload on supported chips."
    ),
    guide!(
        "vrrp",
        "Virtual Router Redundancy Protocol: two or more routers share a virtual IP (and MAC) \
         so LAN hosts keep a gateway if the master fails.",
        "Use it for first-hop redundancy on a LAN. Both routers need a matching VRID and \
         authentication; priority elects the master.",
        "interface, vrid, priority, version (v2/v3), address, preemption-mode, authentication."
    ),
    guide!(
        "bonding",
        "Link aggregation: several Ethernet (or similar) ports act as one logical interface \
         for capacity or failover (802.3ad, active-backup, balance-*, …).",
        "Use it when the switch or peer supports the same mode. Members must match speed and \
         usually sit on the same switch chip for offload.",
        "mode, slaves/members, transmit-hash-policy, lacp-rate, min-links, mtu."
    ),
    guide!(
        "lte",
        "LTE/5G modems: APN profiles, band, and connection state for the cellular interface.",
        "Use it on devices with a modem (or USB LTE) to bring up a WAN over mobile data. \
         Carrier APN names, authentication, and PDN type live on LTE APN.",
        "apn-profiles, network-mode, band, PIN, allow-roaming, running and disabled."
    ),
    guide!(
        "lte-apn",
        "LTE APN profiles (`/interface lte apn`): the carrier access point, authentication, \
         and IP type the modem uses when attaching.",
        "Edit these on LTE boards when the SIM needs a named APN, PAP/CHAP, or a specific \
         PDN type. Assign the profile on the LTE interface. Turn off Use Network APN when \
         the carrier-provided APN is wrong.",
        "name, apn, authentication, user, password, ip-type, use-network-apn, use-peer-dns, \
         add-default-route, passthrough interface.",
        "https://manual.mikrotik.com/docs/cli-reference/interface/lte/apn/"
    ),
    guide!(
        "wifi",
        "wifiwave2 / wifi interfaces (RouterOS 7 radio stack): SSIDs, datapath, and radio \
         settings for ax/ac Wave2 hardware.",
        "Use this on modern wifi packages. Legacy `/interface wireless` is a different driver \
         and menu (Wireless).",
        "configuration, datapath, channel, ssid, security, running/disabled. CAPsMAN-managed \
         radios may be mostly read-only here."
    ),
    guide!(
        "wireless",
        "Legacy wireless (wireless package): older 802.11 radios, station/AP modes, and \
         WDS. Distinct from the wifiwave2 wifi menu.",
        "Use it only on hardware that still runs the old wireless driver. New ax devices \
         belong under WiFi.",
        "mode, ssid, frequency, band, security-profile, running/disabled."
    ),
    guide!(
        "macvlan",
        "Virtual interfaces with their own MAC on a parent that already has a MAC. Unlike \
         VLAN, there is no 802.1Q tag — the distinction is the MAC address.",
        "Use it to get extra MACs/IPs or multiple PPPoE sessions from one Ethernet. Not for \
         bridging or stacking a VLAN on top of MACVLAN.",
        "interface (parent), mac-address, mode private (no MACVLAN-to-MACVLAN on the same \
         parent) or bridge (they can talk). Parent must not already be bridged or bonded.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/217874440/MACVLAN"
    ),
    guide!(
        "veth",
        "Virtual Ethernet for containers: RouterOS holds one end and the container namespace \
         holds the other. It can take a static address or run a DHCP client.",
        "Create one before you add a container, then look it up from the container Interface \
         field. Bridge it or address it like any Ethernet.",
        "name, address (repeat IPv4/IPv6), gateway, IPv6 gateway, DHCP, MAC, container MAC.",
        "https://manual.mikrotik.com/docs/containers/veth/"
    ),
    guide!(
        "macsec",
        "MACsec (IEEE 802.1AE) encrypts Ethernet frames on a point-to-point link so traffic \
         on the wire is confidential, intact, and authentic. RouterOS uses GCM-AES-128 and \
         protects all LAN traffic on that hop, including DHCP, ARP, and LLDP.",
        "Use it when two devices share a dedicated Ethernet (or similar) hop and you need \
         Layer-2 encryption rather than IPsec. Matching pre-shared CAK and CKN are required \
         on both ends (no Dot1x keying yet). Hardware offload exists only on some products. \
         Skip this menu if you are not encrypting a single Ethernet hop.",
        "interface is the parent Ethernet (one MACsec per Ethernet). cak (16-byte) and ckn \
         (32-byte) must match the peer; omit them to let RouterOS generate values, then copy \
         those to the other side. profile selects key-server priority. status is read-only \
         (negotiating, open-encrypted, …). MTU defaults to 1468; L2MTU is derived (parent \
         minus 32 bytes) and is not settable.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/macsec/"
    ),
    guide!(
        "macsec-profiles",
        "MACsec profiles used to elect the MKA key server on a point-to-point link. The key \
         server creates the Secure Association Key (SAK) that actually encrypts frames.",
        "Change this only when you need a specific side to be key server. Most setups keep \
         the default profile.",
        "name and server-priority (0–255, lower wins). Equal priority falls back to the \
         lowest MAC address.",
        "https://manual.mikrotik.com/docs/bridging-and-switching/macsec/"
    ),
    guide!(
        "vrf",
        "Virtual Routing and Forwarding: isolated routing tables so the same prefixes can \
         exist in more than one tenant or WAN context.",
        "Use it for overlapping VPNs, multi-tenant CE, or keeping management routing apart \
         from data. Interfaces are assigned to a VRF; connected routes follow.",
        "name, interfaces, and comments. Routes themselves still live under IP/IPv6 Routes \
         with a routing-table/VRF mark."
    ),
    guide!(
        "detect-internet",
        "Singleton that classifies interfaces as internet, WAN, or LAN using reachability \
         tests, then can populate interface lists.",
        "Useful on simple CPE so WAN/LAN lists stay correct. Turn it off if you assign WAN \
         membership yourself and do not want surprise list changes.",
        "detect-interface-list, internet/wan/lan-interface-list, and related state."
    ),
    guide!(
        "6to4",
        "IPv6-in-IPv4 6to4 tunnels (`/interface 6to4`).",
        "Use when you need 6to4 or similar IPv6 overlay without GRE.",
        "name, local-address, remote-address, mtu, disabled."
    ),
    guide!(
        "gre6",
        "GRE tunnels that carry IPv6 (`/interface gre6`).",
        "Same idea as GRE, for IPv6 endpoints.",
        "name, local-address, remote-address, mtu, keepalive, disabled."
    ),
    guide!(
        "wifi-security",
        "Reusable wifiwave2 security profiles (WPA2/WPA3 PSK, enterprise).",
        "Share one passphrase/profile across several WiFi interfaces or provisioned CAPs.",
        "name, authentication-types, passphrase, disabled."
    ),
    guide!(
        "wifi-channel",
        "Reusable channel/band/width profiles for wifiwave2 radios.",
        "Keep frequency plans in one place instead of editing each radio.",
        "name, band, frequency, width, disabled."
    ),
    guide!(
        "wifi-datapath",
        "Bridge/VLAN datapath profiles for wifiwave2.",
        "Steer stations onto a bridge or VLAN without repeating the same keys on every radio.",
        "name, bridge, vlan-id, disabled."
    ),
    guide!(
        "wifi-configuration",
        "Named wifiwave2 configurations that bind SSID, country, security, datapath, and channel.",
        "CAPsMAN and local WiFi both point at these names.",
        "name, ssid, country, security, datapath, channel, disabled."
    ),
    guide!(
        "wifi-provisioning",
        "Rules that assign a master configuration to matching CAP radios.",
        "Use on a CAPsMAN controller to auto-configure new APs by band.",
        "action, supported-bands, master-configuration, disabled."
    ),
    guide!(
        "wifi-cap",
        "Client CAP settings: which CAPsMAN this radio should join.",
        "Enable on AP hardware that should be managed, not on the controller.",
        "enabled, caps-man-addresses, discovery-interfaces."
    ),
    guide!(
        "wifi-capsman",
        "Controller (CAPsMAN) enablement and certificates.",
        "Turn on only on the manager. Certificates stay secrets.",
        "enabled, ca-certificate, certificate."
    ),
    guide!(
        "wireless-security-profiles",
        "Legacy `/interface wireless security-profiles` (WPA PSK and friends).",
        "Needed only when the old wireless package is installed.",
        "name, mode, authentication-types, wpa2-pre-shared-key."
    ),
    guide!(
        "wireless-access-list",
        "Legacy wireless access-list (allow/deny by MAC).",
        "Lock a radio to known stations or block a noisy client.",
        "mac-address, interface, authentication, forwarding, disabled."
    ),
    guide!(
        "wireless-registration-table",
        "Live stations associated to a legacy wireless radio. Read-only besides disconnect.",
        "See who is on the AP. Disconnect with remove; scan is on the radio row.",
        "mac-address, interface, ap, signal-strength, uptime."
    ),
];
