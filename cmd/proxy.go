package cmd

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"

	tea "charm.land/bubbletea/v2"
	"github.com/spf13/cobra"
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
				if m.cursor >= m.viewport+m.height-5 {
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
				result := gjson.Get(string(get("proxies/"+url.PathEscape(m.items[m.cursor]))), "all")
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
				put("proxies/"+m.selected, jsonData)
			}

		case "backspace":
			if m.level == Providers {
				m.level = Proxies
				m.selected = ""
				result := gjson.Get(string(get("proxies")), "proxies")
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
	s += fmt.Sprintf("%d\n", m.level)
	v := tea.NewView(s)
	// v.AltScreen = true
	// Send the UI for rendering
	return v
}

func initialModel() model {
	proxies := []string{}
	result := gjson.Get(string(get("proxies")), "proxies")
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

func init() {
	proxyQueryCmd.Flags().StringP("name", "n", "", "name of the proxy")
	if err := proxyQueryCmd.MarkFlagRequired("name"); err != nil {
		panic(err)
	}

	proxySetCmd.Flags().StringP("name", "n", "", "name of the proxy")

	if err := proxySetCmd.MarkFlagRequired("name"); err != nil {
		panic(err)
	}
	proxySetCmd.Flags().StringP("provider", "p", "", "name of the provider")
	if err := proxySetCmd.MarkFlagRequired("provider"); err != nil {
		panic(err)
	}

	proxyCmd.AddCommand(proxyListCmd, proxyQueryCmd, proxySetCmd)
	rootCmd.AddCommand(proxyCmd)
}

var proxyCmd = &cobra.Command{
	Use:   "proxy",
	Short: "Hello from yact proxy",
	Long:  `There will be proxies..`,
	RunE: func(cmd *cobra.Command, args []string) error {
		p := tea.NewProgram(initialModel())
		if _, err := p.Run(); err != nil {
			return err
		}
		return nil
	},
}

var proxyListCmd = &cobra.Command{
	Use:   "list",
	Short: "Hello from yact proxy",
	Long:  `There will be proxies..`,
	RunE: func(cmd *cobra.Command, args []string) error {
		raw, err := cmd.Flags().GetBool("raw")
		if err != nil {
			return err
		}
		body := get("proxies")
		if !raw {
			result := gjson.Get(string(body), "proxies")
			result.ForEach(func(key, value gjson.Result) bool {
				proxyType := gjson.Get(value.Raw, "type").Str
				if proxyType == "Selector" {
					fmt.Println(key.Str + "->" + gjson.Get(value.Raw, "now").Str)
				}
				return true // keep iterating
			})
		} else {
			fmt.Println(string(body))
		}
		return nil
	},
}

var proxyQueryCmd = &cobra.Command{
	Use:   "query",
	Short: "Hello from yact proxy",
	Long:  `There will be proxies..`,
	RunE: func(cmd *cobra.Command, args []string) error {
		client := &http.Client{
			Timeout: 0,
		}
		raw, err := cmd.Flags().GetBool("raw")
		if err != nil {
			return err
		}
		name, err := cmd.Flags().GetString("name")
		if err != nil {
			return err
		}
		urlString := "http://localhost:9097/proxies/" + url.PathEscape(name)
		req, _ := http.NewRequest(http.MethodGet, urlString, nil)
		req.Header.Add("Authorization", "Bearer 123456")
		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Req failed", err)
			return err
		}
		defer resp.Body.Close()
		body, _ := io.ReadAll(resp.Body)
		if !raw {
			allProviders := gjson.Get(string(body), "all")
			allProviders.ForEach(func(key, value gjson.Result) bool {
				fmt.Println(value.Str)
				return true
			})
			provider := gjson.Get(string(body), "now")
			fmt.Println("Now: " + provider.Str)

		} else {
			fmt.Println(string(body))
		}
		return nil
	},
}

var proxySetCmd = &cobra.Command{
	Use:   "set",
	Short: "Hello from yact proxy",
	Long:  `There will be proxies..`,
	RunE: func(cmd *cobra.Command, args []string) error {
		client := &http.Client{
			Timeout: 0,
		}
		raw, err := cmd.Flags().GetBool("raw")
		if err != nil {
			return err
		}
		name, err := cmd.Flags().GetString("name")
		if err != nil {
			return err
		}
		provider, err := cmd.Flags().GetString("provider")
		if err != nil {
			return err
		}
		selection := map[string]string{"name": provider}
		jsonData, _ := json.Marshal(selection)
		urlString := "http://localhost:9097/proxies/" + url.PathEscape(name)
		req, _ := http.NewRequest(http.MethodPut, urlString, bytes.NewBuffer(jsonData))
		req.Header.Add("Authorization", "Bearer 123456")
		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Req failed", err)
			return err
		}
		defer resp.Body.Close()
		body, _ := io.ReadAll(resp.Body)
		if !raw {
			fmt.Println(resp.Status)
		} else {
			fmt.Println(string(body))
		}
		return nil
	},
}
