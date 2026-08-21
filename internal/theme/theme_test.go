package theme

import (
	"strings"
	"testing"

	"github.com/charmbracelet/lipgloss"
)

func TestDefaultPaletteTokens(t *testing.T) {
	tests := map[string]string{
		"void":   string(DefaultPalette.Void),
		"panel":  string(DefaultPalette.Panel),
		"text":   string(DefaultPalette.Text),
		"focus":  string(DefaultPalette.Focus),
		"signal": string(DefaultPalette.Signal),
		"alert":  string(DefaultPalette.Alert),
		"muted":  string(DefaultPalette.Muted),
		"border": string(DefaultPalette.Border),
		"error":  string(DefaultPalette.Error),
	}
	for name, value := range tests {
		if len(value) != 7 || value[0] != '#' {
			t.Fatalf("%s token is not a hex color: %q", name, value)
		}
	}
}

func TestDefaultStylesNeverPaintTerminalBackgrounds(t *testing.T) {
	for name, style := range map[string]lipgloss.Style{
		"base":   Default.Base,
		"panel":  Default.Panel,
		"text":   Default.Text,
		"muted":  Default.Muted,
		"focus":  Default.Focus,
		"signal": Default.Signal,
		"alert":  Default.Alert,
		"error":  Default.Error,
	} {
		rendered := style.Render("paint probe   ")
		if strings.Contains(rendered, "[48;") {
			t.Fatalf("%s style emits an ANSI background sequence: %q", name, rendered)
		}
	}
}
