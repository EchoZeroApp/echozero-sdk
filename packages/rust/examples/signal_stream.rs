//! cargo run --example signal_stream --features websocket
use echozero::websocket::SignalClient;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("ECHOZERO_API_KEY").unwrap_or_else(|_| "ez_live_invalid_probe".into());
    match SignalClient::connect("https://mcp.echozero.app", &api_key).await {
        Ok(mut client) => {
            println!("authenticated");
            if let Some(event) = client.next_event().await {
                println!("{} {}", event.name, event.data);
            }
        }
        Err(err) => println!("live result: {err}"),
    }
}
