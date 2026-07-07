from __future__ import annotations

from typing import Any, Mapping

import requests

from .hmac import sign_inbound_webhook, sign_rest_request
from .inbound_canonical import rest_request_body_text


class EchoZeroApiError(RuntimeError):
    def __init__(self, message: str, status: int, code: str | None = None, payload: Any = None):
        super().__init__(message)
        self.status = status
        self.code = code
        self.payload = payload


class EchoZeroClient:
    def __init__(
        self,
        *,
        base_url: str = "https://mcp.echozero.app",
        api_key: str | None = None,
        bearer_token: str | None = None,
        hmac_secret_key: str | None = None,
        session: requests.Session | None = None,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.bearer_token = bearer_token
        self.hmac_secret_key = hmac_secret_key
        self.session = session or requests.Session()

    def get(self, path: str, **kwargs: Any) -> Any:
        return self.request("GET", path, **kwargs)

    def post(self, path: str, json_body: Any | None = None, **kwargs: Any) -> Any:
        return self.request("POST", path, json_body=json_body, **kwargs)

    def patch(self, path: str, json_body: Any | None = None, **kwargs: Any) -> Any:
        return self.request("PATCH", path, json_body=json_body, **kwargs)

    def delete(self, path: str, **kwargs: Any) -> Any:
        return self.request("DELETE", path, **kwargs)

    def post_agent_signal(
        self,
        agent_id: str,
        body: Mapping[str, Any],
        signing_secret: str,
    ) -> Any:
        webhook_headers = sign_inbound_webhook(signing_secret=signing_secret, body=body)
        return self.post(
            f"/api/public/agent-signals/{agent_id}",
            json_body=dict(body),
            headers={
                "X-EZ-Timestamp": webhook_headers["X-EZ-Timestamp"],
                "X-EZ-Signature": webhook_headers["X-EZ-Signature"],
            },
        )

    def request(
        self,
        method: str,
        path: str,
        *,
        query: Mapping[str, Any] | None = None,
        json_body: Any | None = None,
        headers: Mapping[str, str] | None = None,
        hmac: bool = False,
    ) -> Any:
        url = self._url(path, query)
        request_headers = {"Accept": "application/json", **(headers or {})}
        if self.bearer_token:
            request_headers["Authorization"] = f"Bearer {self.bearer_token}"
        elif self.api_key:
            request_headers["x-api-key"] = self.api_key

        body_bytes: bytes | None = None
        body_text = ""
        if json_body is not None:
            body_text = rest_request_body_text(json_body)
            body_bytes = body_text.encode("utf-8")
            request_headers["Content-Type"] = "application/json"

        if hmac:
            if not self.hmac_secret_key:
                raise ValueError("hmac_secret_key is required when hmac=True")
            request_headers.update(
                sign_rest_request(
                    secret_key=self.hmac_secret_key,
                    method=method,
                    path=self._path_with_query(path, query),
                    body=body_text if json_body is not None else None,
                )
            )

        response = self.session.request(
            method.upper(),
            url,
            data=body_bytes,
            headers=request_headers,
        )
        payload = self._read_json(response)
        if not response.ok:
            error = payload.get("error", {}) if isinstance(payload, dict) else {}
            raise EchoZeroApiError(
                error.get("message") or response.reason,
                response.status_code,
                error.get("code"),
                payload,
            )
        if isinstance(payload, dict) and payload.get("success") is True and "data" in payload:
            return payload["data"]
        return payload

    def _url(self, path: str, query: Mapping[str, Any] | None = None) -> str:
        if path.startswith("http://") or path.startswith("https://"):
            base = path
        else:
            base = f"{self.base_url}{path if path.startswith('/') else f'/{path}'}"
        from urllib.parse import urlencode

        query_string = urlencode({k: v for k, v in (query or {}).items() if v is not None})
        return f"{base}?{query_string}" if query_string else base

    def _path_with_query(self, path: str, query: Mapping[str, Any] | None = None) -> str:
        if path.startswith("http://") or path.startswith("https://"):
            from urllib.parse import urlparse

            parsed = urlparse(path)
            path = parsed.path + (f"?{parsed.query}" if parsed.query else "")
        from urllib.parse import urlencode

        query_string = urlencode({k: v for k, v in (query or {}).items() if v is not None})
        return f"{path}?{query_string}" if query_string else path

    @staticmethod
    def _read_json(response: requests.Response) -> Any:
        if not response.text:
            return None
        try:
            return response.json()
        except ValueError:
            return response.text
