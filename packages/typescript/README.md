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

## Signal WebSocket

The gateway speaks Socket.IO (namespace `/ws/signals` on `https://mcp.echozero.app`) and authenticates with your developer API key. It accepts structured envelopes (`eventType`) and the legacy `{ action, tokenAddress, amount }` shape; natural-language `text` signals are HTTP webhook only. Responses are not correlated to requests, so match them with `idempotencyKey` / `signalId`.

```ts
import { EchoZeroSignalClient } from '@echozero/sdk';

const signals = new EchoZeroSignalClient({ apiKey: process.env.ECHOZERO_API_KEY! });
signals.onReceived((event) => console.log('accepted', event.signalId, event.status));
signals.onSignalError((event) => console.error('rejected', event.message));

await signals.connect(); // resolves on `authenticated`, rejects on a bad key

signals.sendSignal(process.env.ECHOZERO_AGENT_ID!, {
  eventType: 'buy',
  chain: 'solana',
  symbol: 'SOL',
  side: 'long',
  tradeType: 'spot',
  amount: 100,
  idempotencyKey: 'entry-001',
  reasoning: 'Breakout above range high',
});
```

`socket.io-client` handles reconnects and re-sends the API key on each reconnect.
