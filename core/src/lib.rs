#![forbid(unsafe_code)]
// Core module for AuxBridge
// This module provides the core accounting and business logic functionality

pub mod config;
pub mod ledger;
pub mod money;
pub mod parallel;
pub mod parser;
pub mod permission;
pub mod value_objects;

// Re-export key types
pub use ledger::Account;
pub use ledger::AuditLog;
pub use ledger::LedgerEngine;
pub use ledger::Transaction;
pub use ledger::TransactionItem;
pub use money::{MoneyError, format_minor, format_minor_plain, parse_decimal_to_minor};
pub use parser::{
    AiSuggestion, DraftEntry, Evidence, ParsedSuggestion, parse_ai_suggestion, parse_unstructured,
};
pub use value_objects::{AccountId, Money};
