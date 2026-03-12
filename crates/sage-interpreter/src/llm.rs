//! LLM backend for `infer` expressions.
//!
//! This module provides the interface to OpenAI-compatible LLM APIs
//! for processing `infer` expressions in Sage programs.

use crate::error::{RuntimeError, RuntimeResult};
use sage_types::Span;
use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for the LLM backend.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Base URL for the API (default: `OpenAI`).
    pub api_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Model to use (default: gpt-4o-mini).
    pub model: String,
    /// Maximum tokens in response.
    pub max_tokens: u32,
    /// Temperature for sampling.
    pub temperature: f32,
}

impl LlmConfig {
    /// Create a new LLM config from environment variables.
    ///
    /// Uses:
    /// - `SAGE_LLM_URL` (default: `https://api.openai.com/v1`)
    /// - `SAGE_API_KEY` (required for OpenAI, optional for local LLMs like Ollama)
    /// - `SAGE_MODEL` (default: `gpt-4o-mini`)
    ///
    /// For local LLMs (Ollama, etc.), set `SAGE_LLM_URL` to the local endpoint
    /// (e.g., `http://192.168.0.11:11434/v1`) and optionally set `SAGE_MODEL`
    /// to the model name (e.g., `llama3`).
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let api_url = env::var("SAGE_LLM_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        // API key is required for OpenAI, but optional for local LLMs
        let api_key = env::var("SAGE_API_KEY").unwrap_or_default();
        let is_local = !api_url.contains("openai.com");

        // If using OpenAI and no API key, return None
        if !is_local && api_key.is_empty() {
            return None;
        }

        // Default model: gpt-4o-mini for OpenAI, llama3.2 for local
        let default_model = if is_local { "llama3.2" } else { "gpt-4o-mini" };
        let model = env::var("SAGE_MODEL").unwrap_or_else(|_| default_model.to_string());

        Some(Self {
            api_url,
            api_key,
            model,
            max_tokens: 1024,
            temperature: 0.7,
        })
    }

    /// Create a mock config for testing.
    #[must_use]
    pub fn mock() -> Self {
        Self {
            api_url: "mock://".to_string(),
            api_key: "mock-key".to_string(),
            model: "mock-model".to_string(),
            max_tokens: 100,
            temperature: 0.0,
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(Self::mock)
    }
}

/// The LLM client for making inference calls.
#[derive(Debug, Clone)]
pub struct LlmClient {
    config: LlmConfig,
    client: reqwest::Client,
}

impl LlmClient {
    /// Create a new LLM client.
    #[must_use]
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Check if this is a mock client (for testing).
    #[must_use]
    pub fn is_mock(&self) -> bool {
        self.config.api_url.starts_with("mock://")
    }

    /// Call the LLM with a prompt and return the response.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails, the API returns an error,
    /// or the response cannot be parsed.
    pub async fn infer(&self, prompt: &str, span: &Span) -> RuntimeResult<String> {
        // If mock, return a placeholder response
        if self.is_mock() {
            return Ok(format!("[Mock LLM response to: {prompt}]"));
        }

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
        };

        let url = format!("{}/chat/completions", self.config.api_url);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Only add Authorization header if API key is present
        if !self.config.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = req
            .json(&request)
            .send()
            .await
            .map_err(|e| RuntimeError::llm_error(format!("HTTP error: {e}"), span))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(RuntimeError::llm_error(
                format!("API error {status}: {body}"),
                span,
            ));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| RuntimeError::llm_error(format!("JSON parse error: {e}"), span))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| RuntimeError::llm_error("No response from LLM", span))
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new(LlmConfig::default())
    }
}

/// OpenAI-compatible chat request.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// A message in a chat conversation.
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible chat response.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// A choice in a chat response.
#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_mock() {
        let config = LlmConfig::mock();
        assert_eq!(config.api_url, "mock://");
        assert!(LlmClient::new(config).is_mock());
    }

    #[tokio::test]
    async fn mock_infer() {
        let client = LlmClient::new(LlmConfig::mock());
        let span = Span::dummy();
        let result = client.infer("Hello", &span).await.unwrap();
        assert!(result.contains("Mock LLM response"));
    }
}
