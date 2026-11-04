use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub date: String,
    pub status: String,
    pub created_by: String,
}
