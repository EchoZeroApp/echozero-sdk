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
                .map(|key| format!("{}:{}", serde_json::to_string(key).unwrap(), stable_json(&map[key])))
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
    let body_text = body.map(serde_json::to_string).transpose().unwrap().unwrap_or_default();
    let signature = hmac_sha256_hex(
        secret_key,
        &format!("{}{}{}{}", timestamp, method.to_uppercase(), path, body_text),
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
