// Package app coordinates the Bubble Tea state machine and application services.
package app

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/url"
	"sort"
	"strconv"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/hafuta/mikrotik-tui/internal/config"
	"github.com/hafuta/mikrotik-tui/internal/credentials"
	"github.com/hafuta/mikrotik-tui/internal/routeros"
	"github.com/hafuta/mikrotik-tui/internal/theme"
	"github.com/hafuta/mikrotik-tui/internal/ui"
)

type screen uint8

const (
	screenLogin screen = iota
	screenConnecting
	screenTrust
	screenDashboard
)

type Options struct {
	Services   Services
	Profile    *config.Profile
	Credential credentials.Credential
}

type Model struct {
	services Services
	screen   screen
	width    int
	height   int
	profile  config.Profile
	password string
	client   routeros.Client
	router   routeros.Resource

	login        ui.LoginInput
	navigation   ui.Navigation
	table        ui.ResourceTable
	inspector    ui.Inspector
	help         ui.HelpOverlay
	palette      ui.CommandPalette
	status       ui.Status
	focus        int
	active       resourceSpec
	records      []routeros.Resource
	loading      bool
	refreshing   bool
	requestID    uint64
	lastRefresh  time.Time
	sessionStart time.Time
	trustPin     string
	logsPaused   bool
	logFollow    bool
	logUnread    int
	logSeverity  string
	logIgnored   map[string]struct{}
	logSeen      map[string]struct{}

	dashboard        bool
	trafficInterface string
	trafficSamples   []ui.TrafficSample
	trafficRXBytes   uint64
	trafficTXBytes   uint64
	trafficRXRate    float64
	trafficTXRate    float64
	trafficUpdated   time.Time
	trafficHasBase   bool
	pollGeneration   uint64

	cpuCoreOrder     []string
	cpuCoreSamples   map[string][]float64
	cpuCoreLoads     map[string]float64
	memorySamples    []float64
	memoryUsedBytes  uint64
	memoryTotalBytes uint64
	firewallPrevious map[string]firewallCounter
	firewallRules    []ui.FirewallRuleMetric
	firewallHasBase  bool
	firewallOffset   int
}

type firewallCounter struct {
	packets uint64
	bytes   uint64
}

type connectMsg struct {
	client      routeros.Client
	router      routeros.Resource
	fingerprint string
	err         error
}

type resourcesMsg struct {
	requestID uint64
	specID    string
	records   []routeros.Resource
	err       error
}

type refreshMsg struct {
	specID     string
	generation uint64
}
type logoutMsg struct{}
type logoutResultMsg struct{ err error }
type navigateMsg struct{ id string }
type trafficTickMsg struct{ generation uint64 }
type trafficMsg struct {
	requestID   uint64
	generation  uint64
	record      routeros.Resource
	system      routeros.Resource
	cores       []routeros.Resource
	firewall    []routeros.Resource
	at          time.Time
	err         error
	systemErr   error
	coresErr    error
	firewallErr error
}

func New(options Options) Model {
	if options.Services.Logger == nil {
		options.Services.Logger = slog.New(slog.NewTextHandler(discardWriter{}, nil))
	}
	if options.Services.NewClient == nil {
		options.Services.NewClient = DefaultClientFactory
	}
	model := Model{
		services:         options.Services,
		screen:           screenLogin,
		width:            100,
		height:           30,
		login:            ui.NewLoginInput(),
		navigation:       ui.NewNavigation(navigationItems()),
		table:            ui.NewResourceTable(nil, nil),
		inspector:        ui.NewInspector("Inspector", ""),
		status:           ui.Status{Text: "Enter a router profile", Kind: ui.Info},
		logFollow:        true,
		logSeverity:      "all",
		logIgnored:       make(map[string]struct{}),
		logSeen:          make(map[string]struct{}),
		cpuCoreSamples:   make(map[string][]float64),
		cpuCoreLoads:     make(map[string]float64),
		firewallPrevious: make(map[string]firewallCounter),
	}
	model.help = ui.HelpOverlay{
		Bindings: []ui.HelpBinding{
			{Key: "↑/↓ · j/k", Description: "move selection or scroll dashboard firewall hits"},
			{Key: "←/→ · h/l", Description: "expand navigation or scroll columns"},
			{Key: "tab", Description: "change focused pane"},
			{Key: "enter", Description: "open resource or inspector"},
			{Key: "/", Description: "filter the current table"},
			{Key: "s", Description: "cycle sorting"},
			{Key: "r", Description: "refresh current data"},
			{Key: "space", Description: "pause or resume RouterOS logs"},
			{Key: "f", Description: "follow the newest RouterOS log event"},
			{Key: "e", Description: "cycle RouterOS log severity"},
			{Key: "c", Description: "clear the local RouterOS log buffer"},
			{Key: "ctrl+l", Description: "log out and forget the saved session"},
			{Key: "ctrl+p", Description: "open command palette"},
			{Key: "q", Description: "quit"},
		},
		Width: 56,
	}
	model.palette = ui.NewCommandPalette(paletteCommands())
	if options.Profile != nil {
		model.profile = *options.Profile
		model.password = options.Credential.Password
		model.login.SetValues(ui.Credentials{
			Address: model.profile.URL, Username: model.profile.Username, Password: model.password,
		})
		model.screen = screenConnecting
		model.status = ui.Status{Text: "Restoring saved session…", Kind: ui.Info}
	}
	return model
}

func (m Model) Init() tea.Cmd {
	if m.screen == screenConnecting {
		return m.connectCmd()
	}
	return nil
}

func (m Model) Update(message tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := message.(type) {
	case tea.WindowSizeMsg:
		m.resize(msg.Width, msg.Height)
		return m, nil
	case connectMsg:
		return m.handleConnect(msg)
	case resourcesMsg:
		return m.handleResources(msg)
	case trafficMsg:
		return m.handleTraffic(msg)
	case trafficTickMsg:
		if m.dashboard && msg.generation == m.pollGeneration && !m.refreshing {
			return m, m.loadTrafficCmd()
		}
		return m, nil
	case refreshMsg:
		if msg.generation != 0 && msg.generation != m.pollGeneration {
			return m, nil
		}
		if m.dashboard && !m.refreshing {
			return m, m.loadTrafficCmd()
		}
		if (msg.specID == "" || msg.specID == m.active.ID) && !m.refreshing {
			return m, m.loadResourcesCmd(m.active)
		}
		return m, nil
	case logoutMsg:
		return m.beginLogout()
	case logoutResultMsg:
		if msg.err != nil {
			m.services.Logger.Warn("local logout cleanup failed", "error", msg.err)
			m.status = ui.Status{Text: "Logged out · saved session cleanup failed", Kind: ui.Warning}
		}
		return m, nil
	case navigateMsg:
		return m.navigateTo(msg.id)
	}

	key, isKey := message.(tea.KeyMsg)
	if isKey && key.String() == "ctrl+c" {
		return m, tea.Quit
	}

	switch m.screen {
	case screenLogin:
		return m.updateLogin(message)
	case screenConnecting:
		if isKey && key.String() == "esc" {
			m.screen = screenLogin
			m.status = ui.Status{Text: "Connection canceled", Kind: ui.Warning}
		}
		return m, nil
	case screenTrust:
		return m.updateTrust(message)
	case screenDashboard:
		return m.updateDashboard(message)
	default:
		return m, nil
	}
}

func (m Model) updateLogin(message tea.Msg) (tea.Model, tea.Cmd) {
	key, isKey := message.(tea.KeyMsg)
	submit := isKey && key.String() == "enter" && m.login.FocusIndex() == 2
	m.login, _ = m.login.Update(message)
	if !submit {
		return m, nil
	}
	values := m.login.Values()
	parsed, err := url.Parse(values.Address)
	if err != nil || parsed.Scheme != "https" || parsed.Host == "" {
		m.status = ui.Status{Text: "Router must be a valid HTTPS URL", Kind: ui.Failure}
		return m, nil
	}
	if values.Username == "" {
		m.status = ui.Status{Text: "Username is required", Kind: ui.Failure}
		return m, nil
	}
	m.profile = config.Profile{Name: "default", URL: strings.TrimRight(values.Address, "/"), Username: values.Username}
	m.password = values.Password
	m.screen = screenConnecting
	m.status = ui.Status{Text: "Negotiating secure connection…", Kind: ui.Info}
	return m, m.connectCmd()
}

func (m Model) updateTrust(message tea.Msg) (tea.Model, tea.Cmd) {
	key, ok := message.(tea.KeyMsg)
	if !ok {
		return m, nil
	}
	switch key.String() {
	case "y", "enter":
		m.profile.CertificateFingerprint = m.trustPin
		m.screen = screenConnecting
		m.status = ui.Status{Text: "Verifying pinned certificate…", Kind: ui.Info}
		return m, m.connectCmd()
	case "n", "esc":
		m.trustPin = ""
		m.screen = screenLogin
		m.status = ui.Status{Text: "Certificate was not trusted", Kind: ui.Warning}
	}
	return m, nil
}

func (m Model) updateDashboard(message tea.Msg) (tea.Model, tea.Cmd) {
	if key, ok := message.(tea.KeyMsg); ok {
		if m.help.Visible {
			m.help, _ = m.help.Update(message)
			return m, nil
		}
		if m.palette.Visible {
			var command tea.Cmd
			m.palette, command = m.palette.Update(message)
			return m, command
		}
		switch key.String() {
		case "q":
			if !m.table.Filtering {
				return m, tea.Quit
			}
		case "?":
			m.help, _ = m.help.Update(message)
			return m, nil
		case "ctrl+p":
			m.palette, _ = m.palette.Update(message)
			return m, nil
		case "ctrl+l":
			return m.beginLogout()
		case "tab":
			m.focus = (m.focus + 1) % m.visiblePaneCount()
			return m, nil
		case "shift+tab":
			m.focus = (m.focus + m.visiblePaneCount() - 1) % m.visiblePaneCount()
			return m, nil
		case "r":
			if !m.table.Filtering && !m.refreshing {
				if m.dashboard {
					return m, m.loadTrafficCmd()
				}
				return m, m.loadResourcesCmd(m.active)
			}
		case "s":
			if !m.table.Filtering {
				if m.active.ID == "logs" {
					return m, nil
				}
				m.cycleSort()
				return m, nil
			}
		case "f":
			if m.active.ID == "logs" && !m.table.Filtering {
				if m.logsPaused {
					m.logsPaused = false
					m.logFollow = true
				} else {
					m.logFollow = !m.logFollow
				}
				if m.logFollow {
					m.followNewestLog()
				}
				return m, nil
			}
		case "e":
			if m.active.ID == "logs" && !m.table.Filtering {
				m.cycleLogSeverity()
				return m, nil
			}
		case "c":
			if m.active.ID == "logs" && !m.table.Filtering {
				m.clearLocalLogs()
				return m, nil
			}
		case " ":
			if m.active.ID == "logs" && !m.table.Filtering {
				m.logsPaused = !m.logsPaused
				if m.logsPaused {
					m.logFollow = false
					m.status = ui.Status{Text: "Log stream paused · space to resume", Kind: ui.Warning}
					return m, nil
				}
				m.status = ui.Status{Text: "Log stream resumed · f to follow newest", Kind: ui.Success}
				return m, nil
			}
		}
	}

	switch m.focus {
	case 0:
		previous := m.navigation.Selected
		m.navigation, _ = m.navigation.Update(message)
		if m.navigation.Selected != previous {
			if m.navigation.Selected == "dashboard" {
				m.activateDashboard()
				return m, m.loadTrafficCmd()
			}
			if spec, ok := specByID(m.navigation.Selected); ok {
				m.activate(spec)
				return m, m.loadResourcesCmd(spec)
			}
		}
		if key, ok := message.(tea.KeyMsg); ok && key.String() == "enter" {
			if m.navigation.Selected == "dashboard" {
				m.activateDashboard()
				m.focus = 1
				return m, m.loadTrafficCmd()
			}
			if spec, exists := specByID(m.navigation.Selected); exists {
				m.activate(spec)
				m.focus = min(1, m.visiblePaneCount()-1)
				return m, m.loadResourcesCmd(spec)
			}
		}
	case 1:
		if m.dashboard {
			m.scrollFirewall(message)
			return m, nil
		}
		if m.active.ID == "logs" {
			if key, ok := message.(tea.KeyMsg); ok {
				switch key.String() {
				case "up", "k", "pgup", "ctrl+u", "home", "g", "/":
					m.logFollow = false
				case "end", "G":
					m.logFollow = true
					m.logUnread = 0
				}
			}
		}
		previous := m.table.SelectedID
		m.table, _ = m.table.Update(message)
		if m.active.ID == "logs" {
			if key, ok := message.(tea.KeyMsg); ok {
				switch key.String() {
				case "down", "j", "pgdown", "ctrl+d", "end", "G":
					rows := m.table.VisibleRows()
					if len(rows) > 0 && m.table.SelectedID == rows[len(rows)-1].ID {
						m.logFollow = true
						m.logUnread = 0
					}
				}
			}
		}
		if m.table.SelectedID != previous {
			m.syncInspector(true)
		}
		if key, ok := message.(tea.KeyMsg); ok && key.String() == "enter" && m.visiblePaneCount() > 2 {
			m.focus = 2
		}
	case 2:
		m.inspector, _ = m.inspector.Update(message)
	}
	return m, nil
}

func (m Model) connectCmd() tea.Cmd {
	profile, password := m.profile, m.password
	newClient, probe := m.services.NewClient, m.services.Probe
	return func() tea.Msg {
		client, err := newClient(profile, password)
		if err != nil {
			return connectMsg{err: err}
		}
		router, err := client.System(context.Background(), routeros.SystemResource{
			Name: "system-resource", Endpoint: "/rest/system/resource",
		})
		if err == nil {
			return connectMsg{client: client, router: router}
		}
		var apiErr *routeros.APIError
		if profile.CertificateFingerprint == "" && errors.As(err, &apiErr) && apiErr.Kind == routeros.ErrorTLS && probe != nil {
			fingerprint, probeErr := probe(context.Background(), profile.URL)
			if probeErr == nil {
				return connectMsg{fingerprint: fingerprint}
			}
		}
		return connectMsg{err: err}
	}
}

func (m Model) handleConnect(msg connectMsg) (tea.Model, tea.Cmd) {
	if msg.fingerprint != "" {
		m.trustPin = msg.fingerprint
		m.screen = screenTrust
		m.status = ui.Status{Text: "Certificate approval required", Kind: ui.Warning}
		return m, nil
	}
	if msg.err != nil {
		m.services.Logger.Error("router connection failed", "error", msg.err)
		m.screen = screenLogin
		m.status = ui.Status{Text: friendlyError(msg.err), Kind: ui.Failure}
		return m, nil
	}
	m.client, m.router = msg.client, msg.router
	m.screen = screenDashboard
	m.sessionStart = time.Now()
	m.status = ui.Status{Text: "Connected securely", Kind: ui.Success}
	if err := m.persist(); err != nil {
		m.services.Logger.Warn("could not persist profile", "error", err)
		m.status = ui.Status{Text: "Connected · profile could not be saved", Kind: ui.Warning}
	}
	m.activateDashboard()
	m.focus = 1
	return m, m.loadTrafficCmd()
}

func (m *Model) loadResourcesCmd(spec resourceSpec) tea.Cmd {
	if m.client == nil || spec.ID == "" {
		return nil
	}
	m.refreshing = true
	m.loading = len(m.records) == 0
	m.requestID++
	requestID, client := m.requestID, m.client
	return func() tea.Msg {
		ctx, cancel := context.WithTimeout(context.Background(), 25*time.Second)
		defer cancel()
		var records []routeros.Resource
		var err error
		if spec.System != nil {
			var record routeros.Resource
			record, err = client.System(ctx, *spec.System)
			if err == nil {
				records = []routeros.Resource{record}
			}
		} else {
			records, err = client.List(ctx, spec.Resource)
		}
		return resourcesMsg{requestID: requestID, specID: spec.ID, records: records, err: err}
	}
}

func (m Model) handleResources(msg resourcesMsg) (tea.Model, tea.Cmd) {
	if msg.requestID != m.requestID || msg.specID != m.active.ID {
		return m, nil
	}
	m.loading = false
	m.refreshing = false
	if msg.err != nil {
		m.services.Logger.Warn("resource refresh failed", "resource", msg.specID, "error", msg.err)
		m.status = ui.Status{Text: friendlyError(msg.err) + " · r to retry", Kind: ui.Failure}
		if msg.specID == "logs" {
			generation := m.pollGeneration
			return m, tea.Tick(2*time.Second, func(time.Time) tea.Msg {
				return refreshMsg{specID: msg.specID, generation: generation}
			})
		}
		return m, nil
	}
	if msg.specID == "logs" {
		m.mergeLogRecords(msg.records)
		m.lastRefresh = time.Now()
		generation := m.pollGeneration
		return m, tea.Tick(m.active.Refresh, func(time.Time) tea.Msg {
			return refreshMsg{specID: msg.specID, generation: generation}
		})
	}
	selectedID := m.table.SelectedID
	m.records = msg.records
	m.table.SetRows(toRows(msg.records))
	m.syncInspector(m.table.SelectedID != selectedID)
	m.lastRefresh = time.Now()
	m.status = ui.Status{Text: fmt.Sprintf("%d records · refreshed now", len(msg.records)), Kind: ui.Success}
	generation := m.pollGeneration
	return m, tea.Tick(m.active.Refresh, func(time.Time) tea.Msg {
		return refreshMsg{specID: msg.specID, generation: generation}
	})
}

func (m *Model) mergeLogRecords(incoming []routeros.Resource) {
	if m.logIgnored == nil {
		m.logIgnored = make(map[string]struct{})
	}
	if m.logSeen == nil {
		m.logSeen = make(map[string]struct{})
	}
	index := make(map[string]int, len(m.records))
	for position, record := range m.records {
		index[rowID(record)] = position
		m.logSeen[rowID(record)] = struct{}{}
	}
	nextSeen := make(map[string]struct{}, len(incoming)+len(m.records))
	newEvents := 0
	for _, record := range incoming {
		id := rowID(record)
		nextSeen[id] = struct{}{}
		if _, ignored := m.logIgnored[id]; ignored {
			continue
		}
		if position, exists := index[id]; exists {
			m.records[position] = record
			continue
		}
		if _, seen := m.logSeen[id]; seen {
			// RouterOS returns the complete in-memory log on every REST poll.
			// Keep knowledge of records outside our bounded display buffer so
			// an old half of a large snapshot is not re-appended every second.
			continue
		}
		index[id] = len(m.records)
		m.logSeen[id] = struct{}{}
		m.records = append(m.records, record)
		newEvents++
	}
	for _, record := range m.records {
		nextSeen[rowID(record)] = struct{}{}
	}
	m.logSeen = nextSeen
	const maxLogBuffer = 500
	if len(m.records) > maxLogBuffer {
		m.records = append([]routeros.Resource(nil), m.records[len(m.records)-maxLogBuffer:]...)
	}
	m.applyLogRows()
	if m.logFollow && !m.logsPaused {
		m.followNewestLog()
	} else {
		m.logUnread += newEvents
	}
	state := "LIVE"
	kind := ui.Success
	if m.logsPaused {
		state = "PAUSED"
		kind = ui.Warning
	}
	m.status = ui.Status{
		Text: fmt.Sprintf("Log stream %s · %d buffered", strings.ToLower(state), len(m.records)),
		Kind: kind,
	}
}

func (m *Model) applyLogRows() {
	visible := make([]routeros.Resource, 0, len(m.records))
	for _, record := range m.records {
		if logMatchesSeverity(record, m.logSeverity) {
			visible = append(visible, record)
		}
	}
	previous := m.table.SelectedID
	m.table.SetRows(toRows(visible))
	if previous != m.table.SelectedID {
		m.syncInspector(true)
	}
}

func (m *Model) followNewestLog() {
	rows := m.table.VisibleRows()
	if len(rows) == 0 {
		m.logUnread = 0
		return
	}
	m.table.SelectedID = rows[len(rows)-1].ID
	m.table.SetRows(m.table.Rows)
	m.logUnread = 0
	m.syncInspector(true)
}

func (m *Model) cycleLogSeverity() {
	switch m.logSeverity {
	case "all":
		m.logSeverity = "info"
	case "info":
		m.logSeverity = "warning"
	case "warning":
		m.logSeverity = "error"
	default:
		m.logSeverity = "all"
	}
	m.applyLogRows()
	if m.logFollow {
		m.followNewestLog()
	}
	m.status = ui.Status{Text: "Log severity · " + strings.ToUpper(m.logSeverity), Kind: ui.Info}
}

func (m *Model) clearLocalLogs() {
	if m.logIgnored == nil {
		m.logIgnored = make(map[string]struct{})
	}
	for _, record := range m.records {
		m.logIgnored[rowID(record)] = struct{}{}
	}
	m.records = nil
	m.table.SetRows(nil)
	m.inspector.SetContent("No log event selected")
	m.logUnread = 0
	m.status = ui.Status{Text: "Local log buffer cleared · router logs were not deleted", Kind: ui.Info}
}

func logMatchesSeverity(record routeros.Resource, severity string) bool {
	if severity == "" || severity == "all" {
		return true
	}
	topics := strings.ToLower(record.Fields["topics"])
	switch severity {
	case "error":
		return strings.Contains(topics, "error") || strings.Contains(topics, "critical")
	case "warning":
		return strings.Contains(topics, "warning")
	case "info":
		return strings.Contains(topics, "info")
	default:
		return true
	}
}

func (m *Model) loadTrafficCmd() tea.Cmd {
	if m.client == nil {
		return nil
	}
	m.refreshing = true
	m.loading = !m.trafficHasBase && len(m.trafficSamples) == 0
	m.requestID++
	requestID, generation, client := m.requestID, m.pollGeneration, m.client
	return func() tea.Msg {
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()

		type telemetryPart struct {
			name    string
			record  routeros.Resource
			records []routeros.Resource
			err     error
		}
		results := make(chan telemetryPart, 4)
		go func() {
			records, err := client.List(ctx, routeros.InterfacesResource)
			var record routeros.Resource
			if err == nil {
				record, err = selectWANInterface(records)
			}
			results <- telemetryPart{name: "interface", record: record, err: err}
		}()
		go func() {
			record, err := client.System(ctx, routeros.SystemResourceInfo)
			results <- telemetryPart{name: "system", record: record, err: err}
		}()
		go func() {
			records, err := client.List(ctx, routeros.CPUResource)
			results <- telemetryPart{name: "cores", records: records, err: err}
		}()
		go func() {
			records, err := client.List(ctx, routeros.FirewallFilterResource)
			results <- telemetryPart{name: "firewall", records: records, err: err}
		}()

		message := trafficMsg{requestID: requestID, generation: generation, at: time.Now()}
		for range 4 {
			part := <-results
			switch part.name {
			case "interface":
				message.record, message.err = part.record, part.err
			case "system":
				message.system, message.systemErr = part.record, part.err
			case "cores":
				message.cores, message.coresErr = part.records, part.err
			case "firewall":
				message.firewall, message.firewallErr = part.records, part.err
			}
		}
		return message
	}
}

func (m Model) handleTraffic(msg trafficMsg) (tea.Model, tea.Cmd) {
	if msg.requestID != m.requestID || msg.generation != m.pollGeneration || !m.dashboard {
		return m, nil
	}
	m.loading = false
	m.refreshing = false
	generation := m.pollGeneration
	next := tea.Tick(2*time.Second, func(time.Time) tea.Msg {
		return trafficTickMsg{generation: generation}
	})
	if msg.systemErr == nil {
		m.updateSystemTelemetry(msg.system)
	}
	if msg.coresErr == nil {
		m.updateCPUTelemetry(msg.cores, msg.system)
	} else if msg.systemErr == nil {
		m.updateCPUTelemetry(nil, msg.system)
	}
	if msg.firewallErr == nil {
		m.updateFirewallTelemetry(msg.firewall)
	}
	if msg.err != nil {
		m.services.Logger.Warn("WAN traffic refresh failed", "error", msg.err)
		m.status = ui.Status{Text: "System telemetry live · WAN " + friendlyError(msg.err) + " · retrying", Kind: ui.Warning}
		return m, next
	}

	rx := resourceCounter(msg.record, "rx-byte", "fp-rx-byte")
	tx := resourceCounter(msg.record, "tx-byte", "fp-tx-byte")
	m.trafficInterface = msg.record.Fields["name"]
	if m.trafficHasBase {
		elapsed := msg.at.Sub(m.trafficUpdated).Seconds()
		if elapsed > 0 {
			m.trafficRXRate = counterRate(m.trafficRXBytes, rx, elapsed)
			m.trafficTXRate = counterRate(m.trafficTXBytes, tx, elapsed)
			if len(m.trafficSamples) == 0 {
				m.trafficSamples = append(m.trafficSamples, ui.TrafficSample{})
			}
			m.trafficSamples = append(m.trafficSamples, ui.TrafficSample{
				RX: m.trafficRXRate,
				TX: m.trafficTXRate,
			})
			if len(m.trafficSamples) > 120 {
				m.trafficSamples = append([]ui.TrafficSample(nil), m.trafficSamples[len(m.trafficSamples)-120:]...)
			}
		}
	}
	m.trafficRXBytes, m.trafficTXBytes = rx, tx
	m.trafficUpdated = msg.at
	m.trafficHasBase = true
	var unavailable []string
	if msg.systemErr != nil {
		unavailable = append(unavailable, "CPU/memory")
	}
	if msg.firewallErr != nil {
		unavailable = append(unavailable, "firewall")
	}
	if len(unavailable) > 0 {
		m.status = ui.Status{
			Text: "WAN live · " + strings.Join(unavailable, " + ") + " telemetry retrying",
			Kind: ui.Warning,
		}
		return m, next
	}
	m.status = ui.Status{Text: "WAN telemetry live · " + m.trafficInterface, Kind: ui.Success}
	return m, next
}

func (m *Model) updateSystemTelemetry(resource routeros.Resource) {
	total := resourceCounter(resource, "total-memory")
	free := resourceCounter(resource, "free-memory")
	if total == 0 || free > total {
		return
	}
	m.memoryTotalBytes = total
	m.memoryUsedBytes = total - free
	usedPercent := float64(m.memoryUsedBytes) * 100 / float64(total)
	m.memorySamples = appendBoundedSample(m.memorySamples, usedPercent, 120)
}

func (m *Model) updateCPUTelemetry(cores []routeros.Resource, system routeros.Resource) {
	if m.cpuCoreSamples == nil {
		m.cpuCoreSamples = make(map[string][]float64)
	}
	if m.cpuCoreLoads == nil {
		m.cpuCoreLoads = make(map[string]float64)
	}
	seen := make(map[string]struct{}, len(cores))
	for index, core := range cores {
		name := field(core, "cpu", field(core, "name", fmt.Sprintf("cpu%d", index)))
		load, err := strconv.ParseFloat(core.Fields["load"], 64)
		if err != nil {
			continue
		}
		seen[name] = struct{}{}
		m.cpuCoreLoads[name] = load
		m.cpuCoreSamples[name] = appendBoundedSample(m.cpuCoreSamples[name], load, 120)
	}
	if len(seen) == 0 {
		if load, err := strconv.ParseFloat(system.Fields["cpu-load"], 64); err == nil {
			seen["cpu"] = struct{}{}
			m.cpuCoreLoads["cpu"] = load
			m.cpuCoreSamples["cpu"] = appendBoundedSample(m.cpuCoreSamples["cpu"], load, 120)
		}
	}
	order := make([]string, 0, len(seen))
	for name := range seen {
		order = append(order, name)
	}
	sort.Strings(order)
	m.cpuCoreOrder = order
}

func (m *Model) updateFirewallTelemetry(records []routeros.Resource) {
	if m.firewallPrevious == nil {
		m.firewallPrevious = make(map[string]firewallCounter)
	}
	history := make(map[string][]float64, len(m.firewallRules))
	for _, rule := range m.firewallRules {
		history[rule.ID] = rule.History
	}
	nextPrevious := make(map[string]firewallCounter, len(records))
	rules := make([]ui.FirewallRuleMetric, 0, len(records))
	for _, record := range records {
		if resourceBool(record, "disabled") {
			continue
		}
		id := rowID(record)
		current := firewallCounter{
			packets: resourceCounter(record, "packets"),
			bytes:   resourceCounter(record, "bytes"),
		}
		nextPrevious[id] = current
		var packetDelta, byteDelta uint64
		if previous, exists := m.firewallPrevious[id]; m.firewallHasBase && exists {
			if current.packets >= previous.packets {
				packetDelta = current.packets - previous.packets
			}
			if current.bytes >= previous.bytes {
				byteDelta = current.bytes - previous.bytes
			}
		}
		// Normalize bytes to approximate full-sized packets, then combine both
		// counters into one heat signal while preserving packet counts in text.
		activity := float64(packetDelta) + float64(byteDelta)/1500
		label := strings.TrimSpace(record.Fields["comment"])
		if label == "" {
			label = strings.TrimSpace(record.Fields["chain"] + " · " + record.Fields["action"])
		}
		if label == "" {
			label = id
		}
		rules = append(rules, ui.FirewallRuleMetric{
			ID:            id,
			Label:         label,
			Action:        record.Fields["action"],
			Packets:       current.packets,
			Bytes:         current.bytes,
			RecentPackets: packetDelta,
			RecentBytes:   byteDelta,
			History:       appendBoundedSample(history[id], activity, 60),
		})
	}
	m.firewallPrevious = nextPrevious
	m.firewallRules = rules
	m.firewallHasBase = true
}

func appendBoundedSample(samples []float64, value float64, limit int) []float64 {
	samples = append(samples, value)
	if len(samples) > limit {
		return append([]float64(nil), samples[len(samples)-limit:]...)
	}
	return samples
}

func (m *Model) activateDashboard() {
	m.requestID++
	m.pollGeneration++
	m.dashboard = true
	m.active = resourceSpec{}
	m.records = nil
	m.loading = len(m.trafficSamples) == 0
	m.refreshing = false
	m.trafficRXRate = 0
	m.trafficTXRate = 0
	m.trafficUpdated = time.Time{}
	m.trafficHasBase = false
	m.firewallHasBase = false
	m.firewallPrevious = make(map[string]firewallCounter)
	m.firewallOffset = 0
	if len(m.trafficSamples) == 0 {
		m.status = ui.Status{Text: "Detecting active WAN interface…", Kind: ui.Info}
	} else {
		m.status = ui.Status{Text: "Resuming recent WAN telemetry…", Kind: ui.Info}
	}
}

func (m *Model) activate(spec resourceSpec) {
	m.pollGeneration++
	m.dashboard = false
	m.active = spec
	m.logsPaused = false
	if spec.ID == "logs" {
		m.logFollow = true
		m.logUnread = 0
		m.logSeverity = "all"
		m.logIgnored = make(map[string]struct{})
		m.logSeen = make(map[string]struct{})
	}
	m.records = nil
	m.loading = true
	m.refreshing = false
	m.table = ui.NewResourceTable(spec.Columns, nil)
	m.inspector = ui.NewInspector(spec.Label, "")
	m.resize(m.width, m.height)
	m.status = ui.Status{Text: "Loading " + spec.Label + "…", Kind: ui.Info}
}

func (m Model) navigateTo(id string) (tea.Model, tea.Cmd) {
	m.navigation.Reveal(id)
	if id == "dashboard" {
		m.activateDashboard()
		m.focus = min(1, m.visiblePaneCount()-1)
		return m, m.loadTrafficCmd()
	}
	spec, ok := specByID(id)
	if !ok {
		return m, nil
	}
	m.activate(spec)
	m.focus = min(1, m.visiblePaneCount()-1)
	return m, m.loadResourcesCmd(spec)
}

func paletteCommands() []ui.Command {
	commands := []ui.Command{
		{ID: "refresh", Title: "Refresh", Description: "reload the current resource", Run: func() tea.Msg { return refreshMsg{} }},
		{ID: "logout", Title: "Log out", Description: "forget this router session", Run: func() tea.Msg { return logoutMsg{} }},
		{ID: "help", Title: "Keyboard help", Description: "show all shortcuts"},
		{ID: "dashboard", Title: "Dashboard", Description: "live WAN overview", Run: func() tea.Msg { return navigateMsg{id: "dashboard"} }},
	}
	for _, spec := range resourceSpecs {
		id := spec.ID
		path := spec.CLIPath()
		commands = append(commands, ui.Command{
			ID:          id,
			Title:       path,
			Description: spec.Label,
			Path:        path,
			Run:         func() tea.Msg { return navigateMsg{id: id} },
		})
	}
	return commands
}

func selectWANInterface(records []routeros.Resource) (routeros.Resource, error) {
	bestScore := -1
	var best routeros.Resource
	for _, record := range records {
		if !resourceBool(record, "running") || resourceBool(record, "disabled") {
			continue
		}
		name := strings.ToLower(record.Fields["name"])
		interfaceType := strings.ToLower(record.Fields["type"])
		score := 0
		switch {
		case strings.Contains(interfaceType, "pppoe") || strings.Contains(name, "pppoe"):
			score = 100
		case strings.Contains(name, "wan"):
			score = 90
		case interfaceType == "ether":
			score = 50
		case interfaceType == "bridge":
			score = 20
		case interfaceType == "loopback":
			continue
		}
		traffic := resourceCounter(record, "rx-byte", "fp-rx-byte") +
			resourceCounter(record, "tx-byte", "fp-tx-byte")
		if score > bestScore || (score == bestScore && traffic >
			resourceCounter(best, "rx-byte", "fp-rx-byte")+resourceCounter(best, "tx-byte", "fp-tx-byte")) {
			bestScore, best = score, record
		}
	}
	if bestScore < 0 {
		return routeros.Resource{}, errors.New("no active WAN interface detected")
	}
	return best, nil
}

func resourceCounter(resource routeros.Resource, names ...string) uint64 {
	for _, name := range names {
		if value, err := strconv.ParseUint(resource.Fields[name], 10, 64); err == nil {
			return value
		}
	}
	return 0
}

func resourceBool(resource routeros.Resource, name string) bool {
	value, err := routeros.ParseBool(resource.Fields[name])
	return err == nil && value
}

func counterRate(previous, current uint64, seconds float64) float64 {
	if current < previous || seconds <= 0 {
		return 0
	}
	return float64(current-previous) * 8 / seconds
}

func (m *Model) syncInspector(resetPosition bool) {
	for _, record := range m.records {
		if rowID(record) == m.table.SelectedID {
			m.inspector.Title = m.active.Label + " · " + displayIdentity(record)
			content := formatFields(record)
			if resetPosition {
				m.inspector.SetContent(content)
			} else {
				m.inspector.SetContentPreservingOffset(content)
			}
			return
		}
	}
	m.inspector.Title = m.active.Label
	m.inspector.SetContent("No record selected")
}

func (m *Model) cycleSort() {
	if len(m.active.Columns) == 0 {
		return
	}
	column := m.active.Columns[0].Key
	switch m.table.SortDirection {
	case ui.SortNone:
		m.table.SetSort(column, ui.SortAscending)
	case ui.SortAscending:
		m.table.SetSort(column, ui.SortDescending)
	default:
		m.table.SetSort("", ui.SortNone)
	}
}

func (m *Model) resize(width, height int) {
	m.width, m.height = max(40, width), max(12, height)
	m.login.SetSize(min(54, m.width-6))
	m.help.SetSize(min(64, m.width-4), m.height)
	m.palette.Width = min(64, m.width-4)
	layout := paneLayoutFor(m.width, m.height)
	m.navigation.SetSize(innerPaneWidth(layout.navigationWidth))
	m.navigation.SetHeight(layout.contentHeight)
	tableHeight := layout.contentHeight
	if m.active.ID == "logs" {
		tableHeight = max(1, tableHeight-2)
	}
	m.table.SetSize(innerPaneWidth(layout.tableWidth), tableHeight)
	m.inspector.SetSize(innerPaneWidth(layout.inspectorWidth), layout.contentHeight)
}

func (m Model) View() string {
	switch m.screen {
	case screenLogin:
		return m.loginView()
	case screenConnecting:
		return m.centeredView(ui.Loading{Label: "Connecting to RouterOS"}.View() + "\n\n" + m.status.View())
	case screenTrust:
		return m.trustView()
	case screenDashboard:
		return m.dashboardView()
	default:
		return ""
	}
}

func (m Model) loginView() string {
	logo := theme.Default.Focus.Bold(true).Render("ROUTERDECK")
	subtitle := theme.Default.Muted.Render("A precise, read-only RouterOS control deck")
	content := lipgloss.JoinVertical(lipgloss.Left, logo, subtitle, "", m.login.View(), "", m.status.View())
	return m.centeredView(theme.Default.Panel.Border(lipgloss.RoundedBorder()).
		BorderForeground(theme.DefaultPalette.Border).Padding(1, 2).Render(content))
}

func (m Model) trustView() string {
	formatted := formatPin(m.trustPin)
	content := lipgloss.JoinVertical(lipgloss.Left,
		theme.Default.Alert.Bold(true).Render("UNRECOGNIZED ROUTER CERTIFICATE"),
		theme.Default.Text.Render("Verify this SHA-256 fingerprint through a trusted channel:"),
		"",
		theme.Default.Focus.Render(formatted),
		"",
		theme.Default.Muted.Render("y / enter  trust and save     n / esc  cancel"),
	)
	return m.centeredView(theme.Default.Panel.Border(lipgloss.RoundedBorder()).
		BorderForeground(theme.DefaultPalette.Alert).Padding(1, 2).Render(content))
}

func (m Model) dashboardView() string {
	header := m.headerView()
	body := m.bodyView()
	footer := theme.Default.Muted.Render("[? help] [ctrl+p commands] [ctrl+l logout] [/ filter] [r refresh] [q quit]")
	status := m.status.View()
	view := lipgloss.JoinVertical(lipgloss.Left, header, body, status, footer)
	base := theme.Default.Base.Width(m.width).Height(m.height).Render(view)
	if m.help.Visible {
		return ui.Modal(base, m.help.View(), m.width, m.height)
	}
	if m.palette.Visible {
		return ui.Modal(base, m.palette.View(), m.width, m.height)
	}
	return base
}

func (m Model) headerView() string {
	name := field(m.router, "board-name", "RouterOS")
	version := field(m.router, "version", "")
	uptime := field(m.router, "uptime", "")
	rail := ui.SignalRail{Width: max(1, m.width-2), Signals: []ui.Signal{
		{Label: "ROUTERDECK", Value: name, Level: ui.SignalGood},
		{Label: m.profile.URL, Value: "", Level: ui.SignalGood},
		{Label: "user", Value: m.profile.Username, Level: ui.SignalIdle},
		{Label: "session", Value: m.sessionDuration(), Level: ui.SignalIdle},
		{Label: "RouterOS", Value: version, Level: ui.SignalIdle},
		{Label: "uptime", Value: uptime, Level: ui.SignalIdle},
	}}
	return theme.Default.Panel.Width(m.width).Padding(0, 1).Render(rail.View())
}

func (m Model) sessionDuration() string {
	if m.sessionStart.IsZero() {
		return "—"
	}
	elapsed := time.Since(m.sessionStart)
	if elapsed < 0 {
		elapsed = 0
	}
	return elapsed.Truncate(time.Second).String()
}

func (m Model) bodyView() string {
	layout := paneLayoutFor(m.width, m.height)
	breakpoint := ui.BreakpointFor(m.width)
	if m.dashboard {
		return m.dashboardBodyView(layout, breakpoint)
	}
	if breakpoint == ui.Narrow {
		switch m.focus {
		case 0:
			return panel("NAVIGATION", m.navigation.View(), layout.navigationWidth, layout.height, true)
		case 2:
			return panel("INSPECTOR", m.inspector.View(), layout.inspectorWidth, layout.height, true)
		default:
			return panel(strings.ToUpper(m.active.Label), m.tableOrState(), layout.tableWidth, layout.height, true)
		}
	}
	nav := panel("NAVIGATION", m.navigation.View(), layout.navigationWidth, layout.height, m.focus == 0)
	table := panel(strings.ToUpper(m.active.Label), m.tableOrState(), layout.tableWidth, layout.height, m.focus == 1)
	if breakpoint == ui.Medium {
		return lipgloss.JoinHorizontal(lipgloss.Top, nav, table)
	}
	inspector := panel("INSPECTOR", m.inspector.View(), layout.inspectorWidth, layout.height, m.focus == 2)
	return lipgloss.JoinHorizontal(lipgloss.Top, nav, table, inspector)
}

func (m Model) dashboardBodyView(layout paneLayout, breakpoint ui.Breakpoint) string {
	if breakpoint == ui.Narrow {
		if m.focus == 0 {
			return panel("NAVIGATION", m.navigation.View(), layout.navigationWidth, layout.height, true)
		}
		content := m.dashboardContent(innerPaneWidth(layout.tableWidth), layout.contentHeight)
		return panel("DASHBOARD", content, layout.tableWidth, layout.height, true)
	}
	nav := panel("NAVIGATION", m.navigation.View(), layout.navigationWidth, layout.height, m.focus == 0)
	dashboardWidth := layout.tableWidth
	if breakpoint == ui.Wide {
		dashboardWidth += layout.inspectorWidth
	}
	content := m.dashboardContent(innerPaneWidth(dashboardWidth), layout.contentHeight)
	dashboard := panel("DASHBOARD", content, dashboardWidth, layout.height, m.focus == 1)
	return lipgloss.JoinHorizontal(lipgloss.Top, nav, dashboard)
}

type dashboardGeometry struct {
	stacked                      bool
	cpuHeight, memoryHeight      int
	wanHeight, firewallHeight    int
	metricWidthLeft, metricRight int
}

func (m Model) dashboardGeometry(width, height int) dashboardGeometry {
	width, height = max(1, width), max(1, height)
	geometry := dashboardGeometry{stacked: width < 72}
	cpuRows := len(m.cpuCoreOrder)
	if cpuRows == 0 {
		cpuRows = 4
	}
	cpuRows = min(8, max(1, cpuRows))
	if height < 10 {
		return geometry
	}
	if geometry.stacked {
		memoryRows := min(2, cpuRows)
		budget := height - (5 + cpuRows + memoryRows)
		wan, firewall := splitDashboardBudget(budget)
		geometry.cpuHeight, geometry.memoryHeight = cpuRows, memoryRows
		geometry.wanHeight, geometry.firewallHeight = wan, firewall
		return geometry
	}
	left := (width - 2) / 2
	geometry.metricWidthLeft = left
	geometry.metricRight = width - left - 2
	geometry.cpuHeight, geometry.memoryHeight = cpuRows, cpuRows
	wan, firewall := splitDashboardBudget(height - (4 + cpuRows))
	geometry.wanHeight, geometry.firewallHeight = wan, firewall
	return geometry
}

func splitDashboardBudget(budget int) (wan, firewall int) {
	budget = max(2, budget)
	minWAN := max(4, budget*3/5)
	if minWAN > budget-2 {
		minWAN = max(1, budget-2)
	}
	firewall = min(1+ui.MaxFirewallRules, max(2, budget-minWAN))
	wan = max(1, budget-firewall)
	return wan, firewall
}

func (m *Model) scrollFirewall(message tea.Msg) {
	key, ok := message.(tea.KeyMsg)
	if !ok {
		return
	}
	layout := paneLayoutFor(m.width, m.height)
	dashboardWidth := layout.tableWidth
	if ui.BreakpointFor(m.width) == ui.Wide {
		dashboardWidth += layout.inspectorWidth
	}
	geometry := m.dashboardGeometry(innerPaneWidth(dashboardWidth), layout.contentHeight)
	page := max(1, geometry.firewallHeight-1)
	maxOffset := ui.FirewallHitChart{
		Rules: m.firewallRules, Height: geometry.firewallHeight, Offset: m.firewallOffset,
	}.MaxOffset()
	switch key.String() {
	case "up", "k":
		m.firewallOffset--
	case "down", "j":
		m.firewallOffset++
	case "pgup", "ctrl+u":
		m.firewallOffset -= page
	case "pgdown", "ctrl+d":
		m.firewallOffset += page
	case "home", "g":
		m.firewallOffset = 0
	case "end", "G":
		m.firewallOffset = maxOffset
	}
	if m.firewallOffset < 0 {
		m.firewallOffset = 0
	}
	if m.firewallOffset > maxOffset {
		m.firewallOffset = maxOffset
	}
}

func (m Model) dashboardContent(width, height int) string {
	width, height = max(1, width), max(1, height)
	geometry := m.dashboardGeometry(width, height)
	if height < 10 {
		return constrainCanvas(m.compactDashboardView(width), width, height)
	}

	var metrics string
	if geometry.stacked {
		metrics = dashboardSection("CPU CORES", m.cpuDashboardView(width, geometry.cpuHeight), width, geometry.cpuHeight) + "\n" +
			dashboardSection("MEMORY", m.memoryDashboardView(width, geometry.memoryHeight), width, geometry.memoryHeight)
	} else {
		metrics = lipgloss.JoinHorizontal(lipgloss.Top,
			dashboardSection("CPU CORES", m.cpuDashboardView(geometry.metricWidthLeft, geometry.cpuHeight), geometry.metricWidthLeft, geometry.cpuHeight),
			"  ",
			dashboardSection("MEMORY", m.memoryDashboardView(geometry.metricRight, geometry.memoryHeight), geometry.metricRight, geometry.memoryHeight),
		)
	}
	wan := dashboardSection("WAN THROUGHPUT", m.wanDashboardView(width, geometry.wanHeight), width, geometry.wanHeight)
	firewall := dashboardSection("FIREWALL HIT HEAT", ui.FirewallHitChart{
		Rules:  m.firewallRules,
		Width:  width,
		Height: geometry.firewallHeight,
		Offset: m.firewallOffset,
	}.View(), width, geometry.firewallHeight)
	return constrainCanvas(metrics+"\n\n"+wan+"\n"+firewall, width, height)
}

func (m Model) compactDashboardView(width int) string {
	cpu := "CPU collecting"
	if len(m.cpuCoreOrder) > 0 {
		total := 0.0
		for _, name := range m.cpuCoreOrder {
			total += m.cpuCoreLoads[name]
		}
		cpu = fmt.Sprintf("CPU %.0f%% avg", total/float64(len(m.cpuCoreOrder)))
	}
	memory := "memory collecting"
	if m.memoryTotalBytes > 0 {
		memory = fmt.Sprintf("memory %.0f%%", float64(m.memoryUsedBytes)*100/float64(m.memoryTotalBytes))
	}
	wan := "WAN collecting"
	if m.trafficHasBase {
		wan = "WAN ↓ " + formatRate(m.trafficRXRate) + "  ↑ " + formatRate(m.trafficTXRate)
	}
	firewall := fmt.Sprintf("firewall %d enabled rules", len(m.firewallRules))
	return dashboardSection("ROUTER TELEMETRY", theme.Default.Text.Render(cpu+"  ·  "+memory), width, 1) + "\n" +
		dashboardSection("THROUGHPUT", theme.Default.Text.Render(wan), width, 1) + "\n" +
		dashboardSection("FIREWALL", theme.Default.Text.Render(firewall), width, 1)
}

func (m Model) cpuDashboardView(width, height int) string {
	height = max(1, height)
	order := m.cpuCoreOrder
	if len(order) > height {
		order = order[:height]
	}
	lines := make([]string, height)
	if len(m.cpuCoreOrder) == 0 {
		lines[0] = theme.Default.Muted.Render("Collecting per-core load…")
		for index := 1; index < height; index++ {
			lines[index] = ui.BrailleSparkline{Width: width, Height: 1, Min: 0, Max: 100, Style: theme.Default.Muted}.View()
		}
		return strings.Join(lines, "\n")
	}
	for index := 0; index < height; index++ {
		if index >= len(order) {
			lines[index] = ""
			continue
		}
		name := order[index]
		load := m.cpuCoreLoads[name]
		style := theme.Default.Signal
		state := "OK"
		switch {
		case load >= 85:
			style, state = theme.Default.Error, "HIGH"
		case load >= 60:
			style, state = theme.Default.Alert, "BUSY"
		}
		labelWidth := min(8, max(4, width/5))
		valueWidth := 10
		sparkWidth := max(4, width-labelWidth-valueWidth-2)
		spark := ui.BrailleSparkline{
			Samples: m.cpuCoreSamples[name], Width: sparkWidth, Height: 1,
			Min: 0, Max: 100, Style: style,
		}.View()
		lines[index] = theme.Default.Text.Render(fitDashboardCell(name, labelWidth)) + " " +
			spark + " " + style.Render(fmt.Sprintf("%3.0f%% %-4s", load, state))
	}
	return strings.Join(lines, "\n")
}

func (m Model) memoryDashboardView(width, height int) string {
	height = max(1, height)
	if m.memoryTotalBytes == 0 {
		lines := make([]string, height)
		lines[0] = theme.Default.Muted.Render("Collecting memory pressure…")
		for index := 1; index < height; index++ {
			lines[index] = ui.BrailleSparkline{Width: width, Height: 1, Min: 0, Max: 100, Style: theme.Default.Muted}.View()
		}
		return strings.Join(lines, "\n")
	}
	percent := float64(m.memoryUsedBytes) * 100 / float64(m.memoryTotalBytes)
	style := theme.Default.Signal
	state := "HEALTHY"
	switch {
	case percent >= 90:
		style, state = theme.Default.Error, "CRITICAL"
	case percent >= 75:
		style, state = theme.Default.Alert, "PRESSURE"
	}
	summary := style.Bold(true).Render(fmt.Sprintf("%.1f%% %s", percent, state)) + "  " +
		theme.Default.Muted.Render(formatBytes(m.memoryUsedBytes)+" / "+formatBytes(m.memoryTotalBytes))
	if height <= 1 {
		return summary
	}
	spark := ui.BrailleSparkline{
		Samples: m.memorySamples, Width: width, Height: max(1, height-1),
		Min: 0, Max: 100, Style: style,
	}.View()
	return summary + "\n" + spark
}

func (m Model) wanDashboardView(width, height int) string {
	height = max(1, height)
	identity := theme.Default.Muted.Render("Detecting WAN interface…")
	if m.trafficHasBase || m.trafficInterface != "" {
		identity = theme.Default.Text.Bold(true).Render(orDash(m.trafficInterface)) + "  " +
			theme.Default.Signal.Render("● LIVE") + "  " +
			theme.Default.Signal.Bold(true).Render("↓ "+formatRate(m.trafficRXRate)) + "  " +
			theme.Default.Focus.Bold(true).Render("↑ "+formatRate(m.trafficTXRate))
	}
	if height <= 1 {
		return identity
	}
	chart := ui.TrafficChart{
		Samples: m.trafficSamples, Width: width, Height: height - 1, SampleInterval: 2 * time.Second,
	}.View()
	return identity + "\n" + chart
}

func orDash(value string) string {
	if strings.TrimSpace(value) == "" {
		return "—"
	}
	return value
}

func dashboardSection(title, content string, width, contentHeight int) string {
	heading := theme.Default.Focus.Bold(true).Render(title) + " " +
		theme.Default.Muted.Render(strings.Repeat("─", max(0, width-len(title)-1)))
	return heading + "\n" + constrainCanvas(content, width, contentHeight)
}

func fitDashboardCell(value string, width int) string {
	runes := []rune(value)
	if len(runes) > width {
		if width <= 1 {
			return string(runes[:width])
		}
		value = string(runes[:width-1]) + "…"
	}
	return value + strings.Repeat(" ", max(0, width-len([]rune(value))))
}

func formatRate(bitsPerSecond float64) string {
	const (
		kilobit = 1_000
		megabit = 1_000_000
		gigabit = 1_000_000_000
	)
	switch {
	case bitsPerSecond >= gigabit:
		return fmt.Sprintf("%.2f Gbps", bitsPerSecond/gigabit)
	case bitsPerSecond >= megabit:
		return fmt.Sprintf("%.1f Mbps", bitsPerSecond/megabit)
	case bitsPerSecond >= kilobit:
		return fmt.Sprintf("%.1f Kbps", bitsPerSecond/kilobit)
	default:
		return fmt.Sprintf("%.0f bps", bitsPerSecond)
	}
}

func formatBytes(bytes uint64) string {
	const (
		kib = 1024
		mib = 1024 * kib
		gib = 1024 * mib
		tib = 1024 * gib
	)
	switch {
	case bytes >= tib:
		return fmt.Sprintf("%.2f TiB", float64(bytes)/tib)
	case bytes >= gib:
		return fmt.Sprintf("%.2f GiB", float64(bytes)/gib)
	case bytes >= mib:
		return fmt.Sprintf("%.1f MiB", float64(bytes)/mib)
	case bytes >= kib:
		return fmt.Sprintf("%.1f KiB", float64(bytes)/kib)
	default:
		return fmt.Sprintf("%d B", bytes)
	}
}

type paneLayout struct {
	height                                      int
	contentHeight                               int
	navigationWidth, tableWidth, inspectorWidth int
}

func paneLayoutFor(width, height int) paneLayout {
	availableWidth := max(38, width)
	paneHeight := max(6, height-3)
	layout := paneLayout{
		height:        paneHeight,
		contentHeight: max(2, paneHeight-3),
	}
	switch ui.BreakpointFor(width) {
	case ui.Wide:
		layout.navigationWidth = 28
		layout.inspectorWidth = 38
		layout.tableWidth = max(34, availableWidth-layout.navigationWidth-layout.inspectorWidth)
	case ui.Medium:
		layout.navigationWidth = min(28, max(24, availableWidth/3))
		layout.tableWidth = max(30, availableWidth-layout.navigationWidth)
		layout.inspectorWidth = availableWidth
	default:
		layout.navigationWidth = availableWidth
		layout.tableWidth = availableWidth
		layout.inspectorWidth = availableWidth
	}
	return layout
}

func innerPaneWidth(outerWidth int) int {
	// One cell of padding on each side plus the two border cells.
	return max(1, outerWidth-4)
}

func (m Model) tableOrState() string {
	if m.active.ID == "logs" {
		return m.logsView()
	}
	if m.loading {
		return ui.Loading{Label: "Reading " + m.active.Label}.View()
	}
	if len(m.records) == 0 {
		return ui.EmptyState{Title: "No records", Hint: "Press r to refresh"}.View(max(20, m.table.Width))
	}
	return m.table.View()
}

func (m Model) logsView() string {
	stream := theme.Default.Signal.Render("● LIVE")
	if m.logsPaused {
		stream = theme.Default.Alert.Render("Ⅱ PAUSED")
	}
	position := theme.Default.Signal.Render("FOLLOWING")
	if !m.logFollow {
		position = theme.Default.Muted.Render("DETACHED")
	}
	meta := stream + "  " + position +
		"  " + theme.Default.Muted.Render(fmt.Sprintf("%d / 500 events · severity %s", len(m.records), strings.ToUpper(m.logSeverity)))
	if m.logUnread > 0 {
		meta += "  " + theme.Default.Alert.Render(fmt.Sprintf("↓ %d NEW", m.logUnread))
	}
	controls := theme.Default.Muted.Render("space pause  ·  f follow  ·  e severity  ·  / search  ·  c clear local")
	content := m.table.View()
	switch {
	case m.loading && len(m.records) == 0:
		content = ui.Loading{Label: "Opening RouterOS log stream"}.View()
	case len(m.records) == 0:
		content = ui.EmptyState{Title: "Waiting for log events", Hint: "The local stream is active"}.View(max(20, m.table.Width))
	case len(m.table.VisibleRows()) == 0:
		content = ui.EmptyState{Title: "No matching events", Hint: "Change severity or search filter"}.View(max(20, m.table.Width))
	}
	return meta + "\n" + controls + "\n" + content
}

func (m Model) centeredView(content string) string {
	return theme.Default.Base.Width(m.width).Height(m.height).
		Align(lipgloss.Center, lipgloss.Center).Render(content)
}

func (m Model) visiblePaneCount() int {
	if m.dashboard {
		return 2
	}
	switch ui.BreakpointFor(m.width) {
	case ui.Wide:
		return 3
	case ui.Medium:
		return 2
	default:
		return 3
	}
}

func (m Model) beginLogout() (tea.Model, tea.Cmd) {
	profileName := m.profile.Name
	address, username := m.profile.URL, m.profile.Username
	profiles, credentialStore := m.services.Profiles, m.services.Credentials

	m.requestID++
	m.client = nil
	m.router = routeros.Resource{}
	m.records = nil
	m.password = ""
	m.profile = config.Profile{}
	m.sessionStart = time.Time{}
	m.loading = false
	m.refreshing = false
	m.logsPaused = false
	m.trustPin = ""
	m.dashboard = false
	m.pollGeneration++
	m.trafficSamples = nil
	m.trafficHasBase = false
	m.screen = screenLogin
	m.login.SetValues(ui.Credentials{Address: address, Username: username})
	m.status = ui.Status{Text: "Logged out · saved session removed", Kind: ui.Success}

	return m, func() tea.Msg {
		var cleanupErrors []error
		if profiles != nil {
			if err := profiles.Save([]config.Profile{}); err != nil {
				cleanupErrors = append(cleanupErrors, err)
			}
		}
		if credentialStore != nil && profileName != "" {
			if err := credentialStore.Delete(context.Background(), profileName); err != nil {
				cleanupErrors = append(cleanupErrors, err)
			}
		}
		return logoutResultMsg{err: errors.Join(cleanupErrors...)}
	}
}

func (m Model) persist() error {
	ctx := context.Background()
	if m.services.Profiles != nil {
		if err := m.services.Profiles.Save([]config.Profile{m.profile}); err != nil {
			return err
		}
	}
	if m.services.Credentials != nil {
		if err := m.services.Credentials.Put(ctx, m.profile.Name, credentials.Credential{Password: m.password}); err != nil {
			return err
		}
	}
	return nil
}

func toRows(records []routeros.Resource) []ui.ResourceRow {
	rows := make([]ui.ResourceRow, 0, len(records))
	for _, record := range records {
		cells := make(map[string]string, len(record.Fields))
		for key, value := range record.Fields {
			cells[key] = displayValue(key, value)
		}
		rows = append(rows, ui.ResourceRow{ID: rowID(record), Cells: cells})
	}
	return rows
}

func rowID(record routeros.Resource) string {
	if record.ID != "" {
		return record.ID
	}
	for _, key := range []string{"name", "address", "time", "serial-number"} {
		if value := record.Fields[key]; value != "" {
			return key + ":" + value
		}
	}
	return fmt.Sprintf("record:%p", record.Fields)
}

func displayIdentity(record routeros.Resource) string {
	for _, key := range []string{"name", "address", "host-name", "model", "time"} {
		if value := record.Fields[key]; value != "" {
			return value
		}
	}
	return record.ID
}

func formatFields(record routeros.Resource) string {
	keys := make([]string, 0, len(record.Fields)+1)
	if record.ID != "" {
		keys = append(keys, ".id")
	}
	for key := range record.Fields {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	lines := make([]string, 0, len(keys))
	for _, key := range keys {
		value := record.ID
		if key != ".id" {
			value = displayValue(key, record.Fields[key])
		}
		lines = append(lines, theme.Default.Muted.Render(key)+"\n"+theme.Default.Text.Render(value))
	}
	return strings.Join(lines, "\n\n")
}

func displayValue(key, value string) string {
	if sensitiveResourceField(key) {
		if value == "" {
			return "—"
		}
		return "••••••••"
	}
	switch key {
	case "running", "enabled", "dynamic", "disabled", "invalid", "default", "hw", "full-duplex", "vlan-filtering":
		if parsed, err := routeros.ParseBool(value); err == nil {
			if parsed {
				return "yes"
			}
			return "no"
		}
	}
	if value == "" {
		return "—"
	}
	return value
}

func sensitiveResourceField(key string) bool {
	normalized := strings.ToLower(strings.ReplaceAll(strings.TrimSpace(key), "_", "-"))
	switch normalized {
	case "password", "passwd", "secret", "passphrase", "private-key",
		"preshared-key", "pre-shared-key", "authentication-key":
		return true
	}
	return strings.Contains(normalized, "password") ||
		strings.HasSuffix(normalized, "-secret")
}

func panel(title, content string, width, height int, focused bool) string {
	border := theme.DefaultPalette.Border
	titleStyle := theme.Default.Muted
	if focused {
		border = theme.DefaultPalette.Focus
		titleStyle = theme.Default.Focus
	}
	heading := titleStyle.Render(title)
	innerWidth := innerPaneWidth(width)
	innerHeight := max(1, height-2)
	canvas := constrainCanvas(heading+"\n"+content, innerWidth, innerHeight)
	return theme.Default.Panel.Border(lipgloss.RoundedBorder()).BorderForeground(border).
		Padding(0, 1).
		Width(max(1, width-2)).Height(innerHeight).Render(canvas)
}

func constrainCanvas(content string, width, height int) string {
	width, height = max(1, width), max(1, height)
	lines := strings.Split(content, "\n")
	if len(lines) > height {
		lines = lines[:height]
	}
	for len(lines) < height {
		lines = append(lines, "")
	}
	for index := range lines {
		lines[index] = ansi.Truncate(lines[index], width, "")
	}
	return strings.Join(lines, "\n")
}

func friendlyError(err error) string {
	var apiErr *routeros.APIError
	if errors.As(err, &apiErr) {
		switch apiErr.Kind {
		case routeros.ErrorAuth:
			return "Authentication failed; check the RouterOS user and REST policy"
		case routeros.ErrorTLS:
			return "TLS verification failed; check the certificate or saved fingerprint"
		case routeros.ErrorTimeout:
			return "Router did not respond before the timeout"
		case routeros.ErrorCanceled:
			return "Request canceled"
		}
	}
	return "Router request failed: " + err.Error()
}

func field(resource routeros.Resource, key, fallback string) string {
	if value := resource.Fields[key]; value != "" {
		return value
	}
	return fallback
}

func formatPin(pin string) string {
	pin = strings.ToUpper(strings.ReplaceAll(pin, ":", ""))
	parts := make([]string, 0, len(pin)/4)
	for len(pin) > 0 {
		size := min(4, len(pin))
		parts = append(parts, pin[:size])
		pin = pin[size:]
	}
	return strings.Join(parts, " ")
}

type discardWriter struct{}

func (discardWriter) Write(data []byte) (int, error) { return len(data), nil }
