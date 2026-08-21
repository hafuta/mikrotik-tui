package ui

import (
	"fmt"
	"sort"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/x/ansi"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type HelpBinding struct {
	Key, Description string
}

type HelpOverlay struct {
	Title    string
	Bindings []HelpBinding
	Visible  bool
	Width    int
	Height   int
	offset   int
}

func (h HelpOverlay) Init() tea.Cmd { return nil }

func (h *HelpOverlay) SetSize(width, height int) {
	h.Width = max(24, width)
	h.Height = max(8, height)
	h.reconcileOffset()
}

func (h HelpOverlay) Update(msg tea.Msg) (HelpOverlay, tea.Cmd) {
	if key, ok := msg.(tea.KeyMsg); ok {
		if !h.Visible {
			if key.String() == "?" {
				h.Visible = true
				h.offset = 0
			}
			return h, nil
		}
		switch key.String() {
		case "?", "esc":
			h.Visible = false
		case "up", "k":
			h.offset--
		case "down", "j":
			h.offset++
		case "pgup", "ctrl+u":
			h.offset -= h.bindingCapacity()
		case "pgdown", "ctrl+d":
			h.offset += h.bindingCapacity()
		case "home", "g":
			h.offset = 0
		case "end", "G":
			h.offset = len(h.Bindings)
		}
		h.reconcileOffset()
	}
	return h, nil
}

func (h HelpOverlay) View() string {
	if !h.Visible {
		return ""
	}
	title := h.Title
	if title == "" {
		title = "Keyboard help"
	}
	width := clamp(h.Width, 24, 72)
	capacity := h.bindingCapacity()
	start := clamp(h.offset, 0, max(0, len(h.Bindings)-capacity))
	end := min(len(h.Bindings), start+capacity)
	if len(h.Bindings) > capacity {
		title += "  " + theme.Default.Muted.Render(
			formatRange(start+1, end, len(h.Bindings)),
		)
	}
	lines := []string{theme.Default.Focus.Render(title)}
	for _, binding := range h.Bindings[start:end] {
		keyWidth := min(16, max(6, width/4))
		lines = append(lines, theme.Default.Signal.Render(fitCell(binding.Key, keyWidth))+" "+binding.Description)
	}
	lines = append(lines, "", theme.Default.Muted.Render("↑↓ scroll  ? / esc close"))
	return overlayBox(strings.Join(lines, "\n"), width)
}

func (h HelpOverlay) bindingCapacity() int {
	height := h.Height
	if height <= 0 {
		height = 24
	}
	// Four cells are used by the border/padding and three content lines by the
	// title, spacer, and footer. Keep an outer margin around the modal.
	return max(1, height-11)
}

func (h *HelpOverlay) reconcileOffset() {
	h.offset = clamp(h.offset, 0, max(0, len(h.Bindings)-h.bindingCapacity()))
}

func formatRange(start, end, total int) string {
	return fmt.Sprintf("%d–%d/%d", start, end, total)
}

type Command struct {
	ID, Title, Description, Path string
	Run                          tea.Cmd
}

type CommandPalette struct {
	Commands []Command
	Query    string
	Visible  bool
	Width    int
	selected int
}

const paletteVisibleRows = 8

func NewCommandPalette(commands []Command) CommandPalette {
	return CommandPalette{Commands: commands, Width: 56}
}

func (p CommandPalette) matches() []Command {
	query := strings.ToLower(strings.TrimSpace(p.Query))
	if query == "" {
		return append([]Command(nil), p.Commands...)
	}
	type ranked struct {
		command Command
		score   int
		order   int
	}
	rankedMatches := make([]ranked, 0, len(p.Commands))
	for index, command := range p.Commands {
		if score := commandMatchScore(command, query); score > 0 {
			rankedMatches = append(rankedMatches, ranked{command: command, score: score, order: index})
		}
	}
	sort.SliceStable(rankedMatches, func(i, j int) bool {
		if rankedMatches[i].score != rankedMatches[j].score {
			return rankedMatches[i].score > rankedMatches[j].score
		}
		return rankedMatches[i].order < rankedMatches[j].order
	})
	matches := make([]Command, len(rankedMatches))
	for index, item := range rankedMatches {
		matches[index] = item.command
	}
	return matches
}

func commandMatchScore(command Command, query string) int {
	path := strings.ToLower(strings.TrimSpace(command.Path))
	title := strings.ToLower(command.Title)
	description := strings.ToLower(command.Description)
	if path == "" {
		path = title
	}
	switch {
	case path == query || path == "/"+query:
		return 300
	case strings.HasPrefix(path, query) || strings.HasPrefix(path, "/"+query):
		return 200
	case strings.Contains(path, "/"+query):
		return 150
	case strings.Contains(path, query):
		return 100
	case strings.Contains(title, query) || strings.Contains(description, query):
		return 50
	default:
		return 0
	}
}

func (p CommandPalette) Selected() (Command, bool) {
	matches := p.matches()
	if len(matches) == 0 {
		return Command{}, false
	}
	return matches[clamp(p.selected, 0, len(matches)-1)], true
}

func (p CommandPalette) Init() tea.Cmd { return nil }

func (p CommandPalette) Update(msg tea.Msg) (CommandPalette, tea.Cmd) {
	key, ok := msg.(tea.KeyMsg)
	if !ok {
		return p, nil
	}
	if !p.Visible {
		if key.String() == "ctrl+p" {
			p.Visible = true
			p.Query, p.selected = "", 0
		}
		return p, nil
	}
	switch key.Type {
	case tea.KeyEscape:
		p.Visible = false
	case tea.KeyEnter:
		if command, ok := p.Selected(); ok {
			p.Visible = false
			return p, command.Run
		}
	case tea.KeyUp:
		p.selected--
	case tea.KeyDown:
		p.selected++
	case tea.KeyBackspace:
		if len(p.Query) > 0 {
			runes := []rune(p.Query)
			p.Query = string(runes[:len(runes)-1])
			p.selected = 0
		}
	case tea.KeySpace:
		p.Query += " "
		p.selected = 0
	case tea.KeyRunes:
		if value := printableInput(key.Runes); value != "" {
			p.Query += value
			p.selected = 0
		}
	}
	matches := p.matches()
	p.selected = clamp(p.selected, 0, len(matches)-1)
	return p, nil
}

func (p CommandPalette) View() string {
	if !p.Visible {
		return ""
	}
	width := clamp(p.Width, 24, 72)
	matches := p.matches()
	title := theme.Default.Focus.Render("Command palette")
	start, end := p.visibleRange(len(matches))
	if len(matches) > paletteVisibleRows {
		title += "  " + theme.Default.Muted.Render(formatRange(start+1, end, len(matches)))
	}
	lines := []string{title, theme.Default.Text.Render("> " + p.Query + "▏")}
	for index := start; index < end; index++ {
		lines = append(lines, p.renderMatch(matches[index], index == p.selected, width))
	}
	if len(matches) == 0 {
		lines = append(lines, theme.Default.Muted.Render("  No matching commands"))
	}
	lines = append(lines, theme.Default.Muted.Render("↑↓ choose  enter open  esc close"))
	return overlayBox(strings.Join(lines, "\n"), width)
}

func (p CommandPalette) visibleRange(total int) (int, int) {
	if total == 0 {
		return 0, 0
	}
	start := 0
	if p.selected >= paletteVisibleRows {
		start = p.selected - paletteVisibleRows + 1
	}
	return start, min(total, start+paletteVisibleRows)
}

func (p CommandPalette) renderMatch(command Command, selected bool, width int) string {
	query := strings.ToLower(strings.TrimSpace(p.Query))
	base := theme.Default.Text
	if selected {
		base = theme.Default.Focus
	}
	match := theme.Default.Signal
	if selected {
		match = theme.Default.Signal.Bold(true)
	}
	line := highlightMatch(command.Title, query, base, match)
	if command.Description != "" {
		line += base.Render(" — ") + highlightMatch(command.Description, query, theme.Default.Muted, match)
	}
	line = ansi.Cut(line, 0, max(1, width-4))
	if selected {
		return theme.Default.Focus.Render("› ") + line
	}
	return "  " + line
}

func highlightMatch(text, query string, base, match lipgloss.Style) string {
	if query == "" || text == "" {
		return base.Render(text)
	}
	index := strings.Index(strings.ToLower(text), strings.ToLower(query))
	if index < 0 {
		return base.Render(text)
	}
	end := index + len(query)
	if end > len(text) {
		return base.Render(text)
	}
	return base.Render(text[:index]) + match.Render(text[index:end]) + base.Render(text[end:])
}

func overlayBox(content string, width int) string {
	return theme.Default.Panel.BorderStyle(lipgloss.RoundedBorder()).
		BorderForeground(theme.DefaultPalette.Border).Padding(1, 2).Width(max(1, width-6)).Render(content)
}

// Modal centers an opaque terminal dialog over a dimmed, fixed-size copy of
// the current layout. It deliberately uses foreground dimming rather than a
// background color so the backdrop cannot bleed outside the canvas.
func Modal(base, modal string, width, height int) string {
	width, height = max(1, width), max(1, height)
	baseLines := strings.Split(ansi.Strip(base), "\n")
	backdropStyle := lipgloss.NewStyle().Foreground(theme.DefaultPalette.Muted).Faint(true)
	backdrop := make([]string, height)
	for row := 0; row < height; row++ {
		line := ""
		if row < len(baseLines) {
			line = baseLines[row]
		}
		backdrop[row] = backdropStyle.Render(fitDisplayWidth(line, width))
	}

	modalLines := strings.Split(modal, "\n")
	modalWidth := min(width, maxLineWidth(modalLines))
	modalHeight := min(height, len(modalLines))
	left := max(0, (width-modalWidth)/2)
	top := max(0, (height-modalHeight)/2)
	for row := 0; row < modalHeight; row++ {
		modalLine := fitDisplayWidth(modalLines[row], modalWidth)
		backdropRow := backdrop[top+row]
		before := ansi.Cut(backdropRow, 0, left)
		after := ansi.Cut(backdropRow, left+modalWidth, width)
		backdrop[top+row] = before + modalLine + after
	}
	return strings.Join(backdrop, "\n")
}

func fitDisplayWidth(value string, width int) string {
	value = ansi.Cut(value, 0, max(0, width))
	return value + strings.Repeat(" ", max(0, width-lipgloss.Width(value)))
}

func maxLineWidth(lines []string) int {
	width := 0
	for _, line := range lines {
		width = max(width, lipgloss.Width(line))
	}
	return width
}
