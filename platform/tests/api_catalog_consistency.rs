#![forbid(unsafe_code)]
//! G4.4 (D2-06) — API catalog ↔ route registration consistency.
//!
//! `apihub::registry()` is the single source of truth for the endpoint catalog.
//! Every entry must correspond to a real `.route(...)` registration in the
//! router modules; the catalog must not describe a retired backend (Tantivy).

const API_SRC: &str = include_str!("../src/api.rs");
const CROSS_SRC: &str = include_str!("../src/cross_module_api.rs");

/// Strip whitespace so `.route(\n    "/path",\n ...)` matches `.route("/path"`.
fn compact(src: &str) -> String {
    src.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn every_catalog_entry_is_registered() {
    let api = compact(API_SRC);
    let cross = compact(CROSS_SRC);
    let mut missing: Vec<String> = Vec::new();
    for spec in platform::apihub::registry() {
        let needle = format!(".route(\"{}\"", spec.path);
        if !api.contains(&needle) && !cross.contains(&needle) {
            missing.push(format!("{} {}", spec.method, spec.path));
        }
    }
    assert!(
        missing.is_empty(),
        "catalog entries without a route registration: {missing:?}"
    );
}

#[test]
fn catalog_has_no_stale_search_backend() {
    for spec in platform::apihub::registry() {
        assert!(
            !spec.description.contains("Tantivy"),
            "stale search backend in catalog for {}",
            spec.path
        );
    }
}
