#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = echozero::EchoZeroClient::new(
        std::env::var("ECHOZERO_BASE_URL")
            .unwrap_or_else(|_| "https://mcp.echozero.app".to_string()),
    )
    .with_api_key(std::env::var("ECHOZERO_API_KEY")?);

    let me = client.get_json("/api/v1/users/me").await?;
    println!("{me}");

    if let Ok(secret) = std::env::var("ECHOZERO_WEBHOOK_SECRET") {
        let mut body = serde_json::Map::new();
        body.insert("text".into(), serde_json::json!("BUY SOL 500 USDC"));
        let headers = echozero::sign_inbound_webhook(&secret, &body, None);
        println!("{headers:?}");
    }

    Ok(())
}
