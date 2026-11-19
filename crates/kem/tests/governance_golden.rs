#![forbid(unsafe_code)]
//! O6 — 決策治理縱切片黃金場景（5 情境，隔離暫存庫端到端）。
//!
//! 每情境量測 wall-clock 並驗證功能結果；5/5 PASS 且全部 <10 分鐘（M1）。
//! 離線確定性：vet2 不在此測試（網路相依）——由 benchmark live 量測覆蓋。

use kem::evolve;
use kem::filing;
use kem::gov::{self, Disposition};
use kem::ledger::LedgerEntry;
use kem::package;
use kem::schema::{Claim, ClaimStatus, ClaimType, Evidence, KemError, Track};
use kem::vetting::{Outcome, VetContext, record_verdict, stage_claim, vet_t1};
use std::time::Instant;

fn bench_claim(id: &str, title: &str, body: &str, evidence_rel: &str) -> Claim {
    Claim {
        id: id.into(),
        claim_type: ClaimType::Decision,
        status: ClaimStatus::Published,
        track: Track::B,
        confidence: 0.9,
        valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        valid_to: None,
        recorded_at: chrono::Utc::now(),
        supersedes: vec![],
        superseded_by: None,
        evidence: vec![Evidence {
            path: evidence_rel.into(),
            rev: "bench".into(),
            quote: body.into(),
        }],
        topics: vec!["golden".into()],
        source: "raw/golden/2026-11-18-seed.md".into(),
        title: title.into(),
        body: body.into(),
    }
}

fn stage_and_record(
    root: &std::path::Path,
    ledger: &std::path::Path,
    claim: &Claim,
) -> Result<Outcome, KemError> {
    let ctx = VetContext::new(root.to_path_buf(), Vec::new());
    let verdict = vet_t1(claim, &ctx);
    let outcome = verdict.outcome;
    if outcome != Outcome::Accept {
        eprintln!(
            "T1 {:?} for {}: {}",
            outcome,
            claim.id,
            verdict.reasons.join("; ")
        );
    }
    record_verdict(
        ledger,
        "kem-golden",
        &claim.id,
        &verdict,
        chrono::Utc::now(),
    )?;
    if outcome == Outcome::Accept {
        stage_claim(&root.join("claims"), claim)?;
    }
    Ok(outcome)
}

fn write_evidence(root: &std::path::Path, rel: &str, quote: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, quote).unwrap();
}

/// S1 — 快樂路徑：逐字證據 → T1 accept → package → defer → finalize。
#[test]
fn s1_happy_path_accept_package_review() {
    let t0 = Instant::now();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".kem")).unwrap();
    let ledger = root.join(".kem/ledger.jsonl");

    write_evidence(root, "docs/golden.md", "Golden: the decision slice works.");
    let claim = bench_claim(
        "claim-20261118-001",
        "治理切片 S1",
        "Golden: the decision slice works.",
        "docs/golden.md",
    );
    let outcome = stage_and_record(root, &ledger, &claim).unwrap();
    assert_eq!(outcome, Outcome::Accept);

    let claims = filing::load_claims(&root.join("claims")).unwrap();
    let entries: Vec<LedgerEntry> = kem::ledger::read_all(&ledger).unwrap();
    let pkg =
        package::generate(&claims, &entries, &[], &claim.id, root, chrono::Utc::now()).unwrap();
    // Honest tiering: no vet2 verdict yet -> package recommends vet2 first
    // (the accept recommendation with ensemble support is covered by the
    // live benchmark). This is the L-ladder working, not a defect.
    assert!(
        pkg.recommendation.contains("vet2"),
        "{}",
        pkg.recommendation
    );
    let path = package::write_package(root, &pkg).unwrap();
    assert!(path.exists());

    gov::record_disposition(
        &root.join(".kem/review-queue.jsonl"),
        &claim.id,
        Disposition::Accept,
        "golden",
        "S1 happy path",
        &pkg.package_sha,
        chrono::Utc::now(),
    )
    .unwrap();
    let views = filing::load_claims(&root.join("claims")).unwrap();
    assert_eq!(views.len(), 1);
    assert!(
        t0.elapsed().as_secs() < 600,
        "M1: slice must stay under 10 min"
    );
}

/// S2 — fail-closed：捏造引證 → T1 拒絕、不入庫、佇列留痕。
#[test]
fn s2_fabricated_quote_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".kem")).unwrap();
    let ledger = root.join(".kem/ledger.jsonl");

    let mut claim = bench_claim(
        "claim-20261118-002",
        "治理切片 S2（捏造）",
        "Golden: fabricated body.",
        "docs/nonexistent.md",
    );
    claim.evidence[0].quote = "a quote that exists nowhere".into();

    let outcome = stage_and_record(root, &ledger, &claim).unwrap();
    assert_eq!(outcome, Outcome::Reject);
    // fail-closed：claims/ 不得有此主張
    let filed = filing::load_claims(&root.join("claims")).unwrap();
    assert!(filed.iter().all(|c| c.id != claim.id));
    // 帳本留痕（Vet 拒絕）
    let entries = kem::ledger::read_all(&ledger).unwrap();
    assert!(entries.iter().any(|e| e.claim_id == claim.id));
}

/// S3 — 批次內重複：1 accept＋1 review（Fix #2 迴歸護欄）。
#[test]
fn s3_batch_internal_duplicate_detected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".kem")).unwrap();
    write_evidence(root, "docs/golden.md", "Golden: duplicate detection.");

    let a = bench_claim(
        "claim-20261118-003",
        "治理切片 S3-A",
        "Golden: duplicate detection.",
        "docs/golden.md",
    );
    let mut b = a.clone();
    b.id = "claim-20261118-004".into();
    b.title = "治理切片 S3-B（複本）".into();

    let ctx = VetContext::new(root.to_path_buf(), Vec::new());
    let va = vet_t1(&a, &ctx);
    assert_eq!(va.outcome, Outcome::Accept);
    stage_claim(&root.join("claims"), &a).unwrap();

    // 批次內可見性：第二份對照含 A 的既有清單
    let ctx2 = VetContext::new(root.to_path_buf(), vec![a.clone()]);
    let vb = vet_t1(&b, &ctx2);
    assert_eq!(vb.outcome, Outcome::Review, "dup must route to review");
}

/// S4 — 結構取代：Track A 繼任者 → 前任 superseded 降級可檢索。
#[test]
fn s4_supersede_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".kem")).unwrap();
    let ledger = root.join(".kem/ledger.jsonl");
    write_evidence(root, "docs/golden.md", "Golden: supersede lifecycle v1.");

    let old = bench_claim(
        "claim-20261118-005",
        "治理切片 S4-舊",
        "Golden: supersede lifecycle v1.",
        "docs/golden.md",
    );
    let outcome = stage_and_record(root, &ledger, &old).unwrap();
    assert_eq!(outcome, Outcome::Accept);

    let mut successor = old.clone();
    successor.id = "claim-20261118-006".into();
    successor.track = Track::A;
    successor.title = "治理切片 S4-新".into();
    successor.body = "Golden: supersede lifecycle v2.".into();
    let mut claims = filing::load_claims(&root.join("claims")).unwrap();
    evolve::supersede(
        root,
        &ledger,
        &mut claims,
        successor,
        "claim-20261118-005",
        chrono::Utc::now(),
    )
    .unwrap();

    let filed = filing::load_claims(&root.join("claims")).unwrap();
    let o = filed.iter().find(|c| c.id == "claim-20261118-005").unwrap();
    let n = filed.iter().find(|c| c.id == "claim-20261118-006").unwrap();
    assert_eq!(o.status, ClaimStatus::Superseded);
    assert_eq!(o.superseded_by.as_deref(), Some("claim-20261118-006"));
    assert!(o.status.retrievable(), "降級可檢索（取代不刪除）");
    assert_eq!(n.supersedes, vec!["claim-20261118-005"]);
}

/// S5 — 決策包＋defer 处置閉環：package sha 進佇列紀錄。
#[test]
fn s5_package_and_defer_disposition() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".kem")).unwrap();
    let ledger = root.join(".kem/ledger.jsonl");
    write_evidence(
        root,
        "docs/golden.md",
        "Golden: package and defer disposition.",
    );

    let claim = bench_claim(
        "claim-20261118-007",
        "治理切片 S5",
        "Golden: package and defer disposition.",
        "docs/golden.md",
    );
    let outcome = stage_and_record(root, &ledger, &claim).unwrap();
    assert_eq!(outcome, Outcome::Accept);

    let claims = filing::load_claims(&root.join("claims")).unwrap();
    let entries = kem::ledger::read_all(&ledger).unwrap();
    let pkg =
        package::generate(&claims, &entries, &[], &claim.id, root, chrono::Utc::now()).unwrap();
    let path = package::write_package(root, &pkg).unwrap();
    assert!(path.exists());

    let queue = root.join(".kem/review-queue.jsonl");
    gov::record_disposition(
        &queue,
        &claim.id,
        Disposition::Defer,
        "golden",
        "S5 defer",
        &pkg.package_sha,
        chrono::Utc::now(),
    )
    .unwrap();
    // package_sha 進紀錄（可追溯）
    let qtext = std::fs::read_to_string(&queue).unwrap();
    assert!(qtext.contains(&pkg.package_sha));
    // pending 計數歸零（resolved 行不計）
    let pending = gov::pending_queue(&queue).unwrap();
    assert!(pending.is_empty(), "resolved 行不得計入 pending");
}

/// 全情境總時量測（M1）：五情境合計 <10 分鐘。
#[test]
fn all_scenarios_within_target() {
    let t0 = Instant::now();
    // 逐一執行（同進程，量測合計）
    s1_happy_path_accept_package_review();
    s2_fabricated_quote_rejected();
    s3_batch_internal_duplicate_detected();
    s4_supersede_lifecycle();
    s5_package_and_defer_disposition();
    let elapsed = t0.elapsed();
    assert!(
        elapsed.as_millis() < 600_000,
        "M1: all scenarios must complete within 10 min (took {:?})",
        elapsed
    );
}
