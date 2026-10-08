// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct AiConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
}

impl AiConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            base_url: std::env::var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string()),
            model: std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string()),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }
}

#[derive(Clone)]
pub struct AiClient {
    http: Client,
    config: AiConfig,
}

impl AiClient {
    pub fn is_configured(&self) -> bool {
        self.config.is_configured()
    }
}

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ToolDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug)]
pub enum AiError {
    NotConfigured,
    Network(reqwest::Error),
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
    Serialization(serde_json::Error),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::NotConfigured => write!(f, "AI provider is not configured"),
            AiError::Network(e) => write!(f, "AI network error: {e}"),
            AiError::Api { status, body } => write!(f, "AI API error {status}: {body}"),
            AiError::Serialization(e) => write!(f, "AI serialization error: {e}"),
        }
    }
}

impl std::error::Error for AiError {}

impl From<reqwest::Error> for AiError {
    fn from(err: reqwest::Error) -> Self {
        AiError::Network(err)
    }
}

impl From<serde_json::Error> for AiError {
    fn from(err: serde_json::Error) -> Self {
        AiError::Serialization(err)
    }
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            http: Client::new(),
            config,
        }
    }

    pub async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message, AiError> {
        if !self.config.is_configured() {
            return Err(AiError::NotConfigured);
        }

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );
        info!("sending chat completion request to {}", url);

        let request_body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            tools,
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(self.config.api_key.as_deref().unwrap_or(""))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            error!("AI API returned {}: {}", status, body);
            return Err(AiError::Api { status, body });
        }

        let parsed: ChatCompletionResponse = serde_json::from_str(&body)?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| AiError::Api {
                status: reqwest::StatusCode::OK,
                body: "empty choices".to_string(),
            })
    }
}

impl Default for AiClient {
    fn default() -> Self {
        Self::new(AiConfig::from_env())
    }
}
