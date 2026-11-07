#![forbid(unsafe_code)]
//! M5 — Retrieval: BM25 lexical search over retrieval-visible claims.
//!
//! Rules (KEM doc §4.4 / turntakingreview guidance):
//! - lifecycle filter runs BEFORE ranking (never retrieve, then hope the
//!   model ignores what it should not see)
//! - bi-temporal validity: only claims valid at the query date unless
//!   explicitly querying history
//! - `ask` never injects into context; it returns page references

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::schema::{Claim, ClaimStatus, KemError};

pub const K1: f64 = 1.5;
pub const B: f64 = 0.75;

/// A scored hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub id: String,
    pub title: String,
    pub status: ClaimStatus,
    pub confidence: f64,
    pub score: f64,
}

/// BM25 statistics over a claim collection.
pub struct Index {
    /// claim id → term → term frequency
    tf: BTreeMap<String, BTreeMap<String, u32>>,
    /// term → number of documents containing it
    df: BTreeMap<String, u32>,
    /// claim id → document length (tokens)
    len: BTreeMap<String, u32>,
    total_len: f64,
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 1)
        .map(|w| w.to_lowercase())
        .collect()
}

impl Index {
    pub fn build(claims: &[Claim]) -> Self {
        let mut tf = BTreeMap::new();
        let mut df: BTreeMap<String, u32> = BTreeMap::new();
        let mut len = BTreeMap::new();
        let mut total_len = 0.0f64;
        for c in claims {
            let mut terms: BTreeMap<String, u32> = BTreeMap::new();
            let mut doc_tokens: BTreeSet<String> = BTreeSet::new();
            let mut count = 0u32;
            let mut feed = |t: String| {
                *terms.entry(t.clone()).or_insert(0) += 1;
                count += 1;
                doc_tokens.insert(t);
            };
            for t in tokenize(&c.title) {
                feed(t);
            }
            for t in tokenize(&c.body) {
                feed(t);
            }
            for t in &c.topics {
                for tok in tokenize(t) {
                    feed(tok);
                }
            }
            for t in &doc_tokens {
                *df.entry(t.clone()).or_insert(0) += 1;
            }
            total_len += f64::from(count);
            len.insert(c.id.clone(), count);
            tf.insert(c.id.clone(), terms);
        }
        Self {
            tf,
            df,
            len,
            total_len,
        }
    }

    pub fn n_docs(&self) -> usize {
        self.tf.len()
    }

    /// BM25 score of `query` for one claim document.
    pub fn score(&self, id: &str, query_terms: &[String]) -> f64 {
        let n = self.n_docs() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let doc_len = f64::from(*self.len.get(id).unwrap_or(&0));
        let avg = if self.total_len > 0.0 {
            self.total_len / n
        } else {
            1.0
        };
        let terms = match self.tf.get(id) {
            Some(t) => t,
            None => return 0.0,
        };
        let mut score = 0.0;
        for qt in query_terms {
            let f = f64::from(*terms.get(qt).unwrap_or(&0));
            if f == 0.0 {
                continue;
            }
            let df = f64::from(*self.df.get(qt).unwrap_or(&0));
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            let norm = f * (K1 + 1.0) / (f + K1 * (1.0 - B + B * (doc_len / avg)));
            score += idf * norm;
        }
        score
    }

    /// Top-k hits over the given (already lifecycle/validity filtered) claims.
    pub fn search(&self, filtered: &[Claim], query: &str, top: usize) -> Vec<Hit> {
        let qterms = tokenize(query);
        let mut scored: Vec<Hit> = filtered
            .iter()
            .map(|c| Hit {
                id: c.id.clone(),
                title: c.title.clone(),
                status: c.status,
                confidence: c.confidence,
                score: self.score(&c.id, &qterms),
            })
            .filter(|h| h.score > 0.0)
            .collect();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top);
        scored
    }
}

/// Lifecycle + validity filter applied BEFORE ranking.
/// `at` = None means "now" (current-belief query).
pub fn visible_at(claims: &[Claim], at: Option<chrono::NaiveDate>) -> Vec<Claim> {
    let today = chrono::Utc::now().date_naive();
    let at = at.unwrap_or(today);
    claims
        .iter()
        .filter(|c| {
            c.status.retrievable()
                && c.valid_from <= at
                && c.valid_to.is_none_or(|v| at <= v)
        })
        .cloned()
        .collect()
}

/// One golden evaluation case: query plus the id that must rank first,
/// or `expect: "none"` for a refusal case (no hit allowed).
#[derive(Debug, Clone, Deserialize)]
pub struct GoldenCase {
    pub query: String,
    pub expect: String,
}

/// Run a golden set: top-1 accuracy. Refusal cases pass only with 0 hits.
pub fn run_golden(index: &Index, all: &[Claim], cases: &[GoldenCase]) -> Result<(usize, usize, Vec<String>), KemError> {
    let mut pass = 0usize;
    let mut failures = Vec::new();
    for case in cases {
        let filtered = visible_at(all, None);
        let hits = index.search(&filtered, &case.query, 1);
        let ok = if case.expect == "none" {
            hits.is_empty()
        } else {
            hits.first().is_some_and(|h| h.id == case.expect)
        };
        if ok {
            pass += 1;
        } else {
            failures.push(format!(
                "query {:?}: expected {}, got {}",
                case.query,
                case.expect,
                hits.first().map_or("no-hit".to_string(), |h| h.id.clone())
            ));
        }
    }
    Ok((pass, cases.len(), failures))
}

/// Load a golden set file (JSONL: one {query, expect} case per line;
/// blank lines ignored). JSONL keeps the set append-only and diff-friendly.
pub fn load_golden(path: &Path) -> Result<Vec<GoldenCase>, KemError> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line).map_err(|e| {
            KemError::InvalidInput(format!("{path:?} line {}: {e}", i + 1))
        })?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClaimType, Evidence, Track};
    use chrono::{TimeZone, Utc};

    fn claim(id: &str, title: &str, body: &str, status: ClaimStatus) -> Claim {
        Claim {
            id: id.into(),
            claim_type: ClaimType::Fact,
            status,
            track: Track::B,
            confidence: 0.9,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            valid_to: None,
            recorded_at: chrono::Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap(),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "docs/x.md".into(),
                rev: "working-tree".into(),
                quote: "q".into(),
            }],
            topics: vec!["testing".into()],
            source: "raw/t/2026-11-07-a.md".into(),
            title: title.into(),
            body: body.into(),
        }
    }

    #[test]
    fn bm25_ranks_relevant_claim_first() {
        let claims = vec![
            claim(
                "claim-20261107-001",
                "Phase gate status",
                "Phase A is not approved and Phase B is blocked for the platform",
                ClaimStatus::Published,
            ),
            claim(
                "claim-20261107-002",
                "Zero JavaScript",
                "The browser receives only HTML CSS SVG images and fonts",
                ClaimStatus::Published,
            ),
        ];
        let idx = Index::build(&claims);
        let hits = idx.search(&claims, "phase gate approved blocked", 2);
        assert_eq!(hits.first().map(|h| h.id.as_str()), Some("claim-20261107-001"));
    }

    #[test]
    fn drafts_and_retired_are_invisible() {
        let claims = vec![
            claim("claim-20261107-001", "Phase gates", "phase gate blocked", ClaimStatus::Draft),
            claim("claim-20261107-002", "Phase gates two", "phase gate blocked", ClaimStatus::Retired),
        ];
        let filtered = visible_at(&claims, None);
        assert!(filtered.is_empty());
        let idx = Index::build(&claims);
        assert!(idx.search(&filtered, "phase gate", 5).is_empty());
    }

    #[test]
    fn expired_claims_are_invisible_but_history_queries_can_ask_past_dates() {
        let mut c = claim("claim-20261107-001", "Old rule", "color tokens policy", ClaimStatus::Deprecated);
        c.valid_to = Some(chrono::NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());
        assert!(visible_at(&[c.clone()], None).is_empty()); // current query hides it
        let past = chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
        assert_eq!(visible_at(&[c], Some(past)).len(), 1); // point-in-time query sees it
    }

    #[test]
    fn refusal_case_passes_with_no_hits() {
        let claims = vec![claim(
            "claim-20261107-001",
            "Phase gates",
            "phase gate blocked state",
            ClaimStatus::Published,
        )];
        let idx = Index::build(&claims);
        let cases = vec![
            GoldenCase { query: "phase gate".into(), expect: "claim-20261107-001".into() },
            GoldenCase { query: "kubernetes autoscaling cluster".into(), expect: "none".into() },
        ];
        let (pass, total, fails) = run_golden(&idx, &claims, &cases).unwrap();
        assert_eq!((pass, total), (2, 2));
        assert!(fails.is_empty());
    }
}
