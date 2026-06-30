export type SignalClientOptions = {
  url: string;
  apiKey?: string;
  bearerToken?: string;
  WebSocketImpl?: typeof WebSocket;
};

export type SignalEventHandler = (event: MessageEvent) => void;

export class EchoZeroSignalClient {
  private socket?: WebSocket;
  private readonly WebSocketImpl: typeof WebSocket;

  constructor(private readonly options: SignalClientOptions) {
    this.WebSocketImpl = options.WebSocketImpl ?? WebSocket;
  }

  connect(onMessage: SignalEventHandler): WebSocket {
    const url = new URL(this.options.url);
    if (this.options.apiKey) url.searchParams.set('api_key', this.options.apiKey);
    if (this.options.bearerToken) {
      url.searchParams.set('access_token', this.options.bearerToken);
    }

    this.socket = new this.WebSocketImpl(url.toString());
    this.socket.addEventListener('message', onMessage);
    return this.socket;
  }

  sendSignal(signal: unknown): void {
    if (!this.socket || this.socket.readyState !== this.WebSocketImpl.OPEN) {
      throw new Error('Signal WebSocket is not open');
    }
    this.socket.send(JSON.stringify({ event: 'signal', data: signal }));
  }

  close(code?: number, reason?: string): void {
    this.socket?.close(code, reason);
    this.socket = undefined;
  }
}
