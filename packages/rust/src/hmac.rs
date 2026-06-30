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
    body: Option<&Value>,
    timestamp_ms: Option<u128>,
) -> (String, String) {
    let timestamp = timestamp_ms.unwrap_or_else(now_ms).to_string();
    let body_text = body
        .map(serde_json::to_string)
        .transpose()
        .unwrap()
        .unwrap_or_default();
    let signature = hmac_sha256_hex(
        secret_key,
        &format!(
            "{}{}{}{}",
            timestamp,
            method.to_uppercase(),
            path,
            body_text
        ),
    );
    (timestamp, signature)
}

pub fn canonical_webhook_body(body: &Map<String, Value>) -> String {
    stable_json(&Value::Object(body.clone()))
}

pub fn sign_inbound_webhook(
    signing_secret: &str,
    body: &Map<String, Value>,
    timestamp_seconds: Option<u64>,
) -> (String, String) {
    let timestamp = timestamp_seconds.unwrap_or_else(now_seconds).to_string();
    let canonical = canonical_webhook_body(body);
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

        let (_, signature) = sign_rest_request(
            "test_secret",
            "POST",
            "/api/api-keys",
            Some(&body),
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
}
