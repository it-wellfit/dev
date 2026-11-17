-- seed-dev.sql（計畫 1793782615513 §3.5）— dev 環境種子資料
-- 冪等：全部 INSERT OR IGNORE，可安全重放。
-- 契約：雙分錄必平衡（INV-01）；金額為整數分（amount_minor i64）。
-- 套用方式：
--   sqlite3 data/dev.db < scripts/seed-dev.sql
--   （無 sqlite3 CLI 時：python -c "import sqlite3;c=sqlite3.connect('data/dev.db');c.executescript(open('scripts/seed-dev.sql',encoding='utf-8').read());c.commit()"
--     ——僅一次性套用指令，非常駐 Python 工件（C-11））

-- ── 會計科目（3 個標準科目；與 demo prod 資料一致）────────────────
INSERT OR IGNORE INTO accounts (account_id, account_code, account_name, account_type, currency, is_active)
VALUES
    ('CASH-001', '1001', '現金', 'ASSET', 'TWD', 1),
    ('EXP-6001', '6001', '辦公費用', 'EXPENSE', 'TWD', 1),
    ('REV-4001', '4001', '服務收入', 'REVENUE', 'TWD', 1);

-- ── 樣本交易 5 筆（每筆雙分錄：借=貸）────────────────────────────
-- 交易 1：現金收入 50,000.00
INSERT OR IGNORE INTO transactions (transaction_id, transaction_code, transaction_date, description, status, source_type, created_by)
VALUES ('tx-seed-0001', 'TX-SEED-0001', '2026-11-01', '種子：客戶 A 服務收入入帳', 'POSTED', 'manual', 'seed');
INSERT OR IGNORE INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency)
VALUES
    ('ti-seed-0001a', 'tx-seed-0001', 'CASH-001', 'DEBIT',  5000000, 'TWD'),
    ('ti-seed-0001b', 'tx-seed-0001', 'REV-4001', 'CREDIT', 5000000, 'TWD');

-- 交易 2：辦公支出 12,000.00
INSERT OR IGNORE INTO transactions (transaction_id, transaction_code, transaction_date, description, status, source_type, created_by)
VALUES ('tx-seed-0002', 'TX-SEED-0002', '2026-11-03', '種子：辦公用品採購', 'POSTED', 'manual', 'seed');
INSERT OR IGNORE INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency)
VALUES
    ('ti-seed-0002a', 'tx-seed-0002', 'EXP-6001', 'DEBIT',  1200000, 'TWD'),
    ('ti-seed-0002b', 'tx-seed-0002', 'CASH-001', 'CREDIT', 1200000, 'TWD');

-- 交易 3：現金收入 88,000.00
INSERT OR IGNORE INTO transactions (transaction_id, transaction_code, transaction_date, description, status, source_type, created_by)
VALUES ('tx-seed-0003', 'TX-SEED-0003', '2026-11-08', '種子：客戶 B 專案款', 'POSTED', 'manual', 'seed');
INSERT OR IGNORE INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency)
VALUES
    ('ti-seed-0003a', 'tx-seed-0003', 'CASH-001', 'DEBIT',  8800000, 'TWD'),
    ('ti-seed-0003b', 'tx-seed-0003', 'REV-4001', 'CREDIT', 8800000, 'TWD');

-- 交易 4：設備支出 6,500.00
INSERT OR IGNORE INTO transactions (transaction_id, transaction_code, transaction_date, description, status, source_type, created_by)
VALUES ('tx-seed-0004', 'TX-SEED-0004', '2026-11-12', '種子：測試設備採購', 'POSTED', 'manual', 'seed');
INSERT OR IGNORE INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency)
VALUES
    ('ti-seed-0004a', 'tx-seed-0004', 'EXP-6001', 'DEBIT',  650000, 'TWD'),
    ('ti-seed-0004b', 'tx-seed-0004', 'CASH-001', 'CREDIT', 650000, 'TWD');

-- 交易 5：現金收入 30,000.00（DRAFT 狀態樣本）
INSERT OR IGNORE INTO transactions (transaction_id, transaction_code, transaction_date, description, status, source_type, created_by)
VALUES ('tx-seed-0005', 'TX-SEED-0005', '2026-11-15', '種子：待裁決草稿（DRAFT）', 'DRAFT', 'manual', 'seed');
INSERT OR IGNORE INTO transaction_items (item_id, transaction_id, account_id, item_type, amount_minor, currency)
VALUES
    ('ti-seed-0005a', 'tx-seed-0005', 'CASH-001', 'DEBIT',  3000000, 'TWD'),
    ('ti-seed-0005b', 'tx-seed-0005', 'REV-4001', 'CREDIT', 3000000, 'TWD');
