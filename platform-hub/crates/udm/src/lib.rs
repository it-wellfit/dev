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

// ============================================================================
// G2.3 (HP-03/HP-04): canonical entities beyond orders — customer, invoice,
// payment — each carrying an authoritative-system declaration so semantic
// drift ("which system owns this field's truth") is explicit at the type
// level, per plan §8.1.
// ============================================================================

/// Customer master record. Authority lives in the CRM; the channel platform
/// is only a source of contact enrichment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRecord {
    pub customer_id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub authoritative_system: String,
    pub provenance: Provenance,
}

/// Invoice derived from a unified order. Authority lives in the ERP/ledger;
/// channel platforms only propose amounts that require reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRecord {
    pub invoice_id: String,
    pub order_id: String,
    pub issue_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub total: MoneyCents,
    pub currency: String,
    pub status: String,
    pub authoritative_system: String,
    pub provenance: Provenance,
}

/// Payment captured against an invoice or order. Authority lives in the
/// payment provider / bank; channel status is advisory only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub payment_id: String,
    pub invoice_id: Option<String>,
    pub order_id: Option<String>,
    pub amount: MoneyCents,
    pub currency: String,
    pub paid_at: chrono::DateTime<chrono::Utc>,
    pub method: String,
    pub authoritative_system: String,
    pub provenance: Provenance,
}

// ============================================================================
// G2.3 unit contract: UDM `MoneyCents` (i64, 1/100 base unit) and the root
// workspace ledger minor unit (i128, 1/100 base unit) are THE SAME scale but
// different widths. Conversions are explicit and checked — never implicit,
// so the two domains cannot silently mix. This is the documented boundary
// (plan §0.5 semantic-drift ruling: contract now, merge never without an
// owner ADR backed by reconciliation data).
// ============================================================================

/// Widen ledger minor units (i128) into UDM cents. Infallible: i128 → i64
/// here is checked and truncation-free by construction (`?` propagation
/// happens in the fallible direction only).
pub fn money_cents_to_minor_i128(cents: MoneyCents) -> i128 {
    i128::from(cents.0)
}

/// Narrow ledger minor units (i128) into UDM cents. Returns `None` when the
/// value does not fit i64 — the caller decides whether to split, scale down,
/// or reject; silent truncation is not permitted at this boundary.
pub fn minor_i128_to_money_cents(minor: i128) -> Option<MoneyCents> {
    i64::try_from(minor).ok().map(MoneyCents)
}

// ============================================================================
// G2.4 (HP-06): per-flow reconciliation — counts, integer-cent totals, and
// deterministic difference samples. Pure computation over UDM orders so any
// sync flow (channels adapters) can feed it and any renderer (zero-JS page
// or CSV) can consume the serializable report.
// ============================================================================

/// One order present in both flows with differing totals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderDifference {
    pub order_id: String,
    pub source_total: Option<MoneyCents>,
    pub target_total: Option<MoneyCents>,
}

/// Reconciliation result for one sync flow. Lists are deterministic (sorted
/// by order_id); sample lists are capped — full detail lives in the missing /
/// mismatch vectors, samples exist for human review.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReconciliationReport {
    pub flow_id: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub source_count: usize,
    pub target_count: usize,
    pub matched_count: usize,
    pub missing_in_target: Vec<String>,
    pub missing_in_source: Vec<String>,
    pub amount_mismatched: Vec<OrderDifference>,
    /// Difference sample, capped at 10 entries by the constructor.
    pub sample_differences: Vec<OrderDifference>,
    /// Integer-cent totals accumulated in i128 (no overflow within any
    /// realistic flow; no floats per INV-01).
    pub source_total_cents: i128,
    pub target_total_cents: i128,
}

/// Maximum entries retained in `sample_differences`.
pub const RECONCILIATION_SAMPLE_CAP: usize = 10;

/// Reconcile a source flow (e.g., channel orders) against a target flow
/// (e.g., ERP invoices keyed by `order_id`). Order identity is
/// `UnifiedOrder::order_id`; equality compares `total` cents.
pub fn reconcile_orders(
    flow_id: &str,
    source: &[UnifiedOrder],
    target: &[UnifiedOrder],
) -> ReconciliationReport {
    use std::collections::BTreeMap;

    let target_by_id: BTreeMap<&str, i64> = target
        .iter()
        .map(|o| (o.order_id.as_str(), o.total.0))
        .collect();
    let source_by_id: BTreeMap<&str, i64> = source
        .iter()
        .map(|o| (o.order_id.as_str(), o.total.0))
        .collect();

    let mut matched = 0usize;
    let mut mismatched: Vec<OrderDifference> = Vec::new();
    let mut missing_in_target: Vec<String> = Vec::new();
    for order in source {
        match target_by_id.get(order.order_id.as_str()) {
            Some(&target_total) => {
                if target_total == order.total.0 {
                    matched += 1;
                } else {
                    mismatched.push(OrderDifference {
                        order_id: order.order_id.clone(),
                        source_total: Some(order.total),
                        target_total: Some(MoneyCents(target_total)),
                    });
                }
            }
            None => missing_in_target.push(order.order_id.clone()),
        }
    }

    let mut missing_in_source: Vec<String> = target
        .iter()
        .filter(|o| !source_by_id.contains_key(o.order_id.as_str()))
        .map(|o| o.order_id.clone())
        .collect();

    missing_in_target.sort();
    missing_in_source.sort();
    mismatched.sort_by(|a, b| a.order_id.cmp(&b.order_id));

    let source_total = source.iter().map(|o| o.total.0 as i128).sum();
    let target_total = target.iter().map(|o| o.total.0 as i128).sum();

    ReconciliationReport {
        flow_id: flow_id.to_string(),
        generated_at: chrono::Utc::now(),
        source_count: source.len(),
        target_count: target.len(),
        matched_count: matched,
        missing_in_target,
        missing_in_source,
        sample_differences: mismatched
            .iter()
            .take(RECONCILIATION_SAMPLE_CAP)
            .cloned()
            .collect(),
        amount_mismatched: mismatched,
        source_total_cents: source_total,
        target_total_cents: target_total,
    }
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

    // G2.3 — canonical entity serde roundtrips (one per new record).
    fn provenance() -> Provenance {
        Provenance {
            platform: "momo".into(),
            contract_version: "1.0".into(),
            field_source: "unit-test".into(),
        }
    }

    #[test]
    fn customer_record_roundtrip_serde() {
        let customer = CustomerRecord {
            customer_id: "C-1".into(),
            name: "大前實業".into(),
            email: Some("erp@dachen.example".into()),
            phone: None,
            status: "active".into(),
            authoritative_system: "crm".into(),
            provenance: provenance(),
        };
        let back: CustomerRecord =
            serde_json::from_str(&serde_json::to_string(&customer).unwrap()).unwrap();
        assert_eq!(back.customer_id, "C-1");
        assert_eq!(back.authoritative_system, "crm");
    }

    #[test]
    fn invoice_record_roundtrip_serde() {
        let invoice = InvoiceRecord {
            invoice_id: "INV-1".into(),
            order_id: "O1".into(),
            issue_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 9).unwrap(),
            total: MoneyCents(123_450),
            currency: "TWD".into(),
            status: "issued".into(),
            authoritative_system: "erp".into(),
            provenance: provenance(),
        };
        let back: InvoiceRecord =
            serde_json::from_str(&serde_json::to_string(&invoice).unwrap()).unwrap();
        assert_eq!(back.total, MoneyCents(123_450));
        assert_eq!(back.order_id, "O1");
    }

    #[test]
    fn payment_record_roundtrip_serde() {
        let payment = PaymentRecord {
            payment_id: "PAY-1".into(),
            invoice_id: Some("INV-1".into()),
            order_id: None,
            amount: MoneyCents(9_800_000),
            currency: "TWD".into(),
            paid_at: chrono::Utc::now(),
            method: "bank_transfer".into(),
            authoritative_system: "bank".into(),
            provenance: provenance(),
        };
        let back: PaymentRecord =
            serde_json::from_str(&serde_json::to_string(&payment).unwrap()).unwrap();
        assert_eq!(back.amount, MoneyCents(9_800_000));
        assert_eq!(back.authoritative_system, "bank");
    }

    // G2.3 — unit contract fitness: the boundary is lossless inside i64 and
    // refuses silent truncation outside it.
    #[test]
    fn minor_cents_boundary_is_explicit_and_lossless() {
        // Same scale: identity within i64 range.
        let minor: i128 = 123_450;
        let cents = minor_i128_to_money_cents(minor).expect("fits i64");
        assert_eq!(cents, MoneyCents(123_450));
        assert_eq!(money_cents_to_minor_i128(cents), minor);

        // Beyond i64: explicit None, never silent truncation.
        let huge: i128 = i64::MAX as i128 + 1;
        assert!(minor_i128_to_money_cents(huge).is_none());
    }

    // G2.4 — reconciliation recomputation from a fixed fixture: every number
    // in the report is derivable by hand from the inputs.
    fn order(id: &str, total_cents: i64) -> UnifiedOrder {
        UnifiedOrder {
            order_id: id.into(),
            platform_order_id: format!("P-{id}"),
            platform: "momo".into(),
            customer: "C1".into(),
            items: vec![],
            total: MoneyCents(total_cents),
            currency: "TWD".into(),
            status: "pending".into(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            provenance: provenance(),
        }
    }

    #[test]
    fn reconciliation_report_matches_hand_computation() {
        let source = vec![order("O1", 1_000), order("O2", 2_000), order("O3", 3_000)];
        let target = vec![
            order("O1", 1_000),   // exact match
            order("O2", 2_500),   // amount mismatch
            order("O4", 999_000), // in target only
        ];

        let report = reconcile_orders("momo->erp", &source, &target);

        // Recomputed by hand:
        assert_eq!(report.flow_id, "momo->erp");
        assert_eq!(report.source_count, 3);
        assert_eq!(report.target_count, 3);
        assert_eq!(report.matched_count, 1, "only O1 matches exactly");
        assert_eq!(report.missing_in_target, vec!["O3".to_string()]);
        assert_eq!(report.missing_in_source, vec!["O4".to_string()]);
        assert_eq!(
            report.amount_mismatched,
            vec![OrderDifference {
                order_id: "O2".into(),
                source_total: Some(MoneyCents(2_000)),
                target_total: Some(MoneyCents(2_500)),
            }]
        );
        assert_eq!(report.source_total_cents, 6_000);
        assert_eq!(report.target_total_cents, 1_000 + 2_500 + 999_000);

        // Serializable for zero-JS rendering pipelines.
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"matched_count\":1"));

        // Difference sample is capped and deterministic.
        assert!(report.sample_differences.len() <= RECONCILIATION_SAMPLE_CAP);
    }

    #[test]
    fn reconciliation_lists_are_deterministic_regardless_of_input_order() {
        let a = vec![order("B", 1), order("A", 1), order("C", 1)];
        let b = vec![order("C", 1), order("A", 1), order("B", 1)];
        let report = reconcile_orders("t", &a, &b);
        assert_eq!(report.matched_count, 3);
        assert!(report.missing_in_target.is_empty());
        assert!(report.missing_in_source.is_empty());
    }

    #[test]
    fn reconciliation_sample_cap_holds() {
        // 30 mismatches → sample capped at 10, full list still complete.
        let source: Vec<UnifiedOrder> = (0..30).map(|i| order(&format!("O{i:02}"), 100)).collect();
        let target: Vec<UnifiedOrder> = (0..30).map(|i| order(&format!("O{i:02}"), 200)).collect();
        let report = reconcile_orders("cap", &source, &target);
        assert_eq!(report.amount_mismatched.len(), 30);
        assert_eq!(report.sample_differences.len(), RECONCILIATION_SAMPLE_CAP);
    }
}
