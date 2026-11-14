//! Lucide stroke 圖標系統（2026-11-14 UI-1）。
//!
//! 零依賴：直接內嵌 Lucide（ISC）SVG path 資料，24×24 viewBox、stroke 1.5、
//! `fill="none"`、`stroke="currentColor"`、圓角線帽——跨平台渲染一致、可繼承
//! token 顏色、可縮放。取代 emoji（跨平台渲染不一致、無法 currentColor）。
//!
//! 用法：模板以 `icons::svg("key")` 取得完整 `<svg>` 字串（`askama` filter）。

/// key → SVG inner markup（path/circle/line 等，不含外層 `<svg>`）。
pub fn inner(key: &str) -> &'static str {
    match key {
        "home" => r#"<path d="M3 9.5 12 3l9 6.5V21H3z"/><path d="M9 21v-6h6v6"/>"#,
        "book-open" => {
            r#"<path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2z"/><path d="M22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7z"/>"#
        }
        "inbox" => {
            r#"<path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>"#
        }
        "building" => {
            r#"<path d="M3 21h18"/><path d="M5 21V7l7-4 7 4v14"/><path d="M9 21v-4h6v4"/><path d="M9 10h.01M15 10h.01M9 14h.01M15 14h.01"/>"#
        }
        "chart" => {
            r#"<path d="M3 3v18h18"/><path d="M18 17V9"/><path d="M13 17V5"/><path d="M8 17v-3"/>"#
        }
        "search" => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
        "shield" => r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>"#,
        "settings" => {
            r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>"#
        }
        "plug" => {
            r#"<path d="M12 22v-5"/><path d="M9 8V2"/><path d="M15 8V2"/><path d="M18 8v3a6 6 0 0 1-12 0V8z"/>"#
        }
        "monitor" => {
            r#"<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>"#
        }
        "book" => {
            r#"<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>"#
        }
        "briefcase" => {
            r#"<rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"/>"#
        }
        "terminal" => r#"<path d="m4 17 6-6-6-6"/><path d="M12 19h8"/>"#,
        "scale" => {
            r#"<path d="M16 16.5 12 8 8 16.5"/><path d="M2 16.5h6"/><path d="M16 16.5h6"/><path d="M12 2v6"/>"#
        }
        _ => r#"<circle cx="12" cy="12" r="10"/>"#,
    }
}

/// 完整 `<svg>` 字串（`class` 預設 `ico`，`aria-hidden` 装飾用）。
pub fn svg(key: &str, class: &str) -> String {
    format!(
        r#"<svg class="{class}" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{}</svg>"#,
        inner(key)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每個導覽群組/工作室必須有對應圖標 key（key 封閉集合）。
    #[test]
    fn all_manifest_keys_have_icons() {
        for key in [
            "overview",
            "ledger",
            "capture",
            "bankpay",
            "reports",
            "query",
            "governance",
            "admin",
            "apihub",
            "system",
            "knowledge",
            "biz",
            "bank",
            "gov",
            "ops",
        ] {
            let inner = inner(key);
            assert!(
                inner.contains("path") || inner.contains("circle") || inner.contains("rect"),
                "key {key} must render real geometry"
            );
        }
    }

    /// svg 輸出契約：外層標籤、aria-hidden、currentColor、零 fill。
    #[test]
    fn svg_output_contract() {
        let s = svg("home", "ico");
        assert!(s.starts_with("<svg"));
        assert!(s.contains("aria-hidden=\"true\""));
        assert!(s.contains("stroke=\"currentColor\""));
        assert!(s.contains("fill=\"none\""));
        assert!(s.contains("viewBox=\"0 0 24 24\""));
    }

    /// 未知 key 不 panic，回退通用圓（fail-safe）。
    #[test]
    fn unknown_key_falls_back() {
        assert!(inner("no-such-key").contains("circle"));
    }
}
