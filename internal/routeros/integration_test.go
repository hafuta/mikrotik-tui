package routeros

import (
	"context"
	"os"
	"testing"
	"time"
)

// TestReadOnlyHardware is opt-in and never mutates RouterOS state.
func TestReadOnlyHardware(t *testing.T) {
	if os.Getenv("MIKROTIK_TUI_INTEGRATION") != "1" {
		t.Skip("set MIKROTIK_TUI_INTEGRATION=1 to test a real router")
	}
	baseURL := os.Getenv("MIKROTIK_TUI_URL")
	username := os.Getenv("MIKROTIK_TUI_USERNAME")
	password := os.Getenv("MIKROTIK_TUI_PASSWORD")
	if baseURL == "" || username == "" {
		t.Fatal("MIKROTIK_TUI_URL and MIKROTIK_TUI_USERNAME are required")
	}
	pin := os.Getenv("MIKROTIK_TUI_CERT_FINGERPRINT")
	if pin == "" {
		var err error
		pin, err = ProbeCertificate(context.Background(), baseURL)
		if err != nil {
			t.Fatalf("probe certificate: %v", err)
		}
	}
	client, err := NewClient(ClientOptions{
		BaseURL:        baseURL,
		Username:       username,
		Password:       password,
		CertificatePin: pin,
		RequestTimeout: 10 * time.Second,
	})
	if err != nil {
		t.Fatal(err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	system, err := client.System(ctx, SystemResourceInfo)
	if err != nil {
		t.Fatalf("system resource: %v", err)
	}
	if system.Fields["version"] == "" || system.Fields["board-name"] == "" {
		t.Fatalf("incomplete system resource: %#v", system.Fields)
	}
	for _, resource := range []ResourceDescriptor{
		InterfacesResource, InterfaceListsResource, EthernetResource,
		PPPActiveResource, PPPoEClientsResource,
		BridgesResource, BridgePortsResource, BridgeVLANsResource,
		ARPResource, AddressesResource, DHCPServersResource,
		DHCPNetworksResource, DHCPLeasesResource, FirewallFilterResource,
		UsersResource, LogResource, CPUResource,
	} {
		if _, err := client.List(ctx, resource); err != nil {
			t.Errorf("%s: %v", resource.Name, err)
		}
	}
	for _, resource := range []SystemResource{RouterBOARDResource, NTPResource, ClockResource} {
		if _, err := client.System(ctx, resource); err != nil {
			t.Errorf("%s: %v", resource.Name, err)
		}
	}
}
