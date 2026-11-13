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
    /// 結構化解碼約束（Ollama `format` 參數；JSON Schema 物件）。
    /// 2026 SOTA 共識：「停止在提示詞裡懇求，約束解碼器」——`format` 使
    /// 非法 JSON 在解碼層即不可能（llmconfigurator.com 2026-07；AscentCore
    /// 2026-04 小型模型評測）。None＝純提示詞模式（不建議用於抽取）。
    pub format: Option<serde_json::Value>,
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
    format: Option<serde_json::Value>,
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
            format: config.format,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, OllamaError> {
        let url = format!("{}/api/generate", self.base_url);

        let mut payload = json!({
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

        // 結構化解碼約束：format schema 存在時附加（解碼層強制合法 JSON）。
        if let Some(schema) = &self.format {
            payload["format"] = schema.clone();
        }

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

    /// 已安裝模型清單（治理介面的選項來源）。不可達→None（呼叫端降級）。
    pub async fn list_models(&self) -> Option<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await.ok()?;
        if !response.status().is_success() {
            return None;
        }
        let value: serde_json::Value = response.json().await.ok()?;
        Some(
            value["models"]
                .as_array()?
                .iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect(),
        )
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
#[allow(dead_code)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct ModelInfo {
    name: String,
    model: String,
    modified_at: String,
    size: i64,
    digest: String,
    details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct ModelDetails {
    parent_model: String,
    format: String,
    family: String,
    families: Vec<String>,
    parameter_size: String,
    quantization_level: String,
}
