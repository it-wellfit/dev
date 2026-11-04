use std::collections::HashMap; use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export important types
pub use crate::ai::ollama_client::OllamaClient;
pub use crate::ai::chutes_client::ChutesClient;
pub use crate::ai::inference_client::InferenceClient;
pub use crate::ai::decision_gating::{AIDecisionGating, AIContext};

#[derive(Debug, Clone)]
pub struct ApplicationState {
    // AI clients
    pub ollama_client: Arc<OllamaClient>,
    pub chutes_client: Option<Arc<ChutesClient>>,
    pub inference_client: Option<Arc<InferenceClient>>,
    
    // Decision gating
    pub decision_gating: Arc<AIDecisionGating>,
    
    // Model orchestrator
    pub model_orchestrator: Arc<ModelOrchestrator>,
    
    // Connection pool
    pub connection_pool: Arc<RwLock<ConnectionPool>>,
    
    // Cache
    pub cache: Arc<CacheManager>,
}

#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub connections: Vec<DatabaseConnection>,
    pub max_connections: usize,
    pub current_connections: usize,
}

#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct CacheManager {
    pub redis_client: Arc<RedisClient>,
    pub cache_ttl: u64,
    pub max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct RedisClient {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: i32,
}

// Re-export main application state type
pub type AppState = Arc<ApplicationState>;

impl ApplicationState {
    pub fn new(
        ollama_client: Arc<OllamaClient>,
        chutes_client: Option<Arc<ChutesClient>>,
        inference_client: Option<Arc<InferenceClient>>,
    ) -> Self {
        // Initialize connection pool
        let connection_pool = Arc::new(RwLock::new(ConnectionPool {
            connections: vec![
                DatabaseConnection {
                    id: "conn_001".to_string(),
                    host: "localhost".to_string(),
                    port: 5432,
                    database: "flaps_erp".to_string(),
                    username: "postgres".to_string(),
                    is_active: true,
                },
            ],
            max_connections: 10,
            current_connections: 1,
        }));
        
        // Initialize cache
        let cache = Arc::new(CacheManager {
            redis_client: Arc::new(RedisClient {
                host: "localhost".to_string(),
                port: 6379,
                password: None,
                db: 0,
            }),
            cache_ttl: 3600, // 1 hour
            max_cache_size: 10000,
        });
        
        // Initialize model orchestrator
        let mut models = HashMap::new();
        models.insert("ollama".to_string(), ModelStatus {
            model_name: "ollama".to_string(),
            provider: ProviderType::Ollama,
            is_healthy: true,
            response_time: 0,
            error_rate: 0.0,
            last_check: 0,
            capacity: 1.0,
            current_load: 0.0,
        });
        
        // Initialize decision gating configuration
        let decision_gating_config = DecisionGatingConfig {
            accuracy_threshold: 0.9,
            speed_threshold_ms: 1000,
            cost_threshold: 0.1,
            max_alternatives: 5,
            auto_approval_confidence: 0.95,
            requires_human_review_confidence: 0.7,
            enable_model_fallback: true,
            fallback_to_rules_when_confidence_low: true,
        };
        
        // Initialize decision gating
        let decision_gating = Arc::new(AIDecisionGating::new(
            ollama_client.clone(),
            chutes_client.clone(),
            inference_client.clone(),
            decision_gating_config,
        ));
        
        // Initialize model orchestrator
        let model_orchestrator = Arc::new(ModelOrchestrator::new(
            ollama_client,
            chutes_client,
            inference_client,
            decision_gating,
        ));
        
        // Start monitoring
        let orchestrator = model_orchestrator.clone();
        tokio::spawn(async move {
            orchestrator.start_monitoring().await;
        });
        
        Self {
            ollama_client,
            chutes_client,
            inference_client,
            decision_gating,
            model_orchestrator,
            connection_pool,
            cache,
        }
    }
}

use std::collections::HashMap;
use crate::ai::model_orchestrator::{ModelOrchestrator, ModelStatus, ProviderType};





