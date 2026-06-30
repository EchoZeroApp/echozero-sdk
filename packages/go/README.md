# echozero-go

MIT-licensed Go SDK for EchoZero.

## Install

```bash
go get github.com/EchoZeroApp/echozero-sdk/packages/go
```

## REST Client

```go
client := echozero.NewClient("https://mcp.echozero.app").
    WithAPIKey(os.Getenv("ECHOZERO_API_KEY"))

var metadata map[string]any
if err := client.Get(context.Background(), "/public/index.json", &metadata); err != nil {
    panic(err)
}
```

## Inbound Webhook Signing

```go
headers, err := echozero.SignInboundWebhook(
    os.Getenv("ECHOZERO_WEBHOOK_SECRET"),
    map[string]any{"text": "SDK health check message with no market instruction"},
    0,
)
if err != nil {
    panic(err)
}

fmt.Println(headers.Timestamp, headers.Signature != "")
```

## Signal WebSocket

```go
signalClient := echozero.NewSignalClient("wss://mcp.echozero.app/signals").
    WithAPIKey(os.Getenv("ECHOZERO_API_KEY"))

conn, _, err := signalClient.Connect(context.Background())
if err != nil {
    panic(err)
}
defer conn.Close()

if err := signalClient.SendSignal(map[string]any{"text": "BUY SOL 500 USDC"}); err != nil {
    panic(err)
}
```
