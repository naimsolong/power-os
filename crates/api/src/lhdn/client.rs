// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

const TOKEN_MARGIN: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub struct LhdnSettings {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct LhdnClient {
    http: Client,
    tokens: Arc<Mutex<HashMap<String, TokenEntry>>>,
}

struct TokenEntry {
    access_token: String,
    expires_at: Instant,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Serialize, Debug)]
pub struct SubmitPayload {
    pub documents: Vec<DocumentWrapper>,
}

#[derive(Serialize, Debug)]
pub struct DocumentWrapper {
    pub format: String,
    pub document: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    pub uuid: Option<String>,
    #[serde(rename = "submissionUid")]
    pub submission_uid: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentStatusResponse {
    pub uuid: Option<String>,
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
pub enum LhdnError {
    Network(reqwest::Error),
    Api {
        status: StatusCode,
        body: String,
    },
    Token(String),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for LhdnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LhdnError::Network(e) => write!(f, "LHDN network error: {e}"),
            LhdnError::Api { status, body } => {
                write!(f, "LHDN API error {status}: {body}")
            }
            LhdnError::Token(msg) => write!(f, "LHDN token error: {msg}"),
            LhdnError::Serialization(e) => write!(f, "LHDN serialization error: {e}"),
        }
    }
}

impl std::error::Error for LhdnError {}

impl From<reqwest::Error> for LhdnError {
    fn from(err: reqwest::Error) -> Self {
        LhdnError::Network(err)
    }
}

impl From<serde_json::Error> for LhdnError {
    fn from(err: serde_json::Error) -> Self {
        LhdnError::Serialization(err)
    }
}

impl LhdnClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn token_key(base_url: &str, client_id: &str) -> String {
        format!("{base_url}|{client_id}")
    }

    async fn access_token(&self, settings: &LhdnSettings) -> Result<String, LhdnError> {
        let key = Self::token_key(&settings.base_url, &settings.client_id);
        let now = Instant::now();

        {
            let tokens = self.tokens.lock().unwrap();
            if let Some(entry) = tokens.get(&key) {
                if entry.expires_at > now + TOKEN_MARGIN {
                    return Ok(entry.access_token.clone());
                }
            }
        }

        let token_url = format!("{}/connect/token", settings.base_url.trim_end_matches('/'));
        info!("fetching LHDN access token from {}", token_url);

        let response = self
            .http
            .post(&token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &settings.client_id),
                ("client_secret", &settings.client_secret),
                ("scope", "InvoicingAPI"),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            error!("LHDN token endpoint returned {}: {}", status, body);
            return Err(LhdnError::Token(format!("HTTP {status}: {body}")));
        }

        let token: TokenResponse = serde_json::from_str(&body).map_err(|e| {
            error!("failed to parse LHDN token response: {}", e);
            LhdnError::Token(format!("invalid token response: {e}"))
        })?;

        let expires_in = token.expires_in.max(60);
        let expires_at = now + Duration::from_secs(expires_in);

        {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.insert(
                key,
                TokenEntry {
                    access_token: token.access_token.clone(),
                    expires_at,
                },
            );
        }

        Ok(token.access_token)
    }

    pub async fn submit_document(
        &self,
        settings: &LhdnSettings,
        payload: SubmitPayload,
    ) -> Result<SubmitResponse, LhdnError> {
        let token = self.access_token(settings).await?;
        let url = format!(
            "{}/api/v1.0/documents/submit",
            settings.base_url.trim_end_matches('/')
        );

        info!("submitting document to LHDN at {}", url);
        let response = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            error!("LHDN submit returned {}: {}", status, body);
            return Err(LhdnError::Api { status, body });
        }

        let parsed: SubmitResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }

    pub async fn get_document_status(
        &self,
        settings: &LhdnSettings,
        uuid: &str,
    ) -> Result<DocumentStatusResponse, LhdnError> {
        let token = self.access_token(settings).await?;
        let url = format!(
            "{}/api/v1.0/documents/status/{uuid}",
            settings.base_url.trim_end_matches('/')
        );

        info!("fetching LHDN document status from {}", url);
        let response = self.http.get(&url).bearer_auth(token).send().await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            error!("LHDN status returned {}: {}", status, body);
            return Err(LhdnError::Api { status, body });
        }

        let parsed: DocumentStatusResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    }
}

impl Default for LhdnClient {
    fn default() -> Self {
        Self::new()
    }
}
