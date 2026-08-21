package ui

import (
	"strings"
	"unicode"
	"unicode/utf8"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/hafuta/mikrotik-tui/internal/theme"
)

type Credentials struct {
	Address, Username, Password string
}

type loginField struct {
	value  string
	cursor int
}

// LoginInput is a self-contained, tab-navigable login form.
type LoginInput struct {
	fields [3]loginField
	focus  int
	Width  int
}

func NewLoginInput() LoginInput {
	return LoginInput{Width: 48}
}

func (m *LoginInput) SetSize(width int) {
	m.Width = max(20, width)
}

func (m LoginInput) FocusIndex() int { return m.focus }

func (m LoginInput) Values() Credentials {
	return Credentials{
		Address:  strings.TrimSpace(m.fields[0].value),
		Username: strings.TrimSpace(m.fields[1].value),
		Password: m.fields[2].value,
	}
}

func (m *LoginInput) SetValues(credentials Credentials) {
	values := [...]string{credentials.Address, credentials.Username, credentials.Password}
	for index, value := range values {
		m.fields[index] = loginField{value: value, cursor: utf8.RuneCountInString(value)}
	}
}

func (m *LoginInput) setFocus(index int) {
	m.focus = (index + len(m.fields)) % len(m.fields)
}

func (m LoginInput) Init() tea.Cmd { return nil }

func (m LoginInput) Update(msg tea.Msg) (LoginInput, tea.Cmd) {
	if key, ok := msg.(tea.KeyMsg); ok {
		switch key.String() {
		case "tab", "down":
			m.setFocus(m.focus + 1)
			return m, nil
		case "shift+tab", "up":
			m.setFocus(m.focus - 1)
			return m, nil
		case "enter":
			if m.focus < len(m.fields)-1 {
				m.setFocus(m.focus + 1)
				return m, nil
			}
		case "left":
			m.fields[m.focus].cursor--
		case "right":
			m.fields[m.focus].cursor++
		case "home":
			m.fields[m.focus].cursor = 0
		case "end":
			m.fields[m.focus].cursor = utf8.RuneCountInString(m.fields[m.focus].value)
		case "backspace":
			m.deleteBeforeCursor()
		case "delete":
			m.deleteAtCursor()
		default:
			if key.Type == tea.KeyRunes {
				if value := printableInput(key.Runes); value != "" {
					m.insert(value)
				}
			}
		}
	}
	m.clampCursor()
	return m, nil
}

func printableInput(runes []rune) string {
	filtered := make([]rune, 0, len(runes))
	for _, value := range runes {
		// Some terminal backends report modifier-only keys such as Caps Lock
		// as a zero/control rune. They are key state, not text input.
		if unicode.IsPrint(value) && !unicode.IsControl(value) {
			filtered = append(filtered, value)
		}
	}
	return string(filtered)
}

func (m *LoginInput) clampCursor() {
	m.fields[m.focus].cursor = clamp(m.fields[m.focus].cursor, 0, utf8.RuneCountInString(m.fields[m.focus].value))
}

func (m *LoginInput) insert(value string) {
	field := &m.fields[m.focus]
	runes := []rune(field.value)
	field.cursor = clamp(field.cursor, 0, len(runes))
	field.value = string(runes[:field.cursor]) + value + string(runes[field.cursor:])
	field.cursor += utf8.RuneCountInString(value)
}

func (m *LoginInput) deleteBeforeCursor() {
	field := &m.fields[m.focus]
	runes := []rune(field.value)
	if field.cursor > 0 && field.cursor <= len(runes) {
		field.value = string(runes[:field.cursor-1]) + string(runes[field.cursor:])
		field.cursor--
	}
}

func (m *LoginInput) deleteAtCursor() {
	field := &m.fields[m.focus]
	runes := []rune(field.value)
	if field.cursor >= 0 && field.cursor < len(runes) {
		field.value = string(runes[:field.cursor]) + string(runes[field.cursor+1:])
	}
}

func (m LoginInput) View() string {
	labels := [...]string{"Router", "Username", "Password"}
	placeholders := [...]string{"https://192.168.88.1:8443", "admin", "password"}
	lines := []string{theme.Default.Focus.Render("Connect to RouterOS")}
	for index, field := range m.fields {
		label := theme.Default.Muted.Render(labels[index])
		if index == m.focus {
			label = theme.Default.Focus.Render(labels[index])
		}
		value := field.value
		if index == 2 {
			value = strings.Repeat("•", utf8.RuneCountInString(value))
		}
		if value == "" {
			value = theme.Default.Muted.Render(placeholders[index])
		}
		if index == m.focus {
			runes := []rune(value)
			cursor := clamp(field.cursor, 0, len(runes))
			value = string(runes[:cursor]) + theme.Default.Focus.Render("▏") + string(runes[cursor:])
		}
		lines = append(lines, label, fitCell(value, max(8, m.Width-4)))
	}
	lines = append(lines, theme.Default.Muted.Render("tab move  enter continue"))
	return strings.Join(lines, "\n")
}
