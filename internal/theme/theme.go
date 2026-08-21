// Package theme defines the semantic visual language shared by the TUI.
package theme

import "github.com/charmbracelet/lipgloss"

// Palette contains semantic colors. Components should depend on these names,
// never on feature-specific color literals.
type Palette struct {
	Void, Panel, Text, Focus, Signal, Alert lipgloss.Color
	Muted, Border, Error                    lipgloss.Color
}

// DefaultPalette is the application's canonical dark palette.
var DefaultPalette = Palette{
	Void:   lipgloss.Color("#090D13"),
	Panel:  lipgloss.Color("#121A26"),
	Text:   lipgloss.Color("#DCE7F3"),
	Focus:  lipgloss.Color("#62A8FF"),
	Signal: lipgloss.Color("#55D6BE"),
	Alert:  lipgloss.Color("#FFB454"),
	Muted:  lipgloss.Color("#7F8D9E"),
	Border: lipgloss.Color("#293849"),
	Error:  lipgloss.Color("#FF6B7A"),
}

// Styles is a compact set of reusable semantic styles.
type Styles struct {
	Base, Panel, Text, Muted, Focus, Signal, Alert, Error lipgloss.Style
}

func New(p Palette) Styles {
	return Styles{
		// Background colors are intentionally omitted. Terminal background
		// painting is not reliably bounded across renderers and causes bleed
		// around padding, wide glyphs, and partial differential redraws.
		Base:   lipgloss.NewStyle().Foreground(p.Text),
		Panel:  lipgloss.NewStyle().Foreground(p.Text).BorderForeground(p.Border),
		Text:   lipgloss.NewStyle().Foreground(p.Text),
		Muted:  lipgloss.NewStyle().Foreground(p.Muted),
		Focus:  lipgloss.NewStyle().Foreground(p.Focus).Bold(true),
		Signal: lipgloss.NewStyle().Foreground(p.Signal),
		Alert:  lipgloss.NewStyle().Foreground(p.Alert),
		Error:  lipgloss.NewStyle().Foreground(p.Error),
	}
}

var Default = New(DefaultPalette)
