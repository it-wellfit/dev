//! E.Sun BankPay 狀態機邏輯。
//!
//! 定義銀行支付批次的狀態轉換規則，確保狀態變更可追溯、可驗證。

pub use crate::bankpay::model::BankPayStatus;

/// 銀行支付狀態轉換規則（靜態切片，零堆積；`const` 中 `vec!` 不可 const 求值）。
///
/// 每個狀態定義允許轉換到的下一個狀態集合，防止非法狀態跳轉。
fn allowed_targets(from: &BankPayStatus) -> &'static [BankPayStatus] {
    match from {
        BankPayStatus::Pending => &[BankPayStatus::Submitted, BankPayStatus::Failed],
        BankPayStatus::Submitted => &[
            BankPayStatus::Completed,
            BankPayStatus::Failed,
            BankPayStatus::Rejected,
        ],
        BankPayStatus::Completed => &[],
        BankPayStatus::Failed => &[BankPayStatus::Pending],
        BankPayStatus::Rejected => &[BankPayStatus::Pending],
    }
}

/// 驗證狀態轉換是否合法。
///
/// # 參數
/// - `from`: 當前狀態
/// - `to`: 目標狀態
///
/// # 返回
/// 若轉換合法則返回 `Ok(())`，否則返回錯誤訊息字串。
pub fn validate_transition(from: &BankPayStatus, to: &BankPayStatus) -> Result<(), String> {
    if allowed_targets(from).contains(to) {
        return Ok(());
    }
    Err(format!("無效的狀態轉換：{:?} -> {:?}", from, to))
}

/// 根據事件類型推斷目標狀態。
pub fn status_from_event(event_type: &str) -> BankPayStatus {
    match event_type {
        "BANKPAY_EXPORT_SUCCESS" => BankPayStatus::Submitted,
        "BANKPAY_EXPORT_FAILURE" => BankPayStatus::Failed,
        "BANKPAY_QUERY_SUCCESS" => BankPayStatus::Completed,
        "BANKPAY_QUERY_FAILURE" => BankPayStatus::Failed,
        "BANKPAY_IMPORT_SUCCESS" => BankPayStatus::Completed,
        "BANKPAY_IMPORT_FAILURE" => BankPayStatus::Failed,
        "BANKPAY_REJECTED" => BankPayStatus::Rejected,
        _ => BankPayStatus::Pending,
    }
}

/// 檢查狀態是否為終端狀態（不再轉換）。
pub fn is_terminal(status: &BankPayStatus) -> bool {
    matches!(status, BankPayStatus::Completed | BankPayStatus::Rejected)
}

/// 檢查狀態是否為失敗狀態（可重試）。
pub fn is_retryable(status: &BankPayStatus) -> bool {
    matches!(status, BankPayStatus::Failed | BankPayStatus::Rejected)
}

/// 獲取狀態的顯示標籤（繁體中文）。
pub fn status_label(status: &BankPayStatus) -> &'static str {
    match status {
        BankPayStatus::Pending => "等待提交",
        BankPayStatus::Submitted => "處理中",
        BankPayStatus::Completed => "已完成",
        BankPayStatus::Failed => "失敗",
        BankPayStatus::Rejected => "退件",
    }
}

/// 獲取狀態的 CSS 樣式類別名稱。
pub fn status_css_class(status: &BankPayStatus) -> &'static str {
    match status {
        BankPayStatus::Pending => "badge badge-info",
        BankPayStatus::Submitted => "badge badge-warning",
        BankPayStatus::Completed => "badge badge-success",
        BankPayStatus::Failed => "badge badge-danger",
        BankPayStatus::Rejected => "badge badge-warning",
    }
}

/// 獲取狀態的處理建議。
pub fn status_advice(status: &BankPayStatus) -> &'static str {
    match status {
        BankPayStatus::Pending => "請點擊「立即提交」按鈕將批次送至銀行處理。",
        BankPayStatus::Submitted => "銀行正在處理中，請稍後查詢狀態或點擊「查詢狀態」按鈕。",
        BankPayStatus::Completed => "交易已完成，可下載銀行憑證作為憑證。",
        BankPayStatus::Failed => "提交失敗，請檢查錯誤訊息後重新提交。",
        BankPayStatus::Rejected => "銀行退件，請人工複核退件原因後重新建立批次。",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition(&BankPayStatus::Pending, &BankPayStatus::Submitted).is_ok());
        assert!(validate_transition(&BankPayStatus::Pending, &BankPayStatus::Failed).is_ok());
        assert!(validate_transition(&BankPayStatus::Submitted, &BankPayStatus::Completed).is_ok());
        assert!(validate_transition(&BankPayStatus::Submitted, &BankPayStatus::Failed).is_ok());
        assert!(validate_transition(&BankPayStatus::Submitted, &BankPayStatus::Rejected).is_ok());
        assert!(validate_transition(&BankPayStatus::Failed, &BankPayStatus::Pending).is_ok());
        assert!(validate_transition(&BankPayStatus::Rejected, &BankPayStatus::Pending).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition(&BankPayStatus::Pending, &BankPayStatus::Completed).is_err());
        assert!(validate_transition(&BankPayStatus::Completed, &BankPayStatus::Pending).is_err());
        assert!(validate_transition(&BankPayStatus::Completed, &BankPayStatus::Failed).is_err());
        assert!(validate_transition(&BankPayStatus::Failed, &BankPayStatus::Completed).is_err());
    }

    #[test]
    fn test_terminal_states() {
        assert!(is_terminal(&BankPayStatus::Completed));
        assert!(is_terminal(&BankPayStatus::Rejected));
        assert!(!is_terminal(&BankPayStatus::Pending));
        assert!(!is_terminal(&BankPayStatus::Submitted));
        assert!(!is_terminal(&BankPayStatus::Failed));
    }

    #[test]
    fn test_retryable_states() {
        assert!(is_retryable(&BankPayStatus::Failed));
        assert!(is_retryable(&BankPayStatus::Rejected));
        assert!(!is_retryable(&BankPayStatus::Pending));
        assert!(!is_retryable(&BankPayStatus::Submitted));
        assert!(!is_retryable(&BankPayStatus::Completed));
    }

    #[test]
    fn test_status_from_event() {
        assert_eq!(
            status_from_event("BANKPAY_EXPORT_SUCCESS"),
            BankPayStatus::Submitted
        );
        assert_eq!(
            status_from_event("BANKPAY_EXPORT_FAILURE"),
            BankPayStatus::Failed
        );
        assert_eq!(
            status_from_event("BANKPAY_QUERY_SUCCESS"),
            BankPayStatus::Completed
        );
        assert_eq!(
            status_from_event("BANKPAY_QUERY_FAILURE"),
            BankPayStatus::Failed
        );
        assert_eq!(
            status_from_event("BANKPAY_IMPORT_SUCCESS"),
            BankPayStatus::Completed
        );
        assert_eq!(
            status_from_event("BANKPAY_IMPORT_FAILURE"),
            BankPayStatus::Failed
        );
        assert_eq!(
            status_from_event("BANKPAY_REJECTED"),
            BankPayStatus::Rejected
        );
        assert_eq!(status_from_event("UNKNOWN"), BankPayStatus::Pending);
    }

    #[test]
    fn test_status_label() {
        assert_eq!(status_label(&BankPayStatus::Pending), "等待提交");
        assert_eq!(status_label(&BankPayStatus::Submitted), "處理中");
        assert_eq!(status_label(&BankPayStatus::Completed), "已完成");
        assert_eq!(status_label(&BankPayStatus::Failed), "失敗");
        assert_eq!(status_label(&BankPayStatus::Rejected), "退件");
    }

    #[test]
    fn test_status_css_class() {
        assert_eq!(
            status_css_class(&BankPayStatus::Pending),
            "badge badge-info"
        );
        assert_eq!(
            status_css_class(&BankPayStatus::Completed),
            "badge badge-success"
        );
        assert_eq!(
            status_css_class(&BankPayStatus::Failed),
            "badge badge-danger"
        );
    }

    #[test]
    fn test_status_advice() {
        assert!(!status_advice(&BankPayStatus::Pending).is_empty());
        assert!(!status_advice(&BankPayStatus::Completed).is_empty());
        assert!(!status_advice(&BankPayStatus::Failed).is_empty());
    }
}
