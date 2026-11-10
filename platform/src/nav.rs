#![forbid(unsafe_code)]
//! 導覽模型（計畫 1793033568961 W2；grill-B 裁定：middleware 注入）。
//!
//! 單一事實來源＝`console_manifest`（`NAV_GROUPS`＋`PAGES`）；本模組依
//! `NavFlags` 過濾生成每請求的 [`NavModel`]——**無權限入口不渲染**（根治
//! 死連結：導覽可見性＝gate 過濾，而非渲染後擋下）。
//!
//! 計算位置：`auth::require_session` middleware 每認證請求一次
//! （NavFlags 查詢＋純記憶體過濾）；handler 一行從 Extension 複製。

use console_manifest::{Gate, NavKind, NAV_GROUPS, PAGES};

use crate::api::NavFlags;

/// 單一導覽項（來源＝`PageSpec`；金額/語彙零使用者輸入）。
#[derive(Debug, Clone)]
pub struct NavItem {
    pub id: &'static str,
    pub href: String,
    pub label: &'static str,
    /// 目前路徑比對命中時為 true（模板輸出 `aria-current="page"`）。
    pub active: bool,
}

/// 導覽群組檢視（`is_menu`＝`<details>` 下拉；否則頂層連結）。
#[derive(Debug, Clone)]
pub struct NavGroupView {
    pub id: &'static str,
    pub label: &'static str,
    pub is_menu: bool,
    pub items: Vec<NavItem>,
}

/// 完整導覽模型（每請求一份；Askama 模板直接迭代）。
#[derive(Debug, Clone, Default)]
pub struct NavModel {
    pub groups: Vec<NavGroupView>,
    /// 目前角色徽章（bootstrap admin 為空字串時不渲染）。
    pub role_code: String,
}

impl NavModel {
    /// 由 manifest＋權限旗標建構（無 I/O；除 `role_code` 複製外全為借用過濾）。
    pub fn build(flags: &NavFlags, current_path: &str) -> NavModel {
        let mut groups: Vec<NavGroupView> = Vec::new();
        for g in NAV_GROUPS {
            let mut items: Vec<NavItem> = Vec::new();
            for p in PAGES.iter().filter(|p| p.nav_group == Some(g.id)) {
                if !gate_allows(p.gate, flags) {
                    continue;
                }
                let href = console_manifest::nav_href_of(p).to_string();
                let active = is_active_path(current_path, &href);
                items.push(NavItem {
                    id: p.id,
                    href,
                    label: p.title,
                    active,
                });
            }
            // 群組內所有項目皆被過濾 → 整組不渲染（連空 dropdown 都不出現）。
            if items.is_empty() {
                continue;
            }
            groups.push(NavGroupView {
                id: g.id,
                label: g.label,
                is_menu: g.kind == NavKind::Menu,
                items,
            });
        }
        NavModel {
            groups,
            role_code: flags.role_code.clone(),
        }
    }
}

/// 頁面閘 → 權限旗標（fail-closed：Public 頁不入導覽）。
fn gate_allows(gate: Gate, flags: &NavFlags) -> bool {
    match gate {
        Gate::Public => false,
        Gate::All => true,
        Gate::Read => flags.ledger_read,
        Gate::Write => flags.ledger_write,
        Gate::Manage => flags.manage_users,
    }
}

/// aria-current 路徑比對：精確命中或路徑前綴（`/accounts` 命中 `/accounts/:id`）。
/// `href == "/"` 僅精確命中（否則所有頁都會命中首頁）。
fn is_active_path(current: &str, href: &str) -> bool {
    if href == "/" {
        return current == "/";
    }
    current == href || current.starts_with(&format!("{href}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags(read: bool, write: bool, manage: bool) -> NavFlags {
        NavFlags {
            ledger_read: read,
            ledger_write: write,
            manage_users: manage,
            role_code: "ADMIN".to_string(),
        }
    }

    #[test]
    fn visitor_flags_yield_minimal_nav() {
        // 全 false（未登入者不會走到此，但防禦性：零權限→僅 All 頁）。
        let m = NavModel::build(&flags(false, false, false), "/");
        let ids: Vec<&str> = m.groups.iter().map(|g| g.id).collect();
        assert!(ids.contains(&"overview"));
        assert!(ids.contains(&"query"));
        assert!(ids.contains(&"system"));
        assert!(!ids.contains(&"ledger"), "Read 頁不應對零權限者出現");
        assert!(!ids.contains(&"admin"));
        assert!(!ids.contains(&"governance"));
    }

    #[test]
    fn read_only_user_never_sees_audit_link() {
        // 根治死連結：store1（LEDGER_READ）不得見 /audit（Manage 閘）——
        // governance 群組唯一項目被過濾 → 整組不渲染（連空 dropdown 都不出現）。
        let m = NavModel::build(&flags(true, false, false), "/transactions");
        assert!(
            !m.groups.iter().any(|g| g.id == "governance"),
            "governance 群組對 LEDGER_READ-only 者不應渲染"
        );
        assert!(
            !m.groups
                .iter()
                .flat_map(|g| g.items.iter())
                .any(|it| it.id == "audit.page"),
            "稽核連結對 LEDGER_READ-only 者渲染＝死連結（B7a 回歸）"
        );
    }

    #[test]
    fn manage_user_sees_governance_and_admin() {
        let m = NavModel::build(&flags(true, true, true), "/audit");
        let gov = m.groups.iter().find(|g| g.id == "governance").expect("gov");
        assert!(gov
            .items
            .iter()
            .any(|it| it.id == "audit.page" && it.active));
        assert!(m.groups.iter().any(|g| g.id == "admin"));
        assert!(m.groups.iter().any(|g| g.id == "apihub"));
    }

    #[test]
    fn aria_current_exact_and_prefix() {
        let f = flags(true, false, false);
        assert!(NavModel::build(&f, "/transactions")
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .any(|it| it.id == "tx.list" && it.active));
        // 前綴邊界：/accounts/ACC-1 命中 /accounts；/accounts-foo 不命中。
        assert!(NavModel::build(&f, "/accounts/ACC-1")
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .any(|it| it.id == "accounts.list" && it.active));
        assert!(!NavModel::build(&f, "/accounts-foo")
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .any(|it| it.active));
        // 首頁僅精確命中。
        assert!(NavModel::build(&f, "/")
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .any(|it| it.id == "home.index" && it.active));
        assert!(!NavModel::build(&f, "/transactions")
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .any(|it| it.id == "home.index" && it.active));
    }

    #[test]
    fn knowledge_group_present_for_all_authenticated() {
        let m = NavModel::build(&flags(false, false, false), "/");
        let kb = m
            .groups
            .iter()
            .find(|g| g.id == "knowledge")
            .expect("knowledge");
        assert_eq!(kb.items.len(), 1);
        assert_eq!(kb.items[0].href, "/console/index.html");
    }
}
