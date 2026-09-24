#![cfg(feature = "websocket")]

use echozero::websocket::SignalClient;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

/// Minimal Socket.IO v4 stand-in for echozero-be TradeSignalGateway.
async fn fake_gateway() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut ws = tokio_tungstenite::accept_async(tcp).await.unwrap();
                ws.send(Message::Text(r#"0{"sid":"engine","upgrades":[]}"#.into())).await.unwrap();
                while let Some(Ok(Message::Text(packet))) = ws.next().await {
                    if let Some(auth) = packet.strip_prefix("40/ws/signals,") {
                        let auth: Value = serde_json::from_str(auth).unwrap();
                        ws.send(Message::Text(r#"40/ws/signals,{"sid":"ns"}"#.into())).await.unwrap();
                        ws.send(Message::Text("2".into())).await.unwrap();
                        if auth["apiKey"] != "ez_live_good" {
                            ws.send(Message::Text(
                                r#"42/ws/signals,["error",{"message":"Invalid or expired API key"}]"#.into(),
                            ))
                            .await
                            .unwrap();
                            return;
                        }
                        ws.send(Message::Text(r#"42/ws/signals,["authenticated",{"ok":true}]"#.into()))
                            .await
                            .unwrap();
                    } else if let Some(frame) = packet.strip_prefix("42/ws/signals,") {
                        let frame: Value = serde_json::from_str(frame).unwrap();
                        let data = &frame[1];
                        let reply = json!(["signal:received", {
                            "signalId": format!("sig-{}", data["idempotencyKey"].as_str().unwrap()),
                            "status": "accepted",
                            "agent": data["developerAgentId"],
                        }]);
                        ws.send(Message::Text(format!("42/ws/signals,{reply}"))).await.unwrap();
                    }
                }
            });
        }
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn connects_sends_and_receives() {
    let base_url = fake_gateway().await;
    let mut client = SignalClient::connect(&base_url, "ez_live_good").await.unwrap();
    let signal = json!({ "eventType": "buy", "idempotencyKey": "k1" });
    client
        .send_signal("agent-1", signal.as_object().unwrap().clone())
        .unwrap();
    let event = client.next_event().await.unwrap();
    assert_eq!(event.name, "signal:received");
    assert_eq!(event.data["signalId"], "sig-k1");
    assert_eq!(event.data["agent"], "agent-1");
    client.close();
}

#[tokio::test]
async fn invalid_key_is_rejected() {
    let base_url = fake_gateway().await;
    let err = SignalClient::connect(&base_url, "ez_live_bad").await.err().unwrap();
    assert!(err.to_string().contains("Invalid or expired API key"), "{err}");
}
