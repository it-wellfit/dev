use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InferenceError {
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
pub struct InferenceConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_ms: u64,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
}

pub struct InferenceClient {
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

impl InferenceClient {
    pub fn new(config: InferenceConfig) -> Self {
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
                headers.insert(
                    "X-Requested-With",
                    reqwest::header::HeaderValue::from_static("AI-Interface"),
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

    pub async fn generate(&self, prompt: &str) -> Result<String, InferenceError> {
        let url = format!("{}/v1/generate", self.api_url);

        let mut request_data = json!({
            "model": self.model,
            "prompt": prompt,
            "temperature": self.temperature,
            "top_p": self.top_p,
            "max_tokens": self.max_tokens,
            "stream": false
        });

        if let Some(sys_prompt) = &self.system_prompt {
            request_data["system"] = json!(sys_prompt);
        }

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(InferenceError::Api(error_text));
        }

        let inference_response: InferenceResponse = response.json().await?;
        Ok(inference_response.text)
    }

    pub async fn health_check(&self) -> Result<bool, InferenceError> {
        let url = format!("{}/v1/models", self.api_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn list_models(&self) -> Result<Vec<String>, InferenceError> {
        let url = format!("{}/v1/models", self.api_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(InferenceError::Api("Failed to list models".to_string()));
        }

        let models_response: InferenceModelsResponse = response.json().await?;
        Ok(models_response
            .data
            .iter()
            .map(|model| model.id.clone())
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    text: String,
    done: bool,
    total_duration: i64,
    load_duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InferenceModelsResponse {
    data: Vec<ModelInfo>,
    object: String,
    created: i64,
}
