package cmd

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
)

type Core struct {
	address string
	secret  string
}

func get(path string) []byte {

	c := Core{
		address: "http://localhost:9097/",
		secret:  "123456",
	}
	client := &http.Client{
		Timeout: 0,
	}
	url := c.address + path
	req, _ := http.NewRequest(http.MethodGet, url, nil)
	req.Header.Add("Authorization", "Bearer "+c.secret)
	resp, err := client.Do(req)
	if err != nil {
		fmt.Println("Req failed", err)
		panic(err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	return body
}

func put(path string, data []byte) string {

	c := Core{
		address: "http://localhost:9097/",
		secret:  "123456",
	}
	client := &http.Client{
		Timeout: 0,
	}
	url := c.address + path
	req, _ := http.NewRequest(http.MethodPut, url, bytes.NewBuffer(data))
	req.Header.Add("Authorization", "Bearer "+c.secret)
	resp, err := client.Do(req)
	if err != nil {
		fmt.Println("Req failed", err)
		panic(err)
	}
	defer resp.Body.Close()
	return resp.Status
}
