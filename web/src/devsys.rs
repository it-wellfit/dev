use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub id: String,
    pub category: String,
    pub key: String,
    pub value: String,
    pub config_type: String,
    pub description: String,
    pub is_sensitive: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub role: String,
    pub resource: String,
    pub action: String,
    pub description: String,
}
