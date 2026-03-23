package tui

import (
	"encoding/json"
	"fmt"
	"net/url"
	"yact/internal/mihomo"

	tea "charm.land/bubbletea/v2"
	"github.com/tidwall/gjson"
)

const (
	Proxies = iota
	Providers
)

type model struct {
	items    []string
	level    int
	cursor   int
	height   int
	viewport int
	selected string
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	// Is it a key press?
	case tea.KeyPressMsg:

		// Cool, what was the actual key pressed?
		switch msg.String() {

		// These keys should exit the program.
		case "ctrl+c", "q":
			return m, tea.Quit

		// The "up" and "k" keys move the cursor up
		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
				m.viewport = min(m.cursor, m.viewport)
			}

		// The "down" and "j" keys move the cursor down
		case "down", "j":
			if m.cursor < len(m.items)-2 {
				m.cursor++
				if m.cursor >= m.viewport+m.height-3 {
					m.viewport++
				}
			}

		// The "enter" key and the space bar toggle the selected state
		// for the item that the cursor is pointing at.
		case "enter", "space":
			if m.level == Proxies {
				m.level = Providers
				m.selected = m.items[m.cursor]
				// Update items to providers
				result := gjson.Get(string(mihomo.Get("proxies/"+url.PathEscape(m.items[m.cursor]))), "all")
				m.items = []string{}
				result.ForEach(func(key, value gjson.Result) bool {
					m.items = append(m.items, fmt.Sprint(value.Str))
					return true // keep iterating
				})
				m.cursor = 0
				m.viewport = 0
			} else {
				data := map[string]string{"name": m.items[m.cursor]}
				jsonData, _ := json.Marshal(data)
				mihomo.Put("proxies/"+m.selected, jsonData)
			}

		case "backspace":
			if m.level == Providers {
				m.level = Proxies
				m.selected = ""
				result := gjson.Get(string(mihomo.Get("proxies")), "proxies")
				m.items = []string{}
				result.ForEach(func(key, value gjson.Result) bool {
					proxyType := gjson.Get(value.Raw, "type").Str
					if proxyType == "Selector" {
						m.items = append(m.items, fmt.Sprint(key.Str))
					}
					return true // keep iterating
				})
				m.cursor = 0
				m.viewport = 0
			}
		}
	case tea.WindowSizeMsg:
		m.height = msg.Height

	}

	// Return the updated model to the Bubble Tea runtime for processing.
	// Note that we're not returning a command.
	return m, nil
}

func (m model) View() tea.View {
	// The header
	s := ""
	end := min(len(m.items)-1, m.viewport+m.height-3)
	// Iterate over our choices
	for i, choice := range m.items[m.viewport:end] {

		// Is the cursor pointing at this choice?
		cursor := " " // no cursor
		if m.cursor == m.viewport+i {
			cursor = ">" // cursor!
		}
		// Render the row
		s += fmt.Sprintf("%s %s\n", cursor, choice)
	}
	// The footer
	v := tea.NewView(s)
	// v.AltScreen = true
	// Send the UI for rendering
	return v
}

func initialModel() model {
	proxies := []string{}
	result := gjson.Get(string(mihomo.Get("proxies")), "proxies")
	result.ForEach(func(key, value gjson.Result) bool {
		proxyType := gjson.Get(value.Raw, "type").Str
		if proxyType == "Selector" {
			proxies = append(proxies, fmt.Sprint(key.Str))
		}
		return true // keep iterating
	})
	return model{
		items:    proxies,
		level:    Proxies,
		cursor:   0,
		viewport: 0,
		height:   14,
	}
}

func (m model) Init() tea.Cmd {
	return nil
}

func Run() error {
	p := tea.NewProgram(initialModel())
	if _, err := p.Run(); err != nil {
		return err
	}
	return nil
}
