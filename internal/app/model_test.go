package app

import (
	"context"
	"crypto/sha256"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/hafuta/mikrotik-tui/internal/config"
	"github.com/hafuta/mikrotik-tui/internal/credentials"
	"github.com/hafuta/mikrotik-tui/internal/routeros"
	"github.com/hafuta/mikrotik-tui/internal/ui"
)

type fakeClient struct {
	system routeros.Resource
	lists  map[string][]routeros.Resource
	err    error
}

func (f fakeClient) List(_ context.Context, resource routeros.ResourceDescriptor) ([]routeros.Resource, error) {
	if f.err != nil {
		return nil, f.err
	}
	return f.lists[resource.Endpoint], nil
}

func (f fakeClient) Get(context.Context, routeros.ResourceDescriptor, string) (routeros.Resource, error) {
	return routeros.Resource{}, errors.New("not implemented")
}

func (f fakeClient) System(context.Context, routeros.SystemResource) (routeros.Resource, error) {
	return f.system, f.err
}

type memoryProfiles struct{ profiles []config.Profile }

func (s *memoryProfiles) Load() ([]config.Profile, error) { return s.profiles, nil }
func (s *memoryProfiles) Save(profiles []config.Profile) error {
	s.profiles = append([]config.Profile(nil), profiles...)
	return nil
}

type memoryCredentials struct {
	value   credentials.Credential
	deleted bool
}

func (s *memoryCredentials) Get(context.Context, string) (credentials.Credential, error) {
	return s.value, nil
}
func (s *memoryCredentials) Put(_ context.Context, _ string, value credentials.Credential) error {
	s.value = value
	return nil
}
func (s *memoryCredentials) Delete(context.Context, string) error {
	s.value = credentials.Credential{}
	s.deleted = true
	return nil
}

func TestSavedProfileToDashboardAcrossBreakpoints(t *testing.T) {
	profileStore := &memoryProfiles{}
	credentialStore := &memoryCredentials{}
	profile := config.Profile{Name: "default", URL: "https://router.test", Username: "reader", CertificateFingerprint: strings.Repeat("ab", 32)}
	client := fakeClient{
		system: routeros.Resource{Fields: map[string]string{"board-name": "hEX S", "version": "7.23.3", "uptime": "1d"}},
		lists: map[string][]routeros.Resource{
			routeros.EndpointInterfaces: {
				{ID: "*1", Fields: map[string]string{"name": "ether1", "type": "ether", "running": "true", "mtu": "1500"}},
			},
		},
	}
	model := New(Options{
		Services: Services{
			Profiles:    profileStore,
			Credentials: credentialStore,
			NewClient: func(config.Profile, string) (routeros.Client, error) {
				return client, nil
			},
		},
		Profile:    &profile,
		Credential: credentials.Credential{Password: "marker-secret"},
	})

	connect := model.Init()
	message := connect().(connectMsg)
	updated, command := model.Update(message)
	model = updated.(Model)
	if model.screen != screenDashboard || command == nil {
		t.Fatalf("expected dashboard and initial resource command, got screen=%v command=%v", model.screen, command)
	}
	updated, _ = model.Update(command())
	model = updated.(Model)
	if !model.dashboard || model.trafficInterface != "ether1" {
		t.Fatalf("expected WAN dashboard, dashboard=%v interface=%q", model.dashboard, model.trafficInterface)
	}
	if len(profileStore.profiles) != 1 || credentialStore.value.Password != "marker-secret" {
		t.Fatal("connection was not persisted")
	}

	for _, size := range []tea.WindowSizeMsg{{Width: 60, Height: 18}, {Width: 90, Height: 24}, {Width: 140, Height: 38}} {
		updated, _ = model.Update(size)
		view := updated.(Model).View()
		for _, expected := range []string{"ROUTERDECK", "hEX S", "DASHBOARD"} {
			if !strings.Contains(view, expected) {
				t.Fatalf("%dx%d view missing %q", size.Width, size.Height, expected)
			}
		}
	}
}

func TestFirstLoginRequiresCertificateApproval(t *testing.T) {
	pin := strings.Repeat("12", 32)
	trustedClient := fakeClient{
		system: routeros.Resource{Fields: map[string]string{"board-name": "router"}},
		lists:  map[string][]routeros.Resource{routeros.EndpointInterfaces: {}},
	}
	model := New(Options{Services: Services{
		NewClient: func(profile config.Profile, _ string) (routeros.Client, error) {
			if profile.CertificateFingerprint == pin {
				return trustedClient, nil
			}
			return fakeClient{err: &routeros.APIError{Kind: routeros.ErrorTLS, Message: "unknown authority"}}, nil
		},
		Probe: func(context.Context, string) (string, error) { return pin, nil },
	}})
	model.login.SetValues(ui.Credentials{Address: "https://192.168.88.1:8443", Username: "reader", Password: "secret"})
	for range 2 {
		updated, _ := model.Update(tea.KeyMsg{Type: tea.KeyTab})
		model = updated.(Model)
	}
	updated, command := model.Update(tea.KeyMsg{Type: tea.KeyEnter})
	model = updated.(Model)
	if model.screen != screenConnecting || command == nil {
		t.Fatal("login did not start connection")
	}
	updated, _ = model.Update(command())
	model = updated.(Model)
	if model.screen != screenTrust || !strings.Contains(model.View(), "1212 1212") {
		t.Fatalf("expected trust screen, got %v", model.screen)
	}
	updated, command = model.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("y")})
	model = updated.(Model)
	if model.profile.CertificateFingerprint != pin || command == nil {
		t.Fatal("approved pin was not applied")
	}
	updated, _ = model.Update(command())
	model = updated.(Model)
	if model.screen != screenDashboard {
		t.Fatalf("expected connected dashboard, got %v", model.screen)
	}
}

func TestResourceSpecificationsAreCompleteAndUnique(t *testing.T) {
	seen := map[string]bool{}
	paths := map[string]bool{}
	for _, spec := range resourceSpecs {
		if spec.ID == "" || spec.Label == "" || len(spec.Columns) == 0 || spec.Refresh <= 0 {
			t.Fatalf("incomplete resource spec: %#v", spec)
		}
		if seen[spec.ID] {
			t.Fatalf("duplicate resource ID %q", spec.ID)
		}
		seen[spec.ID] = true
		if spec.System == nil && !strings.HasPrefix(spec.Resource.Endpoint, "/rest/") {
			t.Fatalf("invalid endpoint for %s: %s", spec.ID, spec.Resource.Endpoint)
		}
		path := spec.CLIPath()
		if !strings.HasPrefix(path, "/") || strings.Contains(path, "/rest/") || strings.Contains(path, "//") {
			t.Fatalf("invalid RouterOS path for %s: %s", spec.ID, path)
		}
		if paths[path] {
			t.Fatalf("duplicate RouterOS path %q", path)
		}
		paths[path] = true
	}
	if len(seen) != 19 {
		t.Fatalf("expected 19 resource screens, got %d", len(seen))
	}
}

func TestDashboardRenderGoldens(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.profile = config.Profile{URL: "https://192.168.88.1:8443", Username: "reader"}
	model.router = routeros.Resource{Fields: map[string]string{
		"board-name": "hEX S", "version": "7.23.3", "uptime": "1d2h3m",
	}}
	model.activateDashboard()
	model.loading = false
	model.trafficHasBase = true
	model.trafficInterface = "pppoe-out2"
	model.trafficRXBytes = 748_478_759_768
	model.trafficTXBytes = 63_870_706_777
	model.trafficRXRate = 82_400_000
	model.trafficTXRate = 7_800_000
	model.trafficSamples = []ui.TrafficSample{
		{RX: 22_000_000, TX: 4_000_000},
		{RX: 68_000_000, TX: 12_000_000},
		{RX: 31_000_000, TX: 8_000_000},
		{RX: 82_400_000, TX: 7_800_000},
	}
	model.updateSystemTelemetry(routeros.Resource{Fields: map[string]string{
		"total-memory": "268435456", "free-memory": "171966464",
	}})
	model.updateCPUTelemetry([]routeros.Resource{
		{ID: "*0", Fields: map[string]string{"cpu": "cpu0", "load": "18"}},
		{ID: "*1", Fields: map[string]string{"cpu": "cpu1", "load": "64"}},
		{ID: "*2", Fields: map[string]string{"cpu": "cpu2", "load": "7"}},
		{ID: "*3", Fields: map[string]string{"cpu": "cpu3", "load": "91"}},
	}, routeros.Resource{})
	model.updateFirewallTelemetry([]routeros.Resource{
		{ID: "*1", Fields: map[string]string{"chain": "forward", "action": "accept", "comment": "established", "packets": "1000", "bytes": "900000"}},
		{ID: "*2", Fields: map[string]string{"chain": "input", "action": "drop", "comment": "invalid", "packets": "20", "bytes": "1200"}},
		{ID: "*3", Fields: map[string]string{"chain": "input", "action": "drop", "comment": "unused legacy", "packets": "0", "bytes": "0"}},
	})
	model.updateFirewallTelemetry([]routeros.Resource{
		{ID: "*1", Fields: map[string]string{"chain": "forward", "action": "accept", "comment": "established", "packets": "1250", "bytes": "1125000"}},
		{ID: "*2", Fields: map[string]string{"chain": "input", "action": "drop", "comment": "invalid", "packets": "22", "bytes": "1320"}},
		{ID: "*3", Fields: map[string]string{"chain": "input", "action": "drop", "comment": "unused legacy", "packets": "0", "bytes": "0"}},
	})
	model.status = ui.Status{Text: "WAN telemetry live · pppoe-out2", Kind: ui.Success}
	model.focus = 1

	goldens := map[int]string{
		60:  "09545545f94e4b7310083124a238b8251a19fff34e48e6d3337364b66ee6c156",
		90:  "253a242863b9b901519ecaf0679d087fb522f08f380ec45208cc7836ef76d350",
		140: "6db075a07dd51a26094c5d0e86b46d3fe2e8bddbb520d56da9ac9e4f6d8f8237",
	}
	for width, expected := range goldens {
		model.resize(width, 28)
		rendered := ansi.Strip(model.View())
		actual := fmt.Sprintf("%x", sha256.Sum256([]byte(rendered)))
		if actual != expected {
			t.Errorf("width %d golden = %s, want %s", width, actual, expected)
		}
	}
}

func TestWANDetectionPrefersRunningPPPoE(t *testing.T) {
	records := []routeros.Resource{
		{ID: "*1", Fields: map[string]string{
			"name": "ether1", "type": "ether", "running": "true",
			"rx-byte": "9000000", "tx-byte": "1000000",
		}},
		{ID: "*2", Fields: map[string]string{
			"name": "pppoe-out2", "type": "pppoe-out", "running": "true",
			"rx-byte": "100", "tx-byte": "200",
		}},
		{ID: "*3", Fields: map[string]string{
			"name": "wan-backup", "type": "ether", "running": "false",
		}},
	}
	selected, err := selectWANInterface(records)
	if err != nil {
		t.Fatal(err)
	}
	if selected.Fields["name"] != "pppoe-out2" {
		t.Fatalf("selected WAN = %q", selected.Fields["name"])
	}
}

func TestTrafficCounterRateHandlesReset(t *testing.T) {
	if got := counterRate(1_000, 3_000, 2); got != 8_000 {
		t.Fatalf("counter rate = %.0f, want 8000", got)
	}
	if got := counterRate(3_000, 1_000, 2); got != 0 {
		t.Fatalf("reset counter rate = %.0f, want 0", got)
	}
}

func TestDashboardSystemAndFirewallTelemetryHistory(t *testing.T) {
	model := New(Options{})
	model.updateSystemTelemetry(routeros.Resource{Fields: map[string]string{
		"total-memory": "1000", "free-memory": "250",
	}})
	model.updateCPUTelemetry([]routeros.Resource{
		{ID: "*0", Fields: map[string]string{"cpu": "cpu0", "load": "25"}},
		{ID: "*1", Fields: map[string]string{"cpu": "cpu1", "load": "80"}},
	}, routeros.Resource{})
	if model.memoryUsedBytes != 750 || len(model.memorySamples) != 1 || model.memorySamples[0] != 75 {
		t.Fatalf("memory telemetry = used:%d samples:%v", model.memoryUsedBytes, model.memorySamples)
	}
	if len(model.cpuCoreOrder) != 2 || model.cpuCoreLoads["cpu1"] != 80 {
		t.Fatalf("CPU telemetry = order:%v loads:%v", model.cpuCoreOrder, model.cpuCoreLoads)
	}

	first := []routeros.Resource{
		{ID: "*1", Fields: map[string]string{"action": "accept", "comment": "active", "packets": "100", "bytes": "10000"}},
		{ID: "*2", Fields: map[string]string{"action": "drop", "comment": "dead", "packets": "0", "bytes": "0"}},
	}
	model.updateFirewallTelemetry(first)
	second := []routeros.Resource{
		{ID: "*1", Fields: map[string]string{"action": "accept", "comment": "active", "packets": "115", "bytes": "25000"}},
		{ID: "*2", Fields: map[string]string{"action": "drop", "comment": "dead", "packets": "0", "bytes": "0"}},
	}
	model.updateFirewallTelemetry(second)
	if len(model.firewallRules) != 2 {
		t.Fatalf("firewall rule count = %d", len(model.firewallRules))
	}
	if got := model.firewallRules[0]; got.RecentPackets != 15 || got.RecentBytes != 15000 || len(got.History) != 2 {
		t.Fatalf("active firewall telemetry = %#v", got)
	}
	if got := model.firewallRules[1]; got.Packets != 0 || got.RecentPackets != 0 {
		t.Fatalf("dead firewall telemetry = %#v", got)
	}
}

func TestReturningToDashboardPreservesRecentHistoryWithoutGapSpike(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.client = fakeClient{}
	model.trafficInterface = "pppoe-out2"
	model.trafficSamples = []ui.TrafficSample{
		{RX: 10_000_000, TX: 1_000_000},
		{RX: 12_000_000, TX: 2_000_000},
	}
	model.trafficRXBytes = 1_000
	model.trafficTXBytes = 500
	model.trafficUpdated = time.Now().Add(-10 * time.Minute)
	model.trafficHasBase = true

	model.activateDashboard()
	if len(model.trafficSamples) != 2 || model.trafficHasBase {
		t.Fatal("dashboard return discarded history or retained a stale counter baseline")
	}
	first := trafficMsg{
		requestID:  model.requestID,
		generation: model.pollGeneration,
		record: routeros.Resource{Fields: map[string]string{
			"name": "pppoe-out2", "rx-byte": "1000000", "tx-byte": "500000",
		}},
		at: time.Now(),
	}
	updated, _ := model.Update(first)
	model = updated.(Model)
	if len(model.trafficSamples) != 2 || !model.trafficHasBase {
		t.Fatal("resume baseline replayed or replaced recent samples")
	}
	second := first
	second.record.Fields = map[string]string{
		"name": "pppoe-out2", "rx-byte": "1250000", "tx-byte": "625000",
	}
	second.at = first.at.Add(2 * time.Second)
	updated, _ = model.Update(second)
	model = updated.(Model)
	if len(model.trafficSamples) != 3 {
		t.Fatalf("new live sample count = %d, want 3", len(model.trafficSamples))
	}
	if model.trafficRXRate != 1_000_000 || model.trafficTXRate != 500_000 {
		t.Fatalf("resumed rates RX/TX = %.0f/%.0f", model.trafficRXRate, model.trafficTXRate)
	}
}

func TestStaleDashboardTickerCannotCreateParallelPollingLoop(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.client = fakeClient{lists: map[string][]routeros.Resource{
		routeros.EndpointInterfaces: {
			{ID: "*1", Fields: map[string]string{
				"name": "pppoe-out1", "type": "pppoe-out", "running": "true",
			}},
		},
	}}
	model.activateDashboard()
	oldGeneration := model.pollGeneration
	spec, _ := specByID("interfaces")
	model.activate(spec)
	model.activateDashboard()
	currentGeneration := model.pollGeneration
	if currentGeneration == oldGeneration {
		t.Fatal("dashboard generation did not advance across navigation")
	}

	updated, command := model.Update(trafficTickMsg{generation: oldGeneration})
	model = updated.(Model)
	if command != nil || model.refreshing {
		t.Fatal("stale dashboard tick started a second polling loop")
	}
	updated, command = model.Update(trafficTickMsg{generation: currentGeneration})
	model = updated.(Model)
	if command == nil || !model.refreshing {
		t.Fatal("current dashboard tick did not refresh telemetry")
	}
}

func TestSensitiveRouterFieldsAreMaskedInTablesAndInspector(t *testing.T) {
	const marker = "pppoe-password-marker"
	record := routeros.Resource{ID: "*1", Fields: map[string]string{
		"name": "pppoe-out1", "user": "subscriber", "password": marker,
	}}
	inspector := ansi.Strip(formatFields(record))
	if strings.Contains(inspector, marker) || !strings.Contains(inspector, "••••••••") {
		t.Fatalf("inspector did not mask password: %q", inspector)
	}
	rows := toRows([]routeros.Resource{record})
	if len(rows) != 1 || rows[0].Cells["password"] != "••••••••" {
		t.Fatalf("table password = %q", rows[0].Cells["password"])
	}
	for _, key := range []string{"password", "user-password", "secret", "private-key", "pre_shared_key"} {
		if !sensitiveResourceField(key) {
			t.Fatalf("sensitive field %q was not recognized", key)
		}
	}
}

func TestRouterLogPauseAndResume(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.client = fakeClient{lists: map[string][]routeros.Resource{routeros.EndpointLog: {}}}
	spec, _ := specByID("logs")
	model.activate(spec)
	model.loading = false
	model.focus = 1

	updated, command := model.Update(tea.KeyMsg{Type: tea.KeySpace})
	model = updated.(Model)
	if !model.logsPaused || command != nil {
		t.Fatal("space did not pause log polling")
	}
	updated, command = model.Update(refreshMsg{specID: "logs"})
	model = updated.(Model)
	if command == nil {
		t.Fatal("paused view stopped background log ingestion")
	}
	updated, command = model.Update(tea.KeyMsg{Type: tea.KeySpace})
	model = updated.(Model)
	if model.logsPaused || command != nil {
		t.Fatal("space did not resume the live log view")
	}
}

func TestLogStreamDeduplicatesAndTracksUnreadEvents(t *testing.T) {
	model := New(Options{})
	spec, _ := specByID("logs")
	model.activate(spec)
	model.loading = false

	model.mergeLogRecords([]routeros.Resource{
		{ID: "*1", Fields: map[string]string{"time": "10:00:00", "topics": "system,info", "message": "boot"}},
		{ID: "*2", Fields: map[string]string{"time": "10:00:01", "topics": "interface,info", "message": "up"}},
	})
	if len(model.records) != 2 || model.table.SelectedID != "*2" || model.logUnread != 0 {
		t.Fatalf("initial follow state = records:%d selected:%q unread:%d", len(model.records), model.table.SelectedID, model.logUnread)
	}

	model.logFollow = false
	model.mergeLogRecords([]routeros.Resource{
		{ID: "*2", Fields: map[string]string{"time": "10:00:01", "topics": "interface,info", "message": "updated"}},
		{ID: "*3", Fields: map[string]string{"time": "10:00:02", "topics": "system,warning", "message": "warm"}},
	})
	if len(model.records) != 3 || model.logUnread != 1 || model.table.SelectedID != "*2" {
		t.Fatalf("detached state = records:%d selected:%q unread:%d", len(model.records), model.table.SelectedID, model.logUnread)
	}
	if got := model.records[1].Fields["message"]; got != "updated" {
		t.Fatalf("updated duplicate message = %q", got)
	}
}

func TestLogStreamDoesNotReplayRecordsOutsideBoundedBuffer(t *testing.T) {
	model := New(Options{})
	spec, _ := specByID("logs")
	model.activate(spec)
	model.loading = false

	snapshot := make([]routeros.Resource, 1000)
	for index := range snapshot {
		snapshot[index] = routeros.Resource{
			ID: fmt.Sprintf("*%x", index),
			Fields: map[string]string{
				"time":    fmt.Sprintf("2026-08-%02d 10:00:00", 1+index/100),
				"topics":  "system,info",
				"message": fmt.Sprintf("event %d", index),
			},
		}
	}
	model.mergeLogRecords(snapshot)
	if len(model.records) != 500 || rowID(model.records[0]) != "*1f4" || rowID(model.records[499]) != "*3e7" {
		t.Fatalf("initial bounded window = %d records, %q through %q", len(model.records), rowID(model.records[0]), rowID(model.records[len(model.records)-1]))
	}

	model.mergeLogRecords(snapshot)
	if len(model.records) != 500 || rowID(model.records[0]) != "*1f4" || rowID(model.records[499]) != "*3e7" {
		t.Fatalf("repeated full snapshot replayed old logs: %q through %q", rowID(model.records[0]), rowID(model.records[len(model.records)-1]))
	}
}

func TestLogSeverityAndLocalClear(t *testing.T) {
	model := New(Options{})
	spec, _ := specByID("logs")
	model.activate(spec)
	model.loading = false
	model.mergeLogRecords([]routeros.Resource{
		{ID: "*1", Fields: map[string]string{"topics": "system,info", "message": "ready"}},
		{ID: "*2", Fields: map[string]string{"topics": "interface,warning", "message": "flap"}},
		{ID: "*3", Fields: map[string]string{"topics": "critical,error", "message": "failed"}},
	})

	model.logSeverity = "error"
	model.applyLogRows()
	if rows := model.table.VisibleRows(); len(rows) != 1 || rows[0].ID != "*3" {
		t.Fatalf("error filter rows = %#v", rows)
	}

	model.clearLocalLogs()
	model.mergeLogRecords([]routeros.Resource{
		{ID: "*1", Fields: map[string]string{"topics": "system,info", "message": "ready"}},
		{ID: "*4", Fields: map[string]string{"topics": "system,error", "message": "new"}},
	})
	if len(model.records) != 1 || rowID(model.records[0]) != "*4" {
		t.Fatalf("cleared stream replayed old records: %#v", model.records)
	}
}

func TestBackgroundRefreshKeepsCurrentTableVisible(t *testing.T) {
	model := New(Options{})
	model.client = fakeClient{lists: map[string][]routeros.Resource{
		routeros.EndpointInterfaces: {
			{ID: "*1", Fields: map[string]string{"name": "ether1", "running": "true"}},
		},
	}}
	spec, _ := specByID("interfaces")
	model.activate(spec)
	model.records = []routeros.Resource{
		{ID: "*1", Fields: map[string]string{"name": "ether1", "running": "true"}},
	}
	model.table.SetRows(toRows(model.records))
	model.loading = false

	command := model.loadResourcesCmd(spec)
	if command == nil || model.loading || !model.refreshing {
		t.Fatalf("refresh state loading=%v refreshing=%v command=%v", model.loading, model.refreshing, command)
	}
	if view := model.tableOrState(); strings.Contains(view, "Reading") || !strings.Contains(view, "ether1") {
		t.Fatalf("background refresh replaced stable content: %q", view)
	}

	updated, _ := model.Update(command())
	model = updated.(Model)
	if model.loading || model.refreshing {
		t.Fatalf("completed refresh state loading=%v refreshing=%v", model.loading, model.refreshing)
	}
}

func TestPaneGeometryHasPaddingAndEqualHeights(t *testing.T) {
	layout := paneLayoutFor(140, 28)
	if layout.navigationWidth != 28 {
		t.Fatalf("wide navigation width = %d, want 28", layout.navigationWidth)
	}
	panes := []struct {
		title string
		width int
	}{
		{title: "NAVIGATION", width: layout.navigationWidth},
		{title: "INTERFACES", width: layout.tableWidth},
		{title: "INSPECTOR", width: layout.inspectorWidth},
	}
	for index, definition := range panes {
		fullLine := strings.Repeat("x", innerPaneWidth(definition.width))
		content := strings.Repeat(fullLine+"\n", layout.contentHeight-1) + fullLine
		pane := panel(definition.title, content, definition.width, layout.height, index == 0)
		if got := lipgloss.Height(pane); got != layout.height {
			t.Fatalf("pane %d height = %d, want %d", index, got, layout.height)
		}
		if got := lipgloss.Width(pane); got != definition.width {
			t.Fatalf("pane %d width = %d, want %d", index, got, definition.width)
		}
		lines := strings.Split(ansi.Strip(pane), "\n")
		if len(lines) < 2 || !strings.HasPrefix(lines[1], "│ ") {
			t.Fatalf("pane %d is missing left padding: %q", index, lines)
		}
	}
}

func TestPanelClipsOverflowWithoutDisplacingNavigation(t *testing.T) {
	const height = 12
	nav := panel("NAVIGATION", "Dashboard\nInterfaces\nLogs", 28, height, true)
	tallContent := strings.Repeat("telemetry row\n", 40)
	main := panel("DASHBOARD", tallContent, 70, height, false)
	body := lipgloss.JoinHorizontal(lipgloss.Top, nav, main)
	if got := lipgloss.Height(body); got != height {
		t.Fatalf("overflowing dashboard changed body height to %d, want %d", got, height)
	}
	lines := strings.Split(ansi.Strip(body), "\n")
	if len(lines) < 2 || !strings.Contains(lines[1], "NAVIGATION") {
		t.Fatalf("navigation top was displaced: %q", lines)
	}
}

func TestDashboardSectionsKeepStableGeometryWhileLoading(t *testing.T) {
	for _, dimensions := range []struct{ width, height int }{
		{44, 8}, {60, 16}, {90, 20},
	} {
		model := New(Options{})
		model.loading = true
		loading := model.dashboardContent(dimensions.width, dimensions.height)

		model.loading = false
		model.trafficHasBase = true
		model.trafficInterface = "ether1"
		model.trafficSamples = []ui.TrafficSample{{RX: 1000, TX: 500}, {RX: 2000, TX: 800}}
		model.updateSystemTelemetry(routeros.Resource{Fields: map[string]string{
			"total-memory": "1000", "free-memory": "400",
		}})
		model.updateCPUTelemetry([]routeros.Resource{
			{ID: "*0", Fields: map[string]string{"cpu": "cpu0", "load": "10"}},
			{ID: "*1", Fields: map[string]string{"cpu": "cpu1", "load": "20"}},
			{ID: "*2", Fields: map[string]string{"cpu": "cpu2", "load": "30"}},
			{ID: "*3", Fields: map[string]string{"cpu": "cpu3", "load": "40"}},
		}, routeros.Resource{})
		model.updateFirewallTelemetry([]routeros.Resource{
			{ID: "*1", Fields: map[string]string{"action": "accept", "packets": "10", "bytes": "1000"}},
		})
		loaded := model.dashboardContent(dimensions.width, dimensions.height)

		for state, view := range map[string]string{"loading": loading, "loaded": loaded} {
			if got := lipgloss.Height(view); got != dimensions.height {
				t.Errorf("%s dashboard at %dx%d has height %d", state, dimensions.width, dimensions.height, got)
			}
			if got := lipgloss.Width(view); got > dimensions.width {
				t.Errorf("%s dashboard at %dx%d has width %d", state, dimensions.width, dimensions.height, got)
			}
		}
	}
}

func TestDashboardFillsTerminalAndPrefersTrafficHeight(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.activateDashboard()
	model.loading = false
	model.trafficHasBase = true
	model.trafficInterface = "pppoe-out2"
	model.trafficSamples = []ui.TrafficSample{{RX: 1_000_000, TX: 500_000}, {RX: 2_000_000, TX: 800_000}}
	model.updateCPUTelemetry([]routeros.Resource{
		{ID: "*0", Fields: map[string]string{"cpu": "cpu0", "load": "10"}},
		{ID: "*1", Fields: map[string]string{"cpu": "cpu1", "load": "20"}},
		{ID: "*2", Fields: map[string]string{"cpu": "cpu2", "load": "30"}},
		{ID: "*3", Fields: map[string]string{"cpu": "cpu3", "load": "40"}},
	}, routeros.Resource{})
	model.updateSystemTelemetry(routeros.Resource{Fields: map[string]string{
		"total-memory": "1000", "free-memory": "400",
	}})
	rules := make([]routeros.Resource, 14)
	for index := range rules {
		rules[index] = routeros.Resource{
			ID: fmt.Sprintf("*%d", index+1),
			Fields: map[string]string{
				"action": "accept", "comment": fmt.Sprintf("rule %d", index),
				"packets": "10", "bytes": "1000",
			},
		}
	}
	model.updateFirewallTelemetry(rules)

	const width, height = 140, 28
	model.resize(width, height)
	layout := paneLayoutFor(width, height)
	if layout.navigationWidth+layout.tableWidth+layout.inspectorWidth != width {
		t.Fatalf("panes leave a horizontal gutter: nav=%d table=%d inspector=%d width=%d",
			layout.navigationWidth, layout.tableWidth, layout.inspectorWidth, width)
	}
	if layout.height != height-3 {
		t.Fatalf("body height = %d, want %d", layout.height, height-3)
	}
	view := ansi.Strip(model.View())
	if got := lipgloss.Width(view); got != width {
		t.Fatalf("dashboard width = %d, want %d", got, width)
	}
	if got := lipgloss.Height(view); got != height {
		t.Fatalf("dashboard height = %d, want %d", got, height)
	}

	geometry := model.dashboardGeometry(100, 24)
	if geometry.wanHeight <= geometry.cpuHeight {
		t.Fatalf("WAN chart should receive leftover height: %+v", geometry)
	}
	if geometry.firewallHeight > 1+ui.MaxFirewallRules {
		t.Fatalf("firewall pane exceeded 10-rule cap: %+v", geometry)
	}
}

func TestDashboardFirewallScrollsWithKeys(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.dashboard = true
	model.focus = 1
	model.resize(120, 30)
	for index := 0; index < 14; index++ {
		model.firewallRules = append(model.firewallRules, ui.FirewallRuleMetric{
			ID: fmt.Sprintf("*%d", index+1), Label: fmt.Sprintf("rule-%02d", index+1),
			Action: "accept", Packets: uint64(index), History: []float64{float64(index)},
		})
	}
	updated, _ := model.Update(tea.KeyMsg{Type: tea.KeyDown})
	model = updated.(Model)
	if model.firewallOffset != 1 {
		t.Fatalf("down did not scroll firewall, offset=%d", model.firewallOffset)
	}
}

func TestHeaderShowsUserAndLogoutForgetsSession(t *testing.T) {
	profileStore := &memoryProfiles{profiles: []config.Profile{{
		Name: "default", URL: "https://router.test", Username: "reader",
	}}}
	credentialStore := &memoryCredentials{value: credentials.Credential{Password: "secret"}}
	profile := profileStore.profiles[0]
	model := New(Options{
		Services: Services{Profiles: profileStore, Credentials: credentialStore},
		Profile:  &profile,
	})
	model.screen = screenDashboard
	model.client = fakeClient{}
	model.sessionStart = time.Now().Add(-time.Minute)

	header := ansi.Strip(model.headerView())
	if !strings.Contains(header, "user reader") || !strings.Contains(header, "session 1m") {
		t.Fatalf("session identity missing from header: %q", header)
	}

	updated, command := model.Update(tea.KeyMsg{Type: tea.KeyCtrlL})
	model = updated.(Model)
	if model.screen != screenLogin || model.client != nil || !model.sessionStart.IsZero() {
		t.Fatalf("logout did not clear active session: %#v", model)
	}
	values := model.login.Values()
	if values.Address != "https://router.test" || values.Username != "reader" || values.Password != "" {
		t.Fatalf("logout login values = %#v", values)
	}
	if command == nil {
		t.Fatal("logout did not schedule saved-session cleanup")
	}
	updated, _ = model.Update(command())
	model = updated.(Model)
	if len(profileStore.profiles) != 0 || !credentialStore.deleted || credentialStore.value.Password != "" {
		t.Fatal("logout did not remove persisted profile and credentials")
	}
}

func TestHelpUsesCenteredFixedCanvasModal(t *testing.T) {
	model := New(Options{})
	model.screen = screenDashboard
	model.resize(90, 24)
	model.help.Visible = true
	rendered := model.View()
	if lipgloss.Width(rendered) != 90 || lipgloss.Height(rendered) != 24 {
		t.Fatalf("help changed canvas to %dx%d", lipgloss.Width(rendered), lipgloss.Height(rendered))
	}
	lines := strings.Split(ansi.Strip(rendered), "\n")
	top := -1
	for index, line := range lines {
		if strings.Contains(line, "╭") {
			top = index
			break
		}
	}
	if top <= 0 || top >= len(lines)/2 {
		t.Fatalf("help modal is not vertically centered; top=%d", top)
	}
	if !strings.Contains(ansi.Strip(rendered), "Keyboard help") {
		t.Fatal("help modal content is missing")
	}
}

func TestPaletteCommandsCoverEveryRouterOSPath(t *testing.T) {
	commands := paletteCommands()
	byID := map[string]ui.Command{}
	for _, command := range commands {
		if _, exists := byID[command.ID]; exists {
			t.Fatalf("duplicate palette command %q", command.ID)
		}
		byID[command.ID] = command
	}
	if byID["dashboard"].Title != "Dashboard" {
		t.Fatalf("dashboard command = %#v", byID["dashboard"])
	}
	for _, spec := range resourceSpecs {
		command, ok := byID[spec.ID]
		if !ok {
			t.Fatalf("missing palette command for %s", spec.ID)
		}
		if command.Path != spec.CLIPath() || command.Title != spec.CLIPath() {
			t.Fatalf("palette path for %s = %q title=%q", spec.ID, command.Path, command.Title)
		}
	}
}

func TestCommandPaletteNavigatesToRouterOSPath(t *testing.T) {
	client := fakeClient{
		system: routeros.Resource{Fields: map[string]string{"board-name": "hEX S"}},
		lists: map[string][]routeros.Resource{
			routeros.EndpointARP: {
				{ID: "*1", Fields: map[string]string{"address": "192.0.2.1", "interface": "ether1"}},
			},
			routeros.EndpointFirewallFilter: {
				{ID: "*1", Fields: map[string]string{"chain": "forward", "action": "accept"}},
			},
		},
	}
	model := New(Options{Services: Services{
		NewClient: func(config.Profile, string) (routeros.Client, error) { return client, nil },
	}})
	model.client = client
	model.screen = screenDashboard
	model.activateDashboard()
	model.loading = false

	updated, _ := model.Update(tea.KeyMsg{Type: tea.KeyCtrlP})
	model = updated.(Model)
	if !model.palette.Visible {
		t.Fatal("ctrl+p did not open command palette")
	}
	for _, r := range "/IP/firewall" {
		updated, _ = model.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{r}})
		model = updated.(Model)
	}
	view := ansi.Strip(model.palette.View())
	if !strings.Contains(view, "/ip/firewall/filter") {
		t.Fatalf("palette missing firewall path: %q", view)
	}
	if strings.Contains(view, "/ip/arp") {
		t.Fatalf("unrelated /ip path remained visible: %q", view)
	}

	updated, command := model.Update(tea.KeyMsg{Type: tea.KeyEnter})
	model = updated.(Model)
	if model.palette.Visible || command == nil {
		t.Fatal("enter did not close palette and return navigation")
	}
	updated, load := model.Update(command())
	model = updated.(Model)
	if model.dashboard || model.active.ID != "firewall-filter" {
		t.Fatalf("active=%q dashboard=%v", model.active.ID, model.dashboard)
	}
	if model.navigation.Selected != "firewall-filter" {
		t.Fatalf("navigation selected %q", model.navigation.Selected)
	}
	if !strings.Contains(model.navigation.View(), "Firewall") {
		t.Fatal("firewall page was not revealed in the sidebar")
	}
	if load == nil {
		t.Fatal("expected firewall resource load")
	}
}
