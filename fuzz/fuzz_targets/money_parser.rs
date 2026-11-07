#![no_main]

//! Minimal fuzz target for money parsing (FBT, FAULT-04).
//! Invariants: parse never panics; round-trip preserves value when Ok;
//! non-UTF8 input is rejected without panic.

use flaps_core::money::{format_minor, parse_decimal_to_minor};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Cheap UTF-8 coercion; fuzzers generate many non-UTF8 sequences.
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    // Filter to ASCII digits / dot / sign to keep corpus focused.
    let s = s.trim();
    if s.len() > 64 {
        return;
    }

    let result = parse_decimal_to_minor(s, 2);

    // INV1: non-empty valid input never yields negative minor.
    if let Ok(minor) = result {
        assert!(minor >= 0, "negative minor from {s:?}");
    }

    // INV2: round-trip — format then parse returns same minor.
    if let Ok(minor) = parse_decimal_to_minor(s, 2) {
        let formatted = format_minor(minor, 2);
        let reparsed = parse_decimal_to_minor(&formatted.replace(',', ""), 2);
        assert_eq!(reparsed, Ok(minor), "round-trip failed: {s:?} -> {formatted} -> {reparsed:?}");
    }
});
