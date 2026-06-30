from __future__ import annotations

import json
from typing import Any, Callable
from urllib.parse import urlencode

from websocket import WebSocketApp


class EchoZeroSignalClient:
    def __init__(
        self,
        *,
        url: str,
        api_key: str | None = None,
        bearer_token: str | None = None,
    ):
        params = {}
        if api_key:
            params["api_key"] = api_key
        if bearer_token:
            params["access_token"] = bearer_token
        query = urlencode(params)
        self.url = f"{url}?{query}" if query else url
        self.socket: WebSocketApp | None = None

    def connect(
        self,
        on_message: Callable[[Any], None],
        on_error: Callable[[Exception], None] | None = None,
    ) -> WebSocketApp:
        def handle_message(_: WebSocketApp, message: str) -> None:
            try:
                on_message(json.loads(message))
            except json.JSONDecodeError:
                on_message(message)

        self.socket = WebSocketApp(
            self.url,
            on_message=handle_message,
            on_error=lambda _, err: on_error(err) if on_error else None,
        )
        return self.socket

    def send_signal(self, signal: Any) -> None:
        if not self.socket:
            raise RuntimeError("Signal WebSocket is not connected")
        self.socket.send(json.dumps({"event": "signal", "data": signal}))
