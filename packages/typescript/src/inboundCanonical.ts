/** Mirrors `agentInboundWebhook.signing.ts` in mcp-main. */

export type InboundWebhookBody = Record<string, unknown> & {
  text?: string;
  idempotencyKey?: string;
};

type CanonicalValue =
  | string
  | number
  | boolean
  | null
  | CanonicalValue[]
  | { [key: string]: CanonicalValue };

const STRUCTURED_SIGNING_KEYS = [
  'action',
  'amount',
  'chain',
  'changes',
  'clientTimestamp',
  'confidence',
  'context',
  'currentPnlPct',
  'entryPrice',
  'entryZone',
  'eventType',
  'exitPrice',
  'exitReason',
  'expiresAt',
  'ideaId',
  'instrument',
  'leverageX',
  'metadata',
  'orderType',
  'pnlPct',
  'positionRef',
  'reasoning',
  'relatedTradeId',
  'riskMgmt',
  'sellSizePct',
  'side',
  'symbol',
  'tokenAddress',
  'tradeType',
  'triggers',
  'urgency',
  'version',
] as const;

function canonicalizeValue(value: unknown): CanonicalValue | undefined {
  if (value === undefined) return undefined;
  if (value === null) return null;
  if (typeof value === 'string') return value;
  if (typeof value === 'number') return Number.isFinite(value) ? value : undefined;
  if (typeof value === 'boolean') return value;
  if (Array.isArray(value)) {
    return value
      .map((item) => canonicalizeValue(item))
      .filter((item): item is CanonicalValue => item !== undefined);
  }
  if (typeof value === 'object') {
    const out: { [key: string]: CanonicalValue } = {};
    for (const key of Object.keys(value as Record<string, unknown>).sort()) {
      const child = canonicalizeValue((value as Record<string, unknown>)[key]);
      if (child !== undefined) out[key] = child;
    }
    return out;
  }
  return undefined;
}

export function inboundWebhookCanonicalJson(body: InboundWebhookBody): string {
  const sorted: Record<string, CanonicalValue> = {};

  if (body.text !== undefined) {
    sorted.text = body.text;
  }

  const idk =
    typeof body.idempotencyKey === 'string' ? body.idempotencyKey.trim() : '';
  if (idk) {
    sorted.idempotencyKey = idk;
  }

  for (const key of STRUCTURED_SIGNING_KEYS) {
    const value = canonicalizeValue(body[key]);
    if (value !== undefined) {
      sorted[key] = value;
    }
  }

  const keys = Object.keys(sorted).sort();
  const obj: Record<string, CanonicalValue> = {};
  for (const k of keys) {
    obj[k] = sorted[k];
  }
  return JSON.stringify(obj);
}
