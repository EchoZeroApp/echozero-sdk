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
