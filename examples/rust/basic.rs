use echozero::{sign_inbound_webhook, EchoZeroClient};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("ECHOZERO_BASE_URL")
        .unwrap_or_else(|_| "https://mcp.echozero.app".to_string());

    if let Ok(secret) = std::env::var("ECHOZERO_WEBHOOK_SECRET") {
        run_webhook_signing_demo(&secret);
    } else {
        run_webhook_signing_demo("ezw_example_secret_for_local_demo");
        println!("(Set ECHOZERO_WEBHOOK_SECRET to use your agent signing secret.)");
    }

    if std::env::var("ECHOZERO_OFFLINE").is_ok() {
        println!("ECHOZERO_OFFLINE set — skipping HTTP requests.");
        return Ok(());
    }

    let client = match (
        std::env::var("ECHOZERO_API_KEY"),
        std::env::var("ECHOZERO_OAUTH_TOKEN"),
    ) {
        (Ok(api_key), _) => EchoZeroClient::new(&base_url).with_api_key(api_key),
        (_, Ok(token)) => EchoZeroClient::new(&base_url).with_bearer_token(token),
        _ => EchoZeroClient::new(&base_url),
    };

    match client.get_json("/public/index.json").await {
        Ok(metadata) => println!(
            "Connected to: {}",
            metadata
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("EchoZero")
        ),
        Err(err) => {
            eprintln!("Could not reach {base_url}: {err}");
            print_network_help(&base_url);
            return Err(err.into());
        }
    }

    if std::env::var("ECHOZERO_API_KEY").is_ok()
        || std::env::var("ECHOZERO_OAUTH_TOKEN").is_ok()
    {
        let api_keys = client.get_json("/api/api-keys").await?;
        println!("Authenticated request OK: {}", json_type_name(&api_keys));
    }

    Ok(())
}

fn run_webhook_signing_demo(secret: &str) {
    let mut body = serde_json::Map::new();
    body.insert(
        "text".into(),
        Value::String("SDK health check message with no market instruction".into()),
    );
    let (_, signature) = sign_inbound_webhook(secret, &body, None);
    println!("Webhook signature generated: {}", !signature.is_empty());
}

fn print_network_help(base_url: &str) {
    eprintln!();
    eprintln!("This is usually a DNS or network issue on your machine, not the SDK.");
    eprintln!("Try:");
    eprintln!("  getent hosts mcp.echozero.app");
    eprintln!("  curl -I {base_url}/public/index.json");
    eprintln!();
    eprintln!("For local mcp-main:");
    eprintln!("  export ECHOZERO_BASE_URL=http://localhost:4010");
    eprintln!();
    eprintln!("To run signing-only without HTTP:");
    eprintln!("  ECHOZERO_OFFLINE=1 cargo run");
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
