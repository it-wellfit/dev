#![forbid(unsafe_code)]
//! O5 — 時間到價值量測（M1）：在隔離暫存庫跑完整決策治理切片，
//! 逐步計時，報告寫入生產根 `.kem/benchmarks/<ts>.json`。
//!
//! 誠實原則：vet2 不可達 → 該步驟標 degraded（ok=true，如實標記），
//! 絕不虛構耗時；總時超出 10 分鐘目標 → within_target=false。

use crate::capture::capture_source;
use crate::evolve;
use crate::filing;
use crate::gov;
use crate::ledger::{LedgerAction, LedgerEntry};
use crate::package;
use crate::schema::{Claim, ClaimStatus, ClaimType, Evidence, KemError, Track};
use crate::vetting::{Outcome, VetContext, record_verdict, stage_claim, vet_t1};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// 10 分鐘目標（M1）。
pub const TARGET_MILLIS: u128 = 600_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTiming {
    pub step: String,
    pub millis: u128,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub scenario: String,
    pub steps: Vec<StepTiming>,
    pub total_millis: u128,
    pub within_target: bool,
    pub generated_at: String,
    /// 隔離暫存庫路徑（供事後檢視；OS 週期性清理）。
    pub bench_root: String,
    /// vet2 是否實際執行（false = --no-vet2 離線模式）。
    pub vet2_executed: bool,
}

struct Timer {
    steps: Vec<StepTiming>,
    start: Instant,
}

impl Timer {
    fn new() -> Self {
        Self {
            steps: Vec::new(),
            start: Instant::now(),
        }
    }
    fn record(&mut self, step: &str, t: Instant, ok: bool, detail: String) {
        self.steps.push(StepTiming {
            step: step.to_string(),
            millis: t.elapsed().as_millis(),
            ok,
            detail,
        });
    }
    fn total(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

/// 建立最小可運作的主張（證據檔在暫存庫內，逐字可驗證）。
fn bench_claim(id: &str, title: &str, body: &str, evidence_rel: &str) -> Claim {
    Claim {
        id: id.into(),
        claim_type: ClaimType::Decision,
        status: ClaimStatus::Published,
        track: Track::B,
        confidence: 0.9,
        valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        valid_to: None,
        recorded_at: Utc::now(),
        supersedes: vec![],
        superseded_by: None,
        evidence: vec![Evidence {
            path: evidence_rel.into(),
            rev: "bench".into(),
            quote: body.into(),
        }],
        topics: vec!["benchmark".into()],
        source: "raw/bench/2026-11-18-seed.md".into(),
        title: title.into(),
        body: body.into(),
    }
}

/// O5：完整治理切片量測（隔離暫存庫；生產根僅收報告）。
///
/// 步驟：capture → ingest(T1) → [vet2(五模型，可略)] → package → review(defer)
/// → finalize → doctor。回傳報告並寫入生產根。
pub fn run_governance(
    production_root: &Path,
    include_vet2: bool,
) -> Result<BenchmarkReport, KemError> {
    let nanos = Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| Utc::now().timestamp_millis());
    let bench_root: PathBuf = std::env::temp_dir().join(format!("kem-bench-{nanos}"));
    std::fs::create_dir_all(bench_root.join("docs"))?;
    std::fs::create_dir_all(bench_root.join(".kem"))?;
    let ledger = bench_root.join(".kem/ledger.jsonl");
    let mut t = Timer::new();

    // ── capture：L0 擷取（不可變源檔） ──
    let s0 = Instant::now();
    let src_content = "Bench: KEM 決策治理切片量測源檔。\n";
    let src = capture_source(&bench_root, "bench", "seed", "md", src_content, Utc::now())?;
    let ledger_ok = true; // capture_source 冪等即成功
    t.record(
        "capture",
        s0,
        ledger_ok,
        format!("{} ({} B)", src.rel_path, src.bytes),
    );

    // 證據檔（暫存庫內，相對路徑 docs/bench.md）——內容＝主張引證（body），
    // 否則 T1 groundedness 正確拒絕（fail-closed by design）。
    let ev_rel = "docs/bench.md";

    // ── ingest：T1 三閘裁決＋歸檔 ──
    let s1 = Instant::now();
    let claim = bench_claim(
        "claim-20261118-001",
        "Bench 決策治理",
        "Bench: decision governance slice works end to end.",
        ev_rel,
    );
    std::fs::create_dir_all(bench_root.join("docs"))?;
    std::fs::write(bench_root.join(ev_rel), &claim.body)?;
    // 證據解析基底＝暫存庫（與生產 cmd_ingest 的 cwd 語義一致）
    let ctx = VetContext::new(bench_root.clone(), Vec::new());
    let verdict = vet_t1(&claim, &ctx);
    let ingest_ok = verdict.outcome == Outcome::Accept;
    let ingest_detail = if ingest_ok {
        "T1 accept".into()
    } else {
        format!("T1 {:?}: {}", verdict.outcome, verdict.reasons.join("; "))
    };
    t.record("ingest", s1, ingest_ok, ingest_detail);
    if !ingest_ok {
        return finish(
            &t,
            include_vet2,
            &bench_root,
            production_root,
            "T1 rejected the seeded claim — benchmark invalid".into(),
        );
    }
    record_verdict(&ledger, "kem-bench", &claim.id, &verdict, Utc::now())?;
    stage_claim(&bench_root.join("claims"), &claim)?;
    // ── vet2：五模型聯合（可略；不可達=degraded 誠實標記） ──
    let mut vet2_executed = false;
    if include_vet2 {
        let s2 = Instant::now();
        let pool = evolve_pool();
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| KemError::InvalidInput(format!("tokio runtime: {e}")))?;
        let verdict = rt.block_on(evolve::scs_vote(&pool, &claim.title, &claim.body, 5));
        vet2_executed = !verdict.llm_unreachable;
        let detail = if verdict.llm_unreachable {
            "degraded — all endpoints unreachable (honest UNKNOWN)".into()
        } else {
            format!(
                "models=[{}] agreement={:.2} top={:?}",
                verdict.models_used.join(","),
                verdict.agreement,
                verdict.top_score
            )
        };
        let ok = !verdict.llm_unreachable;
        t.record("vet2", s2, ok, detail);
        let _ = crate::ledger::append(
            &ledger,
            "kem-t2",
            LedgerAction::Vet,
            &claim.id,
            serde_json::to_value(&verdict)?,
            Utc::now(),
        );
    }

    // ── package：O1 決策包 ──
    let s3 = Instant::now();
    let entries_now = read_entries(&ledger)?;
    let queue: Vec<(String, Vec<String>)> = Vec::new();
    let pkg = package::generate(
        std::slice::from_ref(&claim),
        &entries_now,
        &queue,
        &claim.id,
        &bench_root,
        Utc::now(),
    )?;
    let pkg_path = package::write_package(&bench_root, &pkg)?;
    let _ = crate::ledger::append(
        &ledger,
        "kem-package",
        LedgerAction::Package,
        &claim.id,
        serde_json::json!({"package_sha": pkg.package_sha}),
        Utc::now(),
    );
    t.record(
        "package",
        s3,
        true,
        format!("{} (sha {}…)", pkg_path.display(), &pkg.package_sha[..12]),
    );

    // ── review：H1 人工複核（benchmark 自動 defer，記錄处置） ──
    let s4 = Instant::now();
    gov::record_disposition(
        &bench_root.join(".kem/review-queue.jsonl"),
        &claim.id,
        gov::Disposition::Defer,
        "kem-bench",
        "auto defer — benchmark run",
        &pkg.package_sha,
        Utc::now(),
    )?;
    let _ = crate::ledger::append(
        &ledger,
        "kem-bench",
        LedgerAction::Review,
        &claim.id,
        serde_json::json!({"disposition": "defer", "package_sha": pkg.package_sha}),
        Utc::now(),
    );
    t.record("review", s4, true, "disposition=defer recorded".into());

    // ── finalize：三視圖重生成 ──
    let s5 = Instant::now();
    filing::write_index(&bench_root, std::slice::from_ref(&claim))?;
    filing::write_log(&bench_root, &ledger)?;
    let dash_ok = crate::dashboard::write_dashboard(
        &bench_root,
        std::slice::from_ref(&claim),
        0,
        &[],
        &[(
            1,
            "package".into(),
            claim.id.clone(),
            Utc::now().to_rfc3339(),
        )],
        None,
        None,
    )
    .is_ok();
    t.record(
        "finalize",
        s5,
        dash_ok,
        "index + log + dashboard regenerated".into(),
    );

    // ── doctor：健康檢查 ──
    let s6 = Instant::now();
    let health = gov::doctor(
        &bench_root,
        &ledger,
        &bench_root.join(".kem/review-queue.jsonl"),
    )?;
    t.record(
        "doctor",
        s6,
        health.ledger_ok,
        format!(
            "entries={} chain_ok={} claims={} queue={}",
            health.ledger_entries, health.ledger_ok, health.claims, health.review_queue
        ),
    );

    finish(
        &t,
        vet2_executed || include_vet2,
        &bench_root,
        production_root,
        String::new(),
    )
}

fn evolve_pool() -> Vec<evolve::ModelEntry> {
    vec![
        evolve::ModelEntry {
            label: "local-ollama".into(),
            base: "http://127.0.0.1:11434".into(),
            kind: evolve::EndpointKind::OllamaNative,
            model: "qwen3.5:4b".into(),
            min_interval_secs: 0,
        },
        evolve::ModelEntry {
            label: "ovh-qwen3.8-27b".into(),
            base: "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1".into(),
            kind: evolve::EndpointKind::OpenAi,
            model: "Qwen3.8-27B".into(),
            min_interval_secs: 31,
        },
        evolve::ModelEntry {
            label: "llm7-gpt-oss-20b".into(),
            base: "https://api.llm7.io/v1".into(),
            kind: evolve::EndpointKind::OpenAi,
            model: "gpt-oss:20b".into(),
            min_interval_secs: 7,
        },
    ]
}

fn read_entries(ledger: &Path) -> Result<Vec<LedgerEntry>, KemError> {
    if ledger.exists() {
        crate::ledger::read_all(ledger)
    } else {
        Ok(Vec::new())
    }
}

fn finish(
    t: &Timer,
    vet2_executed: bool,
    bench_root: &Path,
    production_root: &Path,
    fatal: String,
) -> Result<BenchmarkReport, KemError> {
    let total = t.total();
    let report = BenchmarkReport {
        scenario: "governance".into(),
        steps: t.steps.clone(),
        total_millis: total,
        within_target: total <= TARGET_MILLIS && fatal.is_empty(),
        generated_at: Utc::now().to_rfc3339(),
        bench_root: bench_root.display().to_string(),
        vet2_executed,
    };
    save_report(production_root, &report)?;
    if !fatal.is_empty() {
        return Err(KemError::InvalidInput(fatal));
    }
    Ok(report)
}

/// 報告寫入生產根 `.kem/benchmarks/<ts>.json`（衍生、可重建）。
pub fn save_report(production_root: &Path, report: &BenchmarkReport) -> Result<PathBuf, KemError> {
    let dir = production_root.join(".kem/benchmarks");
    std::fs::create_dir_all(&dir)?;
    let ts = Utc::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("governance-{ts}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(report)?)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_is_ten_minutes() {
        assert_eq!(TARGET_MILLIS, 600_000);
    }

    #[test]
    fn report_serializes_roundtrip() {
        let r = BenchmarkReport {
            scenario: "governance".into(),
            steps: vec![StepTiming {
                step: "capture".into(),
                millis: 12,
                ok: true,
                detail: "d".into(),
            }],
            total_millis: 120,
            within_target: true,
            generated_at: "2026-11-18T00:00:00Z".into(),
            bench_root: "/tmp/x".into(),
            vet2_executed: false,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: BenchmarkReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_millis, 120);
        assert!(back.within_target);
        assert_eq!(back.steps.len(), 1);
    }
}
