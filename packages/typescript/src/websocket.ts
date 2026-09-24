import { io, type ManagerOptions, type Socket, type SocketOptions } from 'socket.io-client';
import type { InboundSignalBody } from './signals.js';

/** Socket.IO namespace of the developer signal gateway. */
export const SIGNAL_NAMESPACE = '/ws/signals';

export type SignalClientOptions = {
  /** Developer API key (`ez_live_...`). Sent as the Socket.IO `auth.apiKey`. */
  apiKey: string;
  /** Defaults to `https://mcp.echozero.app`. */
  baseUrl?: string;
  /** Extra options passed straight to `socket.io-client`. */
  socketOptions?: Partial<ManagerOptions & SocketOptions>;
};

/** Payload of the `signal:received` event. */
export type SignalReceived = {
  signalId: string;
  status: string;
  executionResult?: {
    success: boolean;
    txHash?: string;
    executedAmountUsd?: number;
    errorMessage?: string;
  };
};

/** Payload of the `signal:error` and `error` events. */
export type SignalGatewayError = { message: string };

/**
 * Socket.IO client for `wss://mcp.echozero.app` namespace `/ws/signals`.
 *
 * The WebSocket gateway accepts structured envelopes (`eventType`) and the
 * legacy `{ action, tokenAddress, amount }` shape. Natural-language `text`
 * signals are only accepted by the HTTP inbound webhook.
 *
 * Responses are not correlated to requests: use `idempotencyKey` and the
 * returned `signalId` to match them.
 */
export class EchoZeroSignalClient {
  readonly socket: Socket;

  constructor(options: SignalClientOptions) {
    const baseUrl = (options.baseUrl ?? 'https://mcp.echozero.app').replace(/\/+$/, '');
    this.socket = io(`${baseUrl}${SIGNAL_NAMESPACE}`, {
      transports: ['websocket'],
      autoConnect: false,
      ...options.socketOptions,
      auth: { apiKey: options.apiKey },
    });
  }

  /** Connects and resolves once the gateway emits `authenticated`. */
  connect(timeoutMs = 10_000): Promise<void> {
    return new Promise((resolve, reject) => {
      const cleanup = () => {
        clearTimeout(timer);
        this.socket.off('authenticated', onAuthenticated);
        this.socket.off('error', onError);
        this.socket.off('connect_error', onError);
      };
      const onAuthenticated = () => {
        cleanup();
        resolve();
      };
      const onError = (error: SignalGatewayError | Error) => {
        cleanup();
        this.socket.disconnect();
        reject(new Error(`EchoZero signal gateway: ${error.message}`));
      };
      const timer = setTimeout(
        () => onError(new Error('timed out waiting for authentication')),
        timeoutMs,
      );
      this.socket.on('authenticated', onAuthenticated);
      this.socket.on('error', onError);
      this.socket.on('connect_error', onError);
      this.socket.connect();
    });
  }

  /** Emits a `signal` event for one of your developer agents. */
  sendSignal(developerAgentId: string, signal: InboundSignalBody): void {
    if (!this.socket.connected) {
      throw new Error('Signal WebSocket is not connected');
    }
    this.socket.emit('signal', { ...signal, developerAgentId });
  }

  /** Subscribes to `signal:received`. Returns an unsubscribe function. */
  onReceived(handler: (event: SignalReceived) => void): () => void {
    this.socket.on('signal:received', handler);
    return () => this.socket.off('signal:received', handler);
  }

  /** Subscribes to `signal:error`. Returns an unsubscribe function. */
  onSignalError(handler: (event: SignalGatewayError) => void): () => void {
    this.socket.on('signal:error', handler);
    return () => this.socket.off('signal:error', handler);
  }

  close(): void {
    this.socket.disconnect();
  }
}
