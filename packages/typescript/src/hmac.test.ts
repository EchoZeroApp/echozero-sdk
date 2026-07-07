import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { inboundWebhookCanonicalJson } from './inboundCanonical.js';
import {
  signInboundWebhook,
  signRestRequest,
  verifyInboundWebhook,
  verifyOutboundWebhook,
} from './hmac.js';

describe('signRestRequest', () => {
  it('matches backend REST HMAC vector', async () => {
    const headers = await signRestRequest({
      secretKey: 'test_secret',
      method: 'POST',
      path: '/api/api-keys',
      body: { name: 'SDK HMAC Test' },
      timestampMs: 1_710_000_000_000,
    });
    assert.equal(
      headers['x-signature'],
      '274c9ff280eadf751530e9e7fce2c2a573d8676b13b7108fe353407df7cc9e00',
    );
  });
});

describe('inboundWebhookCanonicalJson', () => {
  it('strips unknown top-level fields', () => {
    assert.equal(
      inboundWebhookCanonicalJson({ text: 'BUY SOL', extraField: 'ignored' }),
      '{"text":"BUY SOL"}',
    );
  });

  it('matches backend structured buy canonical', () => {
    const body = {
      eventType: 'buy',
      idempotencyKey: 'test-1',
      reasoning: 'test',
      tokenAddress: 'So11111111111111111111111111111111111111112',
      amount: 500,
      unknownField: 'should be stripped',
    };
    assert.equal(
      inboundWebhookCanonicalJson(body),
      '{"amount":500,"eventType":"buy","idempotencyKey":"test-1","reasoning":"test","tokenAddress":"So11111111111111111111111111111111111111112"}',
    );
  });
});

describe('signInboundWebhook', () => {
  it('matches backend inbound vector', async () => {
    const headers = await signInboundWebhook({
      signingSecret: 'test_secret',
      body: {
        text: 'BUY SOL 500 USDC',
        idempotencyKey: 'sdk-test-1',
      },
      timestampSeconds: 1_710_000_000,
    });
    assert.equal(
      headers['X-EZ-Signature'],
      '1d30d896fc609e62bcf9be991c1dc1177a9c63219909869ef2846bad4685de3b',
    );
  });

  it('ignores unknown fields when signing', async () => {
    const body = {
      eventType: 'buy',
      idempotencyKey: 'test-1',
      reasoning: 'test',
      tokenAddress: 'So11111111111111111111111111111111111111112',
      amount: 500,
    };
    const withExtra = { ...body, unknownField: 'strip me' };
    const a = await signInboundWebhook({
      signingSecret: 'secret',
      body,
      timestampSeconds: 1_710_000_000,
    });
    const b = await signInboundWebhook({
      signingSecret: 'secret',
      body: withExtra,
      timestampSeconds: 1_710_000_000,
    });
    assert.equal(a['X-EZ-Signature'], b['X-EZ-Signature']);
    assert.equal(
      a['X-EZ-Signature'],
      'be420f61d91e6b871481774c62f972f0aafa6f5f1a727ba1e4a32558784f77c3',
    );
  });
});

describe('verifyInboundWebhook', () => {
  it('accepts valid signatures', async () => {
    const body = { text: 'BUY SOL 500 USDC', idempotencyKey: 'sdk-test-1' };
    const headers = await signInboundWebhook({
      signingSecret: 'test_secret',
      body,
      timestampSeconds: 1_710_000_000,
    });
    const ok = await verifyInboundWebhook({
      signingSecret: 'test_secret',
      body,
      timestampSeconds: headers['X-EZ-Timestamp'],
      signature: headers['X-EZ-Signature'],
      maxSkewSeconds: 999_999_999,
    });
    assert.equal(ok, true);
  });
});

describe('verifyOutboundWebhook', () => {
  it('verifies signal.execution callbacks', async () => {
    const rawBody = JSON.stringify({
      event: 'signal.execution',
      signalId: 'sig_1',
      developerAgentId: 'agent_1',
      status: 'executed',
      timestamp: '2026-07-06T12:00:00.000Z',
    });
    const timestamp = '2026-07-06T12:00:00.000Z';
    const { createHmac } = await import('node:crypto');
    const signature = createHmac('sha256', 'webhook_secret')
      .update(`${timestamp}.${rawBody}`)
      .digest('hex');

    const ok = await verifyOutboundWebhook({
      secretKey: 'webhook_secret',
      rawBody,
      timestamp,
      signature,
    });
    assert.equal(ok, true);
  });
});
