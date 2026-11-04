//! DDD value objects (Slice 10 Layer H / plan v3 §15) — construction-time
//! validation, immutable, no floating point (INV-01).

/// Money as integer minor units plus an ISO-4217-style 3-letter currency.
/// Construction fails closed: positive amount, uppercase 3-letter code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    minor: i64,
    currency: String,
}

impl Money {
    /// Validate and construct. `minor` must be > 0 (a journal entry leg is
    /// always a positive amount with an explicit DEBIT/CREDIT side).
    pub fn new(minor: i64, currency: impl Into<String>) -> Result<Self, String> {
        if minor <= 0 {
            return Err("Money: amount must be positive".to_string());
        }
        let currency = currency.into().trim().to_uppercase();
        if currency.len() != 3 || !currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err("Money: currency must be a 3-letter ASCII code".to_string());
        }
        Ok(Self { minor, currency })
    }

    pub fn minor(&self) -> i64 {
        self.minor
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Two-decimal display form via the exact integer formatter.
    pub fn display(&self) -> String {
        crate::money::format_minor(self.minor, 2)
    }
}

/// Account identifier: non-empty, ≤ 64 chars, no surrounding whitespace.
/// Newtype so an unvalidated String cannot reach the ledger by accident.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountId(String);

impl AccountId {
    pub fn new(v: impl Into<String>) -> Result<Self, String> {
        let v = v.into().trim().to_string();
        if v.is_empty() {
            return Err("AccountId: must not be empty".to_string());
        }
        if v.len() > 64 {
            return Err("AccountId: must be at most 64 characters".to_string());
        }
        Ok(Self(v))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_accepts_positive_amount_with_valid_currency() {
        let m = Money::new(123_450, "twd").expect("valid");
        assert_eq!(m.minor(), 123_450);
        assert_eq!(m.currency(), "TWD");
        assert_eq!(m.display(), "1,234.50");
    }

    #[test]
    fn money_rejects_zero_negative_and_bad_currency() {
        assert!(Money::new(0, "TWD").is_err(), "zero rejected");
        assert!(Money::new(-5, "TWD").is_err(), "negative rejected");
        assert!(Money::new(100, "TW").is_err(), "2-letter rejected");
        assert!(Money::new(100, "TWD1").is_err(), "4-letter rejected");
        assert!(Money::new(100, "t1").is_err(), "digit in code rejected");
        assert!(Money::new(100, "").is_err(), "empty rejected");
    }

    #[test]
    fn account_id_trims_and_enforces_bounds() {
        let id = AccountId::new("  ACC-001  ").expect("valid");
        assert_eq!(id.as_str(), "ACC-001");
        assert!(AccountId::new("   ").is_err(), "empty rejected");
        let long = "A".repeat(65);
        assert!(AccountId::new(long).is_err(), ">64 rejected");
    }
}
