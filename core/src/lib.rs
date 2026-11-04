// Core module for Flaps ERP
// This module provides the core accounting and business logic functionality

pub mod config;
pub mod ledger;
pub mod money;
pub mod parser;
pub mod permission;

// Re-export key types
pub use ledger::Account;
pub use ledger::AuditLog;
pub use ledger::LedgerEngine;
pub use ledger::Transaction;
pub use ledger::TransactionItem;
pub use money::{format_minor, parse_decimal_to_minor, MoneyError};
pub use parser::{parse_unstructured, DraftEntry, Evidence};
