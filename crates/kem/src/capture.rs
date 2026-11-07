#![forbid(unsafe_code)]
//! M1 — Capture: immutable staging of sources under `raw/`.
//!
//! A raw file is written exactly once; re-capture with the same name is
//! refused (immutability), and every capture is hashed into the ledger so
//! later `sync` can detect source drift.

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::KemError;

/// A staged immutable source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedSource {
    /// Path relative to the knowledge root, e.g. `raw/ekpi/2026-11-07-x.md`.
    pub rel_path: String,
    /// SHA-256 of the content at capture time (hex).
    pub sha256: String,
    pub bytes: u64,
    pub captured_at: DateTime<Utc>,
    /// True when this call wrote the file; false for an idempotent re-run.
    pub created: bool,
}

pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Stage `content` under `raw/<topic>/<YYYY-MM-DD>-<name>.<ext>`.
/// Idempotent: an existing file with identical content is accepted as
/// already-captured; an existing file with different content is an
/// immutability violation (raw files never change once captured).
pub fn capture_source(
    root: &Path,
    topic: &str,
    name: &str,
    ext: &str,
    content: &str,
    now: DateTime<Utc>,
) -> Result<CapturedSource, KemError> {
    if topic.trim().is_empty() || name.trim().is_empty() {
        return Err(KemError::InvalidInput(
            "topic and name must be non-empty".into(),
        ));
    }
    // Sanitize path components (no traversal, no separators inside parts).
    for part in [topic, name] {
        if part.contains(['/', '\\', ':']) || part.contains("..") {
            return Err(KemError::InvalidInput(format!("illegal path part: {part}")));
        }
    }
    let date = Local::now().format("%Y-%m-%d"); // filename date uses local day
    let _ = now; // ledger timestamp stays UTC
    let rel = format!("raw/{topic}/{date}-{name}.{ext}");
    let abs = root.join(&rel);
    let captured = CapturedSource {
        rel_path: rel,
        sha256: sha256_hex(content.as_bytes()),
        bytes: u64::try_from(content.len()).unwrap_or(u64::MAX),
        captured_at: now,
        created: false,
    };
    if abs.exists() {
        let existing = std::fs::read(&abs)?;
        if sha256_hex(&existing) == captured.sha256 {
            return Ok(captured); // idempotent re-run
        }
        return Err(KemError::InvalidInput(format!(
            "raw source already captured with different content (immutable): {}",
            captured.rel_path
        )));
    }
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&abs, content)?;
    Ok(CapturedSource {
        created: true,
        ..captured
    })
}

/// Verify a captured source is unchanged (hash match) — used by `sync`.
pub fn verify_source(root: &Path, src: &CapturedSource) -> Result<bool, KemError> {
    let content = std::fs::read_to_string(root.join(&src.rel_path))?;
    Ok(sha256_hex(content.as_bytes()) == src.sha256)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_then_verify_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let src = capture_source(
            dir.path(),
            "ekpi",
            "phase-gates-review",
            "md",
            "# notes\nPhase A not approved\n",
            Utc::now(),
        )
        .unwrap();
        assert!(src.rel_path.starts_with("raw/ekpi/"));
        assert_eq!(src.sha256.len(), 64);
        assert!(verify_source(dir.path(), &src).unwrap());
    }

    #[test]
    fn overwrite_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        capture_source(dir.path(), "t", "n", "md", "a", Utc::now()).unwrap();
        // Idempotent: same content → Ok with created=false.
        let again = capture_source(dir.path(), "t", "n", "md", "a", Utc::now()).unwrap();
        assert!(!again.created);
        // Different content → immutability violation.
        let err = capture_source(dir.path(), "t", "n", "md", "b", Utc::now()).unwrap_err();
        assert!(matches!(err, KemError::InvalidInput(_)));
    }

    #[test]
    fn traversal_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let err = capture_source(dir.path(), "../evil", "n", "md", "a", Utc::now()).unwrap_err();
        assert!(matches!(err, KemError::InvalidInput(_)));
    }

    #[test]
    fn drift_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let src = capture_source(dir.path(), "t", "n", "md", "original", Utc::now()).unwrap();
        std::fs::write(dir.path().join(&src.rel_path), "changed").unwrap();
        assert!(!verify_source(dir.path(), &src).unwrap());
    }
}
