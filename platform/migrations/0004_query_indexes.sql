-- Migration 0004: query index optimization (Slice 12, performance hardening)
--
-- Additional indexes for common query patterns identified during
-- performance review: account-based transaction item lookups,
-- audit log time-range queries, and account code resolution.

CREATE INDEX IF NOT EXISTS idx_transaction_items_account ON transaction_items (account_id);
CREATE INDEX IF NOT EXISTS idx_transaction_items_type ON transaction_items (item_type);
CREATE INDEX IF NOT EXISTS idx_accounts_code ON accounts (account_code);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_status_date ON transactions (status, transaction_date);
