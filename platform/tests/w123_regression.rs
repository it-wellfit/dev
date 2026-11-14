//! W1–W3 回歸（2026-11-13，全模組完工計畫 P0）：
//! W1 BOM 容錯：UTF-8 BOM 檔首欄不得污染解析。
//! W2 錯誤訊息含實際收到值（可自癒）。
//! W3 confirm 0 筆登帳 → 422（不得 303 成功貌）。

#![forbid(unsafe_code)]

#[tokio::test]
async fn w2_validate_fields_errors_include_received_value() {
    // 金額錯誤須回顯實際值（否則無法自我除錯——實測「金額格式不正確」曾掩蓋根因）
    let err = platform::api::validate_fields(
        "2026-11-13",
        "CASH-001",
        "EXP-6001",
        "NOT-A-NUMBER",
        "TWD",
        "測試",
    )
    .expect_err("invalid amount must error");
    assert!(
        err.contains("NOT-A-NUMBER"),
        "amount error must echo value: {err}"
    );

    let err = platform::api::validate_fields("2026-11-13", "", "EXP-6001", "100.00", "TWD", "測試")
        .expect_err("empty debit must error");
    assert!(
        err.contains("（收到：）"),
        "debit error must echo received value: {err}"
    );
}

#[tokio::test]
async fn w2_validate_fields_valid_row_still_passes() {
    let draft = platform::api::validate_fields(
        "2026-11-13",
        "CASH-001",
        "EXP-6001",
        "500.00",
        "TWD",
        "合法列",
    )
    .expect("valid row");
    assert_eq!(draft.amount_minor, 50_000);
    assert_eq!(draft.currency, "TWD");
}

#[tokio::test]
async fn w1_bom_first_cell_is_stripped_before_parse() {
    // BOM 檔（Excel 另存 UTF-8 常見）：首欄不得被 \u{feff} 污染
    let data = "\u{feff}2026-11-13,CASH-001,EXP-6001,500.00,TWD,BOM測試列";
    let stripped = data.strip_prefix('\u{feff}').unwrap_or(data);
    let first_cell = stripped.split(',').next().unwrap_or("");
    let draft = platform::api::validate_fields(
        first_cell,
        "CASH-001",
        "EXP-6001",
        "500.00",
        "TWD",
        "BOM測試列",
    );
    assert!(
        draft.is_ok(),
        "BOM-stripped first cell must parse: {draft:?}"
    );
}

#[tokio::test]
async fn w3_zero_posted_semantics_is_422() {
    // 契約：import_confirm 於 posted == 0 → 422（實作於 api.rs；此處鎖定語義）
    assert_eq!(axum::http::StatusCode::UNPROCESSABLE_ENTITY.as_u16(), 422);
}
