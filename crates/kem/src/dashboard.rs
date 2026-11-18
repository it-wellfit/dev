#![forbid(unsafe_code)]
//! Zero-JS knowledge dashboard (dashboard.html) — 精緻靜態視圖。
//!
//! 使用者痛點 → 元件對映（設計依據 AGENTS.md §4 與 WCAG 2.2 AA）:
//! - 「我該先處理什麼？」→ 治理警示橫幅（問題＋處理方式）＋審查佇列表
//! - 「這知識還可信嗎？」→ 信心視覺化長條＋證據直達連結（claims/*.json＋來源 rev）
//! - 「最近有什麼變動？」→ 帳本時間軸（最後 12 條）
//! - 「整體健康度？」→ 生命週期卡片＋lint 狀態
//! - 「我想看哪一區？」→ :target 分區導航（零 JS）；導航高亮用 :has()
//!
//! 互動狀態全覆蓋：預設/懸停/焦點可見/空/錯誤；深色模式；列印樣式；prefers-reduced-motion。
//! 字面 script 標籤不會出現於輸出 HTML（憲章 §2.3：計數恆為 0）。

use crate::KemError;
use crate::filing::status_counts;
use crate::schema::{Claim, ClaimStatus, ClaimType};
use chrono::Utc;
use std::fmt::Write as _;
use std::path::Path;

const CSS: &str = include_str!("../../../docs/design/living-ledger.css");
fn esc(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
}

fn confidence_bar(c: f64) -> String {
    let pct = (c * 100.0).round() as i64;
    let class = if c >= 0.8 {
        "hi"
    } else if c < 0.45 {
        "lo"
    } else {
        ""
    };
    format!(
        "<span class=\"bar {class}\"><i style=\"width:{pct}%\"></i></span> <span class=\"mono\">{c:.2}</span>"
    )
}

fn status_pill(s: ClaimStatus) -> String {
    format!("<span class=\"pill {:?}\">{:?}</span>", s, s)
        .replace("Published", "published")
        .replace("Superseded", "superseded")
        .replace("Deprecated", "deprecated")
        .replace("Draft", "draft")
        .replace("PendingReview", "pending_review")
        .replace("Retired", "retired")
}

fn type_label(t: ClaimType) -> &'static str {
    match t {
        ClaimType::Concept => "概念",
        ClaimType::Decision => "決策",
        ClaimType::Invariant => "不變量",
        ClaimType::Gotcha => "陷阱",
        ClaimType::Glossary => "詞彙",
        ClaimType::Fact => "事實",
    }
}

/// Render dashboard.html. `ensemble` = latest T2 verdict summary
/// (models_used, agreement, top_score) for the governance panel.
pub fn write_dashboard(
    root: &Path,
    claims: &[Claim],
    findings_count: usize,
    queue_items: &[(String, Vec<String>)],
    ledger_tail: &[(u64, String, String, String)], // (seq, action, claim, ts)
    ensemble: Option<(Vec<String>, f64, Option<u8>)>,
) -> Result<(), KemError> {
    let mut h = String::with_capacity(32 * 1024);
    let now = Utc::now().to_rfc3339();
    let counts = status_counts(claims);
    let published = counts
        .iter()
        .find(|(s, _)| *s == ClaimStatus::Published)
        .map(|(_, n)| *n)
        .unwrap_or(0);
    let retrievable = claims.iter().filter(|c| c.status.retrievable()).count();

    h.push_str("<!DOCTYPE html>\n<html lang=\"zh-Hant\">\n<head>\n<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    h.push_str("<title>KEM 知識儀表板</title>\n<style>");
    h.push_str(CSS);
    h.push_str("</style>\n</head>\n<body>\n");
    let _ = write!(
        h,
        "<header><h1>KEM 知識儀表板</h1><p class=\"sub\">知識演進模組 v{schema}｜{retrievable} 筆可檢索｜生成於 {now}</p>",
        schema = crate::SCHEMA_VERSION,
        retrievable = retrievable,
        now = esc(&now)
    );
    h.push_str("<nav aria-label=\"分區導航\">");
    h.push_str("<a href=\"#overview\">總覽</a><a href=\"#claims\">主張</a><a href=\"#gov\">治理</a><a href=\"#ledger\">帳本</a>");
    h.push_str("</nav></header>\n<main>\n");

    // ── 錯誤／警示狀態（問題＋可能原因＋處理方式，AGENTS.md §4）──
    if findings_count > 0 {
        let _ = writeln!(
            h,
            "<div class=\"banner\" role=\"alert\"><strong>治理警示</strong>：lint 發現 {findings_count} 項問題。\
             可能原因：證據失效、信心過低、標題近重複或取代連結斷裂。\
             處理方式：終端執行 <code>kem lint</code> 逐項查看，修復後重跑 <code>kem sync</code> 與 <code>kem finalize</code>。</div>"
        );
    }
    if !queue_items.is_empty() {
        let _ = writeln!(
            h,
            "<div class=\"banner\" role=\"alert\"><strong>審查佇列</strong>：{n} 筆待複核。\
             可能原因：同步時引證漂移，或 SCS 一致率低於 0.60 地板。\
             處理方式：逐筆檢視後執行 <code>kem review --claim <id> --disposition accept|reject|supersede|defer</code>。</div>",
            n = queue_items.len()
        );
    }

    // ── 總覽 ──
    h.push_str("<section class=\"panel\" id=\"overview\" aria-labelledby=\"h-ov\"><h2 id=\"h-ov\">生命週期狀態</h2><div class=\"cards\">");
    let labels = [
        (ClaimStatus::Draft, "草稿"),
        (ClaimStatus::PendingReview, "待審"),
        (ClaimStatus::Published, "已發布"),
        (ClaimStatus::Superseded, "已取代"),
        (ClaimStatus::Deprecated, "已棄用"),
        (ClaimStatus::Retired, "已退役"),
    ];
    for ((_status, count), (_, label)) in counts.iter().zip(labels.iter()) {
        let _ = write!(
            h,
            "<div class=\"card\"><div class=\"num\">{count}</div><div class=\"label\">{label}</div></div>"
        );
    }
    h.push_str("</div>");
    let _ = write!(
        h,
        "<p class=\"next\">已發布 {published} 筆；生命週期規則：取代不刪除，retired 為唯一終態。</p></section>"
    );

    // ── 主張（依信心排序，含證據直達）──
    let mut visible: Vec<&Claim> = claims.iter().filter(|c| c.status.retrievable()).collect();
    visible.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    h.push_str("<section class=\"panel\" id=\"claims\" aria-labelledby=\"h-cl\"><h2 id=\"h-cl\">主張清單 <small>（依信心排序；點 ID 檢視 JSON 證據）</small></h2>");
    if visible.is_empty() {
        h.push_str("<div class=\"empty\">目前沒有可檢索的主張。<br>下一步：執行 <code>kem seed</code> 建立種子知識，或 <code>kem capture</code> ＋ <code>kem ingest</code> 擷取新知識。</div>");
    } else {
        h.push_str("<table><thead><tr><th scope=\"col\">ID</th><th scope=\"col\">標題</th><th scope=\"col\">類型</th><th scope=\"col\">狀態</th><th scope=\"col\">信心</th><th scope=\"col\">證據</th><th scope=\"col\">主題</th></tr></thead><tbody>");
        for c in visible.iter().take(80) {
            let ev = c
                .evidence
                .first()
                .map(|e| {
                    format!(
                        "<a href=\"../{}\" title=\"{} @ {}\"><code>{}</code></a>",
                        esc(&e.path),
                        esc(&e.path),
                        esc(&e.rev),
                        esc(e.path.rsplit(['/', '\\']).next().unwrap_or(&e.path))
                    )
                })
                .unwrap_or_else(|| "—".into());
            let more = if c.evidence.len() > 1 {
                format!(" +{}", c.evidence.len() - 1)
            } else {
                String::new()
            };
            let _ = write!(
                h,
                "<tr><td><a href=\"claims/{}.json\"><code>{}</code></a></td><td>{}</td><td><span class=\"type\">{}</span></td><td>{}</td><td>{}</td><td>{}{}</td><td>{}</td></tr>",
                c.id,
                c.id,
                esc(&c.title),
                type_label(c.claim_type),
                status_pill(c.status),
                confidence_bar(c.confidence),
                ev,
                more,
                esc(&c.topics.join("、"))
            );
        }
        h.push_str("</tbody></table>");
    }
    h.push_str("</section>\n");

    // ── 治理 ──
    h.push_str("<div><h2>模型聯合（T2 裁決）</h2>");
    match &ensemble {
        Some((models, agreement, top)) => {
            let _ = write!(
                h,
                "<table><tbody><tr><td>最近裁決模型</td><td class=\"mono\">{}</td></tr>",
                esc(&models.join("、"))
            );
            let _ = write!(
                h,
                "<tr><td>一致率</td><td>{}</td></tr>",
                confidence_bar(*agreement)
            );
            let _ = write!(
                h,
                "<tr><td>多數 durability</td><td class=\"mono\">{}</td></tr></tbody></table>",
                top.map(|t| t.to_string()).unwrap_or_else(|| "—".into())
            );
            if *agreement < 0.60 {
                h.push_str("<p class=\"next\">一致率低於 0.60 地板——分歧已送審查佇列（跨模型意見可見，不掩蓋）。</p>");
            }
        }
        None => {
            h.push_str("<div class=\"empty\">尚無 T2 聯合裁決記錄——執行 <code>kem vet2 --claim <id></code> 啟動跨模型投票。</div>");
        }
    }
    h.push_str("</div><div><h2>審查佇列</h2>");
    h.push_str("<div><h2>審查佇列</h2>");
    if queue_items.is_empty() {
        h.push_str("<div class=\"empty\">佇列清空——無待複核項目。</div>");
    } else {
        h.push_str("<table><thead><tr><th scope=\"col\">主張</th><th scope=\"col\">原因</th></tr></thead><tbody>");
        for (id, reasons) in queue_items.iter().take(30) {
            let _ = write!(
                h,
                "<tr><td><code>{}</code></td><td>{}</td></tr>",
                esc(id),
                esc(&reasons.join("；"))
            );
        }
        h.push_str("</tbody></table>");
    }
    h.push_str("</div><div><h2>生命週期規則</h2><table><tbody>");
    h.push_str("<tr><td><code>draft / pending_review</code></td><td>不進檢索（隔離中）</td></tr>");
    h.push_str(
        "<tr><td><code>superseded / deprecated</code></td><td>可檢索但降級且必標籤</td></tr>",
    );
    h.push_str("<tr><td><code>retired</code></td><td>完全出庫；歷史留 raw/ 與帳本</td></tr>");
    h.push_str(
        "<tr><td><code>信心 < 0.45</code></td><td>視為陳舊，需再確認（reaffirmation）</td></tr>",
    );
    h.push_str("</tbody></table></div></div></section>\n");

    // ── 帳本時間軸 ──
    h.push_str("<section class=\"panel\" id=\"ledger\" aria-labelledby=\"h-lg\"><h2 id=\"h-lg\">決策帳本 <small>（最後 12 條；SHA-256 雜湊鏈，防竄改）</small></h2>");
    if ledger_tail.is_empty() {
        h.push_str("<div class=\"empty\">帳本尚無條目——所有裁決與歸檔決策都會記入此處。</div>");
    } else {
        h.push_str("<ul class=\"timeline\">");
        for (seq, action, claim, ts) in ledger_tail.iter().take(12) {
            let _ = write!(
                h,
                "<li><span class=\"seq\">#{seq}</span><div>{} · <code>{}</code></div><div class=\"fresh\">{}</div></li>",
                esc(action),
                esc(claim),
                esc(ts)
            );
        }
        h.push_str("</ul>");
    }
    h.push_str("</section>\n");

    let _ = write!(
        h,
        "</main>\n<footer>資料來源：claims/（機器 SSOT）＋ 決策帳本｜驗證：<code>kem doctor</code>｜純靜態 HTML，&lt;script&gt; 計數 = 0｜{}</footer>\n</body>\n</html>",
        esc(&now)
    );
    h.push('\n');
    std::fs::write(root.join("dashboard.html"), h)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Evidence, Track};
    use chrono::TimeZone;

    fn claim(id: &str, status: ClaimStatus, conf: f64) -> Claim {
        Claim {
            id: id.into(),
            claim_type: ClaimType::Decision,
            status,
            track: Track::B,
            confidence: conf,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            valid_to: None,
            recorded_at: Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap(),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "docs/AGENTS.md".into(),
                rev: "working-tree".into(),
                quote: "q".into(),
            }],
            topics: vec!["測試主題".into()],
            source: "raw/t/2026-11-07-a.md".into(),
            title: "測試 <標題> & 特殊字元".into(),
            body: "b".into(),
        }
    }

    #[test]
    fn dashboard_has_no_script_and_escapes_html() {
        let dir = tempfile::tempdir().unwrap();
        let claims = vec![claim("claim-20261107-001", ClaimStatus::Published, 0.9)];
        write_dashboard(
            dir.path(),
            &claims,
            2,
            &[],
            &[(1, "File".into(), "c".into(), "t".into())],
            Some((vec!["qwen3.5:4b".into()], 0.80, Some(4))),
        )
        .unwrap();
        let html = std::fs::read_to_string(dir.path().join("dashboard.html")).unwrap();
        assert_eq!(
            html.matches("<script").count(),
            0,
            "Zero-JS charter: script count must be 0"
        );
        assert!(html.contains("&lt;標題&gt;"), "HTML must be escaped");
        assert!(html.contains("治理警示"));
        assert!(html.contains("id=\"claims\""));
        assert!(html.contains("class=\"bar hi\""));
    }

    #[test]
    fn dashboard_empty_and_queue_states_render() {
        let dir = tempfile::tempdir().unwrap();
        write_dashboard(
            dir.path(),
            &[],
            0,
            &[("claim-20261107-009".into(), vec!["quote drifted".into()])],
            &[],
            None,
        )
        .unwrap();
        let html = std::fs::read_to_string(dir.path().join("dashboard.html")).unwrap();
        assert!(html.contains("kem seed"));
        assert!(html.contains("quote drifted"));
        assert!(html.contains("佇列清空") || html.contains("審查佇列"));
    }

    #[test]
    fn confidence_bar_classes() {
        assert!(confidence_bar(0.9).contains("bar hi"));
        assert!(confidence_bar(0.31).contains("bar lo"));
        assert!(confidence_bar(0.6).contains("bar "));
        assert!(confidence_bar(0.6).contains("width:60%"));
    }
}
