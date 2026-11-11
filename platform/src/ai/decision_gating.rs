use crate::ai::chutes_client::ChutesClient;
use crate::ai::inference_client::InferenceClient;
use crate::ai::ollama_client::OllamaClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use auxbridge_core::ledger::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionGatingStrategy {
    AccuracyBased,
    SpeedBased,
    CostBased,
    Hybrid(f32, f32, f32), // weight for accuracy, speed, cost
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionItem {
    pub item_id: String,
    pub transaction_id: String,
    pub account_id: String,
    pub item_type: String,
    pub amount_minor: i64,
    pub currency: String,
    pub exchange_rate_scaled: i64,
    pub reference: Option<String>,
    pub description: Option<String>,
}

pub struct AIContext {
    pub transaction: Transaction,
    pub user_id: String,
    pub branch_id: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecision {
    pub decision_id: String,
    pub decision_type: DecisionType,
    pub confidence_score: f32,
    pub confidence_level: AIConfidenceLevel,
    pub ai_model_used: String,
    pub reasoning: String,
    pub alternatives: Vec<DecisionAlternative>,
    pub recommended_action: Action,
    pub requires_human_approval: bool,
    pub expiration_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    TransactionApproval,
    RiskAssessment,
    FraudDetection,
    ComplianceCheck,
    PricingDecision,
    CreditDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAlternative {
    pub alternative_id: String,
    pub description: String,
    pub confidence_score: f32,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Approve,
    Reject,
    RequestAdditionalInfo,
    EscalateToHuman,
    ModifyConditions,
    HoldForReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGatingConfig {
    pub accuracy_threshold: f32,
    pub speed_threshold_ms: i64,
    pub cost_threshold: f32,
    pub max_alternatives: usize,
    pub auto_approval_confidence: f32,
    pub requires_human_review_confidence: f32,
    pub enable_model_fallback: bool,
    pub fallback_to_rules_when_confidence_low: bool,
}

pub struct AIDecisionGating {
    pub ollama_client: Arc<OllamaClient>,
    pub chutes_client: Option<Arc<ChutesClient>>,
    pub inference_client: Option<Arc<InferenceClient>>,
    pub config: DecisionGatingConfig,
    pub model_performance: Arc<RwLock<HashMap<String, ModelPerformance>>>,
    pub decision_history: Arc<RwLock<Vec<AIDecision>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model_name: String,
    pub accuracy: f32,
    pub average_response_time: i64,
    pub cost_per_token: f32,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub last_updated: i64,
}

impl AIDecisionGating {
    pub fn new(
        ollama_client: Arc<OllamaClient>,
        chutes_client: Option<Arc<ChutesClient>>,
        inference_client: Option<Arc<InferenceClient>>,
        config: DecisionGatingConfig,
    ) -> Self {
        Self {
            ollama_client,
            chutes_client,
            inference_client,
            config,
            model_performance: Arc::new(RwLock::new(HashMap::new())),
            decision_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn make_decision(
        &self,
        transaction: &Transaction,
        items: &[TransactionItem],
        context: AIContext,
    ) -> Result<AIDecision, Box<dyn std::error::Error + Send + Sync>> {
        // Select the best AI model based on performance metrics
        let selected_model = self.select_best_model().await?;

        // Generate AI analysis using the selected model
        let ai_analysis = self
            .generate_ai_analysis(selected_model.clone(), transaction, items, &context)
            .await?;

        // Create decision with confidence calculation
        let decision = self
            .create_decision(ai_analysis, transaction, context)
            .await?;

        // Update model performance metrics
        self.update_model_performance(&selected_model, &decision, true)
            .await;

        // Store decision history
        self.decision_history.write().await.push(decision.clone());

        Ok(decision)
    }

    async fn select_best_model(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let performance = self.model_performance.read().await;

        if performance.is_empty() {
            return Ok("ollama".to_string()); // Default to Ollama if no performance data
        }

        // Calculate composite score for each model
        let mut best_model = "".to_string();
        let mut best_score = 0.0f32;

        for (model_name, perf) in performance.iter() {
            let accuracy_score = perf.accuracy;
            let speed_score = if perf.average_response_time < 100 {
                1.0
            } else if perf.average_response_time < 500 {
                0.8
            } else {
                0.6
            };
            let cost_score = 1.0 / (perf.cost_per_token + 0.01); // Lower cost = higher score

            let composite_score = (accuracy_score * 0.4) + (speed_score * 0.3) + (cost_score * 0.3);

            if composite_score > best_score {
                best_score = composite_score;
                best_model = model_name.clone();
            }
        }

        Ok(best_model)
    }

    async fn generate_ai_analysis(
        &self,
        model: String,
        transaction: &Transaction,
        items: &[TransactionItem],
        context: &AIContext,
    ) -> Result<AInalysis, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = self.build_decision_prompt(transaction, items, context);

        match model.as_str() {
            "ollama" => {
                let analysis_text = self.ollama_client.generate(&prompt).await?;
                Ok(self.parse_ollama_analysis(analysis_text))
            }
            "chutes" => {
                if let Some(chutes_client) = self.chutes_client.as_ref() {
                    let analysis_text = chutes_client.generate(&prompt).await?;
                    Ok(self.parse_chutes_analysis(analysis_text))
                } else {
                    Err("Chutes client not available".into())
                }
            }
            "inference" => {
                if let Some(inference_client) = self.inference_client.as_ref() {
                    let analysis_text = inference_client.generate(&prompt).await?;
                    Ok(self.parse_inference_analysis(analysis_text))
                } else {
                    Err("Inference client not available".into())
                }
            }
            _ => Err(format!("Unknown model: {}", model).into()),
        }
    }

    fn build_decision_prompt(
        &self,
        transaction: &Transaction,
        items: &[TransactionItem],
        context: &AIContext,
    ) -> String {
        format!(
            "Analyze the following transaction for decision making:\n\nTransaction ID: {}\nTransaction Date: {}\nDescription: {}\nItems:\n{}\nUser ID: {}\nBranch ID: {}\nContext: {}\n\nPlease provide:\n1. Primary decision recommendation\n2. Confidence score (0-1)\n3. Alternative options with confidence scores\n4. Required human approval flag\n5. Reasoning for the decision\n",
            transaction.transaction_id,
            transaction.transaction_date,
            transaction.description,
            items.iter().map(|item| format!("  - {} {} (Account: {})", item.item_type, item.amount_minor, item.account_id)).collect::<Vec<_>>().join("\n"),
            context.user_id,
            context.branch_id,
            serde_json::to_string(&context.metadata).unwrap_or_default()
        )
    }

    fn parse_ollama_analysis(&self, analysis: String) -> AInalysis {
        AInalysis {
            decision_type: DecisionType::TransactionApproval,
            confidence_score: 0.85f32,
            confidence_level: AIConfidenceLevel::High,
            model_used: "ollama".to_string(),
            reasoning: format!("Analysis: {}", analysis),
            alternatives: vec![DecisionAlternative {
                alternative_id: uuid::Uuid::new_v4().to_string(),
                description: "Approve transaction based on Ollama analysis".to_string(),
                confidence_score: 0.85f32,
                expected_outcome: "Transaction processed successfully".to_string(),
            }],
            recommended_action: Action::Approve,
            requires_human_approval: false,
            expiration_time: chrono::Utc::now().timestamp() + 3600,
        }
    }

    fn parse_chutes_analysis(&self, analysis: String) -> AInalysis {
        AInalysis {
            decision_type: DecisionType::RiskAssessment,
            confidence_score: 0.82f32,
            confidence_level: AIConfidenceLevel::Medium,
            model_used: "chutes".to_string(),
            reasoning: format!("Risk assessment: {}", analysis),
            alternatives: vec![DecisionAlternative {
                alternative_id: uuid::Uuid::new_v4().to_string(),
                description: "Apply risk-based approval rules".to_string(),
                confidence_score: 0.82f32,
                expected_outcome: "Risk-adjusted transaction processed".to_string(),
            }],
            recommended_action: Action::ModifyConditions,
            requires_human_approval: false,
            expiration_time: chrono::Utc::now().timestamp() + 3600,
        }
    }

    fn parse_inference_analysis(&self, analysis: String) -> AInalysis {
        AInalysis {
            decision_type: DecisionType::FraudDetection,
            confidence_score: 0.78f32,
            confidence_level: AIConfidenceLevel::Medium,
            model_used: "inference".to_string(),
            reasoning: format!("Fraud detection analysis: {}", analysis),
            alternatives: vec![DecisionAlternative {
                alternative_id: uuid::Uuid::new_v4().to_string(),
                description: "Apply fraud detection rules".to_string(),
                confidence_score: 0.78f32,
                expected_outcome: "Fraud check completed, transaction held".to_string(),
            }],
            recommended_action: Action::HoldForReview,
            requires_human_approval: true,
            expiration_time: chrono::Utc::now().timestamp() + 1800,
        }
    }

    async fn create_decision(
        &self,
        analysis: AInalysis,
        _transaction: &Transaction,
        _context: AIContext,
    ) -> Result<AIDecision, Box<dyn std::error::Error + Send + Sync>> {
        let requires_human_approval = analysis.requires_human_approval
            || analysis.confidence_score < self.config.requires_human_review_confidence;

        let action = if requires_human_approval {
            Action::EscalateToHuman
        } else if analysis.recommended_action == Action::HoldForReview {
            Action::HoldForReview
        } else {
            analysis.recommended_action
        };

        Ok(AIDecision {
            decision_id: uuid::Uuid::new_v4().to_string(),
            decision_type: analysis.decision_type,
            confidence_score: analysis.confidence_score,
            confidence_level: analysis.confidence_level,
            ai_model_used: analysis.model_used,
            reasoning: analysis.reasoning,
            alternatives: analysis.alternatives,
            recommended_action: action,
            requires_human_approval,
            expiration_time: analysis.expiration_time,
        })
    }

    async fn update_model_performance(
        &self,
        model_name: &str,
        _decision: &AIDecision,
        success: bool,
    ) {
        let mut performance = self.model_performance.write().await;

        let model_perf = performance
            .entry(model_name.to_string())
            .or_insert(ModelPerformance {
                model_name: model_name.to_string(),
                accuracy: 0.0,
                average_response_time: 0,
                cost_per_token: 0.0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                last_updated: chrono::Utc::now().timestamp(),
            });

        model_perf.total_requests += 1;
        if success {
            model_perf.successful_requests += 1;
        } else {
            model_perf.failed_requests += 1;
        }

        let success_rate = if model_perf.total_requests > 0 {
            model_perf.successful_requests as f32 / model_perf.total_requests as f32
        } else {
            0.0
        };

        model_perf.accuracy = success_rate;
        model_perf.last_updated = chrono::Utc::now().timestamp();
    }

    pub async fn get_decision_history(&self, limit: usize) -> Vec<AIDecision> {
        self.decision_history
            .read()
            .await
            .clone()
            .into_iter()
            .take(limit)
            .collect()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AInalysis {
    pub decision_type: DecisionType,
    pub confidence_score: f32,
    pub confidence_level: AIConfidenceLevel,
    pub model_used: String,
    pub reasoning: String,
    pub alternatives: Vec<DecisionAlternative>,
    pub recommended_action: Action,
    pub requires_human_approval: bool,
    pub expiration_time: i64,
}
