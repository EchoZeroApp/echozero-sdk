import { EchoZeroClient, signInboundWebhook } from '../../packages/typescript/src/index.js';

const client = new EchoZeroClient({
  baseUrl: process.env.ECHOZERO_BASE_URL ?? 'https://mcp.echozero.app',
  apiKey: process.env.ECHOZERO_API_KEY,
  bearerToken: process.env.ECHOZERO_OAUTH_TOKEN,
});

const me = await client.get('/api/v1/users/me');
console.log(me);

if (process.env.ECHOZERO_WEBHOOK_SECRET) {
  const body = { text: 'BUY SOL 500 USDC' };
  const headers = await signInboundWebhook({
    signingSecret: process.env.ECHOZERO_WEBHOOK_SECRET,
    body,
  });
  console.log(headers);
}
