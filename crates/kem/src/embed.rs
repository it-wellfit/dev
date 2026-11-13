#![forbid(unsafe_code)]
//! M5b — Embeddings: free keyless embedder pool + persistent vector index.
//!
//! Pool (2026-11-07 live-verified): OVH bge-m3 (batched, 1024-dim, keyless)
//! → Ollama native /api/embeddings fallback. All-unreachable → honest
//! Err (caller degrades to BM25-only, never fabricates vectors).
//!
//! Freshness (KEM doc §8 derived-artifact rule): every vector entry carries
//! the SHA-256 of the claim body it was built from; a mismatched hash means
//! the vector is stale and is treated as missing.

use crate::capture::sha256_hex;
use crate::evolve::EndpointKind;
use crate::schema::{Claim, KemError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One embedder pool entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedEntry {
    pub label: String,
    pub base: String,
    pub kind: EndpointKind,
    pub model: String,
}

/// Default pool: OVH bge-m3 (batched) → Ollama local.
pub fn default_pool() -> Vec<EmbedEntry> {
    vec![
        EmbedEntry {
            label: "ovh-bge-m3".into(),
            base: "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1".into(),
            kind: EndpointKind::OpenAi,
            model: "bge-m3".into(),
        },
        EmbedEntry {
            label: "local-ollama".into(),
            base: "http://127.0.0.1:11434".into(),
            kind: EndpointKind::OllamaNative,
            model: "qwen3.5:4b".into(),
        },
    ]
}

/// Cosine similarity; 0.0 when either vector has zero norm.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (dot, na, nb) = a
        .iter()
        .zip(b.iter())
        .fold((0.0f32, 0.0f32, 0.0f32), |(d, xa, xb), (x, y)| {
            (d + x * y, xa + x * x, xb + y * y)
        });
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Embed all texts via the pool (first success wins). Returns one vector per
/// input, in input order. All pools failing → Err with per-pool reasons.
pub struct EmbedResult {
    pub vecs: Vec<Vec<f32>>,
    /// Label + model that actually produced the vectors (provenance).
    pub used_label: String,
    pub used_model: String,
}

pub async fn embed_batch(pool: &[EmbedEntry], texts: &[String]) -> Result<EmbedResult, String> {
    let mut errors: Vec<String> = Vec::new();
    for entry in pool {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("{}: client build failed: {e}", entry.label));
                continue;
            }
        };
        match entry.kind {
            EndpointKind::OpenAi => {
                let url = format!("{}/embeddings", entry.base);
                let body = serde_json::json!({ "model": entry.model, "input": texts });
                match client.post(&url).json(&body).send().await {
                    Err(e) => {
                        errors.push(format!("{}[{}]: transport: {e}", entry.label, entry.model))
                    }
                    Ok(r) => {
                        let status = r.status();
                        if !status.is_success() {
                            errors.push(format!("{}[{}]: HTTP {status}", entry.label, entry.model));
                            continue;
                        }
                        let parsed: Result<serde_json::Value, _> = r.json().await;
                        let Some(v) = parsed.ok() else {
                            errors.push(format!(
                                "{}[{}]: unparseable body",
                                entry.label, entry.model
                            ));
                            continue;
                        };
                        let rows = match v.get("data").and_then(|d| d.as_array()) {
                            Some(a) => a,
                            None => {
                                errors.push(format!(
                                    "{}[{}]: no data array",
                                    entry.label, entry.model
                                ));
                                continue;
                            }
                        };
                        if rows.len() != texts.len() {
                            errors.push(format!(
                                "{}[{}]: expected {} vectors, got {}",
                                entry.label,
                                entry.model,
                                texts.len(),
                                rows.len()
                            ));
                            continue;
                        }
                        let vecs: Vec<Vec<f32>> = rows
                            .iter()
                            .map(|row| {
                                row.get("embedding")
                                    .and_then(|e| e.as_array())
                                    .map(|a| {
                                        a.iter()
                                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                                            .collect()
                                    })
                                    .unwrap_or_default()
                            })
                            .collect();
                        if vecs.iter().any(|v| v.is_empty()) {
                            errors.push(format!(
                                "{}[{}]: empty vector in batch",
                                entry.label, entry.model
                            ));
                            continue;
                        }
                        return Ok(EmbedResult {
                            vecs,
                            used_label: entry.label.clone(),
                            used_model: entry.model.clone(),
                        });
                    }
                }
            }
            EndpointKind::OllamaNative => {
                let url = format!("{}/api/embeddings", entry.base);
                let client_ref = &client;
                let mut vecs: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
                let mut ok = true;
                for t in texts {
                    let body = serde_json::json!({ "model": entry.model, "prompt": t });
                    match client_ref.post(&url).json(&body).send().await {
                        Err(e) => {
                            errors
                                .push(format!("{}[{}]: transport: {e}", entry.label, entry.model));
                            ok = false;
                            break;
                        }
                        Ok(r) => {
                            let status = r.status();
                            if !status.is_success() {
                                errors.push(format!(
                                    "{}[{}]: HTTP {status}",
                                    entry.label, entry.model
                                ));
                                ok = false;
                                break;
                            }
                            let parsed: Result<serde_json::Value, _> = r.json().await;
                            match parsed.ok().and_then(|v| v.get("embedding").cloned()) {
                                Some(e) => {
                                    let vec: Vec<f32> = e
                                        .as_array()
                                        .map(|a| {
                                            a.iter()
                                                .filter_map(|x| x.as_f64().map(|f| f as f32))
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                    if vec.is_empty() {
                                        errors.push(format!(
                                            "{}[{}]: empty embedding",
                                            entry.label, entry.model
                                        ));
                                        ok = false;
                                        break;
                                    }
                                    vecs.push(vec);
                                }
                                None => {
                                    errors.push(format!(
                                        "{}[{}]: no embedding field",
                                        entry.label, entry.model
                                    ));
                                    ok = false;
                                    break;
                                }
                            }
                        }
                    }
                }
                if ok && vecs.len() == texts.len() {
                    return Ok(EmbedResult {
                        vecs,
                        used_label: entry.label.clone(),
                        used_model: entry.model.clone(),
                    });
                }
            }
        }
    }
    Err(if errors.is_empty() {
        "empty embedder pool".to_string()
    } else {
        errors.join("; ")
    })
}

/// One persisted vector with its source-body hash (staleness guard).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub body_sha: String,
    pub vec: Vec<f32>,
}

/// Persisted vector index (.kem/vectors.json — derived, rebuildable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexFile {
    pub model: String,
    pub dim: usize,
    pub entries: Vec<VectorEntry>,
}

pub fn load_vectors(path: &Path) -> Result<Option<VectorIndexFile>, KemError> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    let stripped = crate::filing::strip_bom(&text);
    Ok(Some(serde_json::from_str(stripped)?))
}

pub fn save_vectors(path: &Path, index: &VectorIndexFile) -> Result<(), KemError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(index)?)?;
    Ok(())
}

/// Build (claim_id → vector) lookup for still-fresh vectors: an entry whose
/// body_sha no longer matches the current claim body is stale and skipped
/// (derived artifacts must never outlive their source silently).
pub fn fresh_lookup<'a>(
    index: &'a VectorIndexFile,
    claims: &'a [Claim],
) -> std::collections::HashMap<&'a str, &'a [f32]> {
    use std::collections::HashMap;
    let mut m: HashMap<&str, &[f32]> = HashMap::new();
    for c in claims {
        let sha = sha256_hex(c.body.as_bytes());
        if let Some(v) = index.entries.iter().find(|e| e.id == c.id)
            && v.body_sha == sha
            && !v.vec.is_empty()
        {
            m.insert(c.id.as_str(), v.vec.as_slice());
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_basics() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6);
        assert!((cosine(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-6);
        assert_eq!(cosine(&[], &[1.0]), 0.0);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 0.0]), 0.0);
    }

    #[test]
    fn body_hash_is_stable_and_binds_body() {
        let h1 = sha256_hex("body text".as_bytes());
        let h2 = sha256_hex("body text".as_bytes());
        assert_eq!(h1, h2);
        assert_ne!(h1, sha256_hex("other".as_bytes()));
    }
}
