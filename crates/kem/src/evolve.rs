#![forbid(unsafe_code)]
//! M6 — Evolution: sync (re-verify evidence), decay (confidence half-life),
//! supersede (structural replacement, Track A), and T2 self-consistency.
//!
//! Rules from KEM doc §4.4/§4.5:
//! - sync never silently deprecates: broken evidence becomes a review item
//! - decay only demotes; retired/deleted never happens implicitly
//! - supersede links both directions and keeps the old claim retrievable
//! - T2 (SCS) calls local Ollama only; unreachable → honest Review, never
//!   a fabricated score (UNKNOWN ≠ PASS rule)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::ledger::LedgerAction;
use crate::schema::{Claim, ClaimStatus, ClaimType, KemError, Track, validate_claim, known_ids_of};

/// Per-type confidence half-life in days (KEM doc §4.5: architecture
/// decisions fade slowly, transient facts fast; concepts/glossary no decay).
pub fn half_life_days(claim_type: ClaimType) -> Option<f64> {
    match claim_type {
        ClaimType::Concept | ClaimType::Glossary => None,
        ClaimType::Decision | ClaimType::Invariant => Some(180.0),
        ClaimType::Fact => Some(90.0),
        ClaimType::Gotcha => Some(60.0),
    }
}

/// Recompute confidence from evidence count + recency (ledger-aware).
/// `last_affirmed` = timestamp of the most recent successful sync/vet;
/// `None` means never re-verified (use recorded_at).
pub fn recompute_confidence(
    claim: &Claim,
    last_affirmed: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    let base = match claim.evidence.len() {
        0 => 0.0,
        1 => 0.8,
        2 => 0.9,
        _ => 0.95,
    };
    let anchor = last_affirmed.unwrap_or(claim.recorded_at);
    let age_days = (now - anchor).num_hours() as f64 / 24.0;
    match half_life_days(claim_type_of(claim)) {
        None => base.max(claim.confidence.min(base)),
        Some(hl) => {
            let decayed = base * 0.5f64.powf(age_days / hl);
            // Floor: demote but never zero — claims fade to "needs reaffirm".
            decayed.clamp(0.30, 1.0)
        }
    }
}

fn claim_type_of(c: &Claim) -> ClaimType {
    c.claim_type
}

/// Sync outcome for one claim.
#[derive(Debug Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncResult {
    pub claim_id: String,
    pub evidence_ok: usize,
    pub evidence_total: usize,
    /// Recomputed confidence (may equal the old one).
    pub confidence: f64,
    /// Reasons a claim needs review (broken quotes, etc.). Empty = healthy.
    pub issues: Vec<String>,
}

/// Re-verify every claim's quotes against the current files and refresh
/// confidence. Writes Sync ledger entries; broken-evidence claims go to the
/// review queue (caller passes a queue appender callback).
pub fn sync_all<F>(
    root: &Path,
    ledger: &Path,
    claims: &mut Vec<Claim>,
    mut queue: F,
    now: DateTime<Utc>,
) -> Result<Vec<SyncResult>, KemError>
where
    F: FnMut(&SyncResult) -> Result<(), KemError>,
{
    let mut results = Vec::new();
    for claim in claims.iter_mut() {
        if claim.status == ClaimStatus::Retired {
            continue;
        }
        let mut evidence_ok = 0usize;
        let mut issues = Vec::new();
        for ev in &claim.evidence {
            let path = std::env::current_dir()
                .unwrap_or_default()
                .join(&ev.path);
            match std::fs::read_to_string(&path) {
                Err(e) => issues.push(format!(
                    "evidence unreadable: {} ({e})",
                    ev.path
                )),
                Ok(content) => {
                    if content.contains(ev.quote.as_str()) {
                        evidence_ok += 1;
                    } else {
                        issues.push(format!(
                            "quote drifted in {}",
                            ev.path
                        ));
                    }
                }
            }
        }
        let healthy = issues.is_empty();
        // Recency anchor: last successful sync for this claim from the ledger.
        let last_affirmed = crate::ledger::read_all(ledger)?
            .iter()
            .filter(|e| e.claim_id == claim.id)
            .filter(|e| matches!(e.action, LedgerAction::Sync | LedgerAction::Vet | LedgerAction::File))
            .filter(|e| {
                // only affirming entries count
                !e.detail
                    .get("issues")
                    .and_then(|v| v.as_array())
                    .map_or(false, |a| !a.is_empty())
            })
            .map(|e| e.ts)
            .max();
        let new_conf = recompute_confidence(claim, if healthy { last_affirmed } else { None }, now);
        claim.confidence = new_conf;
        if !healthy {
            // Review item, never a silent deprecation (KEM doc §4.4).
            queue(&SyncResult {
                claim_id: claim.id.clone(),
                evidence_ok,
                evidence_total: claim.evidence.len(),
                confidence: new_conf,
                issues: issues.clone(),
            })?;
        }
        crate::ledger::append(
            ledger,
            "kem-sync",
            LedgerAction::Sync,
            &claim.id,
            serde_json::json!({
                "evidence_ok": evidence_ok,
                "evidence_total": claim.evidence.len(),
                "confidence": new_conf,
                "issues": issues,
            }),
            now,
        )?;
        results.push(SyncResult {
            claim_id: claim.id.clone(),
            evidence_ok,
            evidence_total: claim.evidence.len(),
            confidence: new_conf,
            issues,
        });
    }
    Ok(results)
}

/// Structural replacement (Track A): file the successor, demote the
/// predecessor with a superseded_by link, ledger both directions.
pub fn supersede(
    root: &Path,
    ledger: &Path,
    claims: &mut Vec<Claim>,
    mut successor: Claim,
    old_id: &str,
    now: DateTime<Utc>,
) -> Result<(), KemError> {
    successor.supersedes = vec![old_id.to_string()];
    if successor.track != Track::A {
        return Err(KemError::InvalidInput(
            "supersede is structural: successor claim must be Track A".into(),
        ));
    }
    let known = known_ids_of(claims.iter());
    let errs = validate_claim(
        &successor,
        Some(&crate::schema::ValidationContext { known_ids: known }),
    );
    if !errs.is_empty() {
        return Err(KemError::SchemaViolation(format!(
            "successor invalid: {}",
            errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
        )));
    }
    let old = claims
        .iter_mut()
        .find(|c| c.id == old_id)
        .ok_or_else(|| KemError::NotFound(old_id.to_string()))?;
    if old.status != ClaimStatus::Published && old.status != ClaimStatus::Deprecated {
        return Err(KemError::InvalidInput(format!(
            "cannot supersede {old_id} in status {:?} (only published/deprecated)",
            old.status
        )));
    }
    old.status = ClaimStatus::Superseded;
    old.superseded_by = Some(successor.id.clone());
    let successor_id = successor.id.clone();
    crate::filing::save_claim(&root.join("claims"), old)?;
    crate::filing::save_claim(&root.join("claims"), &successor)?;
    claims.push(successor.clone());
    crate::ledger::append(
        ledger,
        "kem-evolve",
        LedgerAction::Supersede,
        old_id,
        serde_json::json!({"superseded_by": successor_id}),
        now,
    )?;
    crate::ledger::append(
        ledger,
        "kem-evolve",
        LedgerAction::File,
        &successor_id,
        serde_json::json!({"status": "published", "supersedes": old_id}),
        now,
    )?;
    Ok(())
}

/// One T2 vote (parsed from a local LLM response).
#[derive(Debug Clone, PartialEq, Serialize, Deserialize)]
pub struct ScsVote {
    pub score: u8,
    pub rationale: String,
}

/// T2 SCS verdict: k votes, majority fraction = agreement (KEM doc §4.3.2).
#[derive(Debug Clone, Serialize, Deserialize)]
pub struct ScsVerdict {
    pub k: usize,
    pub votes: Vec<ScsVote>,
    pub agreement: f64,
    pub top_score: Option<u8>,
    /// true when no LLM endpoint was reachable — honest UNKNOWN, never a score.
    pub llm_unreachable: bool,
    /// Endpoint actually used (provenance for R1/R4 routing decisions).
    pub endpoint: String,
}

/// Extract the first JSON object from a noisy LLM response.
pub fn extract_json(text: &str) -> Option<serde_json::Value> {
    let start = text.find('{')?;
    let end = text[start..].find('}')? + start;
    serde_json::from_str(&text[start..=end]).ok()
}

/// Majority helper: returns (top_value, agreement) over parsed votes.
pub fn majority<T: PartialEq + Clone>(items: &[Option<T>]) -> (Option<T>, f64) {
    let valid: Vec<&T> = items.iter().flatten().collect();
    if valid.is_empty() {
        return (None, 0.0);
    }
    let mut best: Option<(&T, usize)> = None;
    for candidate in &valid {
        let count = valid.iter().filter(|v| *v == candidate).count();
        if best.map_or(true, |(_, c)| count > c) {
            best = Some((candidate, count));
        }
    }
    let (value, count) = best?;
    (Some(value.clone()), f64::from(count as u32) / f64::from(valid.len() as u32))
}

/// Run the SCS voting loop against one OpenAI-compatible chat endpoint.
/// Tries endpoints in order (T2a local first); ALL unreachable → honest
/// `llm_unreachable` verdict (never a fabricated score).
pub async fn scs_vote(
    endpoints: &[(&str, &str)], // (label, base_url)
    model: &str,
    claim_title: &str,
    claim_body: &str,
    k: usize,
) -> ScsVerdict {
    let prompt = format!(
        "You are a strict knowledge curator. Judge this knowledge claim.\n\
         Title: {claim_title}\nBody: {claim_body}\n\
         Reply with ONLY a JSON object: {{\"durability\": <0-5 integer>, \"importance\": <0-5 integer>}}\n\
         durability = how long this stays true (5 = permanent invariant)."
    );
    let mut votes: Vec<Option<ScsVote>> = Vec::new();
    let mut used_endpoint = String::from("none");

    'outer: for (label, base) in endpoints {
        let url = format!("{base}/chat/completions");
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
        {
            Ok(c) => c,
            Err(_) => continue,
        };
        for i in 0..k {
            let body = serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": &prompt}],
                "temperature": 0.7,
                "max_tokens": 200,
            });
            let resp = client.post(&url).json(&body).send().await;
            match resp {
                Err(_) => {
                    if i == 0 {
                        continue 'outer; // endpoint down, try next pool
                    }
                    votes.push(None);
                    continue;
                }
                Ok(r) => {
                    if !r.status().is_success() {
                        if i == 0 {
                            continue 'outer; // rate-limited/unauthorized → next pool
                        }
                        votes.push(None);
                        continue;
                    }
                    let parsed: Result<serde_json::Value, _> = r.json().await;
                    let text = parsed
                        .ok()
                        .and_then(|v| {
                            v["choices"][0]["message"]["content"]
                                .as_str()
                                .map(String::from)
                        })
                        .unwrap_or_default();
                    match extract_json(&text) {
                        Some(j) => votes.push(Some(ScsVote {
                            score: j
                                .get("durability")
                                .and_then(|d| d.as_u64())
                                .map(|d| d.min(5) as u8)
                                .unwrap_or(0),
                            rationale: j
                                .get("importance")
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                        })),
                        None => votes.push(None),
                    }
                    used_endpoint = label.to_string();
                }
            }
        }
        if votes.iter().any(|v| v.is_some()) {
            break; // this pool served at least one vote; stop failing over
        }
    }

    let served: Vec<Option<u8>> = votes.iter().map(|v| v.as_ref().map(|s| s.score)).collect();
    let (top, agreement) = majority(&served);
    ScsVerdict {
        k,
        votes: votes.into_iter().flatten().collect(),
        agreement,
        top_score: top,
        llm_unreachable: used_endpoint == "none",
        endpoint: used_endpoint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_from_noisy_text() {
        let v = extract_json("sure! here: {\"durability\": 4, \"importance\": 3} hope that helps");
        assert_eq!(v.unwrap()["durability"], 4);
        assert!(extract_json("no json here").is_none());
    }

    #[test]
    fn majority_computes_agreement() {
        let votes = vec![Some(4u8), Some(4), None, Some(3), Some(4)];
        let (top, agreement) = majority(&votes);
        assert_eq!(top, Some(4));
        assert!((agreement - 0.75).abs() < 1e-9);
    }

    #[test]
    fn all_none_gives_zero_agreement() {
        let (top, agreement) = majority::<u8>(&[None, None, None]);
        assert_eq!(top, None);
        assert!((agreement - 0.0).abs() < 1e-9);
    }
}

    fn claim(kind: ClaimType, days_old: i64) -> Claim {
        Claim {
            id: "claim-20261107-001".into(),
            claim_type: kind,
            status: ClaimStatus::Published,
            track: Track::B,
            confidence: 0.8,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
            valid_to: None,
            recorded_at: Utc::now() - chrono::Duration::days(days_old),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "e.txt".into(),
                rev: "working-tree".into(),
                quote: "q".into(),
            }],
            topics: vec!["t".into()],
            source: "raw/t/2026-11-07-a.md".into(),
            title: "t".into(),
            body: "b".into(),
        }
    }

    #[test]
    fn concept_does_not_decay() {
        let old = claim(ClaimType::Concept, 400);
        let c = recompute_confidence(&old, None, Utc::now());
        assert!((c - 0.8).abs() < 1e-9);
    }

    #[test]
    fn fact_decays_with_90d_half_life() {
        let old = claim(ClaimType::Fact, 90);
        let c = recompute_confidence(&old, None, Utc::now());
        // base 0.8 after one half-life → 0.4
        assert!((c - 0.4).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn decay_never_drops_below_floor() {
        let old = claim(ClaimType::Gotcha, 3650);
        let c = recompute_confidence(&old, None, Utc::now());
        assert!((c - 0.30).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn reaffirmation_restores_recency() {
        let old = claim(ClaimType::Fact, 90);
        let affirmed = Utc::now() - chrono::Duration::days(1);
        let c = recompute_confidence(&old, Some(affirmed), Utc::now());
        assert!(c > 0.7, "reaffirmed claim should be near base, got {c}");
    }

    #[test]
    fn supersede_requires_track_a() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("ledger.jsonl");
        let mut claims = vec![claim(ClaimType::Fact, 0)];
        let mut succ = claim(ClaimType::Fact, 0);
        succ.track = Track::B;
        succ.id = "claim-20261107-002".into();
        let err = supersede(dir.path(), &ledger, &mut claims, succ, "claim-20261107-001", Utc::now())
            .unwrap_err();
        assert!(err.to_string().contains("Track A"));
    }

    #[test]
    fn supersede_links_both_directions() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = dir.path().join("ledger.jsonl");
        let mut claims = vec![claim(ClaimType::Fact, 0)];
        let mut succ = claim(ClaimType::Fact, 0);
        succ.track = Track::A;
        succ.id = "claim-20261107-002".into();
        supersede(dir.path(), &ledger, &mut claims, succ, "claim-20261107-001", Utc::now()).unwrap();
        let old = claims.iter().find(|c| c.id == "claim-20261107-001").unwrap();
        let new = claims.iter().find(|c| c.id == "claim-20261107-002").unwrap();
        assert_eq!(old.status, ClaimStatus::Superseded);
        assert_eq!(old.superseded_by.as_deref(), Some("claim-20261107-002"));
        assert_eq!(new.supersedes, vec!["claim-20261107-001"]);
        assert_eq!(crate::ledger::verify(&ledger).unwrap(), 2);
        // both files exist on disk
        assert!(dir.path().join("claims/claim-20261107-001.json").exists());
        assert!(dir.path().join("claims/claim-20261107-002.json").exists());
    }
}
