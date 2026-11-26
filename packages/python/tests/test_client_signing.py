from echozero import EchoZeroClient


class _Resp:
    ok = True
    status_code = 200
    reason = "OK"
    headers = {"content-type": "application/json"}
    content = b"{}"
    text = "{}"

    def json(self):
        return {}


class _Session:
    def __init__(self):
        self.headers = []

    def request(self, method, url, data=None, headers=None):
        self.headers.append(dict(headers or {}))
        return _Resp()


def _client(**kwargs):
    client = EchoZeroClient(api_key="k", **kwargs)
    client.session = _Session()
    return client


def test_signs_automatically_when_secret_set():
    client = _client(hmac_secret_key="s")
    client.get("/api/v1/users/me")
    sent = client.session.headers[0]
    assert sent["x-timestamp"]
    assert len(sent["x-signature"]) == 64


def test_no_signature_without_secret():
    client = _client()
    client.get("/api/v1/users/me")
    assert "x-signature" not in client.session.headers[0]


def test_hmac_false_opts_out():
    client = _client(hmac_secret_key="s")
    client.get("/api/v1/users/me", hmac=False)
    assert "x-signature" not in client.session.headers[0]
