from __future__ import annotations

import hashlib
import hmac
import json
import time
from typing import Any, Mapping


def stable_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def _hmac_sha256_hex(secret: str, payload: str) -> str:
    return hmac.new(secret.encode(), payload.encode(), hashlib.sha256).hexdigest()


def sign_rest_request(
    *,
    secret_key: str,
    method: str,
    path: str,
    body: Any | None = None,
    timestamp_ms: int | None = None,
) -> dict[str, str]:
    timestamp = str(timestamp_ms or int(time.time() * 1000))
    body_text = "" if body is None else body if isinstance(body, str) else json.dumps(body)
    payload = f"{timestamp}{method.upper()}{path}{body_text}"
    return {
        "x-timestamp": timestamp,
        "x-signature": _hmac_sha256_hex(secret_key, payload),
    }


def canonical_webhook_body(body: Mapping[str, Any]) -> str:
    return stable_json(dict(body))


def sign_inbound_webhook(
    *,
    signing_secret: str,
    body: Mapping[str, Any],
    timestamp_seconds: int | None = None,
) -> dict[str, str]:
    timestamp = str(timestamp_seconds or int(time.time()))
    canonical = canonical_webhook_body(body)
    return {
        "X-EZ-Timestamp": timestamp,
        "X-EZ-Signature": _hmac_sha256_hex(signing_secret, f"{timestamp}.{canonical}"),
    }


def verify_inbound_webhook(
    *,
    signing_secret: str,
    body: Mapping[str, Any],
    timestamp_seconds: int | str,
    signature: str,
    max_skew_seconds: int = 300,
) -> bool:
    try:
        timestamp = int(timestamp_seconds)
    except (TypeError, ValueError):
        return False
    if abs(int(time.time()) - timestamp) > max_skew_seconds:
        return False
    expected = sign_inbound_webhook(
        signing_secret=signing_secret,
        body=body,
        timestamp_seconds=timestamp,
    )["X-EZ-Signature"]
    return hmac.compare_digest(expected, signature.lower())
