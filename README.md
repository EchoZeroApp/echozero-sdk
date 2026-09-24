# EchoZero SDK

Official EchoZero SDK workspace.

Documentation: https://docs.echozero.app

This repository is `EchoZeroApp/echozero-sdk` and ships under the MIT License. It contains public client SDKs for the EchoZero MCP/REST API surface:

- TypeScript package: `@echozero/sdk`
- Python package: `echozero`
- Rust crate: `echozero`
- Go module: `github.com/EchoZeroApp/echozero-sdk/packages/go`

## What Is Included

- Typed REST client primitives for EchoZero MCP REST endpoints.
- Optional HMAC request-signing helpers for `x-signature` / `x-timestamp`.
- Inbound agent webhook signing and verification (`X-EZ-Signature` / `X-EZ-Timestamp`).
- Outbound `signal.execution` webhook verification (`x-echozero-signature`).
- `postAgentSignal` / `post_agent_signal` helpers for developer agent ingress.
- Socket.IO signal client for the `/ws/signals` gateway (all four languages).
- Example projects for TypeScript, Python, Rust, and Go under `examples/`.

## Examples

```bash
# TypeScript
npx tsx examples/typescript/basic.ts

# Python
python examples/python/basic.py

# Go
cd examples/go && go run .

# Rust
cd examples/rust && cargo run

# Rust (signing only, no HTTP — useful when DNS/network is down)
cd examples/rust && ECHOZERO_OFFLINE=1 cargo run
```

Set `ECHOZERO_API_KEY`, `ECHOZERO_OAUTH_TOKEN`, or `ECHOZERO_WEBHOOK_SECRET` as needed.

If you see `failed to lookup address information` or DNS errors, check network access to `mcp.echozero.app`, or point at a local server with `ECHOZERO_BASE_URL=http://localhost:4010`.

## API Source

Endpoint-specific typed clients should be generated from the deployed MCP OpenAPI document when publishing:

```bash
curl -o openapi/echozero-mcp.openapi.json https://mcp.echozero.app/api/docs-json
```

The current checked-in SDK code is intentionally hand-written around stable cross-language concerns: authentication, HMAC signing, webhooks, and transport helpers. Generated endpoint coverage can be layered on top of these primitives without changing public auth/signing behavior.

## Package Status

Published:

- npm: [`@echozero/sdk`](https://www.npmjs.com/package/@echozero/sdk)
- PyPI: [`echozero`](https://pypi.org/project/echozero/)
- crates.io: [`echozero`](https://crates.io/crates/echozero)
- Go module: `github.com/EchoZeroApp/echozero-sdk/packages/go` (tags `packages/go/vX.Y.Z`)

Releases run from the manual `Release` workflow. Tag the Go module separately.

### 0.2.0

- **Breaking:** the signal WebSocket client now speaks Socket.IO to `/ws/signals` and authenticates with `auth.apiKey` / `x-api-key`. The 0.1.0 client opened a plain WebSocket with `?api_key=` and could not connect to the gateway. Constructors take the API key directly and `sendSignal` takes the developer agent id.
- Rust: the `websocket` feature now includes TLS, so `wss://` works.

## Basic TypeScript Usage

```ts
import { EchoZeroClient } from '@echozero/sdk';

const client = new EchoZeroClient({
  baseUrl: 'https://mcp.echozero.app',
  apiKey: process.env.ECHOZERO_API_KEY,
});

const metadata = await client.get('/public/index.json');
console.log(metadata.name);
```

## Basic Python Usage

```py
from echozero import EchoZeroClient

client = EchoZeroClient(
    base_url="https://mcp.echozero.app",
    api_key=os.environ["ECHOZERO_API_KEY"],
)

print(client.get("/public/index.json")["name"])
```

## Basic Rust Usage

```rust
let client = echozero::EchoZeroClient::new("https://mcp.echozero.app")
    .with_api_key(std::env::var("ECHOZERO_API_KEY")?);

let value = client.get_json("/public/index.json").await?;
```

## Basic Go Usage

```go
client := echozero.NewClient("https://mcp.echozero.app").
    WithAPIKey(os.Getenv("ECHOZERO_API_KEY"))

var metadata map[string]any
if err := client.Get(context.Background(), "/public/index.json", &metadata); err != nil {
    panic(err)
}
fmt.Println(metadata["name"])
```
