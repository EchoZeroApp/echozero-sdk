export type HmacHeaders = {
  'x-timestamp': string;
  'x-signature': string;
};

export type InboundWebhookHeaders = {
  'X-EZ-Timestamp': string;
  'X-EZ-Signature': string;
};

const encoder = new TextEncoder();

function toHex(bytes: ArrayBuffer): string {
  return Array.from(new Uint8Array(bytes))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

async function hmacSha256Hex(secret: string, payload: string): Promise<string> {
  const key = await globalThis.crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  );
  const signature = await globalThis.crypto.subtle.sign(
    'HMAC',
    key,
    encoder.encode(payload),
  );
  return toHex(signature);
}

export function stableJson(value: unknown): string {
  if (value === null || typeof value !== 'object') {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableJson(item)).join(',')}]`;
  }

  const record = value as Record<string, unknown>;
  return `{${Object.keys(record)
    .sort()
    .filter((key) => record[key] !== undefined)
    .map((key) => `${JSON.stringify(key)}:${stableJson(record[key])}`)
    .join(',')}}`;
}

export async function signRestRequest(input: {
  secretKey: string;
  method: string;
  path: string;
  body?: unknown;
  timestampMs?: number;
}): Promise<HmacHeaders> {
  const timestamp = String(input.timestampMs ?? Date.now());
  const body =
    input.body === undefined || input.body === null
      ? ''
      : typeof input.body === 'string'
        ? input.body
        : JSON.stringify(input.body);
  const payload = `${timestamp}${input.method.toUpperCase()}${input.path}${body}`;
  return {
    'x-timestamp': timestamp,
    'x-signature': await hmacSha256Hex(input.secretKey, payload),
  };
}

export function canonicalWebhookBody(body: Record<string, unknown>): string {
  return stableJson(body);
}

export async function signInboundWebhook(input: {
  signingSecret: string;
  body: Record<string, unknown>;
  timestampSeconds?: number;
}): Promise<InboundWebhookHeaders> {
  const timestamp = String(input.timestampSeconds ?? Math.floor(Date.now() / 1000));
  const canonical = canonicalWebhookBody(input.body);
  return {
    'X-EZ-Timestamp': timestamp,
    'X-EZ-Signature': await hmacSha256Hex(
      input.signingSecret,
      `${timestamp}.${canonical}`,
    ),
  };
}

export async function verifyInboundWebhook(input: {
  signingSecret: string;
  body: Record<string, unknown>;
  timestampSeconds: string | number;
  signature: string;
  maxSkewSeconds?: number;
}): Promise<boolean> {
  const timestamp = Number(input.timestampSeconds);
  if (!Number.isFinite(timestamp)) return false;
  const maxSkew = input.maxSkewSeconds ?? 300;
  if (Math.abs(Math.floor(Date.now() / 1000) - timestamp) > maxSkew) {
    return false;
  }
  const expected = await signInboundWebhook({
    signingSecret: input.signingSecret,
    body: input.body,
    timestampSeconds: timestamp,
  });
  return expected['X-EZ-Signature'] === input.signature.toLowerCase();
}
