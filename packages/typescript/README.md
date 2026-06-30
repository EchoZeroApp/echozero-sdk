# @echozero/sdk

MIT-licensed TypeScript SDK for EchoZero.

## Install

```bash
npm install @echozero/sdk
```

## REST Client

```ts
import { EchoZeroClient } from '@echozero/sdk';

const client = new EchoZeroClient({
  baseUrl: 'https://mcp.echozero.app',
  apiKey: process.env.ECHOZERO_API_KEY,
});

const profile = await client.get('/api/v1/users/me');
```

## HMAC Request Signing

```ts
await client.post('/api/api-keys', { name: 'Signed request' }, { hmac: true });
```

Pass `hmacSecretKey` in the client options to enable HMAC signing.

## Inbound Webhook Signing

```ts
import { signInboundWebhook } from '@echozero/sdk';

const body = { text: 'BUY SOL 500 USDC' };
const headers = await signInboundWebhook({
  signingSecret: process.env.ECHOZERO_WEBHOOK_SECRET!,
  body,
});
```
