#![forbid(unsafe_code)]
//! M7 — Governance: lint, doctor, status, review-queue dispositions.
//!
//! Lint rules (KEM doc §8 V-tests feed here):
//! - L1 published claim without retrievable evidence file (grounding broken)
//! - L2 supersedes reference pointing at unknown/retired successor mismatch
//! - L3 confidence below the stale threshold on a published claim
//! - L4 duplicate topics+title near-collision (redundant pages)
//! - L5 superseded/deprecated claim missing successor label in title view

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::schema::{Claim, ClaimStatus, KemError};
use crate::vetting::token_overlap;

/// Confidence below this on a published claim = stale (KEM doc §4.5 floor).
pub const STALE_THRESHOLD: f64 = 0.45;
/// Title near-collision overlap for L4.
pub const TITLE_OVERLAP: f64 = 0.9;

/// One lint finding, machine-checkable and actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub rule: &'static str,
    pub claim_id: String,
    pub detail: String,
}

/// Run all lint rules over the claim store. Returns findings; empty = clean.
pub fn lint(claims: &[Claim], now: DateTime<Utc>) -> Vec<Finding> {
    let mut out = Vec::new();
    let ids: std::collections::BTreeSet<&str> = claims.iter().map(|c| c.id.as_str()).collect();

    for c in claims {
        // L1: grounding of published claims re-checked here is cheap; the
        // authoritative check lives in sync — lint flags the structural gap.
        if c.status == ClaimStatus::Published && c.evidence.is_empty() {
            out.push(Finding {
                rule: "L1",
                claim_id: c.id.clone(),
                detail: "published without evidence".into(),
            });
        }
        // L2: superseded claims must have a known successor.
        if c.status == ClaimStatus::Superseded {
            match &c.superseded_by {
                None => out.push(Finding {
                    rule: "L2",
                    claim_id: c.id.clone(),
                    detail: "superseded without superseded_by".into(),
                }),
                Some(s) if !ids.contains(s.as_str()) => out.push(Finding {
                    rule: "L2",
                    claim_id: c.id.clone(),
                    detail: format!("superseded_by unknown id {s}"),
                }),
                _ => {}
            }
        }
        // L3: stale published claims need reaffirmation.
        if c.status == ClaimStatus::Published && c.confidence < STALE_THRESHOLD {
            out.push(Finding {
                rule: "L3",
                claim_id: c.id.clone(),
                detail: format!("confidence {:.2} below {STALE_THRESHOLD}", c.confidence),
            });
        }
        // L4: near-colliding titles on the same status (redundancy signal).
        for other in claims {
            if other.id <= c.id || other.status != c.status {
                continue;
            }
            let overlap = token_overlap(&c.title, &other.title);
            if overlap >= TITLE_OVERLAP {
                out.push(Finding {
                    rule: "L4",
                    claim_id: c.id.clone(),
                    detail: format!("title near-collision with {}", other.id),
                });
            }
        }
    }
    let _ = now; // reserved for time-based rules in v2
    out
}

/// Doctor: deep health check — ledger chain + store coherence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub ledger_entries: usize,
    pub ledger_ok: bool,
    pub claims: usize,
    pub findings: usize,
    pub review_queue: usize,
}

/// Pending review-queue items (unresolved lines only — lines carrying a
/// `disposition` field are resolved history). Single source of truth for
/// both `doctor` and the dashboard/report views (review defect #3).
pub fn pending_queue(queue_path: &Path) -> Result<Vec<(String, Vec<String>)>, KemError> {
    if !queue_path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(queue_path)?;
    let mut out = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v.get("disposition").is_some() {
                continue; // resolved entries are history, not pending
            }
            let id = v
                .get("claim_id")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let reasons = v
                .get("reasons")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|r| r.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            out.push((id, reasons));
        }
    }
    Ok(out)
}

pub fn doctor(root: &Path, ledger_path: &Path, queue_path: &Path) -> Result<Health, KemError> {
    let ledger_entries = if ledger_path.exists() {
        crate::ledger::read_all(ledger_path)?.len()
    } else {
        0
    };
    let ledger_ok = if ledger_path.exists() {
        crate::ledger::verify(ledger_path).is_ok()
    } else {
        true
    };
    let claims = crate::filing::load_claims(&root.join("claims"))?;
    let findings = lint(&claims, Utc::now()).len();
    let review_queue = pending_queue(queue_path)?.len();
    Ok(Health {
        ledger_entries,
        ledger_ok,
        claims: claims.len(),
        findings,
        review_queue,
    })
}

/// Review-queue disposition (KEM doc M7: accept/reject/supersede/defer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Accept,
    Reject,
    Supersede,
    Defer,
}

/// Record a review decision: appends to the queue file as a resolved item
/// (the queue stays append-only; resolution is a new line, never an edit).
pub fn record_disposition(
    queue_path: &Path,
    claim_id: &str,
    disposition: Disposition,
    reviewer: &str,
    note: &str,
    now: DateTime<Utc>,
) -> Result<(), KemError> {
    let item = serde_json::json!({
        "claim_id": claim_id,
        "disposition": disposition,
        "reviewer": reviewer,
        "note": note,
        "resolved_at": now.to_rfc3339(),
    });
    if let Some(parent) = queue_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(queue_path)?;
    f.write_all(serde_json::to_string(&item)?.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClaimType, Evidence, Track};
    use chrono::TimeZone;

    fn claim(id: &str, status: ClaimStatus, conf: f64) -> Claim {
        Claim {
            id: id.into(),
            claim_type: ClaimType::Fact,
            status,
            track: Track::B,
            confidence: conf,
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
            topics: vec!["t".into()],
            source: "raw/t/2026-11-07-a.md".into(),
            title: format!("title {id}"),
            body: "b".into(),
        }
    }

    #[test]
    fn clean_store_has_no_findings() {
        let claims = vec![claim("claim-20261107-001", ClaimStatus::Published, 0.9)];
        assert!(lint(&claims, Utc::now()).is_empty());
    }

    #[test]
    fn stale_published_claim_is_flagged() {
        let claims = vec![claim("claim-20261107-001", ClaimStatus::Published, 0.31)];
        let findings = lint(&claims, Utc::now());
        assert!(findings.iter().any(|f| f.rule == "L3"));
    }

    #[test]
    fn superseded_without_successor_is_flagged() {
        let claims = vec![claim("claim-20261107-001", ClaimStatus::Superseded, 0.9)];
        let findings = lint(&claims, Utc::now());
        assert!(findings.iter().any(|f| f.rule == "L2"));
    }

    #[test]
    fn superseded_with_known_successor_is_clean() {
        let claims = vec![
            {
                let mut c = claim("claim-20261107-001", ClaimStatus::Superseded, 0.9);
                c.superseded_by = Some("claim-20261107-002".into());
                c
            },
            claim("claim-20261107-002", ClaimStatus::Published, 0.9),
        ];
        assert!(lint(&claims, Utc::now()).is_empty());
    }

    #[test]
    fn title_near_collision_is_flagged() {
        let claims = vec![
            {
                let mut c = claim("claim-20261107-001", ClaimStatus::Published, 0.9);
                c.title = "alpha beta gamma delta".into();
                c
            },
            {
                let mut c = claim("claim-20261107-002", ClaimStatus::Published, 0.9);
                c.title = "alpha beta gamma delta!".into();
                c
            },
        ];
        let findings = lint(&claims, Utc::now());
        assert!(findings.iter().any(|f| f.rule == "L4"));
    }

    #[test]
    fn disposition_appends_and_counts() {
        let dir = tempfile::tempdir().unwrap();
        let q = dir.path().join(".kem/review-queue.jsonl");
        record_disposition(
            &q,
            "claim-20261107-001",
            Disposition::Accept,
            "human",
            "ok",
            Utc::now(),
        )
        .unwrap();
        let h = doctor(dir.path(), &dir.path().join(".kem/ledger.jsonl"), &q).unwrap();
        // Resolved dispositions are history — pending count must stay 0
        // (review defect #3: doctor previously counted resolved lines).
        assert_eq!(h.review_queue, 0);
        // An unresolved line does count as pending.
        std::fs::write(
            &q,
            "{\"claim_id\":\"claim-20261107-002\",\"reasons\":[\"quote drifted\"]}\n",
        )
        .unwrap();
        let h2 = doctor(dir.path(), &dir.path().join(".kem/ledger.jsonl"), &q).unwrap();
        assert_eq!(h2.review_queue, 1);
        assert!(h.ledger_ok); // no ledger yet = trivially ok
        assert_eq!(h.claims, 0);
    }
}
