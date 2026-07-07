package echozero

import (
	"encoding/json"
	"math"
	"sort"
	"strings"
)

var structuredSigningKeys = []string{
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
}

func canonicalizeValue(value any) any {
	if value == nil {
		return nil
	}
	switch typed := value.(type) {
	case string:
		return typed
	case bool:
		return typed
	case int:
		return float64(typed)
	case int32:
		return float64(typed)
	case int64:
		return float64(typed)
	case float32:
		if math.IsNaN(float64(typed)) || math.IsInf(float64(typed), 0) {
			return nil
		}
		return float64(typed)
	case float64:
		if math.IsNaN(typed) || math.IsInf(typed, 0) {
			return nil
		}
		return typed
	case json.Number:
		if i, err := typed.Int64(); err == nil {
			return float64(i)
		}
		if f, err := typed.Float64(); err == nil {
			if math.IsNaN(f) || math.IsInf(f, 0) {
				return nil
			}
			return f
		}
		return nil
	case []any:
		out := make([]any, 0, len(typed))
		for _, item := range typed {
			if child := canonicalizeValue(item); child != nil || item == nil {
				out = append(out, child)
			}
		}
		return out
	case map[string]any:
		out := make(map[string]any)
		keys := make([]string, 0, len(typed))
		for key := range typed {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		for _, key := range keys {
			child := canonicalizeValue(typed[key])
			if child != nil || typed[key] == nil {
				out[key] = child
			}
		}
		return out
	default:
		return nil
	}
}

func InboundWebhookCanonicalJSON(body map[string]any) (string, error) {
	sorted := make(map[string]any)

	if text, ok := body["text"].(string); ok {
		sorted["text"] = text
	}
	if idk, ok := body["idempotencyKey"].(string); ok {
		trimmed := strings.TrimSpace(idk)
		if trimmed != "" {
			sorted["idempotencyKey"] = trimmed
		}
	}

	for _, key := range structuredSigningKeys {
		if value, ok := body[key]; ok {
			if child := canonicalizeValue(value); child != nil || value == nil {
				sorted[key] = child
			}
		}
	}

	keys := make([]string, 0, len(sorted))
	for key := range sorted {
		keys = append(keys, key)
	}
	sort.Strings(keys)

	obj := make(map[string]any, len(keys))
	for _, key := range keys {
		obj[key] = sorted[key]
	}
	raw, err := json.Marshal(obj)
	if err != nil {
		return "", err
	}
	return string(raw), nil
}
