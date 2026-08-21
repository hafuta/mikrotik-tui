package routeros

import (
	"encoding/json"
	"fmt"
	"net/url"
	"strings"
)

// Resource is a RouterOS REST record. RouterOS represents record values as
// strings; Fields deliberately keeps those values unmodified.
type Resource struct {
	ID     string
	Fields map[string]string
}

func (r *Resource) UnmarshalJSON(data []byte) error {
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}

	r.Fields = make(map[string]string, len(raw))
	for key, value := range raw {
		var text string
		if err := json.Unmarshal(value, &text); err != nil {
			return fmt.Errorf("field %q is not a string: %w", key, err)
		}
		if key == ".id" {
			r.ID = text
			continue
		}
		r.Fields[key] = text
	}
	return nil
}

// Raw returns an unmodified field value.
func (r Resource) Raw(name string) (string, bool) {
	value, ok := r.Fields[name]
	return value, ok
}

// ResourceDescriptor identifies a list-like RouterOS REST resource.
type ResourceDescriptor struct {
	Name     string
	Endpoint string
}

// SystemResource identifies a singleton or system-scoped RouterOS resource.
type SystemResource struct {
	Name     string
	Endpoint string
}

const (
	EndpointInterfaces     = "/rest/interface"
	EndpointInterfaceLists = "/rest/interface/list"
	EndpointEthernet       = "/rest/interface/ethernet"
	EndpointPPPActive      = "/rest/ppp/active"
	EndpointPPPoEClients   = "/rest/interface/pppoe-client"
	EndpointBridges        = "/rest/interface/bridge"
	EndpointBridgePorts    = "/rest/interface/bridge/port"
	EndpointBridgeVLANs    = "/rest/interface/bridge/vlan"
	EndpointARP            = "/rest/ip/arp"
	EndpointAddresses      = "/rest/ip/address"
	EndpointDHCPServers    = "/rest/ip/dhcp-server"
	EndpointDHCPNetworks   = "/rest/ip/dhcp-server/network"
	EndpointDHCPLeases     = "/rest/ip/dhcp-server/lease"
	EndpointFirewallFilter = "/rest/ip/firewall/filter"
	EndpointUsers          = "/rest/user"
	EndpointRouterBOARD    = "/rest/system/routerboard"
	EndpointSystemResource = "/rest/system/resource"
	EndpointCPUResource    = "/rest/system/resource/cpu"
	EndpointNTPClient      = "/rest/system/ntp/client"
	EndpointClock          = "/rest/system/clock"
	EndpointLog            = "/rest/log"
)

var (
	InterfacesResource     = ResourceDescriptor{"interfaces", EndpointInterfaces}
	InterfaceListsResource = ResourceDescriptor{"interface-lists", EndpointInterfaceLists}
	EthernetResource       = ResourceDescriptor{"ethernet", EndpointEthernet}
	PPPActiveResource      = ResourceDescriptor{"ppp-active", EndpointPPPActive}
	PPPoEClientsResource   = ResourceDescriptor{"pppoe-clients", EndpointPPPoEClients}
	BridgesResource        = ResourceDescriptor{"bridges", EndpointBridges}
	BridgePortsResource    = ResourceDescriptor{"bridge-ports", EndpointBridgePorts}
	BridgeVLANsResource    = ResourceDescriptor{"bridge-vlans", EndpointBridgeVLANs}
	ARPResource            = ResourceDescriptor{"arp", EndpointARP}
	AddressesResource      = ResourceDescriptor{"addresses", EndpointAddresses}
	DHCPServersResource    = ResourceDescriptor{"dhcp-servers", EndpointDHCPServers}
	DHCPNetworksResource   = ResourceDescriptor{"dhcp-networks", EndpointDHCPNetworks}
	DHCPLeasesResource     = ResourceDescriptor{"dhcp-leases", EndpointDHCPLeases}
	FirewallFilterResource = ResourceDescriptor{"firewall-filter", EndpointFirewallFilter}
	UsersResource          = ResourceDescriptor{"users", EndpointUsers}
	LogResource            = ResourceDescriptor{"log", EndpointLog}
	CPUResource            = ResourceDescriptor{"cpu-resource", EndpointCPUResource}

	RouterBOARDResource = SystemResource{"routerboard", EndpointRouterBOARD}
	SystemResourceInfo  = SystemResource{"system-resource", EndpointSystemResource}
	NTPResource         = SystemResource{"ntp", EndpointNTPClient}
	ClockResource       = SystemResource{"clock", EndpointClock}
)

// CLIPath returns the RouterOS terminal path for a REST endpoint, for example
// /rest/ip/firewall/filter -> /ip/firewall/filter.
func CLIPath(endpoint string) string {
	return strings.TrimPrefix(endpoint, "/rest")
}

// EscapePathSegment escapes an opaque RouterOS identifier for one URL path
// segment.
func EscapePathSegment(value string) string {
	return url.PathEscape(value)
}

// ResourceRecordEndpoint returns the endpoint for a single record.
func ResourceRecordEndpoint(resource ResourceDescriptor, id string) string {
	return strings.TrimRight(resource.Endpoint, "/") + "/" + EscapePathSegment(id)
}
