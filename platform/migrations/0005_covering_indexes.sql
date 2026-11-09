-- Migration 0005: covering indexes for hot query paths (Slice 12, performance hardening)
--
-- Covering index on transaction_items so monthly_by_account and monthly_totals
-- can satisfy SUM(CASE ...) aggregations entirely from the index without
-- visiting the table heap. Critical for the 500ms P95 SLA under 100k+
-- transactions (stress test api::tests::stress_10000_transactions_p95_under_500ms).

CREATE INDEX IF NOT EXISTS idx_transaction_items_covering
    ON transaction_items (transaction_id, item_type, amount_minor);

-- Composite index for monthly_totals: date-first lookup then grouping by strftime.
-- The leading column transaction_date enables index range scan on the WHERE clause.
CREATE INDEX IF NOT EXISTS idx_transactions_date_items
    ON transactions (transaction_date DESC, transaction_id);

