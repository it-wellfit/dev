//! Rule-based parser: unstructured text → draft journal entry (Layer C).
//!
//! Design contract (關鍵指標：可解釋／可推導)：
//! - Pure functions, no I/O, no randomness: the same input ALWAYS produces
//!   the same draft (可推導).
//! - Every populated field carries an [`Evidence`] citing the rule and the
//!   matched text (依據卡) — rendered on the confirmation page.
//! - Rules NEVER post anything; drafts are human-confirmed before the ledger
//!   engine is involved (H3 trust model, AI 禁區 untouched).
//! - Integer money only (INV-01): amounts go through [`crate::money`].

use chrono::NaiveDate;

/// One rule hit attached to a draft field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Evidence {
    /// Stable rule identifier, e.g. `R-DATE-ISO`, `R-AMOUNT-YUAN`, `R-LEG-ARROW`,
    /// `R-ACCT-KEYWORD`.
    pub rule: String,
    /// The exact matched text fragment.
    pub matched: String,
}

impl Evidence {
    fn new(rule: &str, matched: impl Into<String>) -> Self {
        Self {
            rule: rule.to_string(),
            matched: matched.into(),
        }
    }
}

/// Draft journal entry produced by the parser (never posted directly).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DraftEntry {
    pub date: Option<NaiveDate>,
    pub amount_minor: Option<i64>,
    pub debit_account: Option<String>,
    pub credit_account: Option<String>,
    pub evidence: Vec<Evidence>,
}

impl DraftEntry {
    /// True when every ledger-critical field is present.
    pub fn is_complete(&self) -> bool {
        self.date.is_some()
            && self.amount_minor.is_some()
            && self.debit_account.is_some()
            && self.credit_account.is_some()
    }
}

/// Trim common CJK/ASCII punctuation around a token.
fn trim_punct(s: &str) -> &str {
    s.trim_matches(|c: char| {
        matches!(
            c,
            ' ' | '\t'
                | '，'
                | '。'
                | '；'
                | '、'
                | '（'
                | '）'
                | '('
                | ')'
                | ','
                | '.'
                | ';'
                | ':'
        )
    })
}

/// Try to parse a date token with the supported format set (rule table).
fn parse_date_token(token: &str) -> Option<NaiveDate> {
    let token = trim_punct(token);
    const FORMATS: [&str; 3] = ["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"];
    FORMATS
        .iter()
        .find_map(|f| chrono::NaiveDate::parse_from_str(token, f).ok())
}

/// Try to parse an amount token (rule table) into minor units.
/// Accepted shapes: `12,000`, `12000`, `12,000.50`, optionally prefixed
/// `NT$`/`$`, optionally suffixed `元`.
fn parse_amount_token(token: &str) -> Option<i64> {
    let token = trim_punct(token);
    let token = token
        .strip_suffix("元")
        .unwrap_or(token)
        .trim_start_matches("NT$")
        .trim_start_matches('$')
        .trim();
    if token.is_empty() || !token.bytes().any(|b| b.is_ascii_digit()) {
        return None;
    }
    // Only digits, commas and at most one dot are acceptable.
    if !token
        .bytes()
        .all(|b| b.is_ascii_digit() || b == b',' || b == b'.')
        || token.matches('.').count() > 1
    {
        return None;
    }
    let cleaned = token.replace(',', "");
    crate::money::parse_decimal_to_minor(&cleaned, 2).ok()
}

/// Map a free-text account mention onto a configured account via the keyword
/// table (longest keyword wins for determinism).
fn map_account(mention: &str, keyword_accounts: &[(String, String)]) -> Option<(String, String)> {
    let mention = trim_punct(mention);
    let mut best: Option<(&String, &String)> = None;
    for (keyword, account) in keyword_accounts {
        let longer = best.is_none_or(|(b, _)| keyword.len() > b.len());
        if mention.contains(keyword.as_str()) && longer {
            best = Some((keyword, account));
        }
    }
    best.map(|(k, a)| (k.clone(), a.clone()))
}

/// Parse unstructured text into a draft entry.
///
/// Rule table (applied in order, first hit wins per field):
/// - `R-DATE-ISO` / `R-DATE-ZH`: first parseable date token.
/// - `R-AMOUNT-YUAN`: first parseable amount token.
/// - `R-LEG-ARROW`: `A→B` (or `->`) — A is the CREDIT side, B is the DEBIT
///   side (money flows out of A into B for payments; the confirmation page
///   always shows both sides for human verification).
/// - `R-ACCT-KEYWORD`: legs mapped through `keyword_accounts` (exact contains,
///   longest keyword wins). Legs that map to nothing keep their raw trimmed
///   text so the human can correct them.
pub fn parse_unstructured(text: &str, keyword_accounts: &[(String, String)]) -> DraftEntry {
    let mut draft = DraftEntry::default();

    // Dates and amounts: token-wise scan, first hit wins.
    for token in text.split(|c: char| c.is_whitespace()) {
        if draft.date.is_none() {
            if let Some(d) = parse_date_token(token) {
                draft.date = Some(d);
                draft
                    .evidence
                    .push(Evidence::new("R-DATE", trim_punct(token)));
                continue;
            }
        }
        if draft.amount_minor.is_none() {
            if let Some(minor) = parse_amount_token(token) {
                draft.amount_minor = Some(minor);
                draft
                    .evidence
                    .push(Evidence::new("R-AMOUNT-YUAN", trim_punct(token)));
            }
        }
    }

    // Legs: explicit arrow pattern first (`A→B` or `A->B`).
    let arrow_split = text
        .split_once("→")
        .map(|(a, b)| (a, b, "→"))
        .or_else(|| text.split_once("->").map(|(a, b)| (a, b, "->")));

    if let Some((left, right, marker)) = arrow_split {
        // The left fragment may itself contain the date/amount; take the tail
        // of the left side (after the last whitespace) as the credit mention.
        let credit_mention = trim_punct(
            left.rsplit(|c: char| c.is_whitespace())
                .next()
                .unwrap_or(left),
        );
        let debit_mention = trim_punct(
            right
                .split(|c: char| c.is_whitespace())
                .next()
                .unwrap_or(right),
        );

        match map_account(credit_mention, keyword_accounts) {
            Some((keyword, account)) => {
                draft.credit_account = Some(account);
                draft.evidence.push(Evidence::new(
                    "R-ACCT-KEYWORD",
                    format!("{marker} 左側命中「{keyword}」"),
                ));
            }
            None => {
                draft.credit_account = Some(credit_mention.to_string());
                draft
                    .evidence
                    .push(Evidence::new("R-LEG-ARROW", credit_mention));
            }
        }
        match map_account(debit_mention, keyword_accounts) {
            Some((keyword, account)) => {
                draft.debit_account = Some(account);
                draft.evidence.push(Evidence::new(
                    "R-ACCT-KEYWORD",
                    format!("{marker} 右側命中「{keyword}」"),
                ));
            }
            None => {
                draft.debit_account = Some(debit_mention.to_string());
                draft
                    .evidence
                    .push(Evidence::new("R-LEG-ARROW", debit_mention));
            }
        }
        let _ = marker;
    }

    draft
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keywords() -> Vec<(String, String)> {
        vec![
            ("現金".to_string(), "CASH".to_string()),
            ("銀行".to_string(), "BANK".to_string()),
            ("租金".to_string(), "RENT-EXP".to_string()),
            ("薪資".to_string(), "PAYROLL-EXP".to_string()),
        ]
    }

    /// DRAFT-001 acceptance: the worked example from the plan.
    #[test]
    fn parse_worked_example_from_plan() {
        let draft = parse_unstructured("2026-11-04 支付租金 12,000 元 現金→租金費用", &keywords());
        assert_eq!(
            draft.date,
            Some(NaiveDate::from_ymd_opt(2026, 11, 4).unwrap())
        );
        assert_eq!(draft.amount_minor, Some(1_200_000));
        assert_eq!(draft.debit_account.as_deref(), Some("RENT-EXP"));
        assert_eq!(draft.credit_account.as_deref(), Some("CASH"));
        assert!(draft.is_complete());
        // 依據卡：每個欄位都有規則來源。
        assert!(draft
            .evidence
            .iter()
            .any(|e| e.rule == "R-DATE" && e.matched.contains("2026-11-04")));
        assert!(draft
            .evidence
            .iter()
            .any(|e| e.rule == "R-AMOUNT-YUAN" && e.matched.contains("12,000")));
        assert!(draft
            .evidence
            .iter()
            .any(|e| e.rule == "R-ACCT-KEYWORD" && e.matched.contains("租金")));
    }

    #[test]
    fn parse_zh_date_and_arrow_ascii() {
        let draft = parse_unstructured("2026年01月05日 轉帳 5,000.25 元 銀行->現金", &keywords());
        assert_eq!(
            draft.date,
            Some(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap())
        );
        assert_eq!(draft.amount_minor, Some(500_025));
        assert_eq!(draft.debit_account.as_deref(), Some("CASH"));
        assert_eq!(draft.credit_account.as_deref(), Some("BANK"));
    }

    #[test]
    fn parse_without_arrow_leaves_legs_empty_but_dates_amounts_filled() {
        let draft = parse_unstructured("2026-02-01 支付薪資 45,000 元", &keywords());
        assert!(draft.date.is_some());
        assert_eq!(draft.amount_minor, Some(4_500_000));
        assert!(
            draft.debit_account.is_none(),
            "no arrow: human fills the legs"
        );
        assert!(draft.credit_account.is_none());
        assert!(!draft.is_complete());
    }

    #[test]
    fn parse_garbage_yields_empty_draft() {
        let draft = parse_unstructured("今天天氣很好", &keywords());
        assert_eq!(draft, DraftEntry::default());
        assert!(!draft.is_complete());
    }

    #[test]
    fn parse_rejects_overprecise_amounts_silently() {
        // 0.005 元 is not representable in 2dp minor units — the amount rule
        // must not fire (fail-closed), leaving the field for the human.
        let draft = parse_unstructured("2026-03-01 付 0.005 元 現金→零用金", &keywords());
        assert!(draft.date.is_some());
        assert_eq!(draft.amount_minor, None);
    }
}
