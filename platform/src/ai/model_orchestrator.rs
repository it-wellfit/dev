use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ai::chutes_client::ChutesClient;
use crate::ai::decision_gating::{AIContext, AIDecision, AIDecisionGating};
use crate::ai::inference_client::InferenceClient;
use crate::ai::ollama_client::OllamaClient;

pub struct ModelOrchestrator {
    pub ollama_client: Arc<OllamaClient>,
    pub chutes_client: Option<Arc<ChutesClient>>,
    pub inference_client: Option<Arc<InferenceClient>>,
    pub decision_gating: Arc<AIDecisionGating>,
    pub active_models: Arc<RwLock<HashMap<String, ModelStatus>>>,
    pub load_balancer: Arc<RwLock<LoadBalancer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub model_name: String,
    pub provider: ProviderType,
    pub is_healthy: bool,
    pub response_time: i64,
    pub error_rate: f32,
    pub last_check: i64,
    pub capacity: f32,
    pub current_load: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    Ollama,
    Chutes,
    Inference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancer {
    pub round_robin_index: usize,
    pub health_check_interval: i64,
    pub min_health_score: f32,
    pub max_capacity_utilization: f32,
}

impl ModelOrchestrator {
    pub fn new(
        ollama_client: Arc<OllamaClient>,
        chutes_client: Option<Arc<ChutesClient>>,
        inference_client: Option<Arc<InferenceClient>>,
        decision_gating: Arc<AIDecisionGating>,
    ) -> Self {
        Self {
            ollama_client,
            chutes_client,
            inference_client,
            decision_gating,
            active_models: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(RwLock::new(LoadBalancer {
                round_robin_index: 0,
                health_check_interval: 60000, // 1 minute in milliseconds
                min_health_score: 0.7,
                max_capacity_utilization: 0.9,
            })),
        }
    }

    pub async fn start_monitoring(&self) {
        let active_models = self.active_models.clone();
        let load_balancer = self.load_balancer.clone();
        let ollama_client = self.ollama_client.clone();
        let chutes_client = self.chutes_client.clone();
        let inference_client = self.inference_client.clone();

        // Initialize model statuses
        let mut models = active_models.write().await;

        // Add Ollama model
        models.insert(
            "ollama".to_string(),
            ModelStatus {
                model_name: "ollama".to_string(),
                provider: ProviderType::Ollama,
                is_healthy: true,
                response_time: 0,
                error_rate: 0.0,
                last_check: chrono::Utc::now().timestamp(),
                capacity: 1.0,
                current_load: 0.0,
            },
        );

        // Add Chutes model if available
        if chutes_client.is_some() {
            models.insert(
                "chutes".to_string(),
                ModelStatus {
                    model_name: "chutes".to_string(),
                    provider: ProviderType::Chutes,
                    is_healthy: true,
                    response_time: 0,
                    error_rate: 0.0,
                    last_check: chrono::Utc::now().timestamp(),
                    capacity: 1.0,
                    current_load: 0.0,
                },
            );
        }

        // Add Inference model if available
        if inference_client.is_some() {
            models.insert(
                "inference".to_string(),
                ModelStatus {
                    model_name: "inference".to_string(),
                    provider: ProviderType::Inference,
                    is_healthy: true,
                    response_time: 0,
                    error_rate: 0.0,
                    last_check: chrono::Utc::now().timestamp(),
                    capacity: 1.0,
                    current_load: 0.0,
                },
            );
        }

        // Start health monitoring task
        let active_models = active_models.clone();
        let load_balancer = load_balancer.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await; // Check every 5 seconds

                // Update health status for each model
                for (model_name, status) in active_models.read().await.iter() {
                    let health = match status.provider {
                        ProviderType::Ollama => {
                            ModelOrchestrator::check_ollama_health(&ollama_client).await
                        }
                        ProviderType::Chutes => {
                            if let Some(chutes_client) = &chutes_client {
                                ModelOrchestrator::check_chutes_health(chutes_client).await
                            } else {
                                Ok(false)
                            }
                        }
                        ProviderType::Inference => {
                            if let Some(inference_client) = &inference_client {
                                ModelOrchestrator::check_inference_health(inference_client).await
                            } else {
                                Ok(false)
                            }
                        }
                    };

                    // Update model status based on health check
                    let mut models = active_models.write().await;
                    if let Some(model_status) = models.get_mut(model_name) {
                        model_status.is_healthy = health.unwrap_or(false);
                        model_status.last_check = chrono::Utc::now().timestamp();
                    }
                }

                // Update load balancer
                Self::update_load_balancer(&load_balancer, &active_models).await;
            }
        });
    }

    pub async fn make_intelligent_decision(
        &self,
        transaction: &auxbridge_core::ledger::transaction::Transaction,
        items: &[crate::ai::decision_gating::TransactionItem],
        context: AIContext,
    ) -> Result<AIDecision, Box<dyn std::error::Error + Send + Sync>> {
        // Get healthy models from load balancer
        let healthy_models = self.get_healthy_models().await;

        if healthy_models.is_empty() {
            return Err("No healthy AI models available".into());
        }

        // Make decision using the selected model
        let decision = self
            .decision_gating
            .make_decision(transaction, items, context)
            .await?;

        // Update model load based on decision
        self.update_model_load(&decision.ai_model_used).await;

        Ok(decision)
    }

    async fn get_healthy_models(&self) -> Vec<String> {
        let models = self.active_models.read().await;
        let load_balancer = self.load_balancer.read().await;

        let mut healthy_models: Vec<String> = models
            .iter()
            .filter(|(_, status)| {
                status.is_healthy
                    && status.error_rate < 0.2
                    && status.current_load < load_balancer.max_capacity_utilization
            })
            .map(|(_, status)| status.model_name.clone())
            .collect();

        // Apply round-robin selection if multiple healthy models
        if healthy_models.len() > 1 {
            let round_robin_index = load_balancer.round_robin_index % healthy_models.len();
            let selected = healthy_models.remove(round_robin_index);
            healthy_models.insert(0, selected);
        }

        healthy_models
    }

    async fn update_model_load(&self, model_name: &str) {
        let mut models = self.active_models.write().await;
        let mut load_balancer = self.load_balancer.write().await;

        if let Some(model_status) = models.get_mut(model_name) {
            model_status.current_load = (model_status.current_load + 0.1).min(1.0);

            // Update round-robin index
            load_balancer.round_robin_index =
                (load_balancer.round_robin_index + 1) % (models.len() + 1);
        }
    }

    async fn check_ollama_health(
        ollama_client: &OllamaClient,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ollama_client.health_check().await.unwrap_or(false))
    }

    async fn check_chutes_health(
        chutes_client: &ChutesClient,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(chutes_client.health_check().await.unwrap_or(false))
    }

    async fn check_inference_health(
        inference_client: &InferenceClient,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(inference_client.health_check().await.unwrap_or(false))
    }

    async fn update_load_balancer(
        load_balancer: &Arc<RwLock<LoadBalancer>>,
        active_models: &Arc<RwLock<HashMap<String, ModelStatus>>>,
    ) {
        let lb = load_balancer.write().await;
        let mut models = active_models.write().await;
        // Calculate load distribution based on model performance
        let total_load: f32 = models.values().map(|s| s.current_load).sum();

        // Rebalance if total load exceeds threshold
        if total_load > lb.max_capacity_utilization * models.len() as f32 * 0.8 {
            for status in models.values_mut() {
                if status.current_load > lb.max_capacity_utilization {
                    status.current_load = (status.current_load * 0.8).max(0.1);
                }
            }
        }
    }
}

// Re-export important types for easy access
pub use ModelOrchestrator as ModelOrchestratorType;
pub use ModelStatus as ModelStatusType;
pub use ProviderType as ProviderTypeType;
