import { EchoZeroClient, signInboundWebhook } from '../../packages/typescript/src/index.js';

const client = new EchoZeroClient({
  baseUrl: process.env.ECHOZERO_BASE_URL ?? 'https://mcp.echozero.app',
  apiKey: process.env.ECHOZERO_API_KEY,
  bearerToken: process.env.ECHOZERO_OAUTH_TOKEN,
});

const metadata = await client.get<{ name?: string }>('/public/index.json');
console.log('Connected to:', metadata.name ?? 'EchoZero');

if (process.env.ECHOZERO_API_KEY || process.env.ECHOZERO_OAUTH_TOKEN) {
  const apiKeys = await client.get('/api/api-keys');
  console.log('Authenticated request OK:', Array.isArray(apiKeys) ? 'array' : typeof apiKeys);
}

if (process.env.ECHOZERO_WEBHOOK_SECRET) {
  const body = { text: 'SDK health check message with no market instruction' };
  const headers = await signInboundWebhook({
    signingSecret: process.env.ECHOZERO_WEBHOOK_SECRET,
    body,
  });
  console.log('Webhook signature generated:', Boolean(headers['X-EZ-Signature']));
}
