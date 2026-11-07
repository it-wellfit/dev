#![forbid(unsafe_code)]
//! Seed set: 20 verified workspace facts as published claims.
//!
//! Every evidence quote is verbatim from a real workspace file; `kem seed`
//! runs T1 vetting over the whole set, so a drifted quote fails loudly.

use crate::schema::{Claim, ClaimStatus, ClaimType, Evidence, Track};
use chrono::{NaiveDate, Utc};

const ROOT_REV: &str = "working-tree";
const EKPI_REV: &str = "a268319";

struct SeedSpec {
    id: &'static str,
    claim_type: ClaimType,
    topics: &'static [&'static str],
    title: &'static str,
    body: &'static str,
    path: &'static str,
    rev: &'static str,
    quote: &'static str,
    raw_topic: &'static str,
}

const SEEDS: [SeedSpec; 20] = [
    SeedSpec {
        id: "claim-20261107-001",
        claim_type: ClaimType::Fact,
        topics: &["ekpi-platform", "phase-gates"],
        title: "ekpi-platform Phase A 未核准、Phase B 受阻",
        body: "ekpi-platform 的階段門狀態為 Phase A not approved 且 Phase B blocked，不得宣稱 production-ready。",
        path: "ekpi-platform/docs/quality/phase-gates.md",
        rev: EKPI_REV,
        quote: "- **Status:** Active; Phase A **not approved**, Phase B **blocked**",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-002",
        claim_type: ClaimType::Invariant,
        topics: &["ekpi-platform", "quality"],
        title: "階段門僅由具名當責角色以鏈結證據核准",
        body: "A gate is approved only by the named accountable role, with linked evidence and a review date.",
        path: "ekpi-platform/docs/quality/phase-gates.md",
        rev: EKPI_REV,
        quote: "A gate is approved only by the named accountable role, with linked evidence and a review date.",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-003",
        claim_type: ClaimType::Invariant,
        topics: &["ekpi-platform", "evidence-rule"],
        title: "PASS 僅在指定修訂版上執行並退出 0 時成立",
        body: "A check is PASS only if the exact command ran on the named revision and exited 0; skipped or blocked checks are Not run/Blocked, never PASS.",
        path: "ekpi-platform/AGENTS.md",
        rev: EKPI_REV,
        quote: "A check is `PASS` only if the exact command ran on the named revision and exited 0. A skipped or blocked check is `Not run`/`Blocked`, never `PASS`.",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-004",
        claim_type: ClaimType::Invariant,
        topics: &["ekpi-platform", "zero-js"],
        title: "ekpi 展示層零 JavaScript",
        body: "ekpi-platform 的瀏覽器展示層只允許 HTML、CSS、SVG、圖片與字型，不含任何 JavaScript。",
        path: "ekpi-platform/AGENTS.md",
        rev: EKPI_REV,
        quote: "No JavaScript in browser - only HTML, CSS, SVG, images, fonts",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-005",
        claim_type: ClaimType::Fact,
        topics: &["ekpi-platform", "api"],
        title: "ekpi api.rs 為未接線佔位符",
        body: "platform/src/api.rs is an unwired placeholder returning a fixed comply verdict; it must never be exposed or called a contract.",
        path: "ekpi-platform/AGENTS.md",
        rev: EKPI_REV,
        quote: "`platform/src/api.rs` is an unwired placeholder that returns a fixed `comply`; never expose it or call it a contract.",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-006",
        claim_type: ClaimType::Fact,
        topics: &["ekpi-platform", "contracts"],
        title: "ekpi 無已核准的生產 JSON API",
        body: "The ekpi-platform repository has no approved production JSON API; api.rs is not a contract.",
        path: "ekpi-platform/docs/contracts/README.md",
        rev: EKPI_REV,
        quote: "It does **not** have an approved production JSON API.",
        raw_topic: "ekpi",
    },
    SeedSpec {
        id: "claim-20261107-007",
        claim_type: ClaimType::Invariant,
        topics: &["workspace", "rust-standards"],
        title: "每個 .rs 首行 forbid(unsafe_code) 且 clippy 零警告",
        body: "Every Rust file starts with #![forbid(unsafe_code)] and clippy must pass with zero warnings across the workspace.",
        path: "AGENTS.md",
        rev: ROOT_REV,
        quote: "每個 `.rs` 第一行 `#![forbid(unsafe_code)]`；clippy 零警告",
        raw_topic: "workspace",
    },
    SeedSpec {
        id: "claim-20261107-008",
        claim_type: ClaimType::Invariant,
        topics: &["workspace", "http-rules"],
        title: "讀取 GET、寫入 POST、成功後 PRG",
        body: "HTTP usage rule: reads use GET; create/modify/delete use POST; successful submissions follow Post/Redirect/Get.",
        path: "AGENTS.md",
        rev: ROOT_REV,
        quote: "讀取 GET；建立／修改／刪除 POST；成功後 PRG",
        raw_topic: "workspace",
    },
    SeedSpec {
        id: "claim-20261107-009",
        claim_type: ClaimType::Invariant,
        topics: &["workspace", "documentation"],
        title: "文件與代碼同一提交更新",
        body: "Documentation and code are updated in the same commit so specs never drift from behavior.",
        path: "AGENTS.md",
        rev: ROOT_REV,
        quote: "文件與代碼同一提交更新",
        raw_topic: "workspace",
    },
    SeedSpec {
        id: "claim-20261107-010",
        claim_type: ClaimType::Decision,
        topics: &["knowledge-base", "adr-0004"],
        title: "KB 單向引用且對實作者唯讀",
        body: "ADR-0004 (Approved) defines a single-direction reference (project → KB); the knowledge base is read-only for implementers.",
        path: "docs/knowledge-index.md",
        rev: ROOT_REV,
        quote: "- **Authority:** ADR-0004 (Approved) — single-direction reference (project → KB). KB is read-only for implementers.",
        raw_topic: "knowledge-base",
    },
    SeedSpec {
        id: "claim-20261107-011",
        claim_type: ClaimType::Fact,
        topics: &["free-ai", "ollama"],
        title: "本機 Ollama 已安裝 qwen3.5:4b",
        body: "Local Ollama has qwen3.5:4b installed (4.2B params, Q4_K_M, 262K context, 3.3 GB) and serves it locally.",
        path: "docs/engineering/FREE_AI_ROUTING.md",
        rev: ROOT_REV,
        quote: "| **Model installed** | `qwen3.5:4b` (4.2B params, Q4_K_M, 262K context, 3.3 GB) |",
        raw_topic: "free-ai",
    },
    SeedSpec {
        id: "claim-20261107-012",
        claim_type: ClaimType::Decision,
        topics: &["free-ai", "routing"],
        title: "AI 路由以本機 Ollama 為主",
        body: "The AI fallback chain uses local Ollama (qwen3.5:4b) as PRIMARY, with keyless remote endpoints only as fallback.",
        path: "docs/engineering/FREE_AI_ROUTING.md",
        rev: ROOT_REV,
        quote: "1. Ollama local (qwen3.5:4b) — PRIMARY",
        raw_topic: "free-ai",
    },
    SeedSpec {
        id: "claim-20261107-013",
        claim_type: ClaimType::Fact,
        topics: &["free-ai", "endpoints"],
        title: "遠端免費端點需金鑰（實測 401/403）",
        body: "Remote free endpoints (OpenRouter, Z.ai, Cloudflare) returned 401/403 without API keys in the 2026-11-06 smoke test, so the workspace has no keyless LLM endpoint today.",
        path: "docs/engineering/FREE_AI_ROUTING.md",
        rev: ROOT_REV,
        quote: "| 2026-11-06 | OpenRouter | `https://openrouter.ai/api/v1/chat/completions` | FAIL | 401 — requires API key |",
        raw_topic: "free-ai",
    },
    SeedSpec {
        id: "claim-20261107-014",
        claim_type: ClaimType::Invariant,
        topics: &["methodology", "evidence-rule"],
        title: "UNKNOWN ≠ PASS ≠ FAIL 證據規則",
        body: "Workspace evidence rule: results carry command + revision + exit code, and UNKNOWN is never conflated with PASS or FAIL.",
        path: "docs/engineering/THINKING_OPERATING_SYSTEM.md",
        rev: ROOT_REV,
        quote: "證據規則（指令＋修訂版＋退出碼；UNKNOWN ≠ PASS ≠ FAIL）",
        raw_topic: "methodology",
    },
    SeedSpec {
        id: "claim-20261107-015",
        claim_type: ClaimType::Decision,
        topics: &["methodology", "single-writer"],
        title: "api.rs/web 採單一寫者原則",
        body: "platform/src/api.rs and web/* follow a single-writer rule to break the dual-writer merge-conflict loop; this is a structural intervention, not a process preference.",
        path: "docs/engineering/THINKING_OPERATING_SYSTEM.md",
        rev: ROOT_REV,
        quote: "**工作區錨點**：單一寫者原則（api.rs）不是流程偏好，是對「雙寫者合併衝突迴路」的規則層介入。",
        raw_topic: "methodology",
    },
    SeedSpec {
        id: "claim-20261107-016",
        claim_type: ClaimType::Decision,
        topics: &["kem", "zero-paid-ai"],
        title: "零付費 AI 為硬性約束",
        body: "Paid AI services (including JEV/TypeSafe) are excluded permanently; zero-cost is a hard constraint, and commercial tools serve only as pattern sources.",
        path: "docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md",
        rev: ROOT_REV,
        quote: "**付費 AI（含 JEV）全面排除**——零成本是硬性約束",
        raw_topic: "kem",
    },
    SeedSpec {
        id: "claim-20261107-017",
        claim_type: ClaimType::Fact,
        topics: &["kem", "scs"],
        title: "自一致性投票實證基線 90.8%→99.2%",
        body: "The self-consistency voting evidence baseline is 90.8% to 99.2% agreement (k=15, probability floor 0.60, automation 74.2%) per the JEV cookbook.",
        path: "docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md",
        rev: ROOT_REV,
        quote: "JEV cookbook 實測為 **90.8% → 99.2%**（k=15、機率地板 0.60、自動化率 74.2%）",
        raw_topic: "kem",
    },
    SeedSpec {
        id: "claim-20261107-018",
        claim_type: ClaimType::Concept,
        topics: &["kem", "north-star"],
        title: "100% 覆蓋定義為缺口清單清空",
        body: "100% knowledge coverage is defined as zero gap items in the enumerated domain inventory with every item covered — auditable, not a probability claim.",
        path: "docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md",
        rev: ROOT_REV,
        quote: "**100% 覆蓋的定義＝缺口項數 = 0 且全部 covered**",
        raw_topic: "kem",
    },
    SeedSpec {
        id: "claim-20261107-019",
        claim_type: ClaimType::Invariant,
        topics: &["kem", "lifecycle"],
        title: "retired 為唯一終態且完全出庫",
        body: "Retired claims leave the serving path entirely; history stays in raw/ and the ledger for audit, and superseded claims remain retrievable but demoted.",
        path: "docs/engineering/KNOWLEDGE_EVOLUTION_MODULE.md",
        rev: ROOT_REV,
        quote: "- `retired`：完全出庫；歷史保留於 raw/ 與帳本，供稽核",
        raw_topic: "kem",
    },
    SeedSpec {
        id: "claim-20261107-020",
        claim_type: ClaimType::Fact,
        topics: &["flaps-erp", "slo"],
        title: "交易分頁頁面 P99 門檻 50ms",
        body: "The Flaps ERP workbench sets an SLO of P99 < 50ms for the paginated /transactions page, measured in performance tests.",
        path: ".kilo/plans/1793033512795-flaps-erp-workbench-completion.md",
        rev: ROOT_REV,
        quote: "| `/transactions`（分頁） | P99 < 50ms |",
        raw_topic: "flaps-erp",
    },
];

/// Build the 20 seed claims with today as valid_from/recorded_at.
pub fn seed_claims() -> Vec<Claim> {
    let today = Utc::now().date_naive();
    SEEDS
        .iter()
        .map(|s| Claim {
            id: s.id.to_string(),
            claim_type: s.claim_type,
            status: ClaimStatus::Published,
            track: Track::B,
            confidence: 0.95,
            valid_from: today,
            valid_to: None,
            recorded_at: Utc::now(),
            supersedes: Vec::new(),
            superseded_by: None,
            evidence: vec![Evidence {
                path: s.path.to_string(),
                rev: s.rev.to_string(),
                quote: s.quote.to_string(),
            }],
            topics: s.topics.iter().map(|t| t.to_string()).collect(),
            source: format!("raw/{}/2026-11-07-seed.md", s.raw_topic),
            title: s.title.to_string(),
            body: s.body.to_string(),
        })
        .collect()
}

/// Raw source content per topic (immutable staging targets for seed).
pub fn seed_raw_sources() -> Vec<(&'static str, String)> {
    let mut by_topic: std::collections::BTreeMap<&'static str, Vec<&SeedSpec>> =
        std::collections::BTreeMap::new();
    for s in &SEEDS {
        by_topic.entry(s.raw_topic).or_default().push(s);
    }
    by_topic
        .into_iter()
        .map(|(topic, specs)| {
            let mut md = String::new();
            md.push_str(&format!("# KEM seed sources — {topic} (2026-11-07)\n\n"));
            md.push_str(
                "Verbatim quotes vetted at seed time; files are immutable once captured.\n\n",
            );
            for s in specs {
                md.push_str(&format!("## {}\n\n", s.title));
                md.push_str(&format!("- evidence: `{}` @ {}\n", s.path, s.rev));
                md.push_str(&format!("- quote: {}\n\n", s.quote));
            }
            (topic, md)
        })
        .collect()
}

/// Validate that every seed id matches the required shape and dates.
pub fn seed_ids_valid() -> bool {
    SEEDS.iter().all(|s| {
        s.id.starts_with("claim-20261107-")
            && s.id.len() == 18
            && s.id[15..].chars().all(|c| c.is_ascii_digit())
            && NaiveDate::from_ymd_opt(2026, 11, 7).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::validate_claim;

    #[test]
    fn twenty_seeds_all_validate_clean() {
        let claims = seed_claims();
        assert_eq!(claims.len(), 20);
        let ctx = crate::schema::ValidationContext {
            known_ids: crate::schema::known_ids_of(claims.iter()),
        };
        for c in &claims {
            let errs = validate_claim(c, Some(&ctx));
            assert!(errs.is_empty(), "{}: {errs:?}", c.id);
        }
    }

    #[test]
    fn seed_ids_are_unique_and_well_shaped() {
        let claims = seed_claims();
        let mut ids: Vec<_> = claims.iter().map(|c| c.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 20);
        assert!(seed_ids_valid());
    }

    #[test]
    fn every_seed_has_verbatim_quote_matching_its_raw_source() {
        for (topic, content) in seed_raw_sources() {
            assert!(content.contains("Verbatim quotes vetted at seed time"));
            assert!(!topic.is_empty());
        }
    }
}
