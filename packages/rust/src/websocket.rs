#[cfg(feature = "websocket")]
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[cfg(feature = "websocket")]
pub async fn connect_signal_stream(
    url: &str,
    api_key: Option<&str>,
    bearer_token: Option<&str>,
) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Error> {
    let mut parsed = url::Url::parse(url).expect("valid websocket url");
    if let Some(api_key) = api_key {
        parsed.query_pairs_mut().append_pair("api_key", api_key);
    }
    if let Some(token) = bearer_token {
        parsed.query_pairs_mut().append_pair("access_token", token);
    }
    let (stream, _) = connect_async(parsed.as_str()).await?;
    Ok(stream)
}

#[cfg(feature = "websocket")]
pub fn signal_message(signal: serde_json::Value) -> Message {
    Message::Text(serde_json::json!({ "event": "signal", "data": signal }).to_string())
}
