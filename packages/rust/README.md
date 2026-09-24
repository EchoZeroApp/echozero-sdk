# echozero

MIT-licensed Rust SDK for EchoZero.

## Install

```toml
[dependencies]
echozero = "0.2"
```

## REST Client

```rust
let client = echozero::EchoZeroClient::new("https://mcp.echozero.app")
    .with_api_key(std::env::var("ECHOZERO_API_KEY")?);

let me = client.get_json("/api/v1/users/me").await?;
```

## Inbound Webhook Signing

```rust
let mut body = serde_json::Map::new();
body.insert("text".into(), serde_json::json!("BUY SOL 500 USDC"));
let (timestamp, signature) =
    echozero::sign_inbound_webhook("ezw_secret", &body, None);
```

## Signal WebSocket

Enable the `websocket` feature:

```toml
[dependencies]
echozero = { version = "0.2", features = ["websocket"] }
```

The gateway speaks Socket.IO (namespace `/ws/signals` on `https://mcp.echozero.app`) and authenticates with your developer API key. It accepts structured envelopes (`eventType`) and the legacy `{ action, tokenAddress, amount }` shape; natural-language `text` signals are HTTP webhook only. Responses are not correlated to requests, so match them with `idempotencyKey` / `signalId`.

```rust
use echozero::websocket::SignalClient;
use serde_json::json;

let mut signals = SignalClient::connect("https://mcp.echozero.app", &api_key).await?;

let signal = json!({
    "eventType": "buy",
    "chain": "solana",
    "symbol": "SOL",
    "side": "long",
    "tradeType": "spot",
    "amount": 100,
    "idempotencyKey": "entry-001",
    "reasoning": "Breakout above range high",
});
signals.send_signal(&agent_id, signal.as_object().unwrap().clone())?;

while let Some(event) = signals.next_event().await {
    println!("{} {}", event.name, event.data); // signal:received, signal:error, disconnect
}
```

There is no automatic reconnect; connect again and re-send unacknowledged signals with the same `idempotencyKey`. See `examples/signal_stream.rs`.
