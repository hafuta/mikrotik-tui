package ui

import (
	"fmt"

	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type Inspector struct {
	Title, Content string
	Width, Height  int
	viewport       viewport.Model
}

func NewInspector(title, content string) Inspector {
	model := Inspector{
		Title:    title,
		Content:  content,
		Width:    40,
		Height:   12,
		viewport: viewport.New(40, 11),
	}
	model.viewport.SetContent(content)
	return model
}

func (i *Inspector) SetSize(width, height int) {
	i.Width, i.Height = max(1, width), max(2, height)
	i.viewport.Width = i.Width
	i.viewport.Height = max(1, i.Height-1)
	i.viewport.SetYOffset(i.viewport.YOffset)
}

func (i *Inspector) SetContent(content string) {
	i.Content = content
	i.viewport.SetContent(content)
	i.viewport.GotoTop()
}

// SetContentPreservingOffset updates live data without snapping an inspector
// the user is reading back to the first line.
func (i *Inspector) SetContentPreservingOffset(content string) {
	offset := i.viewport.YOffset
	i.Content = content
	i.viewport.SetContent(content)
	i.viewport.SetYOffset(offset)
}

func (i Inspector) Init() tea.Cmd { return nil }

func (i Inspector) Update(msg tea.Msg) (Inspector, tea.Cmd) {
	if key, ok := msg.(tea.KeyMsg); ok {
		switch key.String() {
		case "home", "g":
			i.viewport.GotoTop()
			return i, nil
		case "end", "G":
			i.viewport.GotoBottom()
			return i, nil
		}
	}
	var command tea.Cmd
	i.viewport, command = i.viewport.Update(msg)
	return i, command
}

func (i Inspector) View() string {
	if i.Width <= 0 || i.Height <= 0 {
		return ""
	}
	title := fitCell(i.Title, i.Width)
	if i.viewport.TotalLineCount() > i.viewport.Height {
		progress := fmt.Sprintf("line %d/%d", i.viewport.YOffset+1, i.viewport.TotalLineCount())
		if i.viewport.YOffset > 0 {
			progress = "↑ " + progress
		}
		if i.viewport.YOffset+i.viewport.Height < i.viewport.TotalLineCount() {
			progress += " ↓"
		}
		title = fitCell(i.Title+"  "+progress, i.Width)
	}
	return theme.Default.Focus.Render(title) + "\n" + theme.Default.Text.Render(i.viewport.View())
}
