use serde_json::{Map, Value};

const STRUCTURED_SIGNING_KEYS: &[&str] = &[
    "action", "amount", "chain", "changes", "clientTimestamp", "confidence", "context",
    "currentPnlPct", "entryPrice", "entryZone", "eventType", "exitPrice", "exitReason",
    "expiresAt", "ideaId", "instrument", "leverageX", "metadata", "orderType", "pnlPct",
    "positionRef", "reasoning", "relatedTradeId", "riskMgmt", "sellSizePct", "side",
    "symbol", "tokenAddress", "tradeType", "triggers", "urgency", "version",
];

fn canonicalize_value(value: &Value) -> Option<Value> {
    match value {
        Value::Null => Some(Value::Null),
        Value::Bool(_) | Value::String(_) => Some(value.clone()),
        Value::Number(number) => number
            .as_f64()
            .filter(|n| n.is_finite())
            .map(|_| Value::Number(number.clone())),
        Value::Array(items) => Some(Value::Array(
            items
                .iter()
                .filter_map(canonicalize_value)
                .collect::<Vec<_>>(),
        )),
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for key in keys {
                if let Some(child) = canonicalize_value(&map[&key]) {
                    out.insert(key, child);
                } else if map[&key].is_null() {
                    out.insert(key, Value::Null);
                }
            }
            Some(Value::Object(out))
        }
    }
}

pub fn inbound_webhook_canonical_json(body: &Map<String, Value>) -> String {
    let mut sorted = Map::new();

    if let Some(Value::String(text)) = body.get("text") {
        sorted.insert("text".to_string(), Value::String(text.clone()));
    }

    if let Some(Value::String(idk)) = body.get("idempotencyKey") {
        let trimmed = idk.trim();
        if !trimmed.is_empty() {
            sorted.insert(
                "idempotencyKey".to_string(),
                Value::String(trimmed.to_string()),
            );
        }
    }

    for key in STRUCTURED_SIGNING_KEYS {
        if let Some(value) = body.get(*key) {
            if let Some(child) = canonicalize_value(value) {
                sorted.insert(key.to_string(), child);
            } else if value.is_null() {
                sorted.insert(key.to_string(), Value::Null);
            }
        }
    }

    let mut keys: Vec<_> = sorted.keys().cloned().collect();
    keys.sort();
    let mut obj = Map::new();
    for key in keys {
        obj.insert(key.clone(), sorted[&key].clone());
    }
    serde_json::to_string(&Value::Object(obj)).unwrap_or_else(|_| "{}".to_string())
}
