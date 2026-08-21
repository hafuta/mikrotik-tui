package ui

import (
	"sort"
	"strconv"
	"strings"
	"unicode/utf8"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type Column struct {
	Key, Title string
	Width      int
}

type ResourceRow struct {
	ID    string
	Cells map[string]string
}

type SortDirection int8

const (
	SortNone SortDirection = iota
	SortAscending
	SortDescending
)

type ResourceTable struct {
	Columns        []Column
	Rows           []ResourceRow
	Width, Height  int
	Filter         string
	Filtering      bool
	SortColumn     string
	SortDirection  SortDirection
	SelectedID     string
	verticalOffset int
	horizontal     int
}

func NewResourceTable(columns []Column, rows []ResourceRow) ResourceTable {
	t := ResourceTable{Columns: columns, Rows: rows, Width: 80, Height: 12}
	t.reconcile()
	return t
}

func (t *ResourceTable) SetSize(width, height int) {
	t.Width, t.Height = max(1, width), max(2, height)
	t.reconcile()
}

func (t *ResourceTable) SetRows(rows []ResourceRow) {
	t.Rows = rows
	t.reconcile()
}

func (t *ResourceTable) SetFilter(filter string) {
	t.Filter = filter
	t.reconcile()
}

func (t *ResourceTable) SetSort(column string, direction SortDirection) {
	t.SortColumn, t.SortDirection = column, direction
	t.reconcile()
}

func (t ResourceTable) VisibleRows() []ResourceRow {
	query := strings.ToLower(strings.TrimSpace(t.Filter))
	rows := make([]ResourceRow, 0, len(t.Rows))
	for _, row := range t.Rows {
		if query == "" || rowMatches(row, t.Columns, query) {
			rows = append(rows, row)
		}
	}
	if t.SortDirection != SortNone && t.SortColumn != "" {
		sort.SliceStable(rows, func(i, j int) bool {
			a, b := rows[i].Cells[t.SortColumn], rows[j].Cells[t.SortColumn]
			if an, errA := strconv.ParseFloat(a, 64); errA == nil {
				if bn, errB := strconv.ParseFloat(b, 64); errB == nil && an != bn {
					if t.SortDirection == SortAscending {
						return an < bn
					}
					return an > bn
				}
			}
			cmp := strings.Compare(strings.ToLower(a), strings.ToLower(b))
			if cmp == 0 {
				cmp = strings.Compare(rows[i].ID, rows[j].ID)
			}
			if t.SortDirection == SortAscending {
				return cmp < 0
			}
			return cmp > 0
		})
	}
	return rows
}

func rowMatches(row ResourceRow, columns []Column, query string) bool {
	for _, column := range columns {
		if strings.Contains(strings.ToLower(row.Cells[column.Key]), query) {
			return true
		}
	}
	return false
}

func (t *ResourceTable) reconcile() {
	rows := t.VisibleRows()
	if len(rows) == 0 {
		t.SelectedID, t.verticalOffset = "", 0
		return
	}
	index := -1
	for i := range rows {
		if rows[i].ID == t.SelectedID {
			index = i
			break
		}
	}
	if index < 0 {
		index, t.SelectedID = 0, rows[0].ID
	}
	page := max(1, t.Height-2)
	t.verticalOffset = clamp(t.verticalOffset, 0, max(0, len(rows)-page))
	if index < t.verticalOffset {
		t.verticalOffset = index
	}
	if index >= t.verticalOffset+page {
		t.verticalOffset = index - page + 1
	}
	t.horizontal = clamp(t.horizontal, 0, max(0, t.contentWidth()-t.Width))
}

func (t ResourceTable) selectedIndex(rows []ResourceRow) int {
	for i := range rows {
		if rows[i].ID == t.SelectedID {
			return i
		}
	}
	return 0
}

func (t ResourceTable) Init() tea.Cmd { return nil }

func (t ResourceTable) Update(msg tea.Msg) (ResourceTable, tea.Cmd) {
	key, ok := msg.(tea.KeyMsg)
	if !ok {
		return t, nil
	}
	if t.Filtering {
		switch key.Type {
		case tea.KeyEscape:
			t.Filtering = false
		case tea.KeyEnter:
			t.Filtering = false
		case tea.KeyBackspace:
			if len(t.Filter) > 0 {
				_, size := utf8.DecodeLastRuneInString(t.Filter)
				t.Filter = t.Filter[:len(t.Filter)-size]
			}
		case tea.KeyRunes:
			t.Filter += string(key.Runes)
		}
		t.reconcile()
		return t, nil
	}
	rows := t.VisibleRows()
	index := t.selectedIndex(rows)
	page := max(1, t.Height-2)
	switch key.String() {
	case "/":
		t.Filtering = true
	case "up", "k":
		index--
	case "down", "j":
		index++
	case "pgup", "ctrl+u":
		index -= page
	case "pgdown", "ctrl+d":
		index += page
	case "home", "g":
		index = 0
	case "end", "G":
		index = len(rows) - 1
	case "left", "h":
		t.horizontal -= 4
	case "right", "l":
		t.horizontal += 4
	case "ctrl+left":
		t.horizontal = 0
	case "ctrl+right":
		t.horizontal = t.contentWidth()
	}
	if len(rows) > 0 {
		index = clamp(index, 0, len(rows)-1)
		t.SelectedID = rows[index].ID
	}
	t.reconcile()
	return t, nil
}

func (t ResourceTable) contentWidth() int {
	width := 0
	for _, column := range t.Columns {
		width += max(1, column.Width) + 1
	}
	return max(0, width-1)
}

func (t ResourceTable) rawRow(row ResourceRow) string {
	cells := make([]string, len(t.Columns))
	for i, column := range t.Columns {
		value := row.Cells[column.Key]
		cells[i] = fitCell(value, max(1, column.Width))
	}
	return strings.Join(cells, " ")
}

func (t ResourceTable) View() string {
	if t.Width <= 0 || t.Height <= 0 {
		return ""
	}
	headerRow := ResourceRow{Cells: make(map[string]string, len(t.Columns))}
	for _, column := range t.Columns {
		title := column.Title
		if column.Key == t.SortColumn {
			if t.SortDirection == SortAscending {
				title += " ↑"
			} else if t.SortDirection == SortDescending {
				title += " ↓"
			}
		}
		headerRow.Cells[column.Key] = title
	}
	lines := []string{theme.Default.Muted.Bold(true).Render(t.window(t.rawRow(headerRow)))}
	if t.Filtering || t.Filter != "" {
		cursor := ""
		if t.Filtering {
			cursor = "▏"
		}
		lines = append(lines, truncate(theme.Default.Focus.Render("/ "+t.Filter+cursor), t.Width))
	}
	rows := t.VisibleRows()
	start := clamp(t.verticalOffset, 0, len(rows))
	available := max(0, t.Height-len(lines))
	end := min(len(rows), start+available)
	for _, row := range rows[start:end] {
		line := t.window(t.rawRow(row))
		if row.ID == t.SelectedID {
			line = theme.Default.Focus.Render("> " + fitCell(line, max(1, t.Width-2)))
		} else {
			line = "  " + fitCell(line, max(1, t.Width-2))
		}
		lines = append(lines, line)
	}
	if len(rows) == 0 {
		lines = append(lines, theme.Default.Muted.Render("No matching resources"))
	}
	return strings.Join(lines, "\n")
}

func (t ResourceTable) window(value string) string {
	runes := []rune(value)
	start := clamp(t.horizontal, 0, len(runes))
	end := min(len(runes), start+t.Width)
	return fitCell(string(runes[start:end]), t.Width)
}

func fitCell(value string, width int) string {
	runes := []rune(strings.ReplaceAll(value, "\n", " "))
	if len(runes) > width {
		if width == 1 {
			return "…"
		}
		return string(runes[:width-1]) + "…"
	}
	return string(runes) + strings.Repeat(" ", width-len(runes))
}
