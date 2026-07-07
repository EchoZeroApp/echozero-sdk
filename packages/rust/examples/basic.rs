#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("ECHOZERO_BASE_URL")
        .unwrap_or_else(|_| "https://mcp.echozero.app".to_string());

    let client = match (
        std::env::var("ECHOZERO_API_KEY"),
        std::env::var("ECHOZERO_OAUTH_TOKEN"),
    ) {
        (Ok(api_key), _) => echozero::EchoZeroClient::new(&base_url).with_api_key(api_key),
        (_, Ok(token)) => echozero::EchoZeroClient::new(&base_url).with_bearer_token(token),
        _ => echozero::EchoZeroClient::new(&base_url),
    };

    let metadata = client.get_json("/public/index.json").await?;
    println!(
        "Connected to: {}",
        metadata
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("EchoZero")
    );

    if std::env::var("ECHOZERO_API_KEY").is_ok()
        || std::env::var("ECHOZERO_OAUTH_TOKEN").is_ok()
    {
        let api_keys = client.get_json("/api/api-keys").await?;
        println!("Authenticated request OK: {}", json_type_name(&api_keys));
    }

    if let Ok(secret) = std::env::var("ECHOZERO_WEBHOOK_SECRET") {
        let mut body = serde_json::Map::new();
        body.insert(
            "text".into(),
            serde_json::json!("SDK health check message with no market instruction"),
        );
        let (_, signature) = echozero::sign_inbound_webhook(&secret, &body, None);
        println!("Webhook signature generated: {}", !signature.is_empty());
    }

    Ok(())
}

fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
