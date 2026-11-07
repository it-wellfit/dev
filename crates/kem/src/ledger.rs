#![forbid(unsafe_code)]
//! Append-only decision ledger with a SHA-256 hash chain.
//!
//! Every vetting/filing/override/supersede/retire decision is recorded; the
//! chain makes silent tampering detectable (verify recomputes every link).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, Write};
use std::path::Path;

use crate::KemError;

pub const GENESIS_PREV_HASH: &str = "GENESIS";

/// The kind of decision recorded (KEM doc §4.3 / M7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerAction {
    /// L0: source staged immutably under raw/.
    Capture,
    /// T1/T2/T3 verdict on a claim.
    Vet,
    /// Accepted claim filed into the store.
    File,
    /// Human/agent override of a verdict (always recorded, never silent).
    Override,
    /// Structural replacement of one claim by another (Track A).
    Supersede,
    /// Terminal soft delete.
    Retire,
    /// Re-verification of file-linked evidence against current files.
    Sync,
    /// Review-queue disposition (accept/reject/supersede/defer).
    Review,
}

/// One ledger record. `hash` = SHA-256 over the canonical JSON of all fields
/// except `hash` itself, chained to `prev_hash`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub seq: u64,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub action: LedgerAction,
    /// Claim this entry concerns (`kem-*` meta entries use the empty string).
    pub claim_id: String,
    pub detail: serde_json::Value,
    pub prev_hash: String,
    pub hash: String,
}

impl LedgerEntry {
    pub fn compute_hash(&self) -> String {
        let mut partial = self.clone();
        partial.prev_hash = self.prev_hash.clone();
        partial.hash = String::new();
        let bytes = serde_json::to_vec(&partial).unwrap_or_default();
        let digest = Sha256::digest(&bytes);
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        hex
    }
}

/// Append one entry to the JSONL ledger, chaining to the current head.
pub fn append(
    ledger_path: &Path,
    actor: &str,
    action: LedgerAction,
    claim_id: &str,
    detail: serde_json::Value,
    now: DateTime<Utc>,
) -> Result<LedgerEntry, KemError> {
    let prev_hash = head_hash(ledger_path)?;
    let seq = head_seq(ledger_path)?.saturating_add(1);
    let mut entry = LedgerEntry {
        seq,
        ts: now,
        actor: actor.to_string(),
        action,
        claim_id: claim_id.to_string(),
        detail,
        prev_hash,
        hash: String::new(),
    };
    entry.hash = entry.compute_hash();

    let line = serde_json::to_string(&entry)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(entry)
}

/// Recompute the whole chain; returns the number of verified entries.
pub fn verify(ledger_path: &Path) -> Result<usize, KemError> {
    let file = std::fs::File::open(ledger_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => KemError::NotFound("ledger".into()),
        _ => KemError::Io(e),
    })?;
    let reader = std::io::BufReader::new(file);
    let mut prev = GENESIS_PREV_HASH.to_string();
    let mut seq: u64 = 0;
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: LedgerEntry = serde_json::from_str(&line)?;
        seq = entry.seq;
        if entry.prev_hash != prev {
            return Err(KemError::ChainBroken {
                at: i + 1,
                reason: format!(
                    "prev_hash mismatch: expected {prev}, got {}",
                    entry.prev_hash
                ),
            });
        }
        let expected = entry.compute_hash();
        if entry.hash != expected {
            return Err(KemError::ChainBroken {
                at: i + 1,
                reason: "entry hash mismatch (tampering detected)".into(),
            });
        }
        prev = entry.hash.clone();
    }
    Ok(usize::try_from(seq).unwrap_or(0))
}

/// Read all entries (bounded by file size; adequate at workspace scale).
pub fn read_all(ledger_path: &Path) -> Result<Vec<LedgerEntry>, KemError> {
    let file = std::fs::File::open(ledger_path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => KemError::NotFound("ledger".into()),
        _ => KemError::Io(e),
    })?;
    let reader = std::io::BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}

fn head_hash(ledger_path: &Path) -> Result<String, KemError> {
    // A missing file is an empty ledger, not an error (genesis semantics).
    if !ledger_path.exists() {
        return Ok(GENESIS_PREV_HASH.to_string());
    }
    Ok(read_all(ledger_path)?
        .last()
        .map(|e| e.hash.clone())
        .unwrap_or_else(|| GENESIS_PREV_HASH.to_string()))
}

fn head_seq(ledger_path: &Path) -> Result<u64, KemError> {
    if !ledger_path.exists() {
        return Ok(0);
    }
    Ok(read_all(ledger_path)?.last().map(|e| e.seq).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn tmp_ledger() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ledger.jsonl");
        (dir, path)
    }

    fn ts(n: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 7, 0, 0, n).unwrap()
    }

    #[test]
    fn genesis_head_is_genesis_hash() {
        let (_d, path) = tmp_ledger();
        assert_eq!(head_hash(&path).unwrap(), GENESIS_PREV_HASH);
        assert_eq!(head_seq(&path).unwrap(), 0);
    }

    #[test]
    fn append_chains_and_verifies() {
        let (_d, path) = tmp_ledger();
        let e1 = append(
            &path,
            "kem-t1",
            LedgerAction::Vet,
            "claim-20261107-001",
            serde_json::json!({"verdict": "accept"}),
            ts(0),
        )
        .unwrap();
        let e2 = append(
            &path,
            "kem-t1",
            LedgerAction::File,
            "claim-20261107-001",
            serde_json::json!({"page": "topics/fact"}),
            ts(1),
        )
        .unwrap();
        assert_eq!(e1.seq, 1);
        assert_eq!(e2.seq, 2);
        assert_eq!(e2.prev_hash, e1.hash);
        assert_eq!(verify(&path).unwrap(), 2);
    }

    #[test]
    fn tampering_is_detected() {
        let (_d, path) = tmp_ledger();
        append(
            &path,
            "kem-t1",
            LedgerAction::Vet,
            "claim-20261107-001",
            serde_json::json!({"verdict": "accept"}),
            ts(0),
        )
        .unwrap();
        append(
            &path,
            "kem-t1",
            LedgerAction::File,
            "claim-20261107-001",
            serde_json::json!({"page": "x"}),
            ts(1),
        )
        .unwrap();
        // rewrite the first line with a forged verdict
        let content = std::fs::read_to_string(&path).unwrap();
        let forged = content.replace("accept", "REJECT");
        std::fs::write(&path, forged).unwrap();
        let err = verify(&path).unwrap_err();
        assert!(matches!(err, KemError::ChainBroken { .. }));
    }

    #[test]
    fn missing_ledger_is_not_found() {
        let (_d, path) = tmp_ledger();
        let err = verify(&path).unwrap_err();
        assert!(matches!(err, KemError::NotFound(_)));
    }
}
