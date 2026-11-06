#![forbid(unsafe_code)]

//! momo adapter — skeleton only (pending_contract until official API spec obtained).

use async_trait::async_trait;
use platform_hub_channels::{Capabilities, ChannelAdapter, ChannelError, ColumnContract, FormatDeriver};
use platform_hub_udm::{InventoryRecord, UnifiedOrder};

/// momo adapter. All methods return `pending_contract` until the official API spec is obtained.
pub struct MomoAdapter {
    contract_status: &'static str,
}

impl MomoAdapter {
    pub fn new() -> Self {
        Self {
            contract_status: "pending_contract",
        }
    }

    pub fn contract_status(&self) -> &'static str {
        self.contract_status
    }
}

impl Default for MomoAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChannelAdapter for MomoAdapter {
    fn platform_id(&self) -> &'static str {
        "momo"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fetch_orders: false,
            push_inventory: false,
            webhook: false,
        }
    }

    async fn fetch_orders(&self) -> Result<Vec<UnifiedOrder>, ChannelError> {
        Err(ChannelError::NotImplemented(
            "momo API spec not yet obtained (OQ-F2-5)".into(),
        ))
    }

    async fn push_inventory(&self, _records: &[InventoryRecord]) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(
            "momo API spec not yet obtained (OQ-F2-5)".into(),
        ))
    }

    fn verify_webhook_signature(&self, _payload: &[u8], _signature: &str) -> bool {
        false
    }

    async fn health(&self) -> Result<(), ChannelError> {
        Err(ChannelError::NotImplemented(
            "momo API spec not yet obtained (OQ-F2-5)".into(),
        ))
    }
}

impl FormatDeriver for MomoAdapter {
    fn platform_to_udm(&self, _raw: &[u8]) -> Result<Vec<UnifiedOrder>, ChannelError> {
        Err(ChannelError::NotImplemented(
            "momo API spec not yet obtained (OQ-F2-5)".into(),
        ))
    }

    fn udm_to_platform(&self, _records: &[InventoryRecord]) -> Result<Vec<u8>, ChannelError> {
        Err(ChannelError::NotImplemented(
            "momo API spec not yet obtained (OQ-F2-5)".into(),
        ))
    }
}

/// Column contract for momo — placeholder until spec obtained.
pub fn momo_order_contract() -> ColumnContract {
    ColumnContract {
        name: "momo order (pending_contract)",
        required: vec![],
        optional: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn momo_adapter_reports_pending_contract() {
        let adapter = MomoAdapter::new();
        assert_eq!(adapter.contract_status(), "pending_contract");
        assert_eq!(adapter.platform_id(), "momo");
    }
}