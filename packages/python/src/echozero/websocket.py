from __future__ import annotations

import threading
from typing import Any, Callable, Mapping

import socketio

SIGNAL_NAMESPACE = "/ws/signals"


class EchoZeroSignalClient:
    """Socket.IO client for the developer signal gateway (namespace ``/ws/signals``).

    The WebSocket gateway accepts structured envelopes (``eventType``) and the
    legacy ``{action, tokenAddress, amount}`` shape. Natural-language ``text``
    signals are only accepted by the HTTP inbound webhook.

    Responses are not correlated to requests: use ``idempotencyKey`` and the
    returned ``signalId`` to match them.
    """

    def __init__(
        self,
        *,
        api_key: str,
        base_url: str = "https://mcp.echozero.app",
        on_received: Callable[[dict[str, Any]], None] | None = None,
        on_signal_error: Callable[[dict[str, Any]], None] | None = None,
        sio: socketio.Client | None = None,
    ):
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.sio = sio or socketio.Client()
        self._authenticated = threading.Event()
        self._auth_error: str | None = None

        @self.sio.on("authenticated", namespace=SIGNAL_NAMESPACE)
        def _on_authenticated(_: Any = None) -> None:
            self._authenticated.set()

        @self.sio.on("error", namespace=SIGNAL_NAMESPACE)
        def _on_error(data: Any = None) -> None:
            self._auth_error = _message(data)
            self._authenticated.set()

        if on_received:
            self.sio.on("signal:received", on_received, namespace=SIGNAL_NAMESPACE)
        if on_signal_error:
            self.sio.on("signal:error", on_signal_error, namespace=SIGNAL_NAMESPACE)

    def connect(self, timeout: float = 10.0) -> None:
        """Connects and blocks until the gateway emits ``authenticated``."""
        self._authenticated.clear()
        self._auth_error = None
        self.sio.connect(
            self.base_url,
            namespaces=[SIGNAL_NAMESPACE],
            auth={"apiKey": self.api_key},
            transports=["websocket"],
            wait_timeout=timeout,
        )
        if not self._authenticated.wait(timeout):
            self.close()
            raise TimeoutError("EchoZero signal gateway: timed out waiting for authentication")
        if self._auth_error:
            self.close()
            raise PermissionError(f"EchoZero signal gateway: {self._auth_error}")

    def send_signal(self, developer_agent_id: str, signal: Mapping[str, Any]) -> None:
        """Emits a ``signal`` event for one of your developer agents."""
        if not self.sio.connected:
            raise RuntimeError("Signal WebSocket is not connected")
        self.sio.emit(
            "signal",
            {**signal, "developerAgentId": developer_agent_id},
            namespace=SIGNAL_NAMESPACE,
        )

    def wait(self) -> None:
        """Blocks until the connection closes."""
        self.sio.wait()

    def close(self) -> None:
        if self.sio.connected:
            self.sio.disconnect()


def _message(data: Any) -> str:
    if isinstance(data, Mapping):
        return str(data.get("message", data))
    return str(data)
