#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = echozero::EchoZeroClient::new(
        std::env::var("ECHOZERO_BASE_URL")
            .unwrap_or_else(|_| "https://mcp.echozero.app".to_string()),
    )
    .with_api_key(std::env::var("ECHOZERO_API_KEY")?);

    let metadata = client.get_json("/public/index.json").await?;
    println!(
        "Connected to: {}",
        metadata
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("EchoZero")
    );

    let api_keys = client.get_json("/api/api-keys").await?;
    println!("Authenticated request OK: {}", api_keys.type_name());

    if let Ok(secret) = std::env::var("ECHOZERO_WEBHOOK_SECRET") {
        let mut body = serde_json::Map::new();
        body.insert(
            "text".into(),
            serde_json::json!("SDK health check message with no market instruction"),
        );
        let headers = echozero::sign_inbound_webhook(&secret, &body, None);
        println!("Webhook signature generated: {}", !headers.1.is_empty());
    }

    Ok(())
}

trait JsonValueTypeName {
    fn type_name(&self) -> &'static str;
}

impl JsonValueTypeName for serde_json::Value {
    fn type_name(&self) -> &'static str {
        match self {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "bool",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
}
