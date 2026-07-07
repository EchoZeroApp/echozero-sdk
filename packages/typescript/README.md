# @echozero/sdk

MIT-licensed TypeScript SDK for EchoZero.

Documentation: https://docs.echozero.app

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
  hmacSecretKey: process.env.ECHOZERO_SECRET_KEY,
});

const profile = await client.get('/api/v1/users/me');
```

## HMAC Request Signing

```ts
await client.post('/api/api-keys', { name: 'Signed request' }, { hmac: true });
```

Pass `hmacSecretKey` in the client options to enable HMAC signing.

## Inbound Agent Signal

```ts
const result = await client.postAgentSignal(
  process.env.ECHOZERO_AGENT_ID!,
  { text: 'BUY SOL $100' },
  process.env.EZW_SECRET!,
);
```

Or sign manually:

```ts
import { signInboundWebhook } from '@echozero/sdk';

const body = { text: 'BUY SOL 500 USDC' };
const headers = await signInboundWebhook({
  signingSecret: process.env.ECHOZERO_WEBHOOK_SECRET!,
  body,
});
```

## Outbound Execution Webhooks

```ts
import { verifyOutboundWebhook } from '@echozero/sdk';

const ok = await verifyOutboundWebhook({
  secretKey: process.env.WEBHOOK_SECRET!,
  rawBody,
  timestamp: req.headers['x-echozero-timestamp'],
  signature: req.headers['x-echozero-signature'],
});
```
