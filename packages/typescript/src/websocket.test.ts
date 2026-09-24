import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import type { AddressInfo } from 'node:net';
import { after, before, test } from 'node:test';
import { Server } from 'socket.io';
import { EchoZeroSignalClient, SIGNAL_NAMESPACE, type SignalReceived } from './index.js';

// Minimal stand-in for echozero-be TradeSignalGateway.
const httpServer = createServer();
const io = new Server(httpServer);
let baseUrl = '';

before(async () => {
  io.of(SIGNAL_NAMESPACE).on('connection', (socket) => {
    if (socket.handshake.auth?.apiKey !== 'ez_live_good') {
      socket.emit('error', { message: 'Invalid or expired API key' });
      socket.disconnect(true);
      return;
    }
    socket.emit('authenticated', { ok: true });
    socket.on('signal', (data: Record<string, unknown>) => {
      if (!data.developerAgentId) {
        socket.emit('signal:error', { message: 'Missing developerAgentId' });
        return;
      }
      socket.emit('signal:received', {
        signalId: `sig-${data.idempotencyKey}`,
        status: 'accepted',
      });
    });
  });
  await new Promise<void>((resolve) => httpServer.listen(0, resolve));
  baseUrl = `http://127.0.0.1:${(httpServer.address() as AddressInfo).port}`;
});

after(() => {
  io.close();
});

test('connects, sends a signal and receives signal:received', async () => {
  const client = new EchoZeroSignalClient({ apiKey: 'ez_live_good', baseUrl });
  await client.connect();
  const received = new Promise<SignalReceived>((resolve) => client.onReceived(resolve));
  client.sendSignal('agent-1', {
    eventType: 'buy',
    action: 'buy',
    idempotencyKey: 'k1',
  });
  assert.deepEqual(await received, { signalId: 'sig-k1', status: 'accepted' });
  client.close();
});

test('rejects connect when the API key is invalid', async () => {
  const client = new EchoZeroSignalClient({ apiKey: 'ez_live_bad', baseUrl });
  await assert.rejects(client.connect(), /Invalid or expired API key/);
});

test('sendSignal throws before connect', () => {
  const client = new EchoZeroSignalClient({ apiKey: 'ez_live_good', baseUrl });
  assert.throws(() => client.sendSignal('agent-1', {}), /not connected/);
});
