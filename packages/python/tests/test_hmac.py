import json

import pytest

from echozero.hmac import (
    sign_inbound_webhook,
    sign_rest_request,
    verify_inbound_webhook,
    verify_outbound_webhook,
)
from echozero.inbound_canonical import inbound_webhook_canonical_json, rest_request_body_text


def test_sign_rest_request_matches_backend_vector():
    headers = sign_rest_request(
        secret_key="test_secret",
        method="POST",
        path="/api/api-keys",
        body={"name": "SDK HMAC Test"},
        timestamp_ms=1_710_000_000_000,
    )
    assert (
        headers["x-signature"]
        == "274c9ff280eadf751530e9e7fce2c2a573d8676b13b7108fe353407df7cc9e00"
    )


def test_rest_request_body_text_matches_signed_bytes():
    body = {"zzz": "y", "name": "x"}
    body_text = rest_request_body_text(body)
    assert body_text == json.dumps(body, separators=(",", ":"), ensure_ascii=False)
    headers_a = sign_rest_request(
        secret_key="secret",
        method="POST",
        path="/api/api-keys",
        body=body_text,
        timestamp_ms=123,
    )
    headers_b = sign_rest_request(
        secret_key="secret",
        method="POST",
        path="/api/api-keys",
        body=body,
        timestamp_ms=123,
    )
    assert headers_a == headers_b


def test_inbound_canonical_strips_unknown_fields():
    assert (
        inbound_webhook_canonical_json({"text": "BUY SOL", "extraField": "ignored"})
        == '{"text":"BUY SOL"}'
    )


def test_sign_inbound_webhook_matches_backend_vector():
    headers = sign_inbound_webhook(
        signing_secret="test_secret",
        body={"text": "BUY SOL 500 USDC", "idempotencyKey": "sdk-test-1"},
        timestamp_seconds=1_710_000_000,
    )
    assert (
        headers["X-EZ-Signature"]
        == "1d30d896fc609e62bcf9be991c1dc1177a9c63219909869ef2846bad4685de3b"
    )


def test_sign_inbound_webhook_ignores_unknown_fields():
    body = {
        "eventType": "buy",
        "idempotencyKey": "test-1",
        "reasoning": "test",
        "tokenAddress": "So11111111111111111111111111111111111111112",
        "amount": 500,
    }
    with_extra = {**body, "unknownField": "strip me"}
    a = sign_inbound_webhook(
        signing_secret="secret", body=body, timestamp_seconds=1_710_000_000
    )
    b = sign_inbound_webhook(
        signing_secret="secret", body=with_extra, timestamp_seconds=1_710_000_000
    )
    assert a == b
    assert (
        a["X-EZ-Signature"]
        == "be420f61d91e6b871481774c62f972f0aafa6f5f1a727ba1e4a32558784f77c3"
    )


def test_verify_inbound_webhook_accepts_valid_signature():
    body = {"text": "BUY SOL 500 USDC", "idempotencyKey": "sdk-test-1"}
    headers = sign_inbound_webhook(
        signing_secret="test_secret",
        body=body,
        timestamp_seconds=1_710_000_000,
    )
    assert verify_inbound_webhook(
        signing_secret="test_secret",
        body=body,
        timestamp_seconds=headers["X-EZ-Timestamp"],
        signature=headers["X-EZ-Signature"],
        max_skew_seconds=999_999_999,
    )


def test_verify_outbound_webhook():
    raw_body = json.dumps(
        {
            "event": "signal.execution",
            "signalId": "sig_1",
            "developerAgentId": "agent_1",
            "status": "executed",
            "timestamp": "2026-07-06T12:00:00.000Z",
        },
        separators=(",", ":"),
    )
    timestamp = "2026-07-06T12:00:00.000Z"
    import hashlib
    import hmac as hmac_mod

    signature = hmac_mod.new(
        b"webhook_secret",
        f"{timestamp}.{raw_body}".encode(),
        hashlib.sha256,
    ).hexdigest()
    assert verify_outbound_webhook(
        secret_key="webhook_secret",
        raw_body=raw_body,
        timestamp=timestamp,
        signature=signature,
    )
