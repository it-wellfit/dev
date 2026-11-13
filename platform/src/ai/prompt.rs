#![forbid(unsafe_code)]
//! AI 建議任務的提示詞與結構化解碼 schema（單一事實來源）。
//!
//! 使用者：`api.rs::ai_suggest`（生產路徑）與 `examples/ai_bench.rs`（模型
//! 評測 harness）共用本模組——兩者不得各自維護提示詞副本，否則評測證據
//! 與生產行為脫鉤。
//!
//! 來源依據（2026-11 FREE SOTA 研究）：
//! - 解碼約束優先於提示詞（Ollama `format`=JSON Schema；llmconfigurator.com
//!   2026-07、AscentCore 2026-04 小型模型評測）
//! - 溫度 0 為抽取任務標準（slm-json-benchmark：0.7 使 schema 有效性 -20%）
//! - 混合解析：規則解析器對數字/日期是確定性的，作為權威提示注入
//!   （2026-11-13 實證：4B 模型曾把 88,500 截斷為 885.00——財務紅線）

use serde_json::json;

/// AI 建議任務的結構化解碼 schema（Ollama `format` 參數）。
pub fn suggest_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "date": {
                "type": "string",
                "description": "YYYY-MM-DD"
            },
            "debit_account": {"type": "string"},
            "credit_account": {"type": "string"},
            "amount": {
                "type": "string",
                "description": "兩位小數字串，不得使用千分位逗號"
            },
            "currency": {
                "type": "string",
                "description": "三位字母 ISO 4217 代碼（TWD/USD/EUR/JPY/CNY）"
            }
        },
        "required": ["date", "debit_account", "credit_account", "amount", "currency"]
    })
}

/// 建構 AI 建議提示詞。`amount_hint`/`date_hint` 來自規則解析器（權威值），
/// 未解析出時傳 `None`。
pub fn suggest_prompt(text: &str, amount_hint: Option<&str>, date_hint: Option<&str>) -> String {
    let amount_part = amount_hint.unwrap_or("未解析出");
    let date_part = date_hint.unwrap_or("未解析出");
    format!(
        "你是會計分錄解析助理。僅輸出一個 JSON 物件，規則：\n\
         1. date：必須為 YYYY-MM-DD（照抄權威提示的值，不得改格式）。\n\
         2. debit_account、credit_account：必須**原樣保留原文中的科目詞彙**\
         （不得翻譯為英文；例：原文「現金」→ \"現金\"，不是 \"CASH\"）。\n\
         3. amount：兩位小數字串，不得使用千分位逗號（照抄權威提示的值）。\n\
         4. currency：三位字母 ISO 4217 代碼（原文「元／新台幣」對應 TWD）。\n\
         權威提示（規則解析結果）：amount={amount_part}、date={date_part}。\n\
         原文：{text}"
    )
}

/// 建議任務的推論參數（溫度 0：抽取任務的確定性標準）。
pub const SUGGEST_TEMPERATURE: f32 = 0.0;
pub const SUGGEST_TOP_P: f32 = 0.9;
pub const SUGGEST_MAX_TOKENS: u32 = 512;
pub const SUGGEST_TIMEOUT_MS: u64 = 60_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_carries_authoritative_hints() {
        let p = suggest_prompt(
            "2026-11-12 收到客戶貨款 45,000 元：應收帳款→銀行存款",
            Some("45000.00（4500000）"),
            Some("2026-11-12"),
        );
        assert!(p.contains("45000.00"), "amount hint must be in prompt");
        assert!(p.contains("2026-11-12"));
        assert!(p.contains("ISO 4217"), "currency rule must be in prompt");
    }

    #[test]
    fn prompt_handles_missing_hints() {
        let p = suggest_prompt("raw text", None, None);
        assert!(p.contains("未解析出"));
    }

    #[test]
    fn schema_has_required_fields() {
        let s = suggest_schema();
        assert_eq!(s["type"], "object");
        let required = s["required"].as_array().expect("required array");
        assert_eq!(required.len(), 5);
    }
}
