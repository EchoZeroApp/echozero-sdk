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

The gateway speaks Socket.IO (namespace `/ws/signals` on `https://mcp.echozero.app`) and authenticates with your developer API key. It accepts structured envelopes (`eventType`) and the legacy `{ action, tokenAddress, amount }` shape; natural-language `text` signals are HTTP webhook only. Responses are not correlated to requests, so match them with `idempotencyKey` / `signalId`.

```go
signals := echozero.NewSignalClient(os.Getenv("ECHOZERO_API_KEY"))

ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
defer cancel()
if err := signals.Connect(ctx); err != nil { // returns once `authenticated`
    panic(err)
}
defer signals.Close()

err := signals.SendSignal(os.Getenv("ECHOZERO_AGENT_ID"), map[string]any{
    "eventType":      "buy",
    "chain":          "solana",
    "symbol":         "SOL",
    "side":           "long",
    "tradeType":      "spot",
    "amount":         100,
    "idempotencyKey": "entry-001",
    "reasoning":      "Breakout above range high",
})
if err != nil {
    panic(err)
}

for event := range signals.Events() { // "signal:received", "signal:error", then "disconnect"
    fmt.Println(event.Name, string(event.Data))
}
```

Drain `Events()`: the client answers gateway pings from the same loop. There is no automatic reconnect; call `Connect` again and re-send unacknowledged signals with the same `idempotencyKey`.
