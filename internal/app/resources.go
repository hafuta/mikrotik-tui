package app

import (
	"strconv"
	"strings"
	"time"

	"github.com/hafuta/mikrotik-tui/internal/routeros"
	"github.com/hafuta/mikrotik-tui/internal/ui"
)

type resourceSpec struct {
	ID       string
	Group    string
	Label    string
	Resource routeros.ResourceDescriptor
	System   *routeros.SystemResource
	Columns  []ui.Column
	Refresh  time.Duration
}

var resourceSpecs = []resourceSpec{
	{ID: "interfaces", Group: "interfaces-group", Label: "Interface", Resource: routeros.InterfacesResource, Columns: columns("name:Name:18", "type:Type:12", "running:Run:5", "disabled:Off:5", "mtu:MTU:7"), Refresh: 5 * time.Second},
	{ID: "interface-lists", Group: "interfaces-group", Label: "Interface List", Resource: routeros.InterfaceListsResource, Columns: columns("name:Name:20", "comment:Comment:28"), Refresh: 30 * time.Second},
	{ID: "ethernet", Group: "interfaces-group", Label: "Ethernet", Resource: routeros.EthernetResource, Columns: columns("name:Name:18", "mac-address:MAC address:18", "speed:Speed:12", "full-duplex:Duplex:8", "running:Run:5"), Refresh: 5 * time.Second},
	{ID: "ppp-active", Group: "ppp-group", Label: "Active", Resource: routeros.PPPActiveResource, Columns: columns("name:Name:18", "service:Service:10", "caller-id:Caller:18", "address:Address:18", "uptime:Uptime:12"), Refresh: 5 * time.Second},
	{ID: "pppoe-clients", Group: "ppp-group", Label: "PPPoE Clients", Resource: routeros.PPPoEClientsResource, Columns: columns("name:Name:18", "interface:Interface:16", "user:User:18", "status:Status:12", "running:Run:5"), Refresh: 5 * time.Second},
	{ID: "bridges", Group: "bridge-group", Label: "Bridge", Resource: routeros.BridgesResource, Columns: columns("name:Name:18", "protocol-mode:Protocol:12", "vlan-filtering:VLAN:6", "running:Run:5", "disabled:Off:5"), Refresh: 10 * time.Second},
	{ID: "bridge-ports", Group: "bridge-group", Label: "Ports", Resource: routeros.BridgePortsResource, Columns: columns("interface:Interface:18", "bridge:Bridge:18", "pvid:PVID:7", "role:Role:12", "hw:HW:4"), Refresh: 10 * time.Second},
	{ID: "bridge-vlans", Group: "bridge-group", Label: "VLANs", Resource: routeros.BridgeVLANsResource, Columns: columns("bridge:Bridge:16", "vlan-ids:VLAN IDs:14", "tagged:Tagged:24", "untagged:Untagged:24"), Refresh: 15 * time.Second},
	{ID: "arp", Group: "ip-group", Label: "ARP", Resource: routeros.ARPResource, Columns: columns("address:Address:18", "mac-address:MAC address:18", "interface:Interface:16", "status:Status:12", "dynamic:Dyn:5"), Refresh: 5 * time.Second},
	{ID: "addresses", Group: "ip-group", Label: "Addresses", Resource: routeros.AddressesResource, Columns: columns("address:Address:20", "network:Network:18", "interface:Interface:16", "dynamic:Dyn:5", "disabled:Off:5"), Refresh: 15 * time.Second},
	{ID: "dhcp-servers", Group: "ip-group", Label: "DHCP", Resource: routeros.DHCPServersResource, Columns: columns("name:Name:18", "interface:Interface:16", "address-pool:Pool:18", "lease-time:Lease time:12", "status:Status:10"), Refresh: 10 * time.Second},
	{ID: "dhcp-networks", Group: "ip-group", Label: "Networks", Resource: routeros.DHCPNetworksResource, Columns: columns("address:Network:20", "gateway:Gateway:18", "dns-server:DNS:24", "domain:Domain:18"), Refresh: 30 * time.Second},
	{ID: "dhcp-leases", Group: "ip-group", Label: "Leases", Resource: routeros.DHCPLeasesResource, Columns: columns("address:Address:18", "mac-address:MAC address:18", "host-name:Hostname:20", "status:Status:10", "expires-after:Expires:12"), Refresh: 5 * time.Second},
	{ID: "firewall-filter", Group: "ip-group", Label: "Firewall", Resource: routeros.FirewallFilterResource, Columns: columns("chain:Chain:10", "action:Action:12", "protocol:Protocol:9", "src-address:Source:20", "src-port:Src port:10", "dst-address:Destination:20", "dst-port:Dst port:10", "in-interface:In interface:16", "out-interface:Out interface:16", "packets:Packets:12", "bytes:Bytes:14", "disabled:Off:5", "dynamic:Dyn:5", "invalid:Bad:5", "comment:Comment:28"), Refresh: 5 * time.Second},
	{ID: "users", Group: "system-group", Label: "Users", Resource: routeros.UsersResource, Columns: columns("name:Name:18", "group:Group:14", "last-logged-in:Last login:22", "disabled:Off:5"), Refresh: 30 * time.Second},
	{ID: "routerboard", Group: "system-group", Label: "RouterBOARD", System: &routeros.RouterBOARDResource, Columns: columns("model:Model:18", "serial-number:Serial:18", "current-firmware:Current:12", "upgrade-firmware:Upgrade:12"), Refresh: time.Minute},
	{ID: "ntp", Group: "system-group", Label: "NTP Client", System: &routeros.NTPResource, Columns: columns("enabled:Enabled:8", "mode:Mode:12", "servers:Servers:28", "status:Status:12"), Refresh: 30 * time.Second},
	{ID: "clock", Group: "system-group", Label: "Clock", System: &routeros.ClockResource, Columns: columns("time:Time:12", "date:Date:14", "time-zone-name:Time zone:22", "gmt-offset:Offset:10"), Refresh: 10 * time.Second},
	{ID: "logs", Group: "system-group", Label: "Logs", Resource: routeros.LogResource, Columns: columns("time:Time:19", "topics:Topics:24", "message:Message:72"), Refresh: time.Second},
}

func columns(definitions ...string) []ui.Column {
	result := make([]ui.Column, 0, len(definitions))
	for _, definition := range definitions {
		parts := strings.Split(definition, ":")
		width := 12
		if len(parts) == 3 {
			if parsed, err := strconv.Atoi(parts[2]); err == nil && parsed > 0 {
				width = parsed
			}
		}
		result = append(result, ui.Column{Key: parts[0], Title: parts[1], Width: width})
	}
	return result
}

func specByID(id string) (resourceSpec, bool) {
	for _, spec := range resourceSpecs {
		if spec.ID == id {
			return spec, true
		}
	}
	return resourceSpec{}, false
}

func (s resourceSpec) CLIPath() string {
	if s.System != nil {
		return routeros.CLIPath(s.System.Endpoint)
	}
	return routeros.CLIPath(s.Resource.Endpoint)
}

func navigationItems() []ui.NavItem {
	group := func(id, label string) ui.NavItem {
		item := ui.NavItem{ID: id, Label: label}
		for _, spec := range resourceSpecs {
			if spec.Group == id {
				item.Children = append(item.Children, ui.NavItem{ID: spec.ID, Label: spec.Label})
			}
		}
		return item
	}
	return []ui.NavItem{
		{ID: "dashboard", Label: "Dashboard"},
		group("interfaces-group", "Interfaces"),
		group("ppp-group", "PPP"),
		group("bridge-group", "Bridge"),
		group("ip-group", "IP"),
		group("system-group", "System"),
	}
}
