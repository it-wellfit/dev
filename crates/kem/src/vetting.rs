#![forbid(unsafe_code)]
//! M3 ??T1 deterministic vetting (KEM doc 禮4.3).
//!
//! T1 owns every structural judgment with zero LLM calls:
//! groundedness (verbatim quote in evidence file), duplication (normalized
//! body hash + token overlap), sensitivity (topic/secret scan), completeness.
//! Code owns all thresholds; the ledger records every verdict.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::schema::{known_ids_of, validate_claim, Claim, ClaimStatus, KemError, Track};

/// Verdict outcome of T1 vetting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Accept,
    Review,
    Reject,
}

/// T1 scorecard. All thresholds are constants below ??code owns them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scores {
    /// 1.0 when every evidence quote is found verbatim in its file.
    pub groundedness: f64,
    /// 1.0 when no existing claim duplicates this body (??DUP_OVERLAP).
    pub duplication: f64,
    /// 1.0 when no secret patterns or sensitive topics detected.
    pub sensitivity: f64,
    /// 1.0 when schema validation passes.
    pub completeness: f64,
}

/// T1 verdict with human-readable reasons (fed to triage and the ledger).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub outcome: Outcome,
    pub scores: Scores,
    pub reasons: Vec<String>,
}

/// Accept thresholds ??code owns every number (pi-jev-wiki philosophy).
pub const DUP_OVERLAP: f64 = 0.9;
/// Secret patterns that hard-reject a claim (never stored).
pub const SECRET_PATTERNS: [&str; 6] =
    ["-----BEGIN", "AKIA", "sk-", "ghp_", "password=", "api_key="];
/// Topic substrings that hard-reject regardless of content.
pub const SENSITIVE_TOPICS: [&str; 3] = ["secret", "credential", "private-key"];

/// Everything T1 needs: where evidence lives and which claims already exist.
pub struct VetContext {
    /// Workspace root used to resolve evidence paths.
    pub root: PathBuf,
    /// Existing claims (for duplicate detection).
    pub existing: Vec<Claim>,
    /// Known ids (schema cross-reference checks).
    pub known_ids: BTreeSet<String>,
}

impl VetContext {
    pub fn new(root: PathBuf, existing: Vec<Claim>) -> Self {
        let known_ids = known_ids_of(existing.iter());
        Self {
            root,
            existing,
            known_ids,
        }
    }
}

/// Run T1 vetting. Never panics; every rejection carries reasons.
pub fn vet_t1(claim: &Claim, ctx: &VetContext) -> Verdict {
    let mut reasons = Vec::new();

    // Completeness: schema rules first.
    let schema_errors = validate_claim(
        claim,
        Some(&crate::schema::ValidationContext {
            known_ids: ctx.known_ids.clone(),
        }),
    );
    let completeness = if schema_errors.is_empty() { 1.0 } else { 0.0 };
    for e in &schema_errors {
        reasons.push(format!("schema: {e}"));
    }

    // Groundedness: every quote must appear verbatim in its evidence file.
    let (grounded, grounded_reasons) = check_groundedness(claim, &ctx.root);
    let groundedness = grounded;
    reasons.extend(grounded_reasons);

    // Duplication: exact-body and high-overlap reinforcement.
    let (dup_free, dup_reasons) = check_duplication(claim, &ctx.existing);
    let duplication = dup_free;
    reasons.extend(dup_reasons);

    // Sensitivity: secrets and sensitive topics hard-reject.
    let (safe, safe_reasons) = check_sensitivity(claim);
    let sensitivity = safe;
    reasons.extend(safe_reasons);

    let scores = Scores {
        groundedness,
        duplication,
        sensitivity,
        completeness,
    };

    // Decision mapping ??deterministic, thresholds owned here.
    let outcome = if sensitivity < 1.0 {
        Outcome::Reject // secrets are never stored, only triaged
    } else if completeness < 1.0 || groundedness < 1.0 || duplication < 1.0 {
        if schema_errors.is_empty() && groundedness >= 1.0 {
            // Only a near-duplicate: reviewer decides reinforce vs new.
            Outcome::Review
        } else {
            Outcome::Reject
        }
    } else {
        Outcome::Accept
    };

    Verdict {
        outcome,
        scores,
        reasons,
    }
}

fn check_groundedness(claim: &Claim, root: &Path) -> (f64, Vec<String>) {
    if claim.evidence.is_empty() {
        return (0.0, vec!["groundedness: no evidence entries".into()]);
    }
    let mut ok = 0usize;
    let mut reasons = Vec::new();
    for ev in &claim.evidence {
        let path = root.join(&ev.path);
        match std::fs::read_to_string(&path) {
            Err(e) => reasons.push(format!(
                "groundedness: evidence file unreadable {} ({e})",
                ev.path
            )),
            Ok(content) => {
                if content.contains(ev.quote.as_str()) {
                    ok += 1;
                } else {
                    reasons.push(format!(
                        "groundedness: quote not found verbatim in {}",
                        ev.path
                    ));
                }
            }
        }
    }
    let groundedness = f64::from(u32::try_from(ok).unwrap_or(0))
        / f64::from(u32::try_from(claim.evidence.len().max(1)).unwrap_or(1));
    (groundedness, reasons)
}

fn check_duplication(claim: &Claim, existing: &[Claim]) -> (f64, Vec<String>) {
    let mut reasons = Vec::new();
    for other in existing {
        if other.id == claim.id {
            continue;
        }
        let overlap = token_overlap(&claim.body, &other.body);
        if overlap >= DUP_OVERLAP {
            reasons.push(format!(
                "duplication: overlaps {}/{} of existing {} ({:.2})",
                claim.id, other.id, other.id, overlap
            ));
            return (0.0, reasons);
        }
    }
    (1.0, reasons)
}

fn check_sensitivity(claim: &Claim) -> (f64, Vec<String>) {
    let mut reasons = Vec::new();
    let hay = format!("{} {}", claim.body, claim.title);
    for pat in SECRET_PATTERNS {
        if hay.contains(pat) {
            reasons.push(format!("sensitivity: secret pattern {pat:?} in body"));
        }
    }
    for topic in &claim.topics {
        let t = topic.to_lowercase();
        for s in SENSITIVE_TOPICS {
            if t.contains(s) {
                reasons.push(format!("sensitivity: sensitive topic {topic:?}"));
            }
        }
    }
    if reasons.is_empty() {
        (1.0, reasons)
    } else {
        (0.0, reasons)
    }
}

/// Jaccard overlap over lowercase alphanumeric tokens.
pub fn token_overlap(a: &str, b: &str) -> f64 {
    let set = |s: &str| -> BTreeSet<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 1)
            .map(|w| w.to_lowercase())
            .collect()
    };
    let sa = set(a);
    let sb = set(b);
    if sa.is_empty() && sb.is_empty() {
        return 1.0;
    }
    let inter = sa.intersection(&sb).count();
    let union = sa.union(&sb).count();
    if union == 0 {
        return 1.0;
    }
    f64::from(u32::try_from(inter).unwrap_or(0)) / f64::from(u32::try_from(union).unwrap_or(1))
}

/// Apply an accepted/reviewed verdict: write the claim file (SSOT) and record
/// the ledger entries. Structural ops (supersede) on Track A go to review.
pub fn stage_claim(claims_dir: &Path, claim: &Claim) -> Result<PathBuf, KemError> {
    std::fs::create_dir_all(claims_dir)?;
    // Status at staging: Track A goes to pending_review (gated), B published.
    let mut staged = claim.clone();
    if staged.track == Track::A && staged.status == ClaimStatus::Published {
        staged.status = ClaimStatus::PendingReview;
    }
    let path = claims_dir.join(format!("{}.json", staged.id));
    std::fs::write(&path, serde_json::to_string_pretty(&staged)?)?;
    Ok(path)
}

/// Ledger helper shared by CLI commands. The actor names the decision maker
/// (e.g. `kem-seed` for the bootstrap flow, `kem-t1` for normal ingest).
pub fn record_verdict(
    ledger_path: &Path,
    actor: &str,
    claim_id: &str,
    verdict: &Verdict,
    now: DateTime<Utc>,
) -> Result<(), KemError> {
    crate::ledger::append(
        ledger_path,
        actor,
        crate::ledger::LedgerAction::Vet,
        claim_id,
        serde_json::to_value(verdict)?,
        now,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClaimType, Evidence};
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap()
    }

    fn claim_with_body(body: &str) -> Claim {
        Claim {
            id: "claim-20261107-042".into(),
            claim_type: ClaimType::Fact,
            status: ClaimStatus::Published,
            track: Track::B,
            confidence: 0.9,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            valid_to: None,
            recorded_at: now(),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "evidence.txt".into(),
                rev: "working-tree".into(),
                quote: "the canonical sentence".into(),
            }],
            topics: vec!["test".into()],
            source: "raw/test/seed.md".into(),
            title: "t".into(),
            body: body.into(),
        }
    }

    fn ctx(root: &Path, existing: Vec<Claim>) -> VetContext {
        VetContext::new(root.to_path_buf(), existing)
    }

    #[test]
    fn accepts_grounded_unique_safe_claim() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("evidence.txt"),
            "prefix the canonical sentence suffix",
        )
        .unwrap();
        let c = claim_with_body("The canonical sentence holds for testing.");
        let v = vet_t1(&c, &ctx(dir.path(), vec![]));
        assert_eq!(v.outcome, Outcome::Accept, "{:?}", v.reasons);
        assert_eq!(
            v.scores,
            Scores {
                groundedness: 1.0,
                duplication: 1.0,
                sensitivity: 1.0,
                completeness: 1.0
            }
        );
    }

    #[test]
    fn rejects_ungrounded_quote() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.txt"), "something else entirely").unwrap();
        let c = claim_with_body("Some body.");
        let v = vet_t1(&c, &ctx(dir.path(), vec![]));
        assert_eq!(v.outcome, Outcome::Reject);
        assert!(v.reasons.iter().any(|r| r.contains("not found verbatim")));
    }

    #[test]
    fn rejects_secret_patterns() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.txt"), "the canonical sentence").unwrap();
        let c = claim_with_body("store api_key=abc123 here");
        let v = vet_t1(&c, &ctx(dir.path(), vec![]));
        assert_eq!(v.outcome, Outcome::Reject);
        assert!(v.reasons.iter().any(|r| r.contains("api_key=")));
    }

    #[test]
    fn near_duplicate_sends_to_review() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.txt"), "the canonical sentence").unwrap();
        let mut old = claim_with_body("Alpha beta gamma delta epsilon zeta eta theta.");
        old.id = "claim-20261106-001".into();
        let c = claim_with_body("Alpha beta gamma delta epsilon zeta eta theta!");
        let v = vet_t1(&c, &ctx(dir.path(), vec![old]));
        assert_eq!(v.outcome, Outcome::Review);
        assert!(v.reasons.iter().any(|r| r.contains("duplication")));
    }

    #[test]
    fn overlap_metric_bounds() {
        assert!((token_overlap("alpha beta", "alpha beta") - 1.0).abs() < 1e-9);
        assert!(token_overlap("alpha beta", "gamma delta") < 0.01);
    }

    #[test]
    fn track_a_accept_stages_as_pending_review() {
        let dir = tempfile::tempdir().unwrap();
        let claims = dir.path().join("claims");
        let mut c = claim_with_body("Some body.");
        c.track = Track::A;
        let p = stage_claim(&claims, &c).unwrap();
        let stored: Claim = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
        assert_eq!(stored.status, ClaimStatus::PendingReview);
    }

    #[test]
    fn verdict_ledger_entry_chains() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("ledger.jsonl");
        let v = Verdict {
            outcome: Outcome::Accept,
            scores: Scores {
                groundedness: 1.0,
                duplication: 1.0,
                sensitivity: 1.0,
                completeness: 1.0,
            },
            reasons: vec![],
        };
        record_verdict(&ledger, "kem-t1", "claim-20261107-042", &v, now()).unwrap();
        assert_eq!(crate::ledger::verify(&ledger).unwrap(), 1);
    }
}
