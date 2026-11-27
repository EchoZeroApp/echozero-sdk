import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { EchoZeroApiError, EchoZeroClient } from './client.js';

type Seen = { url: string; method?: string; body?: string; headers: Headers };

function capture(status = 200, body: unknown = { success: true, data: { id: 'a1' } }) {
  const seen: Seen[] = [];
  const fetchImpl = (async (url: string, init: RequestInit) => {
    seen.push({
      url,
      method: init.method,
      body: init.body as string | undefined,
      headers: new Headers(init.headers),
    });
    return new Response(JSON.stringify(body), {
      status,
      headers: { 'content-type': 'application/json' },
    });
  }) as unknown as typeof fetch;
  return { seen, fetchImpl };
}

describe('execution callback', () => {
  it('setExecutionCallback PUTs the url and returns the agent', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', fetchImpl });
    const agent = await client.setExecutionCallback('a1', 'https://hooks.example.com/ez');
    assert.deepEqual(agent, { id: 'a1' });
    assert.equal(seen[0].method, 'PUT');
    assert.equal(
      seen[0].url,
      'https://mcp.echozero.app/api/v1/agents/a1/execution-callback',
    );
    assert.deepEqual(JSON.parse(seen[0].body ?? ''), { url: 'https://hooks.example.com/ez' });
    assert.equal(seen[0].headers.get('x-api-key'), 'k');
    assert.equal(seen[0].headers.get('x-signature'), null);
  });

  it('signs the request when hmacSecretKey is set', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', hmacSecretKey: 's', fetchImpl });
    await client.setExecutionCallback('a1', 'https://hooks.example.com/ez');
    assert.match(seen[0].headers.get('x-signature') ?? '', /^[0-9a-f]{64}$/);
    assert.ok(seen[0].headers.get('x-timestamp'));
  });

  it('clearExecutionCallback sends DELETE with no body', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', fetchImpl });
    await client.clearExecutionCallback('a/1');
    assert.equal(seen[0].method, 'DELETE');
    assert.equal(seen[0].body, undefined);
    assert.ok(seen[0].url.endsWith('/api/v1/agents/a%2F1/execution-callback'));
  });

  it('surfaces a 400 for a rejected url', async () => {
    const { fetchImpl } = capture(400, {
      success: false,
      error: { code: 'BAD_REQUEST', message: 'Callback URL must use https' },
    });
    const client = new EchoZeroClient({ apiKey: 'k', fetchImpl });
    await assert.rejects(
      client.setExecutionCallback('a1', 'http://hooks.example.com/ez'),
      (err: unknown) =>
        err instanceof EchoZeroApiError &&
        err.status === 400 &&
        err.message === 'Callback URL must use https',
    );
  });
});
