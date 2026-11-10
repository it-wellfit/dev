//! BM25 lexical ranking over the platform corpus (PHASE-R, RAG-S1).
//!
//! Pure std-only math — zero new dependencies (the Tantivy lesson). This is
//! the lexical leg of the hybrid retriever on `/api/search`; semantics mirror
//! the proven kem implementation (`crates/kem/src/search.rs`: K1/B constants,
//! Robertson IDF) with one deliberate improvement: the tokenizer emits single
//! CJK characters as tokens, where kem's `w.len() > 1` filter would drop
//! them — Chinese queries must match single-character terms.
//!
//! Scores are ranking signals, never amounts (INV-01 governs money only).

/// BM25 ranking constants (Okapi defaults; fixed, not tuned per request).
pub const K1: f64 = 1.2;
pub const B: f64 = 0.75;

/// One indexed document: term frequencies + length in tokens.
#[derive(Debug, Default, Clone)]
pub struct Bm25Document {
    term_freq: std::collections::HashMap<String, f64>,
    doc_len: usize,
}

/// BM25 index over a corpus of raw texts. Grows through `add_text`; the whole
/// index can be rebuilt cheaply on delete (documented in-memory tradeoff).
#[derive(Debug, Default)]
pub struct BM25Index {
    docs: Vec<Bm25Document>,
    /// term → number of documents containing it
    df: std::collections::HashMap<String, f64>,
    total_len: usize,
    n_docs: f64,
}

/// Tokenize text into lowercased tokens. ASCII/alphanumeric runs become word
/// tokens; each non-ASCII (CJK etc.) character becomes its own token so
/// Chinese queries match without a dictionary segmenter (documented
/// tradeoff, see module docs).
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            if ch.is_ascii() {
                buf.push(ch);
            } else {
                if !buf.is_empty() {
                    tokens.push(buf.to_lowercase());
                    buf.clear();
                }
                tokens.push(ch.to_string());
            }
        } else if !buf.is_empty() {
            tokens.push(buf.to_lowercase());
            buf.clear();
        }
    }
    if !buf.is_empty() {
        tokens.push(buf.to_lowercase());
    }
    tokens
}

impl BM25Index {
    /// Build from raw document texts.
    pub fn from_texts(texts: impl IntoIterator<Item = String>) -> Self {
        let mut index = Self::default();
        for text in texts {
            index.add_text(&text);
        }
        index
    }

    /// Add one raw document; maintains df and length statistics.
    pub fn add_text(&mut self, text: &str) {
        let tokens = tokenize(text);
        let mut tf: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for token in &tokens {
            *tf.entry(token.clone()).or_insert(0.0) += 1.0;
        }
        for term in tf.keys() {
            *self.df.entry(term.clone()).or_insert(0.0) += 1.0;
        }
        self.total_len += tokens.len();
        self.n_docs += 1.0;
        self.docs.push(Bm25Document {
            term_freq: tf,
            doc_len: tokens.len(),
        });
    }

    /// Number of documents.
    pub fn doc_count(&self) -> usize {
        self.docs.len()
    }

    fn avg_len(&self) -> f64 {
        if self.n_docs > 0.0 {
            self.total_len as f64 / self.n_docs
        } else {
            1.0
        }
    }

    /// Robertson IDF, identical to kem (`crates/kem/src/search.rs:128`): the
    /// +1.0 inside the logarithm keeps the value positive for every df ≤ n,
    /// so a term present in all documents still ranks by term frequency
    /// (no floor needed — a floor would zero out ubiquitous terms).
    fn idf(&self, term: &str) -> f64 {
        let df = self.df.get(term).copied().unwrap_or(0.0);
        if df <= 0.0 {
            return 0.0;
        }
        let n = self.n_docs;
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// BM25 score of one document (by position) against tokenized query terms.
    fn score_doc(&self, pos: usize, query_terms: &[String]) -> f64 {
        let doc = &self.docs[pos];
        let avglen = self.avg_len();
        let mut score = 0.0;
        for term in query_terms {
            let tf = doc.term_freq.get(term).copied().unwrap_or(0.0);
            if tf <= 0.0 || !self.df.contains_key(term) {
                continue;
            }
            let idf = self.idf(term);
            let denom = tf + K1 * (1.0 - B + B * (doc.doc_len as f64 / avglen));
            score += idf * (tf * (K1 + 1.0) / denom);
        }
        score
    }

    /// Score one document (by position) against tokenized query terms —
    /// used by the fusion engine (terms tokenized once per query).
    pub fn score_at(&self, pos: usize, query_terms: &[String]) -> f64 {
        self.score_doc(pos, query_terms)
    }

    /// Score every document against the query; returns `(position, score)`
    /// for documents with positive score, sorted by score descending.
    pub fn scores_all(&self, query: &str) -> Vec<(usize, f64)> {
        let terms = tokenize(query);
        let mut scored: Vec<(usize, f64)> = (0..self.docs.len())
            .map(|pos| (pos, self.score_doc(pos, &terms)))
            .filter(|(_, s)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_handles_mixed_ascii_and_cjk() {
        let tokens = tokenize("Flaps ERP 傳票 STRESS");
        assert!(tokens.contains(&"flaps".to_string()));
        assert!(tokens.contains(&"erp".to_string()));
        assert!(tokens.contains(&"傳".to_string()), "CJK single-char token");
        assert!(tokens.contains(&"票".to_string()));
        assert!(tokens.contains(&"stress".to_string()));
    }

    #[test]
    fn empty_and_punctuation_only_texts_tokenize_to_nothing() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("！？。、").is_empty());
    }

    #[test]
    fn rarer_terms_score_higher_than_common_terms() {
        // "refund" appears in one doc; "the" appears everywhere → IDF wins.
        let index = BM25Index::from_texts(vec![
            "the refund policy document".to_string(),
            "the the the the the the the other text".to_string(),
            "the policy and the policy and the policy".to_string(),
        ]);
        let scores = index.scores_all("refund");
        assert_eq!(scores.len(), 1, "only the refund doc matches");
        let rare = scores[0].1;

        let common = index.scores_all("the");
        assert!(
            common.iter().all(|(_, s)| *s < rare),
            "rare term must outrank the ubiquitous term: rare={rare}, common={common:?}"
        );
    }

    #[test]
    fn higher_term_frequency_ranks_higher() {
        let index = BM25Index::from_texts(vec![
            "ledger ledger ledger".to_string(),
            "ledger".to_string(),
        ]);
        let scores = index.scores_all("ledger");
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].0, 0, "higher tf ranks first");
        assert!(scores[0].1 > scores[1].1);
    }

    #[test]
    fn unknown_term_scores_zero_and_empty_index_is_safe() {
        let index = BM25Index::from_texts(vec!["hello world".to_string()]);
        assert!(index.scores_all("nonexistent").is_empty());
        let empty = BM25Index::default();
        assert!(empty.scores_all("anything").is_empty());
        assert_eq!(empty.doc_count(), 0);
    }
}
