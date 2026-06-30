# echozero

MIT-licensed Rust SDK for EchoZero.

## Install

```toml
[dependencies]
echozero = "0.1"
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
