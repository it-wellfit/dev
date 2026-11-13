use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_ms: u64,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<u32>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
    #[allow(dead_code)]
    timeout: Duration,
    temperature: f32,
    top_p: f32,
    max_tokens: Option<u32>,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap();

        Self {
            client,
            base_url: config.base_url,
            model: config.model,
            timeout: Duration::from_millis(config.timeout_ms),
            temperature: config.temperature,
            top_p: config.top_p,
            max_tokens: config.max_tokens,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.base_url);

        let payload = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            // qwen3.5:4b 為 thinking model：不加 "think": false 時，實際輸出
            // 落在 thinking 欄位而 response 為空（2026-11-13 實證：schema
            // 驗證 EOF）。與 rerank 模組同一修法。
            "think": false,
            "options": {
                "temperature": self.temperature,
                "top_p": self.top_p,
                "num_predict": self.max_tokens,
            }
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
            return Err(OllamaError::Api(error_text));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }

    pub async fn health_check(&self) -> Result<bool, OllamaError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client.get(&url).send().await?;

        Ok(response.status().is_success())
    }

    pub async fn list_models(&self) -> Result<Vec<String>, OllamaError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(OllamaError::Api("Failed to list models".to_string()));
        }

        let tags_response: TagsResponse = response.json().await?;
        Ok(tags_response
            .models
            .iter()
            .map(|m| m.name.clone())
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    total_duration: i64,
    load_duration: i64,
    prompt_eval_count: i64,
    prompt_eval_duration: i64,
    eval_count: i64,
    eval_duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    name: String,
    model: String,
    modified_at: String,
    size: i64,
    digest: String,
    details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelDetails {
    parent_model: String,
    format: String,
    family: String,
    families: Vec<String>,
    parameter_size: String,
    quantization_level: String,
}
