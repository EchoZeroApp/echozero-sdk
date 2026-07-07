"""Structured signal envelope types (#1098)."""

from __future__ import annotations

from typing import Literal, TypedDict

StructuredSignalEventType = Literal[
    "trade_idea",
    "buy",
    "scale_in",
    "sell",
    "partial_sell",
    "amend",
    "breakeven",
    "cancel",
    "position_update",
    "trade_review",
]

StructuredSignalChain = Literal["solana", "hyperliquid"]
StructuredSignalSide = Literal["long", "short"]
StructuredSignalTradeType = Literal["spot", "perp", "virtual"]


class AgentSignalResponse(TypedDict, total=False):
    outcome: Literal["matched", "unmatched", "skipped", "error"]
    signalId: str
    status: str
    skipReason: str


class OutboundExecutionWebhook(TypedDict, total=False):
    event: Literal["signal.execution", "signal.execution.failed"]
    signalId: str
    developerAgentId: str
    status: str
    executionResult: dict[str, object]
    timestamp: str
