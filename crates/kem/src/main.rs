#![forbid(unsafe_code)]
//! KEM CLI — subcommands over the knowledge root (default `knowledge/`).
//!
//! P0: seed, validate · P1: capture, ingest, finalize · P2: ask ·
//! P3: sync · P4: lint, doctor, status. PowerShell-friendly: one command
//! per invocation, JSON or line output, exit code = result.

use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kem::filing::{load_claims, write_index, write_log};
use kem::ledger::LedgerAction;
use kem::schema::{Claim, KemError, ValidationContext, known_ids_of, validate_claim};
use kem::seed::{seed_claims, seed_raw_sources};
use kem::vetting::{Outcome, VetContext, record_verdict, stage_claim, vet_t1};

struct Args {
    root: PathBuf,
    cmd: String,
    flags: std::collections::HashMap<String, String>,
}

fn parse_args() -> Result<Args, String> {
    let mut argv = std::env::args().skip(1);
    let mut root = PathBuf::from("knowledge");
    let mut cmd = String::new();
    let mut flags = std::collections::HashMap::new();
    let mut pending_flag: Option<String> = None;
    for a in argv.by_ref() {
        if let Some(name) = a.strip_prefix("--") {
            if let Some(p) = pending_flag.take() {
                return Err(format!("flag --{p} missing value"));
            }
            if name == "root" {
                pending_flag = Some(name.to_string());
            } else {
                flags.insert(name.to_string(), String::new());
                pending_flag = Some(name.to_string());
            }
        } else if let Some(p) = pending_flag.take() {
            if p == "root" {
                root = PathBuf::from(a);
            } else {
                flags.insert(p, a);
            }
        } else if cmd.is_empty() {
            cmd = a;
        } else {
            return Err(format!("unexpected positional arg: {a}"));
        }
    }
    if let Some(p) = pending_flag {
        return Err(format!("flag --{p} missing value"));
    }
    if cmd.is_empty() {
        return Err("usage: kem [--root DIR] <seed|validate|capture|ingest|finalize|ask|sync|lint|doctor|status>".into());
    }
    Ok(Args { root, cmd, flags })
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kem: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), KemError> {
    let args = parse_args().map_err(KemError::InvalidInput)?;
    let root = args.root;
    let ledger = root.join(".kem/ledger.jsonl");
    match args.cmd.as_str() {
        "seed" => cmd_seed(&root, &ledger),
        "validate" => cmd_validate(&root),
        "capture" => cmd_capture(&root, &ledger, &args.flags),
        "ingest" => cmd_ingest(&root, &ledger, flag(&args.flags, "file")?),
        "finalize" => cmd_finalize(&root, &ledger),
        other => Err(KemError::InvalidInput(format!(
            "unknown command {other:?} (P2+ commands ship with their phase)"
        ))),
    }
}

fn flag(flags: &std::collections::HashMap<String, String>, name: &str) -> Result<String, KemError> {
    flags
        .get(name)
        .cloned()
        .ok_or_else(|| KemError::InvalidInput(format!("--{name} is required")))
}

/// P0: stage raw sources, vet and file the 20 seed claims, record the ledger.
fn cmd_seed(root: &Path, ledger: &Path) -> Result<(), KemError> {
    std::fs::create_dir_all(root.join(".kem"))?;
    let now = Utc::now();
    let mut staged = 0usize;
    for (topic, content) in seed_raw_sources() {
        let src = kem::capture::capture_source(root, topic, "seed", "md", &content, now)?;
        if src.created {
            kem::ledger::append(
                ledger,
                "kem-seed",
                LedgerAction::Capture,
                "",
                serde_json::json!({"rel": src.rel_path, "sha256": src.sha256}),
                now,
            )?;
        }
        staged += 1;
    }
    let claims = seed_claims();
    // Evidence paths are workspace-relative (KEM doc §4.2): resolve against
    // the workspace root (cwd), not the knowledge store root.
    let ctx = VetContext::new(PathBuf::from("."), Vec::new());
    let mut accepted = 0usize;
    for claim in &claims {
        let verdict = vet_t1(claim, &ctx);
        if verdict.outcome != Outcome::Accept {
            return Err(KemError::SchemaViolation(format!(
                "seed claim {} not accepted: {:?} — {}",
                claim.id, verdict.outcome, verdict.reasons.join("; ")
            )));
        }
        record_verdict(ledger, &claim.id, &verdict, now)?;
        stage_claim(&root.join("claims"), claim)?;
        kem::ledger::append(
            ledger,
            "kem-seed",
            LedgerAction::File,
            &claim.id,
            serde_json::json!({"status": "published"}),
            now,
        )?;
        accepted += 1;
    }
    let all = load_claims(&root.join("claims"))?;
    write_index(root, &all)?;
    write_log(root, ledger)?;
    println!("seed ok: {staged} raw sources, {accepted} claims filed, ledger verified");
    Ok(())
}

/// P0: validate every stored claim against the schema (fail-closed).
fn cmd_validate(root: &Path) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let ctx = ValidationContext {
        known_ids: known_ids_of(claims.iter()),
    };
    let mut bad = 0usize;
    for c in &claims {
        let errs = validate_claim(c, Some(&ctx));
        if !errs.is_empty() {
            bad += 1;
            for e in errs {
                eprintln!("INVALID {}: {e}", c.id);
            }
        }
    }
    println!("validate: {} claims, {bad} invalid", claims.len());
    if bad > 0 {
        Err(KemError::SchemaViolation(format!("{bad} invalid claims")))
    } else {
        Ok(())
    }
}

/// P1 M1: stage an immutable raw source.
fn cmd_capture(root: &Path, ledger: &Path, flags: &std::collections::HashMap<String, String>) -> Result<(), KemError> {
    let topic = flag(flags, "topic")?;
    let name = flag(flags, "name")?;
    let file = flag(flags, "file")?;
    let ext = flags.get("ext").cloned().unwrap_or_else(|| "md".into());
    let content = std::fs::read_to_string(&file)
        .map_err(|e| KemError::InvalidInput(format!("cannot read {file}: {e}")))?;
    let src = kem::capture::capture_source(root, &topic, &name, &ext, &content, Utc::now())?;
    kem::ledger::append(
        ledger,
        "kem-capture",
        LedgerAction::Capture,
        "",
        serde_json::json!({"rel": src.rel_path, "sha256": src.sha256}),
        Utc::now(),
    )?;
    println!("captured: {} ({} bytes)", src.rel_path, src.bytes);
    Ok(())
}

/// P1 M2–M3: vet a batch of drafted claims; accept/review/reject per T1.
fn cmd_ingest(root: &Path, ledger: &Path, batch_file: String) -> Result<(), KemError> {
    let text = std::fs::read_to_string(&batch_file)
        .map_err(|e| KemError::InvalidInput(format!("cannot read {batch_file}: {e}")))?;
    let drafts: Vec<Claim> = serde_json::from_str(&text)?;
    if drafts.is_empty() {
        return Err(KemError::InvalidInput("batch is empty".into()));
    }
    let existing = load_claims(&root.join("claims"))?;
    let ctx = VetContext::new(PathBuf::from("."), existing.clone());
    let mut known = known_ids_of(existing.iter());
    let queue = root.join(".kem/review-queue.jsonl");
    let now = Utc::now();
    let (mut acc, mut rev, mut rej) = (0usize, 0usize, 0usize);
    for mut claim in drafts {
        if claim.id.is_empty() {
            claim.id = next_id(&existing, &known)?;
            known.insert(claim.id.clone());
        }
        let verdict = vet_t1(&claim, &ctx);
        record_verdict(ledger, &claim.id, &verdict, now)?;
        match verdict.outcome {
            Outcome::Accept => {
                stage_claim(&root.join("claims"), &claim)?;
                kem::ledger::append(ledger, "kem-t1", LedgerAction::File, &claim.id,
                    serde_json::json!({"status": format!("{:?}", claim.status).to_lowercase()}), now)?;
                acc += 1;
            }
            Outcome::Review => {
                let item = serde_json::json!({
                    "claim_id": claim.id, "reasons": verdict.reasons,
                    "scores": verdict.scores, "queued_at": now.to_rfc3339(),
                });
                append_jsonl(&queue, &item)?;
                rev += 1;
            }
            Outcome::Reject => {
                let item = serde_json::json!({
                    "claim_id": claim.id, "verdict": "reject",
                    "reasons": verdict.reasons, "triage": "see docs KEM §4.3.1", "at": now.to_rfc3339(),
                });
                append_jsonl(&queue, &item)?;
                rej += 1;
            }
        }
    }
    println!("ingest: {acc} accepted, {rev} review, {rej} rejected");
    Ok(())
}

/// P1 M4: regenerate index.md and log.md from the SSOT.
fn cmd_finalize(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    write_index(root, &claims)?;
    write_log(root, ledger)?;
    println!("finalize: index.md + log.md regenerated ({} claims)", claims.len());
    Ok(())
}

fn next_id(existing: &[Claim], known: &std::collections::BTreeSet<String>) -> Result<String, KemError> {
    let today = Utc::now().date_naive().format("%Y%m%d");
    let prefix = format!("claim-{today}-");
    let mut max = 0u32;
    for c in existing.iter().map(|c| &c.id).chain(known.iter()) {
        if let Some(num) = c.strip_prefix(&prefix) {
            if let Ok(n) = num.parse::<u32>() {
                max = max.max(n);
            }
        }
    }
    Ok(format!("{prefix}{:03}", max + 1))
}

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<(), KemError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(serde_json::to_string(value)?.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}
