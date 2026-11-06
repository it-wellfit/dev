#![forbid(unsafe_code)]

//! Unified Data Model (UDM) for cross-platform ecommerce integration.
//!
//! All monetary values are integer cents (`i64`). No `f64` in money paths.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Money in integer cents (1/100 of the base currency unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MoneyCents(pub i64);

impl MoneyCents {
    pub const ZERO: Self = Self(0);
}

impl fmt::Display for MoneyCents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / 100;
        let frac = (self.0 % 100).unsigned_abs();
        write!(f, "{}.{:02}", whole, frac)
    }
}

/// Fixed-point decimal with 4 decimal places (10^4 scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Decimal4(pub i64);

/// Provenance: source platform + contract version + field source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub platform: String,
    pub contract_version: String,
    pub field_source: String,
}

/// Order line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub sku: String,
    pub quantity: i64,
    pub unit_price: MoneyCents,
    pub line_amount: MoneyCents,
}

/// Unified order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedOrder {
    pub order_id: String,
    pub platform_order_id: String,
    pub platform: String,
    pub customer: String,
    pub items: Vec<OrderItem>,
    pub total: MoneyCents,
    pub currency: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub provenance: Provenance,
}

/// Inventory record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryRecord {
    pub sku: String,
    pub quantity: i64,
    pub warehouse: String,
    pub provenance: Provenance,
}

/// Product record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRecord {
    pub sku: String,
    pub name: String,
    pub barcode: Option<String>,
    pub provenance: Provenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_cents_display() {
        assert_eq!(MoneyCents(600).to_string(), "6.00");
        assert_eq!(MoneyCents(120000).to_string(), "1200.00");
    }

    #[test]
    fn order_roundtrip_serde() {
        let order = UnifiedOrder {
            order_id: "O1".into(),
            platform_order_id: "P1".into(),
            platform: "momo".into(),
            customer: "C1".into(),
            items: vec![],
            total: MoneyCents(1000),
            currency: "TWD".into(),
            status: "pending".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            provenance: Provenance {
                platform: "momo".into(),
                contract_version: "1.0".into(),
                field_source: "order_id".into(),
            },
        };
        let json = serde_json::to_string(&order).unwrap();
        let back: UnifiedOrder = serde_json::from_str(&json).unwrap();
        assert_eq!(back.order_id, "O1");
        assert_eq!(back.total, MoneyCents(1000));
    }
}