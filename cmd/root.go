package cmd

import (
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "yact",
	Short: "Hello from yact",
	Long:  `There will be proxies..`,
	RunE: func(cmd *cobra.Command, args []string) error {
		return nil
	},
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func init() {
	rootCmd.PersistentFlags().BoolP("raw", "r", false, "output raw json")
}

var providersCmd = &cobra.Command{
	Use:   "providers",
	Short: "Hello from yact proxy",
	Long:  `There will be proxies..`,
	Run: func(cmd *cobra.Command, args []string) {
		client := &http.Client{
			Timeout: 0,
		}

		providerStr := "🔰 选择节点"
		providerPath := url.PathEscape(providerStr)
		url := "http://localhost:9097/providers/proxies/" + providerPath
		req, _ := http.NewRequest("GET", url, nil)
		req.Header.Add("Authorization", "Bearer 123456")

		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Req failed", err)
			return
		}
		defer resp.Body.Close()
		body, _ := io.ReadAll(resp.Body)
		fmt.Println(string(body))
	},
}

var groupCommand = &cobra.Command{
	Use:   "group",
	Short: "Hello from yact config",
	Long:  `There will be configs..`,
	Run: func(cmd *cobra.Command, args []string) {
		client := &http.Client{
			Timeout: 0,
		}
		req, _ := http.NewRequest("GET", "http://localhost:9097/group", nil)
		req.Header.Add("Authorization", "Bearer 123456")

		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Req failed", err)
			return
		}
		defer resp.Body.Close()
		body, _ := io.ReadAll(resp.Body)
		fmt.Println(string(body))
	},
}

var ruleCommand = &cobra.Command{
	Use:   "rule",
	Short: "Hello from yact rule",
	Long:  `There will be configs..`,
	Run: func(cmd *cobra.Command, args []string) {
		client := &http.Client{
			Timeout: 0,
		}
		req, _ := http.NewRequest("GET", "http://localhost:9097/providers/rules", nil)
		req.Header.Add("Authorization", "Bearer 123456")

		resp, err := client.Do(req)
		if err != nil {
			fmt.Println("Req failed", err)
			return
		}
		defer resp.Body.Close()
		body, _ := io.ReadAll(resp.Body)
		fmt.Println(string(body))
	},
}
