//! Fixed-point money helpers (INV-01: no floating point in money).
//!
//! Minor units: an `i64` count of 10^-`dp` units (e.g. `dp = 2` for cents).
//! Parsing is exact string arithmetic — never `f64` — so the mapping from
//! user input to stored minor units is deterministic and derivable
//! (可推導): given the same input, the same `i64` always results.

use std::fmt;

/// Maximum digits a minor-unit value may occupy (15 integral + 2 fractional
/// digits for `dp = 2`, matching the DECIMAL(15,2) schema lineage).
pub const MAX_TOTAL_DIGITS: usize = 17;

/// Errors for exact decimal parsing. `Display` texts are written for direct
/// surfacing in the zero-JS UI (problem + reason + next step).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    Empty,
    InvalidFormat,
    TooManyDecimalPlaces { max: u32, got: usize },
    Negative,
    Overflow { max_total_digits: usize },
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "金額為必填欄位，請輸入大於 0 的數字"),
            Self::InvalidFormat => write!(
                f,
                "金額格式不正確（可能原因：含非法字元或多次小數點）；請輸入純數字，例如 1234.50"
            ),
            Self::TooManyDecimalPlaces { max, got } => write!(
                f,
                "小數位數過多（最多 {max} 位，輸入 {got} 位）；請修正後重新送出"
            ),
            Self::Negative => write!(
                f,
                "金額不得為負數；借貸方向請使用借方（DEBIT）或貸方（CREDIT）選項表達"
            ),
            Self::Overflow { max_total_digits } => write!(
                f,
                "金額位數過多（上限 {max_total_digits} 位數字）；請拆分交易或聯絡管理員"
            ),
        }
    }
}

impl std::error::Error for MoneyError {}

/// Parse an exact decimal string into minor units with `dp` fractional digits.
///
/// Rules (deterministic, fail-closed):
/// - optional leading `+`; leading `-` is rejected (use DEBIT/CREDIT instead)
/// - digits with at most one decimal point; thousands separators rejected
/// - more than `dp` fractional digits is rejected (no silent rounding)
/// - the result must fit within [`MAX_TOTAL_DIGITS`] digits
pub fn parse_decimal_to_minor(input: &str, dp: u32) -> Result<i64, MoneyError> {
    let s = input.trim();
    if s.is_empty() {
        return Err(MoneyError::Empty);
    }
    let s = s.strip_prefix('+').unwrap_or(s);
    if s.starts_with('-') {
        return Err(MoneyError::Negative);
    }
    if !s.bytes().all(|b| b.is_ascii_digit() || b == b'.') || s.matches('.').count() > 1 {
        return Err(MoneyError::InvalidFormat);
    }

    let (integral, fractional) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if fractional.len() > dp as usize {
        return Err(MoneyError::TooManyDecimalPlaces {
            max: dp,
            got: fractional.len(),
        });
    }

    let integral_digits = integral.trim_start_matches('0');
    if integral_digits.len() + fractional.len() > MAX_TOTAL_DIGITS {
        return Err(MoneyError::Overflow {
            max_total_digits: MAX_TOTAL_DIGITS,
        });
    }

    let mut digits = String::with_capacity(integral_digits.len() + dp as usize);
    digits.push_str(integral_digits);
    digits.push_str(fractional);
    for _ in 0..(dp as usize - fractional.len()) {
        digits.push('0');
    }

    if digits.is_empty() {
        return Ok(0);
    }
    digits.parse::<i64>().map_err(|_| MoneyError::Overflow {
        max_total_digits: MAX_TOTAL_DIGITS,
    })
}

/// Format minor units as a grouped decimal string, e.g. `12345` with `dp = 2`
/// becomes `123.45` and `-50` becomes `-0.50`. Pure integer arithmetic.
pub fn format_minor(minor: i64, dp: u32) -> String {
    let negative = minor < 0;
    let magnitude = minor.unsigned_abs();
    let scale = 10u64.pow(dp);
    let integral = magnitude / scale;
    let fraction = magnitude % scale;

    let integral_str = integral.to_string();
    let mut grouped = String::with_capacity(integral_str.len() + integral_str.len() / 3);
    for (idx, ch) in integral_str.chars().enumerate() {
        if idx > 0 && (integral_str.len() - idx).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(ch);
    }

    if dp == 0 {
        return format!(
            "{negative_sign}{grouped}",
            negative_sign = if negative { "-" } else { "" }
        );
    }
    let fraction_str = format!("{fraction:0width$}", width = dp as usize);
    format!(
        "{sign}{grouped}.{fraction_str}",
        sign = if negative { "-" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exact_values() {
        assert_eq!(parse_decimal_to_minor("123.45", 2), Ok(12_345));
        assert_eq!(parse_decimal_to_minor("0.5", 2), Ok(50));
        assert_eq!(parse_decimal_to_minor("0.50", 2), Ok(50));
        assert_eq!(parse_decimal_to_minor("1234", 2), Ok(123_400));
        assert_eq!(parse_decimal_to_minor(" 123.45 ", 2), Ok(12_345));
        assert_eq!(parse_decimal_to_minor("+7", 2), Ok(700));
        assert_eq!(parse_decimal_to_minor("0", 2), Ok(0));
        assert_eq!(parse_decimal_to_minor("000123.4", 2), Ok(12_340));
    }

    #[test]
    fn parse_rejects_non_representable_input() {
        assert_eq!(parse_decimal_to_minor("", 2), Err(MoneyError::Empty));
        assert_eq!(parse_decimal_to_minor("   ", 2), Err(MoneyError::Empty));
        assert_eq!(
            parse_decimal_to_minor("0.005", 2),
            Err(MoneyError::TooManyDecimalPlaces { max: 2, got: 3 })
        );
        assert_eq!(parse_decimal_to_minor("-1", 2), Err(MoneyError::Negative));
        assert_eq!(
            parse_decimal_to_minor("1,234", 2),
            Err(MoneyError::InvalidFormat)
        );
        assert_eq!(
            parse_decimal_to_minor("1.2.3", 2),
            Err(MoneyError::InvalidFormat)
        );
        assert_eq!(
            parse_decimal_to_minor("abc", 2),
            Err(MoneyError::InvalidFormat)
        );
        assert_eq!(
            parse_decimal_to_minor("1e5", 2),
            Err(MoneyError::InvalidFormat)
        );
    }

    #[test]
    fn parse_rejects_overflow() {
        // 18 significant digits exceeds the 17-digit cap.
        let too_long = "9".repeat(18);
        assert_eq!(
            parse_decimal_to_minor(&too_long, 2),
            Err(MoneyError::Overflow {
                max_total_digits: MAX_TOTAL_DIGITS
            })
        );
        // Exactly 17 digits is the maximum representable value.
        let max_ok = format!("{}.99", "9".repeat(15));
        assert!(parse_decimal_to_minor(&max_ok, 2).is_ok());
    }

    #[test]
    fn format_round_trips() {
        assert_eq!(format_minor(12_345, 2), "123.45");
        assert_eq!(format_minor(50, 2), "0.50");
        assert_eq!(format_minor(123_400, 2), "1,234.00");
        assert_eq!(format_minor(-50, 2), "-0.50");
        assert_eq!(format_minor(0, 2), "0.00");
        assert_eq!(format_minor(1_234_567_890, 2), "12,345,678.90");
        // Round trip: format(parse(x)) preserves value for 2dp inputs.
        for input in ["123.45", "0.01", "999999999999999.99"] {
            let minor = parse_decimal_to_minor(input, 2).expect("valid input");
            let displayed = format_minor(minor, 2);
            assert_eq!(
                parse_decimal_to_minor(&displayed.replace(',', ""), 2).expect("round trip"),
                minor
            );
        }
    }
}
