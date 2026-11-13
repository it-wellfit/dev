#![forbid(unsafe_code)]
//! AI 模型評測 harness（2026-11-13）——「持續工程演進」的可重跑證據工具。
//!
//! 用途：新模型上市或換機時，重跑本工具取得**實證**的模型選擇，取代
//! 「寫死模型名」的假設。評測對象：會計分錄抽取（生產任務）。
//!
//! 評測指標（每模型每案例）：
//!   schema-pass（parse_ai_suggestion 通過）
//!   date ✓ / debit ✓ / credit ✓ / amount ✓（整數分精確）/ currency ✓
//!
//! 執行：`cargo run --release -p platform --example ai_bench -- <model> [model...]`
//! （未帶參數＝評測 Ollama 已安裝的全部模型）
//!
//! 選擇流程（證據→決策）：本工具輸出分數表 → 最高 schema-pass×欄位正確者
//! → 管理員於 /admin/system 或 erp_config 設定 ai/model。來源依據記錄於
//! docs/engineering/UX_CONTRACTS.md。

use std::time::Instant;

/// 評測案例：非結構化文字＋期望欄位（與規則解析器對照的黃金集）。
struct Case {
    text: &'static str,
    date: &'static str,
    debit: &'static str,
    credit: &'static str,
    amount_minor: i64,
    currency: &'static str,
}

const fn cases() -> [Case; 6] {
    [
        Case {
            text: "2026-11-12 支付辦公室租金 12,000 元：現金→租金費用",
            date: "2026-11-12",
            debit: "租金費用",
            credit: "現金",
            amount_minor: 1_200_000,
            currency: "TWD",
        },
        Case {
            text: "2026-11-12 收到客戶貨款 45,000 元：應收帳款→銀行存款",
            date: "2026-11-12",
            debit: "銀行存款",
            credit: "應收帳款",
            amount_minor: 4_500_000,
            currency: "TWD",
        },
        Case {
            text: "2026/11/13 進貨一批 88,500 元：銀行存款→進貨費用",
            date: "2026-11-13",
            debit: "進貨費用",
            credit: "銀行存款",
            amount_minor: 8_850_000,
            currency: "TWD",
        },
        Case {
            text: "2026年11月13日 支付薪資 250,000 元：銀行存款→薪資費用",
            date: "2026-11-13",
            debit: "薪資費用",
            credit: "銀行存款",
            amount_minor: 25_000_000,
            currency: "TWD",
        },
        Case {
            text: "11/13 支付軟體訂閱費 3,999 元：現金→辦公用品",
            date: "2026-11-13",
            debit: "辦公用品",
            credit: "現金",
            amount_minor: 399_900,
            currency: "TWD",
        },
        Case {
            text: "2026-11-13 收取利息收入 8,250 元：銀行存款→利息收入",
            date: "2026-11-13",
            debit: "利息收入",
            credit: "銀行存款",
            amount_minor: 825_000,
            currency: "TWD",
        },
    ]
}

#[tokio::main]
async fn main() {
    let models = match collect_args() {
        Some(args) => args,
        None => list_installed().await,
    };
    if models.is_empty() {
        eprintln!("ai_bench: Ollama 無已安裝模型，或參數解析失敗");
        std::process::exit(1);
    }
    let base = "http://127.0.0.1:11434";
    let cases = cases();

    println!("ai_bench：會計分錄抽取評測（{} 案例）", cases.len());
    println!(
        "{:<22} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}",
        "model", "schema", "date", "debit", "credit", "amount", "curr", "avg-ms"
    );
    println!("{}", "-".repeat(80));

    let mut summary: Vec<(String, usize, usize)> = Vec::new();
    for model in &models {
        let mut schema_pass = 0usize;
        let mut field_pass = [0usize; 5]; // date, debit, credit, amount, currency
        let mut total_ms = 0u128;
        let mut timed = 0usize;
        for c in &cases {
            // 權威提示：規則解析器對數字/日期是確定性的（生產路徑同一行為）。
            let rule = auxbridge_core::parse_unstructured(c.text, &[]);
            let amount_hint = rule
                .amount_minor
                .map(|m| format!("{}（{}）", auxbridge_core::format_minor_plain(m, 2), m));
            let date_hint = rule.date.map(|d| d.to_string());
            let prompt = platform::ai::prompt::suggest_prompt(
                c.text,
                amount_hint.as_deref(),
                date_hint.as_deref(),
            );
            let client = platform::ai::ollama_client::OllamaClient::new(
                platform::ai::ollama_client::OllamaConfig {
                    base_url: base.to_string(),
                    model: model.clone(),
                    timeout_ms: 90_000,
                    temperature: platform::ai::prompt::SUGGEST_TEMPERATURE,
                    top_p: platform::ai::prompt::SUGGEST_TOP_P,
                    max_tokens: Some(platform::ai::prompt::SUGGEST_MAX_TOKENS),
                    format: Some(platform::ai::prompt::suggest_schema()),
                },
            );
            let t = Instant::now();
            let result = client.generate(&prompt).await;
            total_ms += t.elapsed().as_millis();
            timed += 1;
            match result {
                Ok(raw) => match auxbridge_core::parse_ai_suggestion(&raw) {
                    Ok(p) => {
                        schema_pass += 1;
                        if p.date == c.date {
                            field_pass[0] += 1;
                        }
                        if leg_matches(&p.debit_account, c.debit) {
                            field_pass[1] += 1;
                        }
                        if leg_matches(&p.credit_account, c.credit) {
                            field_pass[2] += 1;
                        }
                        if p.amount_minor == c.amount_minor {
                            field_pass[3] += 1;
                        }
                        if p.currency == c.currency {
                            field_pass[4] += 1;
                        }
                    }
                    Err(msg) => eprintln!("  [{model}] schema FAIL: {msg}"),
                },
                Err(e) => eprintln!("  [{model}] transport FAIL: {e}"),
            }
        }
        let n = cases.len();
        println!(
            "{:<22} {:>4}/{} {:>4}/{} {:>4}/{} {:>4}/{} {:>4}/{} {:>4}/{} {:>6.0}",
            model,
            schema_pass,
            n,
            field_pass[0],
            n,
            field_pass[1],
            n,
            field_pass[2],
            n,
            field_pass[3],
            n,
            field_pass[4],
            n,
            if timed > 0 {
                total_ms / timed as u128
            } else {
                0
            }
        );
        summary.push((model.clone(), schema_pass, field_pass.iter().sum()));
    }

    println!();
    if let Some(best) = summary
        .iter()
        .max_by_key(|(_, schema, fields)| (schema * 1000, *fields))
    {
        println!(
            "verdict: {}（schema {}/{}, 欄位合計 {}/{}）——以最高 schema×欄位正確者為準，\n設定：管理員於 /admin/system 或 erp_config 設定 ai/model。新模型上市時重跑本工具。",
            best.0,
            best.1,
            cases.len(),
            best.2,
            cases.len() * 5
        );
    }
}

fn leg_matches(predicted: &str, expected: &str) -> bool {
    // 科目允許「代碼或原文」兩種表達（人類覆核頁可改）。
    predicted.contains(expected) || expected.contains(predicted)
}

fn collect_args() -> Option<Vec<String>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { None } else { Some(args) }
}

async fn list_installed() -> Vec<String> {
    let url = "http://127.0.0.1:11434/api/tags";
    let Ok(resp) = reqwest::Client::new().get(url).send().await else {
        return Vec::new();
    };
    let Ok(value) = resp.json::<serde_json::Value>().await else {
        return Vec::new();
    };
    value["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
