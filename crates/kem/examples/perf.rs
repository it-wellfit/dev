#![forbid(unsafe_code)]
//! Retrieval performance evidence (2026-11-12): measures per-stage latency
//! of the retrieval pipeline on the seed corpus. Not a gate (timings are
//! machine-dependent); it documents the cost of each SOTA stage so the
//! quality/latency trade-off is a measured fact, not an assumption.
//!
//! Run: `cargo run --release -p kem --example perf`
//! Stages: BM25-only → RRF → RRF+MMR → RRF+PRF (each over N iterations).

use std::time::Instant;

fn main() {
    let claims = kem::seed::seed_claims();
    let index = kem::search::Index::build(&claims);
    let filtered = kem::search::visible_at(&claims, None);
    let queries = [
        "phase gate blocked approved",
        "zero javascript invariant",
        "post redirect get mutation",
        "refusal floor semantic adjacency",
        "ledger hash chain append only",
    ];
    const ITERS: usize = 200;

    // Stage 1: BM25 only.
    let t = Instant::now();
    for _ in 0..ITERS {
        for q in &queries {
            let bm25 = index.scores_all(&filtered, q);
            let hits = kem::search::rrf_fuse(
                &filtered,
                &bm25,
                None,
                None,
                5,
                kem::search::REFUSAL_FLOOR,
                kem::search::RRF_K,
            );
            std::hint::black_box(&hits);
        }
    }
    let bm25_us = t.elapsed().as_secs_f64() * 1e6 / (ITERS * queries.len()) as f64;
    println!("BM25+RRF        : {:>8.1} µs/query", bm25_us);

    // Stage 2: RRF + MMR.
    let t = Instant::now();
    for _ in 0..ITERS {
        for q in &queries {
            let bm25 = index.scores_all(&filtered, q);
            let fused = kem::search::rrf_fuse(
                &filtered,
                &bm25,
                None,
                None,
                5,
                kem::search::REFUSAL_FLOOR,
                kem::search::RRF_K,
            );
            let hits = kem::search::mmr_diversify(
                &fused,
                &filtered,
                None,
                kem::search::MMR_DEFAULT_LAMBDA,
                5,
            );
            std::hint::black_box(&hits);
        }
    }
    let mmr_us = t.elapsed().as_secs_f64() * 1e6 / (ITERS * queries.len()) as f64;
    println!(
        "BM25+RRF+MMR    : {:>8.1} µs/query  (+{:.1}%)",
        mmr_us,
        (mmr_us / bm25_us - 1.0) * 100.0
    );

    // Stage 3: + PRF (expansion cost included).
    let t = Instant::now();
    for _ in 0..ITERS {
        for q in &queries {
            let terms = kem::search::prf_expand(&index, &filtered, q, 3, 8);
            let expanded = if terms.is_empty() {
                q.to_string()
            } else {
                format!("{q} {}", terms.join(" "))
            };
            let bm25 = index.scores_all(&filtered, &expanded);
            let hits = kem::search::rrf_fuse(
                &filtered,
                &bm25,
                None,
                None,
                5,
                kem::search::REFUSAL_FLOOR,
                kem::search::RRF_K,
            );
            std::hint::black_box(&hits);
        }
    }
    let prf_us = t.elapsed().as_secs_f64() * 1e6 / (ITERS * queries.len()) as f64;
    println!(
        "BM25+RRF+PRF    : {:>8.1} µs/query  (+{:.1}%)",
        prf_us,
        (prf_us / bm25_us - 1.0) * 100.0
    );

    println!();
    println!(
        "corpus: {} claims, {} queries, {} iterations (release, this machine)",
        claims.len(),
        queries.len(),
        ITERS
    );
    println!("note: rerank/HyDE incur an Ollama network call (100-2000ms) and are opt-in only.");
}
