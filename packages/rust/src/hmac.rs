use crate::inbound_canonical::inbound_webhook_canonical_json;
use hmac::{Hmac, Mac};
use serde_json::{Map, Value};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub fn stable_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let fields = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap(),
                        stable_json(&map[key])
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{fields}}}")
        }
        Value::Array(values) => {
            let items = values.iter().map(stable_json).collect::<Vec<_>>().join(",");
            format!("[{items}]")
        }
        _ => serde_json::to_string(value).unwrap(),
    }
}

pub fn hmac_sha256_hex(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn sign_rest_request(
    secret_key: &str,
    method: &str,
    path: &str,
    body_text: &str,
    timestamp_ms: Option<u128>,
) -> (String, String) {
    let timestamp = timestamp_ms.unwrap_or_else(now_ms).to_string();
    let signature = hmac_sha256_hex(
        secret_key,
        &format!("{}{}{}{}", timestamp, method.to_uppercase(), path, body_text),
    );
    (timestamp, signature)
}

pub fn canonical_webhook_body(body: &Map<String, Value>) -> String {
    inbound_webhook_canonical_json(body)
}

pub fn sign_inbound_webhook(
    signing_secret: &str,
    body: &Map<String, Value>,
    timestamp_seconds: Option<u64>,
) -> (String, String) {
    let timestamp = timestamp_seconds.unwrap_or_else(now_seconds).to_string();
    let canonical = inbound_webhook_canonical_json(body);
    let signature = hmac_sha256_hex(signing_secret, &format!("{timestamp}.{canonical}"));
    (timestamp, signature)
}

pub fn verify_inbound_webhook(
    signing_secret: &str,
    body: &Map<String, Value>,
    timestamp_seconds: u64,
    signature: &str,
    max_skew_seconds: u64,
) -> bool {
    let now = now_seconds();
    if now.abs_diff(timestamp_seconds) > max_skew_seconds {
        return false;
    }
    let (_, expected) = sign_inbound_webhook(signing_secret, body, Some(timestamp_seconds));
    expected == signature.to_ascii_lowercase()
}

pub fn verify_outbound_webhook(
    secret_key: &str,
    raw_body: &str,
    timestamp: &str,
    signature: &str,
) -> bool {
    let expected = hmac_sha256_hex(secret_key, &format!("{timestamp}.{raw_body}"));
    expected == signature.to_ascii_lowercase()
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis()
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn signs_rest_requests_with_backend_payload_format() {
        let body = json!({ "name": "SDK HMAC Test" });
        let body_text = serde_json::to_string(&body).unwrap();
        let (_, signature) = sign_rest_request(
            "test_secret",
            "POST",
            "/api/api-keys",
            &body_text,
            Some(1_710_000_000_000),
        );

        assert_eq!(
            signature,
            "274c9ff280eadf751530e9e7fce2c2a573d8676b13b7108fe353407df7cc9e00"
        );
    }

    #[test]
    fn signs_inbound_webhooks_with_canonical_json() {
        let mut body = serde_json::Map::new();
        body.insert("text".to_string(), json!("BUY SOL 500 USDC"));
        body.insert("idempotencyKey".to_string(), json!("sdk-test-1"));

        let (_, signature) = sign_inbound_webhook("test_secret", &body, Some(1_710_000_000));

        assert_eq!(
            signature,
            "1d30d896fc609e62bcf9be991c1dc1177a9c63219909869ef2846bad4685de3b"
        );
    }

    #[test]
    fn inbound_canonical_strips_unknown_fields() {
        let body = json!({"text":"BUY SOL","extraField":"ignored"})
            .as_object()
            .cloned()
            .unwrap();
        assert_eq!(inbound_webhook_canonical_json(&body), r#"{"text":"BUY SOL"}"#);
    }

    #[test]
    fn signs_inbound_webhooks_ignoring_unknown_fields() {
        let body = json!({
            "eventType": "buy",
            "idempotencyKey": "test-1",
            "reasoning": "test",
            "tokenAddress": "So11111111111111111111111111111111111111112",
            "amount": 500
        })
        .as_object()
        .cloned()
        .unwrap();
        let canonical = crate::inbound_canonical::inbound_webhook_canonical_json(&body);
        assert_eq!(
            canonical,
            r#"{"amount":500,"eventType":"buy","idempotencyKey":"test-1","reasoning":"test","tokenAddress":"So11111111111111111111111111111111111111112"}"#
        );
        let mut with_extra = body.clone();
        with_extra.insert("unknownField".to_string(), json!("strip me"));

        let (_, a) = sign_inbound_webhook("secret", &body, Some(1_710_000_000));
        let (_, b) = sign_inbound_webhook("secret", &with_extra, Some(1_710_000_000));
        assert_eq!(a, b);
        assert_eq!(
            a,
            "be420f61d91e6b871481774c62f972f0aafa6f5f1a727ba1e4a32558784f77c3"
        );
    }
}
