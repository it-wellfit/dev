#![forbid(unsafe_code)]
//! M4 — Filing and generated views: claims dir (SSOT), index.md, log.md.
//!
//! Claims live as one JSON file each under `claims/`; `index.md` and
//! `log.md` are derived views, always rebuildable (KEM doc §4.2).

use chrono::Utc;
use std::path::Path;

use crate::schema::{Claim, ClaimStatus, KemError};

/// Load every claim file from `claims/`. Unparseable files are errors
/// (fail-closed, never silently skipped).
pub fn load_claims(claims_dir: &Path) -> Result<Vec<Claim>, KemError> {
    if !claims_dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut entries: Vec<_> =
        std::fs::read_dir(claims_dir)?.collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path)?;
        out.push(serde_json::from_str(&text)?);
    }
    Ok(out)
}

/// Persist one claim (SSOT write).
pub fn save_claim(claims_dir: &Path, claim: &Claim) -> Result<(), KemError> {
    std::fs::create_dir_all(claims_dir)?;
    let path = claims_dir.join(format!("{}.json", claim.id));
    std::fs::write(path, serde_json::to_string_pretty(claim)?)?;
    Ok(())
}

/// Regenerate `index.md`: retrieval-visible claims grouped by topic.
/// Draft/pending/retired are excluded (KEM doc §4.4).
pub fn write_index(root: &Path, claims: &[Claim]) -> Result<(), KemError> {
    let mut visible: Vec<&Claim> = claims.iter().filter(|c| c.status.retrievable()).collect();
    visible.sort_by(|a, b| a.id.cmp(&b.id));

    let mut lines = Vec::new();
    lines.push("# Knowledge Index (generated — do not edit)".to_string());
    lines.push(String::new());
    lines.push(format!(
        "- Generated: {} | Schema: {} | Visible claims: {}",
        Utc::now().to_rfc3339(),
        crate::SCHEMA_VERSION,
        visible.len()
    ));
    lines.push(String::new());

    // Group by topic, sorted.
    let mut by_topic: std::collections::BTreeMap<String, Vec<&Claim>> =
        std::collections::BTreeMap::new();
    for c in &visible {
        for t in &c.topics {
            by_topic.entry(t.clone()).or_default().push(c);
        }
    }
    for (topic, items) in &by_topic {
        lines.push(format!("## {topic}"));
        lines.push(String::new());
        lines.push("| ID | Type | Status | Title | Confidence |".to_string());
        lines.push("|----|------|--------|-------|------------|".to_string());
        for c in items {
            lines.push(format!(
                "| {} | {:?} | {:?} | {} | {:.2} |",
                c.id, c.claim_type, c.status, c.title, c.confidence
            ));
        }
        lines.push(String::new());
    }
    std::fs::write(root.join("index.md"), lines.join("\n"))?;
    Ok(())
}

/// Regenerate `log.md`: a human view of the ledger tail.
pub fn write_log(root: &Path, ledger_path: &Path) -> Result<(), KemError> {
    let entries = crate::ledger::read_all(ledger_path)?;
    let mut lines = Vec::new();
    lines.push("# Decision Log (generated — do not edit)".to_string());
    lines.push(String::new());
    lines.push(format!(
        "- Entries: {} | Verified: appended-only",
        entries.len()
    ));
    lines.push(String::new());
    lines.push("| Seq | Time (UTC) | Actor | Action | Claim |".to_string());
    lines.push("|-----|------------|-------|--------|-------|".to_string());
    for e in entries.iter().rev().take(200) {
        lines.push(format!(
            "| {} | {} | {} | {:?} | {} |",
            e.seq,
            e.ts.to_rfc3339(),
            e.actor,
            e.action,
            if e.claim_id.is_empty() {
                "—"
            } else {
                &e.claim_id
            }
        ));
    }
    std::fs::write(root.join("log.md"), lines.join("\n"))?;
    Ok(())
}

/// Count claims per status — feeds `kem status`.
pub fn status_counts(claims: &[Claim]) -> [(ClaimStatus, usize); 6] {
    use ClaimStatus::*;
    let mut out = [
        (Draft, 0),
        (PendingReview, 0),
        (Published, 0),
        (Superseded, 0),
        (Deprecated, 0),
        (Retired, 0),
    ];
    for c in claims {
        for slot in out.iter_mut() {
            if slot.0 == c.status {
                slot.1 += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClaimType, Evidence, Track};
    use chrono::TimeZone;

    fn sample(id: &str, status: ClaimStatus, topics: Vec<&str>) -> Claim {
        Claim {
            id: id.into(),
            claim_type: ClaimType::Fact,
            status,
            track: Track::B,
            confidence: 0.9,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            valid_to: None,
            recorded_at: Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap(),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "docs/x.md".into(),
                rev: "working-tree".into(),
                quote: "q".into(),
            }],
            topics: topics.into_iter().map(String::from).collect(),
            source: "raw/t/2026-11-07-a.md".into(),
            title: "t".into(),
            body: "b".into(),
        }
    }

    #[test]
    fn save_load_roundtrip_is_sorted_and_complete() {
        let dir = tempfile::tempdir().unwrap();
        let claims = dir.path().join("claims");
        save_claim(
            &claims,
            &sample("claim-20261107-002", ClaimStatus::Published, vec!["a"]),
        )
        .unwrap();
        save_claim(
            &claims,
            &sample("claim-20261107-001", ClaimStatus::Draft, vec!["a"]),
        )
        .unwrap();
        let loaded = load_claims(&claims).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "claim-20261107-001"); // sorted
    }

    #[test]
    fn index_excludes_non_retrievable() {
        let dir = tempfile::tempdir().unwrap();
        let claims = vec![
            sample("claim-20261107-001", ClaimStatus::Published, vec!["t"]),
            sample("claim-20261107-002", ClaimStatus::Draft, vec!["t"]),
            sample("claim-20261107-003", ClaimStatus::Retired, vec!["t"]),
        ];
        write_index(dir.path(), &claims).unwrap();
        let idx = std::fs::read_to_string(dir.path().join("index.md")).unwrap();
        assert!(idx.contains("claim-20261107-001"));
        assert!(!idx.contains("claim-20261107-002"));
        assert!(!idx.contains("claim-20261107-003"));
    }

    #[test]
    fn status_counts_match() {
        let claims = vec![
            sample("claim-20261107-001", ClaimStatus::Published, vec!["t"]),
            sample("claim-20261107-002", ClaimStatus::Published, vec!["t"]),
            sample("claim-20261107-003", ClaimStatus::Draft, vec!["t"]),
        ];
        let counts = status_counts(&claims);
        let published = counts
            .iter()
            .find(|(s, _)| *s == ClaimStatus::Published)
            .unwrap();
        assert_eq!(published.1, 2);
    }
}
