"""Mirrors `agentInboundWebhook.signing.ts` in mcp-main."""

from __future__ import annotations

import json
from typing import Any, Mapping

STRUCTURED_SIGNING_KEYS = (
    "action",
    "amount",
    "chain",
    "changes",
    "clientTimestamp",
    "confidence",
    "context",
    "currentPnlPct",
    "entryPrice",
    "entryZone",
    "eventType",
    "exitPrice",
    "exitReason",
    "expiresAt",
    "ideaId",
    "instrument",
    "leverageX",
    "metadata",
    "orderType",
    "pnlPct",
    "positionRef",
    "reasoning",
    "relatedTradeId",
    "riskMgmt",
    "sellSizePct",
    "side",
    "symbol",
    "tokenAddress",
    "tradeType",
    "triggers",
    "urgency",
    "version",
)


def _canonicalize_value(value: Any) -> Any | None:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, (int, float)):
        if isinstance(value, float) and (value != value or value in (float("inf"), float("-inf"))):
            return None
        return value
    if isinstance(value, list):
        return [_canonicalize_value(item) for item in value]
    if isinstance(value, dict):
        out: dict[str, Any] = {}
        for key in sorted(value):
            child = _canonicalize_value(value[key])
            if child is not None or value[key] is None:
                out[key] = child
        return out
    return None


def inbound_webhook_canonical_json(body: Mapping[str, Any]) -> str:
    sorted_body: dict[str, Any] = {}

    if "text" in body and body["text"] is not None:
        sorted_body["text"] = body["text"]

    idk = body.get("idempotencyKey")
    if isinstance(idk, str):
        trimmed = idk.strip()
        if trimmed:
            sorted_body["idempotencyKey"] = trimmed

    for key in STRUCTURED_SIGNING_KEYS:
        if key not in body:
            continue
        value = _canonicalize_value(body[key])
        if value is not None or body[key] is None:
            sorted_body[key] = value

    return json.dumps(
        {key: sorted_body[key] for key in sorted(sorted_body)},
        separators=(",", ":"),
        ensure_ascii=False,
    )


def rest_request_body_text(body: Any | None) -> str:
    if body is None:
        return ""
    if isinstance(body, str):
        return body
    return json.dumps(body, separators=(",", ":"), ensure_ascii=False)
