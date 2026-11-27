import json

import pytest

from echozero import EchoZeroApiError, EchoZeroClient


class FakeResponse:
    def __init__(self, status, payload):
        self.status_code = status
        self.ok = 200 <= status < 300
        self.reason = "Bad Request" if status == 400 else "OK"
        self.text = json.dumps(payload)
        self._payload = payload

    def json(self):
        return self._payload


class FakeSession:
    def __init__(self, status=200, payload=None):
        self.calls = []
        self.status = status
        self.payload = payload if payload is not None else {"success": True, "data": {"id": "a1"}}

    def request(self, method, url, data=None, headers=None):
        self.calls.append({"method": method, "url": url, "data": data, "headers": headers})
        return FakeResponse(self.status, self.payload)


def test_set_execution_callback_puts_url():
    session = FakeSession()
    client = EchoZeroClient(api_key="k", session=session)
    agent = client.set_execution_callback("a1", "https://hooks.example.com/ez")
    call = session.calls[0]
    assert agent == {"id": "a1"}
    assert call["method"] == "PUT"
    assert call["url"] == "https://mcp.echozero.app/api/v1/agents/a1/execution-callback"
    assert json.loads(call["data"]) == {"url": "https://hooks.example.com/ez"}
    assert call["headers"]["x-api-key"] == "k"
    assert "x-signature" not in call["headers"]


def test_set_execution_callback_signs_when_secret_set():
    session = FakeSession()
    client = EchoZeroClient(api_key="k", hmac_secret_key="s", session=session)
    client.set_execution_callback("a1", "https://hooks.example.com/ez")
    headers = session.calls[0]["headers"]
    assert len(headers["x-signature"]) == 64
    assert headers["x-timestamp"]


def test_clear_execution_callback_sends_delete():
    session = FakeSession()
    client = EchoZeroClient(api_key="k", session=session)
    client.clear_execution_callback("a/1")
    call = session.calls[0]
    assert call["method"] == "DELETE"
    assert call["data"] is None
    assert call["url"].endswith("/api/v1/agents/a%2F1/execution-callback")


def test_rejected_url_raises_api_error():
    session = FakeSession(
        400, {"success": False, "error": {"code": "BAD_REQUEST", "message": "Callback URL must use https"}}
    )
    client = EchoZeroClient(api_key="k", session=session)
    with pytest.raises(EchoZeroApiError) as err:
        client.set_execution_callback("a1", "http://hooks.example.com/ez")
    assert err.value.status == 400
    assert str(err.value) == "Callback URL must use https"
