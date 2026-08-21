// Package ui provides reusable, keyboard-first Bubble Tea presentation models.
package ui

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type Breakpoint uint8

const (
	Narrow Breakpoint = iota
	Medium
	Wide
)

const (
	MediumWidth = 72
	WideWidth   = 112
)

func BreakpointFor(width int) Breakpoint {
	switch {
	case width >= WideWidth:
		return Wide
	case width >= MediumWidth:
		return Medium
	default:
		return Narrow
	}
}

func (b Breakpoint) String() string {
	return [...]string{"narrow", "medium", "wide"}[b]
}

type SignalLevel uint8

const (
	SignalIdle SignalLevel = iota
	SignalGood
	SignalWarning
	SignalError
)

type Signal struct {
	Label string
	Value string
	Level SignalLevel
}

type SignalRail struct {
	Signals []Signal
	Width   int
}

func (r SignalRail) View() string {
	if len(r.Signals) == 0 || r.Width <= 0 {
		return ""
	}
	parts := make([]string, 0, len(r.Signals))
	for _, s := range r.Signals {
		text := strings.TrimSpace(s.Label + " " + s.Value)
		style := theme.Default.Muted
		switch s.Level {
		case SignalGood:
			style = theme.Default.Signal
		case SignalWarning:
			style = theme.Default.Alert
		case SignalError:
			style = theme.Default.Error
		}
		parts = append(parts, style.Render(text))
	}
	return truncate(lipgloss.JoinHorizontal(lipgloss.Top, partsWithSeparator(parts)...), r.Width)
}

func partsWithSeparator(parts []string) []string {
	out := make([]string, 0, len(parts)*2-1)
	for i, part := range parts {
		if i > 0 {
			out = append(out, theme.Default.Muted.Render(" │ "))
		}
		out = append(out, part)
	}
	return out
}

type MessageKind uint8

const (
	Info MessageKind = iota
	Success
	Warning
	Failure
)

type Status struct {
	Text string
	Kind MessageKind
}

func (s Status) View() string {
	switch s.Kind {
	case Success:
		return theme.Default.Signal.Render(s.Text)
	case Warning:
		return theme.Default.Alert.Render(s.Text)
	case Failure:
		return theme.Default.Error.Render(s.Text)
	default:
		return theme.Default.Muted.Render(s.Text)
	}
}

type Toast struct {
	Status
	Title string
}

func (t Toast) View(width int) string {
	body := strings.TrimSpace(strings.TrimSpace(t.Title) + " " + strings.TrimSpace(t.Text))
	return theme.Default.Panel.Border(lipgloss.RoundedBorder()).Padding(0, 1).
		Width(max(1, width-4)).Render(Status{Text: body, Kind: t.Kind}.View())
}

type Loading struct{ Label string }

func (l Loading) View() string {
	label := l.Label
	if label == "" {
		label = "Loading"
	}
	return theme.Default.Focus.Render("◌ " + label + "…")
}

type EmptyState struct{ Title, Hint string }

func (e EmptyState) View(width int) string {
	return centered(width, theme.Default.Text.Bold(true).Render(e.Title)+"\n"+theme.Default.Muted.Render(e.Hint))
}

type ErrorState struct{ Title, Detail string }

func (e ErrorState) View(width int) string {
	return centered(width, theme.Default.Error.Bold(true).Render(e.Title)+"\n"+theme.Default.Muted.Render(e.Detail))
}

func centered(width int, value string) string {
	if width <= 0 {
		return ""
	}
	return lipgloss.NewStyle().Width(width).Align(lipgloss.Center).Render(value)
}

func truncate(value string, width int) string {
	if width <= 0 {
		return ""
	}
	return lipgloss.NewStyle().MaxWidth(width).Render(value)
}

func clamp(value, low, high int) int {
	if high < low {
		return low
	}
	if value < low {
		return low
	}
	if value > high {
		return high
	}
	return value
}
