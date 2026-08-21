package ui

import (
	"fmt"
	"sort"
	"strconv"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

// MaxFirewallRules is the maximum number of rule rows a dashboard pane shows.
const MaxFirewallRules = 10

// FirewallRuleMetric is a read-only rule counter and its recent packet deltas.
type FirewallRuleMetric struct {
	ID, Label, Action string
	Packets, Bytes    uint64
	RecentPackets     uint64
	RecentBytes       uint64
	History           []float64
}

// FirewallHitChart ranks rules and renders per-rule hit histories.
type FirewallHitChart struct {
	Rules         []FirewallRuleMetric
	Width, Height int
	Offset        int
}

type firewallColumns struct {
	heat, rule, action, spark, now, total int
	compact                               bool
}

func (c FirewallHitChart) View() string {
	width, height := max(1, c.Width), max(1, c.Height)
	if len(c.Rules) == 0 {
		return constrainLines(EmptyState{
			Title: "No firewall counters",
			Hint:  "Filter rules will appear after telemetry loads",
		}.View(width), width, height)
	}
	rules := rankedFirewallRules(c.Rules)
	visible := c.visibleRows()
	offset := clamp(c.Offset, 0, max(0, len(rules)-visible))
	window := rules[offset:min(len(rules), offset+visible)]
	peak := 1.0
	for _, rule := range rules {
		peak = maxFloat(peak, latestHit(rule))
	}
	cols := firewallColumnLayout(width, rules)

	lines := make([]string, 0, height)
	lines = append(lines, c.header(cols, width, offset, visible, len(rules)))
	for _, rule := range window {
		lines = append(lines, c.row(rule, cols, peak))
	}
	return constrainLines(strings.Join(lines, "\n"), width, height)
}

func (c FirewallHitChart) visibleRows() int {
	return min(MaxFirewallRules, max(0, c.Height-1))
}

// MaxOffset is the largest valid scroll offset for the current size.
func (c FirewallHitChart) MaxOffset() int {
	return max(0, len(c.Rules)-c.visibleRows())
}

func rankedFirewallRules(rules []FirewallRuleMetric) []FirewallRuleMetric {
	ranked := append([]FirewallRuleMetric(nil), rules...)
	sort.SliceStable(ranked, func(i, j int) bool {
		left, right := latestHit(ranked[i]), latestHit(ranked[j])
		if left != right {
			return left > right
		}
		if ranked[i].Packets != ranked[j].Packets {
			return ranked[i].Packets > ranked[j].Packets
		}
		return ranked[i].ID < ranked[j].ID
	})
	return ranked
}

func firewallColumnLayout(width int, rules []FirewallRuleMetric) firewallColumns {
	if width < 80 {
		heat, now, gaps := 5, 6, 3
		flex := max(8, width-heat-now-gaps)
		rule := min(max(8, flex/2), max(8, flex-4))
		spark := max(4, flex-rule)
		return firewallColumns{heat: heat, rule: rule, spark: spark, now: now, compact: true}
	}
	heat, now, total := 7, 10, 14
	action := len("ACTION")
	longestRule := len("RULE")
	for _, rule := range rules {
		action = max(action, len([]rune(rule.Action)))
		longestRule = max(longestRule, len([]rune(rule.Label)))
	}
	action = clamp(action, 6, 12)
	gaps := 5
	flex := max(16, width-heat-action-now-total-gaps)
	minSpark := max(8, flex/4)
	rule := min(longestRule, flex-minSpark)
	rule = max(12, rule)
	if rule > flex-minSpark {
		rule = max(8, flex-minSpark)
	}
	spark := max(minSpark, flex-rule)
	return firewallColumns{heat: heat, rule: rule, action: action, spark: spark, now: now, total: total}
}

func (c FirewallHitChart) header(cols firewallColumns, width, offset, visible, total int) string {
	cells := []string{fitCell("HEAT", cols.heat), fitCell("RULE", cols.rule)}
	if !cols.compact {
		cells = append(cells, fitCell("ACTION", cols.action))
	}
	history := "HIT HISTORY"
	if total > visible {
		start := offset + 1
		end := min(total, offset+visible)
		history = fmt.Sprintf("HIT HISTORY %d-%d/%d", start, end, total)
	}
	cells = append(cells, fitCell(history, cols.spark), fitCell("NOW", cols.now))
	if !cols.compact {
		cells = append(cells, fitCell("TOTAL", cols.total))
	}
	return theme.Default.Muted.Render(fitCell(strings.Join(cells, " "), width))
}

func (c FirewallHitChart) row(rule FirewallRuleMetric, cols firewallColumns, peak float64) string {
	current := latestHit(rule)
	style := firewallHeatStyle(current, peak, rule.Packets)
	state := heatLabel(current, peak, rule.Packets)
	spark := BrailleSparkline{
		Samples: rule.History, Width: cols.spark, Height: 1,
		Min: 0, Max: maxHistory(rule.History), Style: style,
	}.View()
	now := "+" + formatCount(rule.RecentPackets)
	if !cols.compact {
		now += " pkt"
	}
	parts := []string{
		style.Render(fitCell(state, cols.heat)),
		theme.Default.Text.Render(fitCell(rule.Label, cols.rule)),
	}
	if !cols.compact {
		parts = append(parts, theme.Default.Muted.Render(fitCell(rule.Action, cols.action)))
	}
	parts = append(parts, spark, style.Render(fitCell(now, cols.now)))
	if !cols.compact {
		total := formatCount(rule.Packets) + " / " + formatMetricBytes(rule.Bytes)
		parts = append(parts, theme.Default.Muted.Render(fitCell(total, cols.total)))
	}
	return strings.Join(parts, " ")
}

func constrainLines(content string, width, height int) string {
	width, height = max(1, width), max(1, height)
	lines := strings.Split(content, "\n")
	if len(lines) > height {
		lines = lines[:height]
	}
	for len(lines) < height {
		lines = append(lines, "")
	}
	for index, line := range lines {
		visual := lipgloss.Width(line)
		switch {
		case visual > width:
			lines[index] = ansi.Truncate(line, width, "")
		case visual < width:
			lines[index] = line + strings.Repeat(" ", width-visual)
		}
	}
	return strings.Join(lines, "\n")
}

func latestHit(rule FirewallRuleMetric) float64 {
	if len(rule.History) == 0 {
		return 0
	}
	return rule.History[len(rule.History)-1]
}

func maxHistory(history []float64) float64 {
	peak := 1.0
	for _, value := range history {
		peak = maxFloat(peak, value)
	}
	return peak
}

func heatLabel(current, peak float64, total uint64) string {
	switch {
	case total == 0:
		return "○ DEAD"
	case current == 0:
		return "· COLD"
	case current/peak >= .66:
		return "● HOT"
	case current/peak >= .25:
		return "◉ WARM"
	default:
		return "• HIT"
	}
}

func firewallHeatStyle(current, peak float64, total uint64) lipgloss.Style {
	if total == 0 {
		return theme.Default.Muted
	}
	ratio := current / maxFloat(1, peak)
	if current == 0 {
		ratio = .15
	}
	return lipgloss.NewStyle().Foreground(blendColor(theme.DefaultPalette.Muted, theme.DefaultPalette.Alert, ratio)).Bold(ratio >= .66)
}

func blendColor(from, to lipgloss.Color, ratio float64) lipgloss.Color {
	ratio = maxFloat(0, minFloat(1, ratio))
	fr, fg, fb := parseHexColor(string(from))
	tr, tg, tb := parseHexColor(string(to))
	mix := func(a, b int64) int64 { return a + int64(float64(b-a)*ratio) }
	return lipgloss.Color(fmt.Sprintf("#%02X%02X%02X", mix(fr, tr), mix(fg, tg), mix(fb, tb)))
}

func parseHexColor(value string) (int64, int64, int64) {
	value = strings.TrimPrefix(value, "#")
	if len(value) != 6 {
		return 127, 141, 158
	}
	parse := func(part string) int64 {
		number, _ := strconv.ParseInt(part, 16, 64)
		return number
	}
	return parse(value[:2]), parse(value[2:4]), parse(value[4:6])
}

func minFloat(left, right float64) float64 {
	if left < right {
		return left
	}
	return right
}

func formatCount(value uint64) string {
	switch {
	case value >= 1_000_000_000:
		return fmt.Sprintf("%.1fG", float64(value)/1_000_000_000)
	case value >= 1_000_000:
		return fmt.Sprintf("%.1fM", float64(value)/1_000_000)
	case value >= 1_000:
		return fmt.Sprintf("%.1fK", float64(value)/1_000)
	default:
		return strconv.FormatUint(value, 10)
	}
}

func formatMetricBytes(value uint64) string {
	switch {
	case value >= 1<<30:
		return fmt.Sprintf("%.1fG", float64(value)/(1<<30))
	case value >= 1<<20:
		return fmt.Sprintf("%.1fM", float64(value)/(1<<20))
	case value >= 1<<10:
		return fmt.Sprintf("%.1fK", float64(value)/(1<<10))
	default:
		return strconv.FormatUint(value, 10) + "B"
	}
}
