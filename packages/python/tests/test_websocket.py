import asyncio
import socket
import threading

import pytest
import socketio
from aiohttp import web

from echozero import SIGNAL_NAMESPACE, EchoZeroSignalClient


@pytest.fixture(scope="module")
def gateway_url():
    """Minimal stand-in for echozero-be TradeSignalGateway."""
    sio = socketio.AsyncServer(async_mode="aiohttp")

    @sio.on("connect", namespace=SIGNAL_NAMESPACE)
    async def connect(sid, _environ, auth):
        if (auth or {}).get("apiKey") != "ez_live_good":
            await sio.emit("error", {"message": "Invalid or expired API key"}, to=sid, namespace=SIGNAL_NAMESPACE)
            await sio.disconnect(sid, namespace=SIGNAL_NAMESPACE)
            return
        await sio.emit("authenticated", {"ok": True}, to=sid, namespace=SIGNAL_NAMESPACE)

    @sio.on("signal", namespace=SIGNAL_NAMESPACE)
    async def signal(sid, data):
        if not data.get("developerAgentId"):
            await sio.emit("signal:error", {"message": "Missing developerAgentId"}, to=sid, namespace=SIGNAL_NAMESPACE)
            return
        await sio.emit(
            "signal:received",
            {"signalId": f"sig-{data['idempotencyKey']}", "status": "accepted"},
            to=sid,
            namespace=SIGNAL_NAMESPACE,
        )

    app = web.Application()
    sio.attach(app)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]

    loop = asyncio.new_event_loop()
    runner = web.AppRunner(app)
    started = threading.Event()

    def serve():
        asyncio.set_event_loop(loop)
        loop.run_until_complete(runner.setup())
        loop.run_until_complete(web.TCPSite(runner, "127.0.0.1", port).start())
        started.set()
        loop.run_forever()

    threading.Thread(target=serve, daemon=True).start()
    started.wait(5)
    yield f"http://127.0.0.1:{port}"
    asyncio.run_coroutine_threadsafe(sio.shutdown(), loop).result(5)
    asyncio.run_coroutine_threadsafe(runner.cleanup(), loop).result(5)
    loop.call_soon_threadsafe(loop.stop)


def test_connect_send_and_receive(gateway_url):
    received = threading.Event()
    events = []

    def on_received(data):
        events.append(data)
        received.set()

    client = EchoZeroSignalClient(api_key="ez_live_good", base_url=gateway_url, on_received=on_received)
    client.connect(timeout=5)
    client.send_signal("agent-1", {"eventType": "buy", "idempotencyKey": "k1"})
    assert received.wait(5)
    assert events == [{"signalId": "sig-k1", "status": "accepted"}]
    client.close()


def test_invalid_key_raises(gateway_url):
    client = EchoZeroSignalClient(api_key="ez_live_bad", base_url=gateway_url)
    with pytest.raises(PermissionError, match="Invalid or expired API key"):
        client.connect(timeout=5)


def test_send_before_connect_raises():
    client = EchoZeroSignalClient(api_key="ez_live_good")
    with pytest.raises(RuntimeError, match="not connected"):
        client.send_signal("agent-1", {})
