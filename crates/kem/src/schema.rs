#![forbid(unsafe_code)]
//! Claim schema v1 — the machine SSOT for every unit of knowledge.
//!
//! A claim is an atomic, evidence-backed statement. Validation is
//! deterministic (T1 vetting lives in the schema rules themselves).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Unified error type (KEM doc §4 constraint E-01 pattern: typed, no String-only).
#[derive(Debug, thiserror::Error)]
pub enum KemError {
    #[error("schema violation: {0}")]
    SchemaViolation(String),
    #[error("ledger chain broken at line {at}: {reason}")]
    ChainBroken { at: usize, reason: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(String),
}

/// Knowledge type of a claim (KEM doc §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Concept,
    Decision,
    Invariant,
    Gotcha,
    Glossary,
    Fact,
}

/// Lifecycle state (KEM doc §4.4 state machine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Draft,
    PendingReview,
    Published,
    Superseded,
    Deprecated,
    Retired,
}

impl ClaimStatus {
    /// Terminal-until-retirement rules from eli.ai/Wikidata two-track model:
    /// `superseded`/`deprecated` never return to `published`; exit is `retired`.
    pub fn can_transition_to(self, next: ClaimStatus) -> bool {
        use ClaimStatus::*;
        matches!(
            (self, next),
            (Draft, PendingReview)
                | (Draft, Retired)
                | (PendingReview, Draft)
                | (PendingReview, Published)
                | (PendingReview, Retired)
                | (Published, Superseded)
                | (Published, Deprecated)
                | (Published, Retired)
                | (Superseded, Retired)
                | (Deprecated, Retired)
        )
    }

    /// `draft`/`pending_review` are excluded from retrieval; `retired` is
    /// fully out of the serving path (KEM doc §4.4). `superseded`/`deprecated`
    /// stay retrievable but demoted and labeled.
    pub fn retrievable(self) -> bool {
        matches!(
            self,
            ClaimStatus::Published | ClaimStatus::Superseded | ClaimStatus::Deprecated
        )
    }
}

/// Governance track: `a` = structural (gated), `b` = instance (live-by-default,
/// post-hoc review promotes). See KEM doc §4.3 and eli.ai concept governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Track {
    A,
    B,
}

/// Verbatim-quote evidence binding a claim to a source file at a revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Workspace-relative path of the source file.
    pub path: String,
    /// Revision the quote was observed at (`git -C <root> rev-parse --short HEAD`,
    /// or `working-tree` when uncommitted, or `nogit` when not a repository).
    pub rev: String,
    /// Verbatim substring that must still be found in the file at sync time.
    pub quote: String,
}

/// One atomic unit of knowledge. JSON is the SSOT; markdown views are derived.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    /// `claim-YYYYMMDD-NNN`, never reused. `default` allows ingest drafts to
    /// omit it (ingest auto-assigns); an empty id fails V-A01 validation, so
    /// no id-less claim can ever be published.
    #[serde(default)]
    pub id: String,
    pub claim_type: ClaimType,
    pub status: ClaimStatus,
    pub track: Track,
    /// Derived value in [0,1]: source count + reaffirm recency − contradictions.
    /// Never hand-set for published claims (vetting computes it).
    pub confidence: f64,
    /// Valid time: when the fact became true in the world.
    pub valid_from: NaiveDate,
    /// Valid time end; `None` = currently valid.
    pub valid_to: Option<NaiveDate>,
    /// Transaction time: when this system recorded the claim.
    pub recorded_at: DateTime<Utc>,
    /// IDs this claim replaces (link, never delete).
    #[serde(default)]
    pub supersedes: Vec<String>,
    /// Successor id; required iff status is `superseded`.
    #[serde(default)]
    pub superseded_by: Option<String>,
    /// ≥1 for `published`; quote must be verbatim in `path` at `rev`.
    pub evidence: Vec<Evidence>,
    /// Non-empty for `published`.
    pub topics: Vec<String>,
    /// L0 immutable source path; must start with `raw/`.
    pub source: String,
    /// Short claim title (concept-oriented, evergreen style).
    pub title: String,
    /// The atomic statement itself.
    pub body: String,
}

/// Context needed to validate cross-claim references.
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    /// All claim ids known to the store (for `supersedes` reference checks).
    pub known_ids: BTreeSet<String>,
}

/// Deterministic T1 validation. Returns every violation, not just the first.
pub fn validate_claim(claim: &Claim, ctx: Option<&ValidationContext>) -> Vec<KemError> {
    let mut errs = Vec::new();

    // V-A01: id shape claim-YYYYMMDD-NNN
    if !valid_id_shape(&claim.id) {
        errs.push(KemError::SchemaViolation(format!(
            "V-A01 id shape invalid: {}",
            claim.id
        )));
    }
    // V-A02: confidence in [0,1]
    if !(0.0..=1.0).contains(&claim.confidence) {
        errs.push(KemError::SchemaViolation(format!(
            "V-A02 confidence out of [0,1]: {}",
            claim.confidence
        )));
    }
    // V-A03: body/title non-empty and body is atomic (one sentence-ish; bounded)
    if claim.title.trim().is_empty() {
        errs.push(KemError::SchemaViolation("V-A03 title empty".into()));
    }
    let body = claim.body.trim();
    if body.is_empty() {
        errs.push(KemError::SchemaViolation("V-A03 body empty".into()));
    } else if body.len() > 2048 {
        errs.push(KemError::SchemaViolation(format!(
            "V-A03 body exceeds 2048 bytes: {}",
            body.len()
        )));
    }
    // V-A04: published requires ≥1 evidence
    if claim.status == ClaimStatus::Published && claim.evidence.is_empty() {
        errs.push(KemError::SchemaViolation(
            "V-A04 published claim lacks evidence".into(),
        ));
    }
    // evidence entries well-formed
    for ev in &claim.evidence {
        if ev.path.trim().is_empty() || ev.rev.trim().is_empty() || ev.quote.trim().is_empty() {
            errs.push(KemError::SchemaViolation(format!(
                "V-A04 evidence incomplete for {} (path/rev/quote must be non-empty)",
                claim.id
            )));
        }
    }
    // V-A06: no self-supersede; refs must be known when context is given
    if claim.supersedes.iter().any(|id| id == &claim.id) {
        errs.push(KemError::SchemaViolation(format!(
            "V-A06 {} supersedes itself",
            claim.id
        )));
    }
    if let Some(ctx) = ctx {
        for id in &claim.supersedes {
            if !ctx.known_ids.contains(id) {
                errs.push(KemError::SchemaViolation(format!(
                    "V-A06 supersedes unknown id: {id}"
                )));
            }
        }
    }
    // V-A07: superseded ⇒ superseded_by present and not itself;
    //        published ⇒ superseded_by absent
    match claim.status {
        ClaimStatus::Superseded => match &claim.superseded_by {
            None => errs.push(KemError::SchemaViolation(format!(
                "V-A07 {} is superseded without superseded_by",
                claim.id
            ))),
            Some(s) if s == &claim.id => errs.push(KemError::SchemaViolation(format!(
                "V-A07 {} superseded_by itself",
                claim.id
            ))),
            Some(s) => {
                if let Some(ctx) = ctx
                    && !ctx.known_ids.contains(s)
                {
                    errs.push(KemError::SchemaViolation(format!(
                        "V-A07 superseded_by unknown id: {s}"
                    )));
                }
            }
        },
        ClaimStatus::Published if claim.superseded_by.is_some() => {
            errs.push(KemError::SchemaViolation(format!(
                "V-A07 published {} must not carry superseded_by",
                claim.id
            )));
        }
        _ => {}
    }
    // V-A08: valid_to >= valid_from when present
    if let Some(to) = claim.valid_to
        && to < claim.valid_from
    {
        errs.push(KemError::SchemaViolation(format!(
            "V-A08 valid_to {to} before valid_from {}",
            claim.valid_from
        )));
    }
    // V-A09: published requires topics
    if claim.status == ClaimStatus::Published && claim.topics.is_empty() {
        errs.push(KemError::SchemaViolation(
            "V-A09 published claim lacks topics".into(),
        ));
    }
    // V-A10: source must live under raw/
    if !claim.source.starts_with("raw/") || claim.source.trim().len() <= 4 {
        errs.push(KemError::SchemaViolation(format!(
            "V-A10 source not under raw/: {}",
            claim.source
        )));
    }
    // Two-track guard: supersede/deprecate/retire of another claim is a
    // Track A operation; the claim recording it must be Track A.
    if !claim.supersedes.is_empty() && claim.track == Track::B {
        errs.push(KemError::SchemaViolation(format!(
            "V-A11 {} supersedes on track B (structural op requires track A)",
            claim.id
        )));
    }
    errs
}

fn valid_id_shape(id: &str) -> bool {
    let rest = match id.strip_prefix("claim-") {
        Some(r) => r,
        None => return false,
    };
    let bytes = rest.as_bytes();
    if bytes.len() != 12 {
        return false;
    }
    // YYYYMMDD-NNN
    bytes[0..8].iter().all(u8::is_ascii_digit)
        && bytes[8] == b'-'
        && bytes[9..12].iter().all(u8::is_ascii_digit)
}

/// Build the set of ids referenced by a claim collection (for context checks).
pub fn known_ids_of<'a, I: IntoIterator<Item = &'a Claim>>(claims: I) -> BTreeSet<String> {
    claims.into_iter().map(|c| c.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn base(id: &str, status: ClaimStatus) -> Claim {
        Claim {
            id: id.to_string(),
            claim_type: ClaimType::Fact,
            status,
            track: Track::B,
            confidence: 0.9,
            valid_from: NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            valid_to: None,
            recorded_at: Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap(),
            supersedes: Vec::new(),
            superseded_by: None,
            evidence: vec![Evidence {
                path: "docs/README.md".into(),
                rev: "working-tree".into(),
                quote: "Knowledge Flow".into(),
            }],
            topics: vec!["test".into()],
            source: "raw/test/2026-11-07-seed.md".into(),
            title: "test claim".into(),
            body: "A test claim body.".into(),
        }
    }

    #[test]
    fn valid_published_claim_has_no_errors() {
        let c = base("claim-20261107-001", ClaimStatus::Published);
        assert!(validate_claim(&c, None).is_empty(), "unexpected errors");
    }

    #[test]
    fn id_shape_is_enforced() {
        let mut c = base("claim-20261107-001", ClaimStatus::Draft);
        c.id = "claim-01".into();
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A01")));
    }

    #[test]
    fn published_requires_evidence() {
        let mut c = base("claim-20261107-001", ClaimStatus::Published);
        c.evidence.clear();
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A04")));
    }

    #[test]
    fn superseded_requires_successor() {
        let c = base("claim-20261107-001", ClaimStatus::Superseded);
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A07")));
    }

    #[test]
    fn published_must_not_carry_successor() {
        let mut c = base("claim-20261107-001", ClaimStatus::Published);
        c.superseded_by = Some("claim-20261107-002".into());
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A07")));
    }

    #[test]
    fn track_b_cannot_supersede() {
        let mut c = base("claim-20261107-001", ClaimStatus::Published);
        c.supersedes = vec!["claim-20261106-001".into()];
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A11")));
    }

    #[test]
    fn state_machine_rejects_resurrection() {
        assert!(!ClaimStatus::Superseded.can_transition_to(ClaimStatus::Published));
        assert!(!ClaimStatus::Deprecated.can_transition_to(ClaimStatus::Published));
        assert!(!ClaimStatus::Retired.can_transition_to(ClaimStatus::Draft));
        assert!(ClaimStatus::PendingReview.can_transition_to(ClaimStatus::Published));
    }

    #[test]
    fn drafts_are_not_retrievable() {
        assert!(!ClaimStatus::Draft.retrievable());
        assert!(!ClaimStatus::PendingReview.retrievable());
        assert!(ClaimStatus::Published.retrievable());
        // superseded/deprecated stay retrievable but demoted (KEM doc §4.4)
        assert!(ClaimStatus::Superseded.retrievable());
        assert!(ClaimStatus::Deprecated.retrievable());
        assert!(!ClaimStatus::Retired.retrievable());
    }

    #[test]
    fn valid_to_must_not_precede_valid_from() {
        let mut c = base("claim-20261107-001", ClaimStatus::Draft);
        c.valid_to = Some(NaiveDate::from_ymd_opt(2026, 11, 6).unwrap());
        let errs = validate_claim(&c, None);
        assert!(errs.iter().any(|e| e.to_string().contains("V-A08")));
    }

    #[test]
    fn unknown_supersedes_reference_is_reported() {
        let mut c = base("claim-20261107-001", ClaimStatus::Draft);
        c.track = Track::A;
        c.supersedes = vec!["claim-20261106-999".into()];
        let ctx = ValidationContext {
            known_ids: ["claim-20261107-001".to_string()].into_iter().collect(),
        };
        let errs = validate_claim(&c, Some(&ctx));
        assert!(errs.iter().any(|e| e.to_string().contains("V-A06")));
    }
}
