package ui

import (
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type NavItem struct {
	ID, Label string
	Children  []NavItem
}

type navRow struct {
	item  NavItem
	depth int
}

type Navigation struct {
	Items    []NavItem
	Selected string
	Width    int
	Height   int
	expanded map[string]bool
	offset   int
}

func NewNavigation(items []NavItem) Navigation {
	n := Navigation{Items: items, Width: 24, Height: 20, expanded: make(map[string]bool)}
	if rows := n.rows(); len(rows) > 0 {
		n.Selected = rows[0].item.ID
	}
	return n
}

func (n *Navigation) SetSize(width int) { n.Width = max(1, width) }

func (n *Navigation) SetHeight(height int) {
	n.Height = max(1, height)
	n.ensure()
}

func (n *Navigation) ensure() {
	if n.expanded == nil {
		n.expanded = make(map[string]bool)
	}
	rows := n.rows()
	if len(rows) == 0 {
		n.Selected = ""
		n.offset = 0
		return
	}
	for _, row := range rows {
		if row.item.ID == n.Selected {
			n.reconcileOffset(rows)
			return
		}
	}
	n.Selected = rows[0].item.ID
	n.reconcileOffset(rows)
}

func (n *Navigation) reconcileOffset(rows []navRow) {
	selected := 0
	for index := range rows {
		if rows[index].item.ID == n.Selected {
			selected = index
			break
		}
	}
	n.offset = clamp(n.offset, 0, max(0, len(rows)-n.Height))
	if selected < n.offset {
		n.offset = selected
	}
	if selected >= n.offset+n.Height {
		n.offset = selected - n.Height + 1
	}
}

func (n Navigation) rows() []navRow {
	var rows []navRow
	var walk func([]NavItem, int)
	walk = func(items []NavItem, depth int) {
		for _, item := range items {
			rows = append(rows, navRow{item: item, depth: depth})
			if n.expanded[item.ID] {
				walk(item.Children, depth+1)
			}
		}
	}
	walk(n.Items, 0)
	return rows
}

func (n Navigation) SelectedItem() (NavItem, bool) {
	for _, row := range n.rows() {
		if row.item.ID == n.Selected {
			return row.item, true
		}
	}
	return NavItem{}, false
}

// Reveal expands ancestor groups and selects the item so nested pages become
// visible after command-palette navigation.
func (n *Navigation) Reveal(id string) bool {
	n.ensure()
	ancestors, ok := findNavAncestors(n.Items, id, nil)
	if !ok {
		return false
	}
	clear(n.expanded)
	for _, ancestor := range ancestors {
		n.expanded[ancestor] = true
	}
	n.Selected = id
	n.ensure()
	return true
}

func findNavAncestors(items []NavItem, id string, ancestors []string) ([]string, bool) {
	for _, item := range items {
		if item.ID == id {
			return ancestors, true
		}
		next := append(append([]string{}, ancestors...), item.ID)
		if found, ok := findNavAncestors(item.Children, id, next); ok {
			return found, true
		}
	}
	return nil, false
}

func (n Navigation) Init() tea.Cmd { return nil }

func (n Navigation) Update(msg tea.Msg) (Navigation, tea.Cmd) {
	n.ensure()
	key, ok := msg.(tea.KeyMsg)
	if !ok {
		return n, nil
	}
	rows := n.rows()
	index := 0
	for i := range rows {
		if rows[i].item.ID == n.Selected {
			index = i
			break
		}
	}
	switch key.String() {
	case "up", "k":
		index = max(0, index-1)
		n.Selected = rows[index].item.ID
	case "down", "j":
		index = min(len(rows)-1, index+1)
		n.Selected = rows[index].item.ID
	case "right", "l", "enter", " ":
		if len(rows[index].item.Children) > 0 {
			if !n.expanded[n.Selected] {
				clear(n.expanded)
				n.expanded[n.Selected] = true
			}
		}
	case "left", "h":
		if n.expanded[n.Selected] {
			n.expanded[n.Selected] = false
		} else if rows[index].depth > 0 {
			targetDepth := rows[index].depth - 1
			for i := index - 1; i >= 0; i-- {
				if rows[i].depth == targetDepth {
					n.Selected = rows[i].item.ID
					break
				}
			}
		}
	case "home", "g":
		n.Selected = rows[0].item.ID
	case "end", "G":
		n.Selected = rows[len(rows)-1].item.ID
	}
	n.ensure()
	return n, nil
}

func (n Navigation) View() string {
	if n.Width <= 0 {
		return ""
	}
	rows := n.rows()
	start := clamp(n.offset, 0, len(rows))
	end := min(len(rows), start+max(1, n.Height))
	lines := make([]string, 0, end-start)
	for _, row := range rows[start:end] {
		prefix := "  "
		if len(row.item.Children) > 0 {
			prefix = "▸ "
			if n.expanded[row.item.ID] {
				prefix = "▾ "
			}
		}
		line := strings.Repeat("  ", row.depth) + prefix + row.item.Label
		line = truncate(line, n.Width)
		if row.item.ID == n.Selected {
			line = theme.Default.Focus.Render(line)
		} else {
			line = theme.Default.Text.Render(line)
		}
		lines = append(lines, line)
	}
	return strings.Join(lines, "\n")
}
