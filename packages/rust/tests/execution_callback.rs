use echozero::{EchoZeroClient, EchoZeroError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// One-shot HTTP stand-in. Replies with `status` and `body`, and returns the
/// raw request (headers and body) through the join handle.
async fn fake_server(status: &'static str, body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let (mut tcp, _) = listener.accept().await.unwrap();
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = tcp.read(&mut buf).await.unwrap();
            raw.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&raw).to_string();
            if let Some(split) = text.find("\r\n\r\n") {
                let lower = text[..split].to_lowercase();
                let len = lower
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:"))
                    .and_then(|v| v.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if raw.len() >= split + 4 + len {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        tcp.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&raw).to_string()
    });
    (base, handle)
}

#[tokio::test]
async fn set_execution_callback_puts_url() {
    let (base, request) = fake_server("200 OK", r#"{"success":true,"data":{"id":"a1"}}"#).await;
    let client = EchoZeroClient::new(base).with_api_key("k");
    let agent = client
        .set_execution_callback("a1", "https://hooks.example.com/ez")
        .await
        .unwrap();
    assert_eq!(agent["id"], "a1");
    let raw = request.await.unwrap();
    assert!(raw.starts_with("PUT /api/v1/agents/a1/execution-callback "));
    assert!(raw.to_lowercase().contains("x-api-key: k"));
    assert!(!raw.to_lowercase().contains("x-signature"));
    assert!(raw.ends_with(r#"{"url":"https://hooks.example.com/ez"}"#));
}

#[tokio::test]
async fn execution_callback_signs_when_secret_set() {
    let (base, request) = fake_server("200 OK", r#"{"success":true,"data":{}}"#).await;
    let client = EchoZeroClient::new(base)
        .with_api_key("k")
        .with_hmac_secret_key("s");
    client
        .set_execution_callback("a1", "https://hooks.example.com/ez")
        .await
        .unwrap();
    let raw = request.await.unwrap().to_lowercase();
    assert!(raw.contains("x-signature: "));
    assert!(raw.contains("x-timestamp: "));
}

#[tokio::test]
async fn clear_execution_callback_sends_delete() {
    let (base, request) = fake_server("200 OK", r#"{"success":true,"data":{}}"#).await;
    let client = EchoZeroClient::new(base).with_api_key("k");
    client.clear_execution_callback("a/1").await.unwrap();
    let raw = request.await.unwrap();
    assert!(raw.starts_with("DELETE /api/v1/agents/a%2F1/execution-callback "));
}

#[tokio::test]
async fn rejected_url_is_an_api_error() {
    let (base, _request) = fake_server(
        "400 Bad Request",
        r#"{"success":false,"error":{"code":"BAD_REQUEST","message":"Callback URL must use https"}}"#,
    )
    .await;
    let client = EchoZeroClient::new(base).with_api_key("k");
    let err = client
        .set_execution_callback("a1", "http://hooks.example.com/ez")
        .await
        .unwrap_err();
    match err {
        EchoZeroError::Api { status, message, .. } => {
            assert_eq!(status, 400);
            assert_eq!(message, "Callback URL must use https");
        }
        other => panic!("unexpected error {other:?}"),
    }
}
