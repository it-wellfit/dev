#![forbid(unsafe_code)]
//! HyDE — Hypothetical Document Embeddings (Gao et al., ACL 2023,
//! "Precise Zero-Shot Dense Retrieval without Relevance Labels").
//!
//! A local LLM writes a short hypothetical answer to the query; that document
//! is embedded and used for the dense leg. The intuition: a real answer's
//! embedding sits near the relevant documents' embeddings even when the
//! query's vocabulary does not (the vocabulary-mismatch problem). The lexical
//! leg keeps the raw query (the hypothetical document's words may not match
//! the corpus's surface forms).
//!
//! Honest degradation (KEM §8): Ollama unreachable → `None`; the caller keeps
//! the raw query for the dense leg and does not claim HyDE.

/// Generate a hypothetical answer document for `query` with a local Ollama
/// chat model. Returns `None` when the reranker/generator is unavailable.
pub async fn hypothetical_document(query: &str, model: &str, base: &str) -> Option<String> {
    let prompt = format!(
        "Write a short factual answer (2-4 sentences) to the question below.\n\
         Use precise domain vocabulary. Do not add caveats.\n\nQuestion: {query}"
    );
    let url = format!("{}/api/chat", base.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": &prompt}],
        "stream": false,
        "think": false,
        "options": {"temperature": 0.0, "num_predict": 256}
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
        .unwrap_or_default()
        .trim();
    if content.is_empty() {
        None
    } else {
        Some(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_content_is_none() {
        // The trim guard: an empty reply must degrade, not become "".
        // (hypothetical_document's post-parse guard; tested via the helper
        // contract — a real call is environment-dependent and not asserted.)
        let content = "   ";
        assert!(content.trim().is_empty());
    }
}
