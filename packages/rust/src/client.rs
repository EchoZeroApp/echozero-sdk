use crate::hmac::sign_inbound_webhook;
use crate::hmac::sign_rest_request;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde_json::{Map, Value};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum EchoZeroError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url error: {0}")]
    Url(#[from] url::ParseError),
    #[error("api error {status}: {message}")]
    Api {
        status: u16,
        message: String,
        code: Option<String>,
        payload: Value,
    },
    #[error("hmac_secret_key is required when hmac=true")]
    MissingHmacSecret,
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct EchoZeroClient {
    base_url: String,
    api_key: Option<String>,
    bearer_token: Option<String>,
    hmac_secret_key: Option<String>,
    http: reqwest::Client,
}

impl EchoZeroClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: None,
            bearer_token: None,
            hmac_secret_key: None,
            http: reqwest::Client::new(),
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_bearer_token(mut self, bearer_token: impl Into<String>) -> Self {
        self.bearer_token = Some(bearer_token.into());
        self
    }

    pub fn with_hmac_secret_key(mut self, hmac_secret_key: impl Into<String>) -> Self {
        self.hmac_secret_key = Some(hmac_secret_key.into());
        self
    }

    pub async fn get_json(&self, path: &str) -> Result<Value, EchoZeroError> {
        self.request_json(reqwest::Method::GET, path, None, false)
            .await
    }

    pub async fn post_json(
        &self,
        path: &str,
        body: Value,
        hmac: bool,
    ) -> Result<Value, EchoZeroError> {
        self.request_json(reqwest::Method::POST, path, Some(body), hmac)
            .await
    }

    pub async fn post_agent_signal(
        &self,
        agent_id: &str,
        body: Map<String, Value>,
        signing_secret: &str,
    ) -> Result<Value, EchoZeroError> {
        let (timestamp, signature) = sign_inbound_webhook(signing_secret, &body, None);
        let url = self.url(&format!("/api/public/agent-signals/{agent_id}"))?;
        let body_value = Value::Object(body);
        let body_text = serde_json::to_string(&body_value)?;
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-EZ-Timestamp", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-EZ-Signature", HeaderValue::from_str(&signature).unwrap());
        if let Some(token) = &self.bearer_token {
            headers.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
        } else if let Some(api_key) = &self.api_key {
            headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
        }

        let response = self
            .http
            .post(url)
            .headers(headers)
            .body(body_text)
            .send()
            .await?;
        self.decode_response(response).await
    }

    pub async fn request_json(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
        hmac: bool,
    ) -> Result<Value, EchoZeroError> {
        let url = self.url(path)?;
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        let body_text = match &body {
            Some(value) => {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                Some(serde_json::to_string(value)?)
            }
            None => None,
        };
        if let Some(token) = &self.bearer_token {
            headers.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
            );
        } else if let Some(api_key) = &self.api_key {
            headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
        }
        if hmac {
            let secret = self
                .hmac_secret_key
                .as_deref()
                .ok_or(EchoZeroError::MissingHmacSecret)?;
            let path_with_query = path_with_query(&url);
            let (timestamp, signature) = sign_rest_request(
                secret,
                method.as_str(),
                &path_with_query,
                body_text.as_deref().unwrap_or(""),
                None,
            );
            headers.insert("x-timestamp", HeaderValue::from_str(&timestamp).unwrap());
            headers.insert("x-signature", HeaderValue::from_str(&signature).unwrap());
        }

        let mut request = self.http.request(method, url).headers(headers);
        if let Some(body_text) = body_text {
            request = request.body(body_text);
        }
        let response = request.send().await?;
        self.decode_response(response).await
    }

    async fn decode_response(
        &self,
        response: reqwest::Response,
    ) -> Result<Value, EchoZeroError> {
        let status = response.status();
        let payload = response.json::<Value>().await.unwrap_or(Value::Null);
        if !status.is_success() {
            let error = payload.get("error").and_then(Value::as_object);
            return Err(EchoZeroError::Api {
                status: status.as_u16(),
                message: error
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("EchoZero API request failed")
                    .to_string(),
                code: error
                    .and_then(|e| e.get("code"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                payload,
            });
        }
        if payload.get("success").and_then(Value::as_bool) == Some(true) {
            Ok(payload.get("data").cloned().unwrap_or(Value::Null))
        } else {
            Ok(payload)
        }
    }

    fn url(&self, path: &str) -> Result<Url, EchoZeroError> {
        if path.starts_with("http://") || path.starts_with("https://") {
            Ok(Url::parse(path)?)
        } else {
            Ok(Url::parse(&format!(
                "{}{}",
                self.base_url,
                if path.starts_with('/') {
                    path.to_string()
                } else {
                    format!("/{path}")
                }
            ))?)
        }
    }
}

fn path_with_query(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{}", url.path(), query),
        None => url.path().to_string(),
    }
}
