package main

import (
	"context"
	"fmt"
	"os"

	echozero "github.com/EchoZeroApp/echozero-sdk/packages/go/echozero"
)

func main() {
	baseURL := os.Getenv("ECHOZERO_BASE_URL")
	if baseURL == "" {
		baseURL = "https://mcp.echozero.app"
	}

	client := echozero.NewClient(baseURL).WithAPIKey(os.Getenv("ECHOZERO_API_KEY"))

	var metadata map[string]any
	if err := client.Get(context.Background(), "/public/index.json", &metadata); err != nil {
		panic(err)
	}
	fmt.Println("Connected to:", stringValue(metadata["name"], "EchoZero"))

	if os.Getenv("ECHOZERO_API_KEY") != "" {
		var apiKeys any
		if err := client.Get(context.Background(), "/api/api-keys", &apiKeys); err != nil {
			panic(err)
		}
		fmt.Println("Authenticated request OK:", valueTypeName(apiKeys))
	}

	if secret := os.Getenv("ECHOZERO_WEBHOOK_SECRET"); secret != "" {
		headers, err := echozero.SignInboundWebhook(
			secret,
			map[string]any{"text": "SDK health check message with no market instruction"},
			0,
		)
		if err != nil {
			panic(err)
		}
		fmt.Println("Webhook signature generated:", headers.Signature != "")
	}
}

func stringValue(value any, fallback string) string {
	if text, ok := value.(string); ok && text != "" {
		return text
	}
	return fallback
}

func valueTypeName(value any) string {
	switch value.(type) {
	case nil:
		return "null"
	case bool:
		return "bool"
	case float64:
		return "number"
	case string:
		return "string"
	case []any:
		return "array"
	case map[string]any:
		return "object"
	default:
		return "unknown"
	}
}
