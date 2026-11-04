use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_transactions: i32,
    pub total_volume: f64,
    pub active_accounts: i32,
    pub pending_transactions: i32,
    pub avg_processing_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub ollama_status: String,
    pub chutes_status: String,
    pub inference_status: String,
    pub database_status: String,
    pub overall_health: String,
}
