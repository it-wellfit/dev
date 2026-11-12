#![forbid(unsafe_code)]
//! B-5.2（計畫 1793033761021 §15）：WCAG 2.2 AA 對比**數值化**驗證。
//!
//! 設計系統（web/assets/design-system.css）是 Token SSOT；本測試把
//! 「對比 ≥ 4.5:1」從文件承諾變成 CI 閘門（free SOTA：WCAG 2.2 相對
//! 亮度公式直接實作，零外部依賴）。
//!
//! 契約：徽章**文字**用 `-strong` 級 Token（2026-11-12 導入——base 級
//! 在 muted 背景上實測僅 ~3:1，違 AA；base 色保留給圖示/邊框等非文字用途）。
//! 淺色模式驗 :root 字面值；深色模式驗 rgba muted 疊加暗色 surface 後
//! 的**複合背景**對比。

const CSS: &str = include_str!("../../web/assets/design-system.css");

/// 只在淺色 :root 區塊（暗色 @media 之前）找 token，避免取到覆寫值。
fn light_token(name: &str) -> String {
    let dark_start = CSS
        .find("@media (prefers-color-scheme: dark)")
        .expect("dark mode block marker");
    let needle = format!("--{name}:");
    let start = CSS[..dark_start]
        .find(&needle)
        .unwrap_or_else(|| panic!("token --{name} not defined in light :root"))
        + needle.len();
    let rest = &CSS[start..];
    let end = rest.find(';').expect("token value terminator");
    rest[..end].trim().to_string()
}

/// 以大括號配對取出暗色 @media 區塊，並在其中找 token。
fn dark_token(name: &str) -> String {
    let start = CSS
        .find("@media (prefers-color-scheme: dark)")
        .expect("dark mode block marker");
    let open = CSS[start..].find('{').expect("dark block opening brace") + start;
    let mut depth = 0usize;
    let mut end = CSS.len();
    for (i, c) in CSS[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let scope = &CSS[open..end];
    let needle = format!("--{name}:");
    let pos = scope
        .find(&needle)
        .unwrap_or_else(|| panic!("token --{name} not defined in dark block"))
        + needle.len();
    let rest = &scope[pos..];
    let stop = rest.find(';').expect("token value terminator");
    rest[..stop].trim().to_string()
}

fn hex_rgb(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim().trim_start_matches('#');
    assert!(hex.len() == 6, "expected #rrggbb, got #{hex}");
    let ch = |i: usize| -> f64 {
        u8::from_str_radix(&hex[i..i + 2], 16).expect("hex channel") as f64 / 255.0
    };
    (ch(0), ch(2), ch(4))
}

/// WCAG 2.2 相對亮度（sRGB 線性化）。
fn luminance(rgb: (f64, f64, f64)) -> f64 {
    let lin = |c: f64| -> f64 {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = rgb;
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// WCAG 對比比（≥4.5 為 AA 正常文字門檻）。
fn contrast(fg: (f64, f64, f64), bg: (f64, f64, f64)) -> f64 {
    let (l1, l2) = (luminance(fg), luminance(bg));
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// rgba() 字面值 → (r, g, b, a)（0..1）。
fn parse_rgba(rgba: &str) -> (f64, f64, f64, f64) {
    let inner = rgba
        .trim()
        .trim_start_matches("rgba(")
        .trim_end_matches(')');
    let parts: Vec<f64> = inner
        .split(',')
        .map(|p| p.trim().parse().expect("rgba channel"))
        .collect();
    assert!(parts.len() == 4, "expected rgba(r, g, b, a)");
    (
        parts[0] / 255.0,
        parts[1] / 255.0,
        parts[2] / 255.0,
        parts[3],
    )
}

/// 半透明前景疊加在不透明背景上的有效色。
fn composite(fg_rgba: (f64, f64, f64, f64), bg: (f64, f64, f64)) -> (f64, f64, f64) {
    let (fr, fgc, fb, a) = fg_rgba;
    (
        a * fr + (1.0 - a) * bg.0,
        a * fgc + (1.0 - a) * bg.1,
        a * fb + (1.0 - a) * bg.2,
    )
}

const AA_NORMAL: f64 = 4.5;

#[test]
fn semantic_badge_strong_pairs_meet_aa_light() {
    let pairs = [
        ("success", "成功"),
        ("warning", "警告"),
        ("error", "錯誤"),
        ("info", "資訊"),
    ];
    for (semantic, label) in pairs {
        let fg = light_token(&format!("color-{semantic}-strong"));
        let bg = light_token(&format!("color-{semantic}-muted"));
        let ratio = contrast(hex_rgb(&fg), hex_rgb(&bg));
        assert!(
            ratio >= AA_NORMAL,
            "badge {label} 文字（{fg} on {bg}）對比 {ratio:.2}:1 < {AA_NORMAL}:1——違 WCAG 2.2 AA；\
             -strong Token 須再加深（D-3 Human-centered：可讀性優先）"
        );
    }
}

#[test]
fn semantic_badge_strong_pairs_meet_aa_dark_composite() {
    let dark_surface = dark_token("color-surface");
    assert!(
        dark_surface.starts_with('#'),
        "dark surface must be literal hex"
    );
    let surface_rgb = hex_rgb(&dark_surface);
    let pairs = [
        ("success", "成功"),
        ("warning", "警告"),
        ("error", "錯誤"),
        ("info", "資訊"),
    ];
    for (semantic, label) in pairs {
        let fg = dark_token(&format!("color-{semantic}-strong"));
        let muted = dark_token(&format!("color-{semantic}-muted"));
        assert!(
            muted.starts_with("rgba("),
            "dark muted must be rgba composite token, got {muted}"
        );
        let bg = composite(parse_rgba(&muted), surface_rgb);
        let ratio = contrast(hex_rgb(&fg), bg);
        assert!(
            ratio >= AA_NORMAL,
            "badge {label} 文字（{fg} on 複合{bg:?}）對比 {ratio:.2}:1 < {AA_NORMAL}:1——違 WCAG 2.2 AA（深色模式）"
        );
    }
}

#[test]
fn body_text_meets_aa() {
    let fg = light_token("color-text-primary");
    let bg = light_token("color-surface");
    let ratio = contrast(hex_rgb(&fg), hex_rgb(&bg));
    assert!(
        ratio >= AA_NORMAL,
        "正文（{fg} on {bg}）對比 {ratio:.2}:1 < {AA_NORMAL}:1——違 WCAG 2.2 AA"
    );
}
