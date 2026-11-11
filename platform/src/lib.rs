pub mod api;
pub mod apihub;
pub mod auth;
pub mod bankpay;
pub mod bm25;
pub mod com;
pub mod controls;
pub mod cross_module_api;
pub mod db;
pub mod ingest;
pub mod nav;
pub mod ocr;
pub mod pages;
pub mod search;
pub mod status;
pub mod sync;
pub mod workbench;

// AI Integration Module
pub mod ai;

// Re-export AI types for easy access
pub use ai::*;
