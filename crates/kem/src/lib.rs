#![forbid(unsafe_code)]
//! KEM — Knowledge Evolution Module (schema v1).
//!
//! Layered per `docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md`:
//! L0 capture (immutable raw) → L1 vetting (rules → local LLM → distilled)
//! → L2 store (claims as JSON SSOT) → L3 retrieval (BM25) → L4 evolution
//! (sync/decay/supersede) → L5 governance (ledger/review/lint).
//!
//! Invariants:
//! - every source file starts with `#![forbid(unsafe_code)]`
//! - claims are the machine SSOT; markdown pages are generated views
//! - the ledger is append-only with a SHA-256 hash chain
//! - supersede, never delete: `retired` is the only terminal state

pub mod capture;
pub mod dashboard;
pub mod embed;
pub mod evolve;
pub mod filing;
pub mod gov;
pub mod ledger;
pub mod schema;
pub mod search;
pub mod seed;
pub mod vetting;

pub const SCHEMA_VERSION: &str = "1";

pub use ledger::{LedgerAction, LedgerEntry};
pub use schema::{validate_claim, Claim, ClaimStatus, ClaimType, Evidence, KemError, Track};
