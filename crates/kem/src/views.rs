#![forbid(unsafe_code)]
//! 視圖重生成（M4）：index.md＋log.md＋dashboard.html 的單一實作。
//!
//! 從 main.rs 上移至函式庫（2026-11-17）：benchmark／測試／CLI 共用同一條
//! 重生成路徑——衍生視圖永不分叉。

use crate::filing::{load_claims, status_counts, write_index, write_log};
use crate::ledger::LedgerEntry;
use crate::schema::{Claim, KemError};
use chrono::Utc;
use std::path::Path;

/// finalize_views 的回傳：宣告、lint 發現數、待審佇列項目。
pub struct FinalizedViews {
    pub claims: Vec<Claim>,
    pub findings: usize,
    pub queue: Vec<(String, Vec<String>)>,
}

/// 共用視圖重生成：index.md＋log.md＋dashboard.html。
pub fn finalize_views(root: &Path, ledger: &Path) -> Result<FinalizedViews, KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let findings = crate::gov::lint(&claims, Utc::now());
    let findings_count = findings.len();
    write_index(root, &claims)?;
    write_log(root, ledger)?;
    // 審查佇列：僅未解項（gov::pending_queue 單一事實來源）。
    let queue_path = root.join(".kem/review-queue.jsonl");
    let queue_items = crate::gov::pending_queue(&queue_path)?;
    let ledger_entries = if ledger.exists() {
        crate::ledger::read_all(ledger)?
    } else {
        Vec::new()
    };
    let ledger_tail: Vec<(u64, String, String, String)> = ledger_entries
        .iter()
        .rev()
        .take(12)
        .map(|e| {
            (
                e.seq,
                format!("{:?}", e.action).to_lowercase(),
                e.claim_id.clone(),
                e.ts.to_rfc3339(),
            )
        })
        .collect();
    // 最新 T2 聯合裁決（actor=kem-t2）供治理面板。
    let ensemble = ledger_entries
        .iter()
        .rev()
        .find(|e| e.actor == "kem-t2")
        .and_then(|e| {
            let models: Vec<String> = e
                .detail
                .get("models_used")
                .and_then(|m| m.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let agreement = e.detail.get("agreement").and_then(|a| a.as_f64())?;
            let top = e
                .detail
                .get("top_score")
                .and_then(|t| t.as_u64())
                .map(|t| t as u8);
            (!models.is_empty()).then_some((models, agreement, top))
        });
    // 最新決策包（actor=kem-package）供治理面板（O4）。
    let latest_package = ledger_entries.iter().rev().find_map(|e| {
        if e.actor != "kem-package" || e.claim_id.is_empty() {
            return None;
        }
        let sha = e
            .detail
            .get("package_sha")
            .and_then(|s| s.as_str())
            .unwrap_or_default();
        (!sha.is_empty()).then(|| (e.claim_id.clone(), sha.to_string()))
    });
    crate::dashboard::write_dashboard(
        root,
        &claims,
        findings_count,
        &queue_items,
        &ledger_tail,
        ensemble,
        latest_package,
    )?;
    Ok(FinalizedViews {
        claims,
        findings: findings_count,
        queue: queue_items,
    })
}

/// 狀態卡計數薄封裝（與 status_counts 同源，避免呼叫端再匯入）。
pub fn lifecycle_counts(claims: &[Claim]) -> [(crate::schema::ClaimStatus, usize); 6] {
    status_counts(claims)
}
