use crate::hmac::sign_rest_request;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde_json::Value;
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
        self.request_json(reqwest::Method::GET, path, None, false).await
    }

    pub async fn post_json(
        &self,
        path: &str,
        body: Value,
        hmac: bool,
    ) -> Result<Value, EchoZeroError> {
        self.request_json(reqwest::Method::POST, path, Some(body), hmac).await
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
        if body.is_some() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
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
            let (timestamp, signature) =
                sign_rest_request(secret, method.as_str(), &path_with_query, body.as_ref(), None);
            headers.insert("x-timestamp", HeaderValue::from_str(&timestamp).unwrap());
            headers.insert("x-signature", HeaderValue::from_str(&signature).unwrap());
        }

        let mut request = self.http.request(method, url).headers(headers);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
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
