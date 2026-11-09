use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// ERP Configuration item structure
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Config {
    pub config_id: String,
    pub config_category: String, // Categories: ORG (組織), PERM (權限), ACT (動作), STORE (門市), POS (POS 終端), INV (存貨)
    pub config_key: String,
    pub config_value: String,
    pub config_type: String, // STRING, NUMBER, BOOLEAN, JSON
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Config {
    /// Create a new configuration item
    pub fn new(
        config_id: String,
        config_category: String,
        config_key: String,
        config_value: String,
        config_type: String,
        description: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            config_id,
            config_category,
            config_key,
            config_value,
            config_type,
            description,
            is_sensitive: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Validate configuration value based on type
    pub fn validate_value(&self) -> Result<(), String> {
        match self.config_type.as_str() {
            "STRING" => {
                if self.config_value.is_empty() {
                    return Err("String configuration value cannot be empty".to_string());
                }
            }
            "NUMBER" => {
                if self.config_value.parse::<f64>().is_err() {
                    return Err("Invalid number format".to_string());
                }
            }
            "BOOLEAN" => {
                if !["true", "false", "1", "0"].contains(&self.config_value.to_lowercase().as_str())
                {
                    return Err("Boolean value must be true/false or 1/0".to_string());
                }
            }
            "JSON" if serde_json::from_str::<serde_json::Value>(&self.config_value).is_err() => {
                return Err("Invalid JSON format".to_string());
            }
            _ => {}
        }
        Ok(())
    }
}

/// Configuration category definitions
pub mod categories {
    /// Organization Management (ORG) - 6 items
    pub const ORG: [&str; 6] = [
        "ORG-01", // Company/Organization Structure
        "ORG-02", // Department/Business Units
        "ORG-03", // Position/Job Titles
        "ORG-04", // Employee/Personal Information
        "ORG-05", // Shift/Scheduling
        "ORG-06", // Holiday Calendar
    ];

    /// Permission Management (PERM) - 6 items
    pub const PERM: [&str; 6] = [
        "PERM-01", // Role Definition (RBAC)
        "PERM-02", // Functional Permissions
        "PERM-03", // Data Permissions
        "PERM-04", // Approval Permissions
        "PERM-05", // Operation Permissions
        "PERM-06", // Time-based Permissions
    ];

    /// Activity Management (ACT) - 6 items
    pub const ACT: [&str; 6] = [
        "ACT-01", // Promotion Activities
        "ACT-02", // Coupon Management
        "ACT-03", // Member Activities
        "ACT-04", // Holiday Activities
        "ACT-05", // Workflow Management
        "ACT-06", // Event Notifications
    ];

    /// Store Management (STORE) - 6 items
    pub const STORE: [&str; 6] = [
        "STORE-01", // Store Basic Information
        "STORE-02", // Store Type
        "STORE-03", // Store Business Hours
        "STORE-04", // Store Warehouse
        "STORE-05", // Store Pricing
        "STORE-06", // Store Staff
    ];

    /// POS Device and Cash Register (POS) - 7 items
    pub const POS: [&str; 7] = [
        "POS-01", // POS Device Basic Information
        "POS-02", // Cash Register Settings
        "POS-03", // Payment Methods
        "POS-04", // Invoice Settings
        "POS-05", // Cashier Management
        "POS-06", // Cash Box Settings
        "POS-07", // Connection Settings
    ];

    /// Inventory Management (INV) - 6 items
    pub const INV: [&str; 6] = [
        "INV-01", // Inventory Types
        "INV-02", // Product Information
        "INV-03", // Stock Levels
        "INV-04", // Stock Operations
        "INV-05", // Supplier Management
        "INV-06", // Purchase Orders
    ];
}

/// Configuration manager for CRUD operations
pub struct ConfigManager {
    pool: sqlx::SqlitePool,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all configuration items
    pub async fn get_all(&self) -> Result<Vec<Config>, Box<dyn std::error::Error>> {
        let pool = &self.pool;
        sqlx::query_as("SELECT * FROM erp_config ORDER BY config_category, config_key")
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    /// Get configuration by key
    pub async fn get_by_key(
        &self,
        config_key: &str,
        config_category: &str,
    ) -> Result<Option<Config>, Box<dyn std::error::Error>> {
        let pool = &self.pool;
        sqlx::query_as("SELECT * FROM erp_config WHERE config_key = ? AND config_category = ?")
            .bind(config_key)
            .bind(config_category)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.into())
    }

    /// Add or update configuration
    pub async fn upsert(
        &self,
        config: &Config,
        changed_by: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = &self.pool;

        // Check if config exists
        let existing: Option<Config> =
            sqlx::query_as("SELECT * FROM erp_config WHERE config_key = ? AND config_category = ?")
                .bind(&config.config_key)
                .bind(&config.config_category)
                .fetch_optional(pool)
                .await?;

        if existing.is_some() {
            // Update existing
            sqlx::query(
                "UPDATE erp_config SET config_value = ?, config_type = ?, description = ?, is_sensitive = ?, updated_at = CURRENT_TIMESTAMP WHERE config_key = ? AND config_category = ?"
            )
            .bind(&config.config_value)
            .bind(&config.config_type)
            .bind(&config.description)
            .bind(config.is_sensitive)
            .bind(&config.config_key)
            .bind(&config.config_category)
            .execute(pool)
            .await?;
        } else {
            // Insert new
            sqlx::query(
                "INSERT INTO erp_config (config_id, config_category, config_key, config_value, config_type, description, is_sensitive) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&config.config_id)
            .bind(&config.config_category)
            .bind(&config.config_key)
            .bind(&config.config_value)
            .bind(&config.config_type)
            .bind(&config.description)
            .bind(config.is_sensitive)
            .execute(pool)
            .await?;
        }

        // Log change
        let history_id = uuid::Uuid::new_v4().to_string();
        let old_value = existing.as_ref().map(|c| c.config_value.clone());
        let new_value = config.config_value.clone();

        sqlx::query(
            "INSERT INTO config_history (history_id, config_id, old_value, new_value, changed_by, change_reason) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(history_id)
        .bind(&config.config_id)
        .bind(&old_value)
        .bind(&new_value)
        .bind(changed_by)
        .bind("Updated via API")
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete configuration
    pub async fn delete(
        &self,
        config_id: &str,
        deleted_by: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = &self.pool;

        // Get current value before deletion
        let config = sqlx::query_as::<_, Config>("SELECT * FROM erp_config WHERE config_id = ?")
            .bind(config_id)
            .fetch_one(pool)
            .await?;

        // Delete
        sqlx::query("DELETE FROM erp_config WHERE config_id = ?")
            .bind(config_id)
            .execute(pool)
            .await?;

        // Log deletion
        let history_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO config_history (history_id, config_id, old_value, new_value, changed_by, change_reason) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(history_id)
        .bind(config_id)
        .bind(&config.config_value)
        .bind(None::<String>)
        .bind(deleted_by)
        .bind("Deleted")
        .execute(pool)
        .await?;

        Ok(())
    }
}
