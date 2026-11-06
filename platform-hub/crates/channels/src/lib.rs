#![forbid(unsafe_code)]

//! Channel adapter trait and format derivation.

use async_trait::async_trait;
use platform_hub_udm::{InventoryRecord, UnifiedOrder};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Capabilities a channel adapter supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub fetch_orders: bool,
    pub push_inventory: bool,
    pub webhook: bool,
}

/// Channel adapter trait — implemented per platform (momo, PChome, Cyberbiz, ...).
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn platform_id(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError>;
    async fn push_inventory(&self, records: &[InventoryRecord]) -> Result<(), ChannelError>;
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
    async fn health(&self) -> Result<(), ChannelError>;
}

/// Bidirectional format derivation.
pub trait FormatDeriver: Send + Sync {
    fn platform_to_udm(&self, raw: &[u8]) -> Result<Vec<UnifiedOrder>, ChannelError>;
    fn udm_to_platform(&self, records: &[InventoryRecord]) -> Result<Vec<u8>, ChannelError>;
}

/// Column contract — name-based resolution, hard failure on missing required columns.
#[derive(Debug, Clone)]
pub struct ColumnContract {
    pub name: &'static str,
    pub required: Vec<&'static str>,
    pub optional: Vec<&'static str>,
}

impl ColumnContract {
    pub fn validate_header(&self, header: &[String]) -> Result<(), ChannelError> {
        let header_set: std::collections::HashSet<&str> = header.iter().map(|s| s.as_str()).collect();
        let missing: Vec<&str> = self
            .required
            .iter()
            .copied()
            .filter(|col| !header_set.contains(col))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(ChannelError::MissingColumns {
                contract: self.name,
                missing,
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("missing required columns for contract {contract}: {missing:?}")]
    MissingColumns {
        contract: &'static str,
        missing: Vec<&'static str>,
    },
    #[error("http error: {0}")]
    Http(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("rate limited")]
    RateLimited,
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

/// Adapter stub for platforms whose official API contract has not been
/// obtained yet (Phase E1 / E7). Every operation returns
/// `ChannelError::NotImplemented` referencing the pending contract, and
/// `contract_status()` reports `pending_contract`. No contract fact is ever
/// guessed (Knowledge research-protocol §1.1).
pub struct PendingContractAdapter {
    platform_id: &'static str,
    spec_request_ref: &'static str,
}

impl PendingContractAdapter {
    pub fn new(platform_id: &'static str, spec_request_ref: &'static str) -> Self {
        Self {
            platform_id,
            spec_request_ref,
        }
    }

    pub fn contract_status(&self) -> &'static str {
        "pending_contract"
    }

    pub fn spec_request_ref(&self) -> &'static str {
        self.spec_request_ref
    }
}

#[async_trait]
impl ChannelAdapter for PendingContractAdapter {
    fn platform_id(&self) -> &'static str {
        self.platform_id
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fetch_orders: false,
            push_inventory: false,
            webhook: false,
        }
    }

    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }

    async fn push_inventory(&self, _records: &[InventoryRecord]) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }

    fn verify_webhook_signature(&self, _payload: &[u8], _signature: &str) -> bool {
        false
    }

    async fn health(&self) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(format!(
            "{}: API contract pending ({})",
            self.platform_id, self.spec_request_ref
        )))
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fetch_orders={} push_inventory={} webhook={}",
            self.fetch_orders, self.push_inventory, self.webhook
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_contract_rejects_missing() {
        let c = ColumnContract {
            name: "test",
            required: vec!["A", "B"],
            optional: vec![],
        };
        assert!(c.validate_header(&["A".into()]).is_err());
        assert!(c.validate_header(&["A".into(), "B".into()]).is_ok());
    }

    #[tokio::test]
    async fn pending_contract_adapter_refuses_all_operations() {
        let pchome = PendingContractAdapter::new("pchome", "OQ-F2-5 (PChome spec request)");
        let cyberbiz = PendingContractAdapter::new("cyberbiz", "OQ-F2-5 (Cyberbiz spec request)");

        for adapter in [&pchome, &cyberbiz] {
            assert_eq!(adapter.contract_status(), "pending_contract");
            let caps = adapter.capabilities();
            assert!(!caps.fetch_orders && !caps.push_inventory && !caps.webhook);
            assert!(adapter.fetch_orders().await.is_err());
            assert!(adapter.health().await.is_err());
            assert!(!adapter.verify_webhook_signature(b"{}", "sig"));
        }
    }
}