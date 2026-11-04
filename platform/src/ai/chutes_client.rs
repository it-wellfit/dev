use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChutesError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChutesConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_ms: u64,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
}

pub struct ChutesClient {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
    #[allow(dead_code)]
    timeout: Duration,
    temperature: f32,
    top_p: f32,
    max_tokens: Option<u32>,
    system_prompt: Option<String>,
}

impl ChutesClient {
    pub fn new(config: ChutesConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.api_key))
                        .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
                );
                headers.insert(
                    "X-API-Key",
                    reqwest::header::HeaderValue::from_str(&config.api_key)
                        .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
                );
                headers
            })
            .build()
            .unwrap();

        Self {
            client,
            api_url: config.api_url,
            api_key: config.api_key,
            model: config.model,
            timeout: Duration::from_millis(config.timeout_ms),
            temperature: config.temperature,
            top_p: config.top_p,
            max_tokens: config.max_tokens,
            system_prompt: config.system_prompt,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, ChutesError> {
        let url = format!("{}/v1/chat/completions", self.api_url);

        let mut messages = vec![json!({
            "role": "system",
            "content": self.system_prompt.clone().unwrap_or_else(|| "You are a helpful AI assistant.".to_string())
        })];

        messages.push(json!({
            "role": "user",
            "content": prompt
        }));

        let payload = json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "max_tokens": self.max_tokens,
            "stream": false
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ChutesError::Api(error_text));
        }

        let chunks_response: ChunksResponse = response.json().await?;
        Ok(chunks_response.choices[0].message.content.clone())
    }

    pub async fn health_check(&self) -> Result<bool, ChutesError> {
        let url = format!("{}/v1/models", self.api_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn list_models(&self) -> Result<Vec<String>, ChutesError> {
        let url = format!("{}/v1/models", self.api_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ChutesError::Api("Failed to list models".to_string()));
        }

        let models_response: ChutesModelsResponse = response.json().await?;
        Ok(models_response
            .data
            .iter()
            .map(|model| model.id.clone())
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChunksResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    index: i32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChutesModelsResponse {
    data: Vec<ModelInfo>,
    object: String,
    created: i64,
    model: String,
}
