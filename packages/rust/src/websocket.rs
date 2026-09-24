//! Socket.IO v4 client (WebSocket transport) for the EchoZero signal gateway.
//!
//! The WebSocket gateway accepts structured envelopes (`eventType`) and the
//! legacy `{action, tokenAddress, amount}` shape. Natural-language `text`
//! signals are only accepted by the HTTP inbound webhook.

#[cfg(feature = "websocket")]
mod imp {
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{Map, Value};
    use thiserror::Error;
    use tokio::sync::mpsc;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::HeaderValue;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::connect_async;

    /// Socket.IO namespace of the developer signal gateway.
    pub const SIGNAL_NAMESPACE: &str = "/ws/signals";

    #[derive(Debug, Error)]
    pub enum SignalError {
        #[error("websocket error: {0}")]
        WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
        #[error("url error: {0}")]
        Url(#[from] url::ParseError),
        #[error("json error: {0}")]
        Json(#[from] serde_json::Error),
        #[error("echozero signal gateway: {0}")]
        Gateway(String),
        #[error("signal websocket is not connected")]
        NotConnected,
    }

    /// A server event such as `signal:received` or `signal:error`.
    /// A final event named `disconnect` is delivered when the connection ends.
    #[derive(Debug, Clone)]
    pub struct SignalEvent {
        pub name: String,
        pub data: Value,
    }

    pub struct SignalClient {
        outgoing: mpsc::UnboundedSender<String>,
        events: mpsc::UnboundedReceiver<SignalEvent>,
    }

    impl SignalClient {
        /// Connects to `base_url` (e.g. `https://mcp.echozero.app`) and waits
        /// for the gateway's `authenticated` event. Wrap in
        /// `tokio::time::timeout` to bound the wait.
        pub async fn connect(base_url: &str, api_key: &str) -> Result<Self, SignalError> {
            let mut request = socket_io_url(base_url)?.as_str().into_client_request()?;
            request.headers_mut().insert(
                "x-api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|_| SignalError::Gateway("invalid API key header".into()))?,
            );
            let (mut stream, _) = connect_async(request).await?;

            // Engine.IO open packet: 0{"sid":...}
            match stream.next().await {
                Some(Ok(Message::Text(packet))) if packet.starts_with('0') => {}
                other => {
                    return Err(SignalError::Gateway(format!("unexpected open packet {other:?}")))
                }
            }

            let auth = serde_json::json!({ "apiKey": api_key });
            stream
                .send(Message::Text(format!("40{SIGNAL_NAMESPACE},{auth}")))
                .await?;

            loop {
                let packet = match stream.next().await {
                    Some(Ok(Message::Text(packet))) => packet,
                    Some(Ok(_)) => continue,
                    Some(Err(err)) => return Err(err.into()),
                    None => return Err(SignalError::Gateway("connection closed".into())),
                };
                if packet == "2" {
                    stream.send(Message::Text("3".into())).await?;
                } else if let Some(payload) = strip_packet(&packet, "44") {
                    return Err(SignalError::Gateway(gateway_message(
                        &serde_json::from_str(payload).unwrap_or(Value::String(payload.into())),
                    )));
                } else if let Some(event) = parse_event(&packet) {
                    match event.name.as_str() {
                        "authenticated" => break,
                        "error" => return Err(SignalError::Gateway(gateway_message(&event.data))),
                        _ => {}
                    }
                }
            }

            let (outgoing, mut outgoing_rx) = mpsc::unbounded_channel::<String>();
            let (events_tx, events) = mpsc::unbounded_channel();
            tokio::spawn(async move {
                let (mut sink, mut source) = stream.split();
                loop {
                    tokio::select! {
                        message = source.next() => {
                            let packet = match message {
                                Some(Ok(Message::Text(packet))) => packet,
                                Some(Ok(_)) => continue,
                                _ => break,
                            };
                            if packet == "2" {
                                if sink.send(Message::Text("3".into())).await.is_err() {
                                    break;
                                }
                            } else if strip_packet(&packet, "41").is_some() {
                                break;
                            } else if let Some(event) = parse_event(&packet) {
                                let _ = events_tx.send(event);
                            }
                        }
                        packet = outgoing_rx.recv() => {
                            match packet {
                                Some(packet) => {
                                    if sink.send(Message::Text(packet)).await.is_err() {
                                        break;
                                    }
                                }
                                None => {
                                    let _ = sink
                                        .send(Message::Text(format!("41{SIGNAL_NAMESPACE},")))
                                        .await;
                                    let _ = sink.close().await;
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = events_tx.send(SignalEvent {
                    name: "disconnect".into(),
                    data: Value::Null,
                });
            });

            Ok(Self { outgoing, events })
        }

        /// Emits a `signal` event for one of your developer agents.
        pub fn send_signal(
            &self,
            developer_agent_id: &str,
            signal: Map<String, Value>,
        ) -> Result<(), SignalError> {
            let mut payload = signal;
            payload.insert(
                "developerAgentId".into(),
                Value::String(developer_agent_id.into()),
            );
            let frame = serde_json::to_string(&serde_json::json!(["signal", payload]))?;
            self.outgoing
                .send(format!("42{SIGNAL_NAMESPACE},{frame}"))
                .map_err(|_| SignalError::NotConnected)
        }

        /// Next server event, or `None` once the connection task has ended.
        pub async fn next_event(&mut self) -> Option<SignalEvent> {
            self.events.recv().await
        }

        /// Sends a namespace disconnect and closes the socket.
        pub fn close(self) {
            drop(self.outgoing);
        }
    }

    fn socket_io_url(base_url: &str) -> Result<url::Url, SignalError> {
        let mut parsed = url::Url::parse(base_url.trim_end_matches('/'))?;
        let scheme = match parsed.scheme() {
            "https" | "wss" => "wss",
            "http" | "ws" => "ws",
            other => return Err(SignalError::Gateway(format!("unsupported scheme {other}"))),
        };
        parsed
            .set_scheme(scheme)
            .map_err(|_| SignalError::Gateway("invalid base URL".into()))?;
        parsed.set_path("/socket.io/");
        parsed.set_query(Some("EIO=4&transport=websocket"));
        Ok(parsed)
    }

    fn strip_packet<'a>(packet: &'a str, packet_type: &str) -> Option<&'a str> {
        packet
            .strip_prefix(packet_type)?
            .strip_prefix(SIGNAL_NAMESPACE)
            .map(|rest| rest.strip_prefix(',').unwrap_or(rest))
    }

    fn parse_event(packet: &str) -> Option<SignalEvent> {
        let frame: Vec<Value> = serde_json::from_str(strip_packet(packet, "42")?).ok()?;
        let mut frame = frame.into_iter();
        let name = frame.next()?.as_str()?.to_string();
        Some(SignalEvent {
            name,
            data: frame.next().unwrap_or(Value::Null),
        })
    }

    fn gateway_message(data: &Value) -> String {
        data.get("message")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| data.to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn builds_socket_io_url() {
            assert_eq!(
                socket_io_url("https://mcp.echozero.app/").unwrap().as_str(),
                "wss://mcp.echozero.app/socket.io/?EIO=4&transport=websocket"
            );
        }

        #[test]
        fn parses_namespaced_event() {
            let event = parse_event(r#"42/ws/signals,["signal:received",{"signalId":"s1"}]"#).unwrap();
            assert_eq!(event.name, "signal:received");
            assert_eq!(event.data["signalId"], "s1");
            assert!(parse_event(r#"42/other,["x"]"#).is_none());
        }
    }
}

#[cfg(feature = "websocket")]
pub use imp::*;
