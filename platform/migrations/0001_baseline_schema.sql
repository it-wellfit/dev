-- Migration 0001: Baseline schema for the Dev root workspace (SQLite)
--
-- SSOT：schema 變更一律新增遷移檔（禁止修改已提交的遷移）。
-- 冪等策略：全部使用 IF NOT EXISTS，使既有動態建表資料庫
-- （core/ledger/engine.rs::ensure_tables、db.rs 舊版 initialize）可安全重放。
--
-- 金額欄位（P0-1 / INV-01）：amount_minor 為整數分（i64），exchange_rate 為
-- 10^4 縮放整數。浮點貨幣不存在於任何欄位。
-- 版本註記（2026-11-04）：本遷移建立於同一工作 session、無任何已遷移資料庫，
-- 因此原始 DECIMAL 草稿直接修正為整數 schema（未發布遷移允許原地修正）。

-- === Ledger（來源：core/src/ledger/engine.rs） ===

CREATE TABLE IF NOT EXISTS accounts (
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
);

CREATE TABLE IF NOT EXISTS transactions (
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
);

CREATE TABLE IF NOT EXISTS transaction_items (
    item_id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    amount_minor INTEGER NOT NULL,
    currency TEXT DEFAULT 'TWD',
    exchange_rate INTEGER DEFAULT 10000,
    reference TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id),
    FOREIGN KEY (account_id) REFERENCES accounts(account_id)
);

CREATE TABLE IF NOT EXISTS audit_log (
    log_id TEXT PRIMARY KEY,
    transaction_id TEXT,
    event_type TEXT NOT NULL,
    event_data TEXT,
    performed_by TEXT,
    performed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (transaction_id) REFERENCES transactions(transaction_id)
);

-- === Config（來源：platform/src/db.rs） ===

CREATE TABLE IF NOT EXISTS erp_config (
    config_id TEXT PRIMARY KEY,
    config_category TEXT NOT NULL,
    config_key TEXT NOT NULL,
    config_value TEXT NOT NULL,
    config_type TEXT NOT NULL,
    description TEXT,
    is_sensitive BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS config_history (
    history_id TEXT PRIMARY KEY,
    config_id TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by TEXT,
    change_reason TEXT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- === RBAC（來源：platform/src/db.rs 與 core/src/permission.rs） ===

CREATE TABLE IF NOT EXISTS roles (
    role_id TEXT PRIMARY KEY,
    role_name TEXT NOT NULL,
    role_code TEXT NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    permission_id TEXT PRIMARY KEY,
    permission_name TEXT NOT NULL,
    permission_code TEXT NOT NULL,
    permission_type TEXT NOT NULL,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT 1
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    granted_by TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    assigned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    assigned_by TEXT NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

-- === 索引（查詢熱路徑：FR-007 交易查詢） ===

CREATE INDEX IF NOT EXISTS idx_transactions_date
    ON transactions (transaction_date DESC);

CREATE INDEX IF NOT EXISTS idx_transactions_status
    ON transactions (status);

CREATE INDEX IF NOT EXISTS idx_transaction_items_transaction
    ON transaction_items (transaction_id);

CREATE INDEX IF NOT EXISTS idx_audit_log_transaction
    ON audit_log (transaction_id);

CREATE INDEX IF NOT EXISTS idx_erp_config_key_category
    ON erp_config (config_key, config_category);
