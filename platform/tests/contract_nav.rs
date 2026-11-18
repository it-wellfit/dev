//! 契約測試（計畫 1793782615513 §6）：導覽 IA 為 SSOT 契約。
//!
//! 快照對象：`NavModel`（由 `console_manifest` SSOT 經 gate 過濾生成）。
//! 任何 manifest/NavModel 結構變更 → snapshot diff → 需人工審核後
//! `cargo insta accept`（或將 `.snap.new` 更名為 `.snap`）。
//!
//! 覆蓋三種角色視角（gate 過濾是 IA 契約的核心語義）：
//! - ADMIN（全閘）：flatten_solo（銀行支付）、contains_active、圖示映射
//! - VIEWER（唯讀）：write/manage 入口不渲染
//! - 無角色（role_code 空字串）

use platform::api::NavFlags;
use platform::nav::NavModel;

fn flags(read: bool, write: bool, manage: bool, role: &str) -> NavFlags {
    NavFlags {
        ledger_read: read,
        ledger_write: write,
        manage_users: manage,
        role_code: role.to_string(),
    }
}

#[test]
fn contract_nav_admin_at_bankpay() {
    let model = NavModel::build(&flags(true, true, true, "ADMIN"), "/bankpay");
    insta::assert_debug_snapshot!("nav_admin_bankpay", model);
}

#[test]
fn contract_nav_viewer_at_home() {
    let model = NavModel::build(&flags(true, false, false, "VIEWER"), "/");
    insta::assert_debug_snapshot!("nav_viewer_home", model);
}

#[test]
fn contract_nav_norole_at_ledger() {
    let model = NavModel::build(&flags(true, false, false, ""), "/transactions");
    insta::assert_debug_snapshot!("nav_norole_ledger", model);
}
