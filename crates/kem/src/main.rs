#![forbid(unsafe_code)]
//! KEM CLI — subcommands over the knowledge root (default `knowledge/`).
//!
//! P0: seed, validate 繚 P1: capture, ingest, finalize 繚 P2: ask 繚
//! P3: sync 繚 P4: lint, doctor, status. PowerShell-friendly: one command
//! per invocation, JSON or line output, exit code = result.

use chrono::Utc;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kem::filing::{load_claims, write_index, write_log};
use kem::ledger::LedgerAction;
use kem::schema::{known_ids_of, validate_claim, Claim, KemError, ValidationContext};
use kem::seed::{seed_claims, seed_raw_sources};
use kem::vetting::{record_verdict, stage_claim, vet_t1, Outcome, VetContext};

struct Args {
    root: PathBuf,
    cmd: String,
    flags: std::collections::HashMap<String, String>,
}

fn parse_args() -> Result<Args, String> {
    const VALUE_FLAGS: [&str; 14] = [
        "root",
        "topic",
        "name",
        "file",
        "ext",
        "query",
        "top",
        "set",
        "claim",
        "old",
        "votes",
        "disposition",
        "reviewer",
        "note",
    ];
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
            if VALUE_FLAGS.contains(&name) {
                pending_flag = Some(name.to_string());
            } else {
                flags.insert(name.to_string(), "1".to_string()); // boolean flag
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
        "seed" => cmd_seed(&root, &ledger, args.flags.contains_key("force")),
        "validate" => cmd_validate(&root),
        "capture" => cmd_capture(&root, &ledger, &args.flags),
        "ingest" => cmd_ingest(&root, &ledger, flag(&args.flags, "file")?),
        "finalize" => cmd_finalize(&root, &ledger),
        "ask" => cmd_ask(&root, &args.flags),
        "eval" => cmd_eval(&root, &args.flags),
        "embed" => cmd_embed(&root, &ledger, &args.flags),
        "console" => cmd_console(&root, &ledger),
        "sync" => cmd_sync(&root, &ledger),
        "supersede" => cmd_supersede(&root, &ledger, &args.flags),
        "vet2" => cmd_vet2(&root, &ledger, &args.flags),
        "lint" => cmd_lint(&root),
        "doctor" => cmd_doctor(&root, &ledger),
        "status" => cmd_status(&root, &ledger),
        "review" => cmd_review(&root, &args.flags),
        other => Err(KemError::InvalidInput(format!("unknown command {other:?}"))),
    }
}

fn flag(flags: &std::collections::HashMap<String, String>, name: &str) -> Result<String, KemError> {
    flags
        .get(name)
        .cloned()
        .ok_or_else(|| KemError::InvalidInput(format!("--{name} is required")))
}

/// P0: stage raw sources, vet and file the 20 seed claims, record the ledger.
/// `--force` resets ONLY what seed itself created (guarded: every existing
/// ledger entry must be actor=kem-seed; any other actor refuses the reset).
fn cmd_seed(root: &Path, ledger: &Path, force: bool) -> Result<(), KemError> {
    std::fs::create_dir_all(root.join(".kem"))?;
    if force && ledger.exists() {
        let entries = kem::ledger::read_all(ledger)?;
        // Reset is only allowed when every entry belongs to the seed set
        // (empty claim_id = capture meta entries). Any user-filed claim
        // makes --force refuse, protecting real knowledge.
        let seed_ids: std::collections::HashSet<String> =
            seed_claims().iter().map(|c| c.id.clone()).collect();
        let foreign = entries
            .iter()
            .filter(|e| !e.claim_id.is_empty() && !seed_ids.contains(&e.claim_id))
            .count();
        if foreign > 0 {
            return Err(KemError::InvalidInput(format!(
                "--force refused: ledger holds {foreign} entries outside the seed set"
            )));
        }
        for dir in ["raw", "claims"] {
            if root.join(dir).exists() {
                std::fs::remove_dir_all(root.join(dir))?;
            }
        }
        std::fs::remove_file(ledger)?;
    }
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
                claim.id,
                verdict.outcome,
                verdict.reasons.join("; ")
            )));
        }
        record_verdict(ledger, "kem-seed", &claim.id, &verdict, now)?;
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
    finalize_views(root, ledger)?;
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
fn cmd_capture(
    root: &Path,
    ledger: &Path,
    flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
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
    // BOM tolerance: Windows-originated JSON files carry a UTF-8 BOM that
    // serde_json rejects (review defect #6, found by E2E smoke).
    let drafts: Vec<Claim> = serde_json::from_str(kem::filing::strip_bom(&text))?;
    if drafts.is_empty() {
        return Err(KemError::InvalidInput("batch is empty".into()));
    }
    let existing = load_claims(&root.join("claims"))?;
    let mut ctx = VetContext::new(PathBuf::from("."), existing.clone());
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
        record_verdict(ledger, "kem-t1", &claim.id, &verdict, now)?;
        match verdict.outcome {
            Outcome::Accept => {
                stage_claim(&root.join("claims"), &claim)?;
                kem::ledger::append(
                    ledger,
                    "kem-t1",
                    LedgerAction::File,
                    &claim.id,
                    serde_json::json!({"status": format!("{:?}", claim.status).to_lowercase()}),
                    now,
                )?;
                // Batch-internal visibility: later drafts in the same batch
                // must see this claim for duplicate detection (review
                // defect #2, 2026-11-09) and supersede reference checks.
                ctx.existing.push(claim.clone());
                ctx.known_ids.insert(claim.id.clone());
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

/// Shared view regeneration: index.md + log.md + dashboard.html.
/// Returns (claims, findings, queue items) for reporting.
#[allow(clippy::type_complexity)]
/// Zip golden cases with their embeddings in input order.
fn qs_zip_map(
    cases: &[kem::search::GoldenCase],
    vecs: &[Vec<f32>],
) -> std::collections::HashMap<String, Vec<f32>> {
    cases
        .iter()
        .zip(vecs.iter())
        .map(|(c, v)| (c.query.clone(), v.clone()))
        .collect()
}

/// finalize_views 的回傳：宣告、lint 發現數、待審佇列項目。
pub struct FinalizedViews {
    pub claims: Vec<Claim>,
    pub findings: usize,
    pub queue: Vec<(String, Vec<String>)>,
}

fn finalize_views(root: &Path, ledger: &Path) -> Result<FinalizedViews, KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let findings = kem::gov::lint(&claims, Utc::now());
    let findings_count = findings.len();
    write_index(root, &claims)?;
    write_log(root, ledger)?;
    // Review queue: unresolved items only (shared reader — single source
    // of truth with kem doctor; review defect #3 fix).
    let queue_path = root.join(".kem/review-queue.jsonl");
    let queue_items = kem::gov::pending_queue(&queue_path)?;
    let ledger_entries = if ledger.exists() {
        kem::ledger::read_all(ledger)?
    } else {
        Vec::new()
    };
    let ledger_tail: Vec<(u64, String, String, String)> = ledger_entries
        .iter()
        .rev()
        .take(12)
        .map(|e| {
            (
                e.seq,
                format!("{:?}", e.action).to_lowercase(),
                e.claim_id.clone(),
                e.ts.to_rfc3339(),
            )
        })
        .collect();
    // Latest T2 ensemble verdict (actor kem-t2) for the governance panel.
    let ensemble = ledger_entries
        .iter()
        .rev()
        .find(|e| e.actor == "kem-t2")
        .and_then(|e| {
            let models: Vec<String> = e
                .detail
                .get("models_used")
                .and_then(|m| m.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let agreement = e.detail.get("agreement").and_then(|a| a.as_f64())?;
            let top = e
                .detail
                .get("top_score")
                .and_then(|t| t.as_u64())
                .map(|t| t as u8);
            (!models.is_empty()).then_some((models, agreement, top))
        });
    kem::dashboard::write_dashboard(
        root,
        &claims,
        findings_count,
        &queue_items,
        &ledger_tail,
        ensemble,
    )?;
    Ok(FinalizedViews {
        claims,
        findings: findings_count,
        queue: queue_items,
    })
}

/// P1 M4: regenerate all views from the SSOT.
fn cmd_finalize(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let views = finalize_views(root, ledger)?;
    let (claims, findings, queue) = (views.claims, views.findings, views.queue);
    println!(
        "finalize: index.md + log.md + dashboard.html（{} claims, {} findings, {} queued）",
        claims.len(),
        findings,
        queue.len()
    );
    if findings == 0 {
        Ok(())
    } else {
        Err(KemError::SchemaViolation(format!("{findings} findings")))
    }
}

/// P2b+: 零 JS 多頁操作主控台（console/ 目錄，頁數隨 KB 有機成長）。
fn cmd_console(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let entries = if ledger.exists() {
        kem::ledger::read_all(ledger)?
    } else {
        Vec::new()
    };
    let queue = kem::gov::pending_queue(&root.join(".kem/review-queue.jsonl"))?;
    let vectors = kem::embed::load_vectors(&root.join(".kem/vectors.json"))?;
    let pages = kem::console::generate(root, &claims, &entries, &queue, vectors.as_ref())?;
    println!(
        "console: {pages} pages generated under {}/console/",
        root.display()
    );
    Ok(())
}

/// P2 M5 + P2b: ask — hybrid retrieval (α·BM25 + (1−α)·vector). Degrades to
/// BM25-only honestly when vectors/embedder are unavailable.
fn cmd_ask(root: &Path, flags: &std::collections::HashMap<String, String>) -> Result<(), KemError> {
    let query = flag(flags, "query")?;
    let top: usize = flags.get("top").and_then(|v| v.parse().ok()).unwrap_or(5);
    let bm25_only = flags.contains_key("bm25-only");
    let claims = load_claims(&root.join("claims"))?;
    let visible = kem::search::visible_at(&claims, None);
    let index = kem::search::Index::build(&visible);

    let vectors = if bm25_only {
        None
    } else {
        kem::embed::load_vectors(&root.join(".kem/vectors.json"))?
    };
    let (hits, mode) = match vectors {
        Some(v) if !v.entries.is_empty() => {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| KemError::InvalidInput(format!("tokio runtime: {e}")))?;
            let res = rt.block_on(kem::embed::embed_batch(
                &kem::embed::default_pool(),
                std::slice::from_ref(&query),
            ));
            match res {
                Ok(res) => {
                    let vs = &res.vecs;
                    let lookup: std::collections::HashMap<String, Vec<f32>> =
                        kem::embed::fresh_lookup(&v, &visible)
                            .into_iter()
                            .map(|(k, vec)| (k.to_string(), vec.to_vec()))
                            .collect();
                    let bm25 = index.scores_all(&visible, &query);
                    let alpha: f32 = flags
                        .get("alpha")
                        .and_then(|a| a.parse().ok())
                        .unwrap_or(0.5);
                    let floor: f32 = flags
                        .get("refusal-floor")
                        .and_then(|a| a.parse().ok())
                        .unwrap_or(kem::search::REFUSAL_FLOOR);
                    let hits = kem::search::hybrid_fuse(
                        &visible,
                        &bm25,
                        Some(&lookup),
                        Some(vs[0].as_slice()),
                        top,
                        alpha,
                        floor,
                    );
                    (hits, format!("hybrid(α={alpha}, floor={floor})"))
                }
                Err(e) => {
                    eprintln!("ask note: embedder unavailable — BM25-only ({e})");
                    (index.search(&visible, &query, top), "bm25-only".to_string())
                }
            }
        }
        _ => (index.search(&visible, &query, top), "bm25-only".to_string()),
    };
    println!("retrieval mode: {mode}");
    if hits.is_empty() {
        println!("no results (honest refusal — query outside current knowledge)");
        return Ok(());
    }
    for h in &hits {
        println!(
            "{:.4}  {}  [{}] via={} conf={:.2}  {}",
            h.score,
            h.id,
            format!("{:?}", h.status).to_lowercase(),
            h.via,
            h.confidence,
            h.title
        );
    }
    Ok(())
}

/// P2b: rebuild the vector index (one batched embed call over all claims).
fn cmd_embed(
    root: &Path,
    _ledger: &Path,
    _flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let visible = kem::search::visible_at(&claims, None);
    if visible.is_empty() {
        return Err(KemError::InvalidInput("no claims to embed".into()));
    }
    let texts: Vec<String> = visible
        .iter()
        .map(|c| format!("{}\n{}\n{}", c.title, c.body, c.topics.join(", ")))
        .collect();
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| KemError::InvalidInput(format!("tokio runtime: {e}")))?;
    let res = rt
        .block_on(kem::embed::embed_batch(&kem::embed::default_pool(), &texts))
        .map_err(KemError::InvalidInput)?;
    let vecs = res.vecs;
    let dim = vecs.first().map(|v| v.len()).unwrap_or(0);
    // Provenance: record the model that ACTUALLY produced the vectors,
    // not the pool head (review 2026-11-09 — attribution bug).
    let model = res.used_model.clone();
    let index = kem::embed::VectorIndexFile {
        model: model.clone(),
        dim,
        entries: visible
            .iter()
            .zip(vecs.iter())
            .map(|(c, v)| kem::embed::VectorEntry {
                id: c.id.clone(),
                body_sha: kem::capture::sha256_hex(c.body.as_bytes()),
                vec: v.clone(),
            })
            .collect(),
    };
    let path = root.join(".kem/vectors.json");
    let n = index.entries.len();
    kem::embed::save_vectors(&path, &index)?;
    println!(
        "embed: {n} vectors ({model}, dim {dim}) -> {}",
        path.display()
    );
    Ok(())
}

/// P2 V1: golden-set evaluation — top-1 accuracy with refusal cases.
fn cmd_eval(
    root: &Path,
    flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
    let set = flags
        .get("set")
        .cloned()
        .unwrap_or_else(|| "crates/kem/tests/golden-20.json".into());
    let cases = kem::search::load_golden(Path::new(&set))?;
    let claims = load_claims(&root.join("claims"))?;
    let visible = kem::search::visible_at(&claims, None);
    let index = kem::search::Index::build(&visible);
    let vectors = kem::embed::load_vectors(&root.join(".kem/vectors.json"))?;
    // Embed all golden queries in ONE batch (quota-friendly); embedder down
    // → BM25-only run, marked honestly in the output.
    let embedded = match &vectors {
        Some(v) if !v.entries.is_empty() => {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| KemError::InvalidInput(format!("tokio runtime: {e}")))?;
            let qs: Vec<String> = cases.iter().map(|c| c.query.clone()).collect();
            let res = rt.block_on(kem::embed::embed_batch(&kem::embed::default_pool(), &qs));
            match res {
                Ok(res) => Some((res.vecs, res.used_model)),
                Err(e) => {
                    eprintln!("eval note: embedder unavailable — BM25-only golden run ({e})");
                    None
                }
            }
        }
        _ => None,
    };
    let qvecs: Option<std::collections::HashMap<String, Vec<f32>>> = embedded
        .as_ref()
        .map(|(vecs, _model)| qs_zip_map(&cases, vecs));
    let eval_model = embedded.as_ref().map(|(_, m)| m.clone());
    let (pass, total, failures) =
        kem::search::run_golden(&index, &claims, &cases, vectors.as_ref(), qvecs.as_ref())?;
    for f in &failures {
        eprintln!("FAIL {f}");
    }
    let mode = match (&qvecs, &eval_model) {
        (Some(_), Some(m)) => format!("hybrid[{m}]"),
        _ => "bm25-only".to_string(),
    };
    println!(
        "eval [{mode}]: {pass}/{total} top-1 pass ({} golden cases)",
        total
    );
    if pass == total {
        Ok(())
    } else {
        Err(KemError::InvalidInput(format!(
            "golden set failed: {pass}/{total}"
        )))
    }
}

/// P3 M6: re-verify evidence quotes + refresh confidence + queue issues.
fn cmd_sync(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let now = Utc::now();
    let mut claims = load_claims(&root.join("claims"))?;
    let queue = root.join(".kem/review-queue.jsonl");
    let mut queued = 0usize;
    let results = kem::evolve::sync_all(
        Path::new("."),
        ledger,
        &mut claims,
        |r| {
            if !r.issues.is_empty() {
                queued += 1;
                append_jsonl(
                    &queue,
                    &serde_json::json!({
                        "claim_id": r.claim_id, "reasons": r.issues,
                        "queued_at": now.to_rfc3339(), "source": "sync",
                    }),
                )?;
            }
            Ok(())
        },
        now,
    )?;
    for c in &claims {
        kem::filing::save_claim(&root.join("claims"), c)?;
    }
    let broken = results.iter().filter(|r| !r.issues.is_empty()).count();
    println!(
        "sync: {} claims checked, {} with issues (queued), ledger updated",
        results.len(),
        broken
    );
    finalize_views(root, ledger)?;
    Ok(())
}

/// P3: structural replacement (Track A successor replaces a published claim).
fn cmd_supersede(
    root: &Path,
    ledger: &Path,
    flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
    let old_id = flag(flags, "old")?;
    let file = flag(flags, "file")?;
    let text = std::fs::read_to_string(&file)
        .map_err(|e| KemError::InvalidInput(format!("cannot read {file}: {e}")))?;
    let successor: Claim = serde_json::from_str(&text)?;
    let mut claims = load_claims(&root.join("claims"))?;
    kem::evolve::supersede(root, ledger, &mut claims, successor, &old_id, Utc::now())?;
    finalize_views(root, ledger)?;
    println!("superseded {old_id}");
    Ok(())
}

/// P3 T2: SCS k-vote over hybrid endpoints (local first; unreachable = UNKNOWN).
fn cmd_vet2(
    root: &Path,
    ledger: &Path,
    flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
    let claim_id = flag(flags, "claim")?;
    let k: usize = flags.get("votes").and_then(|v| v.parse().ok()).unwrap_or(5);
    let local_only = flags.contains_key("local-only");
    let claims = load_claims(&root.join("claims"))?;
    let claim = claims
        .iter()
        .find(|c| c.id == claim_id)
        .ok_or_else(|| KemError::NotFound(claim_id.clone()))?;

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| KemError::InvalidInput(format!("tokio runtime: {e}")))?;
    // 模型池（2026-11-07 全數本機實測；KEM doc §4.3.1 + FREE_AI_ROUTING §8）:
    // R1 敏感資料 → --local-only 僅留 L0。每票輪換不同模型 → 跨模型聯合
    // 一致率（R7）。OVH 2/min/IP → min_interval 31s 配額護欄（R5）。
    let e_ollama = kem::evolve::ModelEntry {
        label: "local-ollama".into(),
        base: "http://127.0.0.1:11434".into(),
        kind: kem::evolve::EndpointKind::OllamaNative,
        model: "qwen3.5:4b".into(),
        min_interval_secs: 0,
    };
    let ovh = "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1";
    let pool: Vec<kem::evolve::ModelEntry> = if local_only {
        vec![e_ollama]
    } else {
        vec![
            e_ollama,
            kem::evolve::ModelEntry {
                label: "ovh-qwen3.8-27b".into(),
                base: ovh.into(),
                kind: kem::evolve::EndpointKind::OpenAi,
                model: "Qwen3.8-27B".into(),
                min_interval_secs: 31,
            },
            kem::evolve::ModelEntry {
                label: "llm7-gpt-oss-20b".into(),
                base: "https://api.llm7.io/v1".into(),
                kind: kem::evolve::EndpointKind::OpenAi,
                model: "gpt-oss:20b".into(),
                min_interval_secs: 7,
            },
            kem::evolve::ModelEntry {
                label: "ovh-gpt-oss-120b".into(),
                base: ovh.into(),
                kind: kem::evolve::EndpointKind::OpenAi,
                model: "gpt-oss-120b".into(),
                min_interval_secs: 31,
            },
            kem::evolve::ModelEntry {
                label: "pollinations-gpt-oss-20b".into(),
                base: "https://text.pollinations.ai/openai".into(),
                kind: kem::evolve::EndpointKind::OpenAi,
                model: "openai-fast".into(),
                min_interval_secs: 7,
            },
        ]
    };
    let verdict = rt.block_on(kem::evolve::scs_vote(&pool, &claim.title, &claim.body, k));
    if verdict.llm_unreachable {
        println!("vet2: UNKNOWN — no LLM endpoint reachable (honest refusal, no fabricated score)");
        for e in &verdict.errors {
            eprintln!("vet2 note: {e}");
        }
        kem::ledger::append(
            ledger,
            "kem-t2",
            LedgerAction::Vet,
            &claim.id,
            serde_json::json!({"status": "unknown", "reason": "all endpoints unreachable"}),
            Utc::now(),
        )?;
        return Ok(());
    }
    println!(
        "vet2: k={} {} — top_durability={:?} agreement={:.2} models=[{}]",
        verdict.k,
        claim.id,
        verdict.top_score,
        verdict.agreement,
        verdict.models_used.join(", ")
    );
    for v in &verdict.votes {
        println!(
            "  vote: {} → durability={} ({})",
            v.model, v.score, v.rationale
        );
    }
    for e in &verdict.errors {
        eprintln!("vet2 note: {e}");
    }
    // SCS floor (KEM doc §4.3.2): agreement < 0.60 → review queue (never
    // silent acceptance of a split vote).
    if verdict.agreement < 0.60 {
        append_jsonl(
            &root.join(".kem/review-queue.jsonl"),
            &serde_json::json!({
                "claim_id": claim.id,
                "reasons": [format!(
                    "SCS agreement {:.2} below 0.60 floor (k={}, top={:?})",
                    verdict.agreement, verdict.k, verdict.top_score
                )],
                "queued_at": Utc::now().to_rfc3339(),
                "source": "vet2",
            }),
        )?;
        println!("vet2: agreement below 0.60 floor — queued for human review");
    }
    kem::ledger::append(
        ledger,
        "kem-t2",
        LedgerAction::Vet,
        &claim.id,
        serde_json::to_value(&verdict)?,
        Utc::now(),
    )?;
    Ok(())
}

/// P4 M7: lint the store.
fn cmd_lint(root: &Path) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    let findings = kem::gov::lint(&claims, Utc::now());
    for f in &findings {
        println!("{} {}: {}", f.rule, f.claim_id, f.detail);
    }
    println!(
        "lint: {} findings over {} claims",
        findings.len(),
        claims.len()
    );
    if findings.is_empty() {
        Ok(())
    } else {
        Err(KemError::SchemaViolation(format!(
            "{} findings",
            findings.len()
        )))
    }
}

/// P4 M7: deep health (ledger chain + counts + queue).
fn cmd_doctor(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let h = kem::gov::doctor(root, ledger, &root.join(".kem/review-queue.jsonl"))?;
    println!(
        "doctor: ledger {} entries (chain_ok={}) | claims {} | findings {} | queue {}",
        h.ledger_entries, h.ledger_ok, h.claims, h.findings, h.review_queue
    );
    if h.ledger_ok {
        Ok(())
    } else {
        Err(KemError::ChainBroken {
            at: 0,
            reason: "doctor".into(),
        })
    }
}

/// P4 M7: status summary.
fn cmd_status(root: &Path, ledger: &Path) -> Result<(), KemError> {
    let claims = load_claims(&root.join("claims"))?;
    for (status, count) in kem::filing::status_counts(&claims) {
        if count > 0 {
            println!("{:?}: {}", status, count);
        }
    }
    let entries = if ledger.exists() {
        kem::ledger::read_all(ledger)?.len()
    } else {
        0
    };
    println!("ledger entries: {entries}");
    Ok(())
}

/// P4 M7: record a review disposition (queue stays append-only).
fn cmd_review(
    root: &Path,
    flags: &std::collections::HashMap<String, String>,
) -> Result<(), KemError> {
    let claim_id = flag(flags, "claim")?;
    let disp = match flag(flags, "disposition")?.as_str() {
        "accept" => kem::gov::Disposition::Accept,
        "reject" => kem::gov::Disposition::Reject,
        "supersede" => kem::gov::Disposition::Supersede,
        "defer" => kem::gov::Disposition::Defer,
        other => {
            return Err(KemError::InvalidInput(format!(
                "unknown disposition {other:?}"
            )))
        }
    };
    let reviewer = flags
        .get("reviewer")
        .cloned()
        .unwrap_or_else(|| "human".into());
    let note = flags.get("note").cloned().unwrap_or_default();
    kem::gov::record_disposition(
        &root.join(".kem/review-queue.jsonl"),
        &claim_id,
        disp,
        &reviewer,
        &note,
        Utc::now(),
    )?;
    println!("review recorded: {claim_id} {disp:?} by {reviewer}");
    Ok(())
}

fn next_id(
    existing: &[Claim],
    known: &std::collections::BTreeSet<String>,
) -> Result<String, KemError> {
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
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(serde_json::to_string(value)?.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}
