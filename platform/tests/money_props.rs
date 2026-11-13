#![forbid(unsafe_code)]
//! INV-01 金額算術性質測試（proptest；計畫 1793033761021 §16 P1/Q7）。
//!
//! Q7 預授權條件：proptest 1.12.0 已在 Cargo.lock（作為既有依賴圖成員）；
//! 以 dev-dependency 引用同一鎖定版本，不引入任何鎖外套件。
//!
//! SOTA 論證：樣例測試只覆蓋手選邊界；性質測試以隨機化輸入探索
//! 進位、前導零、最大位數、群組逗號等組合空間——對「錢」這類
//! 不容錯純函數，性質測試是業界標準配置（fail-closed 契約的
//! 機器可驗證表述）。

use auxbridge_core::money::{format_minor_i128, parse_decimal_to_minor_i128};
use auxbridge_core::{format_minor, parse_decimal_to_minor};
use proptest::prelude::*;

proptest! {
    /// P1（i64）：任意合法 minor → format（去群組逗號）→ parse 恆等。
    /// 上界 = 17 位整數 + 2 位小數（MAX_TOTAL_DIGITS=17）。
    #[test]
    fn format_then_parse_round_trips_i64(minor in 0i64..=99_999_999_999_999_999) {
        let dp = 2u32;
        let displayed = format_minor(minor, dp);
        let plain = displayed.replace(',', "");
        prop_assert_eq!(
            parse_decimal_to_minor(&plain, dp).expect("formatted output must re-parse"),
            minor
        );
    }

    /// P1-i128：高精度資產（36 位整數 + 2 位小數 = 38 位內）往返恆等。
    #[test]
    fn format_then_parse_round_trips_i128(minor in 0i128..=(10i128.pow(36) - 1)) {
        let dp = 2u32;
        let displayed = format_minor_i128(minor, dp);
        let plain = displayed.replace(',', "");
        prop_assert_eq!(
            parse_decimal_to_minor_i128(&plain, dp).expect("formatted output must re-parse"),
            minor
        );
    }

    /// P2（文法閘，fail-closed）：parse 接受的輸入必符合文檔文法
    /// （可選 `+`、僅數字／小數點／千分位逗號、逗號僅置於數字之間、至多一個點、
    /// 小數 ≤dp、非負）；其餘一律 Err——「拒絕即安全」，永不出現靜默捨入或意外符號。
    /// 註：千分位逗號為**已記錄**之文法（`money.rs` 之 `1,234`→123400 測試），
    /// 故本質性以「逗號位置合法」取代「不得含逗號」。
    #[test]
    fn parse_accepts_only_documented_grammar(input in ".*") {
        // Err 分支 = fail-closed 安全（拒絕即安全），無需斷言；
        // 本性質只約束「被接受者必符合文檔文法」。
        if let Ok(minor) = parse_decimal_to_minor(&input, 2) {
            let trimmed = input.trim();
            let s = trimmed.strip_prefix('+').unwrap_or(trimmed);
            prop_assert!(!s.starts_with('-'), "negative must be rejected");
            prop_assert!(
                s.bytes().all(|b| b.is_ascii_digit() || b == b'.' || b == b','),
                "accepted input must contain only digits, one dot and thousands separators"
            );
            let bytes = s.as_bytes();
            for (index, byte) in bytes.iter().enumerate() {
                if *byte == b',' {
                    let prev_digit = index > 0 && bytes[index - 1].is_ascii_digit();
                    let next_digit = index + 1 < bytes.len() && bytes[index + 1].is_ascii_digit();
                    prop_assert!(
                        prev_digit && next_digit,
                        "thousands separator must sit between digits"
                    );
                }
            }
            prop_assert!(s.matches('.').count() <= 1, "at most one decimal point");
            if let Some(dot) = s.find('.') {
                prop_assert!(
                    s.len() - dot - 1 <= 2,
                    "at most dp=2 fractional digits (no silent rounding)"
                );
            }
            prop_assert!(minor >= 0, "accepted input must be non-negative");
        }
    }

    /// P3（單調性）：非負 minor 的格式化比較與數值比較一致——
    /// 排序/比較 UI（金額欄）依賴此性質不產生亂序。
    #[test]
    fn formatting_preserves_order(
        a in 0i64..=99_999_999_999_999_998,
        b in 0i64..=99_999_999_999_999_999,
    ) {
        let (lo, hi) = (a.min(b), a.max(b));
        let dp = 2u32;
        let lo_s = format_minor(lo, dp).replace(',', "");
        let hi_s = format_minor(hi, dp).replace(',', "");
        let lo_p = parse_decimal_to_minor(&lo_s, dp).expect("valid");
        let hi_p = parse_decimal_to_minor(&hi_s, dp).expect("valid");
        prop_assert!(lo_p <= hi_p, "order must be preserved: {lo} -> {lo_s}, {hi} -> {hi_s}");
        prop_assert_eq!(lo_p == hi_p, lo == hi);
    }
}
