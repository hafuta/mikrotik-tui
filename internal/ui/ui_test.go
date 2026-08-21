package ui

import (
	"fmt"
	"strings"
	"testing"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

func key(keyType tea.KeyType) tea.KeyMsg { return tea.KeyMsg{Type: keyType} }

func TestBreakpointFor(t *testing.T) {
	tests := []struct {
		width int
		want  Breakpoint
	}{
		{0, Narrow}, {71, Narrow},
		{72, Medium}, {111, Medium},
		{112, Wide}, {240, Wide},
	}
	for _, test := range tests {
		if got := BreakpointFor(test.width); got != test.want {
			t.Fatalf("BreakpointFor(%d) = %v, want %v", test.width, got, test.want)
		}
	}
}

func TestNavigationHierarchyAndSelection(t *testing.T) {
	navigation := NewNavigation([]NavItem{
		{ID: "system", Label: "System", Children: []NavItem{
			{ID: "identity", Label: "Identity"},
			{ID: "users", Label: "Users"},
		}},
		{ID: "interfaces", Label: "Interfaces"},
	})
	if navigation.Selected != "system" {
		t.Fatalf("initial selection = %q", navigation.Selected)
	}
	navigation, _ = navigation.Update(key(tea.KeyRight))
	navigation, _ = navigation.Update(key(tea.KeyDown))
	if navigation.Selected != "identity" {
		t.Fatalf("child selection = %q", navigation.Selected)
	}
	navigation, _ = navigation.Update(key(tea.KeyLeft))
	if navigation.Selected != "system" {
		t.Fatalf("left should select parent, got %q", navigation.Selected)
	}
	navigation, _ = navigation.Update(key(tea.KeyLeft))
	if strings.Contains(navigation.View(), "Identity") {
		t.Fatal("collapsed navigation still renders child")
	}
}

func TestNavigationExpandingItemCollapsesOthers(t *testing.T) {
	navigation := NewNavigation([]NavItem{
		{ID: "system", Label: "System", Children: []NavItem{
			{ID: "users", Label: "Users"},
		}},
		{ID: "interfaces", Label: "Interfaces", Children: []NavItem{
			{ID: "ethernet", Label: "Ethernet"},
		}},
	})

	navigation, _ = navigation.Update(key(tea.KeyRight))
	navigation, _ = navigation.Update(key(tea.KeyDown))
	navigation, _ = navigation.Update(key(tea.KeyDown))
	if navigation.Selected != "interfaces" {
		t.Fatalf("selection = %q, want interfaces", navigation.Selected)
	}

	navigation, _ = navigation.Update(key(tea.KeyRight))
	view := navigation.View()
	if strings.Contains(view, "Users") {
		t.Fatalf("previous menu item remained expanded: %q", view)
	}
	if !strings.Contains(view, "Ethernet") {
		t.Fatalf("new menu item was not expanded: %q", view)
	}
}

func TestNavigationScrollsWithinFixedHeight(t *testing.T) {
	navigation := NewNavigation([]NavItem{
		{ID: "one", Label: "One"},
		{ID: "two", Label: "Two"},
		{ID: "three", Label: "Three"},
		{ID: "four", Label: "Four"},
	})
	navigation.SetHeight(2)
	for range 3 {
		navigation, _ = navigation.Update(key(tea.KeyDown))
	}
	if navigation.Selected != "four" || navigation.offset != 2 {
		t.Fatalf("selection/offset = %q/%d", navigation.Selected, navigation.offset)
	}
	view := navigation.View()
	if strings.Contains(view, "One") || !strings.Contains(view, "Four") {
		t.Fatalf("fixed-height navigation view = %q", view)
	}
	if lines := strings.Count(view, "\n") + 1; lines != 2 {
		t.Fatalf("navigation rendered %d lines, want 2", lines)
	}
}

func tableFixture() ResourceTable {
	return NewResourceTable(
		[]Column{
			{Key: "name", Title: "Name", Width: 12},
			{Key: "address", Title: "Address", Width: 18},
			{Key: "rx", Title: "RX", Width: 6},
		},
		[]ResourceRow{
			{ID: "ether1", Cells: map[string]string{"name": "WAN", "address": "192.0.2.1", "rx": "10"}},
			{ID: "ether2", Cells: map[string]string{"name": "Office", "address": "10.0.0.1", "rx": "2"}},
			{ID: "wlan1", Cells: map[string]string{"name": "Guest WiFi", "address": "10.0.1.1", "rx": "2"}},
			{ID: "bridge", Cells: map[string]string{"name": "Bridge", "address": "10.0.2.1", "rx": "100"}},
		},
	)
}

func TestTableFilterSortAndStableSelection(t *testing.T) {
	table := tableFixture()
	table.SelectedID = "ether2"
	table.SetSort("rx", SortDescending)
	rows := table.VisibleRows()
	if rows[0].ID != "bridge" || rows[1].ID != "ether1" {
		t.Fatalf("numeric descending order = %v", rowIDs(rows))
	}
	if table.SelectedID != "ether2" {
		t.Fatalf("sort changed stable selection to %q", table.SelectedID)
	}
	table.SetFilter("office")
	rows = table.VisibleRows()
	if len(rows) != 1 || rows[0].ID != "ether2" || table.SelectedID != "ether2" {
		t.Fatalf("filter result = %v, selection = %q", rowIDs(rows), table.SelectedID)
	}
	table.SetFilter("guest")
	if table.SelectedID != "wlan1" {
		t.Fatalf("hidden selection did not reconcile, got %q", table.SelectedID)
	}
}

func TestTableScrollingAndResizing(t *testing.T) {
	table := tableFixture()
	table.SetSize(18, 3)
	table, _ = table.Update(key(tea.KeyEnd))
	if table.SelectedID != "bridge" || table.verticalOffset == 0 {
		t.Fatalf("end selection/offset = %q/%d", table.SelectedID, table.verticalOffset)
	}
	table, _ = table.Update(key(tea.KeyRight))
	if table.horizontal == 0 {
		t.Fatal("horizontal offset did not advance")
	}
	table.SetSize(100, 20)
	if table.verticalOffset != 0 || table.horizontal != 0 {
		t.Fatalf("resize offsets = vertical %d, horizontal %d", table.verticalOffset, table.horizontal)
	}
}

func TestInspectorScrollingAndResize(t *testing.T) {
	inspector := NewInspector("Details", strings.Repeat("abcdefghij\n", 8))
	inspector.SetSize(8, 4)
	inspector, _ = inspector.Update(key(tea.KeyEnd))
	if inspector.viewport.YOffset == 0 {
		t.Fatal("inspector did not scroll")
	}
	inspector.SetSize(80, 30)
	if inspector.viewport.YOffset != 0 {
		t.Fatalf("expanded inspector offset = %d", inspector.viewport.YOffset)
	}
}

func TestInspectorLiveUpdatePreservesReadingPosition(t *testing.T) {
	inspector := NewInspector("Details", strings.Repeat("before\n", 20))
	inspector.SetSize(20, 5)
	inspector.viewport.SetYOffset(7)
	inspector.SetContentPreservingOffset(strings.Repeat("after\n", 20))
	if inspector.viewport.YOffset != 7 {
		t.Fatalf("live update reset offset to %d", inspector.viewport.YOffset)
	}
	inspector.SetContent("new selection")
	if inspector.viewport.YOffset != 0 {
		t.Fatalf("new selection did not reset offset: %d", inspector.viewport.YOffset)
	}
}

func TestInspectorScrollIndicatorsHideAtEdges(t *testing.T) {
	inspector := NewInspector("Firewall · *46", "one\ntwo\nthree\nfour\nfive\nsix")
	inspector.SetSize(40, 4)

	top := ansi.Strip(inspector.View())
	if !strings.Contains(top, "line 1/6 ↓") || strings.Contains(top, "↑ line") {
		t.Fatalf("top scroll indicator = %q", top)
	}
	inspector.viewport.SetYOffset(1)
	middle := ansi.Strip(inspector.View())
	if !strings.Contains(middle, "↑ line 2/6 ↓") {
		t.Fatalf("middle scroll indicator = %q", middle)
	}
	inspector.viewport.GotoBottom()
	bottom := ansi.Strip(inspector.View())
	if !strings.Contains(bottom, "↑ line 4/6") || strings.Contains(bottom, "6 ↓") {
		t.Fatalf("bottom scroll indicator = %q", bottom)
	}
}

func TestLoginFocus(t *testing.T) {
	login := NewLoginInput()
	if view := login.View(); !strings.Contains(view, "https://192.168.88.1:8443") {
		t.Fatalf("login view does not contain REST base placeholder: %q", view)
	}
	if login.FocusIndex() != 0 {
		t.Fatalf("initial focus = %d", login.FocusIndex())
	}
	login, _ = login.Update(key(tea.KeyTab))
	login, _ = login.Update(key(tea.KeyEnter))
	if login.FocusIndex() != 2 {
		t.Fatalf("focus after navigation = %d", login.FocusIndex())
	}
	login, _ = login.Update(tea.KeyMsg{Type: tea.KeyShiftTab})
	if login.FocusIndex() != 1 {
		t.Fatalf("focus after reverse navigation = %d", login.FocusIndex())
	}
}

func TestLoginIgnoresModifierAndControlRunes(t *testing.T) {
	login := NewLoginInput()
	login.SetValues(Credentials{Address: "https://router", Username: "reader"})
	login, _ = login.Update(key(tea.KeyTab))
	login, _ = login.Update(key(tea.KeyTab))

	for _, runes := range [][]rune{{0}, {'\r'}, {'\n'}, {0x1b}} {
		login, _ = login.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: runes})
	}
	if password := login.Values().Password; password != "" {
		t.Fatalf("modifier/control key inserted password data: %q", password)
	}

	login, _ = login.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("Pässword 1")})
	if password := login.Values().Password; password != "Pässword 1" {
		t.Fatalf("printable password input = %q", password)
	}
}

func TestCommandPaletteOpensWithCtrlK(t *testing.T) {
	palette := NewCommandPalette([]Command{{ID: "refresh", Title: "Refresh"}})
	palette, _ = palette.Update(tea.KeyMsg{Type: tea.KeyCtrlP})
	if palette.Visible {
		t.Fatal("ctrl+p opened command palette")
	}
	palette, _ = palette.Update(tea.KeyMsg{Type: tea.KeyCtrlK})
	if !palette.Visible {
		t.Fatal("ctrl+k did not open command palette")
	}
}

func commandPaletteFixture() CommandPalette {
	return NewCommandPalette([]Command{
		{ID: "arp", Title: "/ip/arp", Description: "ARP", Path: "/ip/arp"},
		{ID: "address", Title: "/ip/address", Description: "Addresses", Path: "/ip/address"},
		{ID: "filter", Title: "/ip/firewall/filter", Description: "Firewall", Path: "/ip/firewall/filter"},
		{ID: "iface", Title: "/interface", Description: "Interface", Path: "/interface"},
		{ID: "leases", Title: "/ip/dhcp-server/lease", Description: "Leases", Path: "/ip/dhcp-server/lease"},
		{ID: "refresh", Title: "Refresh", Description: "reload the current resource"},
	})
}

func TestCommandPaletteMatchesRouterOSPaths(t *testing.T) {
	tests := []struct {
		query string
		want  []string
	}{
		{query: "ip", want: []string{"arp", "address", "filter", "leases"}},
		{query: "/IP/firewall", want: []string{"filter"}},
		{query: "leases", want: []string{"leases"}},
		{query: "nat", want: nil},
	}
	for _, test := range tests {
		palette := commandPaletteFixture()
		palette.Query = test.query
		var got []string
		for _, command := range palette.matches() {
			got = append(got, command.ID)
		}
		if len(got) != len(test.want) {
			t.Fatalf("query %q matches = %v, want %v", test.query, got, test.want)
		}
		for index, id := range test.want {
			if got[index] != id {
				t.Fatalf("query %q matches = %v, want %v", test.query, got, test.want)
			}
		}
	}
}

func TestCommandPaletteHighlightsMatchedPath(t *testing.T) {
	base := lipgloss.NewStyle()
	match := lipgloss.NewStyle().Bold(true)
	got := highlightMatch("/ip/firewall/filter", "ip", base, match)
	want := "/" + match.Render("ip") + "/firewall/filter"
	if got != want {
		t.Fatalf("highlight = %q, want %q", got, want)
	}

	palette := commandPaletteFixture()
	palette.Visible, palette.Query = true, "ip"
	view := palette.View()
	if !strings.Contains(ansi.Strip(view), "/ip/arp") {
		t.Fatalf("palette view missing path: %q", ansi.Strip(view))
	}
	highlighted := highlightMatch("/ip/arp", "ip", theme.Default.Focus, theme.Default.Signal.Bold(true))
	if !strings.Contains(view, highlighted) {
		t.Fatal("matched path was not highlighted")
	}
}

func TestCommandPaletteEnterRunsSelectedCommand(t *testing.T) {
	var ran string
	palette := NewCommandPalette([]Command{{
		ID: "filter", Title: "/ip/firewall/filter", Path: "/ip/firewall/filter",
		Run: func() tea.Msg {
			ran = "filter"
			return nil
		},
	}})
	palette.Visible = true
	palette.Query = "firewall"
	palette, command := palette.Update(key(tea.KeyEnter))
	if command == nil {
		t.Fatal("enter did not return the selected command")
	}
	command()
	if ran != "filter" {
		t.Fatalf("ran %q, want filter", ran)
	}
	if palette.Visible {
		t.Fatal("palette stayed open after enter")
	}
}

func TestCommandPaletteIgnoresNonPrintableInput(t *testing.T) {
	palette := commandPaletteFixture()
	palette.Visible = true
	palette, _ = palette.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune{0, 1}})
	if palette.Query != "" {
		t.Fatalf("control runes leaked into query: %q", palette.Query)
	}
	palette, _ = palette.Update(tea.KeyMsg{Type: tea.KeyRunes, Runes: []rune("IP")})
	if palette.Query != "IP" {
		t.Fatalf("query = %q", palette.Query)
	}
}

func TestCommandPaletteScrollsVisibleMatches(t *testing.T) {
	commands := make([]Command, 12)
	for index := range commands {
		commands[index] = Command{ID: fmt.Sprintf("item-%02d", index), Title: fmt.Sprintf("/ip/item-%02d", index), Path: fmt.Sprintf("/ip/item-%02d", index)}
	}
	palette := NewCommandPalette(commands)
	palette.Visible, palette.Query = true, "ip"
	for range 8 {
		palette, _ = palette.Update(key(tea.KeyDown))
	}
	view := ansi.Strip(palette.View())
	if strings.Contains(view, "/ip/item-00") || !strings.Contains(view, "/ip/item-08") {
		t.Fatalf("scrolled palette = %q", view)
	}
}

func TestNavigationRevealExpandsAncestors(t *testing.T) {
	navigation := NewNavigation([]NavItem{
		{ID: "dashboard", Label: "Dashboard"},
		{ID: "ip-group", Label: "IP", Children: []NavItem{
			{ID: "arp", Label: "ARP"},
			{ID: "firewall-filter", Label: "Firewall"},
		}},
		{ID: "system-group", Label: "System", Children: []NavItem{
			{ID: "users", Label: "Users"},
		}},
	})
	navigation, _ = navigation.Update(key(tea.KeyDown))
	navigation, _ = navigation.Update(key(tea.KeyRight))
	if !strings.Contains(navigation.View(), "ARP") {
		t.Fatal("setup failed to expand IP group")
	}
	if !navigation.Reveal("firewall-filter") {
		t.Fatal("Reveal returned false")
	}
	if navigation.Selected != "firewall-filter" {
		t.Fatalf("selected = %q", navigation.Selected)
	}
	view := navigation.View()
	if !strings.Contains(view, "Firewall") {
		t.Fatalf("revealed child missing: %q", view)
	}
	if strings.Contains(view, "Users") {
		t.Fatalf("other group stayed expanded: %q", view)
	}
}

func TestHelpModalScrollsWithinWindowHeight(t *testing.T) {
	help := HelpOverlay{
		Visible: true,
		Bindings: []HelpBinding{
			{Key: "1", Description: "first"},
			{Key: "2", Description: "second"},
			{Key: "3", Description: "third"},
		},
	}
	help.SetSize(40, 12)
	first := ansi.Strip(help.View())
	if !strings.Contains(first, "first") || strings.Contains(first, "second") {
		t.Fatalf("initial clipped help = %q", first)
	}
	help, _ = help.Update(key(tea.KeyDown))
	second := ansi.Strip(help.View())
	if !strings.Contains(second, "second") || strings.Contains(second, "first") {
		t.Fatalf("scrolled help = %q", second)
	}
	if lipgloss.Height(help.View()) > 8 {
		t.Fatalf("help exceeds centered modal allowance: %d", lipgloss.Height(help.View()))
	}
}

func TestModalCentersOverFixedDimmedCanvas(t *testing.T) {
	base := strings.Repeat("0123456789012345678901234567890123456789\n", 14) +
		"0123456789012345678901234567890123456789"
	dialog := overlayBox("Keyboard help\n\nesc close", 24)
	rendered := Modal(base, dialog, 40, 15)
	if lipgloss.Width(rendered) != 40 || lipgloss.Height(rendered) != 15 {
		t.Fatalf("modal canvas = %dx%d", lipgloss.Width(rendered), lipgloss.Height(rendered))
	}
	lines := strings.Split(ansi.Strip(rendered), "\n")
	dialogWidth := lipgloss.Width(dialog)
	dialogHeight := lipgloss.Height(dialog)
	top, left := (15-dialogHeight)/2, (40-dialogWidth)/2
	if got := strings.Index(lines[top], "╭"); got != left {
		t.Fatalf("modal left = %d, want %d; line=%q", got, left, lines[top])
	}
	if !strings.HasPrefix(lines[0], "012345") {
		t.Fatalf("backdrop canvas was discarded: %q", lines[0])
	}
}

func TestTrafficChartIsDeterministicAndBounded(t *testing.T) {
	chart := TrafficChart{
		Width:  30,
		Height: 6,
		Samples: []TrafficSample{
			{RX: 1_000_000, TX: 500_000},
			{RX: 8_000_000, TX: 2_000_000},
			{RX: 3_000_000, TX: 7_000_000},
			{RX: 10_000_000, TX: 4_000_000},
		},
	}
	first, second := chart.View(), chart.View()
	if first != second {
		t.Fatal("traffic chart changed between identical renders")
	}
	if lipgloss.Width(first) != 30 || lipgloss.Height(first) != 6 {
		t.Fatalf("traffic chart = %dx%d", lipgloss.Width(first), lipgloss.Height(first))
	}
	hasBraille := false
	for _, value := range ansi.Strip(first) {
		if value >= 0x2800 && value <= 0x28ff {
			hasBraille = true
			break
		}
	}
	if !hasBraille {
		t.Fatalf("traffic chart contains no Braille plot: %q", first)
	}
	plain := ansi.Strip(first)
	if !strings.Contains(plain, "Mbps") || !strings.Contains(plain, "0 bps") ||
		!strings.Contains(plain, "-32s") || !strings.Contains(plain, "now") {
		t.Fatalf("traffic chart axes lack scale or time units: %q", plain)
	}
	if strings.Contains(first, "[48;") {
		t.Fatal("traffic chart paints a terminal background")
	}
}

func TestTrafficChartRightAlignsSparseRecentSamples(t *testing.T) {
	chart := TrafficChart{
		Width: 40, Height: 5,
		Samples: []TrafficSample{{RX: 1_000_000}, {RX: 8_000_000}},
	}
	for _, line := range strings.Split(ansi.Strip(chart.View()), "\n")[:4] {
		separator := strings.Index(line, "│ ")
		if separator < 0 {
			t.Fatalf("plot row has no axis: %q", line)
		}
		plot := []rune(line[separator+len("│ "):])
		for _, value := range plot[:max(0, len(plot)-4)] {
			if value >= 0x2800 && value <= 0x28ff {
				t.Fatalf("sparse samples were stretched across history: %q", line)
			}
		}
	}
}

func TestBrailleSparklineIsBoundedAndRightAligned(t *testing.T) {
	spark := BrailleSparkline{
		Samples: []float64{10, 40, 20, 90},
		Width:   20, Height: 2, Min: 0, Max: 100, Style: theme.Default.Signal,
	}.View()
	if lipgloss.Width(spark) != 20 || lipgloss.Height(spark) != 2 {
		t.Fatalf("sparkline = %dx%d", lipgloss.Width(spark), lipgloss.Height(spark))
	}
	plain := ansi.Strip(spark)
	if strings.Contains(spark, "[48;") {
		t.Fatal("sparkline paints a terminal background")
	}
	firstBraille := -1
	for index, value := range []rune(plain) {
		if value >= 0x2800 && value <= 0x28ff {
			firstBraille = index
			break
		}
	}
	if firstBraille < 12 {
		t.Fatalf("sparse sparkline was not right aligned: %q", plain)
	}
}

func TestFirewallHitChartShowsHotAndDeadRulesWithoutColor(t *testing.T) {
	chart := FirewallHitChart{
		Width: 80, Height: 5,
		Rules: []FirewallRuleMetric{
			{ID: "*1", Label: "established", Action: "accept", Packets: 1200, Bytes: 900000, RecentPackets: 200, History: []float64{10, 30, 200}},
			{ID: "*2", Label: "dns", Action: "accept", Packets: 50, Bytes: 5000, RecentPackets: 2, History: []float64{1, 0, 2}},
			{ID: "*3", Label: "unused legacy", Action: "drop", Packets: 0, Bytes: 0, History: []float64{0, 0, 0}},
			{ID: "*4", Label: "scanner", Action: "drop", Packets: 800, Bytes: 64000, RecentPackets: 40, History: []float64{5, 20, 40}},
		},
	}
	rendered := chart.View()
	plain := ansi.Strip(rendered)
	if lipgloss.Width(rendered) > 80 || lipgloss.Height(rendered) != 5 {
		t.Fatalf("firewall chart = %dx%d", lipgloss.Width(rendered), lipgloss.Height(rendered))
	}
	if !strings.Contains(plain, "HOT") || !strings.Contains(plain, "DEAD") ||
		!strings.Contains(plain, "unused legacy") {
		t.Fatalf("firewall chart lacks non-color heat states: %q", plain)
	}
	if strings.Contains(rendered, "[48;") {
		t.Fatal("firewall chart paints a terminal background")
	}
}

func TestFirewallHitChartAlignsColumnsAndUsesAvailableWidth(t *testing.T) {
	rules := []FirewallRuleMetric{
		{ID: "*1", Label: "special dummy rule to prevent accidental lockout", Action: "passthrough", Packets: 737_300_000, Bytes: 4_200_000_000, RecentPackets: 227, History: []float64{10, 227}},
		{ID: "*2", Label: "Allow Related, Established", Action: "accept", Packets: 20, Bytes: 2000, RecentPackets: 2, History: []float64{2}},
	}
	chart := FirewallHitChart{Width: 120, Height: 4, Rules: rules}
	cols := firewallColumnLayout(120, rules)
	if cols.rule < 30 {
		t.Fatalf("rule column did not grow with width: %+v", cols)
	}
	if cols.action < len("passthrough") {
		t.Fatalf("action column truncated passthrough: %+v", cols)
	}
	plain := strings.Split(ansi.Strip(chart.View()), "\n")
	if len(plain) < 2 {
		t.Fatal("expected header and at least one rule")
	}
	header, row := plain[0], plain[1]
	nowAt := strings.Index(header, "NOW")
	totalAt := strings.Index(header, "TOTAL")
	if nowAt < 0 || totalAt < 0 {
		t.Fatalf("missing NOW/TOTAL headers: %q", header)
	}
	if !strings.Contains(row[nowAt:], "+227 pkt") {
		t.Fatalf("NOW value is not under NOW header: header=%q row=%q", header, row)
	}
	if !strings.Contains(row[totalAt:], "737.3M") {
		t.Fatalf("TOTAL value is not under TOTAL header: header=%q row=%q", header, row)
	}
	if strings.Contains(row, "passthr…") || !strings.Contains(row, "passthrough") {
		t.Fatalf("action still truncated: %q", row)
	}
	if strings.Contains(row, "special dummy rule to…") {
		t.Fatalf("rule label still truncated despite spare width: %q", row)
	}
}

func TestFirewallHitChartScrollsPastVisibleWindow(t *testing.T) {
	rules := make([]FirewallRuleMetric, 12)
	for index := range rules {
		rules[index] = FirewallRuleMetric{
			ID: fmt.Sprintf("*%d", index+1), Label: fmt.Sprintf("rule-%02d", index+1),
			Action: "accept", Packets: uint64(12-index) * 10, RecentPackets: uint64(12 - index),
			History: []float64{float64(12 - index)},
		}
	}
	chart := FirewallHitChart{Width: 90, Height: 4, Rules: rules}
	first := ansi.Strip(chart.View())
	if !strings.Contains(first, "rule-01") || strings.Contains(first, "rule-12") {
		t.Fatalf("initial window = %q", first)
	}
	chart.Offset = chart.MaxOffset()
	last := ansi.Strip(chart.View())
	if !strings.Contains(last, "rule-12") || strings.Contains(last, "rule-01") {
		t.Fatalf("scrolled window = %q", last)
	}
}

func TestViewsAreDeterministicAndBoundedAtWidths(t *testing.T) {
	for _, width := range []int{32, 80, 128} {
		table := tableFixture()
		table.SetSize(width, 8)
		navigation := NewNavigation([]NavItem{{ID: "a", Label: "Resources"}, {ID: "b", Label: "Logs"}})
		navigation.SetSize(width)
		inspector := NewInspector("Inspector", "first\nsecond\nthird")
		inspector.SetSize(width, 6)
		rail := SignalRail{Width: width, Signals: []Signal{{Label: "router", Value: "online", Level: SignalGood}}}
		login := NewLoginInput()
		login.SetSize(width)
		help := HelpOverlay{
			Visible: true,
			Width:   width,
			Bindings: []HelpBinding{
				{Key: "j/k", Description: "move selection"},
				{Key: "enter", Description: "open resource"},
			},
		}
		palette := NewCommandPalette([]Command{{ID: "refresh", Title: "Refresh resources"}})
		palette.Visible, palette.Width = true, width
		render := func() []string {
			return []string{
				table.View(), navigation.View(), inspector.View(), rail.View(),
				login.View(), help.View(), palette.View(),
				EmptyState{Title: "Nothing here", Hint: "Change the filter"}.View(width),
			}
		}
		views := render()
		for index, view := range views {
			second := render()[index]
			if view != second {
				t.Fatal("view changed between identical renders")
			}
			for _, line := range strings.Split(view, "\n") {
				if got := lipgloss.Width(line); got > width {
					t.Fatalf("width %d view line has width %d: %q", width, got, line)
				}
			}
		}
	}
}

func rowIDs(rows []ResourceRow) []string {
	ids := make([]string, len(rows))
	for index := range rows {
		ids[index] = rows[index].ID
	}
	return ids
}
