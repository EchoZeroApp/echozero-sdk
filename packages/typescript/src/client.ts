import { signInboundWebhook, signRestRequest } from './hmac.js';
import type { AgentSignalResponse, InboundSignalBody } from './signals.js';

export type EchoZeroClientOptions = {
  baseUrl?: string;
  apiKey?: string;
  bearerToken?: string;
  hmacSecretKey?: string;
  fetchImpl?: typeof fetch;
};

export type RequestOptions = Omit<RequestInit, 'body'> & {
  query?: Record<string, string | number | boolean | null | undefined>;
  json?: unknown;
  hmac?: boolean;
};

export class EchoZeroApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string,
    public readonly payload?: unknown,
  ) {
    super(message);
    this.name = 'EchoZeroApiError';
  }
}

export class EchoZeroClient {
  readonly baseUrl: string;
  private readonly apiKey?: string;
  private readonly bearerToken?: string;
  private readonly hmacSecretKey?: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: EchoZeroClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? 'https://mcp.echozero.app').replace(/\/+$/, '');
    this.apiKey = options.apiKey;
    this.bearerToken = options.bearerToken;
    this.hmacSecretKey = options.hmacSecretKey;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  get<T = unknown>(path: string, options: RequestOptions = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'GET' });
  }

  post<T = unknown>(
    path: string,
    json?: unknown,
    options: RequestOptions = {},
  ): Promise<T> {
    return this.request<T>(path, { ...options, method: 'POST', json });
  }

  patch<T = unknown>(
    path: string,
    json?: unknown,
    options: RequestOptions = {},
  ): Promise<T> {
    return this.request<T>(path, { ...options, method: 'PATCH', json });
  }

  delete<T = unknown>(path: string, options: RequestOptions = {}): Promise<T> {
    return this.request<T>(path, { ...options, method: 'DELETE' });
  }

  async postAgentSignal<T = AgentSignalResponse>(
    agentId: string,
    body: InboundSignalBody,
    signingSecret: string,
  ): Promise<T> {
    const webhookHeaders = await signInboundWebhook({ signingSecret, body });
    return this.post<T>(`/api/public/agent-signals/${agentId}`, body, {
      headers: {
        'X-EZ-Timestamp': webhookHeaders['X-EZ-Timestamp'],
        'X-EZ-Signature': webhookHeaders['X-EZ-Signature'],
      },
    });
  }

  async request<T = unknown>(
    path: string,
    options: RequestOptions = {},
  ): Promise<T> {
    const method = (options.method ?? 'GET').toUpperCase();
    const url = this.buildUrl(path, options.query);
    const headers = new Headers(options.headers);
    headers.set('Accept', 'application/json');

    let body: BodyInit | undefined;
    if (options.json !== undefined) {
      headers.set('Content-Type', 'application/json');
      body = JSON.stringify(options.json);
    }

    if (this.bearerToken) {
      headers.set('Authorization', `Bearer ${this.bearerToken}`);
    } else if (this.apiKey) {
      headers.set('x-api-key', this.apiKey);
    }

    if (options.hmac) {
      if (!this.hmacSecretKey) {
        throw new Error('hmacSecretKey is required when hmac=true');
      }
      const hmacHeaders = await signRestRequest({
        secretKey: this.hmacSecretKey,
        method,
        path: new URL(url).pathname + new URL(url).search,
        body: options.json,
      });
      headers.set('x-timestamp', hmacHeaders['x-timestamp']);
      headers.set('x-signature', hmacHeaders['x-signature']);
    }

    const response = await this.fetchImpl(url, {
      ...options,
      method,
      headers,
      body,
    });

    const text = await response.text();
    const payload = text ? safeJsonParse(text) : null;
    if (!response.ok) {
      const error = readApiError(payload);
      throw new EchoZeroApiError(
        error.message ?? response.statusText,
        response.status,
        error.code,
        payload,
      );
    }

    if (isEnvelope<T>(payload)) return payload.data;
    return payload as T;
  }

  buildUrl(
    path: string,
    query?: Record<string, string | number | boolean | null | undefined>,
  ): string {
    const url = new URL(path.startsWith('http') ? path : `${this.baseUrl}${path}`);
    Object.entries(query ?? {}).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== '') {
        url.searchParams.set(key, String(value));
      }
    });
    return url.toString();
  }
}

function safeJsonParse(value: string): unknown {
  try {
    return JSON.parse(value);
  } catch {
    return value;
  }
}

function isEnvelope<T>(value: unknown): value is { success: true; data: T } {
  return (
    typeof value === 'object' &&
    value !== null &&
    (value as { success?: unknown }).success === true &&
    'data' in value
  );
}

function readApiError(value: unknown): { code?: string; message?: string } {
  if (typeof value !== 'object' || value === null) return {};
  const error = (value as { error?: { code?: string; message?: string } }).error;
  return error ?? {};
}
