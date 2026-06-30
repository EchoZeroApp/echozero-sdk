# EchoZero SDK

Official EchoZero SDK workspace.

This repository is intended to become `EchoZeroApp/echozero-sdk` and ships under the MIT License. It contains public client SDKs for the EchoZero MCP/REST API surface:

- TypeScript package: `@echozero/sdk`
- Python package: `echozero`
- Rust crate: `echozero`

## What Is Included

- Typed REST client primitives for EchoZero MCP REST endpoints.
- Optional HMAC request-signing helpers for `x-signature` / `x-timestamp`.
- Inbound agent webhook signing and verification helpers.
- WebSocket signal client helpers for developer signal streams.
- Example projects for TypeScript, Python, and Rust.

## API Source

Endpoint-specific typed clients should be generated from the deployed MCP OpenAPI document when publishing:

```bash
curl -o openapi/echozero-mcp.openapi.json https://mcp.echozero.app/api/docs-json
```

The current checked-in SDK code is intentionally hand-written around stable cross-language concerns: authentication, HMAC signing, webhooks, and transport helpers. Generated endpoint coverage can be layered on top of these primitives without changing public auth/signing behavior.

## Package Status

This is a source scaffold, not a published release. Before publishing:

1. Create or move this folder to `EchoZeroApp/echozero-sdk`.
2. Confirm `LICENSE` remains at repository root.
3. Generate endpoint-specific clients from `https://mcp.echozero.app/api/docs-json`.
4. Run each package's tests/build checks.
5. Publish:
   - npm: `@echozero/sdk`
   - PyPI: `echozero`
   - crates.io: `echozero`

## Basic TypeScript Usage

```ts
import { EchoZeroClient } from '@echozero/sdk';

const client = new EchoZeroClient({
  baseUrl: 'https://mcp.echozero.app',
  apiKey: process.env.ECHOZERO_API_KEY,
});

const me = await client.get('/api/v1/users/me');
console.log(me);
```

## Basic Python Usage

```py
from echozero import EchoZeroClient

client = EchoZeroClient(
    base_url="https://mcp.echozero.app",
    api_key=os.environ["ECHOZERO_API_KEY"],
)

print(client.get("/api/v1/users/me"))
```

## Basic Rust Usage

```rust
let client = echozero::EchoZeroClient::new("https://mcp.echozero.app")
    .with_api_key(std::env::var("ECHOZERO_API_KEY")?);

let value = client.get_json("/api/v1/users/me").await?;
```
