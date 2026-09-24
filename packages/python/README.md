# echozero

MIT-licensed Python SDK for EchoZero.

## Install

```bash
pip install echozero
```

## REST Client

```py
import os
from echozero import EchoZeroClient

client = EchoZeroClient(
    base_url="https://mcp.echozero.app",
    api_key=os.environ["ECHOZERO_API_KEY"],
)

print(client.get("/api/v1/users/me"))
```

## Inbound Webhook Signing

```py
from echozero import sign_inbound_webhook

body = {"text": "BUY SOL 500 USDC"}
headers = sign_inbound_webhook(
    signing_secret="ezw_secret",
    body=body,
)
```

## Signal WebSocket

The gateway speaks Socket.IO (namespace `/ws/signals` on `https://mcp.echozero.app`) and authenticates with your developer API key. It accepts structured envelopes (`eventType`) and the legacy `{ action, tokenAddress, amount }` shape; natural-language `text` signals are HTTP webhook only. Responses are not correlated to requests, so match them with `idempotencyKey` / `signalId`.

```py
import os
from echozero import EchoZeroSignalClient

signals = EchoZeroSignalClient(
    api_key=os.environ["ECHOZERO_API_KEY"],
    on_received=lambda event: print("accepted", event["signalId"], event["status"]),
    on_signal_error=lambda event: print("rejected", event["message"]),
)
signals.connect()  # blocks until `authenticated`, raises PermissionError on a bad key

signals.send_signal(os.environ["ECHOZERO_AGENT_ID"], {
    "eventType": "buy",
    "chain": "solana",
    "symbol": "SOL",
    "side": "long",
    "tradeType": "spot",
    "amount": 100,
    "idempotencyKey": "entry-001",
    "reasoning": "Breakout above range high",
})
signals.wait()
```
