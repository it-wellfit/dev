#![forbid(unsafe_code)]
//! Pointwise LLM reranking (2026-11-12) — the SOTA second stage for
//! retrieval quality. After fusion (RRF/weighted) and optional MMR produce a
//! candidate set, a local LLM scores each candidate's relevance to the query
//! in ONE batched call (pointwise scoring is markedly more reliable than
//! listwise ordering for small local models; the spec's T2 warning applies to
//! structured *ordering*, not per-item scores).
//!
//! Honest degradation (KEM §8): Ollama unreachable or parse failure returns
//! `None`; the caller keeps the fusion order and does NOT claim a rerank.

/// Score candidates 0-10 with a local Ollama chat model.
///
/// `candidates` is `(doc_id, text)`; returns `Some(scores)` with one
/// `(doc_id, score)` per input, or `None` when the reranker is unavailable.
pub async fn rerank_ollama(
    query: &str,
    candidates: &[(String, String)],
    model: &str,
    base: &str,
) -> Option<Vec<(String, f64)>> {
    if candidates.is_empty() {
        return Some(Vec::new());
    }
    let listing = candidates
        .iter()
        .enumerate()
        .map(|(i, (_, text))| format!("[{i}] {text}"))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "Query: {query}\n\nDocuments:\n{listing}\n\n\
         Score each document 0-10 for how well it answers the query (10 = exact\n\
         answer, 0 = unrelated). Reply with ONLY a JSON object:\n\
         {{\"scores\": [{{\"i\": <index>, \"s\": <0-10 integer>}}]}}"
    );
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "scores": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "i": {"type": "integer", "minimum": 0, "maximum": 63},
                        "s": {"type": "integer", "minimum": 0, "maximum": 10}
                    },
                    "required": ["i", "s"]
                }
            }
        },
        "required": ["scores"]
    });
    let url = format!("{}/api/chat", base.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": &prompt}],
        "stream": false,
        "think": false,
        "format": schema,
        "options": {"temperature": 0.0, "num_predict": 512}
    });
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .ok()?;
    let resp = client.post(&url).json(&body).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let parsed: serde_json::Value = resp.json().await.ok()?;
    let content = parsed
        .pointer("/message/content")
        .and_then(|c| c.as_str())
        .unwrap_or_default();
    parse_scores(content, candidates)
}

/// Parse the JSON reply into `(doc_id, score)` pairs. Missing indices fall
/// back to score 0 (honest: an unscored document is not asserted relevant).
fn parse_scores(content: &str, candidates: &[(String, String)]) -> Option<Vec<(String, f64)>> {
    let value: serde_json::Value = serde_json::from_str(content.trim()).ok()?;
    let arr = value.get("scores")?.as_array()?;
    let mut out = Vec::with_capacity(candidates.len());
    for (i, (id, _)) in candidates.iter().enumerate() {
        let score = arr
            .iter()
            .find(|s| s.get("i").and_then(|v| v.as_u64()) == Some(i as u64))
            .and_then(|s| s.get("s"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .clamp(0.0, 10.0);
        out.push((id.clone(), score));
    }
    Some(out)
}

/// Ollama endpoint used by the reranker (local, keyless).
pub const DEFAULT_RERANK_MODEL: &str = "qwen3.5:4b";
pub const DEFAULT_RERANK_BASE: &str = "http://127.0.0.1:11434";

/// Whether the reranker endpoint is reachable (fast probe; used to decide
/// whether to attempt a rerank at all).
pub async fn available(base: &str) -> bool {
    let url = format!("{}/api/tags", base.trim_end_matches('/'));
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    client
        .get(&url)
        .send()
        .await
        .is_ok_and(|r| r.status().is_success())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<(String, String)> {
        vec![
            ("c1".into(), "Phase A is blocked".into()),
            ("c2".into(), "Zero JavaScript invariant".into()),
            ("c3".into(), "Unrelated text".into()),
        ]
    }

    #[test]
    fn parse_scores_maps_indices_to_ids() {
        let content = r#"{"scores": [{"i": 0, "s": 9}, {"i": 1, "s": 3}, {"i": 2, "s": 0}]}"#;
        let parsed = parse_scores(content, &candidates()).expect("parse");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], ("c1".to_string(), 9.0));
        assert_eq!(parsed[1], ("c2".to_string(), 3.0));
        assert_eq!(parsed[2], ("c3".to_string(), 0.0));
    }

    #[test]
    fn parse_scores_missing_index_is_zero() {
        let content = r#"{"scores": [{"i": 0, "s": 7}]}"#;
        let parsed = parse_scores(content, &candidates()).expect("parse");
        // c2/c3 unscored → 0 (honest: not asserted relevant).
        assert_eq!(parsed[1], ("c2".to_string(), 0.0));
        assert_eq!(parsed[2], ("c3".to_string(), 0.0));
    }

    #[test]
    fn parse_scores_clamps_out_of_range() {
        let content = r#"{"scores": [{"i": 0, "s": 99}, {"i": 1, "s": -5}]}"#;
        let parsed = parse_scores(content, &candidates()).expect("parse");
        assert_eq!(parsed[0].1, 10.0);
        assert_eq!(parsed[1].1, 0.0);
    }

    #[test]
    fn parse_scores_rejects_garbage() {
        assert!(parse_scores("not json", &candidates()).is_none());
        assert!(parse_scores(r#"{"other": 1}"#, &candidates()).is_none());
    }
}
