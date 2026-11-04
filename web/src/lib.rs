// Web module for Flaps ERP
// This module provides web interface components for the Flaps ERP system
// The frontend is implemented in HTML/JavaScript (see index.html)
// The backend API is in src-tauri/

pub mod dashboard;
pub mod devsys;
pub mod ledger;

// Re-export important types (simplified versions without Yew)
pub use dashboard::{DashboardStats, SystemHealth};
pub use devsys::{Permission, SystemConfiguration};
pub use ledger::{Account, Transaction};
