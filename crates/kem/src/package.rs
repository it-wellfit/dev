#![forbid(unsafe_code)]
//! O1 — 決策包（Decision Package）：證據＋選項＋建議＋風險的結構化決策工件。
//!
//! 產出 `knowledge/.kem/packages/<id>.md`，供 H1 人工複核（M10：人工 P50 <2 分鐘）
//! 與 M11 審計演練（<5 分鐘完整重建、不執行程式碼）。資料全部來自
//! claims SSOT＋帳本（vet2 聯合裁決）＋審查佇列——零捏造。

use crate::capture::sha256_hex;
use crate::gov;
use crate::ledger::LedgerEntry;
use crate::schema::{Claim, KemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// 從帳本提取的聯合裁決摘要（最新一筆 kem-t2 Vet 條目）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnsembleSummary {
    pub models_used: Vec<String>,
    pub agreement: f64,
    pub top_score: Option<u8>,
    /// (model, durability) 逐票歸因。
    pub votes: Vec<(String, u8)>,
    /// 裁決時間（RFC3339）。
    pub verdict_at: String,
}

/// 風險登記（決策包風險節）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskRegister {
    /// L1–L5 lint findings（rule: detail）。
    pub lint_findings: Vec<String>,
    /// 審查佇列中此主張的未解原因。
    pub queue_reasons: Vec<String>,
    /// 信心 < 0.45（陳舊，需再確認）。
    pub stale_confidence: bool,
    /// 聯合一致率 < 0.60（分歧可見）。
    pub low_agreement: bool,
    /// 逐字驗證失敗的證據筆數。
    pub unverified_evidence: usize,
    /// 補充說明。
    pub notes: Vec<String>,
}

/// 處置選項（四選一，依據資料驅動）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOption {
    /// accept | reject | supersede | defer
    pub disposition: String,
    /// 資料驅動的依據說明。
    pub rationale: String,
}

/// 證據行（含當下逐字驗證結果）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLine {
    pub path: String,
    pub rev: String,
    pub quote: String,
    /// 生成當下：來源檔仍包含逐字引證？
    pub verified: bool,
}

/// 結構化決策包。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPackage {
    pub claim_id: String,
    pub title: String,
    pub status: String,
    pub confidence: f64,
    pub evidence: Vec<EvidenceLine>,
    pub ensemble: Option<EnsembleSummary>,
    pub options: Vec<DecisionOption>,
    /// 資料驅動的建議（含理由）。
    pub recommendation: String,
    pub risk: RiskRegister,
    pub generated_at: String,
    /// sha256（涵蓋除自身外全部欄位的 JSON 正規序列化）。
    pub package_sha: String,
}

/// 從帳本提取指定主張的最新聯合裁決（actor=kem-t2 且 detail 含 models_used）。
pub fn extract_ensemble(
    ledger_entries: &[LedgerEntry],
    claim_id: &str,
) -> Option<EnsembleSummary> {
    let e = ledger_entries
        .iter()
        .rev()
        .find(|e| e.actor == "kem-t2" && e.claim_id == claim_id)?;
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
    if models.is_empty() {
        return None; // UNKNOWN 條目（全端點不可達）——誠實視為無裁決
    }
    let agreement = e.detail.get("agreement").and_then(|a| a.as_f64())?;
    let top_score = e
        .detail
        .get("top_score")
        .and_then(|t| t.as_u64())
        .map(|t| t as u8);
    let votes: Vec<(String, u8)> = e
        .detail
        .get("votes")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| {
                    Some((
                        v.get("model")?.as_str()?.to_string(),
                        v.get("score")?.as_u64()? as u8,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    Some(EnsembleSummary {
        models_used: models,
        agreement,
        top_score,
        votes,
        verdict_at: e.ts.to_rfc3339(),
    })
}

/// 生成決策包（O1 核心）。`evidence_root`：證據路徑的解析基底（工作區根）。
pub fn generate(
    claims: &[Claim],
    ledger_entries: &[LedgerEntry],
    queue_pending: &[(String, Vec<String>)],
    claim_id: &str,
    evidence_root: &Path,
    now: DateTime<Utc>,
) -> Result<DecisionPackage, KemError> {
    let claim = claims
        .iter()
        .find(|c| c.id == claim_id)
        .ok_or_else(|| KemError::NotFound(claim_id.to_string()))?;

    // ── 證據逐字驗證（生成當下） ──
    let mut evidence = Vec::new();
    let mut unverified = 0usize;
    for ev in &claim.evidence {
        let verified = std::fs::read_to_string(evidence_root.join(&ev.path))
            .map(|content| content.contains(ev.quote.as_str()))
            .unwrap_or(false);
        if !verified {
            unverified += 1;
        }
        evidence.push(EvidenceLine {
            path: ev.path.clone(),
            rev: ev.rev.clone(),
            quote: ev.quote.clone(),
            verified,
        });
    }

    // ── 聯合裁決 ──
    let ensemble = extract_ensemble(ledger_entries, claim_id);

    // ── 風險登記 ──
    let lint_findings: Vec<String> = gov::lint(claims, now)
        .iter()
        .filter(|f| f.claim_id == claim_id)
        .map(|f| format!("{}: {}", f.rule, f.detail))
        .collect();
    let queue_reasons: Vec<String> = queue_pending
        .iter()
        .filter(|(id, _)| id == claim_id)
        .flat_map(|(_, reasons)| reasons.clone())
        .collect();
    let stale_confidence = claim.confidence < 0.45;
    let low_agreement = ensemble
        .as_ref()
        .map(|e| e.agreement < 0.60)
        .unwrap_or(false);
    let mut risk = RiskRegister {
        lint_findings,
        queue_reasons,
        stale_confidence,
        low_agreement,
        unverified_evidence: unverified,
        notes: Vec::new(),
    };
    if unverified > 0 {
        risk.notes
            .push(format!("證據 {unverified} 筆逐字驗證失敗——先執行 kem sync 診斷"));
    }

    // ── 選項（四選一全列，依據資料驅動） ──
    let has_risk = !risk.lint_findings.is_empty()
        || !risk.queue_reasons.is_empty()
        || risk.stale_confidence
        || risk.low_agreement
        || unverified > 0;
    let top = ensemble.as_ref().and_then(|e| e.top_score);
    let options = vec![
        DecisionOption {
            disposition: "accept".into(),
            rationale: if has_risk {
                "存在風險訊號——不建議".into()
            } else if matches!(top, Some(t) if t >= 3) {
                "聯合多數支持且零風險訊號".into()
            } else {
                "零風險訊號（尚無聯合裁決）".into()
            },
        },
        DecisionOption {
            disposition: "reject".into(),
            rationale: if risk.lint_findings.is_empty() {
                "無治理缺陷——僅在內容本身錯誤時選擇".into()
            } else {
                "存在治理缺陷（見風險登記）".into()
            },
        },
        DecisionOption {
            disposition: "supersede".into(),
            rationale: if stale_confidence {
                "信心陳舊（<0.45），建議結構性取代".into()
            } else {
                "內容仍有效——僅在有新資訊時選擇（Track A）".into()
            },
        },
        DecisionOption {
            disposition: "defer".into(),
            rationale: if low_agreement {
                format!(
                    "五模型分歧可見（agreement {:.2}），待更多證據",
                    ensemble.as_ref().map(|e| e.agreement).unwrap_or(0.0)
                )
            } else if ensemble.is_none() {
                "尚無聯合裁決——先執行 kem vet2".into()
            } else {
                "證據充分——非必要選項".into()
            },
        },
    ];

    // ── 建議（優先序：缺陷修復 > 分歧 > 無裁決 > 聯合支持 > 陳舊取代） ──
    let recommendation = if !risk.lint_findings.is_empty() {
        format!(
            "先修復 {} 項 lint findings（見風險登記），再裁決",
            risk.lint_findings.len()
        )
    } else if unverified > 0 {
        format!("{unverified} 筆證據逐字驗證失敗——先執行 kem sync 診斷漂移，再裁決")
    } else if low_agreement {
        format!(
            "defer：五模型分歧可見（agreement {:.2} ≤ 0.60 地板）——人工複核",
            ensemble.as_ref().map(|e| e.agreement).unwrap_or(0.0)
        )
    } else if ensemble.is_none() {
        "先執行 kem vet2 取得聯合裁決，再裁決".into()
    } else if stale_confidence {
        format!(
            "supersede：信心 {:.2} 陳舊（<0.45）——結構性取代並更新內容",
            claim.confidence
        )
    } else {
        format!(
            "accept：聯合多數支持（durability={:?}, agreement {:.2}）且零風險訊號",
            top, ensemble.as_ref().map(|e| e.agreement).unwrap_or(0.0)
        )
    };

    let mut pkg = DecisionPackage {
        claim_id: claim.id.clone(),
        title: claim.title.clone(),
        status: format!("{:?}", claim.status).to_lowercase(),
        confidence: claim.confidence,
        evidence,
        ensemble,
        options,
        recommendation,
        risk,
        generated_at: now.to_rfc3339(),
        package_sha: String::new(),
    };
    // sha 涵蓋除自身外全部欄位（正規 JSON 序列化）
    pkg.package_sha = sha256_hex(
        serde_json::to_string(&pkg)
            .map_err(|e| KemError::InvalidInput(format!("package serialize: {e}")))?
            .as_bytes(),
    );
    Ok(pkg)
}

/// Markdown 渲染（人工可讀；M10 決策包）。
pub fn render_markdown(pkg: &DecisionPackage) -> String {
    let mut m = String::with_capacity(4 * 1024);
    let _ = writeln!(m, "# 決策包 — {}", pkg.claim_id);
    let _ = writeln!(
        m,
        "> 生成：{}｜sha256：{}｜狀態：{}｜信心 {:.2}",
        pkg.generated_at, pkg.package_sha, pkg.status, pkg.confidence
    );
    let _ = writeln!(m, "\n## 1. 主張\n\n**{}**\n\n{}", pkg.title, pkg.title);
    let _ = writeln!(m, "\n## 2. 證據（逐字）\n");
    for ev in &pkg.evidence {
        let _ = writeln!(
            m,
            "- [{}] `{}` @ {}：{}",
            if ev.verified { "✓" } else { "✗" },
            ev.path,
            ev.rev,
            ev.quote
        );
    }
    let _ = writeln!(m, "\n## 3. 聯合裁決（五模型）\n");
    match &pkg.ensemble {
        Some(e) => {
            let _ = writeln!(
                m,
                "- 模型：{}",
                e.models_used.join("、")
            );
            let _ = writeln!(
                m,
                "- 一致率：{:.2}｜多數 durability：{}",
                e.agreement,
                e.top_score.map(|t| t.to_string()).unwrap_or_else(|| "—".into())
            );
            for (model, score) in &e.votes {
                let _ = writeln!(m, "  - {model} → durability={score}");
            }
            let _ = writeln!(m, "- 裁決時間：{}", e.verdict_at);
        }
        None => {
            m.push_str("- 尚無聯合裁決——執行 `kem vet2 --claim <id>`\n");
        }
    }
    let _ = writeln!(m, "\n## 4. 選項\n");
    for o in &pkg.options {
        let _ = writeln!(m, "- **{}**：{}", o.disposition, o.rationale);
    }
    let _ = writeln!(m, "\n## 5. 建議\n\n{}\n", pkg.recommendation);
    let _ = writeln!(m, "## 6. 風險登記\n");
    if pkg.risk.lint_findings.is_empty()
        && pkg.risk.queue_reasons.is_empty()
        && pkg.risk.notes.is_empty()
        && !pkg.risk.stale_confidence
        && !pkg.risk.low_agreement
        && pkg.risk.unverified_evidence == 0
    {
        m.push_str("- 零風險訊號\n");
    } else {
        for f in &pkg.risk.lint_findings {
            let _ = writeln!(m, "- lint：{f}");
        }
        for r in &pkg.risk.queue_reasons {
            let _ = writeln!(m, "- 佇列：{r}");
        }
        if pkg.risk.stale_confidence {
            m.push_str("- 信心陳舊（<0.45）\n");
        }
        if pkg.risk.low_agreement {
            m.push_str("- 聯合一致率低於地板\n");
        }
        if pkg.risk.unverified_evidence > 0 {
            let _ = writeln!(m, "- 逐字驗證失敗：{} 筆", pkg.risk.unverified_evidence);
        }
        for n in &pkg.risk.notes {
            let _ = writeln!(m, "- {n}");
        }
    }
    m
}

/// 寫入 `knowledge/.kem/packages/<id>.md`（衍生、可重建）。
pub fn write_package(root: &Path, pkg: &DecisionPackage) -> Result<PathBuf, KemError> {
    let dir = root.join(".kem/packages");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.md", pkg.claim_id));
    std::fs::write(&path, render_markdown(pkg))?;
    Ok(path)
}

/// 最新決策包條目（供儀表板治理面板）： (claim_id, sha, 生成時間)。
pub fn latest_package(ledger_entries: &[LedgerEntry]) -> Option<(String, String, String)> {
    let e = ledger_entries.iter().rev().find(|e| {
        e.actor == "kem-package" && e.claim_id == e.detail.get("claim_id").and_then(|c| c.as_str()).unwrap_or_default()
            || (e.actor == "kem-package" && !e.claim_id.is_empty())
    })?;
    let sha = e
        .detail
        .get("package_sha")
        .and_then(|s| s.as_str())
        .unwrap_or_default()
        .to_string();
    Some((e.claim_id.clone(), sha, e.ts.to_rfc3339()))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::LedgerAction;
    use crate::schema::{ClaimType, Evidence, Track};
    use chrono::TimeZone;

    fn claim(id: &str, conf: f64) -> Claim {
        Claim {
            id: id.into(),
            claim_type: ClaimType::Decision,
            status: crate::schema::ClaimStatus::Published,
            track: Track::B,
            confidence: conf,
            valid_from: chrono::NaiveDate::from_ymd_opt(2026, 11, 17).unwrap(),
            valid_to: None,
            recorded_at: Utc.with_ymd_and_hms(2026, 11, 17, 0, 0, 0).unwrap(),
            supersedes: vec![],
            superseded_by: None,
            evidence: vec![Evidence {
                path: "evidence.md".into(),
                rev: "working-tree".into(),
                quote: "the verbatim quote".into(),
            }],
            topics: vec!["測試".into()],
            source: "raw/t/2026-11-17-a.md".into(),
            title: "決策包測試".into(),
            body: "b".into(),
        }
    }

    fn ledger_with_vet2(claim_id: &str, agreement: f64, top: u8) -> Vec<LedgerEntry> {
        vec![LedgerEntry {
            seq: 1,
            ts: Utc.with_ymd_and_hms(2026, 11, 17, 0, 0, 0).unwrap(),
            actor: "kem-t2".into(),
            action: LedgerAction::Vet,
            claim_id: claim_id.into(),
            detail: serde_json::json!({
                "models_used": ["m1", "m2"],
                "agreement": agreement,
                "top_score": top,
                "votes": [{"model": "m1", "score": top}, {"model": "m2", "score": top}]
            }),
            prev_hash: "GENESIS".into(),
            hash: "h".into(),
        }]
    }

    #[test]
    fn extract_ensemble_reads_latest_kemt2_entry() {
        let entries = ledger_with_vet2("claim-20261117-001", 0.8, 4);
        let e = extract_ensemble(&entries, "claim-20261117-001").expect("ensemble");
        assert_eq!(e.models_used, vec!["m1", "m2"]);
        assert!((e.agreement - 0.8).abs() < 1e-9);
        assert_eq!(e.top_score, Some(4));
        assert_eq!(e.votes.len(), 2);
        // UNKNOWN 條目（無 models_used）→ None（誠實）
        let empty = vec![LedgerEntry {
            seq: 2,
            ts: Utc.with_ymd_and_hms(2026, 11, 17, 1, 0, 0).unwrap(),
            actor: "kem-t2".into(),
            action: LedgerAction::Vet,
            claim_id: "claim-20261117-001".into(),
            detail: serde_json::json!({"status": "unknown"}),
            prev_hash: "h".into(),
            hash: "h2".into(),
        }];
        assert!(extract_ensemble(&empty, "claim-20261117-001").is_none());
    }

    #[test]
    fn generate_recommends_accept_on_majority_support() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.md"), "the verbatim quote").unwrap();
        let claims = vec![claim("claim-20261117-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261117-001", 0.8, 4);
        let pkg = generate(&claims, &entries, &[], "claim-20261117-001", dir.path(), Utc::now())
            .unwrap();
        assert!(pkg.recommendation.starts_with("accept"), "{}", pkg.recommendation);
        assert!(pkg.evidence[0].verified);
        assert!(!pkg.risk.stale_confidence);
        assert!(!pkg.package_sha.is_empty());
    }

    #[test]
    fn generate_recommends_defer_on_low_agreement() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.md"), "the verbatim quote").unwrap();
        let claims = vec![claim("claim-20261117-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261117-001", 0.4, 2);
        let pkg = generate(&claims, &entries, &[], "claim-20261117-001", dir.path(), Utc::now())
            .unwrap();
        assert!(pkg.recommendation.starts_with("defer"), "{}", pkg.recommendation);
        assert!(pkg.risk.low_agreement);
    }

    #[test]
    fn generate_recommends_supersede_on_stale_confidence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.md"), "the verbatim quote").unwrap();
        let claims = vec![claim("claim-20261117-001", 0.31)];
        let entries = ledger_with_vet2("claim-20261117-001", 0.8, 4);
        let pkg = generate(&claims, &entries, &[], "claim-20261117-001", dir.path(), Utc::now())
            .unwrap();
        assert!(pkg.recommendation.starts_with("supersede"), "{}", pkg.recommendation);
        assert!(pkg.risk.stale_confidence);
    }

    #[test]
    fn generate_flags_unverified_evidence() {
        let dir = tempfile::tempdir().unwrap();
        // 不建立 evidence.md → 逐字驗證失敗
        let claims = vec![claim("claim-20261117-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261117-001", 0.8, 4);
        let pkg = generate(&claims, &entries, &[], "claim-20261117-001", dir.path(), Utc::now())
            .unwrap();
        assert!(!pkg.evidence[0].verified);
        assert_eq!(pkg.risk.unverified_evidence, 1);
        assert!(pkg.recommendation.contains("kem sync"), "{}", pkg.recommendation);
    }

    #[test]
    fn markdown_contains_all_sections_and_sha() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.md"), "the verbatim quote").unwrap();
        let claims = vec![claim("claim-20261107-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261107-001", 0.8, 4);
        let pkg = generate(&claims, &entries, &[], "claim-20261107-001", dir.path(), Utc::now())
            .unwrap();
        let md = render_markdown(&pkg);
        for sec in ["## 1. 主張", "## 2. 證據", "## 3. 聯合裁決", "## 4. 選項", "## 5. 建議", "## 6. 風險登記"] {
            assert!(md.contains(sec), "missing {sec}");
        }
        assert!(md.contains(&pkg.package_sha));
        assert!(md.contains("[✓]"));
        assert_eq!(md.matches("[✗]").count(), 0);
    }

    #[test]
    fn write_package_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("evidence.md"), "the verbatim quote").unwrap();
        let claims = vec![claim("claim-20261117-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261107-001", 0.8, 4);
        let pkg = generate(&claims, &entries, &[], "claim-20261117-001", dir.path(), Utc::now())
            .unwrap();
        let path = write_package(dir.path(), &pkg).unwrap();
        assert!(path.ends_with(".kem/packages/claim-20261117-001.md"));
        assert!(path.exists());
    }

    #[test]
    fn not_found_for_missing_claim() {
        let dir = tempfile::tempdir().unwrap();
        let claims = vec![claim("claim-20261117-001", 0.9)];
        let entries = ledger_with_vet2("claim-20261107-001", 0.8, 4);
        let err = generate(&claims, &entries, &[], "claim-20261117-999", dir.path(), Utc::now())
            .unwrap_err();
        assert!(matches!(err, KemError::NotFound(_)));
    }
}
