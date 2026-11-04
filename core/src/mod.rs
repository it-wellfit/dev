/// Core ledger module for Flaps ERP - accounting engine
/// 
/// This module implements the core accounting functionality including:
/// - Transaction records
/// - Account management  
/// - Ledger engine
/// - Audit logging
/// 
/// All operations are ACID-compliant with SQLite WAL mode for performance.

pub mod transaction;
pub mod account;
pub mod audit;
pub mod engine;

/// Main entry point for ledger operations
pub struct LedgerEngine {
    pub connection: sqlx::SqlitePool,
}

impl LedgerEngine {
    /// Initialize the ledger engine with a database connection
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;
        
        // Enable WAL mode for better performance
        sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await?;
        sqlx::query("PRAGMA cache_size = -64000").execute(&pool).await?;
        
        Self::create_tables(&pool).await?;
        
        Ok(Self { connection: pool })
    }
    
    /// Create all necessary database tables
    async fn create_tables(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
        // Create accounts table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                account_id TEXT PRIMARY KEY,
                account_code TEXT NOT NULL,
                account_name TEXT NOT NULL,
                account_type TEXT NOT NULL,
                parent_account_id TEXT,
                currency TEXT DEFAULT 'TWD',
                description TEXT,
                is_active BOOLEAN DEFAULT true,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(pool).await?;
        
        // Create transactions table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                transaction_id TEXT PRIMARY KEY,
                transaction_code TEXT NOT NULL,
                transaction_date DATE NOT NULL,
                description TEXT NOT NULL,
                reference_number TEXT,
                status TEXT NOT NULL,
                source_type TEXT,
                created_by TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        ).execute(pool).await?;
        
        // Create transaction_items table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transaction_items (
                item_id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                amount DECIMAL(15,2) NOT NULL,
                currency TEXT DEFAULT 'TWD',
                exchange_rate DECIMAL(10,4) DEFAULT 1.0,
                reference TEXT,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id),
                FOREIGN KEY (account_id) REFERENCES accounts(account_id)
            )"
        ).execute(pool).await?;
        
        // Create audit_log table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_log (
                log_id TEXT PRIMARY KEY,
                transaction_id TEXT,
                event_type TEXT NOT NULL,
                event_data TEXT,
                performed_by TEXT,
                performed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id)
            )"
        ).execute(pool).await?;
        
        Ok(())
    }
    
    /// Post a journal entry to the ledger
    pub async fn post_journal_entry(
        &self,
        transaction: Transaction,
        items: Vec<TransactionItem>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = self.connection.begin().await?;
        
        // Insert transaction header
        sqlx::query(
            "INSERT INTO transactions (transaction_id, transaction_code, transaction_date, description, reference_number, status, source_type, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&transaction.transaction_id)
        .bind(&transaction.transaction_code)
        .bind(&transaction.transaction_date)
        .bind(&transaction.description)
        .bind(&transaction.reference_number)
        .bind(&transaction.status)
        .bind(&transaction.source_type)
        .bind(&transaction.created_by)
        .execute(&mut tx).await?;
        
        // Insert transaction items
        for item in items {
            sqlx::query(
                "INSERT INTO transaction_items (item_id, transaction_id, account_id, item_type, amount, currency, exchange_rate, reference, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&item.item_id)
            .bind(&item.transaction_id)
            .bind(&item.account_id)
            .bind(&item.item_type)
            .bind(&item.amount)
            .bind(&item.currency)
            .bind(&item.exchange_rate)
            .bind(&item.reference)
            .bind(&item.description)
            .execute(&mut tx).await?;
        }
        
        // Log the transaction
        sqlx::query(
            "INSERT INTO audit_log (log_id, transaction_id, event_type, event_data, performed_by) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&transaction.transaction_id)
        .bind("TRANSACTION_POSTED")
        .bind(serde_json::to_string(&transaction)?)
        .bind(&transaction.created_by)
        .execute(&mut tx).await?;
        
        tx.commit().await?;
        
        Ok(())
    }
    
    /// Query transactions with filtering
    pub async fn query_transactions(
        &self,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        status: Option<String>,
        limit: i64,
        offset: i64
    ) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let mut query = "SELECT transaction_id, transaction_code, transaction_date, description, reference_number, status, source_type, created_by, created_at FROM transactions WHERE 1=1".to_string();
        let mut conditions = Vec::new();
        
        if let Some(start) = start_date {
            conditions.push(format!("transaction_date >= '{}'", start));
        }
        
        if let Some(end) = end_date {
            conditions.push(format!("transaction_date <= '{}'", end));
        }
        
        if let Some(s) = status {
            conditions.push(format!("status = '{}'", s));
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY transaction_date DESC, created_at DESC LIMIT ? OFFSET ?");
        
        let rows = sqlx::query_as::<_, Transaction>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.connection)
            .await?;
        
        Ok(rows)
    }
}