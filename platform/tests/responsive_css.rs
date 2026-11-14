#![forbid(unsafe_code)]
//! G4.10 (D3-05) — responsive contract for the design system.
//!
//! Mechanical gate: the CSS must declare the 320px-class breakpoint, the
//! 640/768/1024 breakpoints, and container-query support. A manual/browser
//! matrix (320/375/768/1024/1440/1920) is recorded in the operations doc.

const CSS: &str = include_str!("../../web/assets/design-system.css");

#[test]
fn declares_required_breakpoints() {
    for needle in [
        "@media (max-width: 360px)",
        "@media (max-width: 640px)",
        "@media (max-width: 768px)",
        "@media (max-width: 1024px)",
    ] {
        assert!(CSS.contains(needle), "missing breakpoint: {needle}");
    }
}

#[test]
fn declares_container_query_support() {
    assert!(
        CSS.contains("container-type: inline-size"),
        "missing container-type: inline-size"
    );
    assert!(
        CSS.contains("@container page (max-width: 480px)"),
        "missing named container query"
    );
}

#[test]
fn narrow_screen_rules_exist() {
    // 320px-class: single column, full-width buttons, smaller KPI numerals.
    let start = CSS.find("@media (max-width: 360px)").expect("360px block");
    let block = &CSS[start..(start + 800).min(CSS.len())];
    assert!(block.contains(".kpi-grid"), "narrow: kpi grid rule");
    assert!(block.contains(".btn"), "narrow: button rule");
    assert!(
        block.contains(".container"),
        "narrow: container padding rule"
    );
}
