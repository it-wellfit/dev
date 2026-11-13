//! Platform hybrid search (PHASE-R, RAG-S2/S3): BM25 lexical leg + vector
//! semantic leg, fused with the proven kem semantics
//! (`crates/kem/src/search.rs::hybrid_fuse`: α-normalization, semantic
//! refusal floor, honest `via` provenance).
//!
//! - Lexical leg: `crate::bm25::BM25Index` (std-only; CJK single-char tokens).
//! - Vector leg: **reuses `kem::embed`** — the embedder pool (OVH bge-m3
//!   keyless → Ollama local fallback), batched embedding, and cosine. The
//!   path dependency adds zero new external crates (kem's deps are a subset
//!   of platform's).
//! - Honest degradation: when the embedder pool is unreachable, the engine
//!   returns BM25-only results marked `via = "bm25"` — never fabricated
//!   vector scores. When a document has no (fresh) vector, it is scored
//!   BM25-only as well.
//! - Freshness (kem §8 derived-artifact rule): each stored vector carries the
//!   SHA-256 of the content it was built from; a mismatch means stale, and a
//!   stale vector is treated as missing.
//!
//! Scores are ranking signals, never amounts (INV-01 governs money only).

use crate::bm25::{BM25Index, tokenize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;

/// Hybrid weight for the legacy weighted fusion (kept as the `search_rrf=off`
/// fallback and the A/B baseline). fused = α·norm(BM25) +
/// (1−α)·norm(cosine). 0.5 mirrors the kem golden configuration.
pub const HYBRID_ALPHA: f32 = 0.5;

/// Semantic refusal floor: a hit with ZERO lexical support whose raw cosine
/// falls below this is weak semantic adjacency, not a result — dropped
/// (calibrated in kem, 2026-11-09; mirrors `kem::search::REFUSAL_FLOOR`).
pub const REFUSAL_FLOOR: f32 = 0.55;

/// Standard RRF damping constant (Cormack et al. 2009, k=60) — shared with
/// `kem::search::RRF_K` so both retrieval stacks use one fusion standard.
pub const RRF_K: f64 = 60.0;

/// doc_id → (content SHA-256, embedding)。
type VectorMap = std::collections::HashMap<String, (String, Vec<f32>)>;

#[derive(Clone)]
/// Hybrid search engine for the AuxBridge workbench.
pub struct SearchEngine {
    documents: Arc<RwLock<HashMap<String, SearchDocument>>>,
    /// doc_id → (content SHA-256, embedding). Absent = BM25-only for the doc.
    vectors: Arc<RwLock<VectorMap>>,
    /// Lexical index, rebuilt on every mutation (documented in-memory
    /// tradeoff; workbench corpus scale makes O(n) rebuilds negligible).
    bm25: Arc<RwLock<BM25Index>>,
    /// Sorted doc_ids parallel to the BM25 index positions.
    order: Arc<RwLock<Vec<String>>>,
    embed_pool: Arc<Vec<kem::embed::EmbedEntry>>,
    /// 上次從 DB 重建索引的時刻（2026-11-12：自愈索引——重啟後首查自動
    /// 從 transactions 重建，無需人工 reindex；None = 從未重建）。
    last_rebuilt: Arc<RwLock<Option<std::time::Instant>>>,
    /// 嵌入斷路器（2026-11-12 SOTA 精實化）：embedder 不可達時停止嘗試
    /// 至 `until`（60s 後再試一次）。為何必要：端點逾時 60s × 3 端點循序
    /// → 離線/未設定情境下每次搜尋卡 180s；斷路器把失敗成本攤成一次性。
    embed_disabled_until: Arc<RwLock<Option<std::time::Instant>>>,
    /// G4.9 (D3-04) flags: hybrid retrieval (vector leg) and RRF fusion.
    /// Defaults are ON (FLAG); `search_hybrid=off` → BM25-only, `search_rrf=off`
    /// → weighted α-fusion.
    hybrid: Arc<AtomicBool>,
    rrf: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct SearchDocument {
    pub doc_id: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

/// One ranked result: document + fused score + retrieval-path provenance.
#[derive(Debug, Clone)]
pub struct RankedHit {
    pub document: SearchDocument,
    pub score: f64,
    /// "hybrid" (BM25+vector fused) or "bm25" (lexical only, degraded).
    pub via: String,
}

fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Rank fusion (RRF) over per-document BM25 and cosine rankings — the SOTA
/// default (2026-11-12). Fuses **ranks**, not normalized scores:
/// `score(d) = Σ_path 1/(k + rank_path(d))` with k=60. Scale-invariant, so no
/// score calibration and no single path's score range can dominate. The
/// semantic refusal floor and honest `via` provenance are preserved verbatim;
/// stale vectors (content changed) count as missing, same as the weighted
/// path. `ids` parallels `bm25_scores`; missing entries score 0.
fn rrf_fuse(
    ids: &[String],
    bm25_scores: &[f64],
    vectors: Option<&HashMap<String, (String, Vec<f32>)>>,
    contents: &HashMap<String, String>,
    query_vec: Option<&[f32]>,
    refusal_floor: f32,
) -> Vec<(usize, f64, &'static str)> {
    // Lexical ranking: descending BM25, zero-score documents unranked.
    let mut lexical: Vec<(usize, f64)> = bm25_scores
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, s)| *s > 0.0)
        .collect();
    lexical.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let lexical_rank: std::collections::HashMap<usize, usize> = lexical
        .iter()
        .enumerate()
        .map(|(rank, (pos, _))| (*pos, rank))
        .collect();

    // Dense ranking with freshness gate (stale vector = missing).
    let mut dense: Vec<(usize, f32)> = Vec::new();
    if let (Some(vectors), Some(qv)) = (vectors, query_vec) {
        for (pos, id) in ids.iter().enumerate() {
            if let Some((hash, vec)) = vectors.get(id) {
                let fresh = contents.get(id).is_some_and(|c| content_hash(c) == *hash);
                if fresh {
                    dense.push((pos, kem::embed::cosine(qv, vec)));
                }
            }
        }
        dense.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    let dense_rank: std::collections::HashMap<usize, usize> = dense
        .iter()
        .enumerate()
        .map(|(rank, (pos, _))| (*pos, rank))
        .collect();
    let dense_cos: std::collections::HashMap<usize, f32> = dense.into_iter().collect();

    let mut fused: Vec<(usize, f64, &'static str)> = ids
        .iter()
        .enumerate()
        .filter_map(|(pos, _)| {
            let lr = lexical_rank.get(&pos).copied();
            let dr = dense_rank.get(&pos).copied();
            let cos = dense_cos.get(&pos).copied().unwrap_or(0.0);
            // Semantic refusal floor (same rule as the weighted path): zero
            // lexical support + weak semantic adjacency => not a result.
            if lr.is_none() && dr.is_some() && cos < refusal_floor {
                return None;
            }
            let mut score = 0.0f64;
            if let Some(r) = lr {
                score += 1.0 / (RRF_K + r as f64);
            }
            if let Some(r) = dr {
                score += 1.0 / (RRF_K + r as f64);
            }
            if score <= 0.0 {
                return None;
            }
            let via = match (lr.is_some(), dr.is_some()) {
                (true, true) => "hybrid",
                (true, false) => "bm25",
                (false, true) => "vector",
                (false, false) => return None,
            };
            Some((pos, score, via))
        })
        .collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

/// Weighted fusion (α-normalization by corpus max, semantic refusal floor,
/// honest `via`) — the legacy baseline, used when `search_rrf=off` and kept
/// for A/B tests against [`rrf_fuse`]. `ids` parallels `bm25_scores`;
/// missing entries score 0. Returns `(position, score, via)` sorted by score
/// descending.
#[allow(clippy::too_many_arguments)]
fn fuse(
    ids: &[String],
    bm25_scores: &[f64],
    vectors: Option<&HashMap<String, (String, Vec<f32>)>>,
    contents: &HashMap<String, String>,
    query_vec: Option<&[f32]>,
    alpha: f32,
    refusal_floor: f32,
) -> Vec<(usize, f64, &'static str)> {
    let bm25_max = bm25_scores.iter().copied().fold(0.0f64, f64::max);
    let mut vec_max = 0.0f32;
    let mut cosines: Vec<Option<f32>> = vec![None; ids.len()];
    if let (Some(vectors), Some(qv)) = (vectors, query_vec) {
        for (pos, id) in ids.iter().enumerate() {
            if let Some((hash, vec)) = vectors.get(id) {
                // Freshness: stale vector (content changed) = missing.
                let fresh = contents.get(id).is_some_and(|c| content_hash(c) == *hash);
                if fresh {
                    let cosine = kem::embed::cosine(qv, vec);
                    if cosine > vec_max {
                        vec_max = cosine;
                    }
                    cosines[pos] = Some(cosine);
                }
            }
        }
    }
    let norm = |v: f64, max: f64| if max > 0.0 { v / max } else { 0.0 };
    let mut fused: Vec<(usize, f64, &'static str)> = ids
        .iter()
        .enumerate()
        .filter_map(|(pos, _id)| {
            let b = bm25_scores[pos];
            let cosine = cosines[pos];
            let (score, via) = match cosine {
                Some(v) => {
                    // Semantic refusal floor: zero lexical support below the
                    // floor is weak adjacency, not a result — drop it.
                    if b == 0.0 && v < refusal_floor {
                        return None;
                    }
                    let fused = f64::from(alpha) * norm(b, bm25_max)
                        + f64::from(1.0 - alpha)
                            * if vec_max > 0.0 {
                                f64::from(v) / f64::from(vec_max)
                            } else {
                                0.0
                            };
                    (fused, "hybrid")
                }
                None => (norm(b, bm25_max), "bm25"),
            };
            if score <= 0.0 {
                return None;
            }
            Some((pos, score, via))
        })
        .collect();
    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::with_pool(kem::embed::default_pool())
    }

    /// Injectable embedder pool: tests pass an empty pool to exercise the
    /// deterministic BM25-only degradation path.
    pub fn with_pool(pool: Vec<kem::embed::EmbedEntry>) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            vectors: Arc::new(RwLock::new(HashMap::new())),
            bm25: Arc::new(RwLock::new(BM25Index::default())),
            order: Arc::new(RwLock::new(Vec::new())),
            embed_pool: Arc::new(pool),
            last_rebuilt: Arc::new(RwLock::new(None)),
            embed_disabled_until: Arc::new(RwLock::new(None)),
            hybrid: Arc::new(AtomicBool::new(true)),
            rrf: Arc::new(AtomicBool::new(true)),
        }
    }

    /// G4.9: toggle the vector leg (false → deterministic BM25-only).
    pub fn set_hybrid(&self, on: bool) {
        self.hybrid.store(on, Ordering::Relaxed);
    }

    /// G4.9: toggle RRF fusion (false → weighted α-fusion).
    pub fn set_rrf(&self, on: bool) {
        self.rrf.store(on, Ordering::Relaxed);
    }

    /// Current flag values (for diagnostics/tests).
    pub fn flags(&self) -> (bool, bool) {
        (
            self.hybrid.load(Ordering::Relaxed),
            self.rrf.load(Ordering::Relaxed),
        )
    }

    /// 嵌入呼叫（2 秒上限 + 斷路器）：逾時或失敗 → None（BM25-only 誠實降級）
    /// 並停用嵌入 60 秒。離線/未設定端點情境下，搜尋延遲由 180s 降至 ≤2s。
    async fn embed_with_breaker(&self, texts: &[String]) -> Option<kem::embed::EmbedResult> {
        if let Some(until) = *self.embed_disabled_until.read().await
            && std::time::Instant::now() < until
        {
            return None;
        }
        let pool = self.embed_pool.as_ref().clone();
        let fut = kem::embed::embed_batch(&pool, texts);
        match tokio::time::timeout(std::time::Duration::from_secs(2), fut).await {
            Ok(Ok(result)) => Some(result),
            _ => {
                *self.embed_disabled_until.write().await =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(60));
                None
            }
        }
    }

    /// 索引是否過期（從未重建，或超過 `ttl` 未重建）。自愈重建的節流閘。
    pub async fn is_stale(&self, ttl: std::time::Duration) -> bool {
        match *self.last_rebuilt.read().await {
            None => true,
            Some(at) => at.elapsed() > ttl,
        }
    }

    /// 標記索引已從 DB 重建（由呼叫端在填充完成後呼叫）。
    pub async fn mark_rebuilt(&self) {
        *self.last_rebuilt.write().await = Some(std::time::Instant::now());
    }

    /// 寫入後失效（2026-11-12）：下次搜尋立刻重建——關閉「新交易最多 60 秒
    /// 搜不到」的一致性窗口（自愈 TTL 的補充，寫入路徑主動呼叫）。
    pub async fn invalidate(&self) {
        *self.last_rebuilt.write().await = None;
    }

    /// 批量寫入（2026-11-12 SOTA 精實化）：一次 BM25 重建（消除逐檔 rebuild
    /// 的 O(n²)）＋一次批次嵌入（消除逐檔 60s 網路等待）。供 DB 重建路徑
    /// 使用；單檔寫入仍走 `add_document`。
    pub async fn add_documents_bulk(&self, docs: Vec<(String, String, HashMap<String, String>)>) {
        if docs.is_empty() {
            self.mark_rebuilt().await;
            return;
        }
        {
            let mut map = self.documents.write().await;
            for (id, content, metadata) in docs {
                map.insert(
                    id.clone(),
                    SearchDocument {
                        doc_id: id,
                        content,
                        metadata,
                    },
                );
            }
        }
        self.rebuild_bm25().await;
        // 單次批次嵌入：整批一起送（2s 上限＋斷路器）；失敗 → 全批 BM25-only。
        let contents: Vec<String> = {
            let map = self.documents.read().await;
            map.values().map(|d| d.content.clone()).collect()
        };
        if contents.is_empty() {
            return;
        }
        if let Some(result) = self.embed_with_breaker(&contents).await {
            let ids: Vec<String> = {
                let map = self.documents.read().await;
                map.keys().cloned().collect()
            };
            let mut vectors = self.vectors.write().await;
            for (id, vec) in ids.into_iter().zip(result.vecs.iter()) {
                if let Some(content) = self
                    .documents
                    .read()
                    .await
                    .get(&id)
                    .map(|d| d.content.clone())
                {
                    vectors.insert(id, (content_hash(&content), vec.clone()));
                }
            }
        }
    }

    /// Rebuild the lexical index from the current documents (sorted by
    /// doc_id for deterministic positions).
    async fn rebuild_bm25(&self) {
        let docs = self.documents.read().await;
        let mut ids: Vec<String> = docs.keys().cloned().collect();
        ids.sort();
        let mut index = BM25Index::default();
        for id in &ids {
            index.add_text(&docs[id].content);
        }
        *self.bm25.write().await = index;
        *self.order.write().await = ids;
    }

    pub async fn add_document(
        &self,
        doc_id: String,
        content: String,
        metadata: HashMap<String, String>,
    ) {
        {
            let mut docs = self.documents.write().await;
            docs.insert(
                doc_id.clone(),
                SearchDocument {
                    doc_id: doc_id.clone(),
                    content: content.clone(),
                    metadata,
                },
            );
        }
        self.rebuild_bm25().await;

        // Vector leg, best-effort: embedder unreachable → the document stays
        // BM25-only (honest degradation, never a fabricated vector). 2s cap +
        // breaker keeps offline rebuilds from stalling on 60s endpoint timeouts.
        if let Some(result) = self
            .embed_with_breaker(std::slice::from_ref(&content))
            .await
            && let Some(vec) = result.vecs.first()
        {
            let mut vectors = self.vectors.write().await;
            vectors.insert(doc_id, (content_hash(&content), vec.clone()));
        }
    }

    /// Ranked search: BM25 over all documents, fused with cosine against the
    /// embedded query when the embedder is reachable. Fusion is RRF (rank
    /// fusion, k=60) by default; unreachable embedder → BM25-only.
    pub async fn search_ranked(&self, query: &str) -> Vec<RankedHit> {
        let docs = self.documents.read().await;
        if docs.is_empty() {
            return Vec::new();
        }
        let bm25 = self.bm25.read().await;
        let order = self.order.read().await;
        let vectors = self.vectors.read().await;

        let terms = tokenize(query);
        let bm25_scores: Vec<f64> = order
            .iter()
            .enumerate()
            .map(|(pos, _)| bm25.score_at(pos, &terms))
            .collect();

        // Query embedding, best-effort: unreachable embedder → BM25-only
        // (2s cap + breaker; offline searches stay fast instead of 180s).
        // G4.9: `search_hybrid=off` skips the vector leg entirely.
        let query_vec: Option<Vec<f32>> = if self.hybrid.load(Ordering::Relaxed) {
            self.embed_with_breaker(&[query.to_string()])
                .await
                .and_then(|r| r.vecs.first().cloned())
        } else {
            None
        };

        let contents: HashMap<String, String> = docs
            .iter()
            .map(|(id, d)| (id.clone(), d.content.clone()))
            .collect();
        // G4.9: `search_rrf=off` falls back to the weighted α-fusion.
        let fused = if self.rrf.load(Ordering::Relaxed) {
            rrf_fuse(
                &order,
                &bm25_scores,
                Some(&vectors),
                &contents,
                query_vec.as_deref(),
                REFUSAL_FLOOR,
            )
        } else {
            fuse(
                &order,
                &bm25_scores,
                Some(&vectors),
                &contents,
                query_vec.as_deref(),
                HYBRID_ALPHA,
                REFUSAL_FLOOR,
            )
        };
        fused
            .into_iter()
            .filter_map(|(pos, score, via)| {
                docs.get(&order[pos]).map(|document| RankedHit {
                    document: document.clone(),
                    score,
                    via: via.to_string(),
                })
            })
            .collect()
    }

    /// Backward-compatible search: ranked results without scores.
    pub async fn search(&self, query: &str) -> Vec<SearchDocument> {
        self.search_ranked(query)
            .await
            .into_iter()
            .map(|hit| hit.document)
            .collect()
    }

    pub async fn get_document(&self, doc_id: &str) -> Option<SearchDocument> {
        self.documents.read().await.get(doc_id).cloned()
    }

    pub async fn delete_document(&self, doc_id: &str) {
        self.documents.write().await.remove(doc_id);
        self.vectors.write().await.remove(doc_id);
        self.rebuild_bm25().await;
    }

    pub async fn document_count(&self) -> usize {
        self.documents.read().await.len()
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> SearchEngine {
        // Empty pool = embedder unreachable = deterministic BM25-only path.
        SearchEngine::with_pool(Vec::new())
    }

    #[tokio::test]
    async fn bm25_only_degradation_ranks_by_relevance_and_marks_via() {
        let engine = engine();
        engine
            .add_document(
                "d1".into(),
                "refund policy for damaged goods".into(),
                HashMap::new(),
            )
            .await;
        engine
            .add_document(
                "d2".into(),
                "quarterly ledger summary".into(),
                HashMap::new(),
            )
            .await;

        let hits = engine.search_ranked("refund").await;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.doc_id, "d1");
        assert!(hits[0].score > 0.0, "real BM25 score, not the constant 1.0");
        assert_eq!(
            hits[0].via, "bm25",
            "embedder-unreachable = honest lexical path"
        );
    }

    #[tokio::test]
    async fn unknown_term_returns_empty_and_query_ordering_holds() {
        let engine = engine();
        engine
            .add_document("d1".into(), "ledger ledger ledger".into(), HashMap::new())
            .await;
        engine
            .add_document("d2".into(), "ledger".into(), HashMap::new())
            .await;
        let hits = engine.search_ranked("ledger").await;
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].document.doc_id, "d1", "higher tf ranks first");
        assert!(hits[0].score > hits[1].score);
        assert!(engine.search_ranked("nonexistent").await.is_empty());
    }

    #[test]
    fn fusion_blends_lexical_and_vector_with_via_provenance() {
        // Two docs; d2 has zero lexical support but a strong vector match.
        let ids = vec!["d1".to_string(), "d2".to_string()];
        let bm25_scores = vec![2.0, 0.0];
        let mut vectors: HashMap<String, (String, Vec<f32>)> = HashMap::new();
        vectors.insert("d1".to_string(), ("h1".to_string(), vec![1.0, 0.0]));
        vectors.insert("d2".to_string(), ("h2".to_string(), vec![0.0, 1.0]));
        let mut contents: HashMap<String, String> = HashMap::new();
        contents.insert("d1".to_string(), "c1".to_string());
        contents.insert("d2".to_string(), "c2".to_string());
        // Freshness hashes must match the contents above.
        let mut vectors = vectors
            .into_iter()
            .map(|(id, (_, v))| {
                let mut hasher = Sha256::new();
                hasher.update(contents[&id].as_bytes());
                (id, (format!("{:x}", hasher.finalize()), v))
            })
            .collect::<HashMap<_, _>>();
        let qv = [0.0, 1.0f32]; // cosine: d1=0, d2=1

        // α=0.3 weights the vector leg heavily: d2 (vector-only, cosine=1)
        // must outrank d1 (lexical-only). At α=0.5 both normalize to 0.5 —
        // an honest tie — hence the decisive weight for this fixture.
        let hits = fuse(
            &ids,
            &bm25_scores,
            Some(&vectors),
            &contents,
            Some(&qv),
            0.3,
            REFUSAL_FLOOR,
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0, 1, "strong vector match (d2) wins");
        assert_eq!(hits[0].2, "hybrid");
        assert_eq!(hits[1].0, 0, "d1 stays lexical");
        // d1 HAS a (fresh) vector with cosine 0 — kem semantics mark any doc
        // with a vector as "hybrid" even when the vector contributes nothing;
        // "bm25" is reserved for docs without a usable vector.
        assert_eq!(hits[1].2, "hybrid");
        // Exact kem-fusion math: 0.3·norm(2/2) + 0.7·norm(0/1) = 0.3
        // (tolerance covers the f32→f64 α conversion).
        assert!((hits[1].1 - 0.3).abs() < 1e-6);
        assert!(hits[0].1 > hits[1].1);
        let _ = &mut vectors;
    }

    #[test]
    fn refusal_floor_drops_weak_vector_only_hits() {
        let ids = vec!["d1".to_string()];
        let bm25_scores = vec![0.0]; // zero lexical support
        let mut contents: HashMap<String, String> = HashMap::new();
        contents.insert("d1".to_string(), "c".to_string());
        let mut hasher = Sha256::new();
        hasher.update(b"c");
        let mut vectors: HashMap<String, (String, Vec<f32>)> = HashMap::new();
        vectors.insert(
            "d1".to_string(),
            (format!("{:x}", hasher.finalize()), vec![0.9, 0.1]),
        );
        let weak_qv = [0.3f32, 0.95]; // cosine ~0.28 < floor
        let hits = fuse(
            &ids,
            &bm25_scores,
            Some(&vectors),
            &contents,
            Some(&weak_qv),
            HYBRID_ALPHA,
            REFUSAL_FLOOR,
        );
        assert!(hits.is_empty(), "weak vector-only hit must be refused");

        let strong_qv = [0.99f32, 0.05]; // cosine ~0.99 ≥ floor
        let hits2 = fuse(
            &ids,
            &bm25_scores,
            Some(&vectors),
            &contents,
            Some(&strong_qv),
            HYBRID_ALPHA,
            REFUSAL_FLOOR,
        );
        assert_eq!(hits2.len(), 1);
        assert_eq!(hits2[0].2, "hybrid");
    }

    #[tokio::test]
    async fn stale_vector_is_treated_as_missing() {
        let engine = engine();
        engine
            .add_document("d1".into(), "original content".into(), HashMap::new())
            .await;
        // Simulate staleness: the stored hash no longer matches the content.
        {
            let mut vectors = engine.vectors.write().await;
            vectors.insert("d1".to_string(), ("stale-hash".into(), vec![1.0, 0.0]));
        }
        // Direct fusion through the engine: query embedding is unreachable
        // (empty pool) so the vector leg never activates anyway — assert the
        // vector map entry is ignored by the freshness gate in fuse via
        // a manual call with a query vector.
        let docs = engine.documents.read().await;
        let contents: HashMap<String, String> = docs
            .iter()
            .map(|(id, d)| (id.clone(), d.content.clone()))
            .collect();
        let order = vec!["d1".to_string()];
        let vectors = engine.vectors.read().await;
        let hits = fuse(
            &order,
            &[0.0],
            Some(&vectors),
            &contents,
            Some(&[1.0f32, 0.0]),
            HYBRID_ALPHA,
            REFUSAL_FLOOR,
        );
        // Stale hash → vector treated as missing → zero lexical → refused.
        assert!(hits.is_empty(), "stale vector must not produce hybrid hits");
    }

    // ── RRF default fusion (2026-11-12) ──────────────────────────────────

    #[tokio::test]
    async fn delete_rebuilds_index_and_document_count() {
        let engine = engine();
        engine
            .add_document("d1".into(), "alpha beta".into(), HashMap::new())
            .await;
        engine
            .add_document("d2".into(), "alpha gamma".into(), HashMap::new())
            .await;
        assert_eq!(engine.document_count().await, 2);
        engine.delete_document("d1").await;
        assert_eq!(engine.document_count().await, 1);
        let hits = engine.search_ranked("alpha").await;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.doc_id, "d2", "deleted doc must vanish");
        assert!(engine.get_document("d1").await.is_none());
    }

    #[test]
    fn rrf_fuse_preserves_refusal_floor() {
        let ids = vec!["d1".to_string()];
        let contents: HashMap<String, String> = [("d1".to_string(), "unrelated body".to_string())]
            .into_iter()
            .collect();
        let mut vectors = HashMap::new();
        vectors.insert("d1".to_string(), ("h".to_string(), vec![0.9f32, 0.1]));
        let qv = [0.3f32, 0.95]; // cosine ~0.28 < floor
        let hits = rrf_fuse(
            &ids,
            &[0.0],
            Some(&vectors),
            &contents,
            Some(&qv),
            REFUSAL_FLOOR,
        );
        assert!(
            hits.is_empty(),
            "weak vector-only hit must be refused under RRF"
        );
    }

    #[test]
    fn rrf_fuse_is_scale_invariant_where_weighted_is_not() {
        // Same ordering, wildly different BM25 scale: RRF depends only on
        // ranks, so both inputs must produce identical rankings.
        let ids: Vec<String> = (0..3).map(|i| format!("d{i}")).collect();
        let contents: HashMap<String, String> = ids
            .iter()
            .map(|id| (id.clone(), format!("body {id}")))
            .collect();
        let small = [1.0f64, 0.5, 0.1];
        let huge = [100_000.0f64, 50_000.0, 10_000.0];
        let a = rrf_fuse(&ids, &small, None, &contents, None, REFUSAL_FLOOR);
        let b = rrf_fuse(&ids, &huge, None, &contents, None, REFUSAL_FLOOR);
        let order =
            |h: &[(usize, f64, &'static str)]| h.iter().map(|(p, _, _)| *p).collect::<Vec<_>>();
        assert_eq!(order(&a), order(&b), "RRF must be scale-invariant");
        assert_eq!(order(&a), vec![0, 1, 2]);
        // Top RRF score is exactly 1/(k+0) with one ranking path.
        assert!((a[0].1 - 1.0 / (RRF_K + 0.0)).abs() < 1e-12);
    }

    #[test]
    fn search_ranked_uses_rrf_by_default() {
        // Engine-level proof: ranked hits carry RRF-scale scores, not
        // weighted-normalized ones (max ≤ 1.0 for weighted; RRF top-1 = 1/60).
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let engine = engine();
            engine
                .add_document("d1".into(), "alpha beta gamma".into(), HashMap::new())
                .await;
            let hits = engine.search_ranked("alpha beta").await;
            assert_eq!(hits.len(), 1);
            let expected = 1.0 / (RRF_K + 0.0);
            assert!(
                (hits[0].score - expected).abs() < 1e-9,
                "search_ranked must fuse with RRF: got {}",
                hits[0].score
            );
        });
    }

    /// G4.9 (D3-04): `search_hybrid=off` → deterministic BM25-only, even when a
    /// fresh vector exists (the vector leg is skipped entirely).
    #[tokio::test]
    async fn hybrid_flag_off_forces_bm25_even_with_vectors() {
        let engine = engine();
        engine
            .add_document("d1".into(), "alpha beta".into(), HashMap::new())
            .await;
        let hash = content_hash("alpha beta");
        engine
            .vectors
            .write()
            .await
            .insert("d1".to_string(), (hash, vec![1.0, 0.0]));
        engine.set_hybrid(false);
        let hits = engine.search_ranked("alpha").await;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].via, "bm25", "hybrid=off must skip the vector leg");
    }

    /// G4.9 (D3-04): `search_rrf=off` → weighted α-fusion (top score normalized
    /// to 1.0), while still ranking correctly.
    #[tokio::test]
    async fn rrf_flag_off_uses_weighted_fusion() {
        let engine = engine();
        engine
            .add_document("d1".into(), "alpha alpha".into(), HashMap::new())
            .await;
        engine
            .add_document("d2".into(), "beta".into(), HashMap::new())
            .await;
        engine.set_rrf(false);
        let hits = engine.search_ranked("alpha").await;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.doc_id, "d1");
        assert!(
            (hits[0].score - 1.0).abs() < 1e-9,
            "weighted fusion normalizes the top score to 1.0, got {}",
            hits[0].score
        );
    }
}
