//! AI Integration Module for Flaps ERP
//!
//! This module provides comprehensive AI integration capabilities including:
//! - Multiple AI model clients (Ollama, Chutes, Inference.net)
//! - AI decision gating framework for intelligent transaction processing
//! - Model orchestration and load balancing
//! - Free, open-source AI implementation
//!
//! All AI components are designed to work together seamlessly to provide
//! intelligent decision support for financial transactions.

pub mod chutes_client;
pub mod decision_gating;
pub mod inference_client;
pub mod model_orchestrator;
pub mod ollama_client;

pub use chutes_client::ChutesClient;
pub use decision_gating::{AIContext, AIDecision, AIDecisionGating, DecisionGatingStrategy};
pub use inference_client::InferenceClient;
pub use model_orchestrator::ModelOrchestrator;
pub use ollama_client::OllamaClient;

pub use chutes_client::ChutesConfig;
pub use decision_gating::{AIConfidenceLevel, Action, DecisionType};
pub use inference_client::InferenceConfig;
/// Re-export important types for easy access
pub use ollama_client::OllamaConfig;
