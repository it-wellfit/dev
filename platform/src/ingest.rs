#![forbid(unsafe_code)]
//! 進料正規化（V3-B；D-MAP1）：欄位簽名匹配＋同義詞自動對應＋profile。
//!
//! 設計（docs/research/全能進料門戶_V2.5_V3_研究與設計.md §3.3）：
//! - 標準 6 欄（date, debit_account, credit_account, amount, currency, description）
//!   直通，零開銷。
//! - 非標準首列 → 同義詞表自動對應：全部命中且不重複 → 重排（丟棄無關欄）。
//! - 命中結果按簽名（排序欄名 sha256 短碼）存 `erp_config`（category=ingest_profile）
//!   ——重複來源一次設定永久免調；UPSERT 冪等。
//! - 對應不繞資金欄位驗證：validate_fields 仍為最後一關（治理 P7）。
//!
//! 純邏輯、零 I/O（profile 存取由呼叫端注入）——可獨立單元測試。

use sha2::Digest;

/// 目標欄位（標準 6 欄順序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetField {
    Date,
    DebitAccount,
    CreditAccount,
    Amount,
    Currency,
    Description,
}

impl TargetField {
    /// 標準欄位名（與 CSV 模板首列一致）。
    pub fn canonical(self) -> &'static str {
        match self {
            TargetField::Date => "date",
            TargetField::DebitAccount => "debit_account",
            TargetField::CreditAccount => "credit_account",
            TargetField::Amount => "amount",
            TargetField::Currency => "currency",
            TargetField::Description => "description",
        }
    }
}

/// 同義詞表（常見來源欄名；小寫比對）。
const SYNONYMS: [(&str, TargetField); 27] = [
    ("date", TargetField::Date),
    ("日期", TargetField::Date),
    ("交易日期", TargetField::Date),
    ("傳票日期", TargetField::Date),
    ("日期時間", TargetField::Date),
    ("debit", TargetField::DebitAccount),
    ("debit_account", TargetField::DebitAccount),
    ("借方", TargetField::DebitAccount),
    ("借方科目", TargetField::DebitAccount),
    ("付方", TargetField::DebitAccount),
    ("credit", TargetField::CreditAccount),
    ("credit_account", TargetField::CreditAccount),
    ("貸方", TargetField::CreditAccount),
    ("貸方科目", TargetField::CreditAccount),
    ("收方", TargetField::CreditAccount),
    ("amount", TargetField::Amount),
    ("金額", TargetField::Amount),
    ("價稅合計", TargetField::Amount),
    ("應付金額", TargetField::Amount),
    ("currency", TargetField::Currency),
    ("幣別", TargetField::Currency),
    ("幣種", TargetField::Currency),
    ("description", TargetField::Description),
    ("摘要", TargetField::Description),
    ("說明", TargetField::Description),
    ("備註", TargetField::Description),
    ("品名", TargetField::Description),
];

/// 單一標題 → 目標欄位（小寫 trim 比對；未知名 → None）。
pub fn match_header(header: &str) -> Option<TargetField> {
    let key = header.trim().to_lowercase();
    SYNONYMS
        .iter()
        .find(|(syn, _)| *syn == key)
        .map(|(_, field)| *field)
}

/// 首列標題 → 每欄的目標對應。
/// 全部 6 個目標欄位都恰好命中一次（其餘欄視為無關）才回 Some。
pub fn synonym_match(headers: &[String]) -> Option<Vec<Option<TargetField>>> {
    let mapped: Vec<Option<TargetField>> = headers.iter().map(|h| match_header(h)).collect();
    let mut seen = [false; 6];
    for field in mapped.iter().flatten() {
        let idx = *field as usize;
        if seen[idx] {
            return None; // 重複目標欄 → 簽名不可靠
        }
        seen[idx] = true;
    }
    if seen.iter().all(|&s| s) {
        Some(mapped)
    } else {
        None
    }
}

/// 來源簽名（排序欄名 join → sha256 前 12 碼；大小寫/空白正規化）。
pub fn signature(headers: &[String]) -> String {
    let mut names: Vec<String> = headers
        .iter()
        .map(|h| h.trim().to_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
    names.sort();
    let digest = sha2::Sha256::digest(names.join("|").as_bytes());
    let hex = format!("{digest:x}");
    hex.chars().take(12).collect()
}

/// 依對應重排一列（無關欄丟棄；缺欄補空——validate_fields 會攔）。
pub fn apply_mapping(cells: &[String], mapping: &[Option<TargetField>]) -> [String; 6] {
    let mut out: [String; 6] = std::array::from_fn(|_| String::new());
    for (cell, field) in cells.iter().zip(mapping.iter()) {
        if let Some(field) = field {
            out[*field as usize] = cell.trim().to_string();
        }
    }
    out
}

/// profile 鍵（erp_config：category=ingest_profile）。
pub fn profile_key(headers: &[String]) -> String {
    format!("profile.{}", signature(headers))
}

/// 載入已存 profile 的對應（erp_config；derive）。
pub async fn load_profile(
    pool: &sqlx::SqlitePool,
    headers: &[String],
) -> Option<Vec<Option<TargetField>>> {
    let key = profile_key(headers);
    let (json,): (String,) = sqlx::query_as(
        "SELECT config_value FROM erp_config WHERE config_category = 'ingest_profile' AND config_key = ?",
    )
    .bind(&key)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    // JSON 陣列：0-5 對應 TargetField 索引；null = 無關欄。
    let arr: Vec<Option<usize>> = serde_json::from_str(&json).ok()?;
    let decode = |v: Option<usize>| -> Option<TargetField> {
        v.map(|i| match i {
            0 => TargetField::Date,
            1 => TargetField::DebitAccount,
            2 => TargetField::CreditAccount,
            3 => TargetField::Amount,
            4 => TargetField::Currency,
            5 => TargetField::Description,
            _ => TargetField::Date,
        })
    };
    let mapped: Vec<Option<TargetField>> =
        arr.into_iter().take(headers.len()).map(decode).collect();
    synonym_match_consistent(&mapped).then_some(mapped)
}

fn synonym_match_consistent(mapped: &[Option<TargetField>]) -> bool {
    let mut seen = [false; 6];
    for f in mapped.iter().flatten() {
        let idx = *f as usize;
        if seen[idx] {
            return false;
        }
        seen[idx] = true;
    }
    seen.iter().all(|&s| s)
}

/// 儲存 profile（erp_config UPSERT；冪等）。
pub async fn save_profile(
    pool: &sqlx::SqlitePool,
    headers: &[String],
    mapping: &[Option<TargetField>],
) -> Result<(), sqlx::Error> {
    let key = profile_key(headers);
    let json = serde_json::to_string(
        &mapping
            .iter()
            .map(|m| m.map(|f| f as usize))
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default();
    // erp_config 主鍵為 config_id（無 category+key 複合唯一約束）：
    // 以決定性 config_id（ingest_profile.<簽名>）作 UPSERT 冪等鍵。
    let config_id = format!("ingest_profile.{}", signature(headers));
    sqlx::query(
        "INSERT INTO erp_config (config_id, config_category, config_key, config_value, config_type) \
         VALUES (?, 'ingest_profile', ?, ?, 'string') \
         ON CONFLICT(config_id) DO UPDATE SET config_value = excluded.config_value",
    )
    .bind(&config_id)
    .bind(&key)
    .bind(&json)
    .execute(pool)
    .await?;
    Ok(())
}

/// 決策鏈（V3-B）：profile 命中 → 同義詞全命中 → 無（映射待辦）。
/// 回 Some((mapping, needs_reorder))＝自動處理；None＝需使用者映射。
pub async fn decide_mapping(
    pool: &sqlx::SqlitePool,
    headers: &[String],
) -> Option<(Vec<Option<TargetField>>, bool)> {
    if let Some(mapping) = load_profile(pool, headers).await {
        return Some((mapping, true));
    }
    normalize_decision(headers)
}

/// 映射待辦批次持久化：batch（status=mapping）＋列（raw_line=TAB cells、
/// draft_json=cells JSON 陣列、status=mapping）。零 schema 變更——
/// 既有欄位承載待映射狀態（治理：不虛構，mapping 列不冒充合格）。
pub async fn persist_mapping_batch(
    state: &crate::api::AppState,
    batch_id: &str,
    sha_hex: &str,
    rows_all: &[(usize, Vec<String>)],
    created_by: &str,
) -> Result<(), sqlx::Error> {
    let row_total = rows_all.len() as i64;
    sqlx::query(
        "INSERT INTO import_batches (batch_id, file_sha256, row_total, row_ok, status, created_by) \
         VALUES (?, ?, ?, 0, 'mapping', ?)",
    )
    .bind(batch_id)
    .bind(sha_hex)
    .bind(row_total)
    .bind(created_by)
    .execute(&state.pool)
    .await?;
    for (row_no, cells) in rows_all {
        let raw = cells.join("\t");
        let json = serde_json::to_string(cells).unwrap_or_default();
        sqlx::query(
            "INSERT INTO import_rows (batch_id, row_no, raw_line, status, draft_json) \
             VALUES (?, ?, ?, 'mapping', ?)",
        )
        .bind(batch_id)
        .bind(*row_no as i64)
        .bind(&raw)
        .bind(&json)
        .execute(&state.pool)
        .await?;
    }
    Ok(())
}

/// 從 mapping 列還原 cells（draft_json=cells JSON 陣列）。
pub fn cells_from_json(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json).unwrap_or_default()
}

/// 對標題列做正規化決策：回 (mapping, 是否命中 profile/同義詞)。
/// 標準 6 欄 → 恆等對應（直通旗標 false 表示無需重排）。
pub fn normalize_decision(headers: &[String]) -> Option<(Vec<Option<TargetField>>, bool)> {
    // 標準直通：前 6 欄恰好是標準欄位（順序一致）。
    if headers.len() >= 6 {
        let std6 = [
            "date",
            "debit_account",
            "credit_account",
            "amount",
            "currency",
            "description",
        ];
        let head6: Vec<String> = headers
            .iter()
            .take(6)
            .map(|h| h.trim().to_lowercase())
            .collect();
        if head6 == std6 {
            return Some((vec![None; headers.len()], false));
        }
    }
    // profile / 同義詞。
    synonym_match(headers).map(|m| (m, true))
}

/// 對資料列套用對應（無重排需求 → 原樣回傳）。
pub fn apply_row(
    cells: &[String],
    mapping: &[Option<TargetField>],
    needs_reorder: bool,
) -> Vec<String> {
    if !needs_reorder {
        return cells.to_vec();
    }
    apply_mapping(cells, mapping).to_vec()
}

/// 對（已映射或標準順序的）cells 直接驗證——委派 platform 的 validate_fields
/// 不可跨模組（私有），故此處僅做「順序不變時的直通轉發」；實際驗證在
/// 呼叫端（import_preview 的 push_import_row/validate_fields）。
/// 本函式保留為語意別名，避免呼叫端遺漏映射後的驗證步驟。
pub fn validate_mapped(cells: &[String]) -> Result<crate::api::ImportRowDraft, String> {
    if cells.len() < 6 {
        return Err(format!("欄位不足（需 6 欄），實得 {} 欄", cells.len()));
    }
    crate::api::validate_fields(
        &cells[0], &cells[1], &cells[2], &cells[3], &cells[4], &cells[5],
    )
}

/// 單元測試：同義詞對應、簽名穩定、重排正確、重複欄拒絕。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synonym_match_maps_all_six() {
        let headers = vec![
            "交易日期".to_string(),
            "借方科目".to_string(),
            "貸方科目".to_string(),
            "價稅合計".to_string(),
            "幣別".to_string(),
            "摘要".to_string(),
            "備註欄（無關）".to_string(),
        ];
        let m = synonym_match(&headers).expect("all six must match");
        assert_eq!(m[0], Some(TargetField::Date));
        assert_eq!(m[1], Some(TargetField::DebitAccount));
        assert_eq!(m[2], Some(TargetField::CreditAccount));
        assert_eq!(m[3], Some(TargetField::Amount));
        assert_eq!(m[4], Some(TargetField::Currency));
        assert_eq!(m[5], Some(TargetField::Description));
        assert_eq!(m[6], None, "unrelated column is ignored");
    }

    #[test]
    fn synonym_match_rejects_duplicate_targets() {
        let headers = vec![
            "日期".to_string(),
            "日期時間".to_string(), // 同為 Date → 重複
            "借方".to_string(),
            "貸方".to_string(),
            "金額".to_string(),
            "幣別".to_string(),
            "摘要".to_string(),
        ];
        assert!(synonym_match(&headers).is_none());
    }

    #[test]
    fn signature_is_stable_across_order_and_case() {
        let a = vec![
            "Date".to_string(),
            " Amount".to_string(),
            "幣別".to_string(),
        ];
        let b = vec!["amount".to_string(), "date".to_string(), "幣別".to_string()];
        assert_eq!(signature(&a), signature(&b));
    }

    #[test]
    fn apply_mapping_reorders_cells() {
        let headers = vec![
            "金額".to_string(),
            "摘要".to_string(),
            "日期".to_string(),
            "幣別".to_string(),
            "借方".to_string(),
            "貸方".to_string(),
        ];
        let m = synonym_match(&headers).expect("must match");
        let cells = vec![
            "1500.5".to_string(),
            "測試".to_string(),
            "2026-11-09".to_string(),
            "TWD".to_string(),
            "ACC-A".to_string(),
            "ACC-B".to_string(),
        ];
        let reordered = apply_mapping(&cells, &m);
        assert_eq!(reordered[0], "2026-11-09", "date first");
        assert_eq!(reordered[1], "ACC-A", "debit second");
        assert_eq!(reordered[3], "1500.5", "amount fourth");
    }
}
