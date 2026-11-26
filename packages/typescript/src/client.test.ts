import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { EchoZeroClient } from './client.js';

function capture() {
  const seen: Headers[] = [];
  const fetchImpl = (async (_url: unknown, init: RequestInit) => {
    seen.push(new Headers(init.headers));
    return new Response('{}', {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });
  }) as unknown as typeof fetch;
  return { seen, fetchImpl };
}

describe('EchoZeroClient auto-signing', () => {
  it('signs automatically when hmacSecretKey is set', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', hmacSecretKey: 's', fetchImpl });
    await client.get('/api/v1/users/me');
    assert.ok(seen[0].get('x-timestamp'));
    assert.match(seen[0].get('x-signature') ?? '', /^[0-9a-f]{64}$/);
  });

  it('does not sign without a secret', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', fetchImpl });
    await client.get('/api/v1/users/me');
    assert.equal(seen[0].get('x-signature'), null);
  });

  it('hmac: false opts out even with a secret', async () => {
    const { seen, fetchImpl } = capture();
    const client = new EchoZeroClient({ apiKey: 'k', hmacSecretKey: 's', fetchImpl });
    await client.get('/api/v1/users/me', { hmac: false });
    assert.equal(seen[0].get('x-signature'), null);
  });
});
