pub mod api;
pub mod auth;
pub mod com;
pub mod cross_module_api;
pub mod db;
pub mod search;
pub mod sync;

// AI Integration Module
pub mod ai;

// Re-export AI types for easy access
pub use ai::*;
