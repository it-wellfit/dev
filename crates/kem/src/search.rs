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

use crate::embed::{VectorIndexFile, fresh_lookup};
use crate::schema::{Claim, ClaimStatus, KemError};

pub const K1: f64 = 1.5;
pub const B: f64 = 0.75;

/// Semantic refusal floor for hybrid mode: raw-cosine hits with zero lexical
/// support below this are dropped (honest refusal preserved under fusion).
/// Evidence-driven calibration (2026-11-09, `cargo run --example calibrate`):
/// max cosine across all 4 golden refusal queries × their false hits was
/// 0.4872 ("quantum qubit" × claim-011); the cross-lingual positive sits well
/// above. Floor 0.55 separates the two populations.
pub const REFUSAL_FLOOR: f32 = 0.55;

/// A scored hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub id: String,
    pub title: String,
    pub status: ClaimStatus,
    pub confidence: f64,
    pub score: f64,
    /// Retrieval path provenance: "hybrid" (BM25+vector fused) or "bm25".
    #[serde(default)]
    pub via: String,
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
                via: "bm25".into(),
            })
            .filter(|h| h.score > 0.0)
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top);
        scored
    }

    /// Raw BM25 scores for every filtered claim (fusion input, unsorted).
    pub fn scores_all(&self, filtered: &[Claim], query: &str) -> Vec<(String, f64)> {
        let qterms = tokenize(query);
        filtered
            .iter()
            .map(|c| (c.id.clone(), self.score(&c.id, &qterms)))
            .collect()
    }
}

/// Hybrid fusion (KEM doc §8.2 R7 / P2b): α·norm(BM25) + (1−α)·norm(cosine).
/// Claims without a (fresh) vector fall back to BM25-only and are marked
/// via="bm25" — honest degradation, never fabricated vector scores.
pub fn hybrid_fuse(
    filtered: &[Claim],
    bm25_scores: &[(String, f64)],
    vectors: Option<&std::collections::HashMap<String, Vec<f32>>>,
    query_vec: Option<&[f32]>,
    top: usize,
    alpha: f32,
    vec_refusal_floor: f32,
) -> Vec<Hit> {
    let mut bm25: std::collections::HashMap<&str, f64> = std::collections::HashMap::new();
    let mut bm25_max = 0.0f64;
    for (id, s) in bm25_scores {
        bm25.insert(id.as_str(), *s);
        if *s > bm25_max {
            bm25_max = *s;
        }
    }
    let mut vec_scores: std::collections::HashMap<&str, f32> = std::collections::HashMap::new();
    let mut vec_max = 0.0f32;
    if let (Some(vectors), Some(qv)) = (vectors, query_vec) {
        for c in filtered {
            if let Some(v) = vectors.get(c.id.as_str()) {
                let s = crate::embed::cosine(qv, v);
                if s > vec_max {
                    vec_max = s;
                }
                vec_scores.insert(c.id.as_str(), s);
            }
        }
    }
    let norm = |v: f64, max: f64| if max > 0.0 { (v / max) as f32 } else { 0.0 };
    let alpha = f64::from(alpha);
    let mut fused: Vec<Hit> = filtered
        .iter()
        .filter_map(|c| {
            let b = bm25.get(c.id.as_str()).copied().unwrap_or(0.0);
            let vs = vec_scores.get(c.id.as_str()).copied();
            let score = match vs {
                Some(v) => {
                    // Semantic refusal floor: a hit with ZERO lexical support
                    // whose raw cosine sits below the floor is weak semantic
                    // adjacency, not knowledge — drop it (honest refusal
                    // preserved under fusion; review 2026-11-09).
                    if b == 0.0 && v < vec_refusal_floor {
                        return None;
                    }
                    let bn = norm(b, bm25_max) as f64;
                    let vn = if vec_max > 0.0 {
                        (v / vec_max) as f64
                    } else {
                        0.0
                    };
                    alpha * bn + (1.0 - alpha) * vn
                }
                None => norm(b, bm25_max) as f64,
            };
            if score <= 0.0 {
                return None;
            }
            Some(Hit {
                id: c.id.clone(),
                title: c.title.clone(),
                status: c.status,
                confidence: c.confidence,
                score,
                via: if vs.is_some() {
                    "hybrid".into()
                } else {
                    "bm25".into()
                },
            })
        })
        .collect();
    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused.truncate(top);
    fused
}

/// Lifecycle + validity filter applied BEFORE ranking.
/// `at` = None means "now" (current-belief query).
pub fn visible_at(claims: &[Claim], at: Option<chrono::NaiveDate>) -> Vec<Claim> {
    let today = chrono::Utc::now().date_naive();
    let at = at.unwrap_or(today);
    claims
        .iter()
        .filter(|c| {
            c.status.retrievable() && c.valid_from <= at && c.valid_to.is_none_or(|v| at <= v)
        })
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// SOTA retrieval upgrades (2026-11-12): rank fusion, diversity, query expansion
// ---------------------------------------------------------------------------

/// Standard RRF damping constant (Cormack et al. 2009, "Reciprocal Rank Fusion
/// outperforms Condorcet and individual Rank Learning Methods", SIGIR 2009):
/// k = 60 is the value validated across TREC tasks. Rank-based fusion is
/// scale-invariant, so it needs no score calibration and cannot be dominated by
/// a single retrieval path's score range (the weakness of the weighted
/// min-max fusion it complements).
pub const RRF_K: f64 = 60.0;

/// MMR diversity weight: 1.0 = pure relevance, 0.0 = pure diversity.
pub const MMR_DEFAULT_LAMBDA: f32 = 0.7;

/// Reciprocal Rank Fusion over BM25 and (optional) dense-vector rankings.
///
/// Unlike [`hybrid_fuse`], this fuses **ranks**, not normalized scores:
/// `score(d) = Σ_path 1 / (k + rank_path(d))`. The honest-degradation rule is
/// preserved verbatim — a document with zero lexical support whose raw cosine
/// sits below the refusal floor is dropped, so semantic adjacency alone can
/// never manufacture a hit. `score` is the raw RRF score; compare hits by
/// order, not by magnitude.
pub fn rrf_fuse(
    filtered: &[Claim],
    bm25_scores: &[(String, f64)],
    vectors: Option<&std::collections::HashMap<String, Vec<f32>>>,
    query_vec: Option<&[f32]>,
    top: usize,
    vec_refusal_floor: f32,
    k: f64,
) -> Vec<Hit> {
    let mut lexical: Vec<(String, f64)> = bm25_scores.to_vec();
    lexical.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let lexical_rank: std::collections::HashMap<&str, usize> = lexical
        .iter()
        .enumerate()
        .filter(|(_, (_, s))| *s > 0.0)
        .map(|(i, (id, _))| (id.as_str(), i))
        .collect();

    let mut dense: Vec<(&str, f32)> = Vec::new();
    if let (Some(vectors), Some(qv)) = (vectors, query_vec) {
        for c in filtered {
            if let Some(v) = vectors.get(c.id.as_str()) {
                dense.push((c.id.as_str(), crate::embed::cosine(qv, v)));
            }
        }
        dense.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    let dense_rank: std::collections::HashMap<&str, usize> = dense
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();

    let mut fused: Vec<Hit> = filtered
        .iter()
        .filter_map(|c| {
            let id = c.id.as_str();
            let lr = lexical_rank.get(id).copied();
            let dr = dense_rank.get(id).copied();
            let cos = dense
                .iter()
                .find(|(cid, _)| *cid == id)
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
            // Honest degradation (same rule as hybrid_fuse): no lexical
            // support + weak semantic adjacency => not knowledge.
            if lr.is_none() && dr.is_some() && cos < vec_refusal_floor {
                return None;
            }
            let mut score = 0.0f64;
            if let Some(r) = lr {
                score += 1.0 / (k + r as f64);
            }
            if let Some(r) = dr {
                score += 1.0 / (k + r as f64);
            }
            if score <= 0.0 {
                return None;
            }
            Some(Hit {
                id: c.id.clone(),
                title: c.title.clone(),
                status: c.status,
                confidence: c.confidence,
                score,
                via: match (lr.is_some(), dr.is_some()) {
                    (true, true) => "hybrid",
                    (true, false) => "bm25",
                    (false, true) => "vector",
                    (false, false) => return None,
                }
                .into(),
            })
        })
        .collect();
    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    fused.truncate(top);
    fused
}

/// Lexical Jaccard similarity over claim text (title + body + topics).
/// Used as the fallback document-similarity signal for [`mmr_diversify`] when
/// no dense vectors are available — offline, deterministic, zero deps.
fn lexical_sim(a: &Claim, b: &Claim) -> f32 {
    let ta: std::collections::HashSet<String> =
        tokenize(&format!("{} {} {}", a.title, a.body, a.topics.join(" ")))
            .into_iter()
            .collect();
    let tb: std::collections::HashSet<String> =
        tokenize(&format!("{} {} {}", b.title, b.body, b.topics.join(" ")))
            .into_iter()
            .collect();
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count() as f32;
    let union = ta.union(&tb).count() as f32;
    if union == 0.0 { 0.0 } else { inter / union }
}

/// Maximal Marginal Relevance (Carbonell & Goldstein, SIGIR 1998).
///
/// Greedy re-ranking that trades relevance against redundancy:
/// `argmax λ·rel(d) − (1−λ)·max_{s∈S} sim(d, s)`. Applied *after* fusion, it
/// widens the topical coverage of the context window without dropping strong
/// candidates — fewer near-duplicate claims in the prompt means less prompt
/// spend for the same evidence (lean: fewer tokens, same recall).
///
/// `rel` values are normalized to `[0, 1]` internally (corpus max), so the
/// λ trade-off is on a common scale. Similarity uses dense cosine when
/// vectors are present, otherwise lexical Jaccard.
pub fn mmr_diversify(
    fused: &[Hit],
    claims: &[Claim],
    vectors: Option<&std::collections::HashMap<String, Vec<f32>>>,
    lambda: f32,
    top: usize,
) -> Vec<Hit> {
    if fused.len() <= top || top == 0 {
        return fused.to_vec();
    }
    let by_id: std::collections::HashMap<&str, &Claim> =
        claims.iter().map(|c| (c.id.as_str(), c)).collect();
    let max_rel = fused
        .iter()
        .map(|h| h.score)
        .fold(0.0f64, f64::max)
        .max(1e-12);
    let lambda = f64::from(lambda.clamp(0.0, 1.0));

    let mut remaining: Vec<&Hit> = fused.iter().collect();
    let mut selected: Vec<Hit> = Vec::with_capacity(top);
    while selected.len() < top && !remaining.is_empty() {
        let mut best: Option<(f64, usize)> = None;
        for (i, h) in remaining.iter().enumerate() {
            let rel = h.score / max_rel;
            let redundancy = selected
                .iter()
                .map(|s| {
                    let (ca, cb) = match (by_id.get(h.id.as_str()), by_id.get(s.id.as_str())) {
                        (Some(a), Some(b)) => (a, b),
                        _ => return 0.0,
                    };
                    let dense_sim = vectors.and_then(|v| {
                        let va = v.get(ca.id.as_str())?;
                        let vb = v.get(cb.id.as_str())?;
                        Some(f64::from(crate::embed::cosine(va, vb).clamp(0.0, 1.0)))
                    });
                    dense_sim.unwrap_or_else(|| f64::from(lexical_sim(ca, cb)))
                })
                .fold(0.0f64, f64::max);
            let mmr = lambda * rel - (1.0 - lambda) * redundancy;
            match best {
                Some((best_score, _)) if best_score >= mmr => {}
                _ => best = Some((mmr, i)),
            }
        }
        if let Some((_, i)) = best {
            let hit = remaining.remove(i);
            selected.push(hit.clone());
        } else {
            break;
        }
    }
    selected
}

/// Pseudo-relevance feedback (Rocchio, 1971): expand the query with the
/// highest-signal terms of the top-ranked documents.
///
/// Classical, dependency-free query expansion that recovers recall when the
/// user's vocabulary differs from the corpus's (e.g. "gate" vs "phase gate
/// approval"). Terms are ranked by summed BM25 idf × in-doc frequency over the
/// feedback set, and returned in expansion order so callers can append them.
pub fn prf_expand(
    index: &Index,
    filtered: &[Claim],
    query: &str,
    feedback_docs: usize,
    n_terms: usize,
) -> Vec<String> {
    let base_terms = tokenize(query);
    if base_terms.is_empty() {
        return Vec::new();
    }
    let top = index.search(filtered, query, feedback_docs.max(1));
    if top.is_empty() {
        return Vec::new();
    }
    let qset: std::collections::HashSet<&str> = base_terms.iter().map(String::as_str).collect();
    let n = index.n_docs() as f64;
    let mut weights: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for hit in &top {
        let Some(terms) = index.tf.get(&hit.id) else {
            continue;
        };
        for (term, tf) in terms {
            if qset.contains(term.as_str()) {
                continue; // never echo query terms back
            }
            let df = f64::from(*index.df.get(term).unwrap_or(&0));
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
            *weights.entry(term.clone()).or_insert(0.0) += idf * f64::from(*tf);
        }
    }
    let mut ranked: Vec<(String, f64)> = weights.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked
        .into_iter()
        .take(n_terms.max(1))
        .map(|(t, _)| t)
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
pub fn run_golden(
    index: &Index,
    all: &[Claim],
    cases: &[GoldenCase],
    vectors: Option<&VectorIndexFile>,
    qvecs: Option<&std::collections::HashMap<String, Vec<f32>>>,
) -> Result<(usize, usize, Vec<String>), KemError> {
    run_golden_fusion(index, all, cases, vectors, qvecs, false)
}

/// Golden evaluation with an explicit fusion choice (2026-11-12): `rrf=false`
/// runs the legacy weighted α-fusion (baseline), `rrf=true` runs Reciprocal
/// Rank Fusion. Both keep the same refusal floor, so refusal cases stay
/// comparable across fusions.
pub fn run_golden_fusion(
    index: &Index,
    all: &[Claim],
    cases: &[GoldenCase],
    vectors: Option<&VectorIndexFile>,
    qvecs: Option<&std::collections::HashMap<String, Vec<f32>>>,
    rrf: bool,
) -> Result<(usize, usize, Vec<String>), KemError> {
    let vmap: Option<std::collections::HashMap<String, Vec<f32>>> = vectors.map(|v| {
        fresh_lookup(v, all)
            .into_iter()
            .map(|(k, vec)| (k.to_string(), vec.to_vec()))
            .collect()
    });
    let mut pass = 0usize;
    let mut failures = Vec::new();
    for case in cases {
        let filtered = visible_at(all, None);
        let bm25 = index.scores_all(&filtered, &case.query);
        let qvec: Option<&[f32]> = qvecs.and_then(|m| m.get(&case.query)).map(|v| v.as_slice());
        let hits = if rrf {
            rrf_fuse(
                &filtered,
                &bm25,
                vmap.as_ref(),
                qvec,
                1,
                REFUSAL_FLOOR,
                RRF_K,
            )
        } else {
            hybrid_fuse(&filtered, &bm25, vmap.as_ref(), qvec, 1, 0.5, REFUSAL_FLOOR)
        };
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
        out.push(
            serde_json::from_str(line)
                .map_err(|e| KemError::InvalidInput(format!("{path:?} line {}: {e}", i + 1)))?,
        );
    }
    Ok(out)
}

/// TREC-standard retrieval quality metrics (2026-11-12): beyond top-1
/// accuracy, these measure ranking *shape* — where the relevant document
/// lands, not just whether it is first.
///
/// - `ndcg`: mean nDCG@k with binary relevance, `1/log2(rank+1)` per case
///   (position-discounted; a relevant doc at rank 3 scores higher than at
///   rank 9).
/// - `mrr`: mean reciprocal rank of the first relevant document.
/// - `recall`: fraction of cases with the relevant document in the top-k.
#[derive(Debug, Clone, Default)]
pub struct RetrievalMetrics {
    pub ndcg: f64,
    pub mrr: f64,
    pub recall: f64,
    pub evaluated: usize,
    /// Cases whose relevant document ranked first (or honest refusal).
    pub top1: usize,
}

/// Evaluate a golden set with full TREC metrics for both fusion modes.
/// Returns the metrics plus the failure list (query/expected/got) for the
/// stricter top-1 criterion.
pub fn evaluate_golden(
    index: &Index,
    all: &[Claim],
    cases: &[GoldenCase],
    vectors: Option<&VectorIndexFile>,
    qvecs: Option<&std::collections::HashMap<String, Vec<f32>>>,
) -> Result<(RetrievalMetrics, RetrievalMetrics, Vec<String>), KemError> {
    let vmap: Option<std::collections::HashMap<String, Vec<f32>>> = vectors.map(|v| {
        fresh_lookup(v, all)
            .into_iter()
            .map(|(k, vec)| (k.to_string(), vec.to_vec()))
            .collect()
    });
    let k = 10usize;
    let mut weighted = RetrievalMetrics::default();
    let mut rrf = RetrievalMetrics::default();
    let mut failures = Vec::new();
    for case in cases {
        let filtered = visible_at(all, None);
        let bm25 = index.scores_all(&filtered, &case.query);
        let qvec: Option<&[f32]> = qvecs.and_then(|m| m.get(&case.query)).map(|v| v.as_slice());
        let w = hybrid_fuse(&filtered, &bm25, vmap.as_ref(), qvec, k, 0.5, REFUSAL_FLOOR);
        let r = rrf_fuse(
            &filtered,
            &bm25,
            vmap.as_ref(),
            qvec,
            k,
            REFUSAL_FLOOR,
            RRF_K,
        );
        accumulate(&mut weighted, &w, case, k);
        accumulate(&mut rrf, &r, case, k);
        let top1_ok = |hits: &[Hit]| {
            if case.expect == "none" {
                hits.is_empty()
            } else {
                hits.first().is_some_and(|h| h.id == case.expect)
            }
        };
        if !top1_ok(&r) {
            failures.push(format!(
                "query {:?}: expected {}, got {}",
                case.query,
                case.expect,
                r.first().map_or("no-hit".to_string(), |h| h.id.clone())
            ));
        }
    }
    for m in [&mut weighted, &mut rrf] {
        if m.evaluated > 0 {
            let n = m.evaluated as f64;
            m.ndcg /= n;
            m.mrr /= n;
            m.recall /= n;
        }
    }
    Ok((weighted, rrf, failures))
}

/// Accumulate one case's contribution into a metrics accumulator.
fn accumulate(m: &mut RetrievalMetrics, hits: &[Hit], case: &GoldenCase, k: usize) {
    m.evaluated += 1;
    if case.expect == "none" {
        // Refusal case: a hit anywhere in the top-k is a failure. Reward the
        // empty result set with a perfect score (honest refusal is the goal).
        if hits.is_empty() {
            m.ndcg += 1.0;
            m.mrr += 1.0;
            m.recall += 1.0;
            m.top1 += 1;
        }
        return;
    }
    let Some(rank) = hits.iter().position(|h| h.id == case.expect).map(|p| p + 1) else {
        return;
    };
    if rank == 1 {
        m.top1 += 1;
    }
    if rank <= k {
        m.ndcg += 1.0 / ((rank as f64) + 1.0).log2();
        m.recall += 1.0;
    }
    m.mrr += 1.0 / rank as f64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ClaimType, Evidence, Track};
    use crate::seed;
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
            recorded_at: Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, 0).unwrap(),
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
        assert_eq!(
            hits.first().map(|h| h.id.as_str()),
            Some("claim-20261107-001")
        );
    }

    #[test]
    fn drafts_and_retired_are_invisible() {
        let claims = vec![
            claim(
                "claim-20261107-001",
                "Phase gates",
                "phase gate blocked",
                ClaimStatus::Draft,
            ),
            claim(
                "claim-20261107-002",
                "Phase gates two",
                "phase gate blocked",
                ClaimStatus::Retired,
            ),
        ];
        let filtered = visible_at(&claims, None);
        assert!(filtered.is_empty());
        let idx = Index::build(&claims);
        assert!(idx.search(&filtered, "phase gate", 5).is_empty());
    }

    #[test]
    fn expired_claims_are_invisible_but_history_queries_can_ask_past_dates() {
        let mut c = claim(
            "claim-20261107-001",
            "Old rule",
            "color tokens policy",
            ClaimStatus::Deprecated,
        );
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
            GoldenCase {
                query: "phase gate".into(),
                expect: "claim-20261107-001".into(),
            },
            GoldenCase {
                query: "kubernetes autoscaling cluster".into(),
                expect: "none".into(),
            },
        ];
        let (pass, total, fails) = run_golden(&idx, &claims, &cases, None, None).unwrap();
        assert_eq!((pass, total), (2, 2));
        assert!(fails.is_empty());
    }
    #[test]
    fn refusal_floor_drops_weak_vector_only_hits() {
        let c = claim(
            "claim-20261107-009",
            "Redis caching layer",
            "redis cache stores session data",
            ClaimStatus::Published,
        );
        let idx = Index::build(std::slice::from_ref(&c));
        let bm25 = idx.scores_all(std::slice::from_ref(&c), "quantum entanglement physics");
        assert!(bm25.iter().all(|(_, s)| *s == 0.0));
        let mut vectors = std::collections::HashMap::new();
        vectors.insert("claim-20261107-009".to_string(), vec![0.9f32, 0.1]);
        let qv = [0.3f32, 0.95]; // cosine ~0.28 < floor
        let hits = hybrid_fuse(
            std::slice::from_ref(&c),
            &bm25,
            Some(&vectors),
            Some(&qv),
            5,
            0.5,
            REFUSAL_FLOOR,
        );
        assert!(hits.is_empty(), "weak vector-only hit must be refused");
        let qv2 = [0.99f32, 0.05]; // cosine ~0.99 > floor
        let hits2 = hybrid_fuse(
            std::slice::from_ref(&c),
            &bm25,
            Some(&vectors),
            Some(&qv2),
            5,
            0.5,
            REFUSAL_FLOOR,
        );
        assert_eq!(hits2.len(), 1);
        assert_eq!(hits2[0].via, "hybrid");
    }

    // ── SOTA retrieval upgrades (2026-11-12) ──────────────────────────────

    fn three_claims() -> Vec<Claim> {
        vec![
            claim(
                "c-phase",
                "Phase gate status",
                "Phase A is not approved and Phase B is blocked for the platform",
                ClaimStatus::Published,
            ),
            claim(
                "c-zerojs",
                "Zero JavaScript",
                "The browser receives only HTML CSS SVG images and fonts",
                ClaimStatus::Published,
            ),
            claim(
                "c-pgr",
                "Post redirect get",
                "Every mutation redirects with 303 to avoid double submit",
                ClaimStatus::Published,
            ),
        ]
    }

    #[test]
    fn rrf_fuses_ranks_and_keeps_honest_refusal() {
        let claims = three_claims();
        let idx = Index::build(&claims);
        let filtered = visible_at(&claims, None);
        let bm25 = idx.scores_all(&filtered, "phase gate blocked");
        let hits = rrf_fuse(&filtered, &bm25, None, None, 3, REFUSAL_FLOOR, RRF_K);
        assert_eq!(hits.first().map(|h| h.id.as_str()), Some("c-phase"));
        // RRF score of the top hit must be exactly 1/(k+0) with one path.
        let expected = 1.0 / (RRF_K + 0.0);
        assert!(
            (hits[0].score - expected).abs() < 1e-12,
            "rrf top score {} vs {expected}",
            hits[0].score
        );
        // Refusal preserved: zero lexical support + weak cosine => no hit.
        let qv = [0.3f32, 0.95];
        let mut vectors = std::collections::HashMap::new();
        for (i, c) in claims.iter().enumerate() {
            let base = 0.9f32 - i as f32 * 0.2;
            vectors.insert(c.id.clone(), vec![base, 0.1]);
        }
        let hits2 = rrf_fuse(
            &filtered,
            &[],
            Some(&vectors),
            Some(&qv),
            5,
            REFUSAL_FLOOR,
            RRF_K,
        );
        assert!(
            hits2.is_empty(),
            "weak vector-only hits must be refused under RRF too"
        );
    }

    #[test]
    fn mmr_diversifies_near_duplicates() {
        let mut claims = three_claims();
        // Two near-identical claims plus a weaker but topically distinct hit,
        // so the fused set is larger than the window (MMR only engages then).
        claims.push(claim(
            "c-zerojs2",
            "Zero JavaScript again",
            "The browser receives only HTML CSS SVG images and fonts",
            ClaimStatus::Published,
        ));
        claims.push(claim(
            "c-render",
            "Server rendering",
            "The server renders HTML templates with Askama",
            ClaimStatus::Published,
        ));
        let idx = Index::build(&claims);
        let filtered = visible_at(&claims, None);
        let bm25 = idx.scores_all(&filtered, "javascript html css fonts");
        let fused = rrf_fuse(&filtered, &bm25, None, None, 4, REFUSAL_FLOOR, RRF_K);
        assert!(fused.len() > 2, "need more candidates than the window");
        let selected = mmr_diversify(&fused, &claims, None, 0.5, 2);
        assert_eq!(selected.len(), 2);
        // The two zero-JS claims are lexical near-duplicates; at λ=0.5 the
        // redundancy penalty must push one of them out of the top-2.
        let zero_js = selected
            .iter()
            .filter(|h| h.id == "c-zerojs" || h.id == "c-zerojs2")
            .count();
        assert!(
            zero_js <= 1,
            "MMR must not fill the window with near-duplicates: {selected:?}"
        );
    }

    #[test]
    fn prf_expands_with_corpus_terms_not_query_echo() {
        let claims = three_claims();
        let idx = Index::build(&claims);
        let filtered = visible_at(&claims, None);
        let expanded = prf_expand(&idx, &filtered, "phase gate blocked", 1, 4);
        assert!(!expanded.is_empty(), "PRF must return expansion terms");
        for term in &expanded {
            assert!(
                !["phase", "gate", "blocked"].contains(&term.as_str()),
                "query terms must never be echoed back: {expanded:?}"
            );
        }
    }

    #[test]
    fn rrf_matches_or_beats_weighted_fusion_on_golden() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden-20.json");
        let Ok(cases) = load_golden(&path) else {
            return; // golden set absent in this checkout: skip, never fail
        };
        let claims = seed::seed_claims();
        let idx = Index::build(&claims);
        let filtered = visible_at(&claims, None);
        let mut weighted = 0usize;
        let mut rrf = 0usize;
        for case in &cases {
            let bm25 = idx.scores_all(&filtered, &case.query);
            let w = hybrid_fuse(&filtered, &bm25, None, None, 1, 0.5, REFUSAL_FLOOR);
            let r = rrf_fuse(&filtered, &bm25, None, None, 1, REFUSAL_FLOOR, RRF_K);
            let ok = |hits: &[Hit]| {
                if case.expect == "none" {
                    hits.is_empty()
                } else {
                    hits.first().is_some_and(|h| h.id == case.expect)
                }
            };
            weighted += usize::from(ok(&w));
            rrf += usize::from(ok(&r));
        }
        assert!(
            rrf >= weighted,
            "RRF must not regress top-1 accuracy: rrf={rrf} weighted={weighted}"
        );
    }

    #[test]
    fn trec_metrics_are_computed_correctly() {
        // Synthetic case: relevant doc at rank 3 (1-indexed).
        // nDCG@10 = 1/log2(3+1) = 0.5; MRR = 1/3; recall@10 = 1.
        let cases = vec![GoldenCase {
            query: "target".into(),
            expect: "c3".into(),
        }];
        let claims = vec![
            claim("c1", "alpha", "alpha body", ClaimStatus::Published),
            claim("c2", "beta", "beta body", ClaimStatus::Published),
            claim("c3", "target", "target body", ClaimStatus::Published),
        ];
        let idx = Index::build(&claims);
        let filtered = visible_at(&claims, None);
        // Hand-built scores forcing c3 to rank 3 (c1 > c2 > c3).
        let scores = vec![
            ("c1".to_string(), 9.0),
            ("c2".to_string(), 5.0),
            ("c3".to_string(), 1.0),
        ];
        let hits = rrf_fuse(&filtered, &scores, None, None, 10, REFUSAL_FLOOR, RRF_K);
        let mut m = RetrievalMetrics::default();
        accumulate(&mut m, &hits, &cases[0], 10);
        assert_eq!(m.evaluated, 1);
        assert!((m.ndcg - 0.5).abs() < 1e-9, "nDCG = 1/log2(4) = 0.5");
        assert!((m.mrr - 1.0 / 3.0).abs() < 1e-9, "MRR = 1/3");
        assert!((m.recall - 1.0).abs() < 1e-9, "recall@10 = 1");
        assert_eq!(m.top1, 0, "rank-3 is not top-1");
        // The real BM25 eval (c3 top-1 for "target") must yield perfect metrics.
        let (w, _, _) = evaluate_golden(&idx, &claims, &cases, None, None).expect("eval");
        assert_eq!(w.top1, 1);
        assert!((w.ndcg - 1.0).abs() < 1e-9);
        assert!((w.mrr - 1.0).abs() < 1e-9);
        assert!((w.recall - 1.0).abs() < 1e-9);
    }
}
